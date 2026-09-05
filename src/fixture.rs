// SPDX-License-Identifier: Apache-2.0
//! Target-scoped DRS test input. Not a GA4GH requirement. Not HELIOS.
//!
//! The DRS 1.4.0 DrsObject schema does not require object id `test-object-1`.
//! That string is Helix/HelixTest catalog input. Operator `--drs-object-id`
//! replaces it per target. A missing configured object is fixture-unavailable,
//! not target non-conformance.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Default object id for the in-process mock catalog ([docs/FIXTURES.md](../docs/FIXTURES.md)).
pub const DEFAULT_DRS_OBJECT_ID: &str = "test-object-1";

/// Max DRS object id length accepted from the operator. Not a GA4GH limit.
pub const MAX_DRS_OBJECT_ID_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureSource {
    DefaultCatalog,
    OperatorDeclared,
}

/// How checksum evidence is formed. Not a second trusted blob oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumMode {
    /// Operator `--drs-object-sha256` compared to downloaded bytes.
    OperatorDigest,
    /// Advertised GetObject sha256 vs downloaded bytes. Internal consistency only.
    #[default]
    AdvertisedConsistency,
}

impl ChecksumMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OperatorDigest => "operator_digest",
            Self::AdvertisedConsistency => "advertised_consistency",
        }
    }
}

impl FixtureSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DefaultCatalog => "default_catalog",
            Self::OperatorDeclared => "operator_declared",
        }
    }
}

/// Test data Helix will ask the target for. Not certification. Not pack identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrsVerifyFixture {
    pub object_id: String,
    pub unknown_object_id: String,
    pub source: FixtureSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_sha256: Option<String>,
    #[serde(default)]
    pub checksum_mode: ChecksumMode,
}

impl Default for DrsVerifyFixture {
    fn default() -> Self {
        Self::default_catalog()
    }
}

impl DrsVerifyFixture {
    pub fn default_catalog() -> Self {
        let object_id = DEFAULT_DRS_OBJECT_ID.to_string();
        Self {
            unknown_object_id: framework::drs::unknown_object_id_for(&object_id),
            object_id,
            source: FixtureSource::DefaultCatalog,
            expected_sha256: None,
            checksum_mode: ChecksumMode::AdvertisedConsistency,
        }
    }

    pub fn operator_declared(object_id: String, expected_sha256: Option<String>) -> Result<Self> {
        validate_object_id(&object_id)?;
        if let Some(ref hex) = expected_sha256 {
            validate_sha256_hex(hex)?;
        }
        let unknown_object_id = framework::drs::unknown_object_id_for(&object_id);
        Ok(Self {
            object_id,
            unknown_object_id,
            source: FixtureSource::OperatorDeclared,
            checksum_mode: if expected_sha256.is_some() {
                ChecksumMode::OperatorDigest
            } else {
                ChecksumMode::AdvertisedConsistency
            },
            expected_sha256,
        })
    }

    pub fn to_helixtest(&self) -> framework::drs::DrsTestFixture {
        framework::drs::DrsTestFixture {
            object_id: self.object_id.clone(),
            expected_sha256: self.expected_sha256.clone(),
        }
    }
}

pub fn validate_object_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("--drs-object-id must be non-empty");
    }
    if id.len() > MAX_DRS_OBJECT_ID_BYTES {
        bail!("--drs-object-id exceeds {MAX_DRS_OBJECT_ID_BYTES} bytes");
    }
    if id.contains("://")
        || id.contains('/')
        || id.contains('\\')
        || id.contains('?')
        || id.contains('#')
        || id.contains("..")
        || id.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        bail!("--drs-object-id must be a DRS object id, not a URL or path");
    }
    Ok(())
}

pub fn validate_sha256_hex(hex: &str) -> Result<()> {
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("--drs-object-sha256 must be 64 hex characters");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_and_url_object_ids() {
        assert!(validate_object_id("../x").is_err());
        assert!(validate_object_id("http://evil").is_err());
        assert!(validate_object_id("a/b").is_err());
        assert!(validate_object_id("a b").is_err());
        assert!(validate_object_id("").is_err());
        assert!(validate_object_id("b8cd0667-2c33-4c9f-967b-161b905932c9").is_ok());
        assert!(validate_object_id(DEFAULT_DRS_OBJECT_ID).is_ok());
    }

    #[test]
    fn unknown_id_differs_from_positive_fixture() {
        let a = DrsVerifyFixture::default_catalog();
        let b = DrsVerifyFixture::operator_declared(
            "b8cd0667-2c33-4c9f-967b-161b905932c9".into(),
            None,
        )
        .unwrap();
        assert_ne!(a.object_id, a.unknown_object_id);
        assert_ne!(b.object_id, b.unknown_object_id);
        assert_ne!(a.unknown_object_id, b.unknown_object_id);
        assert!(a.unknown_object_id.starts_with("helix.unknown."));
    }
}
