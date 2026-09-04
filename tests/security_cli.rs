// SPDX-License-Identifier: Apache-2.0
//! `helix security` against an in-process auth-gated DRS (dummy HMAC, not production).

use assert_cmd::Command;
use helix::security::classify_bearer;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SECRET: &str = "helix-dummy-hmac-not-for-production-do-not-use";

struct AuthGate;

impl wiremock::Respond for AuthGate {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let header = request
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = header.strip_prefix("Bearer ").unwrap_or("");
        let code = classify_bearer(token, SECRET, "drs", "drs.read");
        ResponseTemplate::new(code)
    }
}

#[tokio::test]
async fn helix_security_cli_passes_on_dummy_hmac_mock() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/objects/test-object-1"))
        .respond_with(AuthGate)
        .mount(&server)
        .await;

    let secret = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test-fixtures/hmac/shared-secret.txt");

    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args([
            "security",
            &server.uri(),
            "--format",
            "json",
            "--hmac-secret-file",
            secret.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Security: valid token grants access",
        ))
        .stdout(predicate::str::contains("Crypt4GH header structure"));
}
