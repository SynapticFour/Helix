// SPDX-License-Identifier: Apache-2.0
//! One-defect HTTP fixtures for the mutation corpus. Local wiremock only.
//! Catalog: helix::mutation::CATALOG / docs/MUTATION.md. Not a pentest.

use helix::http_safety::HTTP_REQUEST_TIMEOUT_SECS;
use helix::security::VerifierPolicy;
use serde_json::{json, Value};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use super::mock_ga4gh_drs::{
    honest_blob, unknown_object_id, valid_drs_object_json, TEST_OBJECT_ID,
};
use super::mock_ga4gh_wes::{
    start_mock_ga4gh_drs_and_wes_mutated, MockGa4ghDrsWes, WesInfoMut, WesLifecycleMut,
};

pub enum MutationTarget {
    Server(MockServer),
    DrsWes(MockGa4ghDrsWes),
}

impl MutationTarget {
    pub fn url(&self) -> String {
        match self {
            Self::Server(s) => s.uri(),
            Self::DrsWes(m) => m.origin(),
        }
    }
}

struct BytesIgnoreRange {
    body: Vec<u8>,
}

impl wiremock::Respond for BytesIgnoreRange {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(self.body.clone())
    }
}

struct Bytes206NoContentRange {
    body: Vec<u8>,
}

impl wiremock::Respond for Bytes206NoContentRange {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let range = request
            .headers
            .get("range")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if range.starts_with("bytes=") {
            let slice = self.body.get(..1024).unwrap_or(&self.body).to_vec();
            return ResponseTemplate::new(206)
                .insert_header("Content-Type", "application/octet-stream")
                .set_body_bytes(slice);
        }
        ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(self.body.clone())
    }
}

struct HonestRange {
    body: Vec<u8>,
}

impl wiremock::Respond for HonestRange {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let total = self.body.len() as u64;
        let range_hdr = request
            .headers
            .get("range")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if let Some(spec) = range_hdr.strip_prefix("bytes=") {
            let (start_s, end_s) = spec.split_once('-').unwrap_or(("0", ""));
            let start: usize = start_s.parse().unwrap_or(0);
            let end: usize = if end_s.is_empty() {
                self.body.len().saturating_sub(1)
            } else {
                end_s.parse().unwrap_or(self.body.len().saturating_sub(1))
            };
            let end = end.min(self.body.len().saturating_sub(1));
            let start = start.min(end);
            let slice = self.body[start..=end].to_vec();
            return ResponseTemplate::new(206)
                .insert_header("Content-Range", format!("bytes {start}-{end}/{total}"))
                .insert_header("Content-Type", "application/octet-stream")
                .set_body_bytes(slice);
        }
        ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(self.body.clone())
    }
}

async fn mount_unknown(server: &MockServer, status: u16, body: Value) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(server)
        .await;
}

async fn mount_unknown_text(server: &MockServer, status: u16, body: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(status).set_body_string(body))
        .mount(server)
        .await;
}

async fn mount_honest_unknown_and_bytes(server: &MockServer) {
    mount_unknown_text(server, 404, "not found").await;
    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(HonestRange {
            body: honest_blob(),
        })
        .mount(server)
        .await;
}

pub async fn start_mutant(id: &str) -> MutationTarget {
    match id {
        "HLX-MUT-001" => {
            let server = MockServer::start().await;
            let mut object = valid_drs_object_json(&server.uri());
            object.as_object_mut().unwrap().remove("self_uri");
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-002" => {
            let server = MockServer::start().await;
            let mut object = valid_drs_object_json(&server.uri());
            object["size"] = json!("4096");
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-003" => {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-004" => {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-005" => {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("Content-Type", "text/html")
                        .set_body_string("<html><body>not json</body></html>"),
                )
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-006" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("Content-Type", "text/plain")
                        .set_body_string(object.to_string()),
                )
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-007" => {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("Content-Type", "application/json")
                        .set_body_string(format!(r#"{{"id":"{TEST_OBJECT_ID}""#)),
                )
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-008" => {
            let server = MockServer::start().await;
            let mut object = valid_drs_object_json(&server.uri());
            object["id"] = json!("not-the-fixture-id");
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-009" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/objects"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "objects": "not-an-array",
                    "next_page_token": "wrong"
                })))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-010" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object.clone()))
                .mount(&server)
                .await;
            let mut unknown = object;
            unknown["id"] = json!(unknown_object_id());
            unknown["name"] = json!(unknown_object_id());
            mount_unknown(&server, 200, unknown).await;
            Mock::given(method("GET"))
                .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
                .respond_with(HonestRange {
                    body: honest_blob(),
                })
                .mount(&server)
                .await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-011" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_unknown_text(&server, 500, "internal").await;
            Mock::given(method("GET"))
                .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
                .respond_with(HonestRange {
                    body: honest_blob(),
                })
                .mount(&server)
                .await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-012" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::Honest,
                WesLifecycleMut::EchoImmediateComplete,
                false,
            )
            .await,
        ),
        "HLX-MUT-013" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::Honest,
                WesLifecycleMut::FailAsComplete,
                false,
            )
            .await,
        ),
        "HLX-MUT-014" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::VersionsOnly2,
                WesLifecycleMut::Honest,
                false,
            )
            .await,
        ),
        "HLX-MUT-015" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::TypeVersion999,
                WesLifecycleMut::Honest,
                false,
            )
            .await,
        ),
        "HLX-MUT-016" | "HLX-MUT-017" => {
            let server = MockServer::start().await;
            let policy = if id == "HLX-MUT-016" {
                VerifierPolicy::ignore_expiry()
            } else {
                VerifierPolicy::reject_all()
            };
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(AuthGate { policy })
                .mount(&server)
                .await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-018" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_unknown_text(&server, 404, "not found").await;
            Mock::given(method("GET"))
                .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
                .respond_with(BytesIgnoreRange {
                    body: honest_blob(),
                })
                .mount(&server)
                .await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-019" => {
            let server = MockServer::start().await;
            let object = valid_drs_object_json(&server.uri());
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_unknown_text(&server, 404, "not found").await;
            Mock::given(method("GET"))
                .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
                .respond_with(Bytes206NoContentRange {
                    body: honest_blob(),
                })
                .mount(&server)
                .await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-020" => {
            let server = MockServer::start().await;
            let mut object = valid_drs_object_json(&server.uri());
            object["unexpected_helix_mutant"] = json!(true);
            Mock::given(method("GET"))
                .and(path(format!("/objects/{TEST_OBJECT_ID}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(object))
                .mount(&server)
                .await;
            mount_honest_unknown_and_bytes(&server).await;
            MutationTarget::Server(server)
        }
        "HLX-MUT-021" => {
            let server = MockServer::start().await;
            let delay = Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECS + 1);
            for p in [
                "/ga4gh/drs/v1/objects/test-object-1",
                "/objects/test-object-1",
            ] {
                Mock::given(method("GET"))
                    .and(path(p))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_delay(delay)
                            .set_body_string("{}"),
                    )
                    .mount(&server)
                    .await;
            }
            MutationTarget::Server(server)
        }
        "HLX-MUT-022" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::MalformedTypes,
                WesLifecycleMut::Honest,
                false,
            )
            .await,
        ),
        "HLX-MUT-023" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(
                WesInfoMut::ContradictoryVersions,
                WesLifecycleMut::Honest,
                false,
            )
            .await,
        ),
        "HLX-MUT-024" => MutationTarget::DrsWes(
            start_mock_ga4gh_drs_and_wes_mutated(WesInfoMut::Honest, WesLifecycleMut::Honest, true)
                .await,
        ),
        other => panic!("no fixture for {other}"),
    }
}

struct AuthGate {
    policy: VerifierPolicy,
}

impl wiremock::Respond for AuthGate {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let header = request
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = header.strip_prefix("Bearer ").unwrap_or("");
        let code = helix::security::classify_bearer_with(
            token,
            "helix-dummy-hmac-not-for-production-do-not-use",
            "drs",
            "drs.read",
            self.policy,
        );
        ResponseTemplate::new(code)
    }
}
