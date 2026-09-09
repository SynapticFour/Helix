// SPDX-License-Identifier: Apache-2.0
//! `helix bench` against two in-process mocks. Warnings never fail the process.

use assert_cmd::Command;
use helix::bench::WORKLOAD_ID;
use predicates::prelude::*;
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mount_tiny_workload(server: &MockServer, object_status: u16) {
    for p in ["/health", "/ga4gh/drs/v1/service-info"] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/objects/test-object-1"))
        .respond_with(ResponseTemplate::new(object_status).set_body_string("obj"))
        .mount(server)
        .await;
}

fn bench_json(baseline: &str, candidate: &str) -> Value {
    let out = Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args([
            "bench",
            "--baseline",
            baseline,
            "--candidate",
            candidate,
            "--baseline-label",
            "vX",
            "--candidate-label",
            "vY",
            "--warmup",
            "0",
            "--repetitions",
            "2",
            "--format",
            "json",
        ])
        .assert()
        .success();
    serde_json::from_slice(&out.get_output().stdout).expect("bench JSON on stdout")
}

#[tokio::test]
async fn helix_bench_warns_on_errors_but_exits_zero() {
    let baseline = MockServer::start().await;
    let candidate = MockServer::start().await;
    mount_tiny_workload(&baseline, 200).await;
    mount_tiny_workload(&candidate, 500).await;

    let v = bench_json(&baseline.uri(), &candidate.uri());
    assert_eq!(v["warning"], true);
    assert!(v["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.as_str().unwrap().contains("error_rate")));
    assert_eq!(v["baseline"]["errors"], 0);
    assert_eq!(v["candidate"]["errors"], 2);
    assert_eq!(v["threshold_pct"], 10.0);
    assert_eq!(v["analysis"]["measurement"], true);
    assert_eq!(v["analysis"]["warning"], true);
    assert_eq!(v["analysis"]["regression"], false);
    assert_eq!(v["analysis"]["verification_failure"], false);
    assert_eq!(v["analysis"]["warning_means"], helix::bench::WARNING_MEANS);
    assert_eq!(v["baseline"]["metadata"]["repetitions"], 2);
    assert_eq!(v["baseline"]["runs"].as_array().unwrap().len(), 2);
    assert_eq!(v["workload_id"], WORKLOAD_ID);
    assert_eq!(v["environment"]["comparable"], true);
    assert_eq!(v["baseline"]["metadata"]["workload_id"], WORKLOAD_ID);
    assert_eq!(v["baseline"]["metadata"]["warmup"], 0);
    assert!(v["baseline"]["metadata"]["helix_version"].is_string());
    assert!(v["baseline"]["metadata"]["os"].is_string());
    assert!(v["baseline"]["metadata"]["arch"].is_string());
    assert!(v["baseline"]["metadata"]["timestamp"].is_string());
    assert!(v["baseline"]["latency"]["median_ms"].is_number());
    assert!(v["baseline"]["latency"]["min_ms"].is_number());
    assert!(v["baseline"]["latency"]["max_ms"].is_number());
    assert!(v["baseline"]["latency"]["p95_ms"].is_null());
    assert!(v["baseline"]["bytes"].as_u64().unwrap() > 0);
    assert_eq!(v["baseline"]["runs"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn helix_bench_quiet_when_both_sides_succeed() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    mount_tiny_workload(&a, 200).await;
    mount_tiny_workload(&b, 200).await;

    let v = bench_json(&a.uri(), &b.uri());
    assert_eq!(v["warning"], false);
    assert_eq!(v["analysis"]["measurement"], true);
    assert_eq!(v["analysis"]["warning"], false);
    assert_eq!(v["analysis"]["regression"], false);
    assert_eq!(v["analysis"]["verification_failure"], false);
    assert_eq!(v["baseline"]["errors"], 0);
    assert_eq!(v["candidate"]["errors"], 0);
    let err = v["diff"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["name"] == "error_rate")
        .unwrap();
    assert_eq!(err["worse"], false);
}

#[tokio::test]
async fn helix_bench_text_does_not_claim_significance_or_fail() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    mount_tiny_workload(&a, 200).await;
    mount_tiny_workload(&b, 200).await;

    Command::cargo_bin("helix")
        .unwrap()
        .env("RUST_LOG", "error")
        .args([
            "bench",
            "--baseline",
            &a.uri(),
            "--candidate",
            &b.uri(),
            "--warmup",
            "0",
            "--repetitions",
            "2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(WORKLOAD_ID))
        .stdout(predicate::str::contains("Not a significance test"))
        .stdout(predicate::str::contains(
            "Performance changed enough to merit human inspection.",
        ))
        .stdout(predicate::str::contains(
            "It does not mean the implementation is incorrect.",
        ))
        .stdout(predicate::str::contains("not a verification failure"))
        .stdout(predicate::str::contains("statistically significant").not());
}
