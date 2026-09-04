// SPDX-License-Identifier: Apache-2.0
//! Black-box HTTP cases from the Security Behavior Profile.
//! Tokens are dummy HS256 JWTs. The target’s implementation is not imported.
//! Not a security audit. Passing does not prove the implementation is secure.

use anyhow::Result;
use common::report::{ServiceKind, ServiceReport, TestCaseResult};

use crate::discover::{discover, http_client, Ga4ghService};
use crate::security::profile::{SecurityBehaviorCase, HMAC_FIXTURE, HTTP_CASES, REQUEST_PATH};

pub fn protected_object_url(drs_base: &str) -> String {
    format!("{}{}", drs_base.trim_end_matches('/'), REQUEST_PATH)
}

fn skip_all(reason: &str) -> ServiceReport {
    ServiceReport {
        service: ServiceKind::Auth,
        tests: HTTP_CASES
            .iter()
            .map(|case| {
                TestCaseResult::skip(
                    case.name(),
                    case.report_level(),
                    case.report_category(),
                    crate::redact::redact_text(reason),
                )
            })
            .collect(),
    }
}

fn fail_all(reason: &str) -> ServiceReport {
    ServiceReport {
        service: ServiceKind::Auth,
        tests: HTTP_CASES
            .iter()
            .map(|case| {
                TestCaseResult::fail(
                    case.name(),
                    case.report_level(),
                    case.report_category(),
                    crate::redact::redact_text(reason),
                )
            })
            .collect(),
    }
}

pub async fn run_auth_http_cases(
    endpoint: &str,
    hmac_secret: Option<&str>,
) -> Result<ServiceReport> {
    let Some(secret) = hmac_secret.filter(|s| !s.is_empty()) else {
        return Ok(skip_all(
            "no HMAC test secret (set HELIX_HMAC_SECRET or --hmac-secret-file; fixture is test-fixtures/hmac/shared-secret.txt — NOT FOR PRODUCTION)",
        ));
    };

    let client = http_client()?;
    let discovery = discover(endpoint, &client).await?;
    let Some(drs) = discovery.get(Ga4ghService::Drs) else {
        return Ok(fail_all(
            "DRS not discovered; cannot probe an auth-protected object URL",
        ));
    };
    let url = protected_object_url(drs.base_url().expect("DETECTED DRS has a base URL"));

    let mut tests = Vec::with_capacity(HTTP_CASES.len());
    for case in &HTTP_CASES {
        tests.push(run_case(&client, &url, secret, case).await);
    }
    Ok(ServiceReport {
        service: ServiceKind::Auth,
        tests,
    })
}

async fn get_bearer(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<reqwest::StatusCode> {
    let resp = client.get(url).bearer_auth(token).send().await?;
    let status = resp.status();
    let _ =
        crate::http_safety::read_body_capped(resp, crate::http_safety::MAX_RESPONSE_BYTES).await;
    Ok(status)
}

async fn run_case(
    client: &reqwest::Client,
    url: &str,
    secret: &str,
    case: &SecurityBehaviorCase,
) -> TestCaseResult {
    let result = async {
        let bearers = case.mint_bearers(secret)?;
        for token in &bearers {
            let status = get_bearer(client, url, token).await?;
            anyhow::ensure!(
                case.acceptable.allows(status),
                "{code}: expected {want}, got {status} — {invariant} (fixture {fixture})",
                code = case.code(),
                want = case.acceptable.as_doc(),
                invariant = case.invariant,
                fixture = HMAC_FIXTURE,
            );
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        case.name(),
        case.report_level(),
        case.report_category(),
        result.map_err(|e| crate::redact::redact_with_secrets(&format!("{e:#}"), &[secret])),
    )
}
