// SPDX-License-Identifier: Apache-2.0
//! B14 verification evidence durability and regression gate.
//!
//! Claims, coverage, and `verified_version` are derived. JSON formatting is
//! not identity. Forged or stale persisted JSON must not become a stronger
//! current claim. Not a DRS coverage expansion. Not HELIOS. Not certification.
//! B13 remains BLOCKED / DEFERRED.

use helix::claim_integrity::{
    finalize_run, ga4gh_requirement_is_verified, reproduction_tuple, tamper_selection,
    validate_artifact_consistency, validate_claim_integrity,
};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::compare::parse_verification_run;
use helix::coverage::CoverageState;
use helix::evidence::{
    classify_evidence, is_current_verification, revalidate_evidence, EvidenceStanding,
};
use helix::live_evidence::{
    PINNED_BINDING_ID, PINNED_CATALOG_ID, PINNED_CHECKER_ID, PINNED_COVERAGE_ID,
    PINNED_EXECUTION_ID, PINNED_HELIXTEST_SHA, PINNED_PACK_INTEGRITY_SHA256, PINNED_RELEASE_COMMIT,
    PINNED_SCHEMA_COMPONENT_SHA256, PINNED_SCHEMA_DOCUMENT_SHA256,
};
use helix::model::{
    Target, VerificationResult, VerificationRun, VerificationStatus, HELIXTEST_SHA,
};
use helix::profile::ProfileId;
use helix::provenance::cites_this_verifier_build;
use helix::redact::REDACTED;
use helix::report::verify_json;
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use serde_json::Value;
use std::path::{Path, PathBuf};

mod support;
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static B14_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn live_dir() -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local/b12")
}

fn load_live_json(name: &str) -> Option<VerificationRun> {
    let path = live_dir().join(name);
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn live_pair() -> Option<(VerificationRun, VerificationRun)> {
    Some((
        load_live_json("starter-kit.json")?,
        load_live_json("bento.json")?,
    ))
}

fn sel(run: &VerificationRun) -> &helix::model::StandardSelection {
    run.standard_selection.as_ref().expect("standard_selection")
}

fn assert_not_current(run: &VerificationRun) {
    assert_ne!(
        classify_evidence(run),
        EvidenceStanding::CurrentVerifierEvidence
    );
    assert!(!is_current_verification(run));
}

fn assert_invalid(run: &VerificationRun) {
    assert_eq!(classify_evidence(run), EvidenceStanding::Invalid);
    assert!(validate_artifact_consistency(run).is_err());
    assert!(validate_claim_integrity(run).is_err());
    assert!(!is_current_verification(run));
    assert!(!ga4gh_requirement_is_verified(run) || validate_artifact_consistency(run).is_err());
}

fn assert_drs140_pins(run: &VerificationRun) {
    let s = sel(run);
    assert_eq!(
        s.standards_source_commit.as_deref(),
        Some(PINNED_RELEASE_COMMIT)
    );
    assert_eq!(
        s.pack_integrity_sha256.as_deref(),
        Some(PINNED_PACK_INTEGRITY_SHA256)
    );
    assert_eq!(
        s.schema_document_sha256.as_deref(),
        Some(PINNED_SCHEMA_DOCUMENT_SHA256)
    );
    assert_eq!(
        s.schema_component_sha256.as_deref(),
        Some(PINNED_SCHEMA_COMPONENT_SHA256)
    );
    assert_eq!(s.checker_id.as_deref(), Some(PINNED_CHECKER_ID));
    assert_eq!(s.binding_id.as_deref(), Some(PINNED_BINDING_ID));
    assert_eq!(s.catalog_id.as_deref(), Some(PINNED_CATALOG_ID));
    assert_eq!(s.execution_id.as_deref(), Some(PINNED_EXECUTION_ID));
    assert_eq!(
        run.coverage.as_ref().and_then(|c| c.coverage_id.as_deref()),
        Some(PINNED_COVERAGE_ID)
    );
    assert_eq!(HELIXTEST_SHA, PINNED_HELIXTEST_SHA);
}

#[tokio::test]
async fn b14_t1_valid_evidence_round_trips() {
    let _g = B14_LOCK.lock().await;
    let run = versioned_mock("b14-t1").await.run;
    assert!(ga4gh_requirement_is_verified(&run));
    assert_drs140_pins(&run);
    validate_claim_integrity(&run).expect("honest emit");
    let json = verify_json(&run).expect("serialize");
    let loaded = parse_verification_run(&json).expect("deserialize+load");
    assert_eq!(sel(&run).execution_id, sel(&loaded).execution_id);
    assert_eq!(
        sel(&run).target_execution_id,
        sel(&loaded).target_execution_id
    );
    assert_eq!(
        run.coverage.as_ref().and_then(|c| c.coverage_id.clone()),
        loaded.coverage.as_ref().and_then(|c| c.coverage_id.clone())
    );
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        evaluate(&loaded).get(ClaimKind::Ga4ghRequirement).status
    );
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("evidence_standing").is_none());
    assert!(v.get("standing").is_none());
    let re = revalidate_evidence(&loaded);
    assert_eq!(
        re.claims.get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
    if cites_this_verifier_build(&loaded) {
        assert_eq!(re.standing, EvidenceStanding::CurrentVerifierEvidence);
        assert!(is_current_verification(&loaded));
    } else {
        assert_eq!(re.standing, EvidenceStanding::HistoricalObservation);
        assert!(!is_current_verification(&loaded));
    }
}

#[tokio::test]
async fn b14_t2_json_formatting_does_not_change_identity() {
    let _g = B14_LOCK.lock().await;
    let run = versioned_mock("b14-t2").await.run;
    let pretty = serde_json::to_string_pretty(&run).unwrap();
    let compact = serde_json::to_string(&run).unwrap();
    assert_ne!(pretty, compact);
    let a: VerificationRun = serde_json::from_str(&pretty).unwrap();
    let b: VerificationRun = serde_json::from_str(&compact).unwrap();
    assert_eq!(sel(&a).execution_id, sel(&b).execution_id);
    assert_eq!(sel(&a).target_execution_id, sel(&b).target_execution_id);
    assert_eq!(
        a.coverage.as_ref().and_then(|c| c.coverage_id.clone()),
        b.coverage.as_ref().and_then(|c| c.coverage_id.clone())
    );
    assert_eq!(reproduction_tuple(&a), reproduction_tuple(&b));
    assert_eq!(classify_evidence(&a), classify_evidence(&b));
}

#[tokio::test]
async fn b14_t3_forged_verified_version_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t3").await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "forged later",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(sel(&run).verified_version.is_none());
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = Some("1.4.0".into());
    }
    assert_invalid(&run);
    let json = serde_json::to_string(&run).unwrap();
    assert!(parse_verification_run(&json).is_err());
}

#[tokio::test]
async fn b14_t4_forged_claim_status_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t4").await.run;
    set_check_status(
        &mut run,
        "drs.object.checksum",
        VerificationStatus::Fail,
        "broken",
    );
    restamp(&mut run);
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );

    let mut json_only = serde_json::to_value(&run).unwrap();
    json_only["claims"] = serde_json::json!([{
        "kind": "ga4gh_requirement",
        "status": "verified",
        "satisfied": [],
        "blocks": []
    }]);
    let dropped: VerificationRun = serde_json::from_value(json_only).unwrap();
    assert_eq!(
        evaluate(&dropped).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );

    if let Some(join) = run.claim_join.as_mut() {
        for c in &mut join.claims {
            if c.kind == ClaimKind::Ga4ghRequirement {
                c.status = ClaimStatus::Verified;
            }
        }
    }
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t5_forged_coverage_state_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t5").await.run;
    assert_eq!(run.coverage.as_ref().unwrap().state, CoverageState::Partial);
    if let Some(cov) = run.coverage.as_mut() {
        cov.state = CoverageState::Complete;
        cov.required_complete = true;
    }
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t6_stale_helix_provenance_cannot_become_current() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t6").await.run;
    run.helix_git_sha = Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
    // C2: helix_git_sha is bound into claim_join. Changing SHA without restamping
    // the join is Invalid, not a silent Current upgrade.
    assert_invalid(&run);
    assert_not_current(&run);
}

#[tokio::test]
async fn b14_t7_checker_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t7").await.run;
    tamper_selection(&mut run, |sel| {
        sel.checker_id = Some(
            "helixtest-drs:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
        );
    });
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t8_binding_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t8").await.run;
    tamper_selection(&mut run, |sel| {
        sel.binding_id = Some("0".repeat(64));
    });
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t9_catalog_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t9").await.run;
    tamper_selection(&mut run, |sel| {
        sel.catalog_id = Some("1".repeat(64));
    });
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t10_coverage_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t10").await.run;
    if let Some(cov) = run.coverage.as_mut() {
        cov.coverage_id = Some("2".repeat(64));
    }
    if let Some(join) = run.claim_join.as_mut() {
        join.coverage_id = Some("2".repeat(64));
    }
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t11_pack_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t11").await.run;
    tamper_selection(&mut run, |sel| {
        sel.pack_integrity_sha256 = Some("ab".repeat(32));
    });
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t12_schema_mutation_invalidates() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t12").await.run;
    tamper_selection(&mut run, |sel| {
        sel.schema_document_sha256 = Some("cd".repeat(32));
    });
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t13_target_relabel_cannot_transfer_evidence() {
    let _g = B14_LOCK.lock().await;
    let a = versioned_mock("b14-target-a").await.run;
    let b = versioned_mock("b14-target-b").await.run;
    assert_eq!(sel(&a).execution_id, sel(&b).execution_id);
    assert_ne!(sel(&a).target_execution_id, sel(&b).target_execution_id);
    let mut relabeled = a.clone();
    relabeled.target = b.target.clone();
    assert_invalid(&relabeled);
    let mut teid_only = a.clone();
    if let Some(s) = teid_only.standard_selection.as_mut() {
        s.target_execution_id = sel(&b).target_execution_id.clone();
    }
    assert_invalid(&teid_only);
}

#[tokio::test]
async fn b14_t14_fixture_mutation_cannot_transfer_evidence() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t14").await.run;
    let original_teid = sel(&run).target_execution_id.clone();
    if let Some(fx) = run.drs_fixture.as_mut() {
        fx.object_id = "other-object-id".into();
        fx.expected_sha256 = Some("ee".repeat(32));
    }
    assert_invalid(&run);
    assert_eq!(sel(&run).target_execution_id, original_teid);
}

#[tokio::test]
async fn b14_t15_required_check_removal_cannot_verify() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t15").await.run;
    run.executed.retain(|r| r.id != "drs.object.not_found");
    run.recompute_summary();
    assert_invalid(&run);

    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(sel(&run).verified_version.is_none());
    if cites_this_verifier_build(&run) {
        assert_eq!(
            classify_evidence(&run),
            EvidenceStanding::CurrentVerifierEvidence
        );
    }
    assert!(!is_current_verification(&run) || !ga4gh_requirement_is_verified(&run));
}

#[tokio::test]
async fn b14_t16_unknown_check_cannot_silently_satisfy_contract() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t16").await.run;
    let mut extra: VerificationResult = run.executed[0].clone();
    extra.id = "drs.invented.not_in_catalog".into();
    extra.code = "HLX-DRS-999".into();
    extra.status = VerificationStatus::Pass;
    run.executed.push(extra.clone());
    run.recompute_summary();
    assert_invalid(&run);

    run.executed.retain(|r| r.id != "drs.object.checksum");
    run.executed.push(extra);
    run.recompute_summary();
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(sel(&run).verified_version.is_none());
}

#[tokio::test]
async fn b14_t17_fail_to_pass_forgery_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t17").await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Pass,
        "forged pass",
    );
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t18_skip_to_pass_forgery_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t18").await.run;
    set_check_status(
        &mut run,
        "drs.object.checksum",
        VerificationStatus::Skip,
        "fixture_unavailable: object not present",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    set_check_status(
        &mut run,
        "drs.object.checksum",
        VerificationStatus::Pass,
        "forged pass",
    );
    assert_invalid(&run);
}

#[tokio::test]
async fn b14_t19_attribution_forgery_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t19").await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    let row = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::TargetFailure));
    for r in run.executed.iter_mut() {
        if r.id == "drs.object.schema" {
            r.attribution = Some(FailureAttribution::SpecFailure);
        }
    }
    assert_invalid(&run);
}

#[test]
fn b14_t20_historical_b12_evidence_is_not_restamped() {
    let Some((sk, bento)) = live_pair() else {
        return;
    };
    assert!(sk.helix_git_sha.is_none());
    assert!(bento.helix_git_sha.is_none());
    assert!(!cites_this_verifier_build(&sk));
    assert!(!cites_this_verifier_build(&bento));
    let sk_raw = std::fs::read_to_string(live_dir().join("starter-kit.json")).unwrap();
    let bento_raw = std::fs::read_to_string(live_dir().join("bento.json")).unwrap();
    assert!(!sk_raw.contains("helix_git_sha"));
    assert!(!bento_raw.contains("helix_git_sha"));
}

#[test]
fn b14_t21_historical_evidence_remains_inspectable() {
    let Some((sk, bento)) = live_pair() else {
        return;
    };
    assert_eq!(
        classify_evidence(&sk),
        EvidenceStanding::HistoricalObservation
    );
    assert_eq!(
        classify_evidence(&bento),
        EvidenceStanding::HistoricalObservation
    );
    assert_not_current(&sk);
    assert_not_current(&bento);
    validate_artifact_consistency(&sk).expect("starter-kit historical consistency");
    validate_artifact_consistency(&bento).expect("bento historical consistency");
    let _ = evaluate(&sk);
    let _ = evaluate(&bento);
    assert_eq!(
        sk.coverage.as_ref().and_then(|c| c.coverage_id.as_deref()),
        Some(PINNED_COVERAGE_ID)
    );
    assert_eq!(sel(&sk).execution_id.as_deref(), Some(PINNED_EXECUTION_ID));
    assert_eq!(
        sel(&bento).execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
}

#[test]
fn b14_t22_target_a_b_evidence_remains_isolated() {
    let Some((sk, bento)) = live_pair() else {
        return;
    };
    assert_eq!(sel(&sk).execution_id, sel(&bento).execution_id);
    assert_ne!(
        sel(&sk).target_execution_id,
        sel(&bento).target_execution_id
    );
    let mut relabeled = sk.clone();
    relabeled.target = bento.target.clone();
    if let Some(s) = relabeled.standard_selection.as_mut() {
        s.target_execution_id = sel(&bento).target_execution_id.clone();
    }
    assert_invalid(&relabeled);
}

#[tokio::test]
async fn b14_t23_no_secret_enters_durable_evidence() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t23").await.run;
    let token = "Bearer supersecret-helix-b14-token-value";
    run.executed[0].message = Some(format!("Authorization: {token}"));
    let json = verify_json(&run).expect("redacted serialize");
    assert!(
        !json.contains("supersecret-helix-b14-token-value"),
        "{json}"
    );
    assert!(json.contains(REDACTED) || !json.to_ascii_lowercase().contains("bearer "));
    assert!(!json.contains("HELIX_HMAC_SECRET"));
}

#[tokio::test]
async fn b14_t24_timestamps_paths_host_do_not_alter_deterministic_identity() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-t24").await.run;
    let exec = sel(&run).execution_id.clone();
    let teid = sel(&run).target_execution_id.clone();
    let cov = run.coverage.as_ref().and_then(|c| c.coverage_id.clone());
    let tuple = reproduction_tuple(&run);
    run.timestamp = "1999-01-01T00:00:00Z".into();
    assert_eq!(sel(&run).execution_id, exec);
    assert_eq!(sel(&run).target_execution_id, teid);
    assert_eq!(
        run.coverage.as_ref().and_then(|c| c.coverage_id.clone()),
        cov
    );
    assert_eq!(reproduction_tuple(&run), tuple);
    assert!(!tuple.endpoint.as_deref().unwrap_or("").contains("/Users/"));
    assert_eq!(exec.as_deref(), Some(PINNED_EXECUTION_ID));
}

#[tokio::test]
async fn b14_combined_claim_coverage_forgery_fails() {
    let _g = B14_LOCK.lock().await;
    let mut run = versioned_mock("b14-combined").await.run;
    set_check_status(
        &mut run,
        "drs.object.range",
        VerificationStatus::Fail,
        "range broken",
    );
    restamp(&mut run);
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = Some("1.4.0".into());
    }
    if let Some(join) = run.claim_join.as_mut() {
        join.verified_version = Some("1.4.0".into());
        for c in &mut join.claims {
            c.status = ClaimStatus::Verified;
        }
    }
    if let Some(cov) = run.coverage.as_mut() {
        cov.state = CoverageState::Complete;
        cov.required_complete = true;
    }
    assert_invalid(&run);
}

#[test]
fn b14_evaluator_example_without_join_still_loads() {
    let raw = include_str!("../docs/evaluator-pack/example-verify.json");
    parse_verification_run(raw).expect("pre-B8 example remains Load-able");
}

#[test]
fn b14_empty_run_is_not_invalid_solely_from_emptiness() {
    let run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
    assert_ne!(classify_evidence(&run), EvidenceStanding::Invalid);
}

#[test]
fn b14_helix_git_sha_is_not_in_spec_identities() {
    assert_eq!(PINNED_EXECUTION_ID.len(), 64);
    assert_eq!(PINNED_COVERAGE_ID.len(), 64);
    if let Some(sha) = helix::provenance::helix_git_sha() {
        assert!(!PINNED_EXECUTION_ID.contains(sha));
        assert!(!PINNED_COVERAGE_ID.contains(sha));
        assert!(!PINNED_BINDING_ID.contains(sha));
        assert!(!PINNED_CATALOG_ID.contains(sha));
    }
}

#[test]
fn b14_live_files_are_optional_and_not_rewritten() {
    let path = live_dir().join("bento.json");
    if path.exists() {
        assert!(Path::new(&path).is_file());
    }
}
