// SPDX-License-Identifier: Apache-2.0
use assert_cmd::Command;
use predicates::prelude::*;

mod support;

use support::mock_ga4gh_drs::start_mock_invalid_drs_object;

#[tokio::test]
async fn helix_verify_json_reports_discovered_drs_without_calling_it_a_pass() {
    let server = start_mock_invalid_drs_object().await;

    // Stub object is enough for discovery; HelixTest DRS schema/range checks FAIL → exit 1.
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("\"service\": \"drs\""));
    assert!(stdout.contains("\"status\": \"fail\""));
    assert!(stdout.contains("\"present\": true"));
    assert!(stdout.contains("\"testable\": true"));
    assert!(
        !stdout.contains("\"id\": \"discovery.drs\""),
        "discovery must not be emitted as a verification check pass/fail: {stdout}"
    );
}

#[tokio::test]
async fn helix_verify_text_uses_detected_not_found() {
    let server = start_mock_invalid_drs_object().await;

    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .env("NO_COLOR", "1")
        .args(["verify", &server.uri()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("DETECTED     TESTABLE"))
        .stdout(predicate::str::contains("NOT_DETECTED"))
        .stdout(predicate::str::contains("DETECTED is not a pass"))
        .stdout(predicate::str::contains("not conformance"))
        .stdout(predicate::str::contains("HELIX VERIFICATION"));
}
