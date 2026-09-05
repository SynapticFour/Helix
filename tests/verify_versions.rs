// SPDX-License-Identifier: Apache-2.0
//! Version-selection cases for `helix verify --standard` / `--version` /
//! `--all-supported-versions`. Default unversioned verify is unchanged.
//! Not HELIOS. Not certification.

use assert_cmd::Command;
use helix::model::VerificationStatus;
use helix::profile::ProfileId;
use helix::standards::{
    default_registry_path, load_path, Registry, AVAILABLE_BUT_NOT_SUPPORTED, INSUFFICIENT,
    SELECTED, UNKNOWN_TO_HELIX, UNVERSIONED,
};
use helix::verify::{verify, verify_with_options, VerifyOptions, VerifySelection};
use predicates::prelude::*;
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod support;

use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_str(String::from_utf8_lossy(stdout).trim()).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): {}",
            String::from_utf8_lossy(stdout)
        );
    })
}

fn closed_origin() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

fn assert_seven_fields(row: &Value) {
    for key in [
        "standard",
        "requested_version",
        "detected_version",
        "selected_version",
        "verified_version",
        "standards_registry_entry",
        "standards_source_commit",
    ] {
        assert!(row.get(key).is_some(), "missing {key} on result: {row}");
    }
}

fn every_result(v: &Value) -> Vec<&Value> {
    let mut rows = Vec::new();
    if let Some(a) = v["executed"].as_array() {
        rows.extend(a);
    }
    if let Some(a) = v["skipped"].as_array() {
        rows.extend(a);
    }
    assert!(!rows.is_empty(), "expected check rows: {v}");
    rows
}

fn registry_with_drs_140_supported() -> Registry {
    load_path(&default_registry_path()).unwrap()
}

#[test]
fn version_without_standard_is_usage() {
    helix()
        .args(["verify", "http://127.0.0.1:9", "--version", "1.5.0"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--standard"));
}

#[test]
fn all_supported_without_standard_is_usage() {
    helix()
        .args(["verify", "http://127.0.0.1:9", "--all-supported-versions"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--standard"));
}

#[test]
fn version_and_all_supported_conflict() {
    helix()
        .args([
            "verify",
            "http://127.0.0.1:9",
            "--standard",
            "drs",
            "--version",
            "1.5.0",
            "--all-supported-versions",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn unknown_release_class_is_usage() {
    helix()
        .args([
            "verify",
            "http://127.0.0.1:9",
            "--standard",
            "drs",
            "--version",
            "1.5.0",
            "--release-class",
            "nightly",
        ])
        .assert()
        .failure()
        .code(2);
}

#[tokio::test]
async fn explicit_drs_1_5_0_is_available_but_not_supported() {
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
        .clone();
    let v = parse_json(&out.stdout);
    let sel = &v["standard_selection"];
    assert_eq!(sel["selection_status"], AVAILABLE_BUT_NOT_SUPPORTED);
    assert_eq!(sel["substituted"], false);
    assert_eq!(sel["requested_version"], "1.5.0");
    assert_eq!(sel["selected_version"], Value::Null);
    assert_eq!(sel["verified_version"], Value::Null);
    assert_eq!(sel["standards_registry_entry"], "ga4gh.drs.1.5.0");
    assert_eq!(
        sel["standards_source_commit"],
        "fe25c3953ae3398a31054d3f9f040d5e27aad517"
    );
    assert_ne!(sel["selected_version"], "1.4.0");
    assert_ne!(sel["verified_version"], "1.4.0");
    assert_eq!(v["summary"]["passed"], 0);
    for row in every_result(&v) {
        assert_seven_fields(row);
        assert_ne!(row["status"], "pass");
        if row["service"] == "drs" {
            assert_eq!(row["requested_version"], "1.5.0");
            assert_eq!(row["selected_version"], Value::Null);
            assert_eq!(row["verified_version"], Value::Null);
            assert_eq!(row["standards_registry_entry"], "ga4gh.drs.1.5.0");
        }
    }
    assert_ne!(
        sel["standards_source_commit"],
        "36145d389e0a454428d1dac5c4a30870995fdd7c"
    );
}

#[tokio::test]
async fn explicit_unknown_1_3_0_does_not_substitute() {
    let url = closed_origin();
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "verify",
                &url,
                "--standard",
                "drs",
                "--version",
                "1.3.0",
                "--format",
                "json",
            ])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    assert_eq!(
        v["standard_selection"]["selection_status"],
        UNKNOWN_TO_HELIX
    );
    assert_eq!(v["standard_selection"]["substituted"], false);
    assert_eq!(v["standard_selection"]["selected_version"], Value::Null);
    assert_eq!(v["standard_selection"]["verified_version"], Value::Null);
    assert_eq!(
        v["standard_selection"]["standards_registry_entry"],
        Value::Null
    );
    let others = v["standard_selection"]["other_rows_not_selected"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let joined = others
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(joined.contains("1.4.0"), "{joined}");
    assert!(joined.contains("1.5.0"), "{joined}");
    assert_eq!(v["summary"]["passed"], 0);
    for row in every_result(&v) {
        assert_seven_fields(row);
        if row["service"] == "drs" {
            assert_eq!(row["requested_version"], "1.3.0");
            assert_eq!(row["selected_version"], Value::Null);
        }
    }
}

#[tokio::test]
async fn explicit_unknown_9_9_9_does_not_substitute() {
    let url = closed_origin();
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "verify",
                &url,
                "--standard",
                "drs",
                "--version",
                "9.9.9",
                "--format",
                "json",
            ])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    assert_eq!(
        v["standard_selection"]["selection_status"],
        UNKNOWN_TO_HELIX
    );
    assert_eq!(v["standard_selection"]["selected_version"], Value::Null);
}

#[tokio::test]
async fn all_supported_versions_does_not_run_available_1_5_0() {
    let mock = start_mock_ga4gh_drs().await;
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "verify",
                &mock.drs_url(),
                "--standard",
                "drs",
                "--all-supported-versions",
                "--format",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(v["standard_selection"]["selection_status"], SELECTED);
    assert_eq!(v["standard_selection"]["substituted"], false);
    assert_eq!(v["standard_selection"]["selected_version"], "1.4.0");
    assert_eq!(v["standard_selection"]["verified_version"], Value::Null);
    assert_ne!(v["standard_selection"]["verified_version"], "1.5.0");
    assert_ne!(v["standard_selection"]["selected_version"], "1.5.0");
    for row in every_result(&v) {
        assert_seven_fields(row);
        if row["service"] == "drs" {
            assert_eq!(row["selected_version"], "1.4.0");
            assert_eq!(row["verified_version"], Value::Null);
            assert_ne!(row["selected_version"], "1.5.0");
        }
    }
}

#[tokio::test]
async fn development_release_class_is_rejected() {
    let url = closed_origin();
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "verify",
                &url,
                "--standard",
                "drs",
                "--version",
                "1.5.0",
                "--release-class",
                "development",
                "--format",
                "json",
            ])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    assert_eq!(
        v["standard_selection"]["selection_status"],
        "DEVELOPMENT_NOT_SELECTABLE"
    );
    assert_eq!(v["standard_selection"]["substituted"], false);
    assert_eq!(v["standard_selection"]["selected_version"], Value::Null);
}

#[tokio::test]
async fn automatic_standard_without_version_does_not_run_available_packs() {
    let mock = start_mock_ga4gh_drs().await;
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "verify",
                &mock.drs_url(),
                "--standard",
                "drs",
                "--format",
                "json",
            ])
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout,
    );
    assert_eq!(v["standard_selection"]["selection_status"], INSUFFICIENT);
    assert_eq!(v["summary"]["passed"], 0);
    for row in every_result(&v) {
        assert_seven_fields(row);
        assert_ne!(row["status"], "pass");
    }
}

#[tokio::test]
async fn default_verify_stays_unversioned_and_still_runs_drs() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify(&mock.drs_url()).await.expect("default verify");
    assert!(outcome.is_success());
    let sel = outcome.run.standard_selection.as_ref().expect("selection");
    assert_eq!(sel.mode, "unversioned");
    assert_eq!(sel.selection_status, UNVERSIONED);
    assert!(!sel.substituted);
    assert!(sel.selected_version.is_none());
    assert!(sel.verified_version.is_none());
    assert!(sel.requested_version.is_none());
    assert!(sel.standards_registry_entry.is_none());
    let drs: Vec<_> = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.service == "drs")
        .collect();
    assert_eq!(drs.len(), 5);
    for r in outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
    {
        assert_eq!(r.standard.as_deref(), Some(r.service.as_str()));
        assert!(r.requested_version.is_none());
        assert!(r.selected_version.is_none());
        assert!(r.verified_version.is_none());
        assert!(r.standards_registry_entry.is_none());
        assert!(r.standards_source_commit.is_none());
        assert_ne!(r.selected_version.as_deref(), Some("1.4.0"));
        assert_ne!(r.verified_version.as_deref(), Some("1.5.0"));
    }
}

#[tokio::test]
async fn detected_1_2_0_is_not_implied_as_requested_1_5_0() {
    let mock = start_mock_ga4gh_drs().await;
    Mock::given(method("GET"))
        .and(path("/ga4gh/drs/v1/service-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "org.ga4gh.drs",
            "name": "Example DRS",
            "version": "0.9.0",
            "type": { "group": "org.ga4gh", "artifact": "drs", "version": "1.2.0" }
        })))
        .mount(&mock.server)
        .await;

    let outcome = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            profile: ProfileId::Generic,
            selection: VerifySelection::Explicit {
                standard: "drs".into(),
                version: "1.5.0".into(),
                release_class: None,
            },
            registry: None,
            vendor_root: None,
            declared_target: helix::target::DeclaredTarget::default(),
        },
    )
    .await
    .expect("mode 1");
    assert!(!outcome.is_success());
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selection_status, AVAILABLE_BUT_NOT_SUPPORTED);
    assert_eq!(sel.requested_version.as_deref(), Some("1.5.0"));
    assert_eq!(sel.detected_version.as_deref(), Some("1.2.0"));
    assert!(sel.selected_version.is_none());
    assert_ne!(sel.detected_version, sel.requested_version);
    assert_ne!(sel.detected_version.as_deref(), Some("1.5.0"));
    for r in outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
        .filter(|r| r.service == "drs")
    {
        assert_eq!(r.status, VerificationStatus::Skip);
        assert_eq!(r.requested_version.as_deref(), Some("1.5.0"));
        assert_eq!(r.detected_version.as_deref(), Some("1.2.0"));
        assert!(r.selected_version.is_none());
        assert!(r.verified_version.is_none());
    }
}

#[tokio::test]
async fn supported_1_4_0_runs_and_does_not_label_1_5_0() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            profile: ProfileId::Generic,
            selection: VerifySelection::Explicit {
                standard: "drs".into(),
                version: "1.4.0".into(),
                release_class: None,
            },
            registry: Some(registry_with_drs_140_supported()),
            vendor_root: None,
            declared_target: helix::target::DeclaredTarget::default(),
        },
    )
    .await
    .expect("supported 1.4.0");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selection_status, "SELECTED");
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    assert_eq!(sel.integrity_ok, Some(true));
    assert!(sel.pack_integrity_sha256.as_ref().unwrap().len() == 64);
    assert!(sel.schema_document_sha256.as_ref().unwrap().len() == 64);
    assert!(sel.schema_component_sha256.as_ref().unwrap().len() == 64);
    assert!(sel.execution_id.as_ref().unwrap().len() == 64);
    assert_eq!(sel.requested_version.as_deref(), Some("1.4.0"));
    assert_eq!(
        sel.standards_registry_entry.as_deref(),
        Some("ga4gh.drs.1.4.0")
    );
    assert!(!sel.substituted);
    assert_ne!(sel.selected_version.as_deref(), Some("1.5.0"));
    let drs_pass = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.service == "drs" && r.status == VerificationStatus::Pass)
        .count();
    assert!(drs_pass > 0, "supported pack should execute DRS checks");
    for r in outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
        .filter(|r| r.service == "drs")
    {
        assert_eq!(r.selected_version.as_deref(), Some("1.4.0"));
        assert!(r.verified_version.is_none());
    }
    for r in outcome.run.skipped.iter().filter(|r| r.service == "wes") {
        assert_eq!(r.selected_version, None);
        assert!(r
            .message
            .as_deref()
            .unwrap_or("")
            .contains("standard not selected"));
    }
}

#[tokio::test]
async fn requested_1_5_0_does_not_run_when_only_1_4_0_is_supported() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            profile: ProfileId::Generic,
            selection: VerifySelection::Explicit {
                standard: "drs".into(),
                version: "1.5.0".into(),
                release_class: None,
            },
            registry: Some(registry_with_drs_140_supported()),
            vendor_root: None,
            declared_target: helix::target::DeclaredTarget::default(),
        },
    )
    .await
    .expect("no downgrade");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selection_status, AVAILABLE_BUT_NOT_SUPPORTED);
    assert_eq!(sel.selected_version, None);
    assert_eq!(sel.verified_version, None);
    assert_eq!(
        sel.standards_registry_entry.as_deref(),
        Some("ga4gh.drs.1.5.0")
    );
    assert_eq!(outcome.run.summary.passed, 0);
}

#[tokio::test]
async fn all_supported_with_one_supported_pack_does_not_also_run_1_5_0() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            profile: ProfileId::Generic,
            selection: VerifySelection::Compatibility {
                standard: "drs".into(),
            },
            registry: Some(registry_with_drs_140_supported()),
            vendor_root: None,
            declared_target: helix::target::DeclaredTarget::default(),
        },
    )
    .await
    .expect("mode 3 n=1");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert_ne!(sel.selected_version.as_deref(), Some("1.5.0"));
    assert_ne!(sel.verified_version.as_deref(), Some("1.5.0"));
}
