// SPDX-License-Identifier: Apache-2.0
//! Controlled DRS HTTP mutations for B10. Changes are on the target, not Helix.
//! Localhost wiremock only. No reverse proxy, no credential logging, no live fork.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use common::util::sha256_bytes;
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use super::mock_ga4gh_drs::{
    unknown_object_id, valid_drs_object_json, MockGa4ghDrs, BLOB_LEN, TEST_OBJECT_ID,
};

/// Path Helix DRS catalog checks never request.
pub const OUTSIDE_CLOSURE_PATH: &str = "/internal/not-in-drs-catalog";

pub struct RangeProbe {
    pub mock: MockGa4ghDrs,
    pub range_header_seen: Arc<AtomicBool>,
}

impl RangeProbe {
    pub fn drs_url(&self) -> String {
        self.mock.drs_url()
    }

    pub fn range_was_sent(&self) -> bool {
        self.range_header_seen.load(Ordering::SeqCst)
    }
}

struct BytesWithOptionalRange {
    body: Vec<u8>,
}

impl wiremock::Respond for BytesWithOptionalRange {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        honor_range(&self.body, request)
    }
}

struct BytesIgnoringRange {
    body: Vec<u8>,
    range_seen: Arc<AtomicBool>,
}

impl wiremock::Respond for BytesIgnoringRange {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        if request.headers.get("range").is_some() {
            self.range_seen.store(true, Ordering::SeqCst);
        }
        ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(self.body.clone())
    }
}

fn honor_range(body: &[u8], request: &Request) -> ResponseTemplate {
    let total = body.len() as u64;
    let range_hdr = request
        .headers
        .get("range")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if let Some(spec) = range_hdr.strip_prefix("bytes=") {
        let (start_s, end_s) = spec.split_once('-').unwrap_or(("0", ""));
        let start: usize = start_s.parse().unwrap_or(0);
        let end: usize = if end_s.is_empty() {
            body.len().saturating_sub(1)
        } else {
            end_s.parse().unwrap_or(body.len().saturating_sub(1))
        };
        let end = end.min(body.len().saturating_sub(1));
        let start = start.min(end);
        let slice = body[start..=end].to_vec();
        return ResponseTemplate::new(206)
            .insert_header("Content-Range", format!("bytes {start}-{end}/{total}"))
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(slice);
    }
    ResponseTemplate::new(200)
        .insert_header("Content-Type", "application/octet-stream")
        .set_body_bytes(body.to_vec())
}

async fn mount_unknown_404(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(server)
        .await;
}

async fn mount_bytes_honor(server: &MockServer, blob: Vec<u8>) {
    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(BytesWithOptionalRange { body: blob })
        .mount(server)
        .await;
}

async fn mount_object_json(server: &MockServer, object: Value) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(server)
        .await;
}

async fn mount_object_raw(server: &MockServer, body: String) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_string(body),
        )
        .mount(server)
        .await;
}

/// M1: omit `access_methods`. Pinned OpenAPI does not require the field.
pub async fn start_m1_missing_access_methods() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let mut object = valid_drs_object_json(&server.uri());
    object
        .as_object_mut()
        .expect("object")
        .remove("access_methods");
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// M2: `size` is a JSON string. Required integer in pinned DrsObject.yaml.
pub async fn start_m2_size_is_string() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let mut object = valid_drs_object_json(&server.uri());
    object["size"] = json!("4096");
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// M3: advertised sha256 does not match unchanged bytes.
pub async fn start_m3_advertised_checksum_lie() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let mut object = valid_drs_object_json(&server.uri());
    object["checksums"][0]["checksum"] = json!("0".repeat(64));
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// M4: advertised checksum of 4096×A; bytes are 4096×B.
pub async fn start_m4_byte_content_lie() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let object = valid_drs_object_json(&server.uri());
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'B'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// M5: Range request returns HTTP 200 full body. Records that Range was sent.
pub async fn start_m5_range_returns_200() -> RangeProbe {
    let server = MockServer::start().await;
    let blob = vec![b'A'; BLOB_LEN];
    let object = valid_drs_object_json(&server.uri());
    let range_seen = Arc::new(AtomicBool::new(false));
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(BytesIgnoringRange {
            body: blob,
            range_seen: Arc::clone(&range_seen),
        })
        .mount(&server)
        .await;
    RangeProbe {
        mock: MockGa4ghDrs { server },
        range_header_seen: range_seen,
    }
}

/// M6: derived unknown object id returns 200 instead of 404.
pub async fn start_m6_unknown_object_200() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let blob = vec![b'A'; BLOB_LEN];
    let object = valid_drs_object_json(&server.uri());
    let unknown_id = unknown_object_id();
    let unknown = json!({
        "id": unknown_id,
        "name": unknown_id,
        "self_uri": format!("drs://example.invalid/{unknown_id}"),
        "size": BLOB_LEN,
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": sha256_bytes(&blob) }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": format!("{}/bytes/{TEST_OBJECT_ID}", server.uri()) }
        }]
    });
    mount_object_json(&server, object).await;
    Mock::given(method("GET"))
        .and(path(format!("/objects/{unknown_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(unknown))
        .mount(&server)
        .await;
    mount_bytes_honor(&server, blob).await;
    MockGa4ghDrs { server }
}

/// P1: same DrsObject, JSON object keys reordered. Not a semantic change.
pub async fn start_p1_key_reorder() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let blob = vec![b'A'; BLOB_LEN];
    let sha = sha256_bytes(&blob);
    let uri = server.uri();
    let body = format!(
        r#"{{"access_methods":[{{"access_url":{{"url":"{uri}/bytes/{TEST_OBJECT_ID}"}},"type":"https"}}],"checksums":[{{"checksum":"{sha}","type":"sha256"}}],"created_time":"2020-01-01T00:00:00Z","id":"{TEST_OBJECT_ID}","name":"{TEST_OBJECT_ID}","self_uri":"drs://example.invalid/{TEST_OBJECT_ID}","size":{BLOB_LEN}}}"#
    );
    mount_object_raw(&server, body).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, blob).await;
    MockGa4ghDrs { server }
}

/// P2: same DrsObject, extra JSON whitespace.
pub async fn start_p2_whitespace() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let object = valid_drs_object_json(&server.uri());
    let pretty = serde_json::to_string_pretty(&object).expect("pretty");
    let body = format!("\n\n{pretty}\n\n");
    mount_object_raw(&server, body).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// P3: schema-allowed optional `description`.
pub async fn start_p3_description() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let mut object = valid_drs_object_json(&server.uri());
    object["description"] = json!("benign optional metadata");
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// Harness self-check: `mime_type` is unused by HLX-DRS-004.
pub async fn start_harness_ineffective_mime_type() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let mut object = valid_drs_object_json(&server.uri());
    object["mime_type"] = json!("application/octet-stream");
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

async fn mount_internal(server: &MockServer, status: u16) {
    Mock::given(method("GET"))
        .and(path(OUTSIDE_CLOSURE_PATH))
        .respond_with(ResponseTemplate::new(status).set_body_string("internal"))
        .mount(server)
        .await;
}

/// Golden catalog object plus an unused internal path (outside-closure baseline).
/// Does not mount `/ga4gh/drs/v1/service-info`: that would move object GETs
/// onto the gateway prefix and is a discovery-shape change, not an unused path.
pub async fn start_golden_with_outside_surface() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    mount_internal(&server, 200).await;
    let object = valid_drs_object_json(&server.uri());
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

/// Outside closure: unused `/internal/not-in-drs-catalog` returns 500 instead of 200.
/// DRS catalog checks never request that path.
pub async fn start_outside_closure() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    mount_internal(&server, 500).await;
    let object = valid_drs_object_json(&server.uri());
    mount_object_json(&server, object).await;
    mount_unknown_404(&server).await;
    mount_bytes_honor(&server, vec![b'A'; BLOB_LEN]).await;
    MockGa4ghDrs { server }
}

pub fn honest_sha256() -> String {
    sha256_bytes(&vec![b'A'; BLOB_LEN])
}

pub fn pinned_size_string_object() -> Value {
    let mut object = json!({
        "id": TEST_OBJECT_ID,
        "name": TEST_OBJECT_ID,
        "self_uri": format!("drs://example.invalid/{TEST_OBJECT_ID}"),
        "size": "4096",
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": honest_sha256() }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": "http://127.0.0.1:9/bytes/test-object-1" }
        }]
    });
    object["size"] = json!("4096");
    object
}
