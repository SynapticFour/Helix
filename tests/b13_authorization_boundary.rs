// SPDX-License-Identifier: Apache-2.0
//! B13 authorization evidence boundary.
//!
//! Live authorization-enabled DRS evidence is unavailable. These tests lock
//! the fail-closed facts: configuration is not evidence, the coverage row stays
//! unevaluated, identities are unchanged, and a forged authorization PASS fails
//! integrity. Not HELIOS. Not a catalog expansion.

use helix::authorization::{
    authorization_is_unevaluated, configuration_is_not_evidence,
    observations_constitute_authorization_evidence, verify_options_must_not_carry_credentials,
    AuthorizationEvidenceBlock, AuthorizationObservation, CredentialClass, ExpectedOutcome,
    RequestClass, AUTHORIZATION_COVERAGE_ID, AUTHORIZATION_POLICY_V1,
};
use helix::claim_integrity::validate_claim_integrity;
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::coverage::{coverage_id, CoverageClass, CoverageState};
use helix::identity::spec;
use helix::live_evidence::{
    live_independent_observation, PINNED_BINDING_ID, PINNED_CATALOG_ID, PINNED_CHECKER_ID,
    PINNED_COVERAGE_ID, PINNED_EXECUTION_ID, PINNED_HELIXTEST_SHA, PINNED_PACK_INTEGRITY_SHA256,
};
use helix::model::{
    Target, VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
};
use helix::standards::DRS_140_CONTRACT;
use helix::target::{DeclaredTarget, FailureAttribution, TargetIdentity, TargetKind};
use helix::verify::VerifyOptions;
use std::path::PathBuf;

fn live_b12(name: &str) -> Option<VerificationRun> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local/b12");
    let raw = std::fs::read_to_string(dir.join(name)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn live_b13_dir() -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B13_LIVE_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local/b13")
}

fn b13_json_files() -> Vec<PathBuf> {
    let dir = live_b13_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    rd.filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect()
}

fn not_found_check() -> VerificationCheck {
    VerificationCheck::from_spec(spec("drs.object.not_found"))
}

#[test]
fn b13_t1_authorization_disabled_cannot_pass() {
    assert!(configuration_is_not_evidence(Some("false")));
    let Some(bento) = live_b12("bento.json") else {
        return;
    };
    assert!(authorization_is_unevaluated(&bento));
    let mut forged = bento.clone();
    if let Some(cov) = forged.coverage.as_mut() {
        if let Some(row) = cov
            .rows
            .iter_mut()
            .find(|r| r.id == AUTHORIZATION_COVERAGE_ID)
        {
            row.classification = CoverageClass::EvaluatedNonNormative;
            row.executed = true;
            row.result = Some(VerificationStatus::Pass);
        }
    }
    assert!(validate_claim_integrity(&forged).is_err());
}

#[test]
fn b13_t2_no_live_allowed_observation() {
    assert!(
        b13_json_files().is_empty(),
        "B13 live JSON must not exist until a genuine authorization-enabled target is observed"
    );
    assert_eq!(
        observations_constitute_authorization_evidence(&[]),
        Err(AuthorizationEvidenceBlock::Empty)
    );
}

#[test]
fn b13_t3_invalid_authorization_not_fabricated() {
    let invalid_only = [AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "object-1".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::Invalid,
        observed_status: 403,
        expected_outcome: ExpectedOutcome::Denied,
        attribution: FailureAttribution::TargetFailure,
    }];
    assert_eq!(
        observations_constitute_authorization_evidence(&invalid_only),
        Err(AuthorizationEvidenceBlock::MissingUnauthorizedObservation)
    );
}

#[test]
fn b13_t4_insufficient_permission_unsupported_without_target_scope_model() {
    let o = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "object-1".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::Insufficient,
        observed_status: 0,
        expected_outcome: ExpectedOutcome::Unsupported,
        attribution: FailureAttribution::UnsupportedTest,
    };
    assert_eq!(
        observations_constitute_authorization_evidence(&[o]),
        Err(AuthorizationEvidenceBlock::MissingUnauthorizedObservation)
    );
}

#[test]
fn b13_t5_transport_failure_is_not_authorization_denial() {
    let err = VerificationResult::error(
        not_found_check(),
        "target unreachable; drs checks not executed",
    );
    let attr = FailureAttribution::from_result(&err).expect("error attribution");
    assert_eq!(attr, FailureAttribution::TransportFailure);
    let obs = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "object-1".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::None,
        observed_status: 0,
        expected_outcome: ExpectedOutcome::Denied,
        attribution: FailureAttribution::TransportFailure,
    };
    assert_eq!(
        observations_constitute_authorization_evidence(&[obs]),
        Err(AuthorizationEvidenceBlock::TransportNotDenial)
    );
}

#[test]
fn b13_t6_404_is_not_authorization_denial() {
    let fail = VerificationResult::fail(not_found_check(), "expected 404, got 200");
    let attr = FailureAttribution::from_result(&fail);
    assert_ne!(attr, Some(FailureAttribution::TransportFailure));
    let obs = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "missing-object".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::None,
        observed_status: 404,
        expected_outcome: ExpectedOutcome::Denied,
        attribution: FailureAttribution::TargetFailure,
    };
    assert_eq!(
        observations_constitute_authorization_evidence(&[obs]),
        Err(AuthorizationEvidenceBlock::MissingUnauthorizedObservation)
    );
}

#[test]
fn b13_t7_forged_authorization_pass_fails_integrity() {
    let Some(mut run) = live_b12("starter-kit.json") else {
        return;
    };
    assert!(authorization_is_unevaluated(&run));
    if let Some(cov) = run.coverage.as_mut() {
        if let Some(row) = cov
            .rows
            .iter_mut()
            .find(|r| r.id == AUTHORIZATION_COVERAGE_ID)
        {
            row.classification = CoverageClass::Normative;
            row.executed = true;
            row.result = Some(VerificationStatus::Pass);
            row.required_for_ga4gh_requirement = true;
        }
    }
    assert!(validate_claim_integrity(&run).is_err());
}

#[test]
fn b13_t8_forged_expected_outcome_is_not_evidence() {
    let denied = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "object-1".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::None,
        observed_status: 403,
        expected_outcome: ExpectedOutcome::Denied,
        attribution: FailureAttribution::TargetFailure,
    };
    let allowed = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "object-1".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::Valid,
        observed_status: 200,
        expected_outcome: ExpectedOutcome::Allowed,
        attribution: FailureAttribution::Unknown,
    };
    assert_eq!(
        observations_constitute_authorization_evidence(&[denied.clone(), allowed.clone()]),
        Ok(true)
    );
    let mut forged = allowed;
    forged.expected_outcome = ExpectedOutcome::Denied;
    assert_eq!(
        observations_constitute_authorization_evidence(&[denied, forged]),
        Err(AuthorizationEvidenceBlock::MissingAuthorizedObservation)
    );
}

#[test]
fn b13_t9_target_identity_mismatch_does_not_create_authorization_evidence() {
    let Some(mut run) = live_b12("bento.json") else {
        return;
    };
    if let Some(id) = run.target.identity.as_mut() {
        id.target_id = "not-the-reviewed-bento".into();
    }
    assert!(!live_independent_observation(&run));
    assert!(authorization_is_unevaluated(&run));
}

#[test]
fn b13_t10_credential_never_appears_in_observation_json() {
    let o = AuthorizationObservation {
        policy_id: AUTHORIZATION_POLICY_V1.into(),
        resource_id: "f23b1635-4a65-40fa-8b29-75b4734b602a".into(),
        request_class: RequestClass::GetObject,
        credential_class: CredentialClass::Valid,
        observed_status: 200,
        expected_outcome: ExpectedOutcome::Allowed,
        attribution: FailureAttribution::Unknown,
    };
    let json = serde_json::to_string(&o).unwrap();
    assert!(!json.to_ascii_lowercase().contains("bearer"));
    assert!(!json.contains("eyJ"));
    assert!(!json.to_ascii_lowercase().contains("cookie"));
}

#[test]
fn b13_t11_credential_never_appears_in_deterministic_ids() {
    let cov = coverage_id(
        &DRS_140_CONTRACT,
        &helix::checker::executed_checker_id(),
        Some(PINNED_BINDING_ID),
        Some(PINNED_PACK_INTEGRITY_SHA256),
    );
    assert_eq!(cov, PINNED_COVERAGE_ID);
    assert!(!cov.contains("bearer"));
    assert_eq!(PINNED_EXECUTION_ID.len(), 64);
    assert_eq!(AUTHORIZATION_POLICY_V1, "authorization-policy-v1");
    assert!(!PINNED_COVERAGE_ID.contains(AUTHORIZATION_POLICY_V1));
}

#[test]
fn b13_t12_authorization_cannot_be_inferred_from_authz_enabled() {
    assert!(configuration_is_not_evidence(Some("true")));
    let Some(bento) = live_b12("bento.json") else {
        return;
    };
    let notes = bento
        .target
        .identity
        .as_ref()
        .map(|i| i.target_id.as_str())
        .unwrap_or("");
    assert!(!notes.to_ascii_lowercase().contains("authz_enabled"));
    assert!(authorization_is_unevaluated(&bento));
}

#[test]
fn b13_t13_authorization_cannot_strengthen_unrelated_claims() {
    let Some(bento) = live_b12("bento.json") else {
        return;
    };
    let Some(sk) = live_b12("starter-kit.json") else {
        return;
    };
    assert_eq!(
        bento
            .standard_selection
            .as_ref()
            .unwrap()
            .verified_version
            .as_deref(),
        Some("1.4.0")
    );
    assert!(sk
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert_eq!(
        evaluate(&bento).get(ClaimKind::Security).status,
        ClaimStatus::NotVerified
    );
    assert_eq!(
        evaluate(&sk).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
    assert_eq!(
        evaluate(&bento).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::Verified
    );
}

#[test]
fn b13_t14_unevaluated_rows_remain_unevaluated() {
    let Some(bento) = live_b12("bento.json") else {
        return;
    };
    let cov = bento.coverage.as_ref().unwrap();
    assert_eq!(cov.state, CoverageState::Partial);
    for id in [
        "drs.op.service_info",
        "drs.op.objects_bulk",
        "drs.op.access",
        "drs.security.authorization",
    ] {
        let row = cov.row(id).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(row.classification, CoverageClass::Unevaluated, "{id}");
        assert!(!row.executed, "{id}");
    }
}

#[test]
fn b13_t15_disabling_authorization_revokes_authorization_evidence() {
    assert!(configuration_is_not_evidence(Some("false")));
    let Some(bento) = live_b12("bento.json") else {
        return;
    };
    assert!(authorization_is_unevaluated(&bento));
    assert!(observations_constitute_authorization_evidence(&[]).is_err());
}

#[test]
fn b13_t16_mock_cannot_become_live_authorization_evidence() {
    let mut run = VerificationRun::new(Target::from_identity(TargetIdentity::from_declared(
        "http://127.0.0.1:9",
        &DeclaredTarget {
            target_id: Some("bento-drs-0.21.5".into()),
            kind: TargetKind::Mock,
            implementation_name: Some("bento_drs".into()),
            implementation_version: Some("0.21.5".into()),
            ..DeclaredTarget::default()
        },
    )));
    run.drs_fixture = Some(
        helix::fixture::DrsVerifyFixture::operator_declared(
            "f23b1635-4a65-40fa-8b29-75b4734b602a".into(),
            None,
        )
        .expect("object id"),
    );
    assert!(!live_independent_observation(&run));
    assert!(authorization_is_unevaluated(&run));
}

#[test]
fn b13_verifier_identities_unchanged() {
    assert_eq!(PINNED_HELIXTEST_SHA, helix::model::HELIXTEST_SHA);
    assert_eq!(helix::checker::executed_checker_id(), PINNED_CHECKER_ID);
    assert_eq!(PINNED_BINDING_ID.len(), 64);
    assert_eq!(PINNED_CATALOG_ID.len(), 64);
    assert_eq!(
        PINNED_COVERAGE_ID,
        coverage_id(
            &DRS_140_CONTRACT,
            PINNED_CHECKER_ID,
            Some(PINNED_BINDING_ID),
            Some(PINNED_PACK_INTEGRITY_SHA256),
        )
    );
}

#[test]
fn b13_verify_options_carry_no_credentials() {
    let opts = VerifyOptions::default();
    let debug = format!("{opts:?}");
    assert!(verify_options_must_not_carry_credentials(&debug));
}
