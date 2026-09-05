// SPDX-License-Identifier: Apache-2.0
//! Same in-process DRS fixture as HelixTest B1 (`HelixTest/helixtest/testing/mock_ga4gh_drs.rs`).
//! Duplicated here so Helix CI compiles against a published HelixTest pin without that file.
//! Not Ferrum. Catalog: [docs/FIXTURES.md](../../docs/FIXTURES.md).
//!
//! Valid object: id `test-object-1`; blob 4096 × `'A'`; checksum type `sha256`.
//! Invalid object: `{ "id": "test-object-1" }` only (DETECTED, checks fail).
//!
//! Unlike HelixTest B1, this copy does **not** mount a WES-shaped `/service-info`.
//! HelixTest uses that path as a Ferrum-name trap; Helix adapter already uses
//! `Mode::Generic`. Helix discovery would treat that JSON as WES DETECTED+TESTABLE.

use common::util::sha256_bytes;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

pub const TEST_OBJECT_ID: &str = "test-object-1";
/// Derived unknown id for the default catalog object. Not a global hard-coded string.
pub fn unknown_object_id() -> String {
    framework::drs::unknown_object_id_for(TEST_OBJECT_ID)
}
pub const BLOB_LEN: usize = 4096;

pub struct MockGa4ghDrs {
    pub server: MockServer,
}

impl MockGa4ghDrs {
    pub fn drs_url(&self) -> String {
        self.server.uri()
    }
}

struct BytesWithOptionalRange {
    body: Vec<u8>,
}

impl wiremock::Respond for BytesWithOptionalRange {
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

pub async fn start_mock_ga4gh_drs() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    mount_ga4gh_drs(&server).await;
    MockGa4ghDrs { server }
}

/// Gateway-prefixed DRS service-info so discovery can DETECT DRS without the
/// configured object existing. Not a WES `/service-info`. Not certification.
pub async fn mount_ga4gh_drs_service_info(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/service-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "org.ga4gh.drs",
            "name": "Mock DRS",
            "version": "0.0.0",
            "type": { "group": "org.ga4gh", "artifact": "drs", "version": "1.4.0" }
        })))
        .mount(server)
        .await;
}

/// DRS object/bytes routes only. No `/service-info` (that is the WES split probe).
pub async fn mount_ga4gh_drs(server: &MockServer) {
    let blob = vec![b'A'; BLOB_LEN];
    let sha256 = sha256_bytes(&blob);
    let access_url = format!("{}/bytes/{TEST_OBJECT_ID}", server.uri());
    let object = json!({
        "id": TEST_OBJECT_ID,
        "name": TEST_OBJECT_ID,
        "self_uri": format!("drs://example.invalid/{TEST_OBJECT_ID}"),
        "size": BLOB_LEN,
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": sha256 }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": access_url }
        }]
    });

    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(BytesWithOptionalRange { body: blob })
        .mount(server)
        .await;
}

/// Intentionally invalid DRS: object JSON is `{ "id": "test-object-1" }` only.
/// Discovery DETECTED + TESTABLE; HelixTest schema/checksum/range/bytes fail.
pub async fn start_mock_invalid_drs_object() -> MockServer {
    let server = MockServer::start().await;
    mount_invalid_drs_object(&server).await;
    server
}

pub async fn mount_invalid_drs_object(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": TEST_OBJECT_ID })))
        .mount(server)
        .await;
}

pub fn valid_drs_object_json(server_uri: &str) -> serde_json::Value {
    let blob = vec![b'A'; BLOB_LEN];
    drs_object_json(server_uri, &sha256_bytes(&blob))
}

pub fn honest_blob() -> Vec<u8> {
    vec![b'A'; BLOB_LEN]
}

fn drs_object_json(server_uri: &str, sha256: &str) -> serde_json::Value {
    let access_url = format!("{server_uri}/bytes/{TEST_OBJECT_ID}");
    json!({
        "id": TEST_OBJECT_ID,
        "name": TEST_OBJECT_ID,
        "self_uri": format!("drs://example.invalid/{TEST_OBJECT_ID}"),
        "size": BLOB_LEN,
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": sha256 }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": access_url }
        }]
    })
}

/// Schema-valid DrsObject (HelixTest extras included) whose bytes do not match checksums.
/// SCHEMA can PASS while BEHAVIOR (checksum) FAILS.
pub async fn start_mock_schema_ok_checksum_wrong() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let claimed = vec![b'A'; BLOB_LEN];
    let served = vec![b'B'; BLOB_LEN];
    let sha256 = sha256_bytes(&claimed);
    let object = drs_object_json(&server.uri(), &sha256);

    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(BytesWithOptionalRange { body: served })
        .mount(&server)
        .await;

    MockGa4ghDrs { server }
}

/// Schema-valid fixture object, but the unknown-id probe returns HTTP 200.
/// SCHEMA can PASS while BEHAVIOR (404) FAILS.
pub async fn start_mock_schema_ok_unknown_id_200() -> MockGa4ghDrs {
    let server = MockServer::start().await;
    let blob = vec![b'A'; BLOB_LEN];
    let sha256 = sha256_bytes(&blob);
    let object = drs_object_json(&server.uri(), &sha256);
    let unknown_id = unknown_object_id();
    let unknown = json!({
        "id": unknown_id,
        "name": unknown_id,
        "self_uri": format!("drs://example.invalid/{unknown_id}"),
        "size": BLOB_LEN,
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": sha256 }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": format!("{}/bytes/{TEST_OBJECT_ID}", server.uri()) }
        }]
    });

    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/objects/{}", unknown_object_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(unknown))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/bytes/{TEST_OBJECT_ID}")))
        .respond_with(BytesWithOptionalRange { body: blob })
        .mount(&server)
        .await;

    MockGa4ghDrs { server }
}
