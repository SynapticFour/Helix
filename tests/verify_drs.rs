// SPDX-License-Identifier: Apache-2.0
//! DRS profile: `helix verify` against in-process fixtures (not Ferrum).

use assert_cmd::Command;
use helix::identity::{DRS_VERIFY_IDS, WES_VERIFY_IDS};
use helix::model::{VerificationStatus, HELIXTEST_PIN};
use helix::verify::{DRS_CHECK_NAMES, WES_CHECK_NAMES};
use predicates::prelude::*;
use serde_json::Value;
use wiremock::MockServer;

mod support;

use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_invalid_drs_object};

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; stdout={stdout}");
    })
}

fn strip_timestamp(v: &mut Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.insert("timestamp".into(), Value::String("TS".into()));
    }
}

fn closed_origin() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

fn drs_row(report: &Value) -> &Value {
    report
        .get("discovery")
        .and_then(|d| d.as_array())
        .expect("discovery")
        .iter()
        .find(|s| s.get("service").and_then(|v| v.as_str()) == Some("drs"))
        .expect("drs discovery row")
}

#[tokio::test]
async fn valid_drs_target_passes_with_stable_ids() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = helix::verify::verify(&mock.drs_url())
        .await
        .expect("verify");
    assert!(outcome.is_success(), "valid DRS should pass");
    assert!(!outcome.has_failures());
    assert_eq!(outcome.run.profile.as_deref(), Some("generic"));
    assert_eq!(
        outcome.run.helixtest_version.as_deref(),
        Some(HELIXTEST_PIN)
    );
    assert_eq!(
        outcome.run.helixtest_sha.as_deref(),
        Some(helix::checker::executed_checker_source_sha256())
    );
    assert_eq!(outcome.run.target.url, mock.drs_url().trim_end_matches('/'));
    let drs_exec: Vec<_> = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.service == "drs")
        .collect();
    assert_eq!(drs_exec.len(), 5);
    for (i, r) in drs_exec.iter().enumerate() {
        assert_eq!(r.id, DRS_VERIFY_IDS[i]);
        assert_eq!(r.status, VerificationStatus::Pass);
        assert_eq!(r.helixtest_name.as_deref(), Some(DRS_CHECK_NAMES[i]));
        assert_eq!(r.service, "drs");
    }
    let wes_skip: Vec<_> = outcome
        .run
        .skipped
        .iter()
        .filter(|r| r.service == "wes")
        .collect();
    assert_eq!(wes_skip.len(), 8, "WES unavailable → skip, not pass");
    for (i, r) in wes_skip.iter().enumerate() {
        assert_eq!(r.id, WES_VERIFY_IDS[i]);
        assert_eq!(r.status, VerificationStatus::Skip);
        assert_ne!(r.status, VerificationStatus::Pass);
        assert_eq!(r.helixtest_name.as_deref(), Some(WES_CHECK_NAMES[i]));
    }
    assert!(
        !outcome
            .run
            .executed
            .iter()
            .any(|r| r.service == "wes" && r.status == VerificationStatus::Pass),
        "missing WES must not pass WES checks"
    );
    let drs = outcome
        .run
        .discovery
        .iter()
        .find(|s| s.service == "drs")
        .expect("drs");
    assert!(drs.present, "DETECTED");
    assert!(
        drs.testable,
        "TESTABLE means checks run, not that they passed"
    );
    assert!(
        !outcome
            .run
            .executed
            .iter()
            .any(|r| r.id.starts_with("discovery.") && r.status == VerificationStatus::Pass),
        "do not call a green discovery result a verification pass"
    );
    let wes = outcome
        .run
        .discovery
        .iter()
        .find(|s| s.service == "wes")
        .expect("wes");
    assert!(!wes.present, "DRS-only fixture must not advertise WES");
    assert!(!wes.testable);
}

#[tokio::test]
async fn helix_verify_cli_json_is_deterministic_verification_run() {
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let first = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "json"])
        .assert()
        .success();
    let second = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--report", "json"])
        .assert()
        .success();
    let mut a = parse_json(&String::from_utf8_lossy(&first.get_output().stdout));
    let mut b = parse_json(&String::from_utf8_lossy(&second.get_output().stdout));
    assert!(a.get("services").is_none(), "not HelixTest OverallReport");
    assert!(a.get("passed").is_none());
    assert_eq!(a["profile"].as_str(), Some("generic"));
    assert_eq!(a["helix_version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(a["helixtest_version"].as_str(), Some(HELIXTEST_PIN));
    assert_eq!(
        a["helixtest_sha"].as_str(),
        Some(helix::checker::executed_checker_source_sha256())
    );
    assert_eq!(a["target"]["url"].as_str(), Some(url.trim_end_matches('/')));
    let executed = a["executed"].as_array().expect("executed");
    let drs_exec: Vec<_> = executed
        .iter()
        .filter(|t| t["service"].as_str() == Some("drs"))
        .collect();
    assert_eq!(drs_exec.len(), 5);
    for (i, t) in drs_exec.iter().enumerate() {
        assert_eq!(t["id"].as_str(), Some(DRS_VERIFY_IDS[i]));
        assert!(t["code"].as_str().unwrap().starts_with("HLX-DRS-"));
        assert_eq!(t["status"].as_str(), Some("pass"));
        assert_eq!(t["service"].as_str(), Some("drs"));
    }
    let skipped = a["skipped"].as_array().expect("skipped");
    assert_eq!(
        skipped
            .iter()
            .filter(|t| t["service"].as_str() == Some("wes"))
            .count(),
        8
    );
    let drs = drs_row(&a);
    assert_eq!(drs["present"], true);
    assert_eq!(drs["testable"], true);
    assert_ne!(
        a["summary"]["passed"], a["discovery"][0]["present"],
        "discovery present is not a verification pass count"
    );
    strip_timestamp(&mut a);
    strip_timestamp(&mut b);
    assert_eq!(a, b, "JSON must be deterministic aside from timestamp");
}

#[tokio::test]
async fn invalid_drs_response_fails_with_failure_codes() {
    let server = start_mock_invalid_drs_object().await;

    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let drs = drs_row(&report);
    assert_eq!(drs["present"], true, "DETECTED");
    assert_eq!(drs["testable"], true, "TESTABLE is not a pass");
    let executed = report["executed"].as_array().expect("executed");
    assert!(
        executed
            .iter()
            .any(|t| t["status"].as_str() == Some("fail")),
        "invalid DRS object must fail checks: {executed:?}"
    );
    let fail = executed
        .iter()
        .find(|t| t["status"].as_str() == Some("fail"))
        .unwrap();
    assert!(fail.get("id").and_then(|v| v.as_str()).is_some());
    assert!(fail["code"].as_str().unwrap().starts_with("HLX-DRS-"));
    assert!(fail.get("failure").and_then(|f| f.get("code")).is_some());
    assert!(fail.get("message").and_then(|m| m.as_str()).is_some());
    let diag = fail.get("diagnostic").expect("fail row diagnostic");
    assert_eq!(diag["id"], fail["id"]);
    assert_eq!(diag["code"], fail["code"]);
    assert!(diag.get("expected").and_then(|v| v.as_str()).is_some());
    assert!(diag.get("observed").and_then(|v| v.as_str()).is_some());
    assert!(diag.get("hint").and_then(|v| v.as_str()).is_some());
    assert!(diag
        .get("possible_causes")
        .and_then(|v| v.as_array())
        .is_some());
    assert!(diag.get("cause").is_none());
    assert!(
        !executed
            .iter()
            .any(|t| t["id"].as_str() == Some("discovery.drs")
                && t["status"].as_str() == Some("pass")),
        "do not treat DETECTED as a verification pass"
    );
}

#[tokio::test]
async fn missing_drs_skips_checks_and_is_not_a_pass() {
    let server = MockServer::start().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let drs = drs_row(&report);
    assert_eq!(drs["present"], false);
    assert_eq!(drs["testable"], false);
    assert!(report["executed"].as_array().unwrap().is_empty());
    let skipped = report["skipped"].as_array().expect("skipped");
    let drs_skip: Vec<_> = skipped
        .iter()
        .filter(|t| t["service"].as_str() == Some("drs"))
        .collect();
    assert_eq!(drs_skip.len(), 5);
    for (i, t) in drs_skip.iter().enumerate() {
        assert_eq!(t["id"].as_str(), Some(DRS_VERIFY_IDS[i]));
        assert_eq!(t["status"].as_str(), Some("skip"));
        assert_ne!(t["status"].as_str(), Some("pass"));
        assert!(t["code"].as_str().unwrap().starts_with("HLX-DRS-"));
        assert!(
            t["message"].as_str().unwrap().contains("DRS not detected"),
            "{t}"
        );
        assert!(t.get("diagnostic").is_none(), "skip has no diagnostic: {t}");
    }
    let wes_skip: Vec<_> = skipped
        .iter()
        .filter(|t| t["service"].as_str() == Some("wes"))
        .collect();
    assert_eq!(wes_skip.len(), 8);
    for t in &wes_skip {
        assert_eq!(t["status"].as_str(), Some("skip"));
        assert!(t["message"].as_str().unwrap().contains("WES not detected"));
    }
    assert_eq!(report["summary"]["skipped"], 13);
    assert_eq!(report["summary"]["passed"], 0);
    assert_eq!(report["profile"].as_str(), Some("generic"));
}

#[tokio::test]
async fn skipped_check_is_never_serialized_as_pass() {
    let server = MockServer::start().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let report = parse_json(&stdout);
    for t in report["skipped"].as_array().unwrap() {
        assert_eq!(t["status"].as_str(), Some("skip"));
        assert!(t.get("passed").is_none());
        assert!(t.get("failure").is_none());
    }
}

#[tokio::test]
async fn unavailable_target_is_error_not_skip_or_pass() {
    let url = closed_origin();
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    assert_eq!(report["target"]["url"].as_str(), Some(url.as_str()));
    let executed = report["executed"].as_array().expect("executed");
    assert_eq!(executed.len(), 13);
    for t in executed {
        assert_eq!(t["status"].as_str(), Some("error"));
        assert_ne!(t["status"].as_str(), Some("pass"));
        assert_ne!(t["status"].as_str(), Some("skip"));
        let code = t["code"].as_str().unwrap();
        assert!(
            code.starts_with("HLX-DRS-") || code.starts_with("HLX-WES-"),
            "{t}"
        );
        assert!(
            t["message"].as_str().unwrap().contains("unreachable"),
            "{t}"
        );
        assert_eq!(t["failure"]["code"].as_str(), t["code"].as_str());
        let diag = t.get("diagnostic").expect("error row diagnostic");
        assert!(
            diag["observed"].as_str().unwrap().contains("not reachable"),
            "{diag}"
        );
    }
    assert!(report["skipped"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn helix_verify_text_uses_detected_not_found_and_helix_ids() {
    let mock = start_mock_ga4gh_drs().await;
    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .env("NO_COLOR", "1")
        .args(["verify", &mock.drs_url()])
        .assert()
        .success()
        .stdout(predicate::str::contains("DETECTED     TESTABLE"))
        .stdout(predicate::str::contains("NOT_DETECTED"))
        .stdout(predicate::str::contains("DETECTED is not a pass"))
        .stdout(predicate::str::contains("HELIX VERIFICATION"))
        .stdout(predicate::str::contains("It is not GA4GH certification."))
        .stdout(predicate::str::contains("drs.object.reachable"))
        .stdout(predicate::str::contains("HLX-DRS-001"))
        .stdout(predicate::str::contains("HLX-DRS-005"))
        .stdout(predicate::str::contains("HLX-WES-001"))
        .stdout(predicate::str::contains("SKIP"))
        .stdout(predicate::str::contains("Summary:"))
        .stdout(predicate::str::contains("Not compared"));
}

#[tokio::test]
async fn helix_verify_exits_1_when_drs_missing() {
    let server = MockServer::start().await;
    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DRS not detected"));
}
