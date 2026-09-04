// SPDX-License-Identifier: Apache-2.0
//! Integration: HelixTest DRS checks via `helix verify` against the B1 mock (not Ferrum).

use assert_cmd::Command;
use common::report::TestStatus;
use helix::verify::DRS_CHECK_NAMES;
use predicates::prelude::*;
use serde_json::Value;

mod support;

use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

#[tokio::test]
async fn helixtest_drs_checks_pass_against_b1_mock() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = helix::verify::verify(&mock.drs_url())
        .await
        .expect("verify");
    assert!(
        !outcome.has_failures(),
        "DRS failures: {:?}",
        outcome
            .drs
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Fail)
            .map(|t| (&t.name, t.error.as_deref()))
            .collect::<Vec<_>>()
    );
    let names: Vec<&str> = outcome.drs.tests.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, DRS_CHECK_NAMES);
    for t in &outcome.drs.tests {
        assert_eq!(t.status, TestStatus::Pass, "{}", t.name);
    }
}

#[tokio::test]
async fn helix_verify_cli_passes_drs_on_b1_mock() {
    let mock = start_mock_ga4gh_drs().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &mock.drs_url(), "--format", "json"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let report: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; stdout={stdout}");
    });
    assert!(
        report.get("discovery").is_none(),
        "JSON must be HelixTest OverallReport, not a discovery wrapper"
    );
    let skipped = report
        .get("skipped_services")
        .and_then(|s| s.as_array())
        .expect("skipped_services");
    assert!(
        skipped
            .iter()
            .any(|s| s.get("service").and_then(|v| v.as_str()) == Some("Wes")),
        "B1 mock exposes WES-shaped /service-info; skip must not count as pass: {skipped:?}"
    );
    let services = report
        .get("services")
        .and_then(|s| s.as_array())
        .expect("services");
    let drs = services
        .iter()
        .find(|s| s.get("service").and_then(|v| v.as_str()) == Some("Drs"))
        .expect("DRS service");
    let tests = drs
        .get("tests")
        .and_then(|t| t.as_array())
        .expect("DRS tests");
    let names: Vec<&str> = tests
        .iter()
        .map(|t| t.get("name").and_then(|n| n.as_str()).unwrap_or(""))
        .collect();
    assert_eq!(names, DRS_CHECK_NAMES);
    for t in tests {
        assert_eq!(
            t.get("status").and_then(|s| s.as_str()),
            Some("pass"),
            "{t}"
        );
    }
}

#[tokio::test]
async fn helix_verify_exits_1_when_drs_missing() {
    let server = wiremock::MockServer::start().await;
    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DRS not discovered"));
}

#[tokio::test]
async fn helix_verify_report_alias_is_json() {
    let mock = start_mock_ga4gh_drs().await;
    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &mock.drs_url(), "--report", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"service\": \"Drs\""));
}
