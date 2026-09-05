// SPDX-License-Identifier: Apache-2.0
//! Independent verification: same fixture → same outcomes (not bit-for-bit files).
//! Authorship is irrelevant. Not HELIOS. Not certification.

use helix::compare::compare_files;
use helix::model::VerificationStatus;
use helix::report::verify_json;
use helix::repro::{canonicalize_verify_json, failure_fingerprint, outcome_fingerprint};
use helix::standards::{default_registry_path, validate_path};

mod support;

use support::mock_ga4gh_drs::{start_mock_ga4gh_drs, start_mock_invalid_drs_object};

fn parse_run_json(run: &helix::model::VerificationRun) -> serde_json::Value {
    serde_json::from_str(&verify_json(run).expect("verify JSON")).expect("JSON")
}

#[tokio::test]
async fn two_verifies_on_the_same_fixture_match_after_stripping_timestamp() {
    let mock = start_mock_ga4gh_drs().await;
    let a = helix::verify::verify(&mock.drs_url())
        .await
        .expect("first verify");
    let b = helix::verify::verify(&mock.drs_url())
        .await
        .expect("second verify");
    assert!(a.is_success());
    assert!(b.is_success());
    assert_eq!(
        outcome_fingerprint(&a.run),
        outcome_fingerprint(&b.run),
        "check id/status/code must match"
    );
    assert_eq!(a.run.target.url, b.run.target.url);
    let mut ja = parse_run_json(&a.run);
    let mut jb = parse_run_json(&b.run);
    canonicalize_verify_json(&mut ja);
    canonicalize_verify_json(&mut jb);
    assert_eq!(
        ja, jb,
        "canonical JSON (timestamp replaced) must match; raw files are not claimed identical"
    );
}

#[tokio::test]
async fn known_bad_fixture_fails_the_same_way_twice() {
    let server = start_mock_invalid_drs_object().await;
    let url = server.uri();
    let a = helix::verify::verify(&url).await.expect("first fail run");
    let b = helix::verify::verify(&url).await.expect("second fail run");
    assert!(!a.is_success());
    assert!(!b.is_success());
    assert_eq!(failure_fingerprint(&a.run), failure_fingerprint(&b.run));
    assert_eq!(outcome_fingerprint(&a.run), outcome_fingerprint(&b.run));
    let schema = a
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .expect("schema");
    assert_eq!(schema.status, VerificationStatus::Fail);
    let diag = schema.diagnostic.as_ref().expect("diagnostic");
    assert!(!diag.expected.is_empty());
    assert!(!diag.observed.is_empty());
}

#[tokio::test]
async fn helix_compare_of_two_fixture_runs_is_not_a_regression() {
    let mock = start_mock_ga4gh_drs().await;
    let a = helix::verify::verify(&mock.drs_url()).await.expect("a");
    let b = helix::verify::verify(&mock.drs_url()).await.expect("b");
    let dir = std::env::temp_dir().join(format!("helix-repro-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmpdir");
    let pa = dir.join("a.json");
    let pb = dir.join("b.json");
    std::fs::write(&pa, verify_json(&a.run).unwrap()).unwrap();
    std::fs::write(&pb, verify_json(&b.run).unwrap()).unwrap();
    let report = compare_files(&pa, &pb).expect("compare");
    assert!(
        !report.has_regression,
        "two honest fixture runs must not be NEW_FAIL"
    );
    assert!(report.same_measurement);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn shipped_standards_validate_without_network() {
    let path = default_registry_path();
    validate_path(&path).expect("registry + vendor sha256; no download");
}
