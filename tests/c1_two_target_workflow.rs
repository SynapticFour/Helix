// SPDX-License-Identifier: Apache-2.0
//! C1 — canonical two-target operator workflow (discoverability).
//!
//! Does not expand DRS coverage, authorization, HELIOS, or identities.

use assert_cmd::Command;
use helix::compare::parse_verification_run;
use helix::evidence::{classify_evidence, EvidenceStanding};
use helix::live_evidence::PINNED_EXECUTION_ID;
use helix::model::{VerificationRun, SCHEMA_VERSION};
use helix::report::verify_json;
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use predicates::prelude::*;
use serde_json::Value;
use std::path::PathBuf;

mod support;
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static C1_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap()
}

fn versioned(declared: DeclaredTarget) -> VerifyOptions {
    VerifyOptions {
        profile: helix::profile::ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: declared,
        ..Default::default()
    }
}

fn live_b12(name: &str) -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p).join(name);
    }
    root().join("local/b12").join(name)
}

#[test]
fn c1_t1_help_exposes_two_target_workflow() {
    let root_help =
        String::from_utf8_lossy(&helix().arg("--help").assert().success().get_output().stdout)
            .into_owned();
    assert!(root_help.contains("docs/INDEPENDENT_DRS.md"), "{root_help}");
    assert!(root_help.contains("does not rank"), "{root_help}");
    assert!(root_help.contains("inspect"), "{root_help}");
    assert!(root_help.contains("differential"), "{root_help}");

    let verify = String::from_utf8_lossy(
        &helix()
            .args(["verify", "--help"])
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .into_owned();
    assert!(verify.contains("--standard"), "{verify}");
    assert!(verify.contains("1.4.0"), "{verify}");
    assert!(verify.contains("--output"), "{verify}");
    assert!(verify.contains("--drs-object-id"), "{verify}");
    assert!(verify.contains("--target-kind"), "{verify}");
    assert!(verify.contains("docs/INDEPENDENT_DRS.md"), "{verify}");
    assert!(
        verify.contains("never becomes verified_version"),
        "{verify}"
    );
    assert!(verify.contains("helix differential"), "{verify}");

    let inspect = String::from_utf8_lossy(
        &helix()
            .args(["inspect", "--help"])
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .into_owned();
    assert!(
        inspect.to_ascii_lowercase().contains("does not rewrite"),
        "{inspect}"
    );
    assert!(inspect.contains("local/b12"), "{inspect}");

    let diff = String::from_utf8_lossy(
        &helix()
            .args(["differential", "--help"])
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .into_owned();
    assert!(diff.contains("does not rank"), "{diff}");
    assert!(diff.contains("does not declare a winner"), "{diff}");
    assert!(diff.contains("execution_id"), "{diff}");
    assert!(diff.contains("target_execution_id"), "{diff}");
    assert!(diff.contains("docs/INDEPENDENT_DRS.md"), "{diff}");
}

#[test]
fn c1_t2_operator_docs_match_cli_syntax() {
    let doc = read("docs/INDEPENDENT_DRS.md");
    for needle in [
        "helix verify TARGET",
        "--standard drs",
        "--version 1.4.0",
        "--output FILE",
        "--drs-object-id",
        "--target-kind real-independent-local-implementation",
        "helix inspect starter-kit.json",
        "helix inspect bento.json",
        "helix differential starter-kit.json bento.json",
        "OPTIONAL LIVE VERIFICATION",
        "make verify-independent",
        "does not rewrite",
        "historical",
        "NOT VERIFIED",
        "UNEVALUATED",
        "DEFERRED",
        "does **not**:",
        "rank implementations",
    ] {
        assert!(doc.contains(needle), "INDEPENDENT_DRS.md missing {needle}");
    }
    assert!(!doc.to_ascii_lowercase().contains("overall score"), "{doc}");
    assert!(!doc.contains("better implementation"), "{doc}");

    let op = read("docs/OPERATOR_VERIFY.md");
    assert!(op.contains("docs/INDEPENDENT_DRS.md") || op.contains("INDEPENDENT_DRS.md"));

    let mk = read("Makefile");
    assert!(mk.contains("verify-independent"));
    assert!(mk.contains("OPTIONAL LIVE"));
    let prove = mk
        .lines()
        .skip_while(|l| !l.starts_with("prove:"))
        .take_while(|l| !l.starts_with("test:"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !prove.contains("verify-independent"),
        "make prove must not invoke live independent verification\n{prove}"
    );
}

#[test]
fn c1_t3_helper_fails_clearly_without_live_targets() {
    let script = read("scripts/verify-independent-drs.sh");
    assert!(script.contains("OPTIONAL LIVE VERIFICATION"));
    assert!(
        !script.contains("docker pull"),
        "helper must not docker pull"
    );
    assert!(script.contains("local/b12"));
    assert!(script.contains("must not be local/b12"));

    let out = Command::new("bash")
        .arg(root().join("scripts/verify-independent-drs.sh"))
        .env_remove("HELIX_BENTO_OBJECT_ID")
        .env("HELIX_STARTER_KIT_URL", "http://127.0.0.1:1")
        .env("HELIX_BENTO_URL", "http://127.0.0.1:1")
        .output()
        .expect("run helper");
    assert_eq!(
        out.status.code(),
        Some(2),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OPTIONAL LIVE VERIFICATION"), "{err}");
    assert!(err.contains("HELIX_BENTO_OBJECT_ID"), "{err}");
    assert!(!err.to_ascii_lowercase().contains("docker pull"), "{err}");
}

#[test]
fn c1_t4_helper_refuses_b12_output_dir() {
    let out = Command::new("bash")
        .arg(root().join("scripts/verify-independent-drs.sh"))
        .env(
            "HELIX_BENTO_OBJECT_ID",
            "00000000-0000-0000-0000-000000000000",
        )
        .env("HELIX_INDEPENDENT_OUT", root().join("local/b12"))
        .output()
        .expect("run helper");
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("must not be local/b12"), "{err}");
}

#[tokio::test]
async fn c1_t5_verify_output_inspect_differential_on_fixtures() {
    let _g = C1_LOCK.lock().await;
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("starter-kit.json");
    let b = dir.path().join("bento.json");

    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock_a.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--target-id",
            "c1-fixture-a",
            "--target-kind",
            "mock",
            "--drs-object-id",
            "test-object-1",
            "--output",
            a.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Wrote helix-verification-v1"));

    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock_b.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--target-id",
            "c1-fixture-b",
            "--target-kind",
            "mock",
            "--drs-object-id",
            "test-object-1",
            "--output",
            b.to_str().unwrap(),
        ])
        .assert()
        .success();

    let raw_a = std::fs::read(&a).unwrap();
    let raw_b = std::fs::read(&b).unwrap();
    let va: Value = serde_json::from_str(std::str::from_utf8(&raw_a).unwrap()).unwrap();
    assert_eq!(va["schema_version"], SCHEMA_VERSION);
    assert_eq!(
        va["standard_selection"]["execution_id"].as_str(),
        Some(PINNED_EXECUTION_ID)
    );

    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", a.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("HELIX EVIDENCE INSPECT"));
    assert_eq!(
        raw_a,
        std::fs::read(&a).unwrap(),
        "inspect must not rewrite"
    );
    assert_eq!(raw_b, std::fs::read(&b).unwrap());

    let diff = helix()
        .env("NO_COLOR", "1")
        .args(["differential", a.to_str().unwrap(), b.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ranking_semantics: absent"))
        .stdout(predicate::str::contains("creates_verification: no"))
        .get_output()
        .stdout
        .clone();
    let d = String::from_utf8_lossy(&diff);
    assert!(!d.to_ascii_lowercase().contains("winner"), "{d}");
    assert!(!d.to_ascii_lowercase().contains("overall score"), "{d}");
    assert!(d.contains("execution_id"), "{d}");

    let json = helix()
        .env("NO_COLOR", "1")
        .args([
            "differential",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let dv: Value = serde_json::from_str(std::str::from_utf8(&json).unwrap()).unwrap();
    assert_eq!(dv["schema_version"], "helix-differential-v1");
    assert_eq!(dv["ranking_semantics"], "absent");
    assert_eq!(dv["creates_verification"], false);
    let targets = dv["targets"].as_array().expect("targets");
    assert_eq!(targets.len(), 2);
    let exec_a = targets[0]["execution_id"].as_str();
    let exec_b = targets[1]["execution_id"].as_str();
    assert_eq!(exec_a, Some(PINNED_EXECUTION_ID));
    assert_eq!(exec_a, exec_b);
    let te_a = targets[0]["target_execution_id"].as_str().unwrap();
    let te_b = targets[1]["target_execution_id"].as_str().unwrap();
    assert_ne!(te_a, te_b);
}

#[test]
fn c1_t6_historical_b12_is_not_restamped() {
    let names = ["starter-kit.json", "bento.json"];
    let mut before = Vec::new();
    for name in names {
        let p = live_b12(name);
        if p.is_file() {
            before.push((p.clone(), std::fs::read(&p).unwrap()));
        }
    }
    // Workflow tests above never write these paths. Re-read.
    for (p, bytes) in &before {
        assert_eq!(
            std::fs::read(p).unwrap(),
            *bytes,
            "{} must not be restamped",
            p.display()
        );
        let run: VerificationRun = parse_verification_run(std::str::from_utf8(bytes).unwrap())
            .unwrap_or_else(|_| serde_json::from_slice(bytes).unwrap());
        assert_eq!(
            classify_evidence(&run),
            EvidenceStanding::HistoricalObservation
        );
    }
}

#[tokio::test]
async fn c1_t7_mock_is_not_independent_evidence() {
    let _g = C1_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_with_options(
        &mock.drs_url(),
        versioned(DeclaredTarget {
            target_id: Some("ga4gh-starter-kit-drs-0.3.2".into()),
            kind: TargetKind::RealIndependentLocalImplementation,
            implementation_name: Some("ga4gh-starter-kit-drs".into()),
            implementation_version: Some("0.3.2".into()),
            ..DeclaredTarget::default()
        }),
    )
    .await
    .unwrap()
    .run;
    assert!(
        !helix::live_evidence::live_independent_observation(&run),
        "operator --target-kind must not turn the in-process mock into independent evidence"
    );
    let json = verify_json(&run).unwrap();
    assert!(json.contains(SCHEMA_VERSION));
}
