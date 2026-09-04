// SPDX-License-Identifier: Apache-2.0
//! Helix-native verification domain model.
//!
//! `helix verify` emits this shape (`VerificationRun`). `helix security` still
//! emits HelixTest `OverallReport`. HelixTest stays a separate git root (D1).
//!
//! Not HELIOS: no signatures, RO-Crate, evidence chains, audit trails, or PDF.
//! No compliance scoring / certification.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

pub use crate::identity::{CheckCategory, CheckSpec, Severity};

/// Helix crate version (`Cargo.toml`).
pub fn helix_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Published HelixTest pin this repo builds against ([VERSIONS.lock](../../VERSIONS.lock)).
/// Tag, not crate `0.1.0`. Update with the lockfile; do not invent a later tag.
pub const HELIXTEST_PIN: &str = "v0.1.3";

/// Git SHA for [`HELIXTEST_PIN`]. Same line as `HELIXTEST_SHA` in VERSIONS.lock.
pub const HELIXTEST_SHA: &str = "1832c043e1679ec283cb2113510ee33684317cce";

/// Frozen machine-readable document id for `helix verify --format json`.
/// File: `schemas/helix-verification-v1.json`. Not a HELIOS evidence schema.
pub const SCHEMA_VERSION: &str = "helix-verification-v1";

/// Documented DRS/WES fixture catalog ([docs/FIXTURES.md], [docs/RUN_IDENTITY.md]).
/// Bump when `test-object-1` bytes, the unknown-id string, or WES TRS fixture URLs change.
/// Not a HELIOS crate version. Not a signature.
pub const FIXTURE_VERSION: &str = "helix-fixtures-v1";

fn default_schema_version() -> String {
    SCHEMA_VERSION.to_string()
}

fn default_fixture_version() -> String {
    FIXTURE_VERSION.to_string()
}

/// Gateway-style origin Helix was pointed at. Not a Ferrum type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub url: String,
}

impl Target {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Stable machine-readable identity of a check.
///
/// `id` is dotted (`drs.object.not_found`). `code` is the Helix catalog code
/// (`HLX-DRS-005`). Both must stay stable across HelixTest pin bumps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckIdentity {
    pub id: String,
    pub code: String,
}

impl CheckIdentity {
    pub fn new(id: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
        }
    }

    pub fn from_spec(spec: &CheckSpec) -> Self {
        Self::new(spec.id, spec.code)
    }
}

/// Catalog lookups. Canonical specs live in [`crate::identity`].
pub mod catalog {
    use super::CheckIdentity;
    use crate::identity;

    pub fn drs_object_reachable() -> CheckIdentity {
        CheckIdentity::from_spec(identity::spec("drs.object.reachable"))
    }

    pub fn drs_object_schema() -> CheckIdentity {
        CheckIdentity::from_spec(identity::spec("drs.object.schema"))
    }

    pub fn drs_object_checksum() -> CheckIdentity {
        CheckIdentity::from_spec(identity::spec("drs.object.checksum"))
    }

    pub fn drs_object_range() -> CheckIdentity {
        CheckIdentity::from_spec(identity::spec("drs.object.range"))
    }

    pub fn drs_object_not_found() -> CheckIdentity {
        CheckIdentity::from_spec(identity::spec("drs.object.not_found"))
    }
}

/// Definition of a check (what would run). Distinct from a [`VerificationResult`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub id: String,
    pub code: String,
    pub name: String,
    /// Open string (`drs`, `wes`, `htsget`, later `beacon`). Not an enum.
    pub service: String,
    pub category: CheckCategory,
    /// Default severity if this check fails.
    pub severity: Severity,
    /// Optional profile (`generic`, `ga4gh-drs`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

impl VerificationCheck {
    pub fn from_spec(spec: &CheckSpec) -> Self {
        Self {
            id: spec.id.to_string(),
            code: spec.code.to_string(),
            name: spec.name.to_string(),
            service: spec.service.to_string(),
            category: spec.category,
            severity: spec.severity,
            profile: None,
        }
    }

    pub fn new(
        identity: CheckIdentity,
        name: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        Self {
            id: identity.id,
            code: identity.code,
            name: name.into(),
            service: service.into(),
            category: CheckCategory::Other,
            severity: Severity::Error,
            profile: None,
        }
    }

    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn identity(&self) -> CheckIdentity {
        CheckIdentity::new(self.id.clone(), self.code.clone())
    }
}

/// PASS / FAIL / SKIP / ERROR. Skip is never pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

impl VerificationStatus {
    pub fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }

    /// CLI / CI red: assertion failed or the runner could not execute the check.
    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Fail | Self::Error)
    }
}

/// Machine-readable why a check failed or errored. Not an evidence pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureCode {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl FailureCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Outcome of one check. Identity is always present (`id` + `code`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResult {
    pub id: String,
    pub code: String,
    pub name: String,
    /// Exact HelixTest `TestCaseResult.name` when this row was translated by the adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helixtest_name: Option<String>,
    pub service: String,
    pub category: CheckCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub status: VerificationStatus,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<FailureCode>,
    /// Deterministic DRS/WES failure diagnostic. Absent on pass/skip and on ids without a catalogued spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<crate::diagnostics::FailureDiagnostic>,
}

impl VerificationResult {
    pub fn from_check(check: VerificationCheck, status: VerificationStatus) -> Self {
        let (severity, failure) = match status {
            VerificationStatus::Pass | VerificationStatus::Skip => (Severity::Info, None),
            VerificationStatus::Fail => {
                (check.severity, Some(FailureCode::new(check.code.clone())))
            }
            VerificationStatus::Error => {
                (Severity::Error, Some(FailureCode::new(check.code.clone())))
            }
        };
        let mut result = Self {
            id: check.id,
            code: check.code,
            name: check.name,
            helixtest_name: None,
            service: check.service,
            category: check.category,
            profile: check.profile,
            status,
            severity,
            message: None,
            failure,
            diagnostic: None,
        };
        crate::diagnostics::attach(&mut result);
        result
    }

    /// Record the original HelixTest test name (not renamed).
    pub fn with_helixtest_name(mut self, name: impl Into<String>) -> Self {
        self.helixtest_name = Some(name.into());
        self
    }

    /// Preserve HelixTest `error` text on `message` and, for fail/error, `failure.detail`.
    pub fn with_error_text(mut self, error: impl Into<String>) -> Self {
        let error = crate::redact::redact_text(&error.into());
        if let Some(f) = self.failure.as_mut() {
            f.detail = Some(error.clone());
        }
        self.message = Some(error);
        crate::diagnostics::attach(&mut self);
        self
    }

    pub fn pass(check: VerificationCheck) -> Self {
        Self::from_check(check, VerificationStatus::Pass)
    }

    pub fn fail(check: VerificationCheck, message: impl Into<String>) -> Self {
        let mut r = Self::from_check(check, VerificationStatus::Fail);
        r.message = Some(crate::redact::redact_text(&message.into()));
        crate::diagnostics::attach(&mut r);
        r
    }

    pub fn skip(check: VerificationCheck, message: impl Into<String>) -> Self {
        let mut r = Self::from_check(check, VerificationStatus::Skip);
        r.message = Some(crate::redact::redact_text(&message.into()));
        r
    }

    pub fn error(check: VerificationCheck, message: impl Into<String>) -> Self {
        let mut r = Self::from_check(check, VerificationStatus::Error);
        r.message = Some(crate::redact::redact_text(&message.into()));
        crate::diagnostics::attach(&mut r);
        r
    }

    pub fn is_pass(&self) -> bool {
        self.status.is_pass()
    }
}

/// Service seen (or not) under the target. `service` is an open string.
/// Distinct from [`crate::discover::DiscoveredService`] (probe enum + URL).
///
/// `present` is DETECTED. `testable` is TESTABLE. Neither is a verification pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredService {
    pub service: String,
    pub present: bool,
    pub testable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_testable_reason: Option<String>,
}

impl DiscoveredService {
    pub fn found(service: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            present: true,
            testable: true,
            base_url: Some(base_url.into()),
            not_testable_reason: None,
        }
    }

    pub fn detected_not_testable(
        service: impl Into<String>,
        base_url: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            service: service.into(),
            present: true,
            testable: false,
            base_url: Some(base_url.into()),
            not_testable_reason: Some(reason.into()),
        }
    }

    pub fn missing(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            present: false,
            testable: false,
            base_url: None,
            not_testable_reason: Some("not detected; nothing to test".into()),
        }
    }
}

/// Counts only. Not a weighted score, not a certification level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerificationSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total: usize,
}

/// One Helix verification run. Future services/profiles add rows, not fields
/// (except optional pin/profile metadata).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRun {
    /// Frozen document id (`helix-verification-v1`). Missing on old files is treated as this value.
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub helix_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helixtest_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helixtest_sha: Option<String>,
    /// Helix profile id (`generic` or `ferrum`). Not HelixTest Mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Fixture catalog id. Default `helix-fixtures-v1` if missing on old files.
    #[serde(default = "default_fixture_version")]
    pub fixture_version: String,
    pub timestamp: String,
    pub target: Target,
    pub discovery: Vec<DiscoveredService>,
    pub executed: Vec<VerificationResult>,
    pub skipped: Vec<VerificationResult>,
    pub summary: VerificationSummary,
}

/// DRS-only helper for tests that still build a single-service run.
pub const DRS_PROFILE: &str = "drs";

impl VerificationRun {
    pub fn new(target: Target) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            helix_version: helix_version().to_string(),
            helixtest_version: Some(HELIXTEST_PIN.to_string()),
            helixtest_sha: Some(HELIXTEST_SHA.to_string()),
            profile: None,
            fixture_version: FIXTURE_VERSION.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            target,
            discovery: Vec::new(),
            executed: Vec::new(),
            skipped: Vec::new(),
            summary: VerificationSummary::default(),
        }
    }

    pub fn drs_profile(target: Target) -> Self {
        let mut run = Self::new(target);
        run.profile = Some(DRS_PROFILE.to_string());
        run
    }

    /// Omit HelixTest pin when the run did not use that engine.
    pub fn without_helixtest(mut self) -> Self {
        self.helixtest_version = None;
        self.helixtest_sha = None;
        self
    }

    /// Stable order for JSON: by `code`, then `id`.
    pub fn sort_deterministic(&mut self) {
        self.executed
            .sort_by(|a, b| a.code.cmp(&b.code).then(a.id.cmp(&b.id)));
        self.skipped
            .sort_by(|a, b| a.code.cmp(&b.code).then(a.id.cmp(&b.id)));
    }

    pub fn push_executed(&mut self, result: VerificationResult) {
        if result.status == VerificationStatus::Skip {
            self.push_skipped(result);
            return;
        }
        self.executed.push(result);
        self.recompute_summary();
    }

    pub fn push_skipped(&mut self, mut result: VerificationResult) {
        // Invariant: a skipped row cannot be stored as PASS.
        result.status = VerificationStatus::Skip;
        if result.severity == Severity::Error {
            result.severity = Severity::Info;
        }
        result.failure = None;
        self.skipped.push(result);
        self.recompute_summary();
    }

    pub fn recompute_summary(&mut self) {
        let mut summary = VerificationSummary::default();
        for r in self.executed.iter().chain(self.skipped.iter()) {
            summary.total += 1;
            match r.status {
                VerificationStatus::Pass => summary.passed += 1,
                VerificationStatus::Fail => summary.failed += 1,
                VerificationStatus::Skip => summary.skipped += 1,
                VerificationStatus::Error => summary.errors += 1,
            }
        }
        self.summary = summary;
    }

    /// Any FAIL or ERROR. Skip does not count.
    pub fn has_failures(&self) -> bool {
        self.executed.iter().any(|r| r.status.is_blocking())
            || self.skipped.iter().any(|r| r.status.is_blocking())
    }

    /// Run-level status. All-skip (or empty) is Skip, never Pass.
    /// ERROR wins over FAIL regardless of list order.
    pub fn overall_status(&self) -> VerificationStatus {
        let mut saw_fail = false;
        let mut saw_pass = false;
        for r in self.executed.iter().chain(self.skipped.iter()) {
            match r.status {
                VerificationStatus::Error => return VerificationStatus::Error,
                VerificationStatus::Fail => saw_fail = true,
                VerificationStatus::Pass => saw_pass = true,
                VerificationStatus::Skip => {}
            }
        }
        if saw_fail {
            VerificationStatus::Fail
        } else if saw_pass {
            VerificationStatus::Pass
        } else {
            VerificationStatus::Skip
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn not_found_check() -> VerificationCheck {
        VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found"))
            .with_profile("generic")
    }

    #[test]
    fn skip_is_never_pass() {
        assert!(!VerificationStatus::Skip.is_pass());
        assert!(!VerificationStatus::Fail.is_pass());
        assert!(!VerificationStatus::Error.is_pass());
        assert!(VerificationStatus::Pass.is_pass());
        let r = VerificationResult::skip(not_found_check(), "not wired");
        assert!(!r.is_pass());
        assert_eq!(r.status, VerificationStatus::Skip);
    }

    #[test]
    fn skip_serializes_as_skip_not_pass() {
        let r = VerificationResult::skip(not_found_check(), "discovered; not executed");
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["status"].as_str(), Some("skip"));
        assert_ne!(v["status"].as_str(), Some("pass"));
        assert!(
            v.get("passed").is_none(),
            "do not revive HelixTest `passed` bool"
        );
    }

    #[test]
    fn check_identity_is_stable() {
        let id = catalog::drs_object_not_found();
        assert_eq!(id.id, "drs.object.not_found");
        assert_eq!(id.code, "HLX-DRS-005");
        let r = VerificationResult::fail(not_found_check(), "got 200");
        assert_eq!(r.id, "drs.object.not_found");
        assert_eq!(r.code, "HLX-DRS-005");
        assert_eq!(
            r.failure.as_ref().map(|f| f.code.as_str()),
            Some("HLX-DRS-005")
        );
    }

    #[test]
    fn fail_redacts_authorization_jwt_and_userinfo() {
        let jwt =
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let r = VerificationResult::fail(
            not_found_check(),
            format!("Authorization: Bearer {jwt} at http://alice:s3cret@example.org/"),
        );
        let msg = r.message.as_deref().unwrap();
        assert!(!msg.contains(jwt), "{msg}");
        assert!(!msg.contains("s3cret"), "{msg}");
        assert!(!msg.contains("Bearer eyJ"), "{msg}");
        let observed = &r.diagnostic.as_ref().unwrap().observed;
        assert!(!observed.contains(jwt), "{observed}");
        assert!(!observed.contains("s3cret"), "{observed}");
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains(jwt), "{json}");
        assert!(!json.contains("s3cret"), "{json}");
    }

    #[test]
    fn fail_and_error_are_distinct() {
        let fail = VerificationResult::fail(not_found_check(), "expected 404");
        let err = VerificationResult::error(not_found_check(), "timeout");
        assert_eq!(fail.status, VerificationStatus::Fail);
        assert_eq!(err.status, VerificationStatus::Error);
        assert_ne!(fail.status, err.status);
        assert!(fail.status.is_blocking());
        assert!(err.status.is_blocking());
    }

    #[test]
    fn summary_does_not_count_skip_as_pass() {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:8080"));
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            crate::identity::spec("drs.object.reachable"),
        )));
        run.push_skipped(VerificationResult::skip(
            VerificationCheck::from_spec(crate::identity::spec("wes.service_info.reachable")),
            "Stage 1 DRS first",
        ));
        assert_eq!(run.summary.passed, 1);
        assert_eq!(run.summary.skipped, 1);
        assert_eq!(run.summary.failed, 0);
        assert_eq!(run.summary.errors, 0);
        assert_eq!(run.summary.total, 2);
        assert!(!run.has_failures());
        assert_eq!(run.overall_status(), VerificationStatus::Pass);
    }

    #[test]
    fn all_skip_overall_is_not_pass() {
        let mut run = VerificationRun::new(Target::new("http://example.invalid"));
        run.push_skipped(VerificationResult::skip(not_found_check(), "no secret"));
        assert_eq!(run.overall_status(), VerificationStatus::Skip);
        assert!(!run.overall_status().is_pass());
        assert_eq!(run.summary.passed, 0);
        assert_eq!(run.summary.skipped, 1);
    }

    #[test]
    fn empty_run_is_not_pass() {
        let run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        assert_eq!(run.overall_status(), VerificationStatus::Skip);
        assert!(!run.overall_status().is_pass());
    }

    #[test]
    fn error_wins_over_fail() {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:8080"));
        run.push_executed(VerificationResult::fail(not_found_check(), "404 missing"));
        run.push_executed(VerificationResult::error(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.reachable")),
            "timeout",
        ));
        assert_eq!(run.overall_status(), VerificationStatus::Error);
        assert!(run.has_failures());
    }

    #[test]
    fn push_skipped_cannot_store_pass() {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:8080"));
        run.push_skipped(VerificationResult::pass(not_found_check()));
        assert_eq!(run.skipped.len(), 1);
        assert_eq!(run.skipped[0].status, VerificationStatus::Skip);
        assert!(!run.skipped[0].is_pass());
        assert_eq!(run.summary.passed, 0);
        assert_eq!(run.summary.skipped, 1);
    }

    #[test]
    fn future_service_does_not_change_shape() {
        let check = VerificationCheck::from_spec(crate::identity::spec("beacon.query.reachable"))
            .with_profile("generic");
        let r = VerificationResult::skip(check, "not in Stage 1");
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["service"].as_str(), Some("beacon"));
        assert_eq!(v["id"].as_str(), Some("beacon.query.reachable"));
        assert_eq!(v["code"].as_str(), Some("HLX-BEACON-001"));
        assert_eq!(v["category"].as_str(), Some("robustness"));
        assert!(v.get("level").is_none());
        assert!(v.get("score").is_none());
        assert!(v.get("signature").is_none());
        assert!(v.get("ro_crate").is_none());
        assert!(v.get("evidence").is_none());
        assert!(v.get("audit_trail").is_none());
        assert!(v.get("pdf").is_none());
    }

    #[test]
    fn run_json_roundtrip() {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:8080"));
        run.timestamp = "2026-09-04T09:00:00Z".into();
        run.discovery = vec![
            DiscoveredService::found("drs", "http://127.0.0.1:8080/ga4gh/drs/v1"),
            DiscoveredService::missing("wes"),
        ];
        run.push_executed(VerificationResult::fail(
            not_found_check(),
            "expected 404, got 200",
        ));
        run.push_skipped(VerificationResult::skip(
            VerificationCheck::from_spec(crate::identity::spec("wes.service_info.reachable")),
            "not discovered",
        ));

        let json = serde_json::to_string(&run).unwrap();
        let back: VerificationRun = serde_json::from_str(&json).unwrap();
        assert_eq!(back, run);
        assert_eq!(back.helix_version, helix_version());
        assert_eq!(back.fixture_version, crate::model::FIXTURE_VERSION);
        assert_eq!(back.helixtest_version.as_deref(), Some(HELIXTEST_PIN));
        assert_eq!(back.target.url, "http://127.0.0.1:8080");
        assert_eq!(back.discovery.len(), 2);
        assert_eq!(back.executed.len(), 1);
        assert_eq!(back.skipped.len(), 1);
        assert_eq!(back.summary.failed, 1);
        assert_eq!(back.summary.skipped, 1);
        assert!(back.has_failures());
        assert_eq!(back.overall_status(), VerificationStatus::Fail);

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("signature").is_none());
        assert!(v.get("ro_crate").is_none());
        assert!(v.get("overall_score").is_none());
        assert!(v.get("overall_level").is_none());
        assert_eq!(v["executed"][0]["id"], "drs.object.not_found");
        assert_eq!(v["executed"][0]["code"], "HLX-DRS-005");
        assert_eq!(v["executed"][0]["name"], "Unknown DRS object returns 404");
        assert_eq!(v["executed"][0]["category"], "robustness");
        assert_eq!(v["executed"][0]["status"], "fail");
        assert_eq!(v["skipped"][0]["status"], "skip");
        assert_eq!(v["schema_version"], SCHEMA_VERSION);
        assert!(v.get("services").is_none());
        assert!(v.get("checks").is_none());
        assert_eq!(v["executed"][0]["diagnostic"]["code"], "HLX-DRS-005");
        assert_eq!(v["executed"][0]["diagnostic"]["observed"], "HTTP 200");
        assert_eq!(
            v["executed"][0]["diagnostic"]["likely_category"],
            "error_handling"
        );
        assert!(v["executed"][0]["diagnostic"].get("cause").is_none());
        assert!(v["skipped"][0].get("diagnostic").is_none());
    }

    #[test]
    fn skip_json_omits_diagnostic() {
        let r = VerificationResult::skip(not_found_check(), "not wired");
        let v = serde_json::to_value(&r).unwrap();
        assert!(v.get("diagnostic").is_none());
    }

    #[test]
    fn helixtest_version_optional() {
        let run = VerificationRun::new(Target::new("http://127.0.0.1:1")).without_helixtest();
        let v = serde_json::to_value(&run).unwrap();
        assert!(v.get("helixtest_version").is_none());
        assert!(v.get("helixtest_sha").is_none());
        assert_eq!(v["helix_version"].as_str(), Some(helix_version()));
        assert_eq!(v["schema_version"].as_str(), Some(SCHEMA_VERSION));
        assert_eq!(v["fixture_version"].as_str(), Some(FIXTURE_VERSION));
    }

    #[test]
    fn missing_schema_version_deserializes_as_v1() {
        let json = r#"{
            "helix_version": "0.1.0",
            "timestamp": "2026-09-04T12:00:00Z",
            "target": { "url": "http://127.0.0.1:9" },
            "discovery": [],
            "executed": [],
            "skipped": [],
            "summary": { "passed": 0, "failed": 0, "skipped": 0, "errors": 0, "total": 0 }
        }"#;
        let run: VerificationRun = serde_json::from_str(json).unwrap();
        assert_eq!(run.schema_version, SCHEMA_VERSION);
        assert_eq!(run.fixture_version, FIXTURE_VERSION);
    }
}
