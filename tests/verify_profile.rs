// SPDX-License-Identifier: Apache-2.0
//! Helix profiles (`generic`, `ferrum`). Not HelixTest `--mode ferrum`.
//! A generic target never auto-switches to ferrum behavior.

use assert_cmd::Command;
use helix::identity::WES_VERIFY_IDS;
use helix::model::VerificationStatus;
use helix::profile::ProfileId;
use helix::verify::verify_with_profile;
use predicates::prelude::*;
use serde_json::Value;

mod support;

use support::mock_ga4gh_drs::start_mock_ga4gh_drs;
use support::mock_ga4gh_wes::{start_mock_ga4gh_drs_and_wes, start_mock_ga4gh_wes_named};

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn parse_json(stdout: &[u8]) -> Value {
    let text = String::from_utf8_lossy(stdout);
    serde_json::from_str(text.trim()).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; stdout={text}");
    })
}

#[tokio::test]
async fn generic_drs_only_passes_and_sets_profile_generic() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = helix::verify::verify(&mock.drs_url())
        .await
        .expect("verify");
    assert!(outcome.is_success());
    assert_eq!(outcome.run.profile.as_deref(), Some("generic"));
    assert!(outcome
        .run
        .skipped
        .iter()
        .any(|r| r.id == "wes.run.scatter_gather" && r.status == VerificationStatus::Skip));
    assert!(!outcome
        .run
        .executed
        .iter()
        .any(|r| r.service == "wes" && r.status == VerificationStatus::Fail));
}

#[tokio::test]
async fn omitted_profile_flag_is_generic() {
    let mock = start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let omitted = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args(["verify", &url, "--format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    let explicit = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args(["verify", &url, "--profile", "generic", "--format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(omitted["profile"].as_str(), Some("generic"));
    assert_eq!(explicit["profile"].as_str(), Some("generic"));
}

#[tokio::test]
async fn ferrum_drs_only_fails_expected_wes() {
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_profile(&mock.drs_url(), ProfileId::Ferrum)
        .await
        .expect("verify");
    assert!(!outcome.is_success());
    assert_eq!(outcome.run.profile.as_deref(), Some("ferrum"));
    let wes_fail: Vec<_> = outcome
        .run
        .executed
        .iter()
        .filter(|r| r.service == "wes")
        .collect();
    assert_eq!(wes_fail.len(), 8);
    for (i, r) in wes_fail.iter().enumerate() {
        assert_eq!(r.id, WES_VERIFY_IDS[i]);
        assert_eq!(r.status, VerificationStatus::Fail);
        assert!(
            r.message
                .as_deref()
                .unwrap_or("")
                .contains("WES expected by profile ferrum but not detected"),
            "{r:?}"
        );
    }
    assert!(outcome.run.skipped.iter().all(|r| r.service != "wes"));

    helix()
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--profile",
            "ferrum",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains(
            "WES expected by profile ferrum but not detected",
        ));
}

#[tokio::test]
async fn generic_does_not_auto_switch_on_ferrum_gateway_name() {
    let mock = start_mock_ga4gh_wes_named("Ferrum Gateway").await;
    let outcome = helix::verify::verify(&mock.origin()).await.expect("verify");
    assert_eq!(outcome.run.profile.as_deref(), Some("generic"));
    let scatter = outcome
        .run
        .skipped
        .iter()
        .chain(outcome.run.executed.iter())
        .find(|r| r.id == "wes.run.scatter_gather")
        .expect("scatter row");
    assert_eq!(
        scatter.status,
        VerificationStatus::Skip,
        "WES name Ferrum Gateway must not enable scatter: {scatter:?}"
    );
    assert!(
        scatter
            .message
            .as_deref()
            .unwrap_or("")
            .contains("supports_scatter_gather=false"),
        "{scatter:?}"
    );
}

#[tokio::test]
async fn ferrum_combined_target_runs_scatter_through_public_http() {
    let mock = start_mock_ga4gh_drs_and_wes().await;
    let outcome = verify_with_profile(&mock.origin(), ProfileId::Ferrum)
        .await
        .expect("verify");
    assert!(
        outcome.is_success(),
        "ferrum DRS+WES mock should pass: {outcome:?}"
    );
    assert_eq!(outcome.run.profile.as_deref(), Some("ferrum"));
    let scatter = outcome
        .run
        .executed
        .iter()
        .find(|r| r.id == "wes.run.scatter_gather")
        .expect("scatter executed");
    assert_eq!(scatter.status, VerificationStatus::Pass);
    assert!(!outcome
        .run
        .skipped
        .iter()
        .any(|r| r.id == "wes.run.scatter_gather"));
}

#[test]
fn unknown_profile_is_usage_exit_2() {
    helix()
        .args(["verify", "http://127.0.0.1:9", "--profile", "ga4gh-drs"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}
