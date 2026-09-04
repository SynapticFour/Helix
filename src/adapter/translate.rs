// SPDX-License-Identifier: Apache-2.0
//! HelixTest `TestCaseResult` → Helix `VerificationResult`.
//!
//! Status comes from HelixTest `status` only. The `passed` boolean is ignored so a
//! malformed Skip row cannot become Pass. Skip is never pass.

use common::report::{ServiceReport, TestCaseResult, TestStatus};

use crate::identity::spec_by_helixtest_name;
use crate::model::{CheckIdentity, VerificationCheck, VerificationResult, VerificationStatus};

/// HelixTest has no Error variant. Fail stays Fail (target assertion). Skip stays Skip.
pub fn map_status(status: TestStatus) -> VerificationStatus {
    match status {
        TestStatus::Pass => VerificationStatus::Pass,
        TestStatus::Fail => VerificationStatus::Fail,
        TestStatus::Skip => VerificationStatus::Skip,
    }
}

fn check_for_case(tc: &TestCaseResult) -> VerificationCheck {
    match spec_by_helixtest_name(&tc.name) {
        Some(spec) => VerificationCheck::from_spec(spec).with_profile("generic"),
        None => VerificationCheck::new(
            CheckIdentity::new("helixtest.unmapped", "UNMAPPED"),
            tc.name.clone(),
            "unknown",
        )
        .with_profile("generic"),
    }
}

/// Translate one HelixTest case. Does not read `passed`.
pub fn translate_test_case(tc: &TestCaseResult) -> VerificationResult {
    let check = check_for_case(tc);
    let status = map_status(tc.status);
    // Map from HelixTest `status` only. Ignore `passed` so a Skip row cannot
    // become Pass even if that boolean is wrong.
    let mut result =
        VerificationResult::from_check(check, status).with_helixtest_name(tc.name.clone());
    if let Some(err) = tc.error.as_ref().filter(|s| !s.is_empty()) {
        result = result.with_error_text(err.clone());
    }
    debug_assert!(
        !(tc.status == TestStatus::Skip && result.status == VerificationStatus::Pass),
        "never convert SKIP into PASS"
    );
    result
}

pub fn translate_service_report(report: &ServiceReport) -> Vec<VerificationResult> {
    report.tests.iter().map(translate_test_case).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::VerificationStatus;
    use common::report::{ComplianceLevel, TestCategory};

    #[test]
    fn skip_is_never_translated_to_pass() {
        let tc = TestCaseResult::skip(
            "DRS object endpoint reachable",
            ComplianceLevel::Level0,
            TestCategory::Other,
            "no fixture",
        );
        assert!(!tc.passed);
        let r = translate_test_case(&tc);
        assert_eq!(r.status, VerificationStatus::Skip);
        assert!(!r.is_pass());
        assert_eq!(r.id, "drs.object.reachable");
        assert_eq!(r.code, "HLX-DRS-001");
        assert_eq!(
            r.helixtest_name.as_deref(),
            Some("DRS object endpoint reachable")
        );
        assert!(
            r.message
                .as_deref()
                .is_some_and(|m| m.contains("no fixture")),
            "skip reason preserved: {:?}",
            r.message
        );
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn ignore_passed_bool_when_status_is_skip() {
        let mut tc = TestCaseResult::skip(
            "DRS invalid object id returns 404",
            ComplianceLevel::Level0,
            TestCategory::Other,
            "not executed",
        );
        tc.passed = true;
        let r = translate_test_case(&tc);
        assert_eq!(r.status, VerificationStatus::Skip);
        assert!(!r.is_pass());
        assert_ne!(r.status, VerificationStatus::Pass);
    }

    #[test]
    fn fail_preserves_error_and_identity() {
        let tc = TestCaseResult::fail(
            "DRS invalid object id returns 404",
            ComplianceLevel::Level0,
            TestCategory::Other,
            "expected 404, got 200",
        );
        let r = translate_test_case(&tc);
        assert_eq!(r.status, VerificationStatus::Fail);
        assert_eq!(r.id, "drs.object.not_found");
        assert_eq!(r.code, "HLX-DRS-005");
        assert_eq!(
            r.helixtest_name.as_deref(),
            Some("DRS invalid object id returns 404")
        );
        assert_eq!(r.message.as_deref(), Some("expected 404, got 200"));
        assert_eq!(
            r.failure.as_ref().and_then(|f| f.detail.as_deref()),
            Some("expected 404, got 200")
        );
        let d = r
            .diagnostic
            .as_ref()
            .expect("catalogued fail has diagnostic");
        assert_eq!(d.code, "HLX-DRS-005");
        assert_eq!(d.observed, "HTTP 200");
        assert_eq!(d.expected, "HTTP 404 for an unknown object ID");
    }

    #[test]
    fn adapter_redacts_authorization_from_helixtest_error() {
        let jwt =
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let tc = TestCaseResult::fail(
            "DRS invalid object id returns 404",
            ComplianceLevel::Level0,
            TestCategory::Other,
            format!("Authorization: Bearer {jwt}"),
        );
        let r = translate_test_case(&tc);
        let msg = r.message.as_deref().unwrap();
        assert!(!msg.contains(jwt), "{msg}");
        assert!(!msg.contains("Bearer eyJ"), "{msg}");
        let observed = &r.diagnostic.as_ref().unwrap().observed;
        assert!(!observed.contains(jwt), "{observed}");
    }

    #[test]
    fn pass_maps_catalog_identity() {
        let tc = TestCaseResult::pass(
            "DRS checksum correctness",
            ComplianceLevel::Level0,
            TestCategory::Other,
        );
        let r = translate_test_case(&tc);
        assert_eq!(r.status, VerificationStatus::Pass);
        assert_eq!(r.id, "drs.object.checksum");
        assert_eq!(r.code, "HLX-DRS-003");
        assert!(r.message.is_none());
        assert!(r.failure.is_none());
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn unmapped_name_keeps_original_identity_string() {
        let tc = TestCaseResult::fail(
            "some future HelixTest name",
            ComplianceLevel::Level0,
            TestCategory::Other,
            "boom",
        );
        let r = translate_test_case(&tc);
        assert_eq!(r.id, "helixtest.unmapped");
        assert_eq!(r.code, "UNMAPPED");
        assert_eq!(r.name, "some future HelixTest name");
        assert_eq!(
            r.helixtest_name.as_deref(),
            Some("some future HelixTest name")
        );
        assert_eq!(r.message.as_deref(), Some("boom"));
        assert!(r.diagnostic.is_none(), "unmapped ids get no guessed story");
    }
}
