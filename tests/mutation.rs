// SPDX-License-Identifier: Apache-2.0
//! Adversarial mutation corpus.
//!
//! correct target → PASS
//! known-bad target → FAIL
//! known-bad target → correct failure reason
//! missed mutants stay listed, with a reason.
//!
//! Not certification. Not HELIOS. Do not weaken Helix to make a mutant pass.

use common::report::TestStatus;
use helix::identity::spec;
use helix::model::VerificationStatus;
use helix::mutation::{
    summary, validate_catalog, DiagnosticClass, Mutant, MutationDetection, MutationSuite, CATALOG,
};
use helix::security::run_security;
use helix::verify::VerifyOutcome;

mod support;

use support::mock_ga4gh_wes::start_mock_ga4gh_drs_and_wes;
use support::mock_mutation::start_mutant;

const HMAC: &str = "helix-dummy-hmac-not-for-production-do-not-use";

fn crypt4gh_ok() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/crypt4gh/well-formed.c4gh")
}

fn find_row<'a>(
    outcome: &'a VerifyOutcome,
    id: &str,
) -> Option<&'a helix::model::VerificationResult> {
    outcome
        .run
        .executed
        .iter()
        .chain(outcome.run.skipped.iter())
        .find(|r| r.id == id)
}

fn status_of(outcome: &VerifyOutcome, id: &str) -> Option<VerificationStatus> {
    find_row(outcome, id).map(|r| r.status)
}

async fn verify_mutant(id: &str) -> (support::mock_mutation::MutationTarget, VerifyOutcome) {
    let target = start_mutant(id).await;
    let outcome = helix::verify::verify(&target.url())
        .await
        .unwrap_or_else(|e| panic!("{id}: verify must terminate: {e:#}"));
    (target, outcome)
}

fn assert_detected_verify(m: &Mutant, outcome: &VerifyOutcome) {
    assert!(
        !outcome.is_success(),
        "{}: known-bad target must not be overall PASS",
        m.id
    );
    if m.diagnostic == DiagnosticClass::FailClosed {
        assert_eq!(
            outcome.run.summary.passed, 0,
            "{}: fail-closed mutant must not record a pass",
            m.id
        );
        return;
    }
    let check = m
        .expected_check_id
        .unwrap_or_else(|| panic!("{} detected verify mutant needs check id", m.id));
    let row = find_row(outcome, check).unwrap_or_else(|| {
        panic!(
            "{}: missing check {check} in {:?}",
            m.id,
            outcome
                .run
                .executed
                .iter()
                .chain(outcome.run.skipped.iter())
                .map(|r| (r.id.as_str(), r.status))
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(
        row.status,
        VerificationStatus::Fail,
        "{}: {check} must FAIL (correct failure), got {:?} msg={:?}",
        m.id,
        row.status,
        row.message
    );
    let diag = row
        .diagnostic
        .as_ref()
        .unwrap_or_else(|| panic!("{}: fail row must have a diagnostic", m.id));
    match m.diagnostic {
        DiagnosticClass::Verify(cat) => {
            assert_eq!(
                diag.likely_category,
                cat,
                "{}: diagnostic class must be {}, got {}",
                m.id,
                cat.as_str(),
                diag.likely_category.as_str()
            );
        }
        other => panic!("{}: verify mutant has diagnostic {:?}", m.id, other),
    }
    if let Some(sub) = m.expected_message_substr {
        let msg = row.message.as_deref().unwrap_or("");
        assert!(
            msg.contains(sub),
            "{}: known-bad must fail for the recorded reason containing {sub:?}, got {msg}",
            m.id
        );
    }
}

async fn assert_detected_security(m: &Mutant) {
    let target = start_mutant(m.id).await;
    let out = run_security(&target.url(), Some(HMAC), Some(&crypt4gh_ok()))
        .await
        .expect("security");
    let check = m
        .expected_check_id
        .unwrap_or_else(|| panic!("{} security mutant needs check id", m.id));
    let name = spec(check).name;
    let row = out
        .auth
        .tests
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("{}: missing security row {name}", m.id));
    assert_eq!(
        row.status,
        TestStatus::Fail,
        "{}: {check} must FAIL, got {:?} err={:?}",
        m.id,
        row.status,
        row.error
    );
    if let Some(sub) = m.expected_message_substr {
        let err = row.error.as_deref().unwrap_or("");
        assert!(
            err.contains(sub),
            "{}: security fail must mention {sub:?}, got {err}",
            m.id
        );
    }
    let _keep = target;
}

fn assert_missed_verify(m: &Mutant, outcome: &VerifyOutcome) {
    if let Some(check) = m.expected_check_id {
        let st = status_of(outcome, check);
        assert_ne!(
            st,
            Some(VerificationStatus::Fail),
            "{} was documented as missed ({}); {check} now FAILs — update CATALOG to Detected. msg={:?}",
            m.id,
            m.miss_reason.unwrap_or(""),
            find_row(outcome, check).and_then(|r| r.message.clone())
        );
        assert_ne!(
            st,
            Some(VerificationStatus::Error),
            "{} was documented as missed; {check} is now ERROR — update CATALOG if this is detection",
            m.id
        );
    } else {
        let drs_failed = outcome
            .run
            .executed
            .iter()
            .any(|r| r.service == "drs" && r.status == VerificationStatus::Fail);
        let wes_failed = outcome
            .run
            .executed
            .iter()
            .any(|r| r.service == "wes" && r.status == VerificationStatus::Fail);
        assert!(
            !drs_failed && !wes_failed,
            "{} was documented as missed (Helix never exercises the broken surface); DRS/WES executed FAILs would mean the catalog is stale: {:?}",
            m.id,
            outcome
                .run
                .executed
                .iter()
                .filter(|r| r.status == VerificationStatus::Fail)
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            outcome.is_success(),
            "{}: honest executed checks should still PASS when the defect is on an unprobed path",
            m.id
        );
    }
}

#[test]
fn catalog_covers_requested_defect_classes() {
    validate_catalog().expect("catalog");
    let s = summary();
    assert_eq!(s.attempted, s.detected + s.missed);
    assert_eq!(s.attempted, CATALOG.len());
}

#[tokio::test]
async fn correct_target_passes() {
    let mock = start_mock_ga4gh_drs_and_wes().await;
    let outcome = helix::verify::verify(&mock.origin())
        .await
        .expect("control verify");
    assert!(outcome.is_success(), "correct target → PASS: {outcome:?}");
    for id in helix::identity::DRS_VERIFY_IDS {
        assert_eq!(
            status_of(&outcome, id),
            Some(VerificationStatus::Pass),
            "{id}"
        );
    }
    for id in helix::identity::WES_VERIFY_IDS.iter().copied() {
        if id == "wes.run.scatter_gather" {
            assert_eq!(status_of(&outcome, id), Some(VerificationStatus::Skip));
        } else {
            assert_eq!(
                status_of(&outcome, id),
                Some(VerificationStatus::Pass),
                "{id}"
            );
        }
    }
}

#[tokio::test]
async fn known_bad_targets_fail_for_the_recorded_reason() {
    for m in CATALOG.iter().filter(|m| m.is_detected()) {
        match m.suite {
            MutationSuite::Verify => {
                let (_keep, outcome) = verify_mutant(m.id).await;
                assert_detected_verify(m, &outcome);
            }
            MutationSuite::Security => {
                assert_detected_security(m).await;
            }
        }
    }
}

#[tokio::test]
async fn missed_mutations_are_recorded_and_not_hidden() {
    let missed: Vec<_> = CATALOG
        .iter()
        .filter(|m| m.detection == MutationDetection::Missed)
        .collect();
    assert!(
        !missed.is_empty(),
        "a corpus with zero misses is hiding gaps"
    );
    for m in missed {
        assert!(m.miss_reason.is_some(), "{} miss must have a reason", m.id);
        assert_eq!(m.diagnostic, DiagnosticClass::None);
        let (_keep, outcome) = verify_mutant(m.id).await;
        assert_missed_verify(m, &outcome);
    }
}

#[test]
fn mutation_summary_names_misses() {
    let s = summary();
    let misses: Vec<_> = CATALOG
        .iter()
        .filter(|m| m.detection == MutationDetection::Missed)
        .map(|m| (m.id, m.miss_reason.unwrap()))
        .collect();
    assert_eq!(misses.len(), s.missed);
    for (id, reason) in misses {
        assert!(!reason.is_empty(), "{id}");
    }
}
