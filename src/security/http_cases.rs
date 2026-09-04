// SPDX-License-Identifier: Apache-2.0
//! Black-box HTTP cases against a DRS object URL discovered under the gateway origin.
//! Tokens are dummy HS256 JWTs. The target’s implementation is not imported.

use anyhow::Result;
use common::report::{ComplianceLevel, ServiceKind, ServiceReport, TestCaseResult, TestCategory};
use reqwest::StatusCode;

use crate::discover::{discover, http_client, Ga4ghService};
use crate::security::jwt::{build_test_jwt, TestJwtSpec};

pub fn protected_object_url(drs_base: &str) -> String {
    format!("{}/objects/test-object-1", drs_base.trim_end_matches('/'))
}

fn skip_all(reason: &str) -> ServiceReport {
    ServiceReport {
        service: ServiceKind::Auth,
        tests: crate::security::AUTH_CASE_NAMES
            .iter()
            .map(|name| {
                TestCaseResult::skip(
                    *name,
                    ComplianceLevel::Level4,
                    TestCategory::Security,
                    reason,
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
        return Ok(ServiceReport {
            service: ServiceKind::Auth,
            tests: vec![TestCaseResult::fail(
                crate::security::AUTH_CASE_NAMES[0],
                ComplianceLevel::Level4,
                TestCategory::Security,
                "DRS not discovered; cannot probe an auth-protected object URL",
            )],
        });
    };
    let url = protected_object_url(&drs.base_url);

    Ok(ServiceReport {
        service: ServiceKind::Auth,
        tests: vec![
            case_valid(&client, &url, secret).await,
            case_expired(&client, &url, secret).await,
            case_wrong_scope(&client, &url, secret).await,
            case_manipulated(&client, &url, secret).await,
            case_cross_service(&client, &url, secret).await,
        ],
    })
}

async fn get_bearer(client: &reqwest::Client, url: &str, token: &str) -> Result<StatusCode> {
    Ok(client.get(url).bearer_auth(token).send().await?.status())
}

/// Valid token → access allowed.
/// Covers: a correctly scoped, unexpired credential must still work — otherwise operators
/// cannot tell fail-closed auth from a broken verifier that rejects everyone.
async fn case_valid(client: &reqwest::Client, url: &str, secret: &str) -> TestCaseResult {
    const NAME: &str = "Security: valid token grants access";
    let result = async {
        let token = build_test_jwt(secret, TestJwtSpec::valid_drs())?;
        let status = get_bearer(client, url, &token).await?;
        anyhow::ensure!(
            status.is_success(),
            "valid dummy JWT should be accepted, got {status}"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level4,
        TestCategory::Security,
        result,
    )
}

/// Expired token → denied with 401.
/// Covers: sessions that outlive `exp` must not keep access (stolen or leftover tokens).
async fn case_expired(client: &reqwest::Client, url: &str, secret: &str) -> TestCaseResult {
    const NAME: &str = "Security: expired token rejected with 401";
    let result = async {
        let token = build_test_jwt(secret, TestJwtSpec::expired_drs())?;
        let status = get_bearer(client, url, &token).await?;
        anyhow::ensure!(
            status == StatusCode::UNAUTHORIZED,
            "expired token must be HTTP 401, got {status}"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level4,
        TestCategory::Security,
        result,
    )
}

/// Wrong scope → denied.
/// Covers: a token that can run WES must not read DRS objects (least privilege / confused deputy).
async fn case_wrong_scope(client: &reqwest::Client, url: &str, secret: &str) -> TestCaseResult {
    const NAME: &str = "Security: wrong scope denied";
    let result = async {
        let token = build_test_jwt(secret, TestJwtSpec::wrong_scope())?;
        let status = get_bearer(client, url, &token).await?;
        anyhow::ensure!(
            status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED,
            "wrong scope must be 403 or 401, got {status}"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level4,
        TestCategory::Security,
        result,
    )
}

/// Manipulated / garbage token → 401.
/// Covers: forged or truncated Bearers must not authenticate (integrity of the token).
async fn case_manipulated(client: &reqwest::Client, url: &str, secret: &str) -> TestCaseResult {
    const NAME: &str = "Security: invalid or manipulated token rejected";
    let result = async {
        let good = build_test_jwt(secret, TestJwtSpec::valid_drs())?;
        let flipped = flip_sig(&good);
        let st_flip = get_bearer(client, url, &flipped).await?;
        anyhow::ensure!(
            st_flip == StatusCode::UNAUTHORIZED,
            "manipulated signature must be 401, got {st_flip}"
        );
        let st_junk = get_bearer(client, url, "not-a-jwt").await?;
        anyhow::ensure!(
            st_junk == StatusCode::UNAUTHORIZED,
            "garbage Bearer must be 401, got {st_junk}"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level4,
        TestCategory::Security,
        result,
    )
}

/// Token minted for service A (WES) sent to service B (DRS) → denied.
/// Covers: audience confusion — a WES access token must not unlock DRS.
async fn case_cross_service(client: &reqwest::Client, url: &str, secret: &str) -> TestCaseResult {
    const NAME: &str = "Security: token for another service rejected";
    let result = async {
        let token = build_test_jwt(secret, TestJwtSpec::wes_audience())?;
        let status = get_bearer(client, url, &token).await?;
        anyhow::ensure!(
            status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED,
            "WES-audience token on DRS must be 403 or 401, got {status}"
        );
        Ok::<(), anyhow::Error>(())
    }
    .await;
    TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level4,
        TestCategory::Security,
        result,
    )
}

fn flip_sig(jwt: &str) -> String {
    let mut chars: Vec<char> = jwt.chars().collect();
    if let Some(last) = chars.last_mut() {
        *last = if *last == 'A' { 'B' } else { 'A' };
    }
    chars.into_iter().collect()
}
