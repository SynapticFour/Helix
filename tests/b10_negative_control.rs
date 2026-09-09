// SPDX-License-Identifier: Apache-2.0
//! B10 mutation / negative-control gate for versioned DRS 1.4.0.
//! Mutations are HTTP-target behavior. Helix is the instrument. Not HELIOS.

use common::ga4gh_schemas::validate_drs_object_with;
use helix::claim_integrity::{ga4gh_requirement_is_verified, validate_claim_integrity, ClaimJoin};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::differential::differential_from_runs;
use helix::fixture::DrsVerifyFixture;
use helix::model::{VerificationRun, VerificationStatus};
use helix::negative_control::{
    check, check_status, ga4gh_requirement, verified_version, ChecksumOracle,
    CoverageClassification, MutationBind, MutationEvidence, MutationScope, VerifierIdentity,
};
use helix::profile::ProfileId;
use helix::report::verify_json;
use helix::standards::{
    binding_id, catalog_id, default_registry_path, helix_repo_root, load_pack, load_path,
    DRS_140_CONTRACT, DRS_140_PACK_ID,
};
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifyOutcome, VerifySelection};

mod support;
use support::mock_b10::{
    honest_sha256, pinned_size_string_object, start_golden_with_outside_surface,
    start_harness_ineffective_mime_type, start_m1_missing_access_methods, start_m2_size_is_string,
    start_m3_advertised_checksum_lie, start_m4_byte_content_lie, start_m5_range_returns_200,
    start_m6_unknown_object_200, start_outside_closure, start_p1_key_reorder, start_p2_whitespace,
    start_p3_description,
};
use support::mock_ga4gh_drs::{
    mount_ga4gh_drs_service_info, start_mock_ga4gh_drs, valid_drs_object_json, TEST_OBJECT_ID,
};

static B10_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn versioned(id: &str) -> VerifyOptions {
    versioned_fx(id, DrsVerifyFixture::default_catalog())
}

fn versioned_fx(id: &str, fx: DrsVerifyFixture) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: DeclaredTarget {
            target_id: Some(id.into()),
            kind: TargetKind::Mock,
            implementation_name: Some("b10-mock-drs".into()),
            implementation_version: Some("0.0.0".into()),
            ..DeclaredTarget::default()
        },
        drs_fixture: fx,
        ..Default::default()
    }
}

async fn verify_named(url: &str, id: &str) -> VerifyOutcome {
    verify_with_options(url, versioned(id))
        .await
        .expect("versioned DRS 1.4.0")
}

fn sel(run: &VerificationRun) -> &helix::model::StandardSelection {
    run.standard_selection.as_ref().expect("standard_selection")
}

fn assert_all_drs_pass(run: &VerificationRun) {
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
        "drs.object.schema.openapi",
    ] {
        assert_eq!(
            check_status(run, id),
            Some(VerificationStatus::Pass),
            "{id}"
        );
    }
}

fn assert_verifier_invariant(baseline: &VerificationRun, mutated: &VerificationRun) {
    let a = sel(baseline);
    let b = sel(mutated);
    assert_eq!(a.execution_id, b.execution_id, "execution_id");
    assert_eq!(a.pack_integrity_sha256, b.pack_integrity_sha256, "pack");
    assert_eq!(
        a.schema_document_sha256, b.schema_document_sha256,
        "schema_document"
    );
    assert_eq!(
        a.schema_component_sha256, b.schema_component_sha256,
        "schema_component"
    );
    assert_eq!(a.checker_id, b.checker_id, "checker");
    assert_eq!(a.binding_id, b.binding_id, "binding");
    assert_eq!(a.catalog_id, b.catalog_id, "catalog");
    assert_eq!(
        VerifierIdentity::from_run(baseline),
        VerifierIdentity::from_run(mutated)
    );
}

fn assert_target_lineage(run: &VerificationRun) {
    let t = run.target.identity.as_ref().expect("target identity");
    assert_eq!(t.target_kind, TargetKind::Mock);
    assert_eq!(t.implementation_name.as_deref(), Some("b10-mock-drs"));
    assert_eq!(t.implementation_version.as_deref(), Some("0.0.0"));
}

fn bind_in_contract<'a>(
    id: &'a str,
    desc: &'a str,
    checks: &'a [&'a str],
    oracle: ChecksumOracle,
    baseline: &'a VerificationRun,
    mutated: &'a VerificationRun,
) -> MutationEvidence {
    MutationEvidence::bind(MutationBind {
        mutation_id: id,
        mutation_description: desc,
        mutation_scope: MutationScope::InContract,
        expected_check_ids: checks,
        checksum_oracle: oracle,
        baseline,
        mutated,
    })
}

fn shipped_drs_140() -> helix::standards::LoadedPack {
    let reg = load_path(&default_registry_path()).expect("registry");
    let v = reg
        .versions
        .into_iter()
        .find(|v| v.pack_id == DRS_140_PACK_ID)
        .expect("DRS 1.4.0");
    load_pack(&v, &helix_repo_root()).expect("load pack")
}

/// B10-T1 golden baseline verifies.
#[tokio::test]
async fn b10_t1_golden_baseline_verifies() {
    let _g = B10_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let out = verify_named(&mock.drs_url(), "b10-golden").await;
    assert_all_drs_pass(&out.run);
    assert_eq!(verified_version(&out.run), Some("1.4.0"));
    assert!(ga4gh_requirement_is_verified(&out.run));
    assert_eq!(ga4gh_requirement(&out.run), ClaimStatus::Verified);
    assert_eq!(sel(&out.run).selected_version.as_deref(), Some("1.4.0"));
    assert_eq!(
        sel(&out.run).standards_registry_entry.as_deref(),
        Some(DRS_140_PACK_ID)
    );
    assert!(mock.drs_url().contains("127.0.0.1"));
}

/// B10-T2 M1 access_methods mutation detected.
#[tokio::test]
async fn b10_t2_m1_access_methods_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m1_missing_access_methods().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        check_status(&mutated.run, "drs.object.schema"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.schema").and_then(|r| r.attribution),
        Some(FailureAttribution::TargetFailure)
    );
    assert_eq!(
        check_status(&mutated.run, "drs.object.schema.openapi"),
        Some(VerificationStatus::Pass),
        "pinned OpenAPI does not require access_methods"
    );
    assert_eq!(
        check_status(&mutated.run, "drs.object.checksum"),
        Some(VerificationStatus::Skip)
    );
    assert_eq!(
        check_status(&mutated.run, "drs.object.range"),
        Some(VerificationStatus::Skip)
    );
    let ev = bind_in_contract(
        "M1",
        "omit access_methods",
        &["drs.object.schema"],
        ChecksumOracle::NotApplicable,
        &baseline.run,
        &mutated.run,
    );
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
    assert_eq!(verified_version(&mutated.run), None);
    assert_eq!(ga4gh_requirement(&mutated.run), ClaimStatus::NotVerified);
}

/// B10-T3 M2 schema mutation detected against the pinned SpecSource.
#[tokio::test]
async fn b10_t3_m2_schema_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let pack = shipped_drs_140();
    let valid = valid_drs_object_json("http://127.0.0.1:9");
    assert!(
        validate_drs_object_with(&pack.spec, &valid).is_ok(),
        "golden object must be valid against pinned DRS 1.4.0"
    );
    let invalid = pinned_size_string_object();
    assert!(
        validate_drs_object_with(&pack.spec, &invalid).is_err(),
        "size as string must violate pinned DrsObject.yaml"
    );

    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m2_size_is_string().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        check_status(&mutated.run, "drs.object.schema.openapi"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.schema.openapi").and_then(|r| r.attribution),
        Some(FailureAttribution::SpecFailure)
    );
    assert_eq!(verified_version(&mutated.run), None);
    let ev = bind_in_contract(
        "M2",
        "size JSON string vs pinned integer",
        &["drs.object.schema.openapi"],
        ChecksumOracle::NotApplicable,
        &baseline.run,
        &mutated.run,
    );
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
}

/// B10-T4 M3 advertised checksum mutation detected (advertised_consistency oracle).
#[tokio::test]
async fn b10_t4_m3_checksum_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    assert!(
        baseline
            .run
            .drs_fixture
            .as_ref()
            .unwrap()
            .expected_sha256
            .is_none(),
        "M3 oracle is advertised vs download, not an independent digest"
    );
    let mutant = start_m3_advertised_checksum_lie().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        check_status(&mutated.run, "drs.object.checksum"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.checksum").and_then(|r| r.attribution),
        Some(FailureAttribution::TargetFailure)
    );
    assert_eq!(verified_version(&mutated.run), None);
    let ev = bind_in_contract(
        "M3",
        "advertised sha256 lie; bytes unchanged",
        &["drs.object.checksum"],
        ChecksumOracle::AdvertisedConsistency,
        &baseline.run,
        &mutated.run,
    );
    assert_eq!(ev.checksum_oracle, ChecksumOracle::AdvertisedConsistency);
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
}

/// B10-T5 M4 byte-content mutation detected against independently supplied digest.
#[tokio::test]
async fn b10_t5_m4_byte_content_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let fx = DrsVerifyFixture::operator_declared(TEST_OBJECT_ID.into(), Some(honest_sha256()))
        .expect("fixture");
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_with_options(&golden.drs_url(), versioned_fx("b10-golden", fx.clone()))
        .await
        .expect("golden with operator digest");
    assert_all_drs_pass(&baseline.run);
    let mutant = start_m4_byte_content_lie().await;
    let mutated = verify_with_options(&mutant.drs_url(), versioned_fx("b10-golden", fx))
        .await
        .expect("mutated");
    assert_eq!(
        check_status(&mutated.run, "drs.object.checksum"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.checksum").and_then(|r| r.attribution),
        Some(FailureAttribution::TargetFailure)
    );
    assert_eq!(verified_version(&mutated.run), None);
    let ev = bind_in_contract(
        "M4",
        "bytes mutated; advertised checksum of original 4096xA kept",
        &["drs.object.checksum"],
        ChecksumOracle::OperatorDigest,
        &baseline.run,
        &mutated.run,
    );
    assert_eq!(ev.checksum_oracle, ChecksumOracle::OperatorDigest);
}

/// B10-T6 M5 Range mutation detected. Proves a Range header was sent.
#[tokio::test]
async fn b10_t6_m5_range_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m5_range_returns_200().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert!(
        mutant.range_was_sent(),
        "HLX-DRS-004 must send Range, not a full GET that accidentally passes"
    );
    assert_eq!(
        check_status(&mutated.run, "drs.object.range"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.range").and_then(|r| r.attribution),
        Some(FailureAttribution::TargetFailure)
    );
    assert_eq!(verified_version(&mutated.run), None);
    let _ = baseline;
}

/// B10-T7 M6 unknown-object mutation detected (derived id, no scan).
#[tokio::test]
async fn b10_t7_m6_unknown_object_mutation_detected() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m6_unknown_object_200().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        check_status(&mutated.run, "drs.object.not_found"),
        Some(VerificationStatus::Fail)
    );
    assert_eq!(
        check(&mutated.run, "drs.object.not_found").and_then(|r| r.attribution),
        Some(FailureAttribution::TargetFailure)
    );
    assert_eq!(verified_version(&mutated.run), None);
    let _ = baseline;
}

/// B10-T8 every relevant mutation revokes verified_version.
#[tokio::test]
async fn b10_t8_every_relevant_mutation_revokes_verified_version() {
    let _g = B10_LOCK.lock().await;
    let mut cases: Vec<(&str, String)> = Vec::new();
    let m1 = start_m1_missing_access_methods().await;
    cases.push(("M1", m1.drs_url()));
    let m2 = start_m2_size_is_string().await;
    cases.push(("M2", m2.drs_url()));
    let m3 = start_m3_advertised_checksum_lie().await;
    cases.push(("M3", m3.drs_url()));
    let m5 = start_m5_range_returns_200().await;
    cases.push(("M5", m5.drs_url()));
    let m6 = start_m6_unknown_object_200().await;
    cases.push(("M6", m6.drs_url()));
    for (id, url) in &cases {
        let out = verify_named(url, "b10-golden").await;
        assert_eq!(verified_version(&out.run), None, "{id}");
        assert_eq!(
            ga4gh_requirement(&out.run),
            ClaimStatus::NotVerified,
            "{id}"
        );
        assert!(!ga4gh_requirement_is_verified(&out.run), "{id}");
    }
    let fx =
        DrsVerifyFixture::operator_declared(TEST_OBJECT_ID.into(), Some(honest_sha256())).unwrap();
    let m4 = start_m4_byte_content_lie().await;
    let out = verify_with_options(&m4.drs_url(), versioned_fx("b10-golden", fx))
        .await
        .unwrap();
    assert_eq!(verified_version(&out.run), None, "M4");
}

/// B10-T9 stale VERIFIED cannot survive a mutated execution.
#[tokio::test]
async fn b10_t9_stale_verified_claim_cannot_survive_mutation() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m1_missing_access_methods().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(verified_version(&baseline.run), Some("1.4.0"));
    assert_eq!(verified_version(&mutated.run), None);
    assert_ne!(
        baseline.run.claim_join.as_ref().map(|j| j.claims.clone()),
        mutated.run.claim_join.as_ref().map(|j| j.claims.clone())
    );

    let mut forged = mutated.run.clone();
    if let Some(sel) = forged.standard_selection.as_mut() {
        sel.verified_version = Some("1.4.0".into());
    }
    for r in forged.executed.iter_mut().chain(forged.skipped.iter_mut()) {
        r.verified_version = Some("1.4.0".into());
    }
    forged.claim_join = Some(ClaimJoin::from_run(&forged));
    assert!(
        validate_claim_integrity(&forged).is_err(),
        "copied VERIFIED must not validate on a failing execution"
    );
}

/// B10-T10 restored target can verify again.
#[tokio::test]
async fn b10_t10_restored_target_can_verify_again() {
    let _g = B10_LOCK.lock().await;
    let mutant = start_m1_missing_access_methods().await;
    let failed = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(verified_version(&failed.run), None);
    drop(mutant);
    let restored = start_mock_ga4gh_drs().await;
    let ok = verify_named(&restored.drs_url(), "b10-golden").await;
    assert_eq!(verified_version(&ok.run), Some("1.4.0"));
    assert_all_drs_pass(&ok.run);
}

/// B10-T11 execution_id remains invariant under target mutation.
#[tokio::test]
async fn b10_t11_execution_id_remains_invariant_under_target_mutation() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m3_advertised_checksum_lie().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        sel(&baseline.run).execution_id,
        sel(&mutated.run).execution_id
    );
    assert!(sel(&baseline.run).execution_id.is_some());
}

/// B10-T12 target_execution_id isolates mutated execution.
#[tokio::test]
async fn b10_t12_target_execution_id_isolates_mutated_execution() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let a = verify_named(&golden.drs_url(), "b10-a").await;
    let b_ok = verify_named(&golden.drs_url(), "b10-b").await;
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b_ok.run).target_execution_id,
        "different target_id must isolate identity"
    );
    let mutant = start_m1_missing_access_methods().await;
    let b_mut = verify_named(&mutant.drs_url(), "b10-b").await;
    assert_ne!(
        sel(&b_ok.run).target_execution_id,
        sel(&b_mut.run).target_execution_id,
        "mutated execution has its own endpoint in target_execution_id"
    );
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b_mut.run).target_execution_id
    );
    assert_eq!(sel(&a.run).execution_id, sel(&b_mut.run).execution_id);
}

/// B10-T13 verifier pack identity remains invariant.
#[tokio::test]
async fn b10_t13_verifier_pack_identity_remains_invariant() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m2_size_is_string().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        sel(&baseline.run).pack_integrity_sha256,
        sel(&mutated.run).pack_integrity_sha256
    );
    assert_eq!(
        sel(&baseline.run).standards_source_commit,
        sel(&mutated.run).standards_source_commit
    );
}

/// B10-T14 verifier schema identity remains invariant.
#[tokio::test]
async fn b10_t14_verifier_schema_identity_remains_invariant() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m2_size_is_string().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(
        sel(&baseline.run).schema_document_sha256,
        sel(&mutated.run).schema_document_sha256
    );
    assert_eq!(
        sel(&baseline.run).schema_component_sha256,
        sel(&mutated.run).schema_component_sha256
    );
}

/// B10-T15 checker identity remains invariant.
#[tokio::test]
async fn b10_t15_checker_identity_remains_invariant() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m5_range_returns_200().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(sel(&baseline.run).checker_id, sel(&mutated.run).checker_id);
}

/// B10-T16 binding identity remains invariant.
#[tokio::test]
async fn b10_t16_binding_identity_remains_invariant() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m6_unknown_object_200().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(sel(&baseline.run).binding_id, sel(&mutated.run).binding_id);
    let expected_binding = binding_id(
        &DRS_140_CONTRACT,
        sel(&baseline.run).pack_integrity_sha256.as_deref().unwrap(),
        sel(&baseline.run)
            .schema_document_sha256
            .as_deref()
            .unwrap(),
        sel(&baseline.run)
            .schema_component_sha256
            .as_deref()
            .unwrap(),
    );
    assert_eq!(
        sel(&baseline.run).binding_id.as_deref(),
        Some(expected_binding.as_str())
    );
    assert_target_lineage(&mutated.run);
}

/// B10-T17 catalog identity remains invariant.
#[tokio::test]
async fn b10_t17_catalog_identity_remains_invariant() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m1_missing_access_methods().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(sel(&baseline.run).catalog_id, sel(&mutated.run).catalog_id);
    let expected_catalog = catalog_id(&DRS_140_CONTRACT);
    assert_eq!(
        sel(&baseline.run).catalog_id.as_deref(),
        Some(expected_catalog.as_str())
    );
}

/// B10-T18 benign JSON ordering does not create failure.
#[tokio::test]
async fn b10_t18_benign_json_ordering_does_not_create_failure() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let p1 = start_p1_key_reorder().await;
    let mutated = verify_named(&p1.drs_url(), "b10-golden").await;
    assert_all_drs_pass(&mutated.run);
    assert_eq!(verified_version(&mutated.run), Some("1.4.0"));
    let ev = MutationEvidence::bind(MutationBind {
        mutation_id: "P1",
        mutation_description: "JSON object key reorder",
        mutation_scope: MutationScope::Benign,
        expected_check_ids: &[],
        checksum_oracle: ChecksumOracle::NotApplicable,
        baseline: &baseline.run,
        mutated: &mutated.run,
    });
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::BenignNoChange
    );
    assert_ne!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
}

/// B10-T19 benign optional metadata does not create failure.
#[tokio::test]
async fn b10_t19_benign_irrelevant_metadata_does_not_create_failure() {
    let _g = B10_LOCK.lock().await;
    let p2 = start_p2_whitespace().await;
    let ws = verify_named(&p2.drs_url(), "b10-golden").await;
    assert_all_drs_pass(&ws.run);
    let p3 = start_p3_description().await;
    let desc = verify_named(&p3.drs_url(), "b10-golden").await;
    assert_all_drs_pass(&desc.run);
    assert_eq!(verified_version(&desc.run), Some("1.4.0"));
    let p4a = start_mock_ga4gh_drs().await;
    let a = verify_named(&p4a.drs_url(), "b10-p4-a").await;
    let b = verify_named(&p4a.drs_url(), "b10-p4-b").await;
    assert_eq!(verified_version(&a.run), Some("1.4.0"));
    assert_eq!(verified_version(&b.run), Some("1.4.0"));
    assert_ne!(
        sel(&a.run).target_execution_id,
        sel(&b.run).target_execution_id
    );
    assert_eq!(sel(&a.run).execution_id, sel(&b.run).execution_id);
}

/// B10-T20 mutation outside closure is explicitly identified.
#[tokio::test]
async fn b10_t20_mutation_outside_closure_is_explicitly_identified() {
    let _g = B10_LOCK.lock().await;
    let golden = start_golden_with_outside_surface().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let outside = start_outside_closure().await;
    let mutated = verify_named(&outside.drs_url(), "b10-golden").await;
    assert_all_drs_pass(&baseline.run);
    assert_all_drs_pass(&mutated.run);
    assert_eq!(verified_version(&mutated.run), Some("1.4.0"));
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
        "drs.object.schema.openapi",
    ] {
        assert_eq!(
            check_status(&baseline.run, id),
            check_status(&mutated.run, id)
        );
    }
    let ev = MutationEvidence::bind(MutationBind {
        mutation_id: "O1",
        mutation_description: "unused GET /internal/not-in-drs-catalog 200→500",
        mutation_scope: MutationScope::OutsideClosure,
        expected_check_ids: &[],
        checksum_oracle: ChecksumOracle::NotApplicable,
        baseline: &baseline.run,
        mutated: &mutated.run,
    });
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::MutationOutsideClosure
    );
    assert_ne!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
}

/// B10-T21 target failure attribution remains target_failure.
#[tokio::test]
async fn b10_t21_target_failure_attribution_remains_target_failure() {
    let _g = B10_LOCK.lock().await;
    let mutant = start_m1_missing_access_methods().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    let row = check(&mutated.run, "drs.object.schema").unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::TargetFailure));
    assert_ne!(
        row.attribution,
        Some(FailureAttribution::HelixExecutionFailure)
    );
}

/// B10-T22 spec violation attribution remains spec_failure.
#[tokio::test]
async fn b10_t22_spec_violation_attribution_remains_spec_failure() {
    let _g = B10_LOCK.lock().await;
    let mutant = start_m2_size_is_string().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    let row = check(&mutated.run, "drs.object.schema.openapi").unwrap();
    assert_eq!(row.attribution, Some(FailureAttribution::SpecFailure));
    assert_ne!(
        row.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );
}

/// B10-T23 fixture-unavailable remains honest SKIP.
#[tokio::test]
async fn b10_t23_fixture_unavailable_remains_honest_skip() {
    let _g = B10_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    mount_ga4gh_drs_service_info(&mock.server).await;
    let fx = DrsVerifyFixture::operator_declared("does-not-exist-on-mock".into(), None).unwrap();
    let out = verify_with_options(&mock.drs_url(), versioned_fx("b10-skip", fx))
        .await
        .expect("verify");
    let schema = check(&out.run, "drs.object.schema.openapi").unwrap();
    assert_eq!(schema.status, VerificationStatus::Skip);
    assert_eq!(
        schema.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );
    let msg = schema.message.as_deref().unwrap_or("");
    assert!(msg.contains(framework::drs::FIXTURE_UNAVAILABLE), "{msg}");
    assert_ne!(schema.status, VerificationStatus::Pass);
    assert_ne!(schema.status, VerificationStatus::Fail);
}

/// B10-T24 target A is unaffected by target B mutation.
#[tokio::test]
async fn b10_t24_target_a_is_unaffected_by_target_b_mutation() {
    let _g = B10_LOCK.lock().await;
    let a_mock = start_mock_ga4gh_drs().await;
    let a = verify_named(&a_mock.drs_url(), "b10-a").await;
    let a_exec = sel(&a.run).execution_id.clone();
    let a_tid = sel(&a.run).target_execution_id.clone();
    let a_claims = evaluate(&a.run);
    let a_json = verify_json(&a.run).unwrap();

    let b_mut = start_m1_missing_access_methods().await;
    let b = verify_named(&b_mut.drs_url(), "b10-b").await;
    assert_eq!(verified_version(&b.run), None);

    assert_eq!(sel(&a.run).execution_id, a_exec);
    assert_eq!(sel(&a.run).target_execution_id, a_tid);
    assert_eq!(evaluate(&a.run), a_claims);
    assert_eq!(verify_json(&a.run).unwrap(), a_json);
    assert_eq!(verified_version(&a.run), Some("1.4.0"));

    let a2 = verify_named(&a_mock.drs_url(), "b10-a").await;
    assert_eq!(verified_version(&a2.run), Some("1.4.0"));
    assert_eq!(sel(&a2.run).execution_id, a_exec);
    assert_eq!(sel(&a2.run).target_execution_id, a_tid);
}

/// B10-T25 differential output contains no ranking.
#[tokio::test]
async fn b10_t25_differential_output_contains_no_ranking() {
    let _g = B10_LOCK.lock().await;
    let a_mock = start_mock_ga4gh_drs().await;
    let a = verify_named(&a_mock.drs_url(), "b10-a").await;
    let b_mut = start_m3_advertised_checksum_lie().await;
    let b = verify_named(&b_mut.drs_url(), "b10-b").await;
    let report = differential_from_runs(&a.run, &b.run);
    assert!(!report.creates_verification);
    assert_eq!(report.ranking_semantics, "absent");
    assert!(!report.contains_ranking_semantics());
    let ev = bind_in_contract(
        "M3",
        "advertised checksum",
        &["drs.object.checksum"],
        ChecksumOracle::AdvertisedConsistency,
        &a.run,
        &b.run,
    );
    assert!(!ev.contains_ranking_semantics());
    let v = serde_json::to_value(&ev).unwrap();
    assert!(v.get("score").is_none());
    assert!(v.get("overall_score").is_none());
}

/// B10-T26 mutation evidence round-trips without claim drift.
#[tokio::test]
async fn b10_t26_mutation_evidence_round_trips_without_claim_drift() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    let mutant = start_m1_missing_access_methods().await;
    let mutated = verify_named(&mutant.drs_url(), "b10-golden").await;
    let ev = bind_in_contract(
        "M1",
        "omit access_methods",
        &["drs.object.schema"],
        ChecksumOracle::NotApplicable,
        &baseline.run,
        &mutated.run,
    );
    let ser = serde_json::to_string(&ev).unwrap();
    let back: MutationEvidence = serde_json::from_str(&ser).unwrap();
    assert_eq!(back.observed_claim_state, ClaimStatus::NotVerified);
    let json = verify_json(&mutated.run).unwrap();
    let parsed: VerificationRun = serde_json::from_str(&json).unwrap();
    assert_eq!(
        evaluate(&parsed).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    let mut lying = back;
    lying.observed_claim_state = ClaimStatus::Verified;
    assert_eq!(
        evaluate(&mutated.run)
            .get(ClaimKind::Ga4ghRequirement)
            .status,
        ClaimStatus::NotVerified,
        "mutation metadata must not override evaluate()"
    );
}

/// B10-T27 no mutation state persists after restoration.
#[tokio::test]
async fn b10_t27_no_mutation_state_persists_after_restoration() {
    let _g = B10_LOCK.lock().await;
    let mutant = start_m6_unknown_object_200().await;
    let failed = verify_named(&mutant.drs_url(), "b10-golden").await;
    assert_eq!(verified_version(&failed.run), None);
    drop(mutant);
    let restored = start_mock_ga4gh_drs().await;
    let ok = verify_named(&restored.drs_url(), "b10-golden").await;
    assert_eq!(verified_version(&ok.run), Some("1.4.0"));
    assert_eq!(
        check_status(&ok.run, "drs.object.not_found"),
        Some(VerificationStatus::Pass)
    );
}

/// B10-T28 no external network dependency; harness self-check is not a false detection.
#[tokio::test]
async fn b10_t28_no_external_network_and_harness_is_not_tautological() {
    let _g = B10_LOCK.lock().await;
    let golden = start_mock_ga4gh_drs().await;
    let baseline = verify_named(&golden.drs_url(), "b10-golden").await;
    assert!(golden.drs_url().starts_with("http://127.0.0.1"));
    let harness = start_harness_ineffective_mime_type().await;
    let mutated = verify_named(&harness.drs_url(), "b10-golden").await;
    assert_eq!(
        check_status(&mutated.run, "drs.object.range"),
        Some(VerificationStatus::Pass),
        "mime_type is unused by HLX-DRS-004"
    );
    assert_eq!(verified_version(&mutated.run), Some("1.4.0"));
    let ev = MutationEvidence::bind(MutationBind {
        mutation_id: "H1",
        mutation_description: "mime_type unused by selected Range check",
        mutation_scope: MutationScope::HarnessIneffective,
        expected_check_ids: &["drs.object.range"],
        checksum_oracle: ChecksumOracle::NotApplicable,
        baseline: &baseline.run,
        mutated: &mutated.run,
    });
    assert_eq!(
        ev.coverage_classification,
        CoverageClassification::NoBehavioralChange
    );
    assert_ne!(
        ev.coverage_classification,
        CoverageClassification::MutationDetected
    );
    let src = include_str!("support/mock_b10.rs");
    assert!(!src.contains("docker pull"));
    assert!(!src.contains("0.0.0.0"));
    assert_verifier_invariant(&baseline.run, &mutated.run);

    let src_neg = include_str!("../src/negative_control.rs");
    assert!(!src_neg.contains("helios"));
    assert!(!src_neg.contains("RO-Crate"));
}
