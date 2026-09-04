// SPDX-License-Identifier: Apache-2.0
//! Frozen public CLI contract (`docs/CLI_CONTRACT.md`).
//! Not certification. Not HELIOS.

use assert_cmd::Command;
use helix::model::{HELIXTEST_PIN, HELIXTEST_SHA};
use predicates::prelude::*;
use serde_json::Value;

mod support;

use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

/// Wall clock is the only field that changes between otherwise identical text runs.
fn strip_verify_timestamp(text: &str) -> String {
    text.lines()
        .map(|line| {
            let t = line.trim();
            if t.len() == 20
                && t.as_bytes().get(4) == Some(&b'-')
                && t.as_bytes().get(10) == Some(&b'T')
                && t.ends_with('Z')
            {
                "  TS"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_json_stdout(stdout: &[u8]) -> Value {
    let text = String::from_utf8_lossy(stdout);
    let trimmed = text.trim();
    assert!(
        trimmed.starts_with('{'),
        "JSON stdout must be a single object, not logs or text: {text:?}"
    );
    serde_json::from_str(trimmed).unwrap_or_else(|e| {
        panic!("stdout is not one JSON value ({e}): {text}");
    })
}

fn closed_origin() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

#[test]
fn helix_version_matches_json_helix_version() {
    let ver = helix()
        .arg("--version")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let banner = String::from_utf8_lossy(&ver);
    let pkg = env!("CARGO_PKG_VERSION");
    assert!(
        banner.contains(pkg),
        "helix --version must report Cargo package version: {banner}"
    );
}

#[tokio::test]
async fn format_text_matches_default_text() {
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let default = helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args(["verify", &url])
        .assert()
        .success();
    let explicit = helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "text"])
        .assert()
        .success();
    let default_text = String::from_utf8_lossy(&default.get_output().stdout);
    let explicit_text = String::from_utf8_lossy(&explicit.get_output().stdout);
    assert_eq!(
        strip_verify_timestamp(&default_text),
        strip_verify_timestamp(&explicit_text),
        "--format text must match the default"
    );
    let text = default_text;
    assert!(text.contains("HELIX VERIFICATION"));
    assert!(text.contains("This is a technical verification signal."));
    assert!(text.contains("It is not GA4GH certification."));
    assert!(text.contains("NOT_DETECTED") || text.contains("DETECTED"));
    assert!(text.contains("DETECTED is not a pass"));
    assert!(text.contains("Target:"));
    assert!(text.contains("Helix:"));
    assert!(text.contains("Test suite:"));
    assert!(text.contains("Services:"));
    assert!(text.contains("Results:"));
    assert!(text.contains("Summary:"));
    assert!(text.contains("Changes:"));
    assert!(text.contains("Not compared"));
    assert!(
        !text.to_lowercase().split_whitespace().any(|w| w == "found"),
        "text must not say found as verified: {text}"
    );
}

#[tokio::test]
async fn format_json_is_verification_run_without_human_marks() {
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let assert = helix()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "json"])
        .assert()
        .success();
    let stdout = &assert.get_output().stdout;
    let raw = String::from_utf8_lossy(stdout);
    assert!(
        !raw.contains('\u{1b}'),
        "JSON must never contain ANSI: {raw:?}"
    );
    assert!(!raw.contains("DETECTED     TESTABLE"), "no discovery table");
    assert!(!raw.contains("  PASS  "));
    assert!(!raw.contains("  SKIP  "));
    let v = parse_json_stdout(stdout);
    assert_eq!(v["helix_version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(v["schema_version"].as_str(), Some("helix-verification-v1"));
    assert_eq!(v["helixtest_version"].as_str(), Some(HELIXTEST_PIN));
    assert_eq!(v["helixtest_sha"].as_str(), Some(HELIXTEST_SHA));
    assert_eq!(v["target"]["url"].as_str(), Some(url.trim_end_matches('/')));
    assert!(v.get("passed").is_none());
    assert!(v.get("services").is_none());
    assert!(v.get("signature").is_none());
    assert!(v.get("ro_crate").is_none());
    assert_eq!(v["profile"].as_str(), Some("generic"));
    assert!(v["timestamp"].as_str().is_some());
    let executed = v["executed"].as_array().expect("executed");
    let skipped = v["skipped"].as_array().expect("skipped");
    for t in executed.iter().chain(skipped.iter()) {
        let status = t["status"].as_str().expect("status");
        assert!(
            matches!(status, "pass" | "fail" | "skip" | "error"),
            "status must be lowercase: {t}"
        );
        assert_ne!(status, "PASS");
        assert!(t.get("passed").is_none());
        assert!(t.get("id").and_then(|x| x.as_str()).is_some());
        assert!(t.get("code").and_then(|x| x.as_str()).is_some());
    }
}

#[tokio::test]
async fn report_json_alias_matches_format_json() {
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = helix()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "json"])
        .assert()
        .success();
    let b = helix()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--report", "json"])
        .assert()
        .success();
    let mut va = parse_json_stdout(&a.get_output().stdout);
    let mut vb = parse_json_stdout(&b.get_output().stdout);
    va.as_object_mut()
        .unwrap()
        .insert("timestamp".into(), Value::String("TS".into()));
    vb.as_object_mut()
        .unwrap()
        .insert("timestamp".into(), Value::String("TS".into()));
    assert_eq!(va, vb);
}

#[tokio::test]
async fn debug_logs_must_not_corrupt_json_stdout() {
    let mock = start_mock_ga4gh_drs().await;
    let assert = helix()
        .env("RUST_LOG", "debug")
        .args(["verify", &mock.drs_url(), "--format", "json"])
        .assert()
        .success();
    let out = assert.get_output();
    parse_json_stdout(&out.stdout);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("GET with retry"),
        "HelixTest logs must not appear on stdout: {stdout}"
    );
}

#[tokio::test]
async fn json_skip_is_never_pass_and_discovery_is_not_a_pass() {
    let mock = start_mock_ga4gh_drs().await;
    let v = parse_json_stdout(
        &helix()
            .env("RUST_LOG", "error")
            .args(["verify", &mock.drs_url(), "--format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    let skipped = v["skipped"].as_array().expect("skipped");
    assert!(skipped.iter().all(|t| t["status"].as_str() == Some("skip")));
    assert!(skipped.iter().any(|t| t["service"].as_str() == Some("wes")));
    let discovery = v["discovery"].as_array().expect("discovery");
    assert_eq!(
        discovery
            .iter()
            .map(|s| s["service"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["drs", "wes", "tes", "trs", "htsget"]
    );
    let wes = discovery
        .iter()
        .find(|s| s["service"].as_str() == Some("wes"))
        .unwrap();
    assert_eq!(wes["present"], false);
    assert_eq!(wes["testable"], false);
    assert_ne!(
        v["summary"]["passed"], wes["present"],
        "discovery present is not summary.passed"
    );
    assert!(
        !v["executed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"].as_str().unwrap().starts_with("discovery.")),
        "discovery must not be executed as a check"
    );
}

#[tokio::test]
async fn skip_only_exits_1() {
    let server = wiremock::MockServer::start().await;
    let out = helix()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let v = parse_json_stdout(&out.stdout);
    assert_eq!(v["summary"]["passed"], 0);
    assert!(v["executed"].as_array().unwrap().is_empty());
    for t in v["skipped"].as_array().unwrap() {
        assert_eq!(t["status"].as_str(), Some("skip"));
    }
}

#[tokio::test]
async fn unreachable_exits_1_with_error_rows() {
    let url = closed_origin();
    let v = parse_json_stdout(
        &helix()
            .env("RUST_LOG", "error")
            .args(["verify", &url, "--format", "json"])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    let executed = v["executed"].as_array().unwrap();
    assert!(executed
        .iter()
        .all(|t| t["status"].as_str() == Some("error")));
    assert!(!executed
        .iter()
        .any(|t| t["status"].as_str() == Some("pass")));
    assert!(!executed
        .iter()
        .any(|t| t["status"].as_str() == Some("skip")));
}

#[test]
fn missing_url_is_usage_exit_2() {
    helix()
        .args(["verify"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn unknown_format_is_usage_exit_2() {
    helix()
        .args(["verify", "http://127.0.0.1:9", "--format", "yaml"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn unknown_profile_is_usage_exit_2() {
    helix()
        .args(["verify", "http://127.0.0.1:9", "--profile", "ga4gh-drs"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn invalid_url_exits_1_without_json_stdout() {
    let out = helix()
        .args(["verify", "ftp://example.invalid", "--format", "json"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim().is_empty() || !stdout.trim().starts_with('{'),
        "runtime URL error must not emit VerificationRun: {stdout:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("http") || stderr.contains("https") || stderr.contains("endpoint"),
        "error on stderr: {stderr}"
    );
}

#[test]
fn reserved_namespaces_are_not_implemented() {
    for ns in ["tes", "trs", "htsget", "beacon", "certify", "helios"] {
        helix().args([ns]).assert().failure().code(2).stderr(
            predicate::str::contains("unrecognized").or(predicate::str::contains("unrecognised")),
        );
    }
}
