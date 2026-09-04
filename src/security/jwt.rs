// SPDX-License-Identifier: Apache-2.0
//! Dummy HS256 JWTs for Stage 3. Uses HelixTest `common::auth::build_jwt`.
//! Never log the secret. Fixture: `test-fixtures/hmac/shared-secret.txt`.

use anyhow::{bail, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::Path;

use common::auth::build_jwt;

pub struct TestJwtSpec {
    pub audience: &'static str,
    pub scope: &'static str,
    pub lifetime: Duration,
}

impl TestJwtSpec {
    pub fn valid_drs() -> Self {
        Self {
            audience: "drs",
            scope: "drs.read",
            lifetime: Duration::minutes(5),
        }
    }

    pub fn expired_drs() -> Self {
        Self {
            audience: "drs",
            scope: "drs.read",
            lifetime: Duration::minutes(-5),
        }
    }

    pub fn wrong_scope() -> Self {
        Self {
            audience: "drs",
            scope: "wes.run",
            lifetime: Duration::minutes(5),
        }
    }

    /// Token minted for WES, sent to DRS — cross-service reuse.
    pub fn wes_audience() -> Self {
        Self {
            audience: "wes",
            scope: "drs.read",
            lifetime: Duration::minutes(5),
        }
    }
}

pub fn build_test_jwt(secret: &str, spec: TestJwtSpec) -> Result<String> {
    build_jwt(
        "https://helix.test.invalid",
        "helix-stage3-fixture-user",
        spec.audience,
        spec.scope,
        spec.lifetime,
        secret,
    )
}

pub fn load_hmac_secret(path: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(path)?;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t.contains("PRODUCTION") && !t.contains("NOT") && !t.contains("NICHT") {
            bail!("refusing to load a secret file that looks unmarked as test-only");
        }
        return Ok(t.to_string());
    }
    bail!("no secret line in {}", path.display());
}

/// HTTP status a fail-closed DRS should return for this Bearer (test mock + docs).
pub fn classify_bearer(token: &str, secret: &str, required_aud: &str, required_scope: &str) -> u16 {
    if token.is_empty() || token == "not-a-jwt" || token.chars().filter(|c| *c == '.').count() != 2
    {
        return 401;
    }
    let mut parts = token.split('.');
    let (Some(h), Some(p), Some(s)) = (parts.next(), parts.next(), parts.next()) else {
        return 401;
    };
    let signing_input = format!("{h}.{p}");
    let Ok(sig) = general_purpose::URL_SAFE_NO_PAD.decode(s) else {
        return 401;
    };
    type HmacSha256 = Hmac<Sha256>;
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return 401;
    };
    mac.update(signing_input.as_bytes());
    if mac.verify_slice(&sig).is_err() {
        return 401;
    }
    let Ok(payload_bytes) = general_purpose::URL_SAFE_NO_PAD.decode(p) else {
        return 401;
    };
    let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&payload_bytes) else {
        return 401;
    };
    let exp = claims.get("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    if exp < Utc::now().timestamp() {
        return 401;
    }
    let aud = claims.get("aud").and_then(|v| v.as_str()).unwrap_or("");
    if aud != required_aud {
        return 403;
    }
    let scope = claims.get("scope").and_then(|v| v.as_str()).unwrap_or("");
    if !scope.split_whitespace().any(|s| s == required_scope) {
        return 403;
    }
    200
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_commented_fixture_file() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/hmac/shared-secret.txt");
        let s = load_hmac_secret(&p).unwrap();
        assert!(s.starts_with("helix-dummy-hmac"));
        assert!(!s.contains('\n'));
    }

    #[test]
    fn expired_classifies_401() {
        let secret = "helix-dummy-hmac-not-for-production-do-not-use";
        let tok = build_test_jwt(secret, TestJwtSpec::expired_drs()).unwrap();
        assert_eq!(classify_bearer(&tok, secret, "drs", "drs.read"), 401);
    }

    #[test]
    fn wes_audience_classifies_403() {
        let secret = "helix-dummy-hmac-not-for-production-do-not-use";
        let tok = build_test_jwt(secret, TestJwtSpec::wes_audience()).unwrap();
        assert_eq!(classify_bearer(&tok, secret, "drs", "drs.read"), 403);
    }
}
