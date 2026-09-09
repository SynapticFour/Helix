// SPDX-License-Identifier: Apache-2.0
//! C2 — current live-evidence protocol.
//!
//! Standing is computed. Historical B12 is not restamped. Forged SHA/standing
//! cannot make an old artifact current. Not a coverage expansion. Not HELIOS.
//! B13 remains BLOCKED / DEFERRED.

use assert_cmd::Command;
use helix::claim_integrity::{finalize_run, validate_artifact_consistency};
use helix::claims::{evaluate, ClaimKind, ClaimStatus};
use helix::evidence::{classify_evidence, is_current_verification, EvidenceStanding};
use helix::live_evidence::{PINNED_COVERAGE_ID, PINNED_EXECUTION_ID};
use helix::model::{VerificationRun, VerificationStatus, SCHEMA_VERSION};
use helix::provenance::{helix_git_dirty, helix_git_sha};
use helix::report::{format_inspect_text, verify_json};
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use predicates::prelude::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

mod support;
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static C2_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const B12_STARTER_SHA256: &str = "5e8cc568f942b93af4e97f3f8da48b5ee58a833975f2e1495310d2106547c08b";
const B12_BENTO_SHA256: &str = "61d99c5e0da5a307e0e577fbdd1e951fa52b4aea6d5ce9cf8bf35f3ae4a5f6e6";

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn live_b12(name: &str) -> PathBuf {
    if let Ok(p) = std::env::var("HELIX_B12_LIVE_DIR") {
        return PathBuf::from(p).join(name);
    }
    root().join("local/b12").join(name)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn versioned(declared: DeclaredTarget) -> VerifyOptions {
    VerifyOptions {
        profile: helix::profile::ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: declared,
        ..Default::default()
    }
}

fn mock_declared(id: &str) -> DeclaredTarget {
    DeclaredTarget {
        target_id: Some(id.into()),
        kind: TargetKind::Mock,
        implementation_name: Some(id.into()),
        implementation_version: Some("0.0.0".into()),
        ..DeclaredTarget::default()
    }
}

async fn versioned_mock(id: &str) -> helix::verify::VerifyOutcome {
    let mock = start_mock_ga4gh_drs().await;
    verify_with_options(&mock.drs_url(), versioned(mock_declared(id)))
        .await
        .expect("versioned DRS 1.4.0")
}

fn restamp(run: &mut VerificationRun) {
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = None;
    }
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        r.verified_version = None;
    }
    run.claim_join = None;
    run.coverage = None;
    finalize_run(run);
}

fn load_b12(name: &str) -> Option<(Vec<u8>, VerificationRun)> {
    let bytes = std::fs::read(live_b12(name)).ok()?;
    let run = serde_json::from_slice(&bytes).ok()?;
    Some((bytes, run))
}

/// Historical but internally consistent: same observations, no Helix git stamp.
fn as_historical(mut run: VerificationRun) -> VerificationRun {
    run.helix_git_sha = None;
    run.helix_git_dirty = None;
    restamp(&mut run);
    run
}

#[tokio::test]
async fn c2_t1_current_verifier_evidence_is_classified() {
    let _g = C2_LOCK.lock().await;
    let Some(_) = helix_git_sha() else {
        return;
    };
    let run = versioned_mock("c2-t1").await.run;
    assert_eq!(
        classify_evidence(&run),
        EvidenceStanding::CurrentVerifierEvidence
    );
    assert!(is_current_verification(&run));
    let t = format_inspect_text(&run);
    assert!(t.contains("current_verifier_evidence"), "{t}");
    assert!(
        t.contains("current_verifier_evidence is not ga4gh_requirement VERIFIED")
            || t.contains("current_verifier_evidence is not VERIFIED"),
        "{t}"
    );
}

#[test]
fn c2_t2_historical_b12_remains_historical() {
    let Some((bytes, run)) = load_b12("starter-kit.json") else {
        return;
    };
    assert_eq!(sha256_hex(&bytes), B12_STARTER_SHA256);
    assert_eq!(
        classify_evidence(&run),
        EvidenceStanding::HistoricalObservation
    );
    assert!(!is_current_verification(&run));
    validate_artifact_consistency(&run).expect("historical B12 remains consistent");
}

#[test]
fn c2_t3_b12_bytes_remain_unchanged() {
    for (name, expected) in [
        ("starter-kit.json", B12_STARTER_SHA256),
        ("bento.json", B12_BENTO_SHA256),
    ] {
        let path = live_b12(name);
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(sha256_hex(&bytes), expected, "{name} must not be restamped");
        assert!(!std::str::from_utf8(&bytes)
            .unwrap()
            .contains("helix_git_sha"));
    }
}

#[tokio::test]
async fn c2_t4_inspect_does_not_rewrite() {
    let _g = C2_LOCK.lock().await;
    let run = versioned_mock("c2-t4").await.run;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    std::fs::write(&path, verify_json(&run).unwrap()).unwrap();
    let before = sha256_hex(&std::fs::read(&path).unwrap());
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .success();
    let after = sha256_hex(&std::fs::read(&path).unwrap());
    assert_eq!(before, after);

    if live_b12("bento.json").is_file() {
        let p = live_b12("bento.json");
        let before = sha256_hex(&std::fs::read(&p).unwrap());
        helix()
            .env("NO_COLOR", "1")
            .args(["inspect", p.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("historical_observation"));
        let after = sha256_hex(&std::fs::read(&p).unwrap());
        assert_eq!(before, after);
        assert_eq!(after, B12_BENTO_SHA256);
    }
}

#[tokio::test]
async fn c2_t5_forged_standing_cannot_make_evidence_current() {
    let _g = C2_LOCK.lock().await;
    let hist = as_historical(versioned_mock("c2-t5").await.run);
    assert_eq!(
        classify_evidence(&hist),
        EvidenceStanding::HistoricalObservation
    );
    let mut v: Value = serde_json::from_str(&verify_json(&hist).unwrap()).unwrap();
    v["standing"] = Value::String("current_verifier_evidence".into());
    v["current_verifier_evidence"] = Value::Bool(true);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forged.json");
    std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    let loaded: VerificationRun = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        classify_evidence(&loaded),
        EvidenceStanding::HistoricalObservation
    );
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("historical_observation"))
        .stdout(predicate::str::contains(
            "\"current_verifier_evidence\": false",
        ));
}

#[tokio::test]
async fn c2_t6_forged_sha_cannot_make_old_evidence_current() {
    let _g = C2_LOCK.lock().await;
    let Some(current_sha) = helix_git_sha() else {
        return;
    };
    let mut hist = as_historical(versioned_mock("c2-t6").await.run);
    assert_eq!(
        classify_evidence(&hist),
        EvidenceStanding::HistoricalObservation
    );
    hist.helix_git_sha = Some(current_sha.to_string());
    hist.helix_git_dirty = helix_git_dirty();
    assert_eq!(classify_evidence(&hist), EvidenceStanding::Invalid);
    assert!(!is_current_verification(&hist));
    assert!(validate_artifact_consistency(&hist).is_err());
}

#[tokio::test]
async fn c2_t7_observation_mutation_is_detected() {
    let _g = C2_LOCK.lock().await;
    let mut run = versioned_mock("c2-t7").await.run;
    let mut flipped = false;
    for r in run.executed.iter_mut() {
        if r.status == VerificationStatus::Pass {
            r.status = VerificationStatus::Fail;
            flipped = true;
            break;
        }
    }
    assert!(flipped);
    assert_eq!(classify_evidence(&run), EvidenceStanding::Invalid);
}

#[tokio::test]
async fn c2_t8_current_evidence_does_not_alter_execution_id() {
    let _g = C2_LOCK.lock().await;
    let run = versioned_mock("c2-t8").await.run;
    assert_eq!(
        run.standard_selection
            .as_ref()
            .and_then(|s| s.execution_id.as_deref()),
        Some(PINNED_EXECUTION_ID)
    );
}

#[tokio::test]
async fn c2_t9_current_evidence_does_not_alter_coverage_id() {
    let _g = C2_LOCK.lock().await;
    let run = versioned_mock("c2-t9").await.run;
    assert_eq!(
        run.coverage.as_ref().and_then(|c| c.coverage_id.as_deref()),
        Some(PINNED_COVERAGE_ID)
    );
}

#[tokio::test]
async fn c2_t10_current_evidence_does_not_imply_verified() {
    let _g = C2_LOCK.lock().await;
    let Some(_) = helix_git_sha() else {
        return;
    };
    let mut run = versioned_mock("c2-t10").await.run;
    // Mock catalog VERIFIED is possible; force a FAIL so standing and claim stay distinct.
    for r in run.executed.iter_mut() {
        if r.id == "drs.object.schema" {
            r.status = VerificationStatus::Fail;
        }
    }
    restamp(&mut run);
    assert_eq!(
        classify_evidence(&run),
        EvidenceStanding::CurrentVerifierEvidence
    );
    assert_eq!(
        evaluate(&run).get(ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[tokio::test]
async fn c2_t11_authorization_remains_unevaluated() {
    let _g = C2_LOCK.lock().await;
    let run = versioned_mock("c2-t11").await.run;
    let json: Value = serde_json::from_str(&verify_json(&run).unwrap()).unwrap();
    let uneval = json["coverage"]["unevaluated"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    assert!(uneval.contains(&"drs.security.authorization"));
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
}

#[tokio::test]
async fn c2_t12_c1_workflow_remains_offline_testable() {
    let _g = C2_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verify.json");
    helix()
        .env("NO_COLOR", "1")
        .env("RUST_LOG", "error")
        .args([
            "verify",
            &mock.drs_url(),
            "--standard",
            "drs",
            "--version",
            "1.4.0",
            "--output",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    helix()
        .env("NO_COLOR", "1")
        .args(["inspect", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("HELIX EVIDENCE INSPECT"));
    let script = std::fs::read_to_string(root().join("scripts/verify-independent-drs.sh")).unwrap();
    assert!(script.contains("starter-kit-current.json"));
    assert!(script.contains("local/b12"));
    assert!(!script.contains("docker pull"));
}

#[tokio::test]
async fn c2_consistent_restamp_is_unsigned_and_not_helios() {
    let _g = C2_LOCK.lock().await;
    let Some(current_sha) = helix_git_sha() else {
        return;
    };
    let mut hist = as_historical(versioned_mock("c2-f").await.run);
    hist.helix_git_sha = Some(current_sha.to_string());
    hist.helix_git_dirty = helix_git_dirty();
    restamp(&mut hist);
    assert_eq!(
        classify_evidence(&hist),
        EvidenceStanding::CurrentVerifierEvidence,
        "unsigned consistent restamp can look current; HELIOS signing is the stronger bound"
    );
}

#[tokio::test]
async fn c2_output_schema_is_still_v1() {
    let _g = C2_LOCK.lock().await;
    let run = versioned_mock("c2-schema").await.run;
    let v: Value = serde_json::from_str(&verify_json(&run).unwrap()).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
    assert!(v.get("standing").is_none());
    if helix_git_sha().is_some() {
        assert!(v["claim_join"]["helix_git_sha"].as_str().is_some());
    }
}
