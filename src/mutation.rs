// SPDX-License-Identifier: Apache-2.0
//! Adversarial mutation corpus: known-bad targets with one defect each.
//!
//! HelixTest already runs DRS/WES/security checks. This catalog productizes
//! which defects those checks catch, and which they do not. A miss is recorded,
//! not hidden. Not GA4GH certification. Not HELIOS. Not a pentest suite.
//!
//! See docs/MUTATION.md.

use crate::diagnostics::DiagnosticCategory;

/// Defect class requested of the corpus. One mutant introduces one class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefectClass {
    MissingRequiredProperty,
    InvalidPropertyType,
    WrongHttpStatus,
    WrongContentType,
    InvalidJson,
    MalformedIdentifier,
    IncorrectPagination,
    IncorrectErrorResponse,
    IncorrectAsyncState,
    IncorrectVersionDeclaration,
    VersionMismatch,
    UnauthorizedAccessAllowed,
    AuthorizedAccessDenied,
    IncorrectRangeSemantics,
    MissingRequiredHeader,
    UnexpectedResponseField,
    TimeoutBehavior,
    MalformedServiceInfo,
    ContradictoryVersionInformation,
}

impl DefectClass {
    pub const ALL: &'static [DefectClass] = &[
        Self::MissingRequiredProperty,
        Self::InvalidPropertyType,
        Self::WrongHttpStatus,
        Self::WrongContentType,
        Self::InvalidJson,
        Self::MalformedIdentifier,
        Self::IncorrectPagination,
        Self::IncorrectErrorResponse,
        Self::IncorrectAsyncState,
        Self::IncorrectVersionDeclaration,
        Self::VersionMismatch,
        Self::UnauthorizedAccessAllowed,
        Self::AuthorizedAccessDenied,
        Self::IncorrectRangeSemantics,
        Self::MissingRequiredHeader,
        Self::UnexpectedResponseField,
        Self::TimeoutBehavior,
        Self::MalformedServiceInfo,
        Self::ContradictoryVersionInformation,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingRequiredProperty => "missing_required_property",
            Self::InvalidPropertyType => "invalid_property_type",
            Self::WrongHttpStatus => "wrong_http_status",
            Self::WrongContentType => "wrong_content_type",
            Self::InvalidJson => "invalid_json",
            Self::MalformedIdentifier => "malformed_identifier",
            Self::IncorrectPagination => "incorrect_pagination",
            Self::IncorrectErrorResponse => "incorrect_error_response",
            Self::IncorrectAsyncState => "incorrect_asynchronous_state",
            Self::IncorrectVersionDeclaration => "incorrect_version_declaration",
            Self::VersionMismatch => "version_mismatch",
            Self::UnauthorizedAccessAllowed => "unauthorized_access_allowed",
            Self::AuthorizedAccessDenied => "authorized_access_denied",
            Self::IncorrectRangeSemantics => "incorrect_range_semantics",
            Self::MissingRequiredHeader => "missing_required_header",
            Self::UnexpectedResponseField => "unexpected_response_field",
            Self::TimeoutBehavior => "timeout_behavior",
            Self::MalformedServiceInfo => "malformed_service_info",
            Self::ContradictoryVersionInformation => "contradictory_version_information",
        }
    }
}

/// Which Helix command exercises this mutant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationSuite {
    /// `helix verify` / `helix::verify::verify`.
    Verify,
    /// `helix security` / `helix::security::run_security`. Dummy HMAC only.
    Security,
}

/// How Helix classifies the failure. Not a root cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticClass {
    Verify(DiagnosticCategory),
    /// Helix security rows have no `diagnostic` object today.
    Security,
    /// Overall is not PASS, but no executed check carries this defect's class.
    FailClosed,
    /// Documented miss: Helix does not fail the hypothesized check.
    None,
}

impl DiagnosticClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verify(c) => c.as_str(),
            Self::Security => "security",
            Self::FailClosed => "fail_closed",
            Self::None => "none",
        }
    }
}

/// Expected outcome of a known-bad target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationDetection {
    Detected,
    Missed,
}

/// One controlled defect. Unique `id`. Not a GA4GH MUST list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mutant {
    pub id: &'static str,
    pub class: DefectClass,
    pub description: &'static str,
    pub suite: MutationSuite,
    pub detection: MutationDetection,
    /// Check that must FAIL when `detection == Detected`. None for fail-closed-only.
    pub expected_check_id: Option<&'static str>,
    pub diagnostic: DiagnosticClass,
    /// Substring of the failing check's message when Detected (fixture / HelixTest text).
    pub expected_message_substr: Option<&'static str>,
    /// Required when `detection == Missed`. Why Helix does not catch this defect.
    pub miss_reason: Option<&'static str>,
}

impl Mutant {
    pub fn is_detected(self) -> bool {
        self.detection == MutationDetection::Detected
    }
}

/// Corpus. Every shipped mutant is listed. Misses stay in the table.
pub const CATALOG: &[Mutant] = &[
    Mutant {
        id: "HLX-MUT-001",
        class: DefectClass::MissingRequiredProperty,
        description: "DrsObject omits required self_uri; remaining fields and bytes are honest",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: Some("self_uri"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-002",
        class: DefectClass::InvalidPropertyType,
        description: "DrsObject.size is a JSON string instead of integer",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: Some("size"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-003",
        class: DefectClass::WrongHttpStatus,
        description: "GET /objects/test-object-1 returns 403 (DETECTED) instead of 2xx",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.reachable"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Reachability),
        expected_message_substr: Some("403"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-004",
        class: DefectClass::WrongHttpStatus,
        description: "GET /objects/test-object-1 returns 500",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: Some("drs.object.reachable"),
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "Discovery treats non-2xx/401/403 as NOT_DETECTED. A 500 object probe never executes drs.object.reachable, so Helix cannot classify wrong HTTP status. Overall is not PASS (skip-only). Fail-closed is not a classified HTTP-semantics fail.",
        ),
    },
    Mutant {
        id: "HLX-MUT-005",
        class: DefectClass::WrongContentType,
        description: "GET object returns text/html whose body is not JSON",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: None,
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-006",
        class: DefectClass::WrongContentType,
        description: "GET object returns schema-valid JSON with Content-Type: text/plain",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "HelixTest get_json parses the body and ignores Content-Type. Helix has no content-negotiation check. Adding one would be a new Helix-owned assertion, not a loaded GA4GH MUST. Not implemented.",
        ),
    },
    Mutant {
        id: "HLX-MUT-007",
        class: DefectClass::InvalidJson,
        description: "GET object returns truncated JSON (200)",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: None,
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-008",
        class: DefectClass::MalformedIdentifier,
        description: "DrsObject.id is not the fixture id test-object-1",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: Some("id mismatch"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-009",
        class: DefectClass::IncorrectPagination,
        description: "GET /objects (bulk list) returns a non-array objects field; object GET is honest",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: None,
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "Helix verify never calls DRS bulk GET /objects. Pagination is uncovered (docs/BEHAVIOR.md). The list defect cannot be observed.",
        ),
    },
    Mutant {
        id: "HLX-MUT-010",
        class: DefectClass::IncorrectErrorResponse,
        description: "Unknown object id returns HTTP 200 with a DrsObject body",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.not_found"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::ErrorHandling),
        expected_message_substr: Some("404"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-011",
        class: DefectClass::IncorrectErrorResponse,
        description: "Unknown object id returns HTTP 500 instead of 404",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.not_found"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::ErrorHandling),
        expected_message_substr: Some("500"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-012",
        class: DefectClass::IncorrectAsyncState,
        description: "WES echo run's first GetRunStatus is COMPLETE (no QUEUED/INITIALIZING/RUNNING)",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("wes.run.lifecycle_success"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Lifecycle),
        expected_message_substr: Some("terminal state"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-013",
        class: DefectClass::IncorrectAsyncState,
        description: "WES fail workflow ends COMPLETE instead of EXECUTOR_ERROR/SYSTEM_ERROR",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("wes.run.failure_state"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Lifecycle),
        expected_message_substr: Some("COMPLETE"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-014",
        class: DefectClass::IncorrectVersionDeclaration,
        description: "WES supported_wes_versions is [\"2.0\"] only (no 1.0 or 1.1)",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("wes.service_info.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: Some("supported_wes_versions"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-015",
        class: DefectClass::VersionMismatch,
        description: "WES type.version is 9.9.9 while supported_wes_versions still contains 1.1",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: Some("wes.service_info.schema"),
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "HelixTest schema accepts the body if supported_wes_versions contains 1.0 or 1.1. Default unversioned verify does not fail because type.version disagrees. Helix does not invent a version-mismatch MUST. Mode 1/2 still fail-closed on AVAILABLE-only packs, which is a different defect.",
        ),
    },
    Mutant {
        id: "HLX-MUT-016",
        class: DefectClass::UnauthorizedAccessAllowed,
        description: "Auth gate ignores JWT exp (expired Bearer returns 200)",
        suite: MutationSuite::Security,
        detection: MutationDetection::Detected,
        expected_check_id: Some("auth.helix.token.expired"),
        diagnostic: DiagnosticClass::Security,
        expected_message_substr: Some("HLX-AUTH-011"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-017",
        class: DefectClass::AuthorizedAccessDenied,
        description: "Auth gate reject_all (valid Bearer returns 401)",
        suite: MutationSuite::Security,
        detection: MutationDetection::Detected,
        expected_check_id: Some("auth.helix.token.valid"),
        diagnostic: DiagnosticClass::Security,
        expected_message_substr: Some("HLX-AUTH-010"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-018",
        class: DefectClass::IncorrectRangeSemantics,
        description: "Bytes endpoint ignores Range and returns HTTP 200 with the full object",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.range"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Range),
        expected_message_substr: Some("206"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-019",
        class: DefectClass::MissingRequiredHeader,
        description: "Bytes Range response is 206 without Content-Range",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("drs.object.range"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Range),
        expected_message_substr: Some("Content-Range"),
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-020",
        class: DefectClass::UnexpectedResponseField,
        description: "DrsObject includes extra property unexpected_helix_mutant",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: Some("drs.object.schema"),
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "HelixTest-vendored DrsObject schema does not set additionalProperties: false. Extra fields are not a fixture fail. Helix does not add a Helix-owned additionalProperties MUST.",
        ),
    },
    Mutant {
        id: "HLX-MUT-021",
        class: DefectClass::TimeoutBehavior,
        description: "DRS object probes delay longer than the Helix-owned request timeout",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: None,
        diagnostic: DiagnosticClass::FailClosed,
        expected_message_substr: None,
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-022",
        class: DefectClass::MalformedServiceInfo,
        description: "WES service-info is 200 JSON with wrong types for id/name/type",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Detected,
        expected_check_id: Some("wes.service_info.schema"),
        diagnostic: DiagnosticClass::Verify(DiagnosticCategory::Schema),
        expected_message_substr: None,
        miss_reason: None,
    },
    Mutant {
        id: "HLX-MUT-023",
        class: DefectClass::ContradictoryVersionInformation,
        description: "WES type.version is 1.0.0 while supported_wes_versions is only [\"1.1\"]",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: Some("wes.service_info.schema"),
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "Both values can satisfy HelixTest's 1.0-or-1.1 list check independently. Unversioned verify does not fail on declared-field disagreement. Helix does not add a contradiction MUST.",
        ),
    },
    Mutant {
        id: "HLX-MUT-024",
        class: DefectClass::IncorrectPagination,
        description: "WES GET /runs list returns runs as a string; submit/status/get are honest",
        suite: MutationSuite::Verify,
        detection: MutationDetection::Missed,
        expected_check_id: None,
        diagnostic: DiagnosticClass::None,
        expected_message_substr: None,
        miss_reason: Some(
            "Helix verify never calls WES ListRuns. Pagination text exists in the vendor file; HelixTest does not load those bytes for a paging check. Uncovered.",
        ),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationSummary {
    pub attempted: usize,
    pub detected: usize,
    pub missed: usize,
}

pub fn summary() -> MutationSummary {
    let mut detected = 0;
    let mut missed = 0;
    for m in CATALOG {
        match m.detection {
            MutationDetection::Detected => detected += 1,
            MutationDetection::Missed => missed += 1,
        }
    }
    MutationSummary {
        attempted: detected + missed,
        detected,
        missed,
    }
}

pub fn by_id(id: &str) -> Option<&'static Mutant> {
    CATALOG.iter().find(|m| m.id == id)
}

pub fn validate_catalog() -> anyhow::Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for m in CATALOG {
        if !m.id.starts_with("HLX-MUT-") {
            anyhow::bail!("{} id must start with HLX-MUT-", m.id);
        }
        if !seen.insert(m.id) {
            anyhow::bail!("duplicate mutation id {}", m.id);
        }
        match m.detection {
            MutationDetection::Missed => {
                if m.miss_reason.is_none() {
                    anyhow::bail!("{} is missed but has no miss_reason", m.id);
                }
                if m.diagnostic != DiagnosticClass::None {
                    anyhow::bail!("{} missed mutants use diagnostic none", m.id);
                }
            }
            MutationDetection::Detected => {
                if m.miss_reason.is_some() {
                    anyhow::bail!("{} is detected but has miss_reason", m.id);
                }
                if m.expected_check_id.is_none() && m.diagnostic != DiagnosticClass::FailClosed {
                    anyhow::bail!(
                        "{} detected mutant needs expected_check_id or fail_closed",
                        m.id
                    );
                }
            }
        }
        if let Some(id) = m.expected_check_id {
            if crate::identity::spec_by_id(id).is_none() {
                anyhow::bail!("{} expected_check_id {} is not in SPECS", m.id, id);
            }
        }
    }
    for class in DefectClass::ALL {
        if !CATALOG.iter().any(|m| m.class == *class) {
            anyhow::bail!("defect class {} has no mutant", class.as_str());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_well_formed() {
        validate_catalog().expect("mutation catalog");
        let s = summary();
        assert_eq!(s.attempted, s.detected + s.missed);
        assert_eq!(s.attempted, CATALOG.len());
    }

    #[test]
    fn summary_has_no_percentage_fields() {
        let s = summary();
        let json = serde_json::json!({
            "attempted": s.attempted,
            "detected": s.detected,
            "missed": s.missed,
        });
        assert!(json.get("percent").is_none());
        assert!(json.get("score").is_none());
        assert!(json.get("compliant").is_none());
    }
}
