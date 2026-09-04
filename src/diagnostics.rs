// SPDX-License-Identifier: Apache-2.0
//! Deterministic diagnostics for Helix verification failures.
//!
//! Not an AI diagnosis system. Not a root-cause claim. Not HELIOS.
//! Helix only reports what the check asserted and what it could parse
//! from the failure text. Use **possible causes**, never **cause**.

use serde::{Deserialize, Serialize};

use crate::model::{VerificationResult, VerificationStatus};

/// Likely failure class. Not a root cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Reachability,
    Schema,
    Checksum,
    Range,
    ErrorHandling,
    Lifecycle,
    Undetermined,
}

impl DiagnosticCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reachability => "reachability",
            Self::Schema => "schema",
            Self::Checksum => "checksum",
            Self::Range => "range",
            Self::ErrorHandling => "error_handling",
            Self::Lifecycle => "lifecycle",
            Self::Undetermined => "undetermined",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureDiagnostic {
    pub code: String,
    pub id: String,
    pub expected: String,
    pub observed: String,
    pub likely_category: DiagnosticCategory,
    pub hint: String,
    /// Possibilities only. Never a single determined cause.
    pub possible_causes: Vec<String>,
}

const NOT_DETERMINED: &str =
    "Not determined. Helix cannot extract a more specific observation from the failure text.";

struct Spec {
    id: &'static str,
    expected: &'static str,
    category: DiagnosticCategory,
    hint: &'static str,
    possible_causes: &'static [&'static str],
}

const SPECS: &[Spec] = &[
    Spec {
        id: "drs.object.reachable",
        expected: "HTTP 2xx or 401 on GET /objects/test-object-1",
        category: DiagnosticCategory::Reachability,
        hint: "The DRS object endpoint did not look reachable to HelixTest. Helix cannot name the network hop that failed.",
        possible_causes: &[
            "The DRS base URL is wrong or the object path differs.",
            "The process is down, firewalled, or returning 4xx/5xx other than 401.",
            "TLS or DNS failed before an HTTP status was produced.",
        ],
    },
    Spec {
        id: "drs.object.schema",
        expected: "DrsObject JSON that validates against the GA4GH schema and includes id, self_uri, name, and a non-empty access_methods array",
        category: DiagnosticCategory::Schema,
        hint: "The object body did not match the documented DrsObject shape. Helix does not infer which field the implementer intended.",
        possible_causes: &[
            "A required field is missing or mistyped.",
            "access_methods is missing or empty.",
            "The response is not a DrsObject (HTML error page, envelope wrapper).",
        ],
    },
    Spec {
        id: "drs.object.checksum",
        expected: "sha256 in DrsObject.checksums matches a download of access_methods[0].access_url.url for test-object-1",
        category: DiagnosticCategory::Checksum,
        hint: "The bytes HelixTest downloaded did not match the advertised sha256, or the download/checksum fields were missing. Helix does not know whether the metadata or the bytes are wrong.",
        possible_causes: &[
            "checksums has no sha256 entry, or the digest is stale.",
            "access_url points at different bytes than the object metadata describes.",
            "The download returned a non-success status so the digest was never compared.",
        ],
    },
    Spec {
        id: "drs.object.range",
        expected: "HTTP 206 Partial Content with a valid Content-Range for Range: bytes=0-1023 and a non-empty body",
        category: DiagnosticCategory::Range,
        hint: "The bytes URL did not satisfy the HelixTest Range contract. Helix cannot tell whether the server ignored Range or a proxy stripped it.",
        possible_causes: &[
            "The server returns 200 with the full object instead of 206.",
            "Content-Range is missing or does not cover bytes=0-1023.",
            "access_url is not a Range-capable bytes endpoint.",
        ],
    },
    Spec {
        id: "drs.object.not_found",
        expected: "HTTP 404 for an unknown object ID",
        category: DiagnosticCategory::ErrorHandling,
        hint: "The target did not return 404 for a documented unknown DRS id. Helix does not know why the lookup succeeded or used another status.",
        possible_causes: &[
            "Unknown ids are treated as existing objects.",
            "A catch-all handler returns 200 or another success status.",
            "Auth or a gateway maps missing objects to 401/403/500 instead of 404.",
        ],
    },
    Spec {
        id: "wes.service_info.reachable",
        expected: "HTTP 2xx or 401 on GET WES /service-info",
        category: DiagnosticCategory::Reachability,
        hint: "WES service-info did not look reachable to HelixTest. Helix cannot name the network hop that failed.",
        possible_causes: &[
            "The WES base URL is wrong.",
            "The process is down or returning an unexpected status.",
            "TLS or DNS failed before an HTTP status was produced.",
        ],
    },
    Spec {
        id: "wes.service_info.schema",
        expected: "WES ServiceInfo JSON that validates, with supported_wes_versions containing 1.0 or 1.1",
        category: DiagnosticCategory::Schema,
        hint: "service-info did not match the HelixTest WES 1.1.0 schema checks. Helix does not guess which missing field the implementer cares about.",
        possible_causes: &[
            "A required Service or WES field is missing.",
            "supported_wes_versions omits 1.0 and 1.1.",
            "The body is not JSON ServiceInfo.",
        ],
    },
    Spec {
        id: "wes.run.lifecycle_success",
        expected: "Echo workflow (trs://test-tool/echo/1.0) reaches COMPLETE, with a pre-terminal state in history and outputs.echo_out == hello-ga4gh",
        category: DiagnosticCategory::Lifecycle,
        hint: "The success-path echo run did not meet the HelixTest lifecycle contract. Helix does not know whether submit, poll, or outputs failed.",
        possible_causes: &[
            "The run never reached COMPLETE.",
            "History has no QUEUED, INITIALIZING, or RUNNING before COMPLETE.",
            "outputs.echo_out is missing or not hello-ga4gh.",
        ],
    },
    Spec {
        id: "wes.run.failure_state",
        expected: "Bad workflow (trs://test-tool/fail/1.0) ends in EXECUTOR_ERROR or SYSTEM_ERROR",
        category: DiagnosticCategory::Lifecycle,
        hint: "A workflow that should fail did not report an error terminal state. Helix does not know whether the engine ran it as success or never executed it.",
        possible_causes: &[
            "The run completed successfully instead of failing.",
            "The engine maps executor failure to COMPLETE or CANCELED.",
            "Submit/poll never reached a terminal error state.",
        ],
    },
    Spec {
        id: "wes.run.missing_inputs",
        expected: "cwl-echo with empty params ends in EXECUTOR_ERROR or SYSTEM_ERROR",
        category: DiagnosticCategory::Lifecycle,
        hint: "A missing-input run did not report an error terminal state. Helix cannot see how the engine treated empty params.",
        possible_causes: &[
            "Empty params are accepted and the run completes.",
            "The engine uses a different error state name than EXECUTOR_ERROR / SYSTEM_ERROR.",
            "The posted workflow_url is not the HelixTest missing-input fixture.",
        ],
    },
    Spec {
        id: "wes.run.incompatible_type",
        expected: "cwl-echo posted as WDL 1.0 ends in EXECUTOR_ERROR or SYSTEM_ERROR",
        category: DiagnosticCategory::Lifecycle,
        hint: "An incompatible workflow_type did not report an error terminal state. Helix cannot see whether the type was ignored or remapped.",
        possible_causes: &[
            "WDL is accepted for a CWL tool.",
            "Type checks happen later than run status polling.",
            "The error state is named something other than EXECUTOR_ERROR / SYSTEM_ERROR.",
        ],
    },
    Spec {
        id: "wes.run.invalid_workflow",
        expected: "trs://nonexistent/invalid/0.0 ends in EXECUTOR_ERROR or SYSTEM_ERROR",
        category: DiagnosticCategory::Lifecycle,
        hint: "An invalid workflow URL did not report an error terminal state. Helix cannot see whether the engine resolved a different tool.",
        possible_causes: &[
            "Unknown TRS URLs are treated as success.",
            "Submit fails before a run exists (transport), which HelixTest also records as this check failing.",
            "The error state is named something other than EXECUTOR_ERROR / SYSTEM_ERROR.",
        ],
    },
    Spec {
        id: "wes.run.scatter_gather",
        expected: "trs://test-tool/scatter-gather/1.0 reaches COMPLETE with outputs.scatter_result present",
        category: DiagnosticCategory::Lifecycle,
        hint: "The scatter/gather fixture did not meet the HelixTest contract. On profile generic this check is skipped, not failed. Helix cannot name the engine defect.",
        possible_causes: &[
            "The run did not reach COMPLETE.",
            "outputs.scatter_result is missing.",
            "The profile enabled scatter but the target does not implement that fixture.",
        ],
    },
];

fn spec(id: &str) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.id == id)
}

/// Attach a diagnostic on fail/error when the id is a catalogued DRS/WES check.
/// Pass/skip stay without a diagnostic. Unknown ids get none (not a guessed story).
pub fn attach(result: &mut VerificationResult) {
    match result.status {
        VerificationStatus::Fail | VerificationStatus::Error => {
            result.diagnostic = diagnose(&result.id, &result.code, result.message.as_deref());
        }
        VerificationStatus::Pass | VerificationStatus::Skip => {
            result.diagnostic = None;
        }
    }
}

pub fn diagnose(id: &str, code: &str, message: Option<&str>) -> Option<FailureDiagnostic> {
    let spec = spec(id)?;
    let msg = message.unwrap_or("");
    let observed = observed_for(id, msg);
    let hint = hint_for(spec, id, &observed);
    Some(FailureDiagnostic {
        code: code.to_string(),
        id: id.to_string(),
        expected: spec.expected.to_string(),
        observed,
        likely_category: spec.category,
        hint,
        possible_causes: spec
            .possible_causes
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    })
}

fn observed_for(id: &str, message: &str) -> String {
    if message.is_empty() {
        return NOT_DETERMINED.to_string();
    }
    if message.contains("target unreachable") {
        return "Target was not reachable; these checks were not executed.".to_string();
    }
    if message.contains("not detected") {
        return "The service was not detected under the target URL.".to_string();
    }
    if message.contains("not TESTABLE") {
        return "The service was detected but is not TESTABLE for helix verify.".to_string();
    }
    if let Some(status) = extract_http_status(message) {
        return format!("HTTP {status}");
    }
    if id.starts_with("wes.") {
        if let Some(state) = extract_wes_state(message) {
            return format!("WES state {state}");
        }
    }
    if id == "drs.object.checksum" {
        if let Some((exp, got)) = extract_checksum_pair(message) {
            return format!("sha256 digest {got} (object advertised {exp})");
        }
    }
    crate::redact::redact_text(&format!("{NOT_DETERMINED} Check output: {message}"))
}

fn hint_for(spec: &Spec, id: &str, observed: &str) -> String {
    if id == "drs.object.not_found" && observed.starts_with("HTTP 2") {
        return "The target appears to return a successful response for an unknown DRS object. Verify object lookup error handling.".to_string();
    }
    if observed.starts_with("Not determined") {
        return format!("{} Helix is not claiming a root cause.", spec.hint);
    }
    spec.hint.to_string()
}

fn extract_http_status(message: &str) -> Option<u16> {
    // HelixTest: "Expected 404 for invalid DRS id, got 200 OK"
    //           "Expected 206 Partial Content for range request, got 200 OK"
    //           "Unexpected HTTP status: 503 Service Unavailable"
    // Adapter tests: "expected 404, got 200"
    let lower = message.to_ascii_lowercase();
    let key = lower
        .rfind("got ")
        .map(|i| i + 4)
        .or_else(|| lower.rfind("status: ").map(|i| i + 8))
        .or_else(|| lower.rfind("status ").map(|i| i + 7))?;
    let tail = message.get(key..)?;
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 3 {
        digits.parse().ok()
    } else {
        None
    }
}

fn extract_wes_state(message: &str) -> Option<String> {
    let idx = message.find("got ")?;
    let tail = message.get(idx + 4..)?;
    let token: String = tail
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || *c == '_')
        .collect();
    if token.is_empty() || token.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some(token)
    }
}

fn extract_checksum_pair(message: &str) -> Option<(String, String)> {
    let rest = message.split("expected ").nth(1)?;
    let (exp, after) = rest.split_once(", got ")?;
    let got = after.split_whitespace().next().unwrap_or(after);
    if exp.len() >= 8 && got.len() >= 8 {
        Some((exp.trim().to_string(), got.trim().to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{spec as catalog_spec, DRS_VERIFY_IDS, WES_VERIFY_IDS};
    use crate::model::{VerificationCheck, VerificationResult, VerificationStatus};

    #[test]
    fn catalog_covers_wired_drs_and_wes() {
        for id in DRS_VERIFY_IDS.iter().chain(WES_VERIFY_IDS.iter()) {
            assert!(spec(id).is_some(), "missing diagnostic spec for {id}");
        }
        assert_eq!(SPECS.len(), DRS_VERIFY_IDS.len() + WES_VERIFY_IDS.len());
    }

    #[test]
    fn drs_005_successful_unknown_object_matches_documented_example() {
        let d = diagnose(
            "drs.object.not_found",
            "HLX-DRS-005",
            Some("Expected 404 for invalid DRS id, got 200 OK"),
        )
        .unwrap();
        assert_eq!(d.code, "HLX-DRS-005");
        assert_eq!(d.id, "drs.object.not_found");
        assert_eq!(d.expected, "HTTP 404 for an unknown object ID");
        assert_eq!(d.observed, "HTTP 200");
        assert_eq!(d.likely_category, DiagnosticCategory::ErrorHandling);
        assert_eq!(
            d.hint,
            "The target appears to return a successful response for an unknown DRS object. Verify object lookup error handling."
        );
        assert!(!d.possible_causes.is_empty());
        assert!(!d.hint.to_lowercase().contains("root cause"));
        assert!(d.possible_causes.iter().all(|c| !c.starts_with("Cause")));
    }

    #[test]
    fn drs_005_compact_got_200_from_adapter_tests() {
        let d = diagnose(
            "drs.object.not_found",
            "HLX-DRS-005",
            Some("expected 404, got 200"),
        )
        .unwrap();
        assert_eq!(d.observed, "HTTP 200");
    }

    #[test]
    fn unparsed_failure_text_is_not_invented_as_http_200() {
        let d = diagnose(
            "drs.object.not_found",
            "HLX-DRS-005",
            Some("object lookup behaved unexpectedly"),
        )
        .unwrap();
        assert!(d.observed.starts_with("Not determined"));
        assert!(!d.observed.contains("HTTP 200"));
        assert!(d.hint.contains("Helix is not claiming a root cause"));
    }

    #[test]
    fn wes_failure_state_parses_complete_as_observed() {
        let d = diagnose(
            "wes.run.failure_state",
            "HLX-WES-004",
            Some("Expected error state, got COMPLETE"),
        )
        .unwrap();
        assert_eq!(d.observed, "WES state COMPLETE");
        assert_eq!(d.likely_category, DiagnosticCategory::Lifecycle);
        assert!(d.expected.contains("EXECUTOR_ERROR"));
    }

    #[test]
    fn range_parses_206_expected_got_200() {
        let d = diagnose(
            "drs.object.range",
            "HLX-DRS-004",
            Some("Expected 206 Partial Content for range request, got 200 OK"),
        )
        .unwrap();
        assert_eq!(d.observed, "HTTP 200");
        assert_eq!(d.likely_category, DiagnosticCategory::Range);
    }

    #[test]
    fn pass_has_no_diagnostic() {
        let mut r = VerificationResult::pass(VerificationCheck::from_spec(catalog_spec(
            "drs.object.not_found",
        )));
        attach(&mut r);
        assert_eq!(r.status, VerificationStatus::Pass);
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn fail_attaches_drs_005() {
        let r = VerificationResult::fail(
            VerificationCheck::from_spec(catalog_spec("drs.object.not_found")),
            "Expected 404 for invalid DRS id, got 200 OK",
        );
        let d = r.diagnostic.as_ref().expect("diagnostic");
        assert_eq!(d.observed, "HTTP 200");
        assert_eq!(d.code, r.code);
        assert_eq!(d.id, r.id);
    }

    #[test]
    fn tes_id_has_no_diagnostic_yet() {
        assert!(diagnose("tes.tasks.reachable", "HLX-TES-001", Some("boom")).is_none());
    }

    #[test]
    fn never_uses_cause_singular_heading() {
        for s in SPECS {
            assert!(!s.hint.contains("Cause:"));
            for c in s.possible_causes {
                assert!(!c.starts_with("Cause:"));
            }
        }
    }
}
