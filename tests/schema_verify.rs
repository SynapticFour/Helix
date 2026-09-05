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
        "evidence_pack",
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
    assert!(schema["$defs"]["check"]["properties"]
        .as_object()
        .unwrap()
        .contains_key("requested_version"));
    assert!(schema["$defs"]["check"]["properties"]
        .as_object()
        .unwrap()
        .contains_key("traceability"));
    assert!(schema["$defs"]["checkKind"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "normative"));
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
    assert_eq!(v["standard_selection"]["mode"], "unversioned");
    assert_eq!(
        v["executed"][0]["selected_version"],
        serde_json::Value::Null
    );
    assert_eq!(
        v["executed"][0]["verified_version"],
        serde_json::Value::Null
    );
    let kind = v["executed"][0]["traceability"]["check_kind"].as_str();
    let category = v["executed"][0]["traceability"]["category"].as_str();
    let scope = v["executed"][0]["traceability"]["claim_scope"].as_str();
    assert_eq!(kind, category);
    assert_ne!(
        kind,
        Some("normative"),
        "HelixTest wrap must not be labeled normative"
    );
    assert_ne!(kind, Some("guidance"));
    assert_ne!(scope, Some("ga4gh_requirement"));
    assert!(
        kind == Some("fixture") || kind == Some("interoperability"),
        "unexpected kind {kind:?}"
    );
    if kind == Some("fixture") {
        assert_eq!(scope, Some("helix_fixture"));
    }
    if kind == Some("interoperability") {
        assert_eq!(scope, Some("interoperability_observation"));
    }
    assert_ne!(
        v["executed"][0]["traceability"]["authority"].as_str(),
        Some("ga4gh")
    );
    assert!(v["layer_summary"]["note"]
        .as_str()
        .unwrap()
        .contains("SCHEMA PASS is not BEHAVIOR PASS"));
    assert!(v["layer_summary"].get("percent").is_none());
    assert!(v["executed"][0]["layer"].as_str().is_some());
    assert!(v.get("services").is_none());
    assert!(v.get("signature").is_none());
    assert_selected_equals_verified(&v);
    let claims = v["claims"].as_array().expect("claims array");
    assert_eq!(claims.len(), 6);
    assert_eq!(claims[0]["kind"], "ga4gh_requirement");
    assert_eq!(claims[0]["status"], "not_verified");
    assert!(
        claims.iter().all(|c| c["status"] == "not_verified"),
        "honest DRS PASS is not a VERIFIED claim: {claims:?}"
    );
    let ga4gh_blocks: Vec<&str> = claims[0]["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["code"].as_str())
        .collect();
    assert!(ga4gh_blocks.contains(&"unversioned_run"));
    assert!(ga4gh_blocks.contains(&"no_normative_checks"));
    assert!(
        !ga4gh_blocks.contains(&"normative_check_failed"),
        "fixture PASS must not produce a MUST-fail block: {ga4gh_blocks:?}"
    );
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
    let ga4gh = v["claims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "ga4gh_requirement")
        .expect("ga4gh_requirement claim");
    assert_eq!(ga4gh["status"], "not_verified");
    let codes: Vec<&str> = ga4gh["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["code"].as_str())
        .collect();
    assert!(
        !codes.contains(&"normative_check_failed"),
        "fixture FAIL is not a GA4GH MUST fail: {codes:?}"
    );
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
async fn generated_available_but_not_supported_json_validates() {
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
        .output()
        .expect("helix verify");
    let v = parse_json_stdout(&out.stdout);
    assert_eq!(
        v["standard_selection"]["selection_status"],
        "AVAILABLE_BUT_NOT_SUPPORTED"
    );
    assert_eq!(v["standard_selection"]["substituted"], false);
    assert_selected_equals_verified(&v);
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
fn helios_ro_crate_pdf_and_audit_trail_are_rejected() {
    for key in ["ro_crate", "pdf", "audit_trail", "evidence_pack"] {
        let mut v = valid_skeleton();
        v.as_object_mut()
            .unwrap()
            .insert(key.into(), serde_json::json!({"id": "no"}));
        assert_invalid(&v, key);
    }
}

#[test]
fn substituted_true_is_schema_invalid() {
    let mut v = valid_skeleton();
    v["standard_selection"] = serde_json::json!({
        "mode": "unversioned",
        "selection_status": "UNVERSIONED",
        "substituted": true
    });
    assert_invalid(&v, "substituted is const false");
}

fn assert_selected_equals_verified(v: &Value) {
    if let Some(sel) = v.get("standard_selection") {
        assert_eq!(sel["substituted"], false, "substituted must be false");
        let selected = sel.get("selected_version").unwrap_or(&Value::Null);
        let verified = sel.get("verified_version").unwrap_or(&Value::Null);
        if !verified.is_null() {
            assert_eq!(
                selected, verified,
                "run verified_version must equal selected_version when set"
            );
        }
    }
    for key in ["executed", "skipped"] {
        for row in v[key].as_array().unwrap_or(&Vec::new()) {
            let selected = row.get("selected_version").unwrap_or(&Value::Null);
            let verified = row.get("verified_version").unwrap_or(&Value::Null);
            if !verified.is_null() {
                assert_eq!(
                    selected, verified,
                    "{} {} verified_version must equal selected_version when set",
                    key, row["id"]
                );
            }
        }
    }
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

fn fixture_traceability() -> Value {
    serde_json::json!({
        "check_id": "drs.object.range",
        "category": "fixture",
        "check_kind": "fixture",
        "claim_scope": "helix_fixture",
        "authority": "helixtest",
        "expected_behavior": "HTTP 206 for Range on fixture bytes",
        "implementation": "HelixTest drs.rs",
        "layer": "behavior",
        "request": "GET access_url with Header Range: bytes=0-1023",
        "untraceable_reason": "HTTP Range is a HelixTest probe."
    })
}

fn check_with_traceability(t: Value) -> Value {
    serde_json::json!({
        "id": "drs.object.range",
        "code": "HLX-DRS-004",
        "name": "x",
        "service": "drs",
        "category": "robustness",
        "status": "pass",
        "severity": "info",
        "traceability": t
    })
}

#[test]
fn fixture_labeled_normative_is_schema_invalid() {
    let mut t = fixture_traceability();
    t["check_kind"] = Value::String("normative".into());
    t["claim_scope"] = Value::String("ga4gh_requirement".into());
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([check_with_traceability(t)]);
    v["summary"] = serde_json::json!({
        "passed": 1, "failed": 0, "skipped": 0, "errors": 0, "total": 1
    });
    assert_invalid(&v, "fixture check cannot be serialized as normative");
}

#[test]
fn fixture_claim_scope_ga4gh_requirement_is_schema_invalid() {
    let mut t = fixture_traceability();
    t["claim_scope"] = Value::String("ga4gh_requirement".into());
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([check_with_traceability(t)]);
    v["summary"] = serde_json::json!({
        "passed": 1, "failed": 0, "skipped": 0, "errors": 0, "total": 1
    });
    assert_invalid(&v, "fixture claim_scope cannot be ga4gh_requirement");
}

#[test]
fn valid_fixture_traceability_is_schema_valid() {
    let mut v = valid_skeleton();
    v["executed"] = serde_json::json!([check_with_traceability(fixture_traceability())]);
    v["summary"] = serde_json::json!({
        "passed": 1, "failed": 0, "skipped": 0, "errors": 0, "total": 1
    });
    assert_valid(&v);
}

#[test]
fn layer_summary_must_not_have_a_percentage() {
    let mut v = valid_skeleton();
    v["layer_summary"] = serde_json::json!({
        "schema": { "passed": 1, "failed": 0, "skipped": 0, "errors": 0, "total": 1 },
        "behavior": { "passed": 0, "failed": 1, "skipped": 0, "errors": 0, "total": 1 },
        "security": { "passed": 0, "failed": 0, "skipped": 0, "errors": 0, "total": 0 },
        "interoperability": { "passed": 0, "failed": 0, "skipped": 0, "errors": 0, "total": 0 },
        "note": "SCHEMA PASS is not BEHAVIOR PASS.",
        "percent": 50
    });
    assert_invalid(&v, "layer_summary must not contain percent");
}
