// SPDX-License-Identifier: Apache-2.0
//! `helix standards` CLI. Provenance only. Does not change `helix verify`.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_str(String::from_utf8_lossy(stdout).trim()).unwrap()
}

#[test]
fn list_includes_provenance_and_empty_default_discovery() {
    let out = helix()
        .args(["standards", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("OFFICIAL"));
    assert!(text.contains("ga4gh.drs.1.4.0"));
    assert!(text.contains("ga4gh.drs.1.5.0"));
    assert!(text.contains("36145d389e0a454428d1dac5c4a30870995fdd7c"));
    assert!(text.contains("fe25c3953ae3398a31054d3f9f040d5e27aad517"));
    assert!(text.contains("https://github.com/ga4gh/data-repository-service-schemas"));
    assert!(text.contains("sha256"));
    assert!(text.contains("does not download GA4GH files at runtime"));
}

#[test]
fn list_json_official_supported_is_drs_1_4_0() {
    let v = parse_json(
        &helix()
            .args(["standards", "list", "--format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(v["schema_version"], "helix-standards-registry-v1");
    assert_eq!(
        v["official_supported"],
        serde_json::json!(["ga4gh.drs.1.4.0"])
    );
    assert_eq!(v["substituted"], false);
    assert!(v.get("signature").is_none());
}

#[test]
fn show_drs_1_5_0_is_available_not_supported() {
    let out = helix()
        .args(["standards", "show", "drs", "1.5.0"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains("available_not_supported") || text.contains("support_status:  available")
    );
    assert!(text.contains("supported:       no") || text.contains("supported:    no"));
    assert!(text.contains("substituted:  no"));
    assert!(text.contains("fe25c3953ae3398a31054d3f9f040d5e27aad517"));
    assert!(text.contains("did not substitute"));
    assert!(!text.contains("36145d389e0a454428d1dac5c4a30870995fdd7c"));
}

#[test]
fn show_unknown_version_does_not_substitute() {
    let out = helix()
        .args(["standards", "show", "drs", "1.3.0"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("unknown_to_helix"));
    assert!(text.contains("did not substitute"));
    assert!(text.contains("not selected"));
}

#[test]
fn show_json_unknown_substituted_false() {
    let v = parse_json(
        &helix()
            .args(["standards", "show", "drs", "9.9.9", "--format", "json"])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    assert_eq!(v["result"], "unknown_to_helix");
    assert_eq!(v["substituted"], false);
    assert_eq!(v["supported"], false);
    assert!(v["record"].is_null());
}

#[test]
fn validate_shipped_registry() {
    let out = helix()
        .args(["standards", "validate"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("ok"));
    assert!(text.contains("fetched: no"));
    assert!(text.contains("official_supported: 1"));
}

#[test]
fn validate_missing_commit() {
    let root = std::env::temp_dir().join(format!("helix-std-cli-miss-{}", std::process::id()));
    let standards = root.join("standards");
    std::fs::create_dir_all(&standards).unwrap();
    let yaml = r#"
schema_version: helix-standards-registry-v1
versions:
  - schema_version: helix-standard-version-v1
    pack_id: ga4gh.drs.1.4.0
    standard: drs
    product: Data Repository Service
    version: "1.4.0"
    release_class: official
    support_status: available
    repository: https://github.com/ga4gh/data-repository-service-schemas
    release_ref: "1.4.0"
    retrieved_at: "2026-09-04"
    normative_sources:
      - path: openapi.yaml
        source_url: https://example.com/openapi.yaml
        role: openapi
        integrity:
          algorithm: sha256
          hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
"#;
    let path = standards.join("registry.yaml");
    std::fs::write(&path, yaml).unwrap();
    helix()
        .args(["standards", "validate", path.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("commit").or(predicate::str::contains("missing")));
}

#[test]
fn standards_without_subcommand_is_usage() {
    helix().args(["standards"]).assert().failure().code(2);
}

#[test]
fn trace_drs_openapi_is_normative() {
    let out = helix()
        .args(["standards", "trace", "drs.object.schema.openapi"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("HELIX TRACEABILITY"));
    assert!(text.contains("category:            normative"));
    assert!(text.contains("claim_scope:         ga4gh_requirement"));
    assert!(text.contains("authority:           ga4gh"));
    assert!(text.contains("DrsObject"));
    assert!(text.contains("36145d389e0a454428d1dac5c4a30870995fdd7c"));
    assert!(text.contains("catalog_id:"));
    assert!(!text.contains("GA4GH certified"));
}

#[test]
fn trace_drs_schema_is_not_normative() {
    let out = helix()
        .args(["standards", "trace", "drs.object.schema"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("HELIX TRACEABILITY"));
    assert!(text.contains("not GA4GH certification"));
    assert!(text.contains("layer:               schema"));
    assert!(text.contains("category:            fixture"));
    assert!(text.contains("check_kind:          fixture"));
    assert!(text.contains("claim_scope:         helix_fixture"));
    assert!(text.contains("authority:           helixtest"));
    assert!(text.contains("conformance_claim:    no"));
    assert!(text.contains("ga4gh.drs.1.4.0"));
    assert!(text.contains("/objects/{object_id}"));
    assert!(text.contains("not a MUST"));
    assert!(text.contains("helix standards show drs 1.4.0"));
}

#[test]
fn trace_json_related_source_is_not_verified_against() {
    let v = parse_json(
        &helix()
            .args(["standards", "trace", "drs.object.range", "--format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(v["certification"], false);
    assert_eq!(v["traceability"]["category"], "fixture");
    assert_eq!(v["traceability"]["check_kind"], "fixture");
    assert_eq!(v["traceability"]["claim_scope"], "helix_fixture");
    assert_eq!(v["traceability"]["authority"], "helixtest");
    assert!(v["traceability"]["untraceable_reason"].as_str().is_some());
    assert!(
        v["traceability"]["related_source"].is_null()
            || v["traceability"].get("related_source").is_none()
    );
}

#[test]
fn trace_unknown_id_fails() {
    helix()
        .args(["standards", "trace", "not.a.check"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("unknown Helix check id"));
}
