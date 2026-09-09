// SPDX-License-Identifier: Apache-2.0
//! WES suite: `helix verify` against in-process fixtures (not Ferrum).
//! TES/TRS/htsget checks are not executed.

use assert_cmd::Command;
use helix::identity::{DRS_VERIFY_IDS, WES_VERIFY_IDS};
use helix::model::{VerificationStatus, HELIXTEST_PIN};
use helix::verify::WES_CHECK_NAMES;
use predicates::prelude::*;
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod support;

use support::mock_ga4gh_wes::{
    start_mock_ga4gh_drs_and_wes, start_mock_ga4gh_wes, start_mock_wes_incomplete_service_info,
};

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

fn discovery_row<'a>(report: &'a Value, service: &str) -> &'a Value {
    report
        .get("discovery")
        .and_then(|d| d.as_array())
        .expect("discovery")
        .iter()
        .find(|s| s.get("service").and_then(|v| v.as_str()) == Some(service))
        .unwrap_or_else(|| panic!("{service} discovery row"))
}

fn by_service<'a>(rows: &'a [Value], service: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|t| t["service"].as_str() == Some(service))
        .collect()
}

#[tokio::test]
async fn valid_wes_target_passes_seven_and_skips_scatter() {
    let mock = start_mock_ga4gh_wes().await;
    let outcome = helix::verify::verify(&mock.origin()).await.expect("verify");
    assert!(outcome.is_success(), "valid WES should pass: {outcome:?}");
    assert!(!outcome.has_failures());

    let wes = outcome
        .run
        .discovery
        .iter()
        .find(|s| s.service == "wes")
        .expect("wes");
    assert!(wes.present, "DETECTED");
    assert!(wes.testable, "TESTABLE is not a pass");

    let wes_exec: Vec<_> = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.service == "wes")
        .collect();
    assert_eq!(wes_exec.len(), 7);
    for r in &wes_exec {
        assert_eq!(r.status, VerificationStatus::Pass);
        assert_eq!(r.service, "wes");
        assert!(WES_VERIFY_IDS.contains(&r.id.as_str()));
        assert!(r.code.starts_with("HLX-WES-"));
        assert_ne!(r.id, "wes.run.scatter_gather");
    }

    let scatter = outcome
        .run
        .skipped
        .iter()
        .find(|r| r.id == "wes.run.scatter_gather")
        .expect("scatter skip");
    assert_eq!(scatter.status, VerificationStatus::Skip);
    assert!(!scatter.is_pass());
    assert_eq!(scatter.code, "HLX-WES-008");
    assert_eq!(scatter.helixtest_name.as_deref(), Some(WES_CHECK_NAMES[7]));
    assert!(
        scatter
            .message
            .as_deref()
            .is_some_and(|m| m.contains("supports_scatter_gather=false")),
        "{scatter:?}"
    );

    let drs_skip: Vec<_> = outcome
        .run
        .skipped
        .iter()
        .filter(|r| r.service == "drs")
        .collect();
    assert_eq!(drs_skip.len(), 5);
    for (i, r) in drs_skip.iter().enumerate() {
        assert_eq!(r.id, DRS_VERIFY_IDS[i]);
        assert_eq!(r.status, VerificationStatus::Skip);
    }
}

#[tokio::test]
async fn helix_verify_cli_json_wes_is_deterministic() {
    let mock = start_mock_ga4gh_wes().await;
    let url = mock.origin();
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
    assert_eq!(a["helixtest_version"].as_str(), Some(HELIXTEST_PIN));
    assert_eq!(
        a["helixtest_sha"].as_str(),
        Some(helix::checker::executed_checker_source_sha256())
    );
    let executed = a["executed"].as_array().expect("executed");
    let wes_exec = by_service(executed, "wes");
    assert_eq!(wes_exec.len(), 7);
    for t in &wes_exec {
        assert_eq!(t["status"].as_str(), Some("pass"));
        assert!(t["code"].as_str().unwrap().starts_with("HLX-WES-"));
        assert!(t.get("failure").is_none());
    }
    let skipped = a["skipped"].as_array().expect("skipped");
    let scatter = skipped
        .iter()
        .find(|t| t["id"].as_str() == Some("wes.run.scatter_gather"))
        .expect("scatter");
    assert_eq!(scatter["status"].as_str(), Some("skip"));
    assert_ne!(scatter["status"].as_str(), Some("pass"));
    assert_eq!(scatter["code"].as_str(), Some("HLX-WES-008"));
    let wes = discovery_row(&a, "wes");
    assert_eq!(wes["present"], true);
    assert_eq!(wes["testable"], true);
    assert_ne!(
        a["summary"]["passed"], wes["present"],
        "discovery present is not a verification pass count"
    );
    strip_timestamp(&mut a);
    strip_timestamp(&mut b);
    assert_eq!(a, b, "JSON must be deterministic aside from timestamp");
}

#[tokio::test]
async fn invalid_wes_fails_with_failure_codes() {
    let server = start_mock_wes_incomplete_service_info().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let wes = discovery_row(&report, "wes");
    assert_eq!(wes["present"], true, "DETECTED");
    assert_eq!(wes["testable"], true, "TESTABLE is not a pass");
    let executed = report["executed"].as_array().expect("executed");
    let wes_exec = by_service(executed, "wes");
    assert!(
        wes_exec
            .iter()
            .any(|t| t["status"].as_str() == Some("fail")),
        "invalid WES service-info must fail checks: {wes_exec:?}"
    );
    let fail = wes_exec
        .iter()
        .find(|t| t["status"].as_str() == Some("fail"))
        .unwrap();
    assert!(fail.get("id").and_then(|v| v.as_str()).is_some());
    assert!(fail["code"].as_str().unwrap().starts_with("HLX-WES-"));
    assert!(fail.get("failure").and_then(|f| f.get("code")).is_some());
    assert!(fail.get("message").and_then(|m| m.as_str()).is_some());
    let diag = fail.get("diagnostic").expect("WES fail row diagnostic");
    assert_eq!(diag["id"], fail["id"]);
    assert_eq!(diag["code"], fail["code"]);
    assert!(diag.get("expected").and_then(|v| v.as_str()).is_some());
    assert!(diag
        .get("possible_causes")
        .and_then(|v| v.as_array())
        .is_some());
    assert!(diag.get("cause").is_none());
    assert!(
        !executed
            .iter()
            .any(|t| t["id"].as_str() == Some("discovery.wes")
                && t["status"].as_str() == Some("pass")),
        "do not treat DETECTED as a verification pass"
    );
}

#[tokio::test]
async fn missing_wes_skips_checks_and_is_not_a_pass() {
    let server = MockServer::start().await;
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let wes = discovery_row(&report, "wes");
    assert_eq!(wes["present"], false);
    assert_eq!(wes["testable"], false);
    let skipped = report["skipped"].as_array().expect("skipped");
    let wes_skip = by_service(skipped, "wes");
    assert_eq!(wes_skip.len(), 8);
    for (i, t) in wes_skip.iter().enumerate() {
        assert_eq!(t["id"].as_str(), Some(WES_VERIFY_IDS[i]));
        assert_eq!(t["status"].as_str(), Some("skip"));
        assert_ne!(t["status"].as_str(), Some("pass"));
        assert!(
            t["message"].as_str().unwrap().contains("WES not detected"),
            "{t}"
        );
    }
    assert_eq!(report["summary"]["passed"], 0);
}

#[tokio::test]
async fn unavailable_target_errors_wes_not_skip_or_pass() {
    let url = closed_origin();
    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &url, "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let executed = report["executed"].as_array().expect("executed");
    let wes_err = by_service(executed, "wes");
    assert_eq!(wes_err.len(), 8);
    for t in wes_err {
        assert_eq!(t["status"].as_str(), Some("error"));
        assert_ne!(t["status"].as_str(), Some("pass"));
        assert_ne!(t["status"].as_str(), Some("skip"));
        assert!(t["code"].as_str().unwrap().starts_with("HLX-WES-"));
        assert!(t["message"].as_str().unwrap().contains("unreachable"));
        assert_eq!(t["failure"]["code"].as_str(), t["code"].as_str());
    }
}

#[tokio::test]
async fn tes_detected_is_not_testable_and_not_executed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/tes/v1/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .failure()
        .code(1);
    let report = parse_json(&String::from_utf8_lossy(&assert.get_output().stdout));
    let tes = discovery_row(&report, "tes");
    assert_eq!(tes["present"], true, "DETECTED");
    assert_eq!(tes["testable"], false, "discovered but not testable");
    assert!(tes["not_testable_reason"]
        .as_str()
        .unwrap()
        .contains("does not execute TES"));
    let executed = report["executed"].as_array().expect("executed");
    let skipped = report["skipped"].as_array().expect("skipped");
    assert!(executed
        .iter()
        .all(|t| t["service"].as_str() != Some("tes")));
    assert!(skipped.iter().all(|t| t["service"].as_str() != Some("tes")));
}

#[tokio::test]
async fn combined_drs_and_wes_both_pass() {
    let mock = start_mock_ga4gh_drs_and_wes().await;
    let outcome = helix::verify::verify(&mock.origin()).await.expect("verify");
    assert!(outcome.is_success());
    assert_eq!(
        outcome
            .run
            .executed
            .iter()
            .filter(|r| r.service == "drs" && r.status == VerificationStatus::Pass)
            .count(),
        5
    );
    assert_eq!(
        outcome
            .run
            .executed
            .iter()
            .filter(|r| r.service == "wes" && r.status == VerificationStatus::Pass)
            .count(),
        7
    );
    assert!(outcome
        .run
        .skipped
        .iter()
        .any(|r| r.id == "wes.run.scatter_gather"));
}

#[tokio::test]
async fn helix_verify_text_prints_wes_ids_and_skip() {
    let mock = start_mock_ga4gh_wes().await;
    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .env("NO_COLOR", "1")
        .args(["verify", &mock.origin()])
        .assert()
        .success()
        .stdout(predicate::str::contains("WES      DETECTED     TESTABLE"))
        .stdout(predicate::str::contains("wes.service_info.reachable"))
        .stdout(predicate::str::contains("HLX-WES-001"))
        .stdout(predicate::str::contains("HLX-WES-008"))
        .stdout(predicate::str::contains("SKIP"))
        .stdout(predicate::str::contains("supports_scatter_gather=false"))
        .stdout(predicate::str::contains("DETECTED is not a pass"))
        .stdout(predicate::str::contains("HELIX VERIFICATION"));
}
