// SPDX-License-Identifier: Apache-2.0
//! B4: same DRS 1.4.0 Supported Pack against independently identified targets.
//! Mocks are not independent implementations. Not VERIFIED. Not HELIOS. Not Ferrum.

mod support;

use framework::drs::{reset_with_spec_calls, set_lie_spec_document_hash, with_spec_calls};
use helix::claims::{evaluate, ClaimStatus};
use helix::model::VerificationStatus;
use helix::profile::ProfileId;
use helix::target::{
    compare_target_runs, verification_cache_key, DeclaredTarget, FailureAttribution, TargetKind,
};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_invalid_drs_object};

static TARGET_JOIN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn versioned(declared: DeclaredTarget) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        registry: None,
        vendor_root: None,
        declared_target: declared,
    }
}

fn mock_declared(id: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::Mock,
        ..DeclaredTarget::default()
    }
}

fn synthetic_declared(id: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::SyntheticTarget,
        ..DeclaredTarget::default()
    }
}

/// Test 1 — target identity changes execution identity (same pack/checker).
#[tokio::test]
async fn test1_target_identity_changes_execution_identity() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(&url, versioned(mock_declared("target-a")))
        .await
        .expect("A");
    let b = verify_with_options(&url, versioned(mock_declared("target-b")))
        .await
        .expect("B");
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(
        sa.execution_id, sb.execution_id,
        "spec-join id is pack-scoped"
    );
    assert_ne!(
        sa.target_execution_id, sb.target_execution_id,
        "target-scoped execution id must change with target_id"
    );
    assert_eq!(
        a.run.target.identity.as_ref().unwrap().target_id,
        "target-a"
    );
    assert_eq!(
        b.run.target.identity.as_ref().unwrap().target_id,
        "target-b"
    );
}

/// Test 2 — Target A and B use the same Supported Pack (runtime values).
#[tokio::test]
async fn test2_same_supported_pack_across_targets() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let a = verify_with_options(&mock_a.drs_url(), versioned(mock_declared("pack-a")))
        .await
        .expect("A");
    let b = verify_with_options(&mock_b.drs_url(), versioned(mock_declared("pack-b")))
        .await
        .expect("B");
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(
        sa.standards_registry_entry.as_deref(),
        Some("ga4gh.drs.1.4.0")
    );
    assert_eq!(sa.standards_registry_entry, sb.standards_registry_entry);
    assert_eq!(sa.pack_integrity_sha256, sb.pack_integrity_sha256);
    assert_eq!(sa.schema_document_sha256, sb.schema_document_sha256);
    assert_eq!(sa.checker_id, sb.checker_id);
    assert_eq!(sa.binding_id, sb.binding_id);
    assert_eq!(sa.catalog_id, sb.catalog_id);
    assert_eq!(
        sa.pack_integrity_sha256.as_deref(),
        Some("c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89")
    );
    assert_eq!(
        sa.schema_document_sha256.as_deref(),
        Some("3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608")
    );
    let cmp = compare_target_runs(&a.run, &b.run);
    assert!(cmp.same_pack);
    assert!(!cmp.independent_implementation_evidence);
}

/// Test 3 — A cannot reuse B's result; B executes independently.
#[tokio::test]
async fn test3_target_b_executes_independently() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let _a = verify_with_options(&mock_a.drs_url(), versioned(mock_declared("indep-a")))
        .await
        .expect("A");
    let after_a = with_spec_calls();
    assert!(after_a >= 1, "A must invoke the versioned checker");
    let _b = verify_with_options(&mock_b.drs_url(), versioned(mock_declared("indep-b")))
        .await
        .expect("B");
    let after_b = with_spec_calls();
    assert!(
        after_b > after_a,
        "B must invoke the checker again, not reuse A"
    );
}

/// Test 4 — no verification cache: B does not receive A's result.
#[tokio::test]
async fn test4_cache_isolation() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let a = verify_with_options(&url, versioned(mock_declared("cache-a")))
        .await
        .expect("A");
    let b = verify_with_options(&url, versioned(mock_declared("cache-b")))
        .await
        .expect("B");
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    let ka = verification_cache_key(a.run.target.identity.as_ref().unwrap(), sa);
    let kb = verification_cache_key(b.run.target.identity.as_ref().unwrap(), sb);
    assert_ne!(ka, kb);
    assert!(with_spec_calls() >= 2);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
}

/// Test 5 — declared target metadata cannot create VERIFIED.
#[tokio::test]
async fn test5_target_metadata_cannot_create_verified() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let declared = DeclaredTarget {
        target_id: Some("meta-target".into()),
        kind: TargetKind::ReferenceImplementation,
        implementation_name: Some("NotAProof".into()),
        implementation_version: Some("1.2.3".into()),
        standard_version: Some("1.4.0".into()),
    };
    let outcome = verify_with_options(&mock.drs_url(), versioned(declared))
        .await
        .expect("verify");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    let id = outcome.run.target.identity.as_ref().unwrap();
    assert_eq!(id.implementation_version.as_deref(), Some("1.2.3"));
    assert_eq!(id.declared.standard_version.as_deref(), Some("1.4.0"));
    let claims = evaluate(&outcome.run);
    assert!(!claims.any_verified());
    assert!(claims
        .items
        .iter()
        .all(|c| c.status != ClaimStatus::Verified));
}

/// Test 6 — mock is not independent implementation.
#[tokio::test]
async fn test6_mock_is_not_independent_implementation() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(&mock.drs_url(), versioned(mock_declared("fixture-mock")))
        .await
        .expect("verify");
    let kind = outcome.run.target.identity.as_ref().unwrap().target_kind;
    assert_eq!(kind, TargetKind::Mock);
    assert!(!kind.qualifies_as_independent_implementation());
    let cmp = compare_target_runs(&outcome.run, &outcome.run);
    assert!(!cmp.independent_implementation_evidence);
}

/// Test 7 — Ferrum is not required (behavioral: mock DRS 1.4.0 path).
#[tokio::test]
async fn test7_ferrum_is_not_required() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(&mock.drs_url(), versioned(mock_declared("no-ferrum")))
        .await
        .expect("verify without Ferrum");
    assert_eq!(
        outcome
            .run
            .standard_selection
            .as_ref()
            .unwrap()
            .selection_status,
        helix::standards::SELECTED
    );
    assert!(outcome
        .run
        .executed
        .iter()
        .any(|r| r.id == "drs.object.schema.openapi" && r.status == VerificationStatus::Pass));
}

/// Test 8 — same HelixTest checker invoked for A and B.
#[tokio::test]
async fn test8_same_checker_actually_invoked() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    let mock_a = start_mock_ga4gh_drs().await;
    let mock_b = start_mock_ga4gh_drs().await;
    let a = verify_with_options(&mock_a.drs_url(), versioned(mock_declared("chk-a")))
        .await
        .expect("A");
    let b = verify_with_options(&mock_b.drs_url(), versioned(mock_declared("chk-b")))
        .await
        .expect("B");
    let ca = a
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .checker_id
        .clone();
    let cb = b
        .run
        .standard_selection
        .as_ref()
        .unwrap()
        .checker_id
        .clone();
    assert_eq!(ca, cb);
    assert_eq!(
        ca.as_deref(),
        Some("v0.1.3:1832c043e1679ec283cb2113510ee33684317cce")
    );
    assert!(with_spec_calls() >= 2);
}

/// Test 9 — target failure is target-scoped (synthetic negative, classified honestly).
#[tokio::test]
async fn test9_target_failure_is_target_scoped() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let honest = start_mock_ga4gh_drs().await;
    let broken = start_mock_invalid_drs_object().await;
    let a = verify_with_options(&honest.drs_url(), versioned(mock_declared("honest")))
        .await
        .expect("A");
    let b = verify_with_options(&broken.uri(), versioned(synthetic_declared("broken")))
        .await
        .expect("B");
    let openapi_a = a
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi");
    let openapi_b = b
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi");
    if let Some(row) = openapi_a {
        assert_ne!(row.status, VerificationStatus::Error);
        assert_eq!(row.target_id.as_deref(), Some("honest"));
    }
    let fail_b = b
        .run
        .executed
        .iter()
        .find(|r| r.status == VerificationStatus::Fail)
        .expect("broken target must fail a check");
    assert_eq!(fail_b.target_id.as_deref(), Some("broken"));
    assert_ne!(
        fail_b.attribution,
        Some(FailureAttribution::HelixExecutionFailure)
    );
    assert!(
        fail_b.attribution == Some(FailureAttribution::SpecFailure)
            || fail_b.attribution == Some(FailureAttribution::TargetFailure)
            || fail_b.attribution == Some(FailureAttribution::TransportFailure)
            || openapi_b.map(|r| r.status) == Some(VerificationStatus::Fail)
    );
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

/// Test 10 — Helix/checker failure is not target non-conformance.
#[tokio::test]
async fn test10_helix_failure_is_not_target_failure() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    set_lie_spec_document_hash(true);
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(&mock.drs_url(), versioned(mock_declared("lie-spec")))
        .await
        .expect("join must discard mismatched SpecSource");
    set_lie_spec_document_hash(false);
    let errors: Vec<_> = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.status == VerificationStatus::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "identity mismatch must be Error, not Fail"
    );
    for e in errors {
        assert_eq!(
            e.attribution,
            Some(FailureAttribution::HelixExecutionFailure)
        );
        let msg = e.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("SpecSource identity mismatch") || msg.contains("adapter error"),
            "{msg}"
        );
    }
}

/// Test 11 — target seam is HTTP identity + base_url (no Ferrum types).
#[test]
fn test11_target_adapter_has_no_ferrum_surface() {
    let src = include_str!("../src/target.rs");
    assert!(!src.contains("ferrum::"));
    assert!(!src.contains("use ferrum"));
    let adapter = include_str!("../src/adapter/mod.rs");
    assert!(!adapter.contains("ferrum::"));
    assert!(adapter.contains("base_url"));
}

/// Test 12 — B3 support semantics still hold on this path (DRS 1.5.0 unsupported).
#[tokio::test]
async fn test12_drs_150_remains_unsupported() {
    let _guard = TARGET_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        VerifyOptions {
            profile: ProfileId::Generic,
            selection: VerifySelection::Explicit {
                standard: "drs".into(),
                version: "1.5.0".into(),
                release_class: None,
            },
            registry: None,
            vendor_root: None,
            declared_target: mock_declared("one-five"),
        },
    )
    .await
    .expect("fail closed");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(
        sel.selection_status,
        helix::standards::AVAILABLE_BUT_NOT_SUPPORTED
    );
    assert!(sel.verified_version.is_none());
}
