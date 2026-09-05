// SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseClass {
    Official,
    Ballot,
    Snapshot,
    Development,
}

impl ReleaseClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Ballot => "ballot",
            Self::Snapshot => "snapshot",
            Self::Development => "development",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportStatus {
    Available,
    Supported,
}

impl SupportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Supported => "supported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRole {
    Openapi,
    JsonSchema,
    Specification,
    Other,
}

/// Claim taxonomy for a Helix check or registry binding.
///
/// JSON names: `traceability.category` and `traceability.check_kind` (must match).
/// This is **not** `VerificationResult.category` (domain: schema, lifecycle, …).
///
/// `normative` is allowed only with a complete GA4GH source locator in a pinned
/// file Helix actually loads. Helix-defined behaviour must not use `normative`.
/// `guidance` is official GA4GH implementation guidance, not HelixTest policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    Normative,
    /// Official implementation guidance. Wire name `guidance`.
    /// `implementation_guidance` is accepted on deserialize only.
    #[serde(alias = "implementation_guidance")]
    Guidance,
    Fixture,
    Interoperability,
    Security,
    Benchmark,
}

impl BindingKind {
    pub const ALL: [BindingKind; 6] = [
        Self::Normative,
        Self::Guidance,
        Self::Fixture,
        Self::Interoperability,
        Self::Security,
        Self::Benchmark,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normative => "normative",
            Self::Guidance => "guidance",
            Self::Fixture => "fixture",
            Self::Interoperability => "interoperability",
            Self::Security => "security",
            Self::Benchmark => "benchmark",
        }
    }

    /// What a PASS of this kind is allowed to mean.
    pub fn claim_scope(self) -> ClaimScope {
        match self {
            Self::Normative => ClaimScope::Ga4ghRequirement,
            Self::Guidance => ClaimScope::GuidanceNotRequirement,
            Self::Fixture => ClaimScope::HelixFixture,
            Self::Interoperability => ClaimScope::InteroperabilityObservation,
            Self::Security => ClaimScope::SecurityBehavior,
            Self::Benchmark => ClaimScope::PerformanceMeasurement,
        }
    }

    /// Only this kind may back a “verified against GA4GH …” sentence.
    pub fn may_claim_ga4gh_requirement(self) -> bool {
        matches!(self, Self::Normative)
    }
}

/// What a PASS is allowed to mean. Must match [`BindingKind::claim_scope`].
///
/// A fixture/security/benchmark/interoperability/guidance PASS is never a
/// GA4GH conformance claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimScope {
    Ga4ghRequirement,
    GuidanceNotRequirement,
    HelixFixture,
    InteroperabilityObservation,
    SecurityBehavior,
    PerformanceMeasurement,
}

impl ClaimScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ga4ghRequirement => "ga4gh_requirement",
            Self::GuidanceNotRequirement => "guidance_not_requirement",
            Self::HelixFixture => "helix_fixture",
            Self::InteroperabilityObservation => "interoperability_observation",
            Self::SecurityBehavior => "security_behavior",
            Self::PerformanceMeasurement => "performance_measurement",
        }
    }

    pub fn may_support_conformance_claim(self) -> bool {
        matches!(self, Self::Ga4ghRequirement)
    }
}

/// Declared check coverage for a supported pack. Not a score or percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageLevel {
    Complete,
    Partial,
    None,
}

impl CoverageLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackCoverage {
    pub schema: CoverageLevel,
    pub behavior: CoverageLevel,
    pub security: CoverageLevel,
    pub interoperability: CoverageLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocatorType {
    OperationId,
    SchemaName,
    JsonPointer,
    HttpPath,
    StatusCode,
    Quote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Integrity {
    pub algorithm: String,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionSource {
    pub path: String,
    pub source_url: String,
    pub role: SourceRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_path: Option<String>,
    pub integrity: Integrity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionCitation {
    pub source_path: String,
    pub locator_type: LocatorType,
    pub locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestBinding {
    pub id: String,
    pub code: String,
    pub kind: BindingKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<VersionCitation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardVersion {
    pub schema_version: String,
    pub pack_id: String,
    pub standard: String,
    pub product: String,
    pub version: String,
    pub release_class: ReleaseClass,
    pub support_status: SupportStatus,
    pub repository: String,
    pub release_ref: String,
    pub commit: String,
    pub retrieved_at: String,
    pub normative_sources: Vec<VersionSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_integrity: Option<Integrity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_entry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_component: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_bindings: Option<Vec<TestBinding>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_catalog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<PackCoverage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl StandardVersion {
    /// SUPPORTED pack that default automation may use only if also OFFICIAL.
    pub fn is_supported(&self) -> bool {
        self.support_status == SupportStatus::Supported
            && self.release_class != ReleaseClass::Development
    }

    pub fn in_default_discovery(&self) -> bool {
        self.release_class == ReleaseClass::Official
            && self.support_status == SupportStatus::Supported
    }

    pub fn summary_label(&self) -> String {
        format!(
            "{} ({}, {})",
            self.pack_id,
            self.support_status.as_str(),
            self.release_class.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guidance_serializes_as_guidance_not_implementation_guidance() {
        let json = serde_json::to_value(BindingKind::Guidance).unwrap();
        assert_eq!(json, serde_json::json!("guidance"));
        let back: BindingKind = serde_json::from_value(json).unwrap();
        assert_eq!(back, BindingKind::Guidance);
        let alias: BindingKind = serde_json::from_str("\"implementation_guidance\"").unwrap();
        assert_eq!(alias, BindingKind::Guidance);
        assert_eq!(alias.as_str(), "guidance");
    }

    #[test]
    fn claim_scope_matches_category() {
        assert_eq!(
            BindingKind::Normative.claim_scope(),
            ClaimScope::Ga4ghRequirement
        );
        assert!(!BindingKind::Fixture
            .claim_scope()
            .may_support_conformance_claim());
        assert!(!BindingKind::Security
            .claim_scope()
            .may_support_conformance_claim());
        assert!(!BindingKind::Benchmark
            .claim_scope()
            .may_support_conformance_claim());
        assert!(!BindingKind::Interoperability
            .claim_scope()
            .may_support_conformance_claim());
        assert!(!BindingKind::Guidance
            .claim_scope()
            .may_support_conformance_claim());
        assert!(BindingKind::Normative.may_claim_ga4gh_requirement());
    }
}
