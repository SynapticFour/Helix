// SPDX-License-Identifier: Apache-2.0
//! B12.1 verifier provenance / exact-commit gate.
//! Not HELIOS. Not a verification identity. Does not change coverage_id.

use helix::claim_integrity::validate_claim_integrity;
use helix::coverage::{coverage_id, CoverageState};
use helix::live_evidence::{
    live_independent_observation, PINNED_BINDING_ID, PINNED_COVERAGE_ID, PINNED_EXECUTION_ID,
    PINNED_PACK_INTEGRITY_SHA256,
};
use helix::model::{VerificationRun, HELIXTEST_SHA};
use helix::provenance::{cites_this_verifier_build, helix_git_dirty, helix_git_sha};
use helix::standards::{DRS_140_CONTRACT, DRS_140_PACK_ID};
use std::path::PathBuf;
use std::process::Command;

fn live_dir() -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("local/b12")
}

fn load_live(name: &str) -> Option<VerificationRun> {
    let raw = std::fs::read_to_string(live_dir().join(name)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn git_head() -> String {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git rev-parse");
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn b12_1_t5_embedded_sha_matches_git_head_when_present() {
    if let Some(sha) = helix_git_sha() {
        assert_eq!(sha, git_head());
        assert_eq!(sha.len(), 40);
        assert_ne!(sha, HELIXTEST_SHA);
    }
}

#[test]
fn b12_1_t6_helix_sha_is_not_spec_identity() {
    if let Some(sha) = helix_git_sha() {
        assert_ne!(sha, PINNED_COVERAGE_ID);
        assert_ne!(sha, PINNED_EXECUTION_ID);
        assert_ne!(sha, PINNED_BINDING_ID);
        let cov = coverage_id(
            &DRS_140_CONTRACT,
            &helix::checker::executed_checker_id(),
            Some(PINNED_BINDING_ID),
            Some(PINNED_PACK_INTEGRITY_SHA256),
        );
        assert_eq!(cov, PINNED_COVERAGE_ID);
        assert!(!cov.contains(sha));
    }
}

#[test]
fn b12_1_t7_empty_git_metadata_does_not_fabricate_sha() {
    assert_eq!(helix::provenance::parse_git_sha(""), None);
    assert_eq!(helix::provenance::parse_git_sha("unknown"), None);
}

#[test]
fn b12_1_t8_timestamp_path_host_are_absent_from_spec_ids() {
    assert_eq!(PINNED_COVERAGE_ID.len(), 64);
    assert_eq!(PINNED_EXECUTION_ID.len(), 64);
    assert!(!PINNED_COVERAGE_ID.contains("http"));
    assert!(!PINNED_EXECUTION_ID.contains("/Users/"));
    assert_eq!(DRS_140_CONTRACT.pack_id, DRS_140_PACK_ID);
    if let Some(sha) = helix_git_sha() {
        assert!(!sha.contains('/'));
        assert!(!sha.contains('@'));
    }
}

#[test]
fn b12_1_t1_forged_helix_git_sha_fails_integrity() {
    let Some(mut run) = load_live("bento.json") else {
        return;
    };
    validate_claim_integrity(&run).expect("pre-commit live JSON still validates");
    run.helix_git_sha = Some("0".repeat(40));
    let err = validate_claim_integrity(&run).unwrap_err().to_string();
    assert!(
        err.contains("helix_git_sha"),
        "expected provenance failure, got {err}"
    );
}

#[test]
fn b12_1_t2_changed_commit_fails_closed() {
    let Some(mut run) = load_live("starter-kit.json") else {
        return;
    };
    run.helix_git_sha = Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
    assert!(validate_claim_integrity(&run).is_err());
}

#[test]
fn b12_1_t3_precommit_artifact_labeled_as_other_commit_fails() {
    let Some(mut run) = load_live("bento.json") else {
        return;
    };
    assert!(
        run.helix_git_sha.is_none(),
        "existing B12 live JSON must remain a pre-commit observation"
    );
    assert!(!cites_this_verifier_build(&run));
    run.helix_git_sha = Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into());
    assert!(validate_claim_integrity(&run).is_err());
    assert!(!cites_this_verifier_build(&run));
}

#[test]
fn b12_1_t4_dirty_cannot_masquerade_as_clean() {
    let mut run = VerificationRun::new(helix::model::Target::new("http://127.0.0.1:9"));
    let built_dirty = helix_git_dirty();
    run.helix_git_sha = helix_git_sha().map(str::to_string);
    run.helix_git_dirty = Some(false);
    if built_dirty == Some(false) {
        run.helix_git_dirty = Some(true);
    }
    if helix_git_sha().is_some() && helix_git_dirty().is_some() {
        assert!(validate_claim_integrity(&run).is_err());
    }
    let mut clean_claim = VerificationRun::new(helix::model::Target::new("http://127.0.0.1:9"));
    clean_claim.helix_git_sha = None;
    clean_claim.helix_git_dirty = Some(false);
    assert!(validate_claim_integrity(&clean_claim).is_err());
}

#[test]
fn b12_1_live_artifacts_remain_precommit_observations() {
    let Some(sk) = load_live("starter-kit.json") else {
        return;
    };
    let Some(bento) = load_live("bento.json") else {
        return;
    };
    assert!(sk.helix_git_sha.is_none());
    assert!(bento.helix_git_sha.is_none());
    assert!(!cites_this_verifier_build(&sk));
    assert!(!cites_this_verifier_build(&bento));
    assert!(live_independent_observation(&sk));
    assert!(live_independent_observation(&bento));
    assert!(sk
        .standard_selection
        .as_ref()
        .unwrap()
        .verified_version
        .is_none());
    assert_eq!(
        bento
            .standard_selection
            .as_ref()
            .unwrap()
            .verified_version
            .as_deref(),
        Some("1.4.0")
    );
    assert_eq!(sk.coverage.as_ref().unwrap().state, CoverageState::Blocked);
    assert_eq!(
        bento.coverage.as_ref().unwrap().state,
        CoverageState::Partial
    );
    assert_eq!(
        sk.coverage.as_ref().unwrap().coverage_id.as_deref(),
        Some(PINNED_COVERAGE_ID)
    );
    assert_eq!(
        bento.coverage.as_ref().unwrap().coverage_id.as_deref(),
        Some(PINNED_COVERAGE_ID)
    );
}
