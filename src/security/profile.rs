// SPDX-License-Identifier: Apache-2.0
//! Security Behavior Profile — five black-box HTTP invariants.
//!
//! HelixTest already has HMAC JWT fixtures. This module productizes a named
//! Helix surface (`helix security`) with stable ids (`HLX-AUTH-010`–`014`).
//! It is **not** a security audit, pentest, or certification.
//! Secrets are test-only (`test-fixtures/hmac/`). Not HELIOS.

use common::report::{ComplianceLevel, TestCategory};
use reqwest::StatusCode;

use crate::identity::{self, CheckSpec, Severity};
use crate::security::jwt::{build_test_jwt, TestJwtSpec};

/// Dummy HMAC file. NICHT FÜR PRODUKTION. Never a production credential.
pub const HMAC_FIXTURE: &str = "test-fixtures/hmac/shared-secret.txt";

/// Protected resource every HTTP case probes (same object id as DRS fixtures).
pub const REQUEST_METHOD: &str = "GET";
pub const REQUEST_PATH: &str = "/objects/test-object-1";

/// Human report must include this sentence. Not an audit claim.
pub const SECURITY_BEHAVIOR_DISCLAIMER: &str = concat!(
    "Helix verifies selected security behavior. It is not a penetration test,\n",
    "security audit, or certification."
);

/// Catalog order matches [`crate::security::AUTH_CASE_NAMES`] / `HLX-AUTH-010`–`014`.
pub const HTTP_CASE_IDS: [&str; 5] = [
    "auth.helix.token.valid",
    "auth.helix.token.expired",
    "auth.helix.token.wrong_scope",
    "auth.helix.token.manipulated",
    "auth.helix.token.wrong_audience",
];

/// HTTP status class the target is allowed to return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    /// Credential accepted. Any 2xx.
    Success2xx,
    /// Token is not authentic (bad/garbage/expired). **401 only**, not 403.
    Unauthorized401,
    /// Authentic enough to parse, but not allowed here. **401 or 403**.
    UnauthorizedOrForbidden,
}

impl StatusClass {
    pub fn allows(self, status: StatusCode) -> bool {
        match self {
            Self::Success2xx => status.is_success(),
            Self::Unauthorized401 => status == StatusCode::UNAUTHORIZED,
            Self::UnauthorizedOrForbidden => {
                status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN
            }
        }
    }

    pub fn as_doc(self) -> &'static str {
        match self {
            Self::Success2xx => "2xx",
            Self::Unauthorized401 => "401",
            Self::UnauthorizedOrForbidden => "401 or 403",
        }
    }
}

/// Which dummy JWT (or garbage) the case sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    ValidDrs,
    ExpiredDrs,
    WrongScope,
    /// Flipped HS256 signature **and** the literal `not-a-jwt`.
    ManipulatedAndGarbage,
    WesAudience,
}

/// One HTTP invariant in the Security Behavior Profile.
#[derive(Debug, Clone, Copy)]
pub struct SecurityBehaviorCase {
    pub id: &'static str,
    /// What must hold. Not a certification claim.
    pub invariant: &'static str,
    pub token: TokenKind,
    pub acceptable: StatusClass,
}

impl SecurityBehaviorCase {
    pub fn spec(self) -> &'static CheckSpec {
        identity::spec(self.id)
    }

    pub fn code(self) -> &'static str {
        self.spec().code
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub fn severity(self) -> Severity {
        self.spec().severity
    }

    pub fn report_level(self) -> ComplianceLevel {
        ComplianceLevel::Level4
    }

    pub fn report_category(self) -> TestCategory {
        TestCategory::Security
    }

    pub fn fixture(self) -> &'static str {
        HMAC_FIXTURE
    }

    pub fn request_summary(self) -> String {
        format!(
            "{REQUEST_METHOD} {{drs_base}}{REQUEST_PATH}  Authorization: Bearer <{token}>",
            token = self.token.as_doc()
        )
    }

    pub fn mint_bearers(self, secret: &str) -> anyhow::Result<Vec<String>> {
        self.token.mint(secret)
    }
}

impl TokenKind {
    pub fn as_doc(self) -> &'static str {
        match self {
            Self::ValidDrs => "valid dummy HS256 (aud=drs, scope=drs.read, unexpired)",
            Self::ExpiredDrs => "expired dummy HS256 (aud=drs, scope=drs.read, exp in the past)",
            Self::WrongScope => "dummy HS256 (aud=drs, scope=wes.run)",
            Self::ManipulatedAndGarbage => "flipped signature, then literal not-a-jwt",
            Self::WesAudience => "dummy HS256 (aud=wes, scope=drs.read)",
        }
    }

    fn mint(self, secret: &str) -> anyhow::Result<Vec<String>> {
        match self {
            Self::ValidDrs => Ok(vec![build_test_jwt(secret, TestJwtSpec::valid_drs())?]),
            Self::ExpiredDrs => Ok(vec![build_test_jwt(secret, TestJwtSpec::expired_drs())?]),
            Self::WrongScope => Ok(vec![build_test_jwt(secret, TestJwtSpec::wrong_scope())?]),
            Self::ManipulatedAndGarbage => {
                let good = build_test_jwt(secret, TestJwtSpec::valid_drs())?;
                Ok(vec![flip_sig(&good), "not-a-jwt".into()])
            }
            Self::WesAudience => Ok(vec![build_test_jwt(secret, TestJwtSpec::wes_audience())?]),
        }
    }
}

pub const HTTP_CASES: [SecurityBehaviorCase; 5] = [
    SecurityBehaviorCase {
        id: "auth.helix.token.valid",
        invariant:
            "A correctly scoped, unexpired dummy token is accepted on the protected DRS object.",
        token: TokenKind::ValidDrs,
        acceptable: StatusClass::Success2xx,
    },
    SecurityBehaviorCase {
        id: "auth.helix.token.expired",
        invariant: "An expired dummy token is rejected (must not keep access after exp).",
        token: TokenKind::ExpiredDrs,
        acceptable: StatusClass::Unauthorized401,
    },
    SecurityBehaviorCase {
        id: "auth.helix.token.wrong_scope",
        invariant: "A dummy token with the wrong scope is denied on this DRS object.",
        token: TokenKind::WrongScope,
        acceptable: StatusClass::UnauthorizedOrForbidden,
    },
    SecurityBehaviorCase {
        id: "auth.helix.token.manipulated",
        invariant: "A forged or garbage Bearer is rejected (token integrity).",
        token: TokenKind::ManipulatedAndGarbage,
        acceptable: StatusClass::Unauthorized401,
    },
    SecurityBehaviorCase {
        id: "auth.helix.token.wrong_audience",
        invariant: "A dummy token minted for another service (WES) is denied on DRS.",
        token: TokenKind::WesAudience,
        acceptable: StatusClass::UnauthorizedOrForbidden,
    },
];

pub fn case_by_id(id: &str) -> Option<&'static SecurityBehaviorCase> {
    HTTP_CASES.iter().find(|c| c.id == id)
}

pub(crate) fn flip_sig(jwt: &str) -> String {
    let Some((head, sig)) = jwt.rsplit_once('.') else {
        return format!("{jwt}x");
    };
    let mut chars: Vec<char> = sig.chars().collect();
    match chars.first_mut() {
        Some(c) => *c = if *c == 'A' { 'B' } else { 'A' },
        None => chars.push('A'),
    }
    format!("{head}.{}", chars.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_identity_and_auth_names() {
        for (case, name) in HTTP_CASES.iter().zip(crate::security::AUTH_CASE_NAMES) {
            assert_eq!(case.name(), name);
            assert_eq!(case.spec().id, case.id);
            assert!(case.code().starts_with("HLX-AUTH-01"));
            assert_eq!(case.severity(), Severity::Error);
            assert!(case.spec().helixtest_names.is_empty());
            assert_eq!(case.fixture(), HMAC_FIXTURE);
        }
        assert_eq!(HTTP_CASES.len(), 5);
    }

    #[test]
    fn disclaimer_is_the_frozen_sentence() {
        assert!(SECURITY_BEHAVIOR_DISCLAIMER.contains("not a penetration test"));
        assert!(SECURITY_BEHAVIOR_DISCLAIMER.contains("security audit"));
        assert!(SECURITY_BEHAVIOR_DISCLAIMER.contains("certification"));
        assert!(!SECURITY_BEHAVIOR_DISCLAIMER
            .to_lowercase()
            .contains("proves"));
        assert!(!SECURITY_BEHAVIOR_DISCLAIMER
            .to_lowercase()
            .contains("secure implementation"));
    }

    #[test]
    fn status_classes() {
        assert!(StatusClass::Success2xx.allows(StatusCode::OK));
        assert!(StatusClass::Success2xx.allows(StatusCode::NO_CONTENT));
        assert!(!StatusClass::Success2xx.allows(StatusCode::UNAUTHORIZED));
        assert!(StatusClass::Unauthorized401.allows(StatusCode::UNAUTHORIZED));
        assert!(!StatusClass::Unauthorized401.allows(StatusCode::FORBIDDEN));
        assert!(StatusClass::UnauthorizedOrForbidden.allows(StatusCode::FORBIDDEN));
        assert!(StatusClass::UnauthorizedOrForbidden.allows(StatusCode::UNAUTHORIZED));
        assert!(!StatusClass::UnauthorizedOrForbidden.allows(StatusCode::OK));
    }
}
