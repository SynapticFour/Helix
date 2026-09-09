// SPDX-License-Identifier: Apache-2.0
//! C3 — independent DRS differential interpretation.
//!
//! Operator-facing comparison of two helix-verification-v1 artifacts.
//! Does not create verification. Does not rank implementations.
//! Does not expand DRS coverage. B13 remains BLOCKED / DEFERRED. Not HELIOS.

use assert_cmd::Command;
use helix::claim_integrity::finalize_run;
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::differential::{
    assert_comparable_runs, classify_difference, differential_files, differential_from_runs,
    format_differential_text, DifferenceClass,
};
use helix::evidence::{classify_evidence, EvidenceStanding};
use helix::live_evidence::{PINNED_COVERAGE_ID, PINNED_EXECUTION_ID};
use helix::model::{VerificationRun, VerificationStatus, SCHEMA_VERSION};
use helix::report::verify_json;
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify, verify_with_options, VerifyOptions, VerifySelection};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

mod support;
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static C3_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const B12_STARTER_SHA256: &str = "5e8cc568f942b93af4e97f3f8da48b5ee58a833975f2e1495310d2106547c08b";
const B12_BENTO_SHA256: &str = "61d99c5e0da5a307e0e577fbdd1e951fa52b4aea6d5ce9cf8bf35f3ae4a5f6e6";

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn live_b12(name: &str) -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p).join(name);
    }
    root().join("local/b12").join(name)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

fn restamp(run: &mut VerificationRun) {
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = None;
    }
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        r.verified_version = None;
    }
    run.claim_join = None;
    run.coverage = None;
    finalize_run(run);
}

fn set_check_status(
    run: &mut VerificationRun,
    id: &str,
    status: VerificationStatus,
    message: &str,
) {
    let mut found = false;
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        if r.id == id {
            r.status = status;
            r.message = Some(message.into());
            helix::target::attach_attribution(r);
            found = true;
        }
    }
    assert!(found, "missing check {id}");
    run.recompute_summary();
}

fn ranking_needles() -> &'static [&'static str] {
    &[
        "winner",
        "loser",
        "leaderboard",
        "overall score",
        "compliance percentage",
        "coverage score",
        "best implementation",
        "quality grade",
        "ranks first",
        "ranked first",
    ]
}

fn assert_no_ranking(text: &str) {
    let lower = text.to_lowercase();
    for n in ranking_needles() {
        assert!(!lower.contains(n), "ranking phrase `{n}` in {text}");
    }
}

#[tokio::test]
async fn c3_t1_same_contract() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t1-a").await.run;
    let b = versioned_mock("c3-t1-b").await.run;
    assert_comparable_runs(&a, &b).expect("same DRS 1.4.0 contract");
    let report = differential_from_runs(&a, &b);
    assert_eq!(
        a.standard_selection
            .as_ref()
            .and_then(|s| s.execution_id.as_deref()),
        Some(PINNED_EXECUTION_ID)
    );
    assert_eq!(
        a.standard_selection
            .as_ref()
            .and_then(|s| s.execution_id.clone()),
        b.standard_selection
            .as_ref()
            .and_then(|s| s.execution_id.clone())
    );
    assert_eq!(report.schema_version, "helix-differential-v1");
    assert!(!report.creates_verification);
}

#[tokio::test]
async fn c3_t2_distinct_targets() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t2-a").await.run;
    let b = versioned_mock("c3-t2-b").await.run;
    let te_a = a
        .standard_selection
        .as_ref()
        .and_then(|s| s.target_execution_id.clone())
        .unwrap();
    let te_b = b
        .standard_selection
        .as_ref()
        .and_then(|s| s.target_execution_id.clone())
        .unwrap();
    assert_ne!(te_a, te_b);
    let text = format_differential_text(&differential_from_runs(&a, &b));
    assert!(text.contains(&te_a), "{text}");
    assert!(text.contains(&te_b), "{text}");
    assert!(text.contains("distinct_target_execution_id: yes"), "{text}");
    assert_eq!(a.target.identity.as_ref().unwrap().target_id, "c3-t2-a");
    assert_eq!(b.target.identity.as_ref().unwrap().target_id, "c3-t2-b");
}

#[tokio::test]
async fn c3_t3_pass_pass() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t3-a").await.run;
    let b = versioned_mock("c3-t3-b").await.run;
    let report = differential_from_runs(&a, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.a_status, Some(VerificationStatus::Pass));
    assert_eq!(row.b_status, Some(VerificationStatus::Pass));
    assert_eq!(row.class, DifferenceClass::SameBehavior);
    let text = format_differential_text(&report);
    assert!(text.contains("drs.object.schema"), "{text}");
    assert!(text.contains("PASS | PASS"), "{text}");
}

#[tokio::test]
async fn c3_t4_fail_pass() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t4-a").await.run;
    let mut b = versioned_mock("c3-t4-b").await.run;
    set_check_status(
        &mut b,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut b);
    let report = differential_from_runs(&a, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.a_status, Some(VerificationStatus::Pass));
    assert_eq!(row.b_status, Some(VerificationStatus::Fail));
    assert_eq!(row.class, DifferenceClass::TargetBehaviorDifference);
    let text = format_differential_text(&report);
    assert!(text.contains("target_behavior_difference"), "{text}");
    assert!(text.contains("FAIL (target_failure)"), "{text}");
    assert!(
        !text.to_ascii_lowercase().contains("wins this check"),
        "{text}"
    );
}

#[tokio::test]
async fn c3_t5_skip_pass() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t5-a").await.run;
    let mut b = versioned_mock("c3-t5-b").await.run;
    set_check_status(
        &mut b,
        "drs.object.checksum",
        VerificationStatus::Skip,
        "fixture_unavailable: no access_url",
    );
    restamp(&mut b);
    let report = differential_from_runs(&a, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.checksum")
        .unwrap();
    assert_eq!(row.a_status, Some(VerificationStatus::Pass));
    assert_eq!(row.b_status, Some(VerificationStatus::Skip));
    assert_eq!(row.class, DifferenceClass::FixtureCapabilityDifference);
    let text = format_differential_text(&report);
    assert!(text.contains("SKIP"), "{text}");
    assert!(text.contains("fixture_capability_difference"), "{text}");
    assert!(!text.contains("SKIP converted to FAIL"), "{text}");
}

#[tokio::test]
async fn c3_t6_attribution_preserved() {
    let _g = C3_LOCK.lock().await;
    let mut a = versioned_mock("c3-t6-a").await.run;
    let b = versioned_mock("c3-t6-b").await.run;
    set_check_status(
        &mut a,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut a);
    let report = differential_from_runs(&a, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.a_attribution, Some(FailureAttribution::TargetFailure));
    assert_eq!(row.b_attribution, None);
    let text = format_differential_text(&report);
    assert!(text.contains("target_failure"), "{text}");
}

#[tokio::test]
async fn c3_t7_verified_vs_not_verified() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t7-a").await.run;
    let mut b = versioned_mock("c3-t7-b").await.run;
    set_check_status(
        &mut b,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut b);
    assert_eq!(
        evaluate(&a).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
    assert_eq!(
        evaluate(&b).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    let text = format_differential_text(&differential_from_runs(&a, &b));
    assert!(text.contains("VERIFIED"), "{text}");
    assert!(text.contains("NOT_VERIFIED"), "{text}");
    assert!(
        text.contains("VERIFIED / NOT_VERIFIED is the derived claim, not a count of PASS rows"),
        "{text}"
    );
    assert_no_ranking(&text);
}

#[tokio::test]
async fn c3_t8_invalid_evidence_rejected() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t8-a").await.run;
    let mut b = versioned_mock("c3-t8-b").await.run;
    set_check_status(
        &mut b,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    assert_eq!(classify_evidence(&b), EvidenceStanding::Invalid);
    assert!(assert_comparable_runs(&a, &b).is_err());

    let dir = tempfile::tempdir().unwrap();
    let pa = dir.path().join("a.json");
    let pb = dir.path().join("b.json");
    std::fs::write(&pa, verify_json(&a).unwrap()).unwrap();
    std::fs::write(&pb, serde_json::to_string_pretty(&b).unwrap()).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["differential", pa.to_str().unwrap(), pb.to_str().unwrap()])
        .assert()
        .failure();
}

#[tokio::test]
async fn c3_t9_incompatible_execution_ids() {
    let _g = C3_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let versioned = verify_with_options(&mock.drs_url(), versioned(mock_declared("c3-t9-v")))
        .await
        .unwrap()
        .run;
    let unversioned = verify(&mock.drs_url()).await.unwrap().run;
    let err = assert_comparable_runs(&versioned, &unversioned)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("verification contract")
            || err.contains("execution_id")
            || err.contains("standard"),
        "{err}"
    );
}

#[tokio::test]
async fn c3_t10_different_standard_version() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t10-a").await.run;
    let mut b = versioned_mock("c3-t10-b").await.run;
    if let Some(sel) = b.standard_selection.as_mut() {
        sel.standard = Some("wes".into());
    }
    restamp(&mut b);
    let err = assert_comparable_runs(&a, &b).unwrap_err().to_string();
    assert!(err.contains("standard") || err.contains("version"), "{err}");

    let dir = tempfile::tempdir().unwrap();
    let pa = dir.path().join("a.json");
    let pb = dir.path().join("b.json");
    std::fs::write(&pa, verify_json(&a).unwrap()).unwrap();
    std::fs::write(&pb, verify_json(&b).unwrap()).unwrap();
    helix()
        .env("NO_COLOR", "1")
        .args(["differential", pa.to_str().unwrap(), pb.to_str().unwrap()])
        .assert()
        .failure();
}

#[tokio::test]
async fn c3_t11_no_mutation() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t11-a").await.run;
    let b = versioned_mock("c3-t11-b").await.run;
    let dir = tempfile::tempdir().unwrap();
    let pa = dir.path().join("a.json");
    let pb = dir.path().join("b.json");
    std::fs::write(&pa, verify_json(&a).unwrap()).unwrap();
    std::fs::write(&pb, verify_json(&b).unwrap()).unwrap();
    let before_a = sha256_hex(&std::fs::read(&pa).unwrap());
    let before_b = sha256_hex(&std::fs::read(&pb).unwrap());
    helix()
        .env("NO_COLOR", "1")
        .args(["differential", pa.to_str().unwrap(), pb.to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(before_a, sha256_hex(&std::fs::read(&pa).unwrap()));
    assert_eq!(before_b, sha256_hex(&std::fs::read(&pb).unwrap()));
}

#[tokio::test]
async fn c3_t12_no_ranking() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t12-a").await.run;
    let mut b = versioned_mock("c3-t12-b").await.run;
    set_check_status(
        &mut b,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut b);
    let report = differential_from_runs(&a, &b);
    let text = format_differential_text(&report);
    let json = helix::differential::differential_json(&report).unwrap();
    assert_no_ranking(&text);
    assert_no_ranking(&json);
    assert_eq!(report.ranking_semantics, "absent");
    assert!(text.contains("does not rank implementations"), "{text}");
}

#[test]
fn c3_t13_historical_b12() {
    let pa = live_b12("starter-kit.json");
    let pb = live_b12("bento.json");
    if !pa.is_file() || !pb.is_file() {
        return;
    }
    let before_a = sha256_hex(&std::fs::read(&pa).unwrap());
    let before_b = sha256_hex(&std::fs::read(&pb).unwrap());
    assert_eq!(before_a, B12_STARTER_SHA256);
    assert_eq!(before_b, B12_BENTO_SHA256);
    let report = differential_files(&pa, &pb).expect("B12 remains comparable");
    assert_eq!(
        report.targets[0].evidence_standing,
        "historical_observation"
    );
    assert_eq!(
        report.targets[1].evidence_standing,
        "historical_observation"
    );
    assert_eq!(
        report.targets[0].execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_eq!(
        report.targets[1].execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_ne!(
        report.targets[0].target_execution_id,
        report.targets[1].target_execution_id
    );
    let text = format_differential_text(&report);
    assert!(text.contains("NOT_VERIFIED"), "{text}");
    assert!(text.contains("VERIFIED"), "{text}");
    assert!(
        text.contains("ABSENT") || text.contains("PASS | PASS") || text.contains("FAIL"),
        "{text}"
    );
    assert_no_ranking(&text);
    helix()
        .env("NO_COLOR", "1")
        .args(["differential", pa.to_str().unwrap(), pb.to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(before_a, sha256_hex(&std::fs::read(&pa).unwrap()));
    assert_eq!(before_b, sha256_hex(&std::fs::read(&pb).unwrap()));
}

#[tokio::test]
async fn c3_t14_current_evidence_standing_unchanged() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-t14-a").await.run;
    let b = versioned_mock("c3-t14-b").await.run;
    if helix::provenance::helix_git_sha().is_none() {
        return;
    }
    assert_eq!(
        classify_evidence(&a),
        EvidenceStanding::CurrentVerifierEvidence
    );
    let dir = tempfile::tempdir().unwrap();
    let pa = dir.path().join("a.json");
    let pb = dir.path().join("b.json");
    std::fs::write(&pa, verify_json(&a).unwrap()).unwrap();
    std::fs::write(&pb, verify_json(&b).unwrap()).unwrap();
    let out = helix()
        .env("NO_COLOR", "1")
        .args(["differential", pa.to_str().unwrap(), pb.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let t = String::from_utf8_lossy(&out);
    assert!(t.contains("current_verifier_evidence"), "{t}");
    let loaded: VerificationRun = serde_json::from_slice(&std::fs::read(&pa).unwrap()).unwrap();
    assert_eq!(
        classify_evidence(&loaded),
        EvidenceStanding::CurrentVerifierEvidence
    );
    assert_eq!(loaded.schema_version, SCHEMA_VERSION);
}

#[tokio::test]
async fn c3_negative_missing_check_is_absent_not_skip() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-abs-a").await.run;
    let mut b = versioned_mock("c3-abs-b").await.run;
    b.executed.retain(|r| r.id != "drs.object.schema");
    b.skipped.retain(|r| r.id != "drs.object.schema");
    restamp(&mut b);
    let report = differential_from_runs(&a, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    assert!(row.a_status.is_some());
    assert!(row.b_status.is_none());
    assert_eq!(row.class, DifferenceClass::InsufficientEvidence);
    let text = format_differential_text(&report);
    assert!(text.contains("ABSENT"), "{text}");
    assert!(!text.contains("ABSENT converted to SKIP"), "{text}");
}

#[tokio::test]
async fn c3_negative_result_change_changes_differential() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-chg-a").await.run;
    let b_same = versioned_mock("c3-chg-b").await.run;
    let mut b_fail = b_same.clone();
    set_check_status(
        &mut b_fail,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut b_fail);
    let same = differential_from_runs(&a, &b_same);
    let diff = differential_from_runs(&a, &b_fail);
    let same_row = same
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    let diff_row = diff
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .unwrap();
    assert_eq!(same_row.class, DifferenceClass::SameBehavior);
    assert_eq!(diff_row.class, DifferenceClass::TargetBehaviorDifference);
}

#[tokio::test]
async fn c3_coverage_and_authorization_unchanged() {
    let _g = C3_LOCK.lock().await;
    let a = versioned_mock("c3-cov-a").await.run;
    let b = versioned_mock("c3-cov-b").await.run;
    assert_eq!(
        a.coverage.as_ref().and_then(|c| c.coverage_id.as_deref()),
        Some(PINNED_COVERAGE_ID)
    );
    let report = differential_from_runs(&a, &b);
    assert_eq!(report.coverage_id.as_deref(), Some(PINNED_COVERAGE_ID));
    let uneval = a
        .coverage
        .as_ref()
        .unwrap()
        .unevaluated
        .iter()
        .any(|s| s == "drs.security.authorization");
    assert!(uneval);
    assert_eq!(a.schema_version, SCHEMA_VERSION);
}

#[test]
fn c3_classify_skip_vs_pass_is_not_target_failure() {
    let mut ra = helix::model::VerificationResult::skip(
        helix::model::VerificationCheck::from_spec(helix::identity::spec("drs.object.checksum"))
            .with_profile("generic"),
        "fixture_unavailable: no access_url",
    );
    helix::target::attach_attribution(&mut ra);
    let mut rb = helix::model::VerificationResult::pass(
        helix::model::VerificationCheck::from_spec(helix::identity::spec("drs.object.checksum"))
            .with_profile("generic"),
    );
    helix::target::attach_attribution(&mut rb);
    assert_eq!(
        classify_difference(Some(&ra), Some(&rb)),
        DifferenceClass::FixtureCapabilityDifference
    );
}
