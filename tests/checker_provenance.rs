// SPDX-License-Identifier: Apache-2.0
//! B7: executed checker identity is bound to compiled HelixTest sources.
//! VERSIONS.lock git SHA is a checkout pin, not proof of the checker that ran.
//! Not HELIOS. Not certification.

mod support;

use helix::fixture::DrsVerifyFixture;
use helix::profile::ProfileId;
use helix::standards::execution_id;
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

static JOIN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct ClearCheckerLie;

impl Drop for ClearCheckerLie {
    fn drop(&mut self) {
        helix::checker::set_lie_lock_checker_digest(None);
    }
}

fn versioned() -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: DeclaredTarget {
            target_id: Some("provenance-target".into()),
            kind: TargetKind::Mock,
            ..DeclaredTarget::default()
        },
        drs_fixture: DrsVerifyFixture::default_catalog(),
        ..Default::default()
    }
}

/// T1 — checker_id is the compile-time source digest, not VERSIONS.lock git SHA.
#[test]
fn t1_checker_id_is_executed_source_digest() {
    let id = helix::checker::executed_checker_id();
    let digest = helix::checker::executed_checker_source_sha256();
    assert_eq!(id, format!("helixtest-drs:{digest}"));
    assert_eq!(digest.len(), 64);
    assert_eq!(digest, framework::drs::executed_checker_source_sha256());
    assert_ne!(digest, helix::model::HELIXTEST_SHA);
    assert!(!id.contains(helix::model::HELIXTEST_SHA));
    assert_ne!(id, format!("v0.1.3:{}", helix::model::HELIXTEST_SHA));
}

/// T2 — stale lock metadata cannot masquerade as the executed checker.
#[tokio::test]
async fn t2_stale_lock_metadata_fails_verification() {
    let _g = JOIN_LOCK.lock().await;
    let _clear = ClearCheckerLie;
    helix::checker::set_lie_lock_checker_digest(Some(&"0".repeat(64)));
    let err = verify_with_options("http://127.0.0.1:9", versioned())
        .await
        .expect_err("stale lock must fail closed");
    let msg = err.to_string();
    assert!(msg.contains("checker identity mismatch"), "{msg}");
    assert!(
        msg.contains(&helix::checker::executed_checker_id()),
        "{msg}"
    );
    assert!(!msg.contains("v0.1.3:"), "{msg}");
}

/// T3/T4 — executed identity is on the run and survives JSON serialization.
#[tokio::test]
async fn t3_t4_result_and_json_preserve_executed_checker() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(&mock.drs_url(), versioned())
        .await
        .expect("verify");
    let want = helix::checker::executed_checker_id();
    let sel = outcome.run.standard_selection.as_ref().expect("selection");
    assert_eq!(sel.checker_id.as_deref(), Some(want.as_str()));
    assert_eq!(
        outcome.run.helixtest_sha.as_deref(),
        Some(helix::checker::executed_checker_source_sha256())
    );
    let v = serde_json::to_value(&outcome.run).expect("json");
    assert_eq!(
        v["standard_selection"]["checker_id"].as_str(),
        Some(want.as_str())
    );
    assert_eq!(
        v["helixtest_sha"].as_str(),
        Some(helix::checker::executed_checker_source_sha256())
    );
    assert_eq!(
        v["helixtest_git_sha"].as_str(),
        Some(helix::model::HELIXTEST_SHA)
    );
}

/// T5 — checker identity is an ingredient of execution_id; fixture is not.
#[test]
fn t5_checker_changes_execution_id_fixture_does_not() {
    let pack = "ga4gh.drs.1.4.0";
    let h = "ab".repeat(32);
    let a = execution_id(pack, &h, &h, &h, "helixtest-drs:aaa", "e", "c");
    let b = execution_id(pack, &h, &h, &h, "helixtest-drs:bbb", "e", "c");
    assert_ne!(a, b);
    let same_checker = helix::checker::executed_checker_id();
    let c = execution_id(pack, &h, &h, &h, &same_checker, "e", "c");
    let d = execution_id(pack, &h, &h, &h, &same_checker, "e", "c");
    assert_eq!(c, d);
}

/// T21 — download cap remains compiled into the executed checker.
#[test]
fn t21_checksum_download_limit_enforced() {
    assert_eq!(framework::drs::CHECKSUM_BODY_LIMIT, 2 * 1024 * 1024);
}

/// T22 — production verify/adapter/checker have no starter-kit branch.
#[test]
fn t22_no_hidden_starter_kit_branch() {
    let uuid = "b8cd0667-2c33-4c9f-967b-161b905932c9";
    for (name, src) in [
        ("verify.rs", include_str!("../src/verify.rs")),
        ("adapter/mod.rs", include_str!("../src/adapter/mod.rs")),
        ("checker.rs", include_str!("../src/checker.rs")),
        (
            "standards/support.rs",
            include_str!("../src/standards/support.rs"),
        ),
        (
            "standards/pack.rs",
            include_str!("../src/standards/pack.rs"),
        ),
    ] {
        assert!(
            !src.contains(uuid),
            "{name} must not hard-code the starter-kit object UUID"
        );
        assert!(
            !src.contains("ga4gh-starter-kit-drs"),
            "{name} must not branch on starter-kit image name"
        );
        assert!(
            !src.contains("127.0.0.1:4500"),
            "{name} must not hard-code starter-kit listen address"
        );
    }
}

/// T7/T20 — sibling git pin and recorded digest match the compiled checker.
#[test]
fn git_pin_and_source_digest_match_lock_and_sibling() {
    let lock = include_str!("../VERSIONS.lock");
    let git = lock
        .lines()
        .find_map(|l| l.strip_prefix("HELIXTEST_SHA="))
        .expect("HELIXTEST_SHA");
    let digest = lock
        .lines()
        .find_map(|l| l.strip_prefix("HELIXTEST_CHECKER_SOURCE_SHA256="))
        .expect("digest");
    assert_eq!(git, helix::model::HELIXTEST_SHA);
    assert_eq!(git.len(), 40);
    assert_ne!(digest, git);
    assert_eq!(digest, helix::checker::executed_checker_source_sha256());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../HelixTest");
    let head = std::process::Command::new("git")
        .args(["-C", root.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .expect("git");
    assert!(head.status.success());
    let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    assert_eq!(head, git);
    let porcelain = std::process::Command::new("git")
        .args(["-C", root.to_str().unwrap(), "status", "--porcelain"])
        .output()
        .expect("git status");
    assert!(
        String::from_utf8_lossy(&porcelain.stdout).trim().is_empty(),
        "HelixTest working tree must be clean for the pin"
    );
}

/// T6 — README is outside the DRS checker source closure.
#[test]
fn t6_readme_is_not_in_checker_closure() {
    let paths = framework::checker_identity::listed_paths();
    assert!(!paths.iter().any(|p| p.contains("README")));
    let root = framework::checker_identity::helixtest_root_from_framework_manifest();
    let base = framework::checker_identity::digest_from_helixtest_root(&root);
    assert_eq!(base, helix::checker::executed_checker_source_sha256());
}

/// T1–T5 via HelixTest hasher: relevant overrides change identity.
#[test]
fn relevant_closure_overrides_change_identity() {
    let root = framework::checker_identity::helixtest_root_from_framework_manifest();
    let base = framework::checker_identity::digest_from_helixtest_root(&root);
    for rel in [
        "crates/framework/src/drs.rs",
        "crates/framework/src/level0.rs",
        "crates/common/src/ga4gh_schemas.rs",
        "crates/common/src/spec_source.rs",
        "crates/common/src/http.rs",
        "crates/common/src/util.rs",
        "schemas/ga4gh/drs-openapi.yaml",
    ] {
        let mut b = std::fs::read(root.join(rel)).unwrap();
        b.extend_from_slice(b"\n");
        let changed = framework::checker_identity::digest_with_override(&root, rel, &b);
        assert_ne!(base, changed, "{rel}");
    }
}
