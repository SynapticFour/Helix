// SPDX-License-Identifier: Apache-2.0
//! B8 claim integrity, evidence join, tamper, skip, and attribution tests.
//! Not certification. Not HELIOS.

use helix::claim_integrity::{
    finalize_run, ga4gh_requirement_is_verified, reproduction_tuple, tamper_selection,
    validate_claim_integrity, ClaimJoin,
};
use helix::claims::{evaluate, ClaimBlockCode, ClaimKind, ClaimStatus};
use helix::diagnostics::DiagnosticCategory;
use helix::fixture::DrsVerifyFixture;
use helix::guardrails::check_run;
use helix::identity::spec;
use helix::model::{
    Target, VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
};
use helix::profile::ProfileId;
use helix::report::verify_json;
use helix::standards::{
    binding_id, catalog_id, contract_for, declared_checker_id, execution_id, DRS_140_CONTRACT,
    DRS_OPENAPI_SPECSOURCE_CHECK,
};
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify, verify_with_options, VerifyOptions, VerifySelection};
use serde_json::Value;

mod support;
use support::mock_ga4gh_drs::{
    start_mock_ga4gh_drs, start_mock_ga4gh_drs_advertising, start_mock_invalid_drs_object,
};

static B8_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
        ..DeclaredTarget::default()
    }
}

async fn versioned_mock() -> helix::verify::VerifyOutcome {
    let mock = start_mock_ga4gh_drs().await;
    verify_with_options(&mock.drs_url(), versioned(mock_declared("b8-mock")))
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

fn claim_codes(run: &VerificationRun, kind: ClaimKind) -> Vec<ClaimBlockCode> {
    evaluate(run).get(kind).block_codes()
}

#[tokio::test]
async fn b8_t1_verified_requires_all_mandatory_checks() {
    let _g = B8_LOCK.lock().await;
    let outcome = versioned_mock().await;
    assert!(
        ga4gh_requirement_is_verified(&outcome.run),
        "honest catalog PASS must derive ga4gh_requirement VERIFIED"
    );
    assert_eq!(
        outcome
            .run
            .standard_selection
            .as_ref()
            .unwrap()
            .verified_version
            .as_deref(),
        Some("1.4.0")
    );

    let mut broken = outcome.run.clone();
    set_check_status(
        &mut broken,
        "drs.object.not_found",
        VerificationStatus::Fail,
        "got 200",
    );
    restamp(&mut broken);
    assert!(!ga4gh_requirement_is_verified(&broken));
    assert!(broken
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
}

#[tokio::test]
async fn b8_t2_mandatory_skip_prevents_verified() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
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
    assert!(run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
}

#[tokio::test]
async fn b8_t3_mandatory_error_prevents_verified() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    set_check_status(
        &mut run,
        "drs.object.range",
        VerificationStatus::Error,
        "HelixTest adapter error: boom",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(
        claim_codes(&run, ClaimKind::Ga4ghRequirement).contains(&ClaimBlockCode::CatalogCheckError)
    );
}

#[tokio::test]
async fn b8_t4_target_failure_prevents_verified() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(claim_codes(&run, ClaimKind::Ga4ghRequirement)
        .contains(&ClaimBlockCode::CatalogCheckFailed));
    let row = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::TargetFailure));
}

#[tokio::test]
async fn b8_t5_spec_failure_prevents_verified() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    set_check_status(
        &mut run,
        "drs.object.schema.openapi",
        VerificationStatus::Fail,
        "body mismatch",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(claim_codes(&run, ClaimKind::Ga4ghRequirement)
        .contains(&ClaimBlockCode::NormativeCheckFailed));
    let row = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi")
        .unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::SpecFailure));
}

#[tokio::test]
async fn b8_t6_stale_checker_identity_invalidates_claim() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    assert!(ga4gh_requirement_is_verified(&run));
    tamper_selection(&mut run, |sel| {
        sel.checker_id = Some("helixtest-drs:deadbeef".repeat(2));
    });
    run.claim_join = Some(ClaimJoin::from_run(&run));
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(claim_codes(&run, ClaimKind::Ga4ghRequirement)
        .contains(&ClaimBlockCode::CheckerIdentityMismatch));
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b8_t7_stale_pack_identity_invalidates_claim() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    let before = run
        .standard_selection
        .as_ref()
        .unwrap()
        .execution_id
        .clone();
    tamper_selection(&mut run, |sel| {
        sel.pack_integrity_sha256 = Some("ab".repeat(32));
    });
    let after = helix::standards::execution_id(
        run.standard_selection
            .as_ref()
            .unwrap()
            .standards_registry_entry
            .as_deref()
            .unwrap(),
        run.standard_selection
            .as_ref()
            .unwrap()
            .pack_integrity_sha256
            .as_deref()
            .unwrap(),
        run.standard_selection
            .as_ref()
            .unwrap()
            .schema_document_sha256
            .as_deref()
            .unwrap(),
        run.standard_selection
            .as_ref()
            .unwrap()
            .schema_component_sha256
            .as_deref()
            .unwrap(),
        run.standard_selection
            .as_ref()
            .unwrap()
            .checker_id
            .as_deref()
            .unwrap(),
        run.standard_selection
            .as_ref()
            .unwrap()
            .schema_entry
            .as_deref()
            .unwrap(),
        "DrsObject",
    );
    assert_ne!(before.as_deref(), Some(after.as_str()));
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b8_t8_stale_schema_identity_invalidates_claim() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    tamper_selection(&mut run, |sel| {
        sel.schema_document_sha256 = Some("cd".repeat(32));
    });
    assert!(validate_claim_integrity(&run).is_err());
    tamper_selection(&mut run, |sel| {
        sel.schema_component_sha256 = Some("ef".repeat(32));
    });
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b8_t9_stale_binding_invalidates_claim() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    tamper_selection(&mut run, |sel| {
        sel.binding_id = Some("00".repeat(32));
    });
    assert!(claim_codes(&run, ClaimKind::Ga4ghRequirement)
        .contains(&ClaimBlockCode::BindingIdentityMismatch));
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b8_t10_stale_catalog_invalidates_claim() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    tamper_selection(&mut run, |sel| {
        sel.catalog_id = Some("11".repeat(32));
    });
    assert!(claim_codes(&run, ClaimKind::Ga4ghRequirement)
        .contains(&ClaimBlockCode::CatalogIdentityMismatch));
    assert!(!ga4gh_requirement_is_verified(&run));
    assert!(validate_claim_integrity(&run).is_err());
}

#[tokio::test]
async fn b8_t11_target_identity_changes_target_execution_id() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(&url, versioned(mock_declared("t-a")))
        .await
        .unwrap();
    let b = verify_with_options(&url, versioned(mock_declared("t-b")))
        .await
        .unwrap();
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
    assert_ne!(
        reproduction_tuple(&a.run).target_id,
        reproduction_tuple(&b.run).target_id
    );
}

#[tokio::test]
async fn b8_t12_fixture_identity_changes_target_execution_id() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let mut opts_a = versioned(mock_declared("fix-a"));
    opts_a.drs_fixture = DrsVerifyFixture::default_catalog();
    let mut opts_b = versioned(mock_declared("fix-a"));
    opts_b.drs_fixture =
        DrsVerifyFixture::operator_declared("test-object-1".into(), Some("ff".repeat(32)))
            .expect("fixture");
    let a = verify_with_options(&url, opts_a).await.unwrap();
    let b = verify_with_options(&url, opts_b).await.unwrap();
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
}

#[tokio::test]
async fn b8_t13_verifier_identity_changes_execution_id() {
    let contract = contract_for("ga4gh.drs.1.4.0").unwrap();
    let p = "aa".repeat(32);
    let d = "bb".repeat(32);
    let c = "cc".repeat(32);
    let a = execution_id(
        contract.pack_id,
        &p,
        &d,
        &c,
        "helixtest-drs:aaa",
        contract.schema_entry,
        contract.schema_component,
    );
    let b = execution_id(
        contract.pack_id,
        &p,
        &d,
        &c,
        "helixtest-drs:bbb",
        contract.schema_entry,
        contract.schema_component,
    );
    assert_ne!(a, b);
}

#[tokio::test]
async fn b8_t14_detected_version_cannot_create_verification() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs_advertising("1.3.0experimental").await;
    let unversioned = verify(&mock.drs_url()).await.unwrap();
    let drs_row = unversioned
        .run
        .executed
        .iter()
        .chain(unversioned.run.skipped.iter())
        .find(|r| r.service == "drs")
        .expect("drs row");
    assert_eq!(
        drs_row.detected_version.as_deref(),
        Some("1.3.0experimental")
    );
    assert!(unversioned
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert!(!ga4gh_requirement_is_verified(&unversioned.run));

    let none_detected = start_mock_ga4gh_drs().await;
    let none_out = verify(&none_detected.drs_url()).await.unwrap();
    let none_row = none_out
        .run
        .executed
        .iter()
        .chain(none_out.run.skipped.iter())
        .find(|r| r.service == "drs")
        .expect("drs row");
    assert!(none_row.detected_version.is_none());
    assert!(none_out
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());

    let versioned_out = verify_with_options(&mock.drs_url(), versioned(mock_declared("det")))
        .await
        .unwrap();
    let sel = versioned_out.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.detected_version.as_deref(), Some("1.3.0experimental"));
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert_ne!(sel.detected_version, sel.verified_version);
    if let Some(v) = sel.verified_version.as_deref() {
        assert_eq!(v, "1.4.0");
        assert!(ga4gh_requirement_is_verified(&versioned_out.run));
    }
}

#[tokio::test]
async fn b8_t15_declared_version_cannot_create_verification() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_invalid_drs_object().await;
    let declared = DeclaredTarget {
        target_id: Some("declared-lie".into()),
        kind: TargetKind::Mock,
        standard_version: Some("1.4.0".into()),
        ..DeclaredTarget::default()
    };
    let out = verify_with_options(&mock.uri(), versioned(declared))
        .await
        .unwrap();
    let sel = out.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    assert!(!ga4gh_requirement_is_verified(&out.run));
}

#[tokio::test]
async fn b8_t16_operator_implementation_version_cannot_create_verification() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let declared = DeclaredTarget {
        target_id: Some("impl-lie".into()),
        kind: TargetKind::Mock,
        implementation_version: Some("1.4.0".into()),
        ..DeclaredTarget::default()
    };
    let unversioned = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            declared_target: declared.clone(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        unversioned
            .run
            .target
            .identity
            .as_ref()
            .unwrap()
            .implementation_version
            .as_deref(),
        Some("1.4.0")
    );
    assert!(unversioned
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert!(!ga4gh_requirement_is_verified(&unversioned.run));
}

#[tokio::test]
async fn b8_t17_serialized_result_preserves_semantic_identity() {
    let _g = B8_LOCK.lock().await;
    let run = versioned_mock().await.run;
    check_run(&run).expect("emit");
    let text = verify_json(&run).expect("json");
    let v: Value = serde_json::from_str(&text).unwrap();
    let back: VerificationRun = serde_json::from_str(&text).unwrap();
    let sel = run.standard_selection.as_ref().unwrap();
    let bsel = back.standard_selection.as_ref().unwrap();
    assert_eq!(bsel.execution_id, sel.execution_id);
    assert_eq!(bsel.target_execution_id, sel.target_execution_id);
    assert_eq!(bsel.checker_id, sel.checker_id);
    assert_eq!(bsel.binding_id, sel.binding_id);
    assert_eq!(bsel.catalog_id, sel.catalog_id);
    assert_eq!(bsel.selected_version, sel.selected_version);
    assert_eq!(bsel.verified_version, sel.verified_version);
    assert_eq!(evaluate(&back), evaluate(&run));
    assert_eq!(back.claim_join, run.claim_join);
    assert_eq!(ClaimJoin::from_run(&back), run.claim_join.clone().unwrap());
    for (orig, got) in run.executed.iter().zip(back.executed.iter()) {
        assert_eq!(got.status, orig.status);
        assert_eq!(got.attribution, orig.attribution);
        assert_eq!(got.id, orig.id);
    }
    assert_eq!(
        run.drs_fixture.as_ref().map(|f| &f.object_id),
        back.drs_fixture.as_ref().map(|f| &f.object_id)
    );
    assert_eq!(v["claims"][0]["kind"], "ga4gh_requirement");
    assert_eq!(
        v["claims"][0]["status"],
        evaluate(&run)
            .get(ClaimKind::Ga4ghRequirement)
            .status
            .as_str()
    );
}

#[tokio::test]
async fn b8_t18_forged_verified_state_is_rejected() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "forged still claims verified",
    );
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = Some("1.4.0".into());
    }
    run.claim_join = Some(ClaimJoin::from_run(&run));
    assert!(!ga4gh_requirement_is_verified(&run));
    let err = validate_claim_integrity(&run).unwrap_err().to_string();
    assert!(
        err.contains("ga4gh_requirement") || err.contains("claim_join") || err.contains("verified"),
        "{err}"
    );
    assert!(check_run(&run).is_err());
}

#[tokio::test]
async fn b8_t19_forged_evidence_provenance_is_rejected() {
    let _g = B8_LOCK.lock().await;
    let mut run = versioned_mock().await.run;
    let mut join = run.claim_join.clone().unwrap();
    join.checker_id = Some("helixtest-drs:forged".into());
    join.claims[0].status = ClaimStatus::Verified;
    run.claim_join = Some(join);
    assert!(validate_claim_integrity(&run).is_err());
    assert!(check_run(&run).is_err());
}

#[test]
fn b8_t20_failure_attribution_is_causal() {
    let reachable = spec("drs.object.reachable");
    let mut transport = VerificationResult::fail(
        VerificationCheck::from_spec(reachable),
        "target unreachable; drs checks not executed",
    );
    helix::diagnostics::attach(&mut transport);
    helix::target::attach_attribution(&mut transport);
    assert_eq!(
        transport.attribution,
        Some(FailureAttribution::TransportFailure)
    );

    let mut http404 = VerificationResult::fail(
        VerificationCheck::from_spec(reachable),
        "Unexpected HTTP status: 404 Not Found",
    );
    helix::diagnostics::attach(&mut http404);
    helix::target::attach_attribution(&mut http404);
    assert_eq!(http404.attribution, Some(FailureAttribution::TargetFailure));
    assert_eq!(
        http404.diagnostic.as_ref().unwrap().likely_category,
        DiagnosticCategory::Reachability
    );

    let mut spec_fail = VerificationResult::fail(
        VerificationCheck::from_spec(spec("drs.object.schema.openapi")),
        "schema mismatch",
    );
    spec_fail.traceability.as_mut().unwrap().category = helix::standards::BindingKind::Normative;
    spec_fail.traceability.as_mut().unwrap().check_kind = helix::standards::BindingKind::Normative;
    helix::target::attach_attribution(&mut spec_fail);
    assert_eq!(spec_fail.attribution, Some(FailureAttribution::SpecFailure));

    let mut helix_err = VerificationResult::error(
        VerificationCheck::from_spec(spec("drs.object.schema")),
        "HelixTest adapter error: wall clock",
    );
    helix::target::attach_attribution(&mut helix_err);
    assert_eq!(
        helix_err.attribution,
        Some(FailureAttribution::HelixExecutionFailure)
    );

    let skip = VerificationResult::skip(
        VerificationCheck::from_spec(spec("drs.object.checksum")),
        "fixture_unavailable: configured object missing",
    );
    assert_eq!(
        skip.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );

    let unsupported = VerificationResult::skip(
        VerificationCheck::from_spec(spec("wes.run.scatter_gather")),
        "profile generic does not run scatter",
    );
    assert_eq!(
        unsupported.attribution,
        Some(FailureAttribution::UnsupportedTest)
    );
}

#[tokio::test]
async fn b8_t21_multi_target_results_remain_independent() {
    let _g = B8_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let bad = start_mock_invalid_drs_object().await;
    let good = verify_with_options(&mock.drs_url(), versioned(mock_declared("good")))
        .await
        .unwrap();
    let fail = verify_with_options(&bad.uri(), versioned(mock_declared("bad")))
        .await
        .unwrap();
    let sg = good.run.standard_selection.as_ref().unwrap();
    let sf = fail.run.standard_selection.as_ref().unwrap();
    assert_eq!(sg.execution_id, sf.execution_id);
    assert_ne!(sg.target_execution_id, sf.target_execution_id);
    assert!(ga4gh_requirement_is_verified(&good.run));
    assert!(!ga4gh_requirement_is_verified(&fail.run));
    assert_eq!(sg.verified_version.as_deref(), Some("1.4.0"));
    assert!(sf.verified_version.is_none());
}

#[test]
fn b8_t22_starter_kit_negative_result_remains_not_verified() {
    let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
    let mut sel = helix::model::StandardSelection::unversioned();
    sel.mode = "explicit".into();
    sel.selection_status = helix::standards::SELECTED.into();
    sel.standard = Some("drs".into());
    sel.detected_version = Some("1.3.0experimental".into());
    sel.selected_version = Some("1.4.0".into());
    sel.verified_version = None;
    sel.standards_registry_entry = Some("ga4gh.drs.1.4.0".into());
    sel.standards_source_commit = Some("36145d389e0a454428d1dac5c4a30870995fdd7c".into());
    sel.integrity_validated = true;
    sel.integrity_ok = Some(true);
    sel.pack_integrity_sha256 =
        Some("c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89".into());
    sel.schema_document_sha256 =
        Some("3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608".into());
    sel.schema_component_sha256 =
        Some("b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003".into());
    run.standard_selection = Some(sel);

    fn row(id: &str, status: VerificationStatus, msg: &str) -> VerificationResult {
        match status {
            VerificationStatus::Pass => {
                VerificationResult::pass(VerificationCheck::from_spec(spec(id)))
            }
            VerificationStatus::Fail => {
                VerificationResult::fail(VerificationCheck::from_spec(spec(id)), msg)
            }
            VerificationStatus::Skip => {
                VerificationResult::skip(VerificationCheck::from_spec(spec(id)), msg)
            }
            VerificationStatus::Error => {
                VerificationResult::error(VerificationCheck::from_spec(spec(id)), msg)
            }
        }
    }
    run.push_executed(row("drs.object.reachable", VerificationStatus::Pass, ""));
    run.push_executed(row(
        "drs.object.schema.openapi",
        VerificationStatus::Pass,
        "",
    ));
    run.push_executed(row(
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    ));
    run.push_skipped(row(
        "drs.object.checksum",
        VerificationStatus::Skip,
        "fixture_unavailable: bytes not advertised",
    ));
    run.push_skipped(row(
        "drs.object.range",
        VerificationStatus::Skip,
        "fixture_unavailable: bytes not advertised",
    ));
    run.push_executed(row("drs.object.not_found", VerificationStatus::Pass, ""));
    helix::traceability::bind_run(&mut run).ok();
    finalize_run(&mut run);
    let set = evaluate(&run);
    assert_eq!(
        set.get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    assert!(set
        .get(ClaimKind::Ga4ghRequirement)
        .has_block(ClaimBlockCode::CatalogCheckFailed));
    assert!(set
        .get(ClaimKind::Ga4ghRequirement)
        .has_block(ClaimBlockCode::RequiredEvidenceUnavailable));
    assert!(run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert_eq!(
        run.standard_selection
            .as_ref()
            .unwrap()
            .detected_version
            .as_deref(),
        Some("1.3.0experimental")
    );
    assert_eq!(
        run.standard_selection
            .as_ref()
            .unwrap()
            .selected_version
            .as_deref(),
        Some("1.4.0")
    );

    let standing =
        std::path::Path::new("/Users/SynapticFour/devel/b71b-clean/starter-kit-b71b.json");
    if standing.is_file() {
        let raw = std::fs::read_to_string(standing).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["standard_selection"]["selected_version"], "1.4.0");
        assert_eq!(
            v["standard_selection"]["detected_version"],
            "1.3.0experimental"
        );
        assert!(v["standard_selection"]["verified_version"].is_null());
        for c in v["claims"].as_array().unwrap() {
            assert_eq!(c["status"], "not_verified");
        }
    }
}

#[tokio::test]
async fn b8_t23_normative_drs_140_check_remains_specsource_backed() {
    let _g = B8_LOCK.lock().await;
    let run = versioned_mock().await.run;
    let openapi = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi")
        .expect("HLX-DRS-006");
    assert_eq!(openapi.code, "HLX-DRS-006");
    assert_eq!(
        openapi.helixtest_name.as_deref(),
        Some(DRS_OPENAPI_SPECSOURCE_CHECK)
    );
    assert_eq!(
        openapi.traceability.as_ref().unwrap().category,
        helix::standards::BindingKind::Normative
    );
    let fixture = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .expect("HLX-DRS-002");
    assert_eq!(
        fixture.traceability.as_ref().unwrap().category,
        helix::standards::BindingKind::Fixture
    );
    assert_ne!(
        fixture.traceability.as_ref().unwrap().category,
        helix::standards::BindingKind::Normative
    );
}

#[test]
fn b8_t24_no_helios_dependency_introduced() {
    let cargo = include_str!("../Cargo.toml");
    for line in cargo.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.starts_with('[') {
            continue;
        }
        assert!(
            !t.contains("helios-audit") && !t.starts_with("helios ") && !t.starts_with("helios="),
            "Cargo.toml must not depend on HELIOS: {t}"
        );
    }
    let integrity = include_str!("../src/claim_integrity.rs");
    for needle in ["ro_crate", "RO-Crate", "helios-audit", "application/pdf"] {
        assert!(
            !integrity.contains(needle),
            "claim_integrity must not import HELIOS concepts ({needle})"
        );
    }
    let p = "aa".repeat(32);
    let d = "bb".repeat(32);
    let c = "cc".repeat(32);
    assert_eq!(binding_id(&DRS_140_CONTRACT, &p, &d, &c).len(), 64);
    assert!(!catalog_id(&DRS_140_CONTRACT).is_empty());
    assert_eq!(declared_checker_id(), helix::checker::executed_checker_id());
}
