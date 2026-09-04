// SPDX-License-Identifier: Apache-2.0
//! Local adversarial HTTP fixtures. Not a pentest, not a security product, not HELIOS.
//!
//! Every server is in-process (wiremock or a localhost TCP closer). Nothing here
//! is sent at a real-world target. Catalog: [docs/FIXTURES.md](../../docs/FIXTURES.md) §16.

use helix::http_safety::MAX_RESPONSE_BYTES;
use std::net::TcpListener;
use std::thread::JoinHandle;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::mock_ga4gh_drs::TEST_OBJECT_ID;

/// JWT-shaped decoy that fixtures may put in bodies/headers. Must never appear in Helix output.
pub const ADVERSARIAL_JWT: &str =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

/// Password embedded only in a redirect `Location` userinfo. Must never be echoed.
pub const ADVERSARIAL_USERINFO: &str = "s3cret-adversarial-fixture";

/// Deterministic oversize field. Under the 2 MiB Helix-owned body cap.
pub const LONG_STRING_LEN: usize = 32_768;

/// Delay just beyond [`helix::http_safety::HTTP_REQUEST_TIMEOUT_SECS`].
pub const SLOW_DELAY: Duration = Duration::from_secs(6);

const DRS_PROBES: &[&str] = &[
    "/ga4gh/drs/v1/objects/test-object-1",
    "/ga4gh/drs/v1/service-info",
    "/objects/test-object-1",
];

pub struct ResetOrigin {
    pub url: String,
    _accept: JoinHandle<()>,
}

/// 200 with truncated JSON (and a decoy Bearer) on the split DRS object path.
pub async fn start_malformed_json() -> MockServer {
    let server = MockServer::start().await;
    let body =
        format!(r#"{{ "id": "{TEST_OBJECT_ID}", "authorization": "Bearer {ADVERSARIAL_JWT}""#);
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_string(body),
        )
        .mount(&server)
        .await;
    server
}

/// Body larger than the Helix-owned read cap on every DRS probe path.
pub async fn start_huge_json() -> MockServer {
    let server = MockServer::start().await;
    let mut body = Vec::with_capacity(MAX_RESPONSE_BYTES + 16);
    body.extend_from_slice(br#"{"id":""#);
    body.resize(MAX_RESPONSE_BYTES + 1, b'A');
    body.extend_from_slice(br#""}"#);
    for p in DRS_PROBES {
        Mock::given(method("GET"))
            .and(path(*p))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "application/json")
                    .set_body_bytes(body.clone()),
            )
            .mount(&server)
            .await;
    }
    server
}

/// 200 incomplete DrsObject plus non-JSON Content-Type and a Bearer in WWW-Authenticate.
pub async fn start_invalid_headers() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Content-Type",
                    "application/json; charset=utf-8; boundary=;;;;",
                )
                .insert_header("WWW-Authenticate", format!("Bearer {ADVERSARIAL_JWT}"))
                .insert_header("X-Malformed", "::::not-a-token")
                .set_body_string(format!(r#"{{"id":"{TEST_OBJECT_ID}"}}"#)),
        )
        .mount(&server)
        .await;
    server
}

/// 302 on every DRS probe to a hidden 200. Helix-owned client must not follow.
pub async fn start_redirect() -> MockServer {
    let server = MockServer::start().await;
    let hidden = format!("{}/hidden/objects/{TEST_OBJECT_ID}", server.uri());
    let bait = format!("http://alice:{ADVERSARIAL_USERINFO}@127.0.0.1:9/objects/{TEST_OBJECT_ID}");
    for p in DRS_PROBES {
        let location = if *p == "/ga4gh/drs/v1/objects/test-object-1" {
            bait.as_str()
        } else {
            hidden.as_str()
        };
        Mock::given(method("GET"))
            .and(path(*p))
            .respond_with(ResponseTemplate::new(302).insert_header("Location", location))
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path(format!("/hidden/objects/{TEST_OBJECT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_json(serde_json::json!({
                    "id": TEST_OBJECT_ID,
                    "name": TEST_OBJECT_ID,
                    "self_uri": format!("drs://example.invalid/{TEST_OBJECT_ID}"),
                    "size": 1,
                    "created_time": "2020-01-01T00:00:00Z",
                    "checksums": [{ "type": "sha256", "checksum": "00" }],
                    "access_methods": [{ "type": "https", "access_url": { "url": "http://127.0.0.1:9/x" } }]
                })),
        )
        .expect(0)
        .mount(&server)
        .await;
    server
}

/// First DRS probe waits longer than the Helix-owned request timeout.
pub async fn start_slow_response() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/objects/test-object-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(SLOW_DELAY)
                .set_body_string("{}"),
        )
        .mount(&server)
        .await;
    server
}

/// Accepts TCP then closes without an HTTP response (peer reset / unexpected EOF).
pub fn start_connection_reset() -> ResetOrigin {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind adversarial closer");
    let addr = listener.local_addr().expect("addr");
    let url = format!("http://{addr}");
    let accept = std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => drop(s),
                Err(_) => break,
            }
        }
    });
    ResetOrigin {
        url,
        _accept: accept,
    }
}

/// `text/html` body that is not a DrsObject, with a decoy Authorization line.
pub async fn start_invalid_content_type() -> MockServer {
    let server = MockServer::start().await;
    let html = format!("<html><body>Authorization: Bearer {ADVERSARIAL_JWT}</body></html>");
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/html")
                .set_body_string(html),
        )
        .mount(&server)
        .await;
    server
}

/// 418 on every DRS probe (not 2xx / 401 / 403, so not DETECTED).
pub async fn start_unexpected_status() -> MockServer {
    let server = MockServer::start().await;
    for p in DRS_PROBES {
        Mock::given(method("GET"))
            .and(path(*p))
            .respond_with(ResponseTemplate::new(418).set_body_string("teapot"))
            .mount(&server)
            .await;
    }
    server
}

/// Gateway DRS service-info is 200 JSON with the wrong shape (not a ServiceInfo object).
pub async fn start_malformed_service_info() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/objects/test-object-1"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/service-info"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_string(format!(
                    r#"{{"id":true,"name":["not","a","string"],"type":"drs","version":{{"nested":true}},"note":"Bearer {ADVERSARIAL_JWT}"}}"#
                )),
        )
        .mount(&server)
        .await;
    server
}

/// DrsObject-shaped JSON whose `name` is tens of kilobytes; required fields still missing.
pub async fn start_extremely_long_strings() -> MockServer {
    let server = MockServer::start().await;
    let name = "A".repeat(LONG_STRING_LEN);
    let body = serde_json::json!({
        "id": TEST_OBJECT_ID,
        "name": name,
    });
    Mock::given(method("GET"))
        .and(path(format!("/objects/{TEST_OBJECT_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}
