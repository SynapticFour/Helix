// SPDX-License-Identifier: Apache-2.0
//! Trust-model source scans and emit/load wiring. Prefer these over contributor
//! instructions. Map: docs/ARCHITECTURE_GUARDRAILS.md. Not HELIOS. Not certification.

use helix::compare::parse_verification_run;
use helix::guardrails::{check_run, CheckMode};
use helix::identity::spec;
use helix::model::{
    StandardSelection, VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
};
use helix::standards::validate_path;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn example_verify_json() -> Value {
    serde_json::from_str(include_str!("../docs/evaluator-pack/example-verify.json"))
        .expect("example-verify.json")
}

#[test]
fn cargo_toml_has_no_ferrum_or_helios_crate() {
    let toml = include_str!("../Cargo.toml");
    for line in toml.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        assert!(
            !(t.starts_with("ferrum ") || t.starts_with("ferrum=")),
            "Ferrum crate must not enter Helix Cargo.toml: {line}"
        );
        assert!(
            !t.contains("helios-audit") && !t.starts_with("helios ") && !t.starts_with("helios="),
            "HELIOS crate must not enter Helix Cargo.toml: {line}"
        );
    }
    assert!(
        toml.contains("helixtest-common") && toml.contains("helixtest-framework"),
        "generic verifier uses pinned HelixTest crates, not Ferrum"
    );
}

#[test]
fn src_must_not_use_mode_ferrum() {
    for path in rust_files(&repo_root().join("src")) {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("Mode::Ferrum"),
            "{} must not use Mode::Ferrum; profiles map to Features on Mode::Generic",
            path.display()
        );
    }
}

#[test]
fn src_must_not_fetch_standard_sources_from_the_network() {
    for path in rust_files(&repo_root().join("src")) {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("raw.githubusercontent.com"),
            "{} must not mention raw.githubusercontent.com (registry records a URL; Helix must not fetch it)",
            path.display()
        );
        assert!(
            !text.contains("ga4gh.github.io"),
            "{} must not fetch ga4gh.github.io spec pages",
            path.display()
        );
    }
}

#[test]
fn src_must_not_import_helios() {
    for path in rust_files(&repo_root().join("src")) {
        let text = std::fs::read_to_string(&path).unwrap();
        for needle in [
            "use helios",
            "extern crate helios",
            "helios-audit",
            "helios_audit",
        ] {
            assert!(
                !text.contains(needle),
                "{} must not import HELIOS ({needle})",
                path.display()
            );
        }
    }
}

#[test]
fn framework_imports_stay_in_the_adapter() {
    for path in rust_files(&repo_root().join("src")) {
        if path.components().any(|c| c.as_os_str() == "adapter") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("use framework::"),
            "{} must not import HelixTest framework; go through src/adapter",
            path.display()
        );
    }
}

#[test]
fn adapter_stays_on_mode_generic() {
    let adapter = include_str!("../src/adapter/mod.rs");
    assert!(adapter.contains("Mode::Generic"));
    assert!(!adapter.contains("Mode::Ferrum"));
    assert!(adapter.contains("Ferrum is not imported"));
}

#[test]
fn emit_and_load_paths_call_check_run() {
    let report = include_str!("../src/report.rs");
    assert!(
        report.contains("guardrails::check_run"),
        "helix verify must not print without check_run"
    );
    assert!(
        report.contains("check_serialized_claims"),
        "emitted claims[] must be checked against evaluate"
    );
    let bind = include_str!("../src/traceability.rs");
    assert!(
        bind.contains("guardrails::check_run"),
        "bind_run must fail closed through check_run"
    );
    let compare = include_str!("../src/compare.rs");
    assert!(
        compare.contains("forbid_helios_keys"),
        "compare must inspect raw JSON before serde drops unknown HELIOS keys"
    );
    assert!(
        compare.contains("CheckMode::Load"),
        "compare must load through check_run_with(Load)"
    );
}

#[test]
fn shipped_registry_still_validates() {
    validate_path(&helix::standards::default_registry_path()).expect("shipped registry");
}

#[test]
fn load_accepts_evaluator_example_without_traceability() {
    let raw = include_str!("../docs/evaluator-pack/example-verify.json");
    parse_verification_run(raw).expect("example JSON is a Load-mode run");
}

#[test]
fn load_rejects_helios_ro_crate() {
    let mut v = example_verify_json();
    v["ro_crate"] = serde_json::json!({"id": "no"});
    let err = parse_verification_run(&v.to_string())
        .unwrap_err()
        .to_string();
    assert!(err.contains("ro_crate"), "{err}");
}

#[test]
fn load_rejects_substituted_true() {
    let mut v = example_verify_json();
    v["standard_selection"] = serde_json::json!({
        "mode": "unversioned",
        "selection_status": "UNVERSIONED",
        "substituted": true,
        "selected_version": null,
        "verified_version": null
    });
    let err = parse_verification_run(&v.to_string())
        .unwrap_err()
        .to_string();
    assert!(err.contains("substituted"), "{err}");
}

#[test]
fn load_rejects_selected_ne_verified() {
    let mut v = example_verify_json();
    v["standard_selection"] = serde_json::json!({
        "mode": "explicit",
        "selection_status": "SELECTED",
        "substituted": false,
        "selected_version": "1.4.0",
        "verified_version": "1.5.0"
    });
    let err = parse_verification_run(&v.to_string())
        .unwrap_err()
        .to_string();
    assert!(err.contains("1.4.0") && err.contains("1.5.0"), "{err}");
}

#[test]
fn emit_rejects_fixture_serialized_as_normative() {
    let mut run = VerificationRun::new(helix::model::Target::new("http://127.0.0.1:9"));
    run.push_executed(VerificationResult::from_check(
        VerificationCheck::from_spec(spec("drs.object.reachable")),
        VerificationStatus::Pass,
    ));
    helix::traceability::bind_run(&mut run).expect("bind fixture run");
    let t = run.executed[0].traceability.as_mut().unwrap();
    t.category = helix::standards::BindingKind::Fixture;
    t.check_kind = helix::standards::BindingKind::Normative;
    t.claim_scope = helix::standards::ClaimScope::Ga4ghRequirement;
    let err = check_run(&run).unwrap_err().to_string();
    assert!(
        err.contains("cannot be serialized as a normative requirement"),
        "{err}"
    );
}

#[test]
fn emit_rejects_silent_substitution_flag() {
    let mut run = VerificationRun::new(helix::model::Target::new("http://127.0.0.1:9"));
    run.push_executed(VerificationResult::from_check(
        VerificationCheck::from_spec(spec("drs.object.reachable")),
        VerificationStatus::Pass,
    ));
    helix::traceability::bind_run(&mut run).expect("bind");
    run.standard_selection = Some(StandardSelection {
        substituted: true,
        ..StandardSelection::unversioned()
    });
    let err = check_run(&run).unwrap_err().to_string();
    assert!(err.contains("substituted"), "{err}");
}

#[test]
fn check_mode_names_are_stable() {
    assert_eq!(format!("{:?}", CheckMode::Emit), "Emit");
    assert_eq!(format!("{:?}", CheckMode::Load), "Load");
}
