// SPDX-License-Identifier: Apache-2.0
//! Regression tests for Helix-as-a-client mitigations (`docs/THREAT_MODEL.md`).
//! Helix is not a security product. These tests lock leak prevention and HTTP limits.

use assert_cmd::Command;
use helix::http_safety::{MAX_COMPARE_FILE_BYTES, MAX_SECRET_FILE_BYTES};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SECRET: &str = "helix-dummy-hmac-not-for-production-do-not-use";
const JWT: &str =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/threat-model");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{}-{}", name, std::process::id()))
}

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

#[test]
fn verify_rejects_url_userinfo_without_echoing_the_password() {
    let assert = helix()
        .env("RUST_LOG", "error")
        .args([
            "verify",
            "http://alice:s3cret@127.0.0.1:1/",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stdout.trim().is_empty(), "no JSON on parse error: {stdout}");
    assert!(
        stderr.contains("userinfo"),
        "expected userinfo rejection: {stderr}"
    );
    assert!(!stderr.contains("s3cret"), "{stderr}");
    assert!(!stdout.contains("s3cret"), "{stdout}");
    assert!(!stderr.contains("alice:s3cret"), "{stderr}");
}

struct ReflectAuth;

impl wiremock::Respond for ReflectAuth {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let header = request
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        ResponseTemplate::new(401).set_body_string(format!(
            "Authorization: {header}\nsecret={SECRET}\ntoken={JWT}"
        ))
    }
}

#[tokio::test]
async fn security_output_redacts_reflected_authorization_and_secret() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/objects/test-object-1"))
        .respond_with(ReflectAuth)
        .mount(&server)
        .await;

    let secret = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test-fixtures/hmac/shared-secret.txt");

    let assert = helix()
        .env("RUST_LOG", "error")
        .env("HELIX_HMAC_SECRET", SECRET)
        .args([
            "security",
            &server.uri(),
            "--format",
            "json",
            "--hmac-secret-file",
            secret.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(!stdout.contains(SECRET), "{stdout}");
    assert!(!stderr.contains(SECRET), "{stderr}");
    assert!(!stdout.contains(JWT), "{stdout}");
    assert!(!stderr.contains(JWT), "{stderr}");
    assert!(!stdout.contains("Authorization: Bearer"), "{stdout}");
    assert!(!stderr.contains("Authorization: Bearer"), "{stderr}");
}

#[test]
fn compare_oversize_file_is_rejected_without_dumping_contents() {
    let previous = scratch("prev.json");
    let current = scratch("curr.json");
    std::fs::write(&previous, vec![b'X'; (MAX_COMPARE_FILE_BYTES as usize) + 1]).unwrap();
    std::fs::write(&current, "{}").unwrap();
    let assert = helix()
        .args([
            "compare",
            previous.to_str().unwrap(),
            current.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    std::fs::remove_file(&previous).ok();
    std::fs::remove_file(&current).ok();
    assert!(stderr.contains("bytes"), "{stderr}");
    assert!(!stderr.contains(&"X".repeat(20)), "{stderr}");
    assert!(!stdout.contains(&"X".repeat(20)), "{stdout}");
}

#[test]
fn hmac_secret_file_oversize_does_not_print_contents() {
    let p = scratch("hmac");
    std::fs::write(&p, vec![b'Q'; (MAX_SECRET_FILE_BYTES as usize) + 1]).unwrap();
    let assert = helix()
        .env("RUST_LOG", "error")
        .env_remove("HELIX_HMAC_SECRET")
        .args([
            "security",
            "http://127.0.0.1:1",
            "--hmac-secret-file",
            p.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    std::fs::remove_file(&p).ok();
    assert!(!stdout.contains(&"Q".repeat(20)), "{stdout}");
    assert!(!stderr.contains(&"Q".repeat(20)), "{stderr}");
}
