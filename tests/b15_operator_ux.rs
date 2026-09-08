// SPDX-License-Identifier: Apache-2.0
//! B15 operator verification UX and evidence contract.
//!
//! Presentation and inspectability only. Does not expand DRS coverage,
//! authorization, HELIOS, or verification identities. Not certification.

use assert_cmd::Command;
use helix::claim_integrity::{tamper_selection, validate_artifact_consistency};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::compare::parse_verification_run;
use helix::coverage::CoverageState;
use helix::evidence::{classify_evidence, EvidenceStanding};
use helix::live_evidence::{
    PINNED_BINDING_ID, PINNED_CATALOG_ID, PINNED_CHECKER_ID, PINNED_COVERAGE_ID,
    PINNED_EXECUTION_ID, PINNED_PACK_INTEGRITY_SHA256,
};
use helix::model::{VerificationRun, VerificationStatus};
use helix::profile::ProfileId;
use helix::provenance::cites_this_verifier_build;
use helix::report::{format_inspect_text, format_verify_text, verify_json};
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify, verify_with_options, VerifyOptions, VerifySelection};
use predicates::prelude::*;
use serde_json::Value;
use std::path::PathBuf;

mod support;
use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_ga4gh_drs_advertising};

static B15_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn inspect(run: &VerificationRun) -> String {
    format_inspect_text(run)
}

fn set_status(run: &mut VerificationRun, id: &str, status: VerificationStatus) {
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        if r.id == id {
            r.status = status;
            helix::target::attach_attribution(r);
        }
    }
    run.recompute_summary();
}

#[test]
fn b15_t1_help_exposes_operator_workflow() {
    let verify_help = helix()
        .args(["verify", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v = String::from_utf8_lossy(&verify_help);
    assert!(v.contains("--standard"), "{v}");
    assert!(v.contains("--version"), "{v}");
    assert!(v.contains("--format"), "{v}");
    assert!(v.contains("--drs-object-id"), "{v}");
    assert!(v.contains("--target-id"), "{v}");
    assert!(v.contains("--target-kind"), "{v}");
    assert!(
        v.contains("PASS is a check outcome") && v.contains("VERIFIED"),
        "{v}"
    );
    assert!(v.contains("helix inspect"), "{v}");
    let root = helix()
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let h = String::from_utf8_lossy(&root);
    assert!(h.contains("inspect"), "{h}");
    assert!(h.contains("Not GA4GH certification"), "{h}");
}

#[tokio::test]
async fn b15_t2_output_shows_selected_standard_version() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t2").await.run;
    let t = text(&run);
    assert!(t.contains("standard: drs"), "{t}");
    assert!(t.contains("selected: 1.4.0"), "{t}");
    assert!(t.contains("selected_version: 1.4.0"), "{t}");
}

#[tokio::test]
async fn b15_t3_detected_selected_verified_are_distinct() {
    let _g = B15_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs_advertising("1.3.0experimental").await;
    let run = verify_with_options(&mock.drs_url(), versioned(mock_declared("b15-t3")))
        .await
        .unwrap()
        .run;
    let t = text(&run);
    assert!(t.contains("detected: 1.3.0experimental"), "{t}");
    assert!(t.contains("selected: 1.4.0"), "{t}");
    assert!(t.contains("declared: (none)"), "{t}");
    assert!(
        !t.contains("declared: 0.0.0"),
        "implementation_version must not be printed as declared GA4GH version:\n{t}"
    );
    let sel = run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.detected_version.as_deref(), Some("1.3.0experimental"));
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert_ne!(
        sel.detected_version.as_deref(),
        sel.verified_version.as_deref()
    );
}

#[tokio::test]
async fn b15_t4_individual_check_states_are_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t4").await.run);
    assert!(t.contains("drs.object.reachable"), "{t}");
    assert!(t.contains("HLX-DRS-001"), "{t}");
    assert!(t.contains("PASS"), "{t}");
}

#[tokio::test]
async fn b15_t5_fail_attribution_is_visible() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t5").await.run;
    set_status(&mut run, "drs.object.schema", VerificationStatus::Fail);
    let t = text(&run);
    assert!(t.contains("FAIL"), "{t}");
    assert!(t.contains("attribution: target_failure"), "{t}");
    let row = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::TargetFailure));
}

#[tokio::test]
async fn b15_t6_skip_reason_is_visible() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t6").await.run;
    set_status(&mut run, "drs.object.checksum", VerificationStatus::Skip);
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        if r.id == "drs.object.checksum" {
            r.message = Some("fixture_unavailable: object not present".into());
            helix::target::attach_attribution(r);
        }
    }
    let t = text(&run);
    assert!(t.contains("SKIP"), "{t}");
    assert!(t.contains("fixture_unavailable"), "{t}");
    assert!(
        t.contains("attribution: target_configuration_failure"),
        "{t}"
    );
}

#[tokio::test]
async fn b15_t7_coverage_is_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t7").await.run);
    assert!(t.contains("Coverage"), "{t}");
    assert!(t.contains("state: partial"), "{t}");
    assert!(t.contains("coverage_id:"), "{t}");
}

#[tokio::test]
async fn b15_t8_unevaluated_areas_are_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t8").await.run);
    assert!(t.contains("unevaluated:"), "{t}");
    assert!(t.contains("drs.op.service_info"), "{t}");
    assert!(t.contains("drs.security.authorization"), "{t}");
    assert!(
        t.contains("GetServiceInfo") || t.contains("service-info"),
        "{t}"
    );
}

#[tokio::test]
async fn b15_t9_verified_claim_distinct_from_pass() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t9").await.run;
    let t = text(&run);
    assert!(t.contains("ga4gh_requirement"), "{t}");
    assert!(t.contains("VERIFIED"), "{t}");
    assert!(t.contains("PASS is a check outcome"), "{t}");
    assert!(
        t.contains("check_outcome PASS is not ga4gh_requirement VERIFIED"),
        "{t}"
    );
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
}

#[tokio::test]
async fn b15_t10_not_verified_distinct_from_process_failure() {
    let _g = B15_LOCK.lock().await;
    let run = verify(&start_mock_ga4gh_drs().await.drs_url())
        .await
        .unwrap()
        .run;
    let t = text(&run);
    assert!(t.contains("NOT_VERIFIED"), "{t}");
    assert!(t.contains("Exit 0 means executed checks passed"), "{t}");
    assert!(
        run.summary.failed == 0 && run.summary.errors == 0,
        "unversioned mock should pass checks"
    );
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[tokio::test]
async fn b15_t11_target_identity_is_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-target").await.run);
    assert!(t.contains("target_id: b15-target"), "{t}");
    assert!(t.contains("target_kind: mock"), "{t}");
}

#[tokio::test]
async fn b15_t12_fixture_identity_is_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t12").await.run);
    assert!(t.contains("object_id:"), "{t}");
    assert!(t.contains("DRS fixture"), "{t}");
}

#[tokio::test]
async fn b15_t13_checker_identity_is_visible() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t13").await.run;
    let t = text(&run);
    assert!(t.contains(PINNED_CHECKER_ID), "{t}");
    assert!(t.contains("executed checker:"), "{t}");
}

#[tokio::test]
async fn b15_t14_pack_schema_binding_catalog_visible() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t14").await.run);
    assert!(t.contains(PINNED_BINDING_ID), "{t}");
    assert!(t.contains(PINNED_CATALOG_ID), "{t}");
    assert!(t.contains(PINNED_PACK_INTEGRITY_SHA256), "{t}");
    assert!(t.contains("schema_document_sha256:"), "{t}");
    assert!(t.contains("execution_id:"), "{t}");
    assert!(t.contains(PINNED_EXECUTION_ID), "{t}");
}

#[tokio::test]
async fn b15_t15_persisted_json_reloads() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t15").await.run;
    let json = verify_json(&run).unwrap();
    let loaded = parse_verification_run(&json).expect("reload");
    assert_eq!(
        run.standard_selection.as_ref().unwrap().execution_id,
        loaded.standard_selection.as_ref().unwrap().execution_id
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    std::fs::write(&path, &json).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("HELIX EVIDENCE INSPECT"));
}

#[tokio::test]
async fn b15_t16_formatting_preserves_identity() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t16").await.run;
    let pretty = serde_json::to_string_pretty(&run).unwrap();
    let compact = serde_json::to_string(&run).unwrap();
    let a: VerificationRun = serde_json::from_str(&pretty).unwrap();
    let b: VerificationRun = serde_json::from_str(&compact).unwrap();
    assert_eq!(
        a.standard_selection.as_ref().unwrap().execution_id,
        b.standard_selection.as_ref().unwrap().execution_id
    );
    assert_eq!(classify_evidence(&a), classify_evidence(&b));
}

#[tokio::test]
async fn b15_t17_forged_claim_rejected() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t17").await.run;
    set_status(&mut run, "drs.object.schema", VerificationStatus::Fail);
    helix::claim_integrity::finalize_run(&mut run);
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = Some("1.4.0".into());
    }
    assert!(validate_artifact_consistency(&run).is_err());
    assert_eq!(classify_evidence(&run), EvidenceStanding::Invalid);
    assert!(!inspect(&run).contains("current_verifier_evidence: yes"));
}

#[tokio::test]
async fn b15_t18_forged_check_rejected() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t18").await.run;
    set_status(&mut run, "drs.object.schema", VerificationStatus::Fail);
    helix::claim_integrity::finalize_run(&mut run);
    set_status(&mut run, "drs.object.schema", VerificationStatus::Pass);
    assert_eq!(classify_evidence(&run), EvidenceStanding::Invalid);
}

#[tokio::test]
async fn b15_t19_forged_coverage_rejected() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t19").await.run;
    if let Some(c) = run.coverage.as_mut() {
        c.state = CoverageState::Complete;
        c.unevaluated.clear();
    }
    assert_eq!(classify_evidence(&run), EvidenceStanding::Invalid);
}

#[tokio::test]
async fn b15_t20_target_relabel_rejected() {
    let _g = B15_LOCK.lock().await;
    let a = versioned_mock("b15-a").await.run;
    let b = versioned_mock("b15-b").await.run;
    let mut relabeled = a.clone();
    relabeled.target = b.target.clone();
    assert_eq!(classify_evidence(&relabeled), EvidenceStanding::Invalid);
}

#[tokio::test]
async fn b15_t21_historical_not_current() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-t21").await.run;
    run.helix_git_sha = Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
    assert_eq!(
        classify_evidence(&run),
        EvidenceStanding::HistoricalObservation
    );
    let t = inspect(&run);
    assert!(t.contains("historical_observation"), "{t}");
    assert!(t.contains("current_verifier_evidence: no"), "{t}");
    assert!(validate_artifact_consistency(&run).is_ok());
}

#[tokio::test]
async fn b15_t22_two_targets_distinct_target_execution_id() {
    let _g = B15_LOCK.lock().await;
    let a = versioned_mock("b15-iso-a").await.run;
    let b = versioned_mock("b15-iso-b").await.run;
    assert_eq!(
        a.standard_selection.as_ref().unwrap().execution_id,
        b.standard_selection.as_ref().unwrap().execution_id
    );
    assert_ne!(
        a.standard_selection.as_ref().unwrap().target_execution_id,
        b.standard_selection.as_ref().unwrap().target_execution_id
    );
}

#[test]
fn b15_t23_starter_kit_remains_not_verified() {
    let Some(sk) = load_live("starter-kit.json") else {
        return;
    };
    let t = text(&sk);
    assert!(t.contains("NOT_VERIFIED"), "{t}");
    assert!(t.contains("1.3.0experimental"), "{t}");
    assert!(
        t.contains("selected: 1.4.0") || t.contains("selected_version: 1.4.0"),
        "{t}"
    );
    assert!(
        t.contains("verified: (none)") || t.contains("verified_version: (none)"),
        "{t}"
    );
    assert!(
        t.contains("declared: (none)"),
        "declared must be the operator-declared GA4GH version, not the image tag:\n{t}"
    );
    assert!(
        !t.contains("declared: 0.3.2"),
        "implementation_version must not be printed as declared GA4GH version:\n{t}"
    );
    assert_eq!(
        evaluate(&sk).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    assert!(!cites_this_verifier_build(&sk));
    assert_eq!(
        classify_evidence(&sk),
        EvidenceStanding::HistoricalObservation
    );
}

#[test]
fn b15_t24_bento_exposes_verified_partial() {
    let Some(bento) = load_live("bento.json") else {
        return;
    };
    let t = text(&bento);
    assert!(t.contains("ga4gh_requirement"), "{t}");
    assert!(t.contains("VERIFIED"), "{t}");
    assert!(t.contains("state: partial"), "{t}");
    assert!(t.contains("unevaluated:"), "{t}");
    assert_eq!(
        evaluate(&bento).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
    assert_eq!(
        bento.coverage.as_ref().unwrap().state,
        CoverageState::Partial
    );
    assert_eq!(
        classify_evidence(&bento),
        EvidenceStanding::HistoricalObservation
    );
    assert!(inspect(&bento).contains("historical_observation"));
}

#[tokio::test]
async fn b15_t25_passing_check_cannot_manufacture_verified() {
    let _g = B15_LOCK.lock().await;
    let run = verify(&start_mock_ga4gh_drs().await.drs_url())
        .await
        .unwrap()
        .run;
    assert!(run
        .executed
        .iter()
        .any(|r| r.status == VerificationStatus::Pass));
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    let t = text(&run);
    assert!(t.contains("NOT_VERIFIED"), "{t}");
    assert!(!t.contains("ga4gh_requirement  VERIFIED\n"), "{t}");
}

#[tokio::test]
async fn b15_t26_cli_exit_semantics() {
    let _g = B15_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args(["verify", &mock.drs_url()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("NOT_VERIFIED"));
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
        .code(0)
        .stdout(predicate::str::contains("ga4gh_requirement"))
        .stdout(predicate::str::contains("VERIFIED"));
    helix().args(["verify"]).assert().code(2);
    helix()
        .args(["inspect", "/no/such/verify.json"])
        .assert()
        .code(1);
}

#[tokio::test]
async fn b15_t27_human_and_json_same_result() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-t27").await.run;
    let t = text(&run);
    let v: Value = serde_json::from_str(&verify_json(&run).unwrap()).unwrap();
    assert_eq!(
        v["standard_selection"]["selected_version"].as_str(),
        Some("1.4.0")
    );
    assert!(t.contains("selected_version: 1.4.0"));
    assert_eq!(
        v["standard_selection"]["execution_id"].as_str(),
        Some(PINNED_EXECUTION_ID)
    );
    assert!(t.contains(PINNED_EXECUTION_ID));
    assert_eq!(
        v["coverage"]["coverage_id"].as_str(),
        Some(PINNED_COVERAGE_ID)
    );
    assert_eq!(v["claims"][0]["kind"].as_str(), Some("ga4gh_requirement"));
    assert!(t.contains("Claims"));
    assert!(v.get("evidence_standing").is_none());
    assert!(t.contains("Evidence standing"));
}

#[tokio::test]
async fn b15_t28_no_certification_claim() {
    let _g = B15_LOCK.lock().await;
    let t = text(&versioned_mock("b15-t28").await.run);
    assert!(
        t.contains("not GA4GH certification") || t.contains("Not GA4GH certification"),
        "{t}"
    );
    let lower = t.to_ascii_lowercase();
    assert!(!lower.contains("ga4gh certified"));
    assert!(!lower.contains("official certification"));
}

#[test]
fn b15_live_targets_share_execution_differ_in_target() {
    let Some(sk) = load_live("starter-kit.json") else {
        return;
    };
    let Some(bento) = load_live("bento.json") else {
        return;
    };
    assert_eq!(
        sk.standard_selection.as_ref().unwrap().execution_id,
        bento.standard_selection.as_ref().unwrap().execution_id
    );
    assert_ne!(
        sk.standard_selection.as_ref().unwrap().target_execution_id,
        bento
            .standard_selection
            .as_ref()
            .unwrap()
            .target_execution_id
    );
}

#[tokio::test]
async fn b15_inspect_json_is_not_verification_schema() {
    let _g = B15_LOCK.lock().await;
    let run = versioned_mock("b15-inspect-json").await.run;
    let json = verify_json(&run).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    std::fs::write(&path, json).unwrap();
    let out = helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8_lossy(&out)).unwrap();
    assert_eq!(v["not_helix_verification_v1"], true);
    assert!(v.get("executed").is_none());
    assert!(v.get("standing").is_some());
}

#[tokio::test]
async fn b15_tamper_via_inspect_exits_1() {
    let _g = B15_LOCK.lock().await;
    let mut run = versioned_mock("b15-tamper").await.run;
    tamper_selection(&mut run, |sel| {
        sel.checker_id = Some(
            "helixtest-drs:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
        );
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forged.json");
    std::fs::write(&path, serde_json::to_string(&run).unwrap()).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("invalid"));
}
