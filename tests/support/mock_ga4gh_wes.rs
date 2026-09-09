// SPDX-License-Identifier: Apache-2.0
//! In-process WES fixture matching HelixTest `framework::wes` HTTP usage.
//! Catalog: [docs/FIXTURES.md](../../docs/FIXTURES.md). Not Ferrum.
//! GET `/ga4gh/wes/v1/service-info`, POST `/ga4gh/wes/v1/runs`, GET status, GET run.
//! Scatter/gather: HelixTest `trs://test-tool/scatter-gather/1.0` → COMPLETE + `scatter_result`
//! when a profile enables `supports_scatter_gather`. Generic profile skips before POST.
//!
//! Fixture URLs (from HelixTest, not invented):
//! - `trs://test-tool/echo/1.0` + CWL → COMPLETE, `outputs.echo_out` = `workflow_params.message`
//! - `trs://test-tool/fail/1.0` → EXECUTOR_ERROR
//! - `trs://test-tool/cwl-echo/1.0` (missing inputs or WDL) → EXECUTOR_ERROR
//! - `trs://nonexistent/invalid/0.0` → EXECUTOR_ERROR
//! - `trs://test-tool/scatter-gather/1.0` → COMPLETE, `outputs.scatter_result` present
//!
//! First status poll is RUNNING (not terminal). Later polls are terminal.
//! HelixTest requires a pre-terminal state and rejects a first-state terminal.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

pub struct MockGa4ghWes {
    pub server: MockServer,
}

impl MockGa4ghWes {
    pub fn origin(&self) -> String {
        self.server.uri()
    }

    pub fn wes_url(&self) -> String {
        format!("{}/ga4gh/wes/v1", self.server.uri())
    }
}

pub struct MockGa4ghDrsWes {
    pub server: MockServer,
}

impl MockGa4ghDrsWes {
    pub fn origin(&self) -> String {
        self.server.uri()
    }

    pub fn wes_url(&self) -> String {
        format!("{}/ga4gh/wes/v1", self.server.uri())
    }
}

struct RunRec {
    workflow_url: String,
    workflow_type: String,
    workflow_params: Value,
    polls: u32,
    outputs: Value,
}

struct WesInner {
    next_id: u64,
    runs: HashMap<String, RunRec>,
}

#[derive(Clone)]
struct WesStore {
    inner: Arc<Mutex<WesInner>>,
    lifecycle: WesLifecycleMut,
}

/// Lifecycle defect for mutation tests. Default is the honest fixture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WesLifecycleMut {
    #[default]
    Honest,
    /// First GetRunStatus for echo is already COMPLETE.
    EchoImmediateComplete,
    /// fail/1.0 ends COMPLETE.
    FailAsComplete,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WesInfoMut {
    #[default]
    Honest,
    Incomplete,
    VersionsOnly2,
    MalformedTypes,
    ContradictoryVersions,
    TypeVersion999,
}

/// `name` is a WES service-info field only. Helix must not switch profile from it.
fn wes_service_info_json_named(name: &str) -> Value {
    json!({
        "id": "org.example.helix-mock-wes",
        "name": name,
        "type": { "group": "org.ga4gh", "artifact": "wes", "version": "1.1.0" },
        "organization": { "name": "Synaptic Four", "url": "https://example.invalid/" },
        "version": "1.1.0",
        "workflow_type_versions": {
            "CWL": { "workflow_type_version": ["v1.2"] }
        },
        "supported_wes_versions": ["1.1"],
        "supported_filesystem_protocols": ["https"],
        "workflow_engine_versions": {
            "helix-mock": { "workflow_engine_version": ["1.0"] }
        },
        "default_workflow_engine_parameters": [],
        "system_state_counts": {},
        "auth_instructions_url": "https://example.invalid/auth",
        "tags": {}
    })
}

fn wes_service_info_mutated(name: &str, info: WesInfoMut) -> Value {
    match info {
        WesInfoMut::Honest => wes_service_info_json_named(name),
        WesInfoMut::Incomplete => json!({
            "name": "wes",
            "type": { "artifact": "wes", "version": "1.0.0" }
        }),
        WesInfoMut::VersionsOnly2 => {
            let mut v = wes_service_info_json_named(name);
            v["supported_wes_versions"] = json!(["2.0"]);
            v
        }
        WesInfoMut::MalformedTypes => json!({
            "id": true,
            "name": ["not", "a", "string"],
            "type": "wes",
            "organization": { "name": "Synaptic Four", "url": "https://example.invalid/" },
            "version": "1.1.0",
            "supported_wes_versions": ["1.1"],
        }),
        WesInfoMut::ContradictoryVersions => {
            let mut v = wes_service_info_json_named(name);
            v["type"]["version"] = json!("1.0.0");
            v["supported_wes_versions"] = json!(["1.1"]);
            v
        }
        WesInfoMut::TypeVersion999 => {
            let mut v = wes_service_info_json_named(name);
            v["type"]["version"] = json!("9.9.9");
            v
        }
    }
}

/// HelixTest `wes.rs` fixture table. Echo and scatter-gather COMPLETE; others EXECUTOR_ERROR.
fn terminal_for(run: &RunRec) -> (String, Value) {
    if run.workflow_url == "trs://test-tool/echo/1.0"
        && run.workflow_type.eq_ignore_ascii_case("CWL")
    {
        let echoed = run
            .workflow_params
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return ("COMPLETE".to_string(), json!({ "echo_out": echoed }));
    }
    if run.workflow_url == "trs://test-tool/scatter-gather/1.0" {
        return (
            "COMPLETE".to_string(),
            json!({ "scatter_result": run.workflow_params.get("items").cloned().unwrap_or(json!([])) }),
        );
    }
    ("EXECUTOR_ERROR".to_string(), json!({}))
}

fn run_id_from_path(url_path: &str, suffix: &str) -> Option<String> {
    let marker = "/runs/";
    let rest = url_path.split_once(marker)?.1;
    let id = if suffix.is_empty() {
        rest.trim_end_matches('/')
    } else {
        rest.strip_suffix(suffix)?.trim_end_matches('/')
    };
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

struct WesSubmit {
    store: WesStore,
}

impl wiremock::Respond for WesSubmit {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap_or_else(|_| json!({}));
        let workflow_url = body
            .get("workflow_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let workflow_type = body
            .get("workflow_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let workflow_params = body.get("workflow_params").cloned().unwrap_or(json!({}));
        let mut inner = self.store.inner.lock().expect("wes store");
        inner.next_id += 1;
        let run_id = format!("run-{}", inner.next_id);
        inner.runs.insert(
            run_id.clone(),
            RunRec {
                workflow_url,
                workflow_type,
                workflow_params,
                polls: 0,
                outputs: json!({}),
            },
        );
        ResponseTemplate::new(200).set_body_json(json!({ "run_id": run_id }))
    }
}

struct WesStatus {
    store: WesStore,
}

impl wiremock::Respond for WesStatus {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let path = request.url.path();
        let Some(run_id) = run_id_from_path(path, "/status") else {
            return ResponseTemplate::new(404);
        };
        let mut inner = self.store.inner.lock().expect("wes store");
        let Some(run) = inner.runs.get_mut(&run_id) else {
            return ResponseTemplate::new(404);
        };
        run.polls += 1;
        let is_echo = run.workflow_url == "trs://test-tool/echo/1.0"
            && run.workflow_type.eq_ignore_ascii_case("CWL");
        let is_fail = run.workflow_url == "trs://test-tool/fail/1.0";
        let state = match self.store.lifecycle {
            WesLifecycleMut::EchoImmediateComplete if is_echo => {
                let (_term, outputs) = terminal_for(run);
                run.outputs = outputs;
                "COMPLETE".to_string()
            }
            WesLifecycleMut::FailAsComplete if is_fail => {
                if run.polls == 1 {
                    "RUNNING".to_string()
                } else {
                    run.outputs = json!({});
                    "COMPLETE".to_string()
                }
            }
            _ => {
                if run.polls == 1 {
                    "RUNNING".to_string()
                } else {
                    let (term, outputs) = terminal_for(run);
                    run.outputs = outputs;
                    term
                }
            }
        };
        ResponseTemplate::new(200).set_body_json(json!({
            "run_id": run_id,
            "state": state
        }))
    }
}

struct WesRunGet {
    store: WesStore,
}

impl wiremock::Respond for WesRunGet {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let path = request.url.path();
        let Some(run_id) = run_id_from_path(path, "") else {
            return ResponseTemplate::new(404);
        };
        let inner = self.store.inner.lock().expect("wes store");
        let Some(run) = inner.runs.get(&run_id) else {
            return ResponseTemplate::new(404);
        };
        ResponseTemplate::new(200).set_body_json(json!({ "outputs": run.outputs }))
    }
}

pub async fn start_mock_ga4gh_wes() -> MockGa4ghWes {
    start_mock_ga4gh_wes_named("Helix in-process WES fixture").await
}

/// Same HTTP fixture with a chosen service-info `name` (e.g. “Ferrum Gateway”).
pub async fn start_mock_ga4gh_wes_named(name: &str) -> MockGa4ghWes {
    let server = MockServer::start().await;
    mount_ga4gh_wes_named(&server, name).await;
    MockGa4ghWes { server }
}

/// Incomplete service-info (DETECTED + TESTABLE, schema check fails).
pub async fn start_mock_wes_incomplete_service_info() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/wes/v1/service-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "wes",
            "type": { "artifact": "wes", "version": "1.0.0" }
        })))
        .mount(&server)
        .await;
    server
}

pub async fn mount_ga4gh_wes(server: &MockServer) {
    mount_ga4gh_wes_named(server, "Helix in-process WES fixture").await;
}

pub async fn mount_ga4gh_wes_named(server: &MockServer, name: &str) {
    mount_ga4gh_wes_mutated(
        server,
        name,
        WesInfoMut::Honest,
        WesLifecycleMut::Honest,
        false,
    )
    .await;
}

/// WES fixture with one controlled defect. Runs stay mounted so other checks can pass.
pub async fn mount_ga4gh_wes_mutated(
    server: &MockServer,
    name: &str,
    info: WesInfoMut,
    lifecycle: WesLifecycleMut,
    list_runs_broken: bool,
) {
    let store = WesStore {
        inner: Arc::new(Mutex::new(WesInner {
            next_id: 0,
            runs: HashMap::new(),
        })),
        lifecycle,
    };

    Mock::given(method("GET"))
        .and(path("/ga4gh/wes/v1/service-info"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(wes_service_info_mutated(name, info)),
        )
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/ga4gh/wes/v1/runs"))
        .respond_with(WesSubmit {
            store: store.clone(),
        })
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/ga4gh/wes/v1/runs/[^/]+/status$"))
        .respond_with(WesStatus {
            store: store.clone(),
        })
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/ga4gh/wes/v1/runs/[^/]+$"))
        .respond_with(WesRunGet { store })
        .mount(server)
        .await;

    if list_runs_broken {
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/runs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "runs": "not-an-array",
                "next_page_token": 1
            })))
            .mount(server)
            .await;
    }
}

pub async fn start_mock_ga4gh_drs_and_wes() -> MockGa4ghDrsWes {
    let server = MockServer::start().await;
    super::mock_ga4gh_drs::mount_ga4gh_drs(&server).await;
    mount_ga4gh_wes(&server).await;
    MockGa4ghDrsWes { server }
}

pub async fn start_mock_ga4gh_drs_and_wes_mutated(
    info: WesInfoMut,
    lifecycle: WesLifecycleMut,
    list_runs_broken: bool,
) -> MockGa4ghDrsWes {
    let server = MockServer::start().await;
    super::mock_ga4gh_drs::mount_ga4gh_drs(&server).await;
    mount_ga4gh_wes_mutated(
        &server,
        "Helix in-process WES fixture",
        info,
        lifecycle,
        list_runs_broken,
    )
    .await;
    MockGa4ghDrsWes { server }
}
