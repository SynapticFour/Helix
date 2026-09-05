// SPDX-License-Identifier: Apache-2.0
//! Interop matrix CLI and schema. Fixtures are not independent evidence.
//! Not certification. Not HELIOS.

use std::sync::OnceLock;

use assert_cmd::Command;
use jsonschema::JSONSchema;
use predicates::prelude::*;
use serde_json::Value;

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn schema_json() -> Value {
    serde_json::from_str(include_str!("../schemas/helix-interop-matrix-v1.json"))
        .expect("helix-interop-matrix-v1.json")
}

fn compiled() -> &'static JSONSchema {
    static SCHEMA: OnceLock<JSONSchema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let leaked: &'static Value = Box::leak(Box::new(schema_json()));
        JSONSchema::compile(leaked).expect("interop matrix schema compiles")
    })
}

fn assert_valid(instance: &Value) {
    if let Err(errors) = compiled().validate(instance) {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("schema rejected JSON: {msgs:?}\n{instance}");
    }
}

#[test]
fn pending_matrix_json_validates_and_is_not_independent_evidence() {
    let out = helix()
        .args(["matrix", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("matrix JSON");
    assert_eq!(v["schema_version"], "helix-interop-matrix-v1");
    assert_eq!(v["independent_evidence"], false);
    assert_eq!(v["external_validation"], "pending");
    assert!(v["note"]
        .as_str()
        .unwrap()
        .contains("not independent evidence"));
    let impls = v["implementations"].as_array().unwrap();
    assert!(impls
        .iter()
        .any(|s| s["id"] == "ferrum" && s["status"] == "pending"));
    assert!(impls
        .iter()
        .any(|s| s["id"] == "independent" && s["status"] == "pending"));
    for row in v["rows"].as_array().unwrap() {
        assert_eq!(row["result"], "pending");
        assert!(row.get("standard").is_some());
        assert!(row.get("check").is_some());
        assert!(row.get("implementation").is_some());
        assert!(row.get("expected").is_some());
    }
    assert!(v.get("signature").is_none());
    assert!(v.get("ro_crate").is_none());
    assert_valid(&v);
}

#[test]
fn pending_matrix_text_says_pending() {
    let out = helix()
        .args(["matrix", "--format", "text"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("HELIX INTEROP MATRIX"));
    assert!(text.contains("PENDING"));
    assert!(text.contains("not independent evidence"));
    assert!(text.contains("It is not GA4GH certification."));
    assert!(text.contains("standard  version  check  implementation  expected  observed  result"));
    assert!(!text.to_lowercase().contains("validation complete"));
}

#[test]
fn kind_without_run_is_usage_or_error() {
    helix()
        .args(["matrix", "--kind", "ferrum=reference_target"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("--kind"));
}

#[test]
fn matrix_is_not_a_reserved_stub() {
    helix()
        .args(["matrix", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--run"));
}
