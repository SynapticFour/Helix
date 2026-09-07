// SPDX-License-Identifier: Apache-2.0
//! DRS authorization evidence boundary. Not HELIOS. Not a catalog check.
//!
//! DRS 1.4.0 Auth.md leaves policy to the implementer (Basic / Bearer / Passport
//! / other). Helix does not treat `AUTHZ_ENABLED`, Docker labels, or an
//! Authorization header as proof. `helix verify` does not send credentials.
//! `helix security` dummy HMAC is a different surface.
//!
//! Public scenario id only — never a secret, never hashed into `coverage_id`.

use serde::{Deserialize, Serialize};

use crate::coverage::{CoverageClass, CoverageReport};
use crate::model::VerificationRun;
use crate::target::FailureAttribution;

/// Public, non-secret scenario identity. Credential rotation must not change it.
pub const AUTHORIZATION_POLICY_V1: &str = "authorization-policy-v1";

/// Coverage row that remains unevaluated until a genuine live matrix exists.
pub const AUTHORIZATION_COVERAGE_ID: &str = "drs.security.authorization";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialClass {
    None,
    Valid,
    Invalid,
    Insufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    Denied,
    Allowed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestClass {
    GetObject,
}

/// One black-box observation. No header values. No tokens. No cookies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationObservation {
    pub policy_id: String,
    pub resource_id: String,
    pub request_class: RequestClass,
    pub credential_class: CredentialClass,
    pub observed_status: u16,
    pub expected_outcome: ExpectedOutcome,
    pub attribution: FailureAttribution,
}

/// Why a set of observations is not authorization evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationEvidenceBlock {
    Empty,
    ConfigurationLabelOnly,
    TransportNotDenial,
    MissingUnauthorizedObservation,
    MissingAuthorizedObservation,
    SecretMaterialPresent,
    PolicyIdNotPublicScenario,
}

impl AuthorizationObservation {
    pub fn contains_secret_material(&self) -> bool {
        field_looks_like_secret(&self.policy_id) || field_looks_like_secret(&self.resource_id)
    }
}

fn field_looks_like_secret(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    l.contains("bearer ")
        || l.contains("authorization:")
        || l.contains("cookie=")
        || s.starts_with("eyJ")
        || s.contains("://") && s.contains('@')
}

/// Environment / Docker / README labels never constitute authorization evidence.
pub fn configuration_is_not_evidence(authz_enabled_label: Option<&str>) -> bool {
    let _ = authz_enabled_label;
    true
}

/// True only when denied-without-credentials and allowed-with-valid-credentials
/// were both observed as HTTP results, not transport failures, and no secrets.
pub fn observations_constitute_authorization_evidence(
    observations: &[AuthorizationObservation],
) -> Result<bool, AuthorizationEvidenceBlock> {
    if observations.is_empty() {
        return Err(AuthorizationEvidenceBlock::Empty);
    }
    for o in observations {
        if o.contains_secret_material() {
            return Err(AuthorizationEvidenceBlock::SecretMaterialPresent);
        }
        if o.policy_id != AUTHORIZATION_POLICY_V1 {
            return Err(AuthorizationEvidenceBlock::PolicyIdNotPublicScenario);
        }
        if o.attribution == FailureAttribution::TransportFailure
            && o.expected_outcome == ExpectedOutcome::Denied
        {
            return Err(AuthorizationEvidenceBlock::TransportNotDenial);
        }
    }
    let denied_none = observations.iter().any(|o| {
        o.credential_class == CredentialClass::None
            && o.expected_outcome == ExpectedOutcome::Denied
            && o.attribution != FailureAttribution::TransportFailure
            && o.observed_status != 0
            && o.observed_status != 404
    });
    let allowed_valid = observations.iter().any(|o| {
        o.credential_class == CredentialClass::Valid
            && o.expected_outcome == ExpectedOutcome::Allowed
            && o.observed_status >= 200
            && o.observed_status < 300
    });
    if !denied_none {
        return Err(AuthorizationEvidenceBlock::MissingUnauthorizedObservation);
    }
    if !allowed_valid {
        return Err(AuthorizationEvidenceBlock::MissingAuthorizedObservation);
    }
    Ok(true)
}

pub fn authorization_is_unevaluated(run: &VerificationRun) -> bool {
    let cov = CoverageReport::from_run(run);
    match cov.rows.iter().find(|r| r.id == AUTHORIZATION_COVERAGE_ID) {
        Some(row) => {
            row.classification == CoverageClass::Unevaluated
                && !row.executed
                && row.result.is_none()
        }
        None => true,
    }
}

/// `helix verify` has no credential field. A debug dump of options must not
/// grow an Authorization header without an explicit, tested design.
pub fn verify_options_must_not_carry_credentials(debug: &str) -> bool {
    let l = debug.to_ascii_lowercase();
    !l.contains("bearer")
        && !l.contains("authorization")
        && !l.contains("credential")
        && !l.contains("cookie")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authz_enabled_label_is_never_evidence() {
        assert!(configuration_is_not_evidence(Some("true")));
        assert!(configuration_is_not_evidence(Some("false")));
        assert!(configuration_is_not_evidence(None));
        assert_eq!(
            observations_constitute_authorization_evidence(&[]),
            Err(AuthorizationEvidenceBlock::Empty)
        );
    }

    #[test]
    fn jwt_shaped_resource_id_is_rejected() {
        let o = AuthorizationObservation {
            policy_id: AUTHORIZATION_POLICY_V1.into(),
            resource_id: "eyJhbGciOiJIUzI1NiJ9.aaa.bbb".into(),
            request_class: RequestClass::GetObject,
            credential_class: CredentialClass::Valid,
            observed_status: 200,
            expected_outcome: ExpectedOutcome::Allowed,
            attribution: FailureAttribution::Unknown,
        };
        assert!(o.contains_secret_material());
        assert_eq!(
            observations_constitute_authorization_evidence(&[o]),
            Err(AuthorizationEvidenceBlock::SecretMaterialPresent)
        );
    }
}
