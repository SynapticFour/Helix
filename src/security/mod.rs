// SPDX-License-Identifier: Apache-2.0
//! Security Behavior Profile: black-box HTTP against auth-protected GA4GH
//! endpoints, then Crypt4GH protocol-layout checks (not encryption).
//!
//! Builds on ga4gh-infra / Ferrum as **targets**, not as libraries. Not HELIOS.
//! HelixTest already has HMAC JWT fixtures and env-gated Crypt4GH HTTP (secret keys).
//! Helix does not call that Crypt4GH HTTP path. Passing does not prove the
//! implementation is secure.

mod crypt4gh_header;
mod http_cases;
mod jwt;
pub mod profile;

use anyhow::Result;
use common::report::{ServiceReport, TestStatus};

pub use crypt4gh_header::{
    run_crypt4gh_cases, validate_crypt4gh_header, Crypt4ghHeaderError, CRYPT4GH_CASE_IDS,
};
pub use jwt::{
    build_test_jwt, classify_bearer, classify_bearer_with, load_hmac_secret, TestJwtSpec,
    VerifierPolicy,
};
pub use profile::{
    StatusClass, HMAC_FIXTURE, HTTP_CASES, HTTP_CASE_IDS, SECURITY_BEHAVIOR_DISCLAIMER,
};

pub const AUTH_CASE_NAMES: [&str; 5] = [
    "Security: valid token grants access",
    "Security: expired token rejected with 401",
    "Security: wrong scope denied",
    "Security: invalid or manipulated token rejected",
    "Security: token for another service rejected",
];

pub const CRYPT4GH_CASE_NAME: &str =
    "Security: Crypt4GH header structure is well-formed (no key material in output)";

#[derive(Debug, Clone)]
pub struct SecurityOutcome {
    pub auth: ServiceReport,
    pub crypt4gh: ServiceReport,
}

impl SecurityOutcome {
    pub fn has_failures(&self) -> bool {
        self.auth
            .tests
            .iter()
            .chain(self.crypt4gh.tests.iter())
            .any(|t| t.status == TestStatus::Fail)
    }

    pub fn auth_status(&self, name: &str) -> Option<TestStatus> {
        self.auth
            .tests
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.status)
    }
}

/// Run the Security Behavior Profile against `endpoint` (DRS object URL discovered under it).
pub async fn run_security(
    endpoint: &str,
    hmac_secret: Option<&str>,
    crypt4gh_path: Option<&std::path::Path>,
) -> Result<SecurityOutcome> {
    let auth = http_cases::run_auth_http_cases(endpoint, hmac_secret).await?;
    let crypt4gh = crypt4gh_header::run_crypt4gh_cases(endpoint, crypt4gh_path).await?;
    Ok(SecurityOutcome { auth, crypt4gh })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::http_cases::protected_object_url;
    use common::report::TestStatus;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    const SECRET: &str = "helix-dummy-hmac-not-for-production-do-not-use";

    struct AuthGate {
        policy: VerifierPolicy,
    }

    impl wiremock::Respond for AuthGate {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let header = request
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let token = header.strip_prefix("Bearer ").unwrap_or("");
            let code = classify_bearer_with(token, SECRET, "drs", "drs.read", self.policy);
            ResponseTemplate::new(code).set_body_string(if code == 200 {
                r#"{"id":"test-object-1"}"#
            } else {
                "denied"
            })
        }
    }

    async fn start_auth_mock(policy: VerifierPolicy) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(AuthGate { policy })
            .mount(&server)
            .await;
        server
    }

    fn well_formed() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/crypt4gh/well-formed.c4gh")
    }

    async fn run(policy: VerifierPolicy) -> SecurityOutcome {
        let server = start_auth_mock(policy).await;
        let fixture = well_formed();
        run_security(&server.uri(), Some(SECRET), Some(&fixture))
            .await
            .expect("security")
    }

    fn names(out: &SecurityOutcome) -> Vec<&str> {
        out.auth.tests.iter().map(|t| t.name.as_str()).collect()
    }

    #[tokio::test]
    async fn fail_closed_mock_passes_all_five_http_cases() {
        let out = run(VerifierPolicy::fail_closed()).await;
        assert!(!out.has_failures(), "{:?}", out.auth.tests);
        assert_eq!(names(&out), AUTH_CASE_NAMES);
        for t in &out.auth.tests {
            assert_eq!(t.status, TestStatus::Pass, "{}", t.name);
        }
        assert_eq!(out.crypt4gh.tests.len(), 3);
        assert_eq!(out.crypt4gh.tests[0].name, CRYPT4GH_CASE_NAME);
        assert_eq!(out.crypt4gh.tests[0].status, TestStatus::Pass);
        assert_eq!(out.crypt4gh.tests[1].status, TestStatus::Pass);
        assert_eq!(
            out.crypt4gh.tests[2].status,
            TestStatus::Skip,
            "plaintext auth mock is not a Crypt4GH envelope"
        );
        let server = start_auth_mock(VerifierPolicy::fail_closed()).await;
        assert!(protected_object_url(&server.uri()).contains("test-object-1"));
    }

    #[tokio::test]
    async fn missing_secret_skips_http_not_pass() {
        let server = start_auth_mock(VerifierPolicy::fail_closed()).await;
        let fixture = well_formed();
        let out = run_security(&server.uri(), None, Some(&fixture))
            .await
            .expect("security");
        assert!(!out.has_failures());
        assert!(out.auth.tests.iter().all(|t| t.status == TestStatus::Skip));
        assert_eq!(out.auth.tests.len(), 5);
        assert_eq!(out.crypt4gh.tests[0].status, TestStatus::Pass);
    }

    #[tokio::test]
    async fn closed_gate_is_detected_as_valid_token_failure() {
        let out = run(VerifierPolicy::reject_all()).await;
        assert_eq!(
            out.auth_status(AUTH_CASE_NAMES[0]),
            Some(TestStatus::Fail),
            "valid token must fail when the mock rejects everyone"
        );
        assert!(
            out.auth.tests[0]
                .error
                .as_deref()
                .unwrap_or("")
                .contains("HLX-AUTH-010"),
            "{:?}",
            out.auth.tests[0].error
        );
        // Denial cases still expect 401, so they pass on a closed gate.
        for name in &AUTH_CASE_NAMES[1..] {
            assert_eq!(
                out.auth_status(name),
                Some(TestStatus::Pass),
                "{name} should still pass on always-401"
            );
        }
    }

    #[tokio::test]
    async fn ignore_expiry_is_detected() {
        let out = run(VerifierPolicy::ignore_expiry()).await;
        assert_eq!(
            out.auth_status("Security: expired token rejected with 401"),
            Some(TestStatus::Fail)
        );
        assert_eq!(
            out.auth_status("Security: valid token grants access"),
            Some(TestStatus::Pass)
        );
    }

    #[tokio::test]
    async fn ignore_scope_is_detected() {
        let out = run(VerifierPolicy::ignore_scope()).await;
        assert_eq!(
            out.auth_status("Security: wrong scope denied"),
            Some(TestStatus::Fail)
        );
        assert_eq!(
            out.auth_status("Security: valid token grants access"),
            Some(TestStatus::Pass)
        );
    }

    #[tokio::test]
    async fn ignore_audience_is_detected() {
        let out = run(VerifierPolicy::ignore_audience()).await;
        assert_eq!(
            out.auth_status("Security: token for another service rejected"),
            Some(TestStatus::Fail)
        );
        assert_eq!(
            out.auth_status("Security: valid token grants access"),
            Some(TestStatus::Pass)
        );
    }

    #[tokio::test]
    async fn ignore_signature_is_detected() {
        let out = run(VerifierPolicy::ignore_signature()).await;
        assert_eq!(
            out.auth_status("Security: invalid or manipulated token rejected"),
            Some(TestStatus::Fail)
        );
        assert_eq!(
            out.auth_status("Security: valid token grants access"),
            Some(TestStatus::Pass)
        );
    }
}
