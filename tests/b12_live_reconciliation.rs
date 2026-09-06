// SPDX-License-Identifier: Apache-2.0
//! B12 live evidence reconciliation gate.
//!
//! Live HTTP is optional. `make prove` must pass without Docker or running
//! targets. When `local/b12/*.json` (or `HELIX_B12_LIVE_DIR`) is present, the
//! live tamper / differential assertions run against those CLI artifacts.
//! Do not commit volatile live JSON. Not HELIOS. Not certification. Not ranking.

use helix::claim_integrity::{
    ga4gh_requirement_is_verified, tamper_selection, validate_claim_integrity,
};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::coverage::{coverage_id, CoverageClass, CoverageState};
use helix::fixture::DrsVerifyFixture;
use helix::independence::run_counts_as_independent;
use helix::live_evidence::{
    live_independent_observation, BENTO_OBJECT_ID, BENTO_OBJECT_SHA256, BENTO_TARGET_ID,
    PINNED_BINDING_ID, PINNED_CATALOG_ID, PINNED_CHECKER_ID, PINNED_CHECKER_SOURCE_SHA256,
    PINNED_COVERAGE_ID, PINNED_EXECUTION_ID, PINNED_HELIXTEST_SHA, PINNED_PACK_INTEGRITY_SHA256,
    PINNED_RELEASE_COMMIT, PINNED_SCHEMA_COMPONENT_SHA256, PINNED_SCHEMA_DOCUMENT_SHA256,
    STARTER_KIT_OBJECT_ID, STARTER_KIT_TARGET_ID,
};
use helix::model::{VerificationRun, VerificationStatus, HELIXTEST_SHA};
use helix::profile::ProfileId;
use helix::report::verify_json;
use helix::standards::{
    binding_id, catalog_id, execution_id, DRS_140_CONTRACT, DRS_140_PACK_ID,
    DRS_140_SCHEMA_COMPONENT, DRS_140_SCHEMA_ENTRY,
};
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use serde_json::Value;
use std::path::{Path, PathBuf};

mod support;
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static B12_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn versioned(declared: DeclaredTarget, fixture: DrsVerifyFixture) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: declared,
        drs_fixture: fixture,
        ..Default::default()
    }
}

fn mock_declared(id: &str, name: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::Mock,
        implementation_name: Some(name.into()),
        implementation_version: Some("0.0.0".into()),
        ..DeclaredTarget::default()
    }
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

fn cov(run: &VerificationRun) -> &helix::coverage::CoverageReport {
    run.coverage.as_ref().expect("coverage")
}

fn check_status(run: &VerificationRun, id: &str) -> Option<VerificationStatus> {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id)
        .map(|r| r.status)
}

fn claim(run: &VerificationRun, kind: ClaimKind) -> ClaimStatus {
    evaluate(run).get(kind).status
}

fn assert_integrity_err(run: &VerificationRun) {
    assert!(
        validate_claim_integrity(run).is_err(),
        "expected claim integrity failure"
    );
}

fn set_coverage_row_pass(run: &mut VerificationRun, id: &str) {
    let row = run
        .coverage
        .as_mut()
        .expect("coverage")
        .rows
        .iter_mut()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("missing coverage row {id}"));
    row.executed = true;
    row.result = Some(VerificationStatus::Pass);
}

#[test]
fn b12_verifier_identities_match_b11_pins() {
    assert_eq!(HELIXTEST_SHA, PINNED_HELIXTEST_SHA);
    assert_eq!(
        helix::checker::executed_checker_source_sha256(),
        PINNED_CHECKER_SOURCE_SHA256
    );
    assert_eq!(helix::checker::executed_checker_id(), PINNED_CHECKER_ID);
    assert_eq!(DRS_140_CONTRACT.release_commit, PINNED_RELEASE_COMMIT);
    assert_eq!(DRS_140_CONTRACT.pack_id, DRS_140_PACK_ID);
    assert_eq!(catalog_id(&DRS_140_CONTRACT), PINNED_CATALOG_ID);
    assert_eq!(
        binding_id(
            &DRS_140_CONTRACT,
            PINNED_PACK_INTEGRITY_SHA256,
            PINNED_SCHEMA_DOCUMENT_SHA256,
            PINNED_SCHEMA_COMPONENT_SHA256,
        ),
        PINNED_BINDING_ID
    );
    assert_eq!(
        execution_id(
            DRS_140_PACK_ID,
            PINNED_PACK_INTEGRITY_SHA256,
            PINNED_SCHEMA_DOCUMENT_SHA256,
            PINNED_SCHEMA_COMPONENT_SHA256,
            PINNED_CHECKER_ID,
            DRS_140_SCHEMA_ENTRY,
            DRS_140_SCHEMA_COMPONENT,
        ),
        PINNED_EXECUTION_ID
    );
    assert_eq!(
        coverage_id(
            &DRS_140_CONTRACT,
            PINNED_CHECKER_ID,
            Some(PINNED_BINDING_ID),
            Some(PINNED_PACK_INTEGRITY_SHA256),
        ),
        PINNED_COVERAGE_ID
    );
}

#[test]
fn b12_no_target_name_branches_in_verifier() {
    for rel in [
        "src/verify.rs",
        "src/adapter/mod.rs",
        "src/coverage.rs",
        "src/claims.rs",
        "src/claim_integrity.rs",
        "src/fixture.rs",
        "src/live_evidence.rs",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let lower = src.to_lowercase();
        assert!(
            !lower.contains("if target =="),
            "{rel} must not branch on target identity"
        );
        assert!(
            !lower.contains("if target_id"),
            "{rel} must not branch on target_id"
        );
    }
    let verify =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/verify.rs"))
            .unwrap();
    let lower = verify.to_lowercase();
    assert!(!lower.contains("bento_drs"));
    assert!(!lower.contains("starter-kit"));
    assert!(!lower.contains(":4500"));
    assert!(!lower.contains(":5000"));
}

#[test]
fn b12_live_artifacts_are_optional_for_prove() {
    // Absence is an environment limitation, not a software failure.
    let _ = live_pair();
}

#[tokio::test]
async fn b12_mock_named_bento_is_rejected_as_live() {
    let _g = B12_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        versioned(
            mock_declared(BENTO_TARGET_ID, "bento_drs"),
            DrsVerifyFixture::default_catalog(),
        ),
    )
    .await
    .unwrap();
    assert!(!run_counts_as_independent(&outcome.run));
    assert!(!live_independent_observation(&outcome.run));
    assert_eq!(
        outcome.run.target.identity.as_ref().unwrap().target_kind,
        TargetKind::Mock
    );
}

#[tokio::test]
async fn b12_target_and_fixture_do_not_choose_coverage_or_execution_id() {
    let _g = B12_LOCK.lock().await;
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let a = verify_with_options(
        &mock_a.drs_url(),
        versioned(
            mock_declared("b12-sk-standin", "ga4gh-starter-kit-drs"),
            DrsVerifyFixture::default_catalog(),
        ),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &mock_b.drs_url(),
        versioned(
            mock_declared("b12-bento-standin", "bento_drs"),
            DrsVerifyFixture::operator_declared(
                support::mock_ga4gh_drs::TEST_OBJECT_ID.into(),
                Some(BENTO_OBJECT_SHA256.into()),
            )
            .unwrap(),
        ),
    )
    .await
    .unwrap();
    assert_eq!(cov(&a.run).coverage_id.as_deref(), Some(PINNED_COVERAGE_ID));
    assert_eq!(cov(&b.run).coverage_id.as_deref(), Some(PINNED_COVERAGE_ID));
    assert_eq!(
        sel(&a.run).execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_eq!(
        sel(&b.run).execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b.run).target_execution_id
    );
    let a_json = verify_json(&a.run).unwrap();
    let _ = verify_json(&b.run).unwrap();
    let a_again: VerificationRun = serde_json::from_str(&a_json).unwrap();
    assert_eq!(
        sel(&a_again).target_execution_id,
        sel(&a.run).target_execution_id
    );
    assert_eq!(cov(&a_again).coverage_id, cov(&a.run).coverage_id);
    assert!(!live_independent_observation(&a.run));
    assert!(!live_independent_observation(&b.run));
}

#[tokio::test]
async fn b12_authorization_and_service_info_remain_unevaluated() {
    let _g = B12_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_with_options(
        &mock.drs_url(),
        versioned(
            mock_declared("b12-un-eval", "b12-mock"),
            DrsVerifyFixture::default_catalog(),
        ),
    )
    .await
    .unwrap()
    .run;
    assert!(ga4gh_requirement_is_verified(&run));
    assert_eq!(cov(&run).state, CoverageState::Partial);
    for id in [
        "drs.security.authorization",
        "drs.op.service_info",
        "drs.checksum.types.other",
        "helix.helios",
        "http.discovery.redirect_follow",
    ] {
        let row = cov(&run).row(id).unwrap_or_else(|| panic!("{id}"));
        assert!(!row.executed, "{id}");
        assert_ne!(row.result, Some(VerificationStatus::Pass), "{id}");
    }
    assert_eq!(
        cov(&run)
            .row("drs.security.authorization")
            .unwrap()
            .classification,
        CoverageClass::Unevaluated
    );
    assert_eq!(
        cov(&run).row("helix.helios").unwrap().classification,
        CoverageClass::OutOfScope
    );
}

#[test]
fn b12_live_reconciliation_and_tamper_when_artifacts_present() {
    let Some((sk, bento)) = live_pair() else {
        return;
    };

    validate_claim_integrity(&sk).expect("starter-kit live integrity");
    validate_claim_integrity(&bento).expect("bento live integrity");
    assert!(live_independent_observation(&sk));
    assert!(live_independent_observation(&bento));
    assert_eq!(
        sk.target.identity.as_ref().unwrap().target_id,
        STARTER_KIT_TARGET_ID
    );
    assert_eq!(
        bento.target.identity.as_ref().unwrap().target_id,
        BENTO_TARGET_ID
    );
    assert_eq!(
        sk.target.identity.as_ref().unwrap().target_kind,
        TargetKind::RealIndependentLocalImplementation
    );
    assert_eq!(
        bento.target.identity.as_ref().unwrap().target_kind,
        TargetKind::RealIndependentLocalImplementation
    );
    assert_eq!(
        sk.drs_fixture.as_ref().unwrap().object_id,
        STARTER_KIT_OBJECT_ID
    );
    assert_eq!(
        bento.drs_fixture.as_ref().unwrap().object_id,
        BENTO_OBJECT_ID
    );
    assert_eq!(
        bento
            .drs_fixture
            .as_ref()
            .unwrap()
            .expected_sha256
            .as_deref(),
        Some(BENTO_OBJECT_SHA256)
    );

    assert_eq!(cov(&sk).coverage_id.as_deref(), Some(PINNED_COVERAGE_ID));
    assert_eq!(cov(&bento).coverage_id.as_deref(), Some(PINNED_COVERAGE_ID));
    assert_eq!(sel(&sk).execution_id.as_deref(), Some(PINNED_EXECUTION_ID));
    assert_eq!(
        sel(&bento).execution_id.as_deref(),
        Some(PINNED_EXECUTION_ID)
    );
    assert_ne!(
        sel(&sk).target_execution_id,
        sel(&bento).target_execution_id
    );
    assert_eq!(sel(&sk).checker_id.as_deref(), Some(PINNED_CHECKER_ID));
    assert_eq!(sel(&bento).checker_id.as_deref(), Some(PINNED_CHECKER_ID));
    assert_eq!(sk.helixtest_git_sha.as_deref(), Some(PINNED_HELIXTEST_SHA));
    assert_eq!(
        bento.helixtest_git_sha.as_deref(),
        Some(PINNED_HELIXTEST_SHA)
    );

    assert_eq!(
        sel(&sk).detected_version.as_deref(),
        Some("1.3.0experimental")
    );
    assert_eq!(sel(&sk).selected_version.as_deref(), Some("1.4.0"));
    assert!(sel(&sk).verified_version.is_none());
    assert!(!ga4gh_requirement_is_verified(&sk));
    assert_eq!(cov(&sk).state, CoverageState::Blocked);
    assert_eq!(
        check_status(&sk, "drs.object.reachable"),
        Some(VerificationStatus::Pass)
    );
    assert_eq!(
        check_status(&sk, "drs.object.schema"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check_status(&sk, "drs.object.checksum"),
        Some(VerificationStatus::Skip)
    );
    assert_eq!(
        check_status(&sk, "drs.object.range"),
        Some(VerificationStatus::Skip)
    );
    assert_eq!(
        check_status(&sk, "drs.object.not_found"),
        Some(VerificationStatus::Pass)
    );
    assert_eq!(
        check_status(&sk, "drs.object.schema.openapi"),
        Some(VerificationStatus::Pass)
    );
    assert_eq!(
        claim(&sk, ClaimKind::Ga4ghRequirement),
        ClaimStatus::NotVerified
    );

    assert_eq!(sel(&bento).detected_version.as_deref(), Some("1.4.0"));
    assert_eq!(sel(&bento).selected_version.as_deref(), Some("1.4.0"));
    assert_eq!(sel(&bento).verified_version.as_deref(), Some("1.4.0"));
    assert!(ga4gh_requirement_is_verified(&bento));
    assert_eq!(cov(&bento).state, CoverageState::Partial);
    assert!(cov(&bento).required_complete);
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
        "drs.object.schema.openapi",
    ] {
        assert_eq!(
            check_status(&bento, id),
            Some(VerificationStatus::Pass),
            "{id}"
        );
    }
    assert_eq!(
        cov(&bento)
            .row("drs.security.authorization")
            .unwrap()
            .classification,
        CoverageClass::Unevaluated
    );
    assert!(
        !cov(&bento)
            .row("drs.security.authorization")
            .unwrap()
            .executed
    );
    assert_eq!(cov(&sk).unevaluated, cov(&bento).unevaluated);
    assert_eq!(cov(&sk).out_of_scope, cov(&bento).out_of_scope);

    // Isolation: re-read Starter Kit after Bento was loaded.
    let sk_again = load_live_json("starter-kit.json").expect("sk reread");
    assert_eq!(
        sel(&sk_again).target_execution_id,
        sel(&sk).target_execution_id
    );
    assert_eq!(sel(&sk_again).verified_version, sel(&sk).verified_version);
    assert_eq!(
        check_status(&sk_again, "drs.object.schema"),
        Some(VerificationStatus::Fail)
    );

    // B12-T1: forge verified_version on Starter Kit.
    let mut t1 = sk.clone();
    tamper_selection(&mut t1, |s| s.verified_version = Some("1.4.0".into()));
    assert_integrity_err(&t1);

    // B12-T2: forge coverage.state complete.
    let mut t2 = bento.clone();
    t2.coverage.as_mut().unwrap().state = CoverageState::Complete;
    assert_integrity_err(&t2);

    // B12-T3: forge coverage_id.
    let mut t3 = bento.clone();
    t3.coverage.as_mut().unwrap().coverage_id = Some("0".repeat(64));
    assert_integrity_err(&t3);

    // B12-T4: forge execution_id.
    let mut t4 = bento.clone();
    tamper_selection(&mut t4, |s| s.execution_id = Some("0".repeat(64)));
    assert_integrity_err(&t4);

    // B12-T5: change target identity, keep recorded teid.
    let mut t5 = sk.clone();
    if let Some(id) = t5.target.identity.as_mut() {
        id.target_id = "forged-target".into();
        id.verified.target_id = "forged-target".into();
    }
    assert_integrity_err(&t5);

    // B12-T6: rename Starter Kit as Bento. Must not create Bento evidence.
    let mut t6 = sk.clone();
    if let Some(id) = t6.target.identity.as_mut() {
        id.implementation_name = Some("bento_drs".into());
        id.declared.implementation_name = Some("bento_drs".into());
        id.target_id = BENTO_TARGET_ID.into();
        id.declared.target_id = Some(BENTO_TARGET_ID.into());
        id.verified.target_id = BENTO_TARGET_ID.into();
    }
    assert!(!ga4gh_requirement_is_verified(&t6));
    assert!(sel(&t6).verified_version.is_none());
    assert_eq!(
        check_status(&t6, "drs.object.schema"),
        Some(VerificationStatus::Fail)
    );
    assert_ne!(t6.drs_fixture.as_ref().unwrap().object_id, BENTO_OBJECT_ID);
    assert_integrity_err(&t6);

    // B12-T7: replace endpoint.
    let mut t7 = bento.clone();
    if let Some(id) = t7.target.identity.as_mut() {
        id.endpoint = "http://127.0.0.1:9".into();
        id.verified.endpoint = "http://127.0.0.1:9".into();
    }
    assert_integrity_err(&t7);

    // B12-T8: keep PASS in join, delete executed evidence.
    let mut t8 = bento.clone();
    t8.executed.retain(|r| r.id != "drs.object.reachable");
    assert_integrity_err(&t8);

    // B12-T9: SKIP fixture_unavailable → PASS.
    let mut t9 = sk.clone();
    let mut row = t9
        .skipped
        .iter()
        .find(|r| r.id == "drs.object.checksum")
        .cloned()
        .expect("sk checksum skip");
    t9.skipped.retain(|r| r.id != "drs.object.checksum");
    row.status = VerificationStatus::Pass;
    row.attribution = None;
    t9.executed.push(row);
    assert_integrity_err(&t9);

    // B12-T10: UNEVALUATED → PASS.
    let mut t10 = bento.clone();
    set_coverage_row_pass(&mut t10, "drs.op.service_info");
    assert_integrity_err(&t10);

    // B12-T11: OUT_OF_SCOPE → PASS.
    let mut t11 = bento.clone();
    set_coverage_row_pass(&mut t11, "helix.helios");
    t11.coverage
        .as_mut()
        .unwrap()
        .rows
        .iter_mut()
        .find(|r| r.id == "helix.helios")
        .unwrap()
        .classification = CoverageClass::Normative;
    assert_integrity_err(&t11);

    // B12-T12: authorization verified while AUTHZ_ENABLED=false config is unevaluated.
    let mut t12 = bento.clone();
    set_coverage_row_pass(&mut t12, "drs.security.authorization");
    t12.coverage
        .as_mut()
        .unwrap()
        .rows
        .iter_mut()
        .find(|r| r.id == "drs.security.authorization")
        .unwrap()
        .classification = CoverageClass::Normative;
    assert_integrity_err(&t12);

    // Serialized coverage.state=complete still fails.
    let json = verify_json(&bento).unwrap();
    let mut value: Value = serde_json::from_str(&json).unwrap();
    value["coverage"]["state"] = Value::String("complete".into());
    let tampered: VerificationRun = serde_json::from_value(value).unwrap();
    assert_integrity_err(&tampered);
}
