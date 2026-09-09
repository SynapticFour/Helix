// SPDX-License-Identifier: Apache-2.0
//! Hostile / malformed in-process targets. Local fixtures only — not real-world attacks.
//! Helix is not a security product. Malformed HTTP must never be an overall PASS.

use std::time::Instant;

use assert_cmd::Command;
use helix::http_safety::HTTP_REQUEST_TIMEOUT_SECS;
use helix::model::VerificationStatus;
use helix::report::verify_json;
use helix::verify::VerifyOutcome;

mod support;

use support::mock_adversarial::{
    start_ansi_and_log_injection, start_connection_reset, start_extremely_long_strings,
    start_huge_json, start_invalid_content_type, start_invalid_headers, start_malformed_json,
    start_malformed_service_info, start_redirect, start_slow_response, start_unexpected_status,
    ADVERSARIAL_JWT, ADVERSARIAL_USERINFO, SLOW_DELAY,
};

fn assert_no_leaks(text: &str, case: &str) {
    assert!(
        !text.contains(ADVERSARIAL_JWT),
        "{case}: JWT decoy leaked:\n{text}"
    );
    assert!(
        !text.contains(ADVERSARIAL_USERINFO),
        "{case}: URL userinfo leaked:\n{text}"
    );
    assert!(
        !text.contains("Authorization: Bearer eyJ"),
        "{case}: Authorization header leaked:\n{text}"
    );
    assert!(
        !text.contains("helix-dummy-hmac-not-for-production-do-not-use"),
        "{case}: dummy HMAC leaked:\n{text}"
    );
}

fn assert_hostile(outcome: &VerifyOutcome, case: &str) -> String {
    assert!(
        !outcome.is_success(),
        "{case}: overall success on a hostile fixture"
    );
    assert_ne!(
        outcome.run.overall_status(),
        VerificationStatus::Pass,
        "{case}: overall_status pass"
    );
    for r in outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
    {
        if r.id == "drs.object.schema" {
            assert_ne!(
                r.status,
                VerificationStatus::Pass,
                "{case}: schema must not PASS on a malformed/hostile body ({:?})",
                r.status
            );
        }
    }
    let json = verify_json(&outcome.run).expect("verify JSON");
    serde_json::from_str::<serde_json::Value>(&json).expect("redacted JSON stays JSON");
    assert_no_leaks(&json, case);
    json
}

fn statuses(outcome: &VerifyOutcome) -> Vec<(String, VerificationStatus)> {
    outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
        .map(|r| (r.id.clone(), r.status))
        .collect()
}

async fn twice(url: &str, case: &str) -> VerifyOutcome {
    let a = helix::verify::verify(url)
        .await
        .unwrap_or_else(|e| panic!("{case}: verify error (must terminate): {e:#}"));
    let json = assert_hostile(&a, case);
    let b = helix::verify::verify(url)
        .await
        .unwrap_or_else(|e| panic!("{case}: second verify error: {e:#}"));
    assert_hostile(&b, &format!("{case} (repeat)"));
    assert_eq!(
        statuses(&a),
        statuses(&b),
        "{case}: failure information must be deterministic (status by id)"
    );
    let _ = json;
    a
}

#[tokio::test]
async fn malformed_json_is_not_a_pass() {
    let server = start_malformed_json().await;
    let outcome = twice(&server.uri(), "malformed JSON").await;
    let schema = outcome
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .expect("schema row");
    assert_eq!(schema.status, VerificationStatus::Fail);
}

#[tokio::test]
async fn huge_json_terminates_without_pass() {
    let server = start_huge_json().await;
    let outcome = twice(&server.uri(), "huge JSON").await;
    assert_eq!(
        outcome
            .run
            .discovery
            .iter()
            .find(|d| d.service == "drs")
            .map(|d| d.present),
        Some(false)
    );
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn invalid_headers_are_not_a_pass() {
    let server = start_invalid_headers().await;
    let _ = twice(&server.uri(), "invalid headers").await;
}

#[tokio::test]
async fn redirect_is_not_followed_and_not_a_pass() {
    let server = start_redirect().await;
    let outcome = twice(&server.uri(), "redirect").await;
    assert_eq!(
        outcome
            .run
            .discovery
            .iter()
            .find(|d| d.service == "drs")
            .map(|d| d.present),
        Some(false),
        "302 must not become DETECTED"
    );
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn slow_response_respects_timeout() {
    let server = start_slow_response().await;
    let start = Instant::now();
    let outcome = helix::verify::verify(&server.uri())
        .await
        .expect("slow fixture must not hang the process");
    let elapsed = start.elapsed();
    assert_hostile(&outcome, "slow response");
    assert!(
        elapsed < std::time::Duration::from_secs(20),
        "Helix hung on a slow fixture: {elapsed:?} (timeout {HTTP_REQUEST_TIMEOUT_SECS}s, delay {:?})",
        SLOW_DELAY
    );
    assert!(
        elapsed >= std::time::Duration::from_millis(3_500),
        "expected the Helix-owned client to hit its request timeout, got {elapsed:?}"
    );
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn connection_reset_terminates_without_pass() {
    let origin = start_connection_reset();
    let outcome = twice(&origin.url, "connection reset").await;
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn invalid_content_type_is_not_a_pass() {
    let server = start_invalid_content_type().await;
    let _ = twice(&server.uri(), "invalid content type").await;
}

#[tokio::test]
async fn unexpected_status_is_not_detected_or_pass() {
    let server = start_unexpected_status().await;
    let outcome = twice(&server.uri(), "unexpected status 418").await;
    assert_eq!(
        outcome
            .run
            .discovery
            .iter()
            .find(|d| d.service == "drs")
            .map(|d| d.present),
        Some(false)
    );
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn malformed_service_info_is_not_a_pass() {
    let server = start_malformed_service_info().await;
    let outcome = twice(&server.uri(), "malformed service-info").await;
    let drs = outcome
        .run
        .discovery
        .iter()
        .find(|d| d.service == "drs")
        .expect("drs");
    assert!(drs.present, "200 service-info is DETECTED");
    assert!(!outcome.is_success());
}

#[tokio::test]
async fn extremely_long_strings_do_not_panic_or_pass_schema() {
    let server = start_extremely_long_strings().await;
    let outcome = twice(&server.uri(), "extremely long strings").await;
    let schema = outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
        .find(|r| r.id == "drs.object.schema")
        .expect("schema");
    assert_ne!(schema.status, VerificationStatus::Pass);
}

#[tokio::test]
async fn ansi_and_forged_newlines_do_not_control_the_report() {
    let server = start_ansi_and_log_injection().await;
    let outcome = twice(&server.uri(), "ansi / log injection").await;
    let json = verify_json(&outcome.run).expect("json");
    let text = helix::report::format_verify_text(&outcome.run, false);
    assert!(!json.contains('\u{1b}'), "{json}");
    assert!(!text.contains('\u{1b}'), "{text:?}");
    assert!(text.starts_with("HELIX VERIFICATION\n"));
    assert!(
        !text.contains("\nHELIX VERIFICATION\n"),
        "target must not inject a second report header:\n{text}"
    );
}

#[tokio::test]
async fn helix_cli_malformed_json_exits_1_without_leaking() {
    let server = start_malformed_json().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("CLI JSON");
    assert!(
        v["summary"]["failed"].as_u64().unwrap_or(0) >= 1,
        "CLI must fail at least one check on malformed JSON: {stdout}"
    );
    assert_ne!(
        v["summary"]["passed"].as_u64(),
        Some(5),
        "CLI must not treat malformed JSON as five DRS passes: {stdout}"
    );
    assert_no_leaks(&stdout, "cli malformed stdout");
    assert_no_leaks(&stderr, "cli malformed stderr");
}
