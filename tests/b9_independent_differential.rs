// SPDX-License-Identifier: Apache-2.0
//! B9 independent-implementation differential verification.
//! Not ranking. Not certification. Not HELIOS.

use helix::claim_integrity::{finalize_run, ga4gh_requirement_is_verified};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::differential::{
    classify_difference, differential_from_runs, differential_json, DifferenceClass,
    DIFFERENTIAL_SCHEMA_VERSION,
};
use helix::fixture::DrsVerifyFixture;
use helix::independence::{reviewed_record, run_counts_as_independent};
use helix::model::{VerificationRun, VerificationStatus};
use helix::profile::ProfileId;
use helix::report::verify_json;
use helix::standards::{binding_id, catalog_id, DRS_140_CONTRACT};
use helix::target::{compare_target_runs, DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use jsonschema::JSONSchema;
use serde_json::Value;

mod support;
use support::mock_ga4gh_drs::{
    mount_ga4gh_drs_service_info, start_mock_ga4gh_drs, start_mock_ga4gh_drs_advertising,
    start_mock_invalid_drs_object,
};

static B9_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

fn declared(
    id: &str,
    kind: TargetKind,
    name: Option<&str>,
    impl_ver: Option<&str>,
) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind,
        implementation_name: name.map(str::to_string),
        implementation_version: impl_ver.map(str::to_string),
        ..DeclaredTarget::default()
    }
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

fn ranking_needles() -> &'static [&'static str] {
    &[
        "winner",
        "leaderboard",
        "ranks second",
        "ranked first",
        "best implementation",
        "worst implementation",
        "more compliant",
        "less compliant",
        "compliance percentage",
        "overall score",
        "market comparison",
    ]
}

fn assert_no_ranking(text: &str) {
    let lower = text.to_lowercase();
    for n in ranking_needles() {
        assert!(!lower.contains(n), "ranking phrase `{n}` in {text}");
    }
}

fn differential_schema() -> JSONSchema {
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/helix-differential-v1.json")).unwrap();
    let leaked = Box::leak(Box::new(schema));
    JSONSchema::compile(leaked).expect("helix-differential-v1 compiles")
}

#[tokio::test]
async fn b9_t1_independent_classification_cannot_be_forged_by_cli_metadata() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        versioned(declared(
            "forged-independent",
            TargetKind::RealIndependentLocalImplementation,
            None,
            None,
        )),
    )
    .await
    .expect("verify");
    assert_eq!(
        outcome.run.target.identity.as_ref().unwrap().target_kind,
        TargetKind::RealIndependentLocalImplementation
    );
    assert!(!run_counts_as_independent(&outcome.run));
}

#[tokio::test]
async fn b9_t2_reference_implementation_cannot_count_as_independent() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let as_ref = verify_with_options(
        &mock.drs_url(),
        versioned(declared(
            "ferrum",
            TargetKind::ReferenceImplementation,
            Some("Ferrum"),
            None,
        )),
    )
    .await
    .unwrap();
    assert!(!run_counts_as_independent(&as_ref.run));
    let relabeled = verify_with_options(
        &mock.drs_url(),
        versioned(declared(
            "ferrum",
            TargetKind::RealIndependentLocalImplementation,
            Some("Ferrum"),
            None,
        )),
    )
    .await
    .unwrap();
    assert!(!run_counts_as_independent(&relabeled.run));
    assert_eq!(
        reviewed_record("ferrum").unwrap().classification,
        TargetKind::ReferenceImplementation
    );
}

#[tokio::test]
async fn b9_t3_mock_cannot_count_as_independent() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let as_mock = verify_with_options(
        &mock.drs_url(),
        versioned(declared(
            "helix-in-process-mock",
            TargetKind::Mock,
            Some("helix-wiremock"),
            None,
        )),
    )
    .await
    .unwrap();
    assert!(!run_counts_as_independent(&as_mock.run));
    let relabeled = verify_with_options(
        &mock.drs_url(),
        versioned(declared(
            "helix-in-process-mock",
            TargetKind::RealIndependentLocalImplementation,
            Some("helix-wiremock"),
            None,
        )),
    )
    .await
    .unwrap();
    assert!(!run_counts_as_independent(&relabeled.run));
}

#[tokio::test]
async fn b9_t4_same_verifier_spec_same_execution_id() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(
        &url,
        versioned(declared("target-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &url,
        versioned(declared("target-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_eq!(sa.checker_id, sb.checker_id);
    assert_eq!(sa.pack_integrity_sha256, sb.pack_integrity_sha256);
    assert_eq!(sa.catalog_id, sb.catalog_id);
    assert_eq!(sa.binding_id, sb.binding_id);
}

#[tokio::test]
async fn b9_t5_different_target_different_target_execution_id() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(
        &url,
        versioned(declared("target-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &url,
        versioned(declared("target-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    assert_ne!(
        a.run
            .standard_selection
            .as_ref()
            .unwrap()
            .target_execution_id,
        b.run
            .standard_selection
            .as_ref()
            .unwrap()
            .target_execution_id
    );
}

#[tokio::test]
async fn b9_t6_different_fixture_different_target_execution_id() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    mount_ga4gh_drs_service_info(&mock.server).await;
    let url = mock.drs_url();
    let mut opts = versioned(declared("same-target", TargetKind::Mock, None, None));
    opts.drs_fixture = DrsVerifyFixture::default_catalog();
    let a = verify_with_options(&url, opts.clone()).await.unwrap();
    opts.drs_fixture = DrsVerifyFixture::operator_declared("other-object".into(), None).unwrap();
    let b = verify_with_options(&url, opts).await.unwrap();
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
}

#[tokio::test]
async fn b9_t7_target_a_survives_target_b() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(
        &url,
        versioned(declared("survive-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let snap = verify_json(&a.run).unwrap();
    let _b = verify_with_options(
        &url,
        versioned(declared("survive-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    assert_eq!(verify_json(&a.run).unwrap(), snap);
}

#[tokio::test]
async fn b9_t8_target_b_survives_target_a() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let b = verify_with_options(
        &url,
        versioned(declared("survive-b2", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let snap = verify_json(&b.run).unwrap();
    let _a = verify_with_options(
        &url,
        versioned(declared("survive-a2", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    assert_eq!(verify_json(&b.run).unwrap(), snap);
}

#[tokio::test]
async fn b9_t9_implementation_version_metadata_cannot_create_verified() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_invalid_drs_object().await;
    let mut opts = versioned(declared(
        "lie-version",
        TargetKind::Mock,
        None,
        Some("1.4.0"),
    ));
    opts.declared_target.implementation_version = Some("1.4.0".into());
    let outcome = verify_with_options(&mock.uri(), opts).await.unwrap();
    assert_eq!(
        outcome
            .run
            .target
            .identity
            .as_ref()
            .unwrap()
            .implementation_version
            .as_deref(),
        Some("1.4.0")
    );
    assert!(outcome
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert!(!ga4gh_requirement_is_verified(&outcome.run));
}

#[tokio::test]
async fn b9_t10_declared_version_remains_separate_from_selected() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let mut d = declared("decl-sep", TargetKind::Mock, None, Some("9.9.9"));
    d.standard_version = Some("1.3.0".into());
    let outcome = verify_with_options(&mock.drs_url(), versioned(d))
        .await
        .unwrap();
    let id = outcome.run.target.identity.as_ref().unwrap();
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(id.declared.standard_version.as_deref(), Some("1.3.0"));
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert_ne!(
        id.declared.standard_version.as_deref(),
        sel.selected_version.as_deref()
    );
}

#[tokio::test]
async fn b9_t11_detected_version_remains_separate_from_verified() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs_advertising("1.3.0experimental").await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        versioned(declared("det-sep", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    let detected = outcome
        .run
        .target
        .identity
        .as_ref()
        .unwrap()
        .detected
        .standard_version
        .clone();
    assert_eq!(detected.as_deref(), Some("1.3.0experimental"));
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    if ga4gh_requirement_is_verified(&outcome.run) {
        assert_eq!(sel.verified_version.as_deref(), Some("1.4.0"));
        assert_ne!(detected.as_deref(), sel.verified_version.as_deref());
    } else {
        assert!(sel.verified_version.is_none());
    }
}

#[tokio::test]
async fn b9_t12_mandatory_skip_prevents_verified() {
    let _g = B9_LOCK.lock().await;
    let mut run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("skip-ver", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    set_check_status(
        &mut run,
        "drs.object.checksum",
        VerificationStatus::Skip,
        "fixture_unavailable: object not present",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[tokio::test]
async fn b9_t13_target_fail_prevents_verified() {
    let _g = B9_LOCK.lock().await;
    let mut run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("fail-ver", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    assert!(!ga4gh_requirement_is_verified(&run));
}

#[tokio::test]
async fn b9_t14_normative_specsource_pass_remains_normative() {
    let _g = B9_LOCK.lock().await;
    let run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("norm", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    let openapi = run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi")
        .expect("HLX-DRS-006");
    assert_eq!(openapi.status, VerificationStatus::Pass);
    assert_eq!(
        openapi.traceability.as_ref().unwrap().category,
        helix::standards::BindingKind::Normative
    );
}

#[tokio::test]
async fn b9_t15_fixture_checks_remain_non_normative() {
    let _g = B9_LOCK.lock().await;
    let run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("fix", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    for id in [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
    ] {
        let row = run
            .executed
            .iter()
            .chain(run.skipped.iter())
            .find(|r| r.id == id)
            .unwrap();
        assert_eq!(
            row.traceability.as_ref().unwrap().category,
            helix::standards::BindingKind::Fixture
        );
    }
}

#[tokio::test]
async fn b9_t16_differential_contains_check_level_attribution() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(
        &url,
        versioned(declared(
            "ga4gh-starter-kit-drs-0.3.2",
            TargetKind::RealIndependentLocalImplementation,
            Some("ga4gh-starter-kit-drs"),
            Some("0.3.2"),
        )),
    )
    .await
    .unwrap();
    let mut b = a.run.clone();
    if let Some(id) = b.target.identity.as_mut() {
        id.target_id = "bento-drs-0.21.5".into();
        id.implementation_name = Some("bento_drs".into());
        id.declared.target_id = Some("bento-drs-0.21.5".into());
        id.declared.implementation_name = Some("bento_drs".into());
        id.declared.kind = TargetKind::RealIndependentLocalImplementation;
        id.target_kind = TargetKind::RealIndependentLocalImplementation;
        id.verified.target_id = "bento-drs-0.21.5".into();
    }
    set_check_status(
        &mut b,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut b);
    let report = differential_from_runs(&a.run, &b);
    let row = report
        .differences
        .iter()
        .find(|d| d.check_id == "drs.object.schema")
        .expect("schema row");
    assert_eq!(row.class, DifferenceClass::TargetBehaviorDifference);
    assert!(row.a_status.is_some());
    assert!(row.b_status.is_some());
}

#[tokio::test]
async fn b9_t17_differential_output_contains_no_ranking_semantics() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let a = verify_with_options(
        &mock.drs_url(),
        versioned(declared("diff-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &mock.drs_url(),
        versioned(declared("diff-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let report = differential_from_runs(&a.run, &b.run);
    let json = differential_json(&report).unwrap();
    let text = helix::differential::format_differential_text(&report);
    assert_no_ranking(&json);
    assert_no_ranking(&text);
    assert_eq!(report.ranking_semantics, "absent");
    assert!(!report.creates_verification);
}

#[tokio::test]
async fn b9_t18_target_failure_does_not_alter_verifier_identity() {
    let _g = B9_LOCK.lock().await;
    let mut run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("vf", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    let checker = run.standard_selection.as_ref().unwrap().checker_id.clone();
    set_check_status(
        &mut run,
        "drs.object.schema",
        VerificationStatus::Fail,
        "access_methods missing",
    );
    restamp(&mut run);
    assert_eq!(run.standard_selection.as_ref().unwrap().checker_id, checker);
}

#[tokio::test]
async fn b9_t19_target_failure_does_not_alter_pack_identity() {
    let _g = B9_LOCK.lock().await;
    let mut run = verify_with_options(
        &start_mock_ga4gh_drs().await.drs_url(),
        versioned(declared("pf", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap()
    .run;
    let pack = run
        .standard_selection
        .as_ref()
        .unwrap()
        .pack_integrity_sha256
        .clone();
    let commit = run
        .standard_selection
        .as_ref()
        .unwrap()
        .standards_source_commit
        .clone();
    set_check_status(
        &mut run,
        "drs.object.not_found",
        VerificationStatus::Fail,
        "got 200",
    );
    restamp(&mut run);
    let sel = run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.pack_integrity_sha256, pack);
    assert_eq!(sel.standards_source_commit, commit);
}

#[tokio::test]
async fn b9_t20_fixture_mutation_changes_only_target_execution_identity() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    mount_ga4gh_drs_service_info(&mock.server).await;
    let url = mock.drs_url();
    let mut opts = versioned(declared("fix-mut", TargetKind::Mock, None, None));
    let a = verify_with_options(&url, opts.clone()).await.unwrap();
    opts.drs_fixture = DrsVerifyFixture::operator_declared("mutated-object".into(), None).unwrap();
    let b = verify_with_options(&url, opts).await.unwrap();
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_eq!(sa.checker_id, sb.checker_id);
    assert_eq!(sa.pack_integrity_sha256, sb.pack_integrity_sha256);
    assert_eq!(sa.catalog_id, sb.catalog_id);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
}

#[tokio::test]
async fn b9_t21_serialized_differential_round_trips() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let a = verify_with_options(
        &mock.drs_url(),
        versioned(declared("rt-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let b = verify_with_options(
        &mock.drs_url(),
        versioned(declared("rt-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let report = differential_from_runs(&a.run, &b.run);
    let json = differential_json(&report).unwrap();
    let parsed: helix::differential::DifferentialReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, report);
    let schema = differential_schema();
    let value: Value = serde_json::from_str(&json).unwrap();
    if let Err(errs) = schema.validate(&value) {
        let msgs: Vec<_> = errs.map(|e| e.to_string()).collect();
        panic!("schema invalid: {msgs:?}");
    }
    assert_eq!(parsed.schema_version, DIFFERENTIAL_SCHEMA_VERSION);
}

#[tokio::test]
async fn b9_t22_no_global_latest_result_contamination() {
    let _g = B9_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(
        &url,
        versioned(declared("latest-a", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    let a_exec = a
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .target_execution_id
        .clone();
    let b = verify_with_options(
        &url,
        versioned(declared("latest-b", TargetKind::Mock, None, None)),
    )
    .await
    .unwrap();
    assert_eq!(
        a.run
            .standard_selection
            .as_ref()
            .unwrap()
            .target_execution_id,
        a_exec
    );
    assert_ne!(
        a_exec,
        b.run
            .standard_selection
            .as_ref()
            .unwrap()
            .target_execution_id
    );
    let cmp = compare_target_runs(&a.run, &b.run);
    assert!(!cmp.independent_implementation_evidence);
}

#[test]
fn b9_t23_immutable_artifact_identity_is_recorded() {
    let sk = reviewed_record("ga4gh-starter-kit-drs-0.3.2").unwrap();
    assert_eq!(
        sk.artifact_identity.as_deref(),
        Some("sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1")
    );
    let bento = reviewed_record("bento-drs-0.21.5").unwrap();
    assert_eq!(
        bento.artifact_identity.as_deref(),
        Some("1dc55ebea90185b1fec2c78c8c52909dd0ca889e")
    );
    assert_eq!(bento.artifact_kind, "source_release");
}

#[test]
fn b9_t24_helios_remains_absent() {
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
    for src in [
        include_str!("../src/differential.rs"),
        include_str!("../src/independence.rs"),
    ] {
        for needle in ["helios-audit", "ro_crate", "RO-Crate", "application/pdf"] {
            assert!(
                !src.contains(needle),
                "B9 source must not import HELIOS ({needle})"
            );
        }
    }
    let _ = binding_id(
        &DRS_140_CONTRACT,
        &"aa".repeat(32),
        &"bb".repeat(32),
        &"cc".repeat(32),
    );
    assert!(!catalog_id(&DRS_140_CONTRACT).is_empty());
}

#[test]
fn b9_skip_vs_pass_is_not_called_spec_compliance() {
    let mut skip = helix::model::VerificationResult::skip(
        helix::model::VerificationCheck::from_spec(helix::identity::spec("drs.object.range"))
            .with_profile("generic"),
        "fixture_unavailable: no access_url",
    );
    helix::target::attach_attribution(&mut skip);
    let mut pass = helix::model::VerificationResult::pass(
        helix::model::VerificationCheck::from_spec(helix::identity::spec("drs.object.range"))
            .with_profile("generic"),
    );
    helix::target::attach_attribution(&mut pass);
    assert_eq!(
        classify_difference(Some(&skip), Some(&pass)),
        DifferenceClass::FixtureCapabilityDifference
    );
    assert_eq!(
        skip.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );
}
