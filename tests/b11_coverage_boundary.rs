// SPDX-License-Identifier: Apache-2.0
//! B11 verification-boundary / coverage-completeness gate.
//! Not a score. Not certification. Not HELIOS.

use helix::claim_integrity::{
    finalize_run, ga4gh_requirement_is_verified, tamper_selection, validate_claim_integrity,
};
use helix::claims::{evaluate, ClaimBlockCode, ClaimKind, ClaimStatus};
use helix::coverage::{
    compiled_contract_is_sound, coverage_canonical, coverage_id, CoverageClass, CoverageReport,
    CoverageState,
};
use helix::differential::differential_from_runs;
use helix::fixture::DrsVerifyFixture;
use helix::guardrails::check_run;
use helix::model::{VerificationRun, VerificationStatus};
use helix::profile::ProfileId;
use helix::report::verify_json;
use helix::standards::{catalog_id, DRS_140_CONTRACT};
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifyOutcome, VerifySelection};
use serde_json::Value;

mod support;
use support::mock_b10::{
    start_m1_missing_access_methods, start_m2_size_is_string, start_m5_range_returns_200,
    start_outside_closure,
};
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static B11_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn declared(id: &str, name: &str, ver: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::Mock,
        implementation_name: Some(name.into()),
        implementation_version: Some(ver.into()),
        ..DeclaredTarget::default()
    }
}

async fn verify_named(url: &str, id: &str) -> VerifyOutcome {
    verify_with_options(url, versioned(declared(id, "b11-mock-drs", "0.0.0")))
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
    run.coverage = None;
    run.claim_join = None;
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

fn drop_check(run: &mut VerificationRun, id: &str) {
    run.executed.retain(|r| r.id != id);
    run.skipped.retain(|r| r.id != id);
    run.recompute_summary();
}

fn cov(run: &VerificationRun) -> &CoverageReport {
    run.coverage.as_ref().expect("coverage")
}

fn sel(run: &VerificationRun) -> &helix::model::StandardSelection {
    run.standard_selection.as_ref().expect("standard_selection")
}

fn class_of(run: &VerificationRun, id: &str) -> CoverageClass {
    cov(run).row(id).expect(id).classification
}

#[tokio::test]
async fn b11_t1_known_good_drs_140_mock_verifies() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_named(&mock.drs_url(), "b11-t1").await;
    assert!(ga4gh_requirement_is_verified(&outcome.run));
    assert_eq!(sel(&outcome.run).verified_version.as_deref(), Some("1.4.0"));
    let c = cov(&outcome.run);
    assert!(c.required_complete);
    assert_eq!(c.state, CoverageState::Partial);
    assert_eq!(
        class_of(&outcome.run, "drs.object.schema.openapi"),
        CoverageClass::Normative
    );
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
    ] {
        assert_eq!(
            class_of(&outcome.run, id),
            CoverageClass::EvaluatedNonNormative
        );
    }
    compiled_contract_is_sound(&DRS_140_CONTRACT).unwrap();
}

#[tokio::test]
async fn b11_t2_coverage_id_is_deterministic() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let a = verify_named(&mock.drs_url(), "b11-t2a").await;
    let b = verify_named(&mock.drs_url(), "b11-t2b").await;
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
    let id = cov(&a.run).coverage_id.as_deref().expect("coverage_id");
    assert_eq!(id.len(), 64);
}

#[tokio::test]
async fn b11_t3_changing_coverage_contract_changes_identity() {
    let checker = helix::checker::executed_checker_id();
    let a = coverage_canonical(&DRS_140_CONTRACT, &checker, Some("bind"), Some("pack"));
    let mut b = a.clone();
    b.push_str("row=forged.required|out_of_scope|true|true||none\n");
    assert_ne!(a, b);
    assert_ne!(
        coverage_id(&DRS_140_CONTRACT, &checker, Some("bind"), Some("pack")),
        coverage_id(
            &DRS_140_CONTRACT,
            &checker,
            Some("bind-other"),
            Some("pack")
        )
    );
}

#[tokio::test]
async fn b11_t4_changing_target_does_not_change_coverage_id() {
    let _g = B11_LOCK.lock().await;
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let a = verify_named(&mock_a.drs_url(), "starter-kit-standin").await;
    let b = verify_with_options(
        &mock_b.drs_url(),
        versioned(declared("bento-standin", "bento_drs", "v0.21.5")),
    )
    .await
    .unwrap();
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
}

#[tokio::test]
async fn b11_t5_changing_target_changes_target_execution_id() {
    let _g = B11_LOCK.lock().await;
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let a = verify_named(&mock_a.drs_url(), "b11-t5a").await;
    let b = verify_named(&mock_b.drs_url(), "b11-t5b").await;
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b.run).target_execution_id
    );
    assert_eq!(sel(&a.run).execution_id, sel(&b.run).execution_id);
}

#[tokio::test]
async fn b11_t6_forged_coverage_complete_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t6").await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    assert!(!cov(&run).required_complete);
    if let Some(c) = run.coverage.as_mut() {
        c.required_complete = true;
        c.state = CoverageState::Complete;
    }
    let err = validate_claim_integrity(&run).unwrap_err().to_string();
    assert!(err.contains("coverage"), "{err}");
}

#[tokio::test]
async fn b11_t7_forged_coverage_id_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t7").await.run;
    if let Some(c) = run.coverage.as_mut() {
        c.coverage_id = Some("known-good".into());
    }
    let err = validate_claim_integrity(&run).unwrap_err().to_string();
    assert!(err.contains("coverage"), "{err}");
}

#[tokio::test]
async fn b11_t8_forged_out_of_scope_for_required_normative_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t8").await.run;
    let row = run
        .coverage
        .as_mut()
        .unwrap()
        .rows
        .iter_mut()
        .find(|r| r.id == "drs.object.schema.openapi")
        .unwrap();
    row.classification = CoverageClass::OutOfScope;
    row.required_for_ga4gh_requirement = false;
    let err = validate_claim_integrity(&run).unwrap_err().to_string();
    assert!(err.contains("coverage"), "{err}");
}

#[tokio::test]
async fn b11_t9_unevaluated_normative_cannot_produce_verified() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t9").await.run;
    drop_check(&mut run, "drs.object.schema.openapi");
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(sel(&run).verified_version.is_none());
    assert!(!cov(&run).required_complete);
    assert_eq!(cov(&run).state, CoverageState::Blocked);
    assert!(evaluate(&run)
        .get(ClaimKind::Ga4ghRequirement)
        .has_block(ClaimBlockCode::CatalogCheckMissing));
}

#[tokio::test]
async fn b11_t10_non_normative_cannot_independently_produce_ga4gh_verified() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t10").await.run;
    drop_check(&mut run, "drs.object.schema.openapi");
    restamp(&mut run);
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
    ] {
        assert_eq!(
            run.executed.iter().find(|r| r.id == id).map(|r| r.status),
            Some(VerificationStatus::Pass)
        );
    }
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[tokio::test]
async fn b11_t11_required_fixture_skip_prevents_full_verification() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t11").await.run;
    set_check_status(
        &mut run,
        "drs.object.checksum",
        VerificationStatus::Skip,
        "fixture_unavailable: object not present",
    );
    restamp(&mut run);
    let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
    assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
    assert!(ga4gh.has_block(ClaimBlockCode::RequiredEvidenceUnavailable));
    assert!(!cov(&run).required_complete);
    assert_eq!(cov(&run).state, CoverageState::Blocked);
}

#[tokio::test]
async fn b11_t12_unsupported_optional_does_not_revoke_when_not_required() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t12").await.run;
    assert!(ga4gh_requirement_is_verified(&run));
    let access = cov(&run).row("drs.op.access").expect("access");
    assert_eq!(access.classification, CoverageClass::Unevaluated);
    assert!(!access.required_for_ga4gh_requirement);
    assert!(!access.executed);
}

#[test]
fn b11_t13_required_check_cannot_be_compiled_out_of_scope() {
    compiled_contract_is_sound(&DRS_140_CONTRACT).unwrap();
    for id in helix::coverage::required_ga4gh_ids(&DRS_140_CONTRACT) {
        assert!(
            DRS_140_CONTRACT.checks.iter().any(|c| c.id == id),
            "{id} required but not a catalog check"
        );
    }
}

#[tokio::test]
async fn b11_t14_target_metadata_cannot_expand_coverage() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_named(&url, "b11-t14a").await;
    let b = verify_with_options(
        &url,
        versioned(DeclaredTarget {
            target_id: Some("expanded".into()),
            kind: TargetKind::RealIndependentLocalImplementation,
            implementation_name: Some("bento_drs".into()),
            implementation_version: Some("99.0.0".into()),
            ..DeclaredTarget::default()
        }),
    )
    .await
    .unwrap();
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
    assert_eq!(cov(&a.run).rows.len(), cov(&b.run).rows.len());
}

#[tokio::test]
async fn b11_t15_implementation_name_version_cannot_create_verification() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t15").await.run;
    drop_check(&mut run, "drs.object.schema.openapi");
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    if let Some(id) = run.target.identity.as_mut() {
        id.implementation_name = Some("certified-drs".into());
        id.implementation_version = Some("1.4.0".into());
    }
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(sel(&run).verified_version.is_none());
}

#[tokio::test]
async fn b11_t16_starter_kit_and_bento_share_coverage_contract_identity() {
    let _g = B11_LOCK.lock().await;
    let sk = start_mock_ga4gh_drs().await;
    let bento = start_mock_ga4gh_drs().await;
    let a = verify_with_options(
        &sk.drs_url(),
        versioned(declared(
            "ga4gh-starter-kit-drs",
            "ga4gh-starter-kit-drs",
            "1.3.0experimental",
        )),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &bento.drs_url(),
        versioned(declared("bento_drs", "bento_drs", "v0.21.5")),
    )
    .await
    .unwrap();
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
}

#[tokio::test]
async fn b11_t17_starter_kit_and_bento_share_execution_id() {
    let _g = B11_LOCK.lock().await;
    let sk = start_mock_ga4gh_drs().await;
    let bento = start_mock_ga4gh_drs().await;
    let a = verify_named(&sk.drs_url(), "ga4gh-starter-kit-drs").await;
    let b = verify_named(&bento.drs_url(), "bento_drs").await;
    assert_eq!(sel(&a.run).execution_id, sel(&b.run).execution_id);
}

#[tokio::test]
async fn b11_t18_starter_kit_and_bento_have_distinct_target_execution_id() {
    let _g = B11_LOCK.lock().await;
    let sk = start_mock_ga4gh_drs().await;
    let bento = start_mock_ga4gh_drs().await;
    let a = verify_named(&sk.drs_url(), "ga4gh-starter-kit-drs").await;
    let b = verify_named(&bento.drs_url(), "bento_drs").await;
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b.run).target_execution_id
    );
}

#[tokio::test]
async fn b11_t19_b9_differential_semantics_remain() {
    let _g = B11_LOCK.lock().await;
    let a_mock = start_mock_ga4gh_drs().await;
    let b_mock = start_m1_missing_access_methods().await;
    let a = verify_named(&a_mock.drs_url(), "b11-t19a").await;
    let b = verify_named(&b_mock.drs_url(), "b11-t19b").await;
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
    let diff = differential_from_runs(&a.run, &b.run);
    assert!(diff.same_specification);
    assert!(diff.same_verifier);
    assert!(!diff.creates_verification);
    assert_eq!(diff.ranking_semantics, "absent");
    assert!(!diff.contains_ranking_semantics());
}

#[tokio::test]
async fn b11_t20_b10_m1_mutation_remains_claim_revoking() {
    let _g = B11_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let mutated = start_m1_missing_access_methods().await;
    let a = verify_named(&golden.drs_url(), "b11-t20a").await;
    let b = verify_named(&mutated.drs_url(), "b11-t20b").await;
    assert!(ga4gh_requirement_is_verified(&a.run));
    assert!(!ga4gh_requirement_is_verified(&b.run));
    assert_eq!(cov(&a.run).coverage_id, cov(&b.run).coverage_id);
    assert_eq!(sel(&a.run).execution_id, sel(&b.run).execution_id);
}

#[tokio::test]
async fn b11_t21_b10_m2_mutation_remains_claim_revoking() {
    let _g = B11_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let mutated = start_m2_size_is_string().await;
    let a = verify_named(&golden.drs_url(), "b11-t21a").await;
    let b = verify_named(&mutated.drs_url(), "b11-t21b").await;
    assert!(ga4gh_requirement_is_verified(&a.run));
    assert!(!ga4gh_requirement_is_verified(&b.run));
}

#[tokio::test]
async fn b11_t22_b10_m5_range_mutation_remains_claim_revoking() {
    let _g = B11_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let mutated = start_m5_range_returns_200().await;
    let a = verify_named(&golden.drs_url(), "b11-t22a").await;
    let b = verify_named(&mutated.drs_url(), "b11-t22b").await;
    assert!(ga4gh_requirement_is_verified(&a.run));
    assert!(!ga4gh_requirement_is_verified(&b.run));
}

#[tokio::test]
async fn b11_t23_outside_closure_mutation_does_not_alter_in_scope_claim() {
    let _g = B11_LOCK.lock().await;
    let outside = start_outside_closure().await;
    let run = verify_named(&outside.drs_url(), "b11-t23").await.run;
    assert!(ga4gh_requirement_is_verified(&run));
    let row = cov(&run).row("target.non_drs_http").expect("non_drs");
    assert_eq!(row.classification, CoverageClass::OutOfScope);
    assert!(!row.required_for_ga4gh_requirement);
    assert_eq!(row.claim_contribution, "none");
}

#[tokio::test]
async fn b11_t24_authorization_disabled_cannot_emit_authorization_verified() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t24").await.run;
    assert_eq!(
        evaluate(&run).get(ClaimKind::Security).status,
        ClaimStatus::NotVerified
    );
    let auth = cov(&run)
        .row("drs.security.authorization")
        .expect("authorization");
    assert_eq!(auth.classification, CoverageClass::Unevaluated);
    assert_ne!(auth.result, Some(VerificationStatus::Pass));
    assert!(!auth.executed);
}

#[tokio::test]
async fn b11_t25_checksum_coverage_cannot_be_overstated() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t25").await.run;
    assert_eq!(
        run.executed
            .iter()
            .find(|r| r.id == "drs.object.checksum")
            .map(|r| r.status),
        Some(VerificationStatus::Pass)
    );
    let other = cov(&run)
        .row("drs.checksum.types.other")
        .expect("other checksums");
    assert_eq!(other.classification, CoverageClass::Unevaluated);
    assert!(!other.executed);
}

#[tokio::test]
async fn b11_t26_unevaluated_optional_endpoint_never_silent_pass() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t26").await.run;
    let mut fake = run.executed[0].clone();
    fake.id = "drs.op.access".into();
    fake.code = "HLX-DRS-ACCESS".into();
    fake.status = VerificationStatus::Pass;
    run.executed.push(fake);
    run.recompute_summary();
    restamp(&mut run);
    let access = cov(&run).row("drs.op.access").unwrap();
    assert_eq!(access.classification, CoverageClass::Unevaluated);
    assert_ne!(access.result, Some(VerificationStatus::Pass));
    assert!(!access.executed);
}

#[tokio::test]
async fn b11_t27_json_roundtrip_preserves_coverage_state_and_identity() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t27").await.run;
    let json = verify_json(&run).unwrap();
    let back: VerificationRun = serde_json::from_str(&json).unwrap();
    assert_eq!(cov(&run).coverage_id, cov(&back).coverage_id);
    assert_eq!(cov(&run).state, cov(&back).state);
    assert_eq!(cov(&run).required_complete, cov(&back).required_complete);
    validate_claim_integrity(&back).unwrap();
}

#[tokio::test]
async fn b11_t28_tampering_serialized_coverage_fails_integrity() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t28").await.run;
    let json = verify_json(&run).unwrap();
    let mut value: Value = serde_json::from_str(&json).unwrap();
    value["coverage"]["state"] = Value::String("complete".into());
    value["coverage"]["required_complete"] = Value::Bool(true);
    let tampered: VerificationRun = serde_json::from_value(value).unwrap();
    assert!(validate_claim_integrity(&tampered).is_err());
    assert!(check_run(&tampered).is_err());
}

#[tokio::test]
async fn b11_t29_stale_coverage_from_another_standard_version_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t29").await.run;
    if let Some(c) = run.coverage.as_mut() {
        c.selected_version = Some("1.5.0".into());
        c.coverage_id = Some(coverage_id(
            &DRS_140_CONTRACT,
            &helix::checker::executed_checker_id(),
            c.binding_id.as_deref(),
            Some("stale-1.5.0-pack"),
        ));
    }
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b11_t30_stale_coverage_from_another_checker_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t30").await.run;
    let stale = coverage_id(
        &DRS_140_CONTRACT,
        "helixtest-drs:deadbeef",
        sel(&run).binding_id.as_deref(),
        sel(&run).pack_integrity_sha256.as_deref(),
    );
    if let Some(c) = run.coverage.as_mut() {
        c.coverage_id = Some(stale.clone());
        c.checker_id = Some("helixtest-drs:deadbeef".into());
    }
    if let Some(j) = run.claim_join.as_mut() {
        j.coverage_id = Some(stale);
    }
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b11_t31_stale_coverage_from_another_binding_catalog_fails() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-t31").await.run;
    let expected_catalog = catalog_id(&DRS_140_CONTRACT);
    let stale = coverage_id(
        &DRS_140_CONTRACT,
        sel(&run).checker_id.as_deref().unwrap(),
        Some("0000000000000000000000000000000000000000000000000000000000000000"),
        sel(&run).pack_integrity_sha256.as_deref(),
    );
    if let Some(c) = run.coverage.as_mut() {
        c.coverage_id = Some(stale.clone());
        c.binding_id =
            Some("0000000000000000000000000000000000000000000000000000000000000000".into());
        c.catalog_id = Some(expected_catalog);
    }
    if let Some(j) = run.claim_join.as_mut() {
        j.coverage_id = Some(stale);
    }
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b11_t32_coverage_identity_excludes_target_url() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let run = verify_named(&mock.drs_url(), "b11-t32").await.run;
    let canon = coverage_canonical(
        &DRS_140_CONTRACT,
        sel(&run).checker_id.as_deref().unwrap(),
        sel(&run).binding_id.as_deref(),
        sel(&run).pack_integrity_sha256.as_deref(),
    );
    assert!(!canon.contains("http://"));
    assert!(!canon.contains(&mock.drs_url()));
    assert!(!canon.contains("timestamp"));
    assert_eq!(
        cov(&run).coverage_id.as_deref(),
        Some(
            coverage_id(
                &DRS_140_CONTRACT,
                sel(&run).checker_id.as_deref().unwrap(),
                sel(&run).binding_id.as_deref(),
                sel(&run).pack_integrity_sha256.as_deref(),
            )
            .as_str()
        )
    );
}

#[tokio::test]
async fn b11_no_silent_coverage_inflation() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-inflate").await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    let blocked_id = cov(&run).coverage_id.clone();
    assert!(!ga4gh_requirement_is_verified(&run));
    tamper_selection(&mut run, |s| {
        s.detected_version = Some("1.4.0".into());
        s.requested_version = Some("1.4.0".into());
    });
    if let Some(id) = run.target.identity.as_mut() {
        id.target_id = "promoted".into();
        id.target_kind = TargetKind::RealIndependentLocalImplementation;
        id.implementation_name = Some("bento_drs".into());
        id.implementation_version = Some("1.4.0".into());
    }
    if let Some(fx) = run.drs_fixture.as_mut() {
        *fx = DrsVerifyFixture::default_catalog();
    }
    restamp(&mut run);
    assert_eq!(cov(&run).coverage_id, blocked_id);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    assert!(sel(&run).verified_version.is_none());
}

#[tokio::test]
async fn b11_no_silent_coverage_reduction() {
    let _g = B11_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut run = verify_named(&mock.drs_url(), "b11-reduce").await.run;
    assert!(ga4gh_requirement_is_verified(&run));
    if let Some(id) = run.target.identity.as_mut() {
        id.implementation_name = Some("limited-drs".into());
    }
    restamp(&mut run);
    assert_eq!(
        class_of(&run, "drs.object.schema.openapi"),
        CoverageClass::Normative
    );
    assert!(
        cov(&run)
            .row("drs.object.schema.openapi")
            .unwrap()
            .required_for_ga4gh_requirement
    );
    assert!(ga4gh_requirement_is_verified(&run));
}
