// SPDX-License-Identifier: Apache-2.0
//! `helix security` against an in-process auth-gated DRS (dummy HMAC, not production).
//! Not a security audit. Passing does not prove the implementation is secure.

use assert_cmd::Command;
use helix::security::{classify_bearer_with, VerifierPolicy, SECURITY_BEHAVIOR_DISCLAIMER};
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SECRET: &str = "helix-dummy-hmac-not-for-production-do-not-use";

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
        let code = classify_bearer_with(token, SECRET, "drs", "drs.read", self.policy);
        ResponseTemplate::new(code)
    }
}

async fn start_mock(policy: VerifierPolicy) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/objects/test-object-1"))
        .respond_with(AuthGate { policy })
        .mount(&server)
        .await;
    server
}

fn secret_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/hmac/shared-secret.txt")
}

#[tokio::test]
async fn helix_security_cli_passes_on_dummy_hmac_mock() {
    let server = start_mock(VerifierPolicy::fail_closed()).await;
    let secret = secret_path();

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
        .stdout(predicate::str::contains("Crypt4GH header structure"))
        .stdout(predicate::str::contains(SECRET).not())
        .stdout(predicate::str::contains("eyJ").not())
        .stdout(predicate::str::contains("Authorization: Bearer").not());
}

#[tokio::test]
async fn helix_security_text_prints_behavior_disclaimer() {
    let server = start_mock(VerifierPolicy::fail_closed()).await;
    let secret = secret_path();

    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args([
            "security",
            &server.uri(),
            "--hmac-secret-file",
            secret.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(SECURITY_BEHAVIOR_DISCLAIMER))
        .stdout(predicate::str::contains(
            "does not prove the implementation is secure",
        ))
        .stdout(predicate::str::contains("penetration test"))
        .stdout(predicate::str::contains(
            "Crypt4GH (protocol layout only; not encryption, not secure)",
        ))
        .stdout(predicate::str::contains(SECRET).not())
        .stdout(predicate::str::contains("eyJ").not());
}

#[tokio::test]
async fn helix_security_detects_broken_mock_that_ignores_expiry() {
    let server = start_mock(VerifierPolicy::ignore_expiry()).await;
    let secret = secret_path();

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
        .failure()
        .code(1)
        .stdout(predicate::str::contains(
            "Security: expired token rejected with 401",
        ))
        .stdout(predicate::str::contains("HLX-AUTH-011"))
        .stdout(predicate::str::contains(SECRET).not())
        .stdout(predicate::str::contains("eyJ").not())
        .stdout(predicate::str::contains("Authorization: Bearer").not());
}
