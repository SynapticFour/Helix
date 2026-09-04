// SPDX-License-Identifier: Apache-2.0
//! `helix bench` against two in-process mocks. Warnings never fail the process.

use assert_cmd::Command;
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
    assert_eq!(v["candidate"]["errors"], 1);
    assert_eq!(v["threshold_pct"], 10.0);
}

#[tokio::test]
async fn helix_bench_quiet_when_both_sides_succeed() {
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    mount_tiny_workload(&a, 200).await;
    mount_tiny_workload(&b, 200).await;

    let v = bench_json(&a.uri(), &b.uri());
    assert_eq!(v["warning"], false);
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
