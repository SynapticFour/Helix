// SPDX-License-Identifier: Apache-2.0
//! P2 first usable Helix release — operator workflow, not a new verification gate.
//!
//! Does not expand DRS coverage, authorization, HELIOS, or identities.

use assert_cmd::Command;
use helix::claim_integrity::finalize_run;
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::compare::parse_verification_run;
use helix::coverage::CoverageState;
use helix::evidence::{classify_evidence, EvidenceStanding};
use helix::live_evidence::{PINNED_EXECUTION_ID, PINNED_PACK_INTEGRITY_SHA256};
use helix::model::{VerificationRun, VerificationStatus, SCHEMA_VERSION};
use helix::profile::ProfileId;
use helix::report::{format_verify_text, verify_json};
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use predicates::prelude::*;
use serde_json::Value;
use std::path::PathBuf;

mod support;
use support::mock_ga4gh_drs::{
    mount_ga4gh_drs_service_info, start_mock_ga4gh_drs, start_mock_ga4gh_drs_advertising,
};

static P2_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn versioned(declared: DeclaredTarget) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: declared,
        ..Default::default()
    }
}

fn mock_declared(id: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::Mock,
        implementation_name: Some(id.into()),
        implementation_version: Some("0.0.0".into()),
        ..DeclaredTarget::default()
    }
}

async fn versioned_mock(id: &str) -> helix::verify::VerifyOutcome {
    let mock = start_mock_ga4gh_drs().await;
    verify_with_options(&mock.drs_url(), versioned(mock_declared(id)))
        .await
        .expect("versioned DRS 1.4.0")
}

fn live_dir() -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local/b12")
}

fn load_live(name: &str) -> Option<VerificationRun> {
    serde_json::from_str(&std::fs::read_to_string(live_dir().join(name)).ok()?).ok()
}

fn text(run: &VerificationRun) -> String {
    format_verify_text(run, false)
}

#[test]
fn p2_t1_fresh_operator_discovers_supported_workflow() {
    let v = String::from_utf8_lossy(
        &helix()
            .args(["verify", "--help"])
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .into_owned();
    assert!(v.contains("--standard"), "{v}");
    assert!(v.contains("1.4.0"), "{v}");
    assert!(v.contains("--output"), "{v}");
    assert!(v.contains("helix inspect"), "{v}");
    let root =
        String::from_utf8_lossy(&helix().arg("--help").assert().success().get_output().stdout)
            .into_owned();
    assert!(root.contains("inspect"), "{root}");
    let ver = String::from_utf8_lossy(
        &helix()
            .arg("--version")
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .into_owned();
    assert!(ver.contains(env!("CARGO_PKG_VERSION")), "{ver}");
    assert!(ver.contains("HelixTest pin"), "{ver}");
    assert!(ver.contains("Not GA4GH certification"), "{ver}");
}

#[tokio::test]
async fn p2_t2_supported_drs_140_invocation_succeeds() {
    let _g = P2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("selected: 1.4.0"))
        .stdout(predicate::str::contains("HELIX VERIFICATION"));
}

#[tokio::test]
async fn p2_t3_human_output_identifies_target_and_version() {
    let _g = P2_LOCK.lock().await;
    let t = text(&versioned_mock("p2-t3").await.run);
    assert!(t.contains("target_id: p2-t3"), "{t}");
    assert!(t.contains("selected: 1.4.0"), "{t}");
    assert!(t.contains("standard: drs"), "{t}");
}

#[tokio::test]
async fn p2_t4_pass_distinct_from_verified() {
    let _g = P2_LOCK.lock().await;
    let t = text(&versioned_mock("p2-t4").await.run);
    assert!(t.contains("PASS is a check outcome"), "{t}");
    assert!(t.contains("ga4gh_requirement"), "{t}");
    assert!(
        t.contains("check_outcome PASS is not ga4gh_requirement VERIFIED"),
        "{t}"
    );
}

#[tokio::test]
async fn p2_t5_partial_coverage_visible() {
    let _g = P2_LOCK.lock().await;
    let run = versioned_mock("p2-t5").await.run;
    let t = text(&run);
    assert!(t.contains("state: partial"), "{t}");
    assert!(t.contains("unevaluated:"), "{t}");
    assert_eq!(run.coverage.as_ref().unwrap().state, CoverageState::Partial);
}

#[tokio::test]
async fn p2_t6_fail_skip_attribution_visible() {
    let _g = P2_LOCK.lock().await;
    let mut run = versioned_mock("p2-t6").await.run;
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        if r.id == "drs.object.schema" {
            r.status = VerificationStatus::Fail;
            helix::target::attach_attribution(r);
        }
        if r.id == "drs.object.checksum" {
            r.status = VerificationStatus::Skip;
            r.message = Some("fixture_unavailable: object not present".into());
            helix::target::attach_attribution(r);
        }
    }
    let t = text(&run);
    assert!(t.contains("attribution: target_failure"), "{t}");
    assert!(t.contains("fixture_unavailable"), "{t}");
    assert!(
        t.contains("attribution: target_configuration_failure"),
        "{t}"
    );
}

#[tokio::test]
async fn p2_t7_json_evidence_emitted() {
    let _g = P2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--output",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Wrote helix-verification-v1"));
    let raw = std::fs::read_to_string(&path).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
    assert_eq!(
        v["standard_selection"]["execution_id"].as_str(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_eq!(
        v["standard_selection"]["pack_integrity_sha256"].as_str(),
        Some(PINNED_PACK_INTEGRITY_SHA256)
    );
}

#[tokio::test]
async fn p2_t8_json_reloads() {
    let _g = P2_LOCK.lock().await;
    let run = versioned_mock("p2-t8").await.run;
    let json = verify_json(&run).unwrap();
    let back = parse_verification_run(&json).unwrap();
    assert_eq!(
        back.standard_selection
            .as_ref()
            .unwrap()
            .selected_version
            .as_deref(),
        Some("1.4.0")
    );
}

#[tokio::test]
async fn p2_t9_inspect_does_not_mutate_evidence() {
    let _g = P2_LOCK.lock().await;
    let run = versioned_mock("p2-t9").await.run;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    let json = verify_json(&run).unwrap();
    std::fs::write(&path, &json).unwrap();
    let before = std::fs::read(&path).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .success();
    let after = std::fs::read(&path).unwrap();
    assert_eq!(before, after, "helix inspect must not rewrite the file");
}

#[tokio::test]
async fn p2_t10_historical_evidence_identified() {
    let _g = P2_LOCK.lock().await;
    let mut run = versioned_mock("p2-t10").await.run;
    run.helix_git_sha = Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
    run.helix_git_dirty = Some(false);
    finalize_run(&mut run);
    assert_eq!(
        classify_evidence(&run),
        EvidenceStanding::HistoricalObservation
    );
    let t = helix::report::format_inspect_text(&run);
    assert!(t.contains("historical_observation"), "{t}");
    assert!(t.contains("current_verifier_evidence: no"), "{t}");
}

#[tokio::test]
async fn p2_t11_unsupported_version_fails_closed() {
    let _g = P2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    helix()
        .env("NO_COLOR", "1")
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
        .stdout(predicate::str::contains("AVAILABLE_BUT_NOT_SUPPORTED"))
        .stdout(predicate::str::contains("\"substituted\": false"))
        .stdout(predicate::str::contains("1.5.0"));
}

#[tokio::test]
async fn p2_t12_unavailable_target_fails_clearly() {
    let _g = P2_LOCK.lock().await;
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            "http://127.0.0.1:1",
            "--standard",
            "drs",
            "--version",
            "1.4.0",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("target unreachable"))
        .stdout(predicate::str::contains("NOT_VERIFIED"));
}

#[tokio::test]
async fn p2_t13_fixture_unavailable_is_skip_not_pass() {
    let _g = P2_LOCK.lock().await;
    // DRS must still be DETECTED (service-info). A 404 on the configured object is
    // fixture_unavailable, not "DRS not present" / unsupported_test.
    let mock = start_mock_ga4gh_drs().await;
    mount_ga4gh_drs_service_info(&mock.server).await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--drs-object-id",
            "missing-object-does-not-exist",
            "--output",
            path.to_str().unwrap(),
        ])
        .assert()
        .code(predicate::in_iter([0, 1]));
    let run: VerificationRun =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let schema = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == "drs.object.schema.openapi")
        .expect("schema.openapi row");
    assert_eq!(schema.status, VerificationStatus::Skip);
    assert_ne!(schema.status, VerificationStatus::Pass);
    assert_eq!(
        schema.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );
    let msg = schema.message.as_deref().unwrap_or("");
    assert!(msg.contains("fixture_unavailable"), "{msg}");
    let checksum = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == "drs.object.checksum");
    if let Some(checksum) = checksum {
        assert_ne!(checksum.status, VerificationStatus::Pass);
        if checksum.status == VerificationStatus::Skip {
            assert_eq!(
                checksum.attribution,
                Some(FailureAttribution::TargetConfigurationFailure)
            );
        }
    }
    let t = text(&run);
    assert!(t.contains("fixture_unavailable"), "{t}");
    assert!(t.contains("Skip is never pass"), "{t}");
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[test]
fn p2_t14_starter_kit_remains_not_verified() {
    let Some(sk) = load_live("starter-kit.json") else {
        return;
    };
    assert_eq!(
        evaluate(&sk).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    let t = text(&sk);
    assert!(t.contains("NOT_VERIFIED"), "{t}");
    assert!(t.contains("1.3.0experimental"), "{t}");
}

#[test]
fn p2_t15_bento_remains_verified_partial() {
    let Some(bento) = load_live("bento.json") else {
        return;
    };
    assert_eq!(
        evaluate(&bento).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
    assert_eq!(
        bento.coverage.as_ref().unwrap().state,
        CoverageState::Partial
    );
}

#[tokio::test]
async fn p2_t16_no_certification_claim() {
    let _g = P2_LOCK.lock().await;
    let t = text(&versioned_mock("p2-t16").await.run);
    let lower = t.to_ascii_lowercase();
    assert!(lower.contains("not ga4gh certification"), "{t}");
    assert!(!lower.contains("ga4gh certified"));
}

#[tokio::test]
async fn p2_t17_no_authorization_verification() {
    let _g = P2_LOCK.lock().await;
    let run = versioned_mock("p2-t17").await.run;
    let t = text(&run);
    assert!(t.contains("drs.security.authorization"), "{t}");
    assert!(t.contains("unevaluated"), "{t}");
    let json: Value = serde_json::from_str(&verify_json(&run).unwrap()).unwrap();
    let uneval = json["coverage"]["unevaluated"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    assert!(uneval.contains(&"drs.security.authorization"));
}

#[tokio::test]
async fn p2_t18_documented_cli_workflow_in_temp_dir() {
    let _g = P2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .current_dir(dir.path())
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--output",
            path.file_name().unwrap().to_str().unwrap(),
        ])
        .assert()
        .success();
    let json = std::fs::read_to_string(&path).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("HELIX EVIDENCE INSPECT"));
    assert!(json.contains(SCHEMA_VERSION));
}

#[tokio::test]
async fn p2_detected_not_confused_with_selected() {
    let _g = P2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs_advertising("1.3.0experimental").await;
    let run = verify_with_options(&mock.drs_url(), versioned(mock_declared("p2-det")))
        .await
        .unwrap()
        .run;
    let t = text(&run);
    assert!(t.contains("detected: 1.3.0experimental"), "{t}");
    assert!(t.contains("selected: 1.4.0"), "{t}");
}

#[test]
fn p2_makefile_exposes_verify_drs() {
    let mk = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
        .unwrap();
    assert!(mk.contains("verify-drs"));
    let op = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/OPERATOR_VERIFY.md"),
    )
    .unwrap();
    assert!(op.contains("make verify-drs"));
    assert!(op.contains("--output verify.json"));
}
