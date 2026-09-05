// SPDX-License-Identifier: Apache-2.0
//! First-class target identity. A target is not a standard, pack, or checker.
//!
//! Declared metadata is untrusted. Detected service-info is untrusted.
//! Neither becomes `verified_version`. Ferrum is not imported. Not HELIOS.

use common::spec_source::sha256_hex;
use serde::{Deserialize, Serialize};

use crate::diagnostics::DiagnosticCategory;
use crate::model::{StandardSelection, VerificationResult, VerificationStatus};
use crate::standards::BindingKind;

/// B4 classification of what a target is. Operator-declared. Never inferred
/// from HTTP headers, Docker image names, package names, or URL paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// In-process wiremock / HelixTest mock. Not independent evidence.
    Mock,
    /// Documented fixture catalog target. Not independent evidence.
    Fixture,
    /// Deterministic test server used to prove failure isolation. Not independent evidence.
    SyntheticTarget,
    /// Live reference stack (e.g. Ferrum). Real, not a second independent implementation.
    ReferenceImplementation,
    /// Distinct local implementation, operator-declared. Counts as independent evidence.
    RealIndependentLocalImplementation,
    /// Distinct external implementation, operator-declared. Counts as independent evidence.
    RealExternalImplementation,
    /// Kind not stated. Fail closed: not independent evidence.
    #[default]
    Unspecified,
}

impl TargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Fixture => "fixture",
            Self::SyntheticTarget => "synthetic_target",
            Self::ReferenceImplementation => "reference_implementation",
            Self::RealIndependentLocalImplementation => "real_independent_local_implementation",
            Self::RealExternalImplementation => "real_external_implementation",
            Self::Unspecified => "unspecified",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mock" => Some(Self::Mock),
            "fixture" => Some(Self::Fixture),
            "synthetic_target" => Some(Self::SyntheticTarget),
            "reference_implementation" => Some(Self::ReferenceImplementation),
            "real_independent_local_implementation" => {
                Some(Self::RealIndependentLocalImplementation)
            }
            "real_external_implementation" => Some(Self::RealExternalImplementation),
            "unspecified" => Some(Self::Unspecified),
            _ => None,
        }
    }

    /// Section 7: only real external / real independent local count.
    pub fn qualifies_as_independent_implementation(self) -> bool {
        matches!(
            self,
            Self::RealIndependentLocalImplementation | Self::RealExternalImplementation
        )
    }

    pub fn is_mock_or_fixture_or_synthetic(self) -> bool {
        matches!(self, Self::Mock | Self::Fixture | Self::SyntheticTarget)
    }
}

/// Operator-supplied labels. Untrusted. Must not satisfy claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeclaredTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(default)]
    pub kind: TargetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,
    /// Declared GA4GH standard version for this target. Untrusted. Not verified_version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_version: Option<String>,
}

/// Strings Helix observed (e.g. service-info). Untrusted. Not proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DetectedTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_version: Option<String>,
}

/// What Helix actually executed against. Endpoint + declared id. Not a version claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTargetIdentity {
    pub target_id: String,
    pub endpoint: String,
    pub target_kind: TargetKind,
}

/// Answers: what exactly was verified? Distinct from pack/checker/catalog identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetIdentity {
    pub target_id: String,
    pub target_kind: TargetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,
    pub endpoint: String,
    pub declared: DeclaredTarget,
    #[serde(default, skip_serializing_if = "DetectedTarget::is_empty")]
    pub detected: DetectedTarget,
    pub verified: VerifiedTargetIdentity,
}

impl DetectedTarget {
    fn is_empty(&self) -> bool {
        self.implementation_name.is_none()
            && self.implementation_version.is_none()
            && self.standard_version.is_none()
    }
}

impl TargetIdentity {
    /// Identity for an operator URL. Does not infer implementation version.
    pub fn from_declared(endpoint: &str, declared: &DeclaredTarget) -> Self {
        let endpoint = endpoint.trim_end_matches('/').to_string();
        let target_id = declared
            .target_id
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("endpoint:{endpoint}"));
        let kind = declared.kind;
        Self {
            target_id: target_id.clone(),
            target_kind: kind,
            implementation_name: declared.implementation_name.clone(),
            implementation_version: declared.implementation_version.clone(),
            endpoint: endpoint.clone(),
            declared: declared.clone(),
            detected: DetectedTarget::default(),
            verified: VerifiedTargetIdentity {
                target_id,
                endpoint,
                target_kind: kind,
            },
        }
    }

    pub fn unspecified(endpoint: &str) -> Self {
        Self::from_declared(endpoint, &DeclaredTarget::default())
    }
}

/// Minimum HTTP seam. Production impls expose identity + public base URL only.
pub trait DrsTarget: Send + Sync {
    fn identity(&self) -> &TargetIdentity;
    fn base_url(&self) -> &str;
}

/// Public HTTP origin. Cannot carry Ferrum internals: the trait has no Ferrum types.
#[derive(Debug, Clone)]
pub struct HttpDrsTarget {
    identity: TargetIdentity,
}

impl HttpDrsTarget {
    pub fn new(identity: TargetIdentity) -> Self {
        Self { identity }
    }
}

impl DrsTarget for HttpDrsTarget {
    fn identity(&self) -> &TargetIdentity {
        &self.identity
    }

    fn base_url(&self) -> &str {
        &self.identity.endpoint
    }
}

/// Spec-join `execution_id` stays pack/checker only (B2/B3). This identity
/// additionally binds the target so Target A cannot reuse Target B's run.
pub fn target_execution_id(identity: &TargetIdentity, sel: &StandardSelection) -> String {
    let canonical = format!(
        "target_id={}\n\
         target_kind={}\n\
         endpoint={}\n\
         pack_id={}\n\
         pack_integrity_sha256={}\n\
         schema_document_sha256={}\n\
         checker_id={}\n\
         binding_id={}\n\
         catalog_id={}\n\
         selected_version={}\n\
         spec_execution_id={}\n",
        identity.target_id,
        identity.target_kind.as_str(),
        identity.endpoint,
        sel.standards_registry_entry.as_deref().unwrap_or(""),
        sel.pack_integrity_sha256.as_deref().unwrap_or(""),
        sel.schema_document_sha256.as_deref().unwrap_or(""),
        sel.checker_id.as_deref().unwrap_or(""),
        sel.binding_id.as_deref().unwrap_or(""),
        sel.catalog_id.as_deref().unwrap_or(""),
        sel.selected_version.as_deref().unwrap_or(""),
        sel.execution_id.as_deref().unwrap_or(""),
    );
    sha256_hex(canonical.as_bytes())
}

/// Helix does not cache verification results. If a cache is added, it MUST use
/// this key so Target A cannot satisfy Target B.
pub fn verification_cache_key(identity: &TargetIdentity, sel: &StandardSelection) -> String {
    target_execution_id(identity, sel)
}

/// Did the target violate the tested condition, or did Helix fail to test?
/// Maps existing status/diagnostic/kind. Not a parallel score taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureAttribution {
    SpecFailure,
    TargetFailure,
    TransportFailure,
    TargetConfigurationFailure,
    HelixExecutionFailure,
    UnsupportedTest,
    Unknown,
}

impl FailureAttribution {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpecFailure => "spec_failure",
            Self::TargetFailure => "target_failure",
            Self::TransportFailure => "transport_failure",
            Self::TargetConfigurationFailure => "target_configuration_failure",
            Self::HelixExecutionFailure => "helix_execution_failure",
            Self::UnsupportedTest => "unsupported_test",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_result(result: &VerificationResult) -> Option<Self> {
        match result.status {
            VerificationStatus::Pass => None,
            VerificationStatus::Skip => Some(Self::UnsupportedTest),
            VerificationStatus::Error => Some(Self::HelixExecutionFailure),
            VerificationStatus::Fail => Some(fail_attribution(result)),
        }
    }
}

fn fail_attribution(result: &VerificationResult) -> FailureAttribution {
    let msg = result.message.as_deref().unwrap_or("");
    if msg.contains("HelixTest adapter error")
        || msg.contains("SpecSource identity mismatch")
        || msg.contains("wall clock")
    {
        return FailureAttribution::HelixExecutionFailure;
    }
    if msg.contains("not TESTABLE") || msg.contains("not detected") {
        return FailureAttribution::TargetConfigurationFailure;
    }
    if let Some(d) = &result.diagnostic {
        if d.likely_category == DiagnosticCategory::Reachability {
            return FailureAttribution::TransportFailure;
        }
    }
    let normative = result
        .traceability
        .as_ref()
        .map(|t| t.check_kind == BindingKind::Normative || t.category == BindingKind::Normative)
        .unwrap_or(false);
    if normative {
        FailureAttribution::SpecFailure
    } else if result.diagnostic.is_some() {
        FailureAttribution::TargetFailure
    } else {
        FailureAttribution::Unknown
    }
}

pub fn attach_attribution(result: &mut VerificationResult) {
    result.attribution = FailureAttribution::from_result(result);
}

/// Two runs against the same pack: technical facts, not a ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetComparison {
    pub standard: Option<String>,
    pub version: Option<String>,
    pub pack_id: Option<String>,
    pub pack_integrity_sha256: Option<String>,
    pub schema_document_sha256: Option<String>,
    pub checker_id: Option<String>,
    pub binding_id: Option<String>,
    pub catalog_id: Option<String>,
    pub same_pack: bool,
    pub independent_implementation_evidence: bool,
    pub targets: Vec<TargetComparisonRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetComparisonRow {
    pub target_id: String,
    pub target_kind: String,
    pub implementation_name: Option<String>,
    pub implementation_version: Option<String>,
    pub endpoint: String,
    pub spec_execution_id: Option<String>,
    pub target_execution_id: Option<String>,
    pub selected_version: Option<String>,
    pub verified_version: Option<String>,
    pub overall: String,
}

pub fn compare_target_runs(
    a: &crate::model::VerificationRun,
    b: &crate::model::VerificationRun,
) -> TargetComparison {
    let sa = a.standard_selection.as_ref();
    let sb = b.standard_selection.as_ref();
    let pack_id = sa.and_then(|s| s.standards_registry_entry.clone());
    let same_pack = sa.is_some()
        && sb.is_some()
        && sa.and_then(|s| s.standards_registry_entry.as_ref())
            == sb.and_then(|s| s.standards_registry_entry.as_ref())
        && sa.and_then(|s| s.pack_integrity_sha256.as_ref())
            == sb.and_then(|s| s.pack_integrity_sha256.as_ref())
        && sa.and_then(|s| s.schema_document_sha256.as_ref())
            == sb.and_then(|s| s.schema_document_sha256.as_ref())
        && sa.and_then(|s| s.checker_id.as_ref()) == sb.and_then(|s| s.checker_id.as_ref())
        && sa.and_then(|s| s.binding_id.as_ref()) == sb.and_then(|s| s.binding_id.as_ref())
        && sa.and_then(|s| s.catalog_id.as_ref()) == sb.and_then(|s| s.catalog_id.as_ref());
    let ka = kind_of(a);
    let kb = kind_of(b);
    let usable =
        |k: TargetKind| !k.is_mock_or_fixture_or_synthetic() && k != TargetKind::Unspecified;
    let independent_implementation_evidence = usable(ka)
        && usable(kb)
        && (ka.qualifies_as_independent_implementation()
            || kb.qualifies_as_independent_implementation());
    TargetComparison {
        standard: sa.and_then(|s| s.standard.clone()),
        version: sa.and_then(|s| s.selected_version.clone()),
        pack_id,
        pack_integrity_sha256: sa.and_then(|s| s.pack_integrity_sha256.clone()),
        schema_document_sha256: sa.and_then(|s| s.schema_document_sha256.clone()),
        checker_id: sa.and_then(|s| s.checker_id.clone()),
        binding_id: sa.and_then(|s| s.binding_id.clone()),
        catalog_id: sa.and_then(|s| s.catalog_id.clone()),
        same_pack,
        independent_implementation_evidence,
        targets: vec![row(a), row(b)],
    }
}

fn kind_of(run: &crate::model::VerificationRun) -> TargetKind {
    run.target
        .identity
        .as_ref()
        .map(|i| i.target_kind)
        .unwrap_or(TargetKind::Unspecified)
}

fn row(run: &crate::model::VerificationRun) -> TargetComparisonRow {
    let id = run.target.identity.as_ref();
    let sel = run.standard_selection.as_ref();
    let overall = if run
        .executed
        .iter()
        .any(|r| r.status == VerificationStatus::Fail || r.status == VerificationStatus::Error)
    {
        "FAIL"
    } else if run
        .executed
        .iter()
        .any(|r| r.status == VerificationStatus::Pass)
    {
        "PASS"
    } else {
        "NOT_VERIFIED"
    };
    TargetComparisonRow {
        target_id: id
            .map(|i| i.target_id.clone())
            .unwrap_or_else(|| format!("endpoint:{}", run.target.url)),
        target_kind: id
            .map(|i| i.target_kind.as_str())
            .unwrap_or("unspecified")
            .into(),
        implementation_name: id.and_then(|i| i.implementation_name.clone()),
        implementation_version: id.and_then(|i| i.implementation_version.clone()),
        endpoint: id
            .map(|i| i.endpoint.clone())
            .unwrap_or_else(|| run.target.url.clone()),
        spec_execution_id: sel.and_then(|s| s.execution_id.clone()),
        target_execution_id: sel.and_then(|s| s.target_execution_id.clone()),
        selected_version: sel.and_then(|s| s.selected_version.clone()),
        verified_version: sel.and_then(|s| s.verified_version.clone()),
        overall: overall.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Target;

    #[test]
    fn mock_is_not_independent_implementation() {
        assert!(!TargetKind::Mock.qualifies_as_independent_implementation());
        assert!(!TargetKind::Fixture.qualifies_as_independent_implementation());
        assert!(!TargetKind::SyntheticTarget.qualifies_as_independent_implementation());
        assert!(!TargetKind::ReferenceImplementation.qualifies_as_independent_implementation());
        assert!(!TargetKind::Unspecified.qualifies_as_independent_implementation());
        assert!(TargetKind::RealExternalImplementation.qualifies_as_independent_implementation());
        assert!(TargetKind::RealIndependentLocalImplementation
            .qualifies_as_independent_implementation());
    }

    #[test]
    fn never_infers_implementation_version_from_endpoint() {
        let id = TargetIdentity::unspecified("http://ferrum.example:8080/ga4gh/drs/v1");
        assert!(id.implementation_version.is_none());
        assert!(id.declared.implementation_version.is_none());
        assert_eq!(id.target_kind, TargetKind::Unspecified);
        assert!(id.target_id.starts_with("endpoint:"));
    }

    #[test]
    fn declared_metadata_stays_declared() {
        let declared = DeclaredTarget {
            target_id: Some("lab-a".into()),
            kind: TargetKind::ReferenceImplementation,
            implementation_name: Some("Ferrum".into()),
            implementation_version: Some("0.3.2".into()),
            standard_version: Some("1.4.0".into()),
        };
        let id = TargetIdentity::from_declared("http://127.0.0.1:9", &declared);
        assert_eq!(id.implementation_version.as_deref(), Some("0.3.2"));
        assert_eq!(id.declared.standard_version.as_deref(), Some("1.4.0"));
        assert_eq!(id.verified.target_id, "lab-a");
        assert_eq!(id.verified.target_kind, TargetKind::ReferenceImplementation);
    }

    #[test]
    fn target_id_change_changes_execution_and_cache_key() {
        let mut sel = StandardSelection::unversioned();
        sel.standards_registry_entry = Some("ga4gh.drs.1.4.0".into());
        sel.pack_integrity_sha256 = Some("c".repeat(64));
        sel.schema_document_sha256 = Some("d".repeat(64));
        sel.checker_id = Some("v0.1.3:abc".into());
        sel.binding_id = Some("b".repeat(64));
        sel.catalog_id = Some("a".repeat(64));
        sel.selected_version = Some("1.4.0".into());
        sel.execution_id = Some("e".repeat(64));
        let a = TargetIdentity::from_declared(
            "http://127.0.0.1:1",
            &DeclaredTarget {
                target_id: Some("A".into()),
                kind: TargetKind::Mock,
                ..DeclaredTarget::default()
            },
        );
        let b = TargetIdentity::from_declared(
            "http://127.0.0.1:1",
            &DeclaredTarget {
                target_id: Some("B".into()),
                kind: TargetKind::Mock,
                ..DeclaredTarget::default()
            },
        );
        let ea = target_execution_id(&a, &sel);
        let eb = target_execution_id(&b, &sel);
        assert_ne!(ea, eb);
        assert_eq!(ea.len(), 64);
        assert_eq!(verification_cache_key(&a, &sel), ea);
        assert_ne!(
            verification_cache_key(&a, &sel),
            verification_cache_key(&b, &sel)
        );
    }

    #[test]
    fn http_drs_target_exposes_only_identity_and_url() {
        let id = TargetIdentity::unspecified("http://127.0.0.1:9");
        let t = HttpDrsTarget::new(id.clone());
        assert_eq!(t.base_url(), "http://127.0.0.1:9");
        assert_eq!(t.identity().target_id, id.target_id);
        let _ = Target::from_identity(id);
    }

    #[test]
    fn two_mocks_are_not_independent_evidence() {
        let mut a = crate::model::VerificationRun::new(Target::from_identity(
            TargetIdentity::from_declared(
                "http://127.0.0.1:1",
                &DeclaredTarget {
                    target_id: Some("mock-a".into()),
                    kind: TargetKind::Mock,
                    ..DeclaredTarget::default()
                },
            ),
        ));
        let b = crate::model::VerificationRun::new(Target::from_identity(
            TargetIdentity::from_declared(
                "http://127.0.0.1:2",
                &DeclaredTarget {
                    target_id: Some("mock-b".into()),
                    kind: TargetKind::Mock,
                    ..DeclaredTarget::default()
                },
            ),
        ));
        a.standard_selection = Some(StandardSelection::unversioned());
        let cmp = compare_target_runs(&a, &b);
        assert!(!cmp.independent_implementation_evidence);
    }
}
