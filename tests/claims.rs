// SPDX-License-Identifier: Apache-2.0
//! Overclaiming tests for live `helix verify` JSON. Constructed-run predicates
//! live in `src/claims.rs`. Not certification. Not HELIOS.

use assert_cmd::Command;
use serde_json::Value;

mod support;

use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_invalid_drs_object};

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn verify_json(url: &str) -> Value {
    let out = helix()
        .env("RUST_LOG", "error")
        .args(["verify", url, "--format", "json"])
        .assert()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).expect("verify JSON")
}

fn claim<'a>(v: &'a Value, kind: &str) -> &'a Value {
    v["claims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == kind)
        .unwrap_or_else(|| panic!("missing claim {kind}"))
}

fn block_codes(claim: &Value) -> Vec<&str> {
    claim["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["code"].as_str())
        .collect()
}

#[tokio::test]
async fn honest_drs_pass_is_not_verified() {
    let mock = start_mock_ga4gh_drs().await;
    let v = verify_json(&mock.drs_url());
    let claims = v["claims"].as_array().expect("claims");
    assert_eq!(claims.len(), 6);
    for kind in [
        "ga4gh_requirement",
        "schema",
        "behavior",
        "interoperability",
        "security",
        "benchmark",
    ] {
        assert_eq!(claim(&v, kind)["status"], "not_verified", "{kind}");
    }
    let ga4gh = claim(&v, "ga4gh_requirement");
    let codes = block_codes(ga4gh);
    assert!(codes.contains(&"unversioned_run"), "{codes:?}");
    assert!(codes.contains(&"no_normative_checks"), "{codes:?}");
    assert!(
        !codes.contains(&"normative_check_failed"),
        "PASS fixture is not a MUST fail: {codes:?}"
    );
    assert_eq!(v["standard_selection"]["integrity_validated"], false);
}

#[tokio::test]
async fn fixture_schema_fail_is_not_a_normative_failure_claim() {
    let server = start_mock_invalid_drs_object().await;
    let v = verify_json(&server.uri());
    assert!(v["executed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["status"] == "fail"));
    for kind in ["ga4gh_requirement", "schema", "behavior"] {
        let codes = block_codes(claim(&v, kind));
        assert!(
            !codes.contains(&"normative_check_failed"),
            "{kind} fixture FAIL must not be a MUST fail: {codes:?}"
        );
        assert_eq!(claim(&v, kind)["status"], "not_verified");
    }
}

#[tokio::test]
async fn text_report_claims_come_from_the_model() {
    let mock = start_mock_ga4gh_drs().await;
    let out = helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args(["verify", &mock.drs_url(), "--format", "text"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    let claims = text
        .split("What:")
        .next()
        .expect("Claims section precedes What:");
    assert!(claims.contains("Claims (predicates; not GA4GH certification):"));
    assert!(claims.contains("No VERIFIED claim is justified by this run."));
    assert!(claims.contains("ga4gh_requirement  NOT_VERIFIED"));
    assert!(claims.contains("schema  NOT_VERIFIED"));
    assert!(claims.contains("behavior  NOT_VERIFIED"));
    assert!(claims.contains("interoperability  NOT_VERIFIED"));
    assert!(claims.contains("security  NOT_VERIFIED"));
    assert!(claims.contains("benchmark  NOT_VERIFIED"));
    assert!(claims.contains("Why not verified:"));
    assert!(claims.contains("unversioned_run"));
    assert!(!claims.contains("ga4gh_requirement  VERIFIED"));
    assert!(!claims.contains("normative_check_failed"));
}

#[tokio::test]
async fn available_but_not_supported_is_not_verified() {
    let mock = start_mock_ga4gh_drs().await;
    let out = helix()
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.5.0",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("verify JSON");
    let ga4gh = claim(&v, "ga4gh_requirement");
    assert_eq!(ga4gh["status"], "not_verified");
    let codes = block_codes(ga4gh);
    assert!(codes.contains(&"available_but_not_supported"), "{codes:?}");
    assert!(v["claims"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["status"] == "not_verified"));
}
