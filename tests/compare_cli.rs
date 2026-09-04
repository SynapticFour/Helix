// SPDX-License-Identifier: Apache-2.0
//! `helix compare` CLI. Regression is PASS→FAIL at stable id, not a score drop.

use assert_cmd::Command;
use helix::compare::{compare_runs, CompareKind};
use helix::identity;
use helix::model::{
    Target, VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
};
use predicates::prelude::*;
use serde_json::Value;
use std::path::PathBuf;

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn check(id: &str) -> VerificationCheck {
    VerificationCheck::from_spec(identity::spec(id)).with_profile("generic")
}

fn push(run: &mut VerificationRun, id: &str, status: VerificationStatus) {
    let c = check(id);
    match status {
        VerificationStatus::Pass => run.push_executed(VerificationResult::pass(c)),
        VerificationStatus::Fail => run.push_executed(VerificationResult::fail(c, "x")),
        VerificationStatus::Error => run.push_executed(VerificationResult::error(c, "x")),
        VerificationStatus::Skip => run.push_skipped(VerificationResult::skip(c, "x")),
    }
}

fn run_of(pairs: &[(&str, VerificationStatus)]) -> VerificationRun {
    let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
    run.timestamp = "2026-09-04T12:00:00Z".into();
    for (id, status) in pairs {
        push(&mut run, id, *status);
    }
    run
}

fn write_run(label: &str, run: &VerificationRun) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "helix-compare-{}-{}-{}.json",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, serde_json::to_string_pretty(run).unwrap()).unwrap();
    path
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_str(String::from_utf8_lossy(stdout).trim()).unwrap()
}

const NOT_FOUND: &str = "drs.object.not_found";
const REACHABLE: &str = "drs.object.reachable";
const SCATTER: &str = "wes.run.scatter_gather";

#[test]
fn missing_paths_is_usage_exit_2() {
    helix()
        .args(["compare"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn unknown_format_is_usage_exit_2() {
    helix()
        .args(["compare", "a.json", "b.json", "--format", "yaml"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn missing_file_exits_1() {
    helix()
        .args(["compare", "/no/such/previous.json", "/no/such/current.json"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn pass_to_fail_exits_1_with_new_fail_json() {
    let prev = write_run(
        "prev-pass",
        &run_of(&[(NOT_FOUND, VerificationStatus::Pass)]),
    );
    let curr = write_run(
        "curr-fail",
        &run_of(&[(NOT_FOUND, VerificationStatus::Fail)]),
    );
    let out = helix()
        .env("RUST_LOG", "error")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let v = parse_json(&out.stdout);
    assert_eq!(v["has_regression"], true);
    assert_eq!(v["summary"]["new_fail"], 1);
    assert_eq!(v["rows"][0]["id"].as_str(), Some(NOT_FOUND));
    assert_eq!(v["rows"][0]["kind"].as_str(), Some("NEW_FAIL"));
    assert_eq!(v["rows"][0]["regression"], true);
    assert_eq!(v["rows"][0]["previous"].as_str(), Some("pass"));
    assert_eq!(v["rows"][0]["current"].as_str(), Some("fail"));
    assert!(v.get("overall_score").is_none());
    assert!(v.get("signature").is_none());
    assert_eq!(v["same_measurement"], true);
    assert!(v.get("previous_identity").is_some());
    assert_eq!(
        v["previous_identity"]["fixture_version"].as_str(),
        Some("helix-fixtures-v1")
    );
}

#[test]
fn fail_to_fail_exits_0_existing_failure() {
    let prev = write_run(
        "prev-fail",
        &run_of(&[(NOT_FOUND, VerificationStatus::Fail)]),
    );
    let curr = write_run(
        "curr-fail",
        &run_of(&[(NOT_FOUND, VerificationStatus::Fail)]),
    );
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "compare",
                prev.to_str().unwrap(),
                curr.to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(v["has_regression"], false);
    assert_eq!(v["rows"][0]["kind"].as_str(), Some("UNCHANGED_FAIL"));
    assert_eq!(v["rows"][0]["regression"], false);
}

#[test]
fn score_drop_from_skip_exits_0() {
    let prev = write_run(
        "prev-2pass",
        &run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Pass),
        ]),
    );
    let curr = write_run(
        "curr-1pass",
        &run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Skip),
        ]),
    );
    let v = parse_json(
        &helix()
            .env("RUST_LOG", "error")
            .args([
                "compare",
                prev.to_str().unwrap(),
                curr.to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout,
    );
    assert_eq!(v["has_regression"], false);
    assert_eq!(v["summary"]["new_skip"], 1);
    assert_eq!(v["summary"]["new_fail"], 0);
}

#[test]
fn skip_to_pass_is_fixed_skip_in_text_and_json() {
    let prev = write_run("prev-skip", &run_of(&[(SCATTER, VerificationStatus::Skip)]));
    let curr = write_run("curr-pass", &run_of(&[(SCATTER, VerificationStatus::Pass)]));
    let json = helix()
        .env("RUST_LOG", "error")
        .env("NO_COLOR", "1")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    let v = parse_json(&json.get_output().stdout);
    assert_eq!(v["rows"][0]["kind"].as_str(), Some("FIXED_SKIP"));
    assert_eq!(v["rows"][0]["skip_became_pass"], true);
    assert_ne!(v["rows"][0]["kind"].as_str(), Some("UNCHANGED_PASS"));

    let text = helix()
        .env("NO_COLOR", "1")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let t = String::from_utf8_lossy(&text);
    assert!(t.contains("FIXED_SKIP"));
    assert!(t.contains("SKIP must not silently become PASS"));
    assert!(t.contains("NO_NEW_REGRESSION"));
}

#[test]
fn report_json_alias_matches_format_json() {
    let prev = write_run("alias-a", &run_of(&[(REACHABLE, VerificationStatus::Pass)]));
    let curr = write_run("alias-b", &run_of(&[(REACHABLE, VerificationStatus::Pass)]));
    let a = helix()
        .env("RUST_LOG", "error")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    let b = helix()
        .env("RUST_LOG", "error")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--report",
            "json",
        ])
        .assert()
        .success();
    assert_eq!(
        parse_json(&a.get_output().stdout),
        parse_json(&b.get_output().stdout)
    );
}

#[test]
fn default_text_matches_format_text() {
    let prev = write_run("txt-a", &run_of(&[(REACHABLE, VerificationStatus::Pass)]));
    let curr = write_run("txt-b", &run_of(&[(REACHABLE, VerificationStatus::Pass)]));
    let default = helix()
        .env("NO_COLOR", "1")
        .args(["compare", prev.to_str().unwrap(), curr.to_str().unwrap()])
        .assert()
        .success();
    let explicit = helix()
        .env("NO_COLOR", "1")
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "text",
        ])
        .assert()
        .success();
    assert_eq!(default.get_output().stdout, explicit.get_output().stdout);
}

#[test]
fn overall_report_is_rejected() {
    let path = std::env::temp_dir().join(format!("helix-overall-{}.json", std::process::id()));
    std::fs::write(&path, r#"{"services":[],"enabled_services":[]}"#).unwrap();
    helix()
        .args(["compare", path.to_str().unwrap(), path.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("OverallReport"));
}

#[test]
fn json_stdout_has_no_ansi() {
    let prev = write_run("ansi-a", &run_of(&[(NOT_FOUND, VerificationStatus::Pass)]));
    let curr = write_run("ansi-b", &run_of(&[(NOT_FOUND, VerificationStatus::Fail)]));
    let out = helix()
        .args([
            "compare",
            prev.to_str().unwrap(),
            curr.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let raw = String::from_utf8_lossy(&out);
    assert!(!raw.contains('\u{1b}'));
    parse_json(&out);
}

#[test]
fn library_compare_matches_cli_kind() {
    let prev = run_of(&[(NOT_FOUND, VerificationStatus::Pass)]);
    let curr = run_of(&[(NOT_FOUND, VerificationStatus::Fail)]);
    let report = compare_runs(&prev, &curr).unwrap();
    assert_eq!(report.rows[0].kind, CompareKind::NewFail);
    assert_eq!(report.process_exit_code(), 1);
}
