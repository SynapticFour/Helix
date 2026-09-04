// SPDX-License-Identifier: Apache-2.0
//! Generated `helix verify --format json` must validate against
//! `schemas/helix-verification-v1.json`. Not HELIOS. Not certification.

use std::sync::OnceLock;

use assert_cmd::Command;
use jsonschema::JSONSchema;
use serde_json::Value;
use wiremock::MockServer;

mod support;

use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_invalid_drs_object};

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn schema_json() -> Value {
    serde_json::from_str(include_str!("../schemas/helix-verification-v1.json"))
        .expect("schemas/helix-verification-v1.json")
}

fn compiled() -> &'static JSONSchema {
    static SCHEMA: OnceLock<JSONSchema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let leaked: &'static Value = Box::leak(Box::new(schema_json()));
        JSONSchema::compile(leaked).expect("helix-verification-v1 compiles")
    })
}

fn assert_valid(instance: &Value) {
    if let Err(errors) = compiled().validate(instance) {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("schema rejected JSON: {msgs:?}\n{instance}");
    }
}

fn assert_invalid(instance: &Value, why: &str) {
    assert!(
        !compiled().is_valid(instance),
        "schema should reject ({why}): {instance}"
    );
}

fn parse_json_stdout(stdout: &[u8]) -> Value {
    let text = String::from_utf8_lossy(stdout);
    let trimmed = text.trim();
    serde_json::from_str(trimmed).unwrap_or_else(|e| {
        panic!("stdout is not JSON ({e}): {text}");
    })
}

fn verify_json(url: &str) -> Value {
    let out = helix()
        .env("RUST_LOG", "error")
        .args(["verify", url, "--format", "json"])
        .output()
        .expect("helix verify");
    parse_json_stdout(&out.stdout)
}

fn closed_origin() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

fn valid_skeleton() -> Value {
    serde_json::json!({
        "schema_version": "helix-verification-v1",
        "helix_version": "0.1.0",
        "timestamp": "2026-09-04T12:00:00Z",
        "target": { "url": "http://127.0.0.1:8080" },
        "discovery": [],
        "executed": [],
        "skipped": [],
        "summary": { "passed": 0, "failed": 0, "skipped": 0, "errors": 0, "total": 0 }
    })
}

#[test]
fn schema_file_compiles_and_forbids_helios_keys() {
    let schema = schema_json();
    assert_eq!(
        schema["properties"]["schema_version"]["const"],
        "helix-verification-v1"
    );
    assert_eq!(schema["additionalProperties"], false);
    let props = schema["properties"].as_object().unwrap();
    for forbidden in [
        "signature",
        "ro_crate",
        "audit_trail",
        "evidence",
        "pdf",
        "services",
        "checks",
        "passed",
        "overall_score",
    ] {
        assert!(
            !props.contains_key(forbidden),
            "v1 must not define HELIOS/OverallReport key {forbidden}"
        );
    }
    let diag = schema["$defs"]["diagnostic"]["properties"]
        .as_object()
        .unwrap();
    assert!(!diag.contains_key("cause"));
    assert!(diag.contains_key("possible_causes"));
    assert_eq!(schema["properties"]["fixture_version"]["minLength"], 1);
    let status = schema["$defs"]["status"]["enum"].as_array().unwrap();
    assert_eq!(
        status,
        &vec![
            Value::from("pass"),
            Value::from("fail"),
            Value::from("skip"),
            Value::from("error")
        ]
    );
}

#[test]
fn empty_run_fixture_validates() {
    assert_valid(&valid_skeleton());
}

#[tokio::test]
async fn generated_pass_json_validates() {
    let mock = start_mock_ga4gh_drs().await;
    let v = verify_json(&mock.drs_url());
    assert_eq!(v["schema_version"], "helix-verification-v1");
    assert_eq!(v["fixture_version"], "helix-fixtures-v1");
    assert_eq!(v["executed"][0]["status"], "pass");
    assert!(v.get("services").is_none());
    assert!(v.get("signature").is_none());
    assert_valid(&v);
}

#[tokio::test]
async fn generated_fail_json_with_diagnostic_validates() {
    let server = start_mock_invalid_drs_object().await;
    let v = verify_json(&server.uri());
    let fail = v["executed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["status"] == "fail")
        .expect("fail row");
    assert_eq!(fail["status"], "fail");
    assert!(fail.get("failure").is_some());
    assert!(fail.get("diagnostic").is_some());
    assert!(fail["diagnostic"].get("cause").is_none());
    assert_valid(&v);
}

#[tokio::test]
async fn generated_skip_json_validates() {
    let server = MockServer::start().await;
    let v = verify_json(&server.uri());
    assert_eq!(v["skipped"][0]["status"], "skip");
    assert!(v["executed"].as_array().unwrap().is_empty());
    assert_valid(&v);
}

#[tokio::test]
async fn generated_error_json_validates() {
    let v = verify_json(&closed_origin());
    assert_eq!(v["executed"][0]["status"], "error");
    assert!(v["executed"][0].get("failure").is_some());
    assert_valid(&v);
}

#[test]
fn uppercase_pass_is_rejected() {
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([{
        "id": "drs.object.reachable",
        "code": "HLX-DRS-001",
        "name": "x",
        "service": "drs",
        "category": "robustness",
        "status": "PASS",
        "severity": "info"
    }]);
    v["summary"] = serde_json::json!({
        "passed": 1, "failed": 0, "skipped": 0, "errors": 0, "total": 1
    });
    assert_invalid(&v, "JSON status is lowercase pass, not PASS");
}

#[test]
fn skip_must_not_appear_in_executed() {
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([{
        "id": "drs.object.reachable",
        "code": "HLX-DRS-001",
        "name": "x",
        "service": "drs",
        "category": "robustness",
        "status": "skip",
        "severity": "info"
    }]);
    v["summary"] = serde_json::json!({
        "passed": 0, "failed": 0, "skipped": 1, "errors": 0, "total": 1
    });
    assert_invalid(&v, "skip belongs in skipped[], not executed[]");
}

#[test]
fn helios_signature_is_rejected() {
    let mut v = valid_skeleton();
    v.as_object_mut()
        .unwrap()
        .insert("signature".into(), Value::String("nope".into()));
    assert_invalid(&v, "HELIOS signature must not appear");
}

#[test]
fn overall_report_services_key_is_rejected() {
    let mut v = valid_skeleton();
    v.as_object_mut()
        .unwrap()
        .insert("services".into(), serde_json::json!([]));
    assert_invalid(&v, "HelixTest OverallReport services must not appear");
}

#[test]
fn diagnostic_must_not_have_cause() {
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([{
        "id": "drs.object.not_found",
        "code": "HLX-DRS-005",
        "name": "x",
        "service": "drs",
        "category": "robustness",
        "status": "fail",
        "severity": "error",
        "failure": { "code": "HLX-DRS-005" },
        "diagnostic": {
            "code": "HLX-DRS-005",
            "id": "drs.object.not_found",
            "expected": "HTTP 404",
            "observed": "HTTP 200",
            "likely_category": "error_handling",
            "hint": "x",
            "possible_causes": ["y"],
            "cause": "invented root cause"
        }
    }]);
    v["summary"] = serde_json::json!({
        "passed": 0, "failed": 1, "skipped": 0, "errors": 0, "total": 1
    });
    assert_invalid(&v, "diagnostic has possible_causes, not cause");
}

#[test]
fn evaluator_pack_example_json_validates() {
    let v: Value = serde_json::from_str(include_str!("../docs/evaluator-pack/example-verify.json"))
        .expect("docs/evaluator-pack/example-verify.json");
    assert_eq!(v["schema_version"], "helix-verification-v1");
    assert_eq!(v["fixture_version"], "helix-fixtures-v1");
    assert!(v.get("services").is_none());
    assert!(v.get("signature").is_none());
    assert_eq!(v["executed"].as_array().unwrap().len(), 5);
    assert_eq!(v["skipped"].as_array().unwrap().len(), 8);
    assert_eq!(v["summary"]["passed"], 5);
    assert_eq!(v["summary"]["skipped"], 8);
    assert_valid(&v);
}
