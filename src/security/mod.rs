// SPDX-License-Identifier: Apache-2.0
//! Stage 3 security behaviour: black-box HTTP against auth-protected GA4GH endpoints,
//! plus Crypt4GH header structure (no key material in output).
//!
//! Builds on ga4gh-infra / Ferrum as **targets**, not as libraries. Not HELIOS.
//! HelixTest already has HMAC JWT fixtures; this module is the Helix-named surface.

mod crypt4gh_header;
mod http_cases;
mod jwt;

use anyhow::Result;
use common::report::{ServiceKind, ServiceReport, TestStatus};

pub use crypt4gh_header::{validate_crypt4gh_header, Crypt4ghHeaderError};
pub use jwt::{build_test_jwt, classify_bearer, load_hmac_secret, TestJwtSpec};

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
}

/// Run Stage 3 cases against `endpoint` (DRS object URL discovered under it).
pub async fn run_security(
    endpoint: &str,
    hmac_secret: Option<&str>,
    crypt4gh_path: Option<&std::path::Path>,
) -> Result<SecurityOutcome> {
    let auth = http_cases::run_auth_http_cases(endpoint, hmac_secret).await?;
    let crypt4gh = ServiceReport {
        service: ServiceKind::Crypt4gh,
        tests: vec![crypt4gh_header::crypt4gh_header_case(crypt4gh_path)?],
    };
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

    struct AuthGate;

    impl wiremock::Respond for AuthGate {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let header = request
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let token = header.strip_prefix("Bearer ").unwrap_or("");
            let code = jwt::classify_bearer(token, SECRET, "drs", "drs.read");
            ResponseTemplate::new(code).set_body_string(if code == 200 {
                r#"{"id":"test-object-1"}"#
            } else {
                "denied"
            })
        }
    }

    async fn start_auth_mock() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(AuthGate)
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn six_stage3_cases_against_mock() {
        let server = start_auth_mock().await;
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/crypt4gh/well-formed.c4gh");
        let out = run_security(&server.uri(), Some(SECRET), Some(&fixture))
            .await
            .expect("security");
        assert!(!out.has_failures(), "{:?}", out.auth.tests);
        let names: Vec<_> = out.auth.tests.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, AUTH_CASE_NAMES);
        for t in &out.auth.tests {
            assert_eq!(t.status, TestStatus::Pass, "{}", t.name);
        }
        assert_eq!(out.crypt4gh.tests[0].name, CRYPT4GH_CASE_NAME);
        assert_eq!(out.crypt4gh.tests[0].status, TestStatus::Pass);
        assert!(protected_object_url(&server.uri()).contains("test-object-1"));
    }

    #[tokio::test]
    async fn missing_secret_skips_http_not_pass() {
        let server = start_auth_mock().await;
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/crypt4gh/well-formed.c4gh");
        let out = run_security(&server.uri(), None, Some(&fixture))
            .await
            .expect("security");
        assert!(!out.has_failures());
        assert!(out.auth.tests.iter().all(|t| t.status == TestStatus::Skip));
        assert_eq!(out.crypt4gh.tests[0].status, TestStatus::Pass);
    }
}
