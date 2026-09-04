// SPDX-License-Identifier: Apache-2.0
//! Verification regression at stable Helix `id` (not a score delta).
//!
//! A **regression** is a previously **passing** check that now **fails**
//! (`NEW_FAIL`). Fail→fail is `UNCHANGED_FAIL` (existing failure), not a
//! new regression. Skip must not silently become pass (`FIXED_SKIP`).
//!
//! Not HELIOS. Not certification. Not Ferrum-specific.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::{helix_version, VerificationResult, VerificationRun, VerificationStatus};
use crate::run_identity::{IdentityMismatch, RunIdentity};

/// Outcome of one stable-id comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompareKind {
    /// Previous pass, current fail or error. **This is a regression.**
    NewFail,
    /// Previous fail or error, current pass.
    Fixed,
    /// Previous fail or error, current fail or error. Existing failure, not a new regression.
    UnchangedFail,
    /// Previous pass, current pass.
    UnchangedPass,
    /// Previously executed (pass/fail/error), now skip or absent.
    NewSkip,
    /// Previously skip, now executed (pass/fail/error). Skip→pass is never `UNCHANGED_PASS`.
    FixedSkip,
    /// Skip→skip (or skip→absent). Not a regression.
    UnchangedSkip,
    /// Id only in current. Not `NEW_FAIL` even if current is fail (it never passed).
    Added,
}

impl CompareKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NewFail => "NEW_FAIL",
            Self::Fixed => "FIXED",
            Self::UnchangedFail => "UNCHANGED_FAIL",
            Self::UnchangedPass => "UNCHANGED_PASS",
            Self::NewSkip => "NEW_SKIP",
            Self::FixedSkip => "FIXED_SKIP",
            Self::UnchangedSkip => "UNCHANGED_SKIP",
            Self::Added => "ADDED",
        }
    }

    /// PASS → FAIL/ERROR only.
    pub fn is_regression(self) -> bool {
        matches!(self, Self::NewFail)
    }

    pub fn is_existing_failure(self) -> bool {
        matches!(self, Self::UnchangedFail)
    }
}

impl std::fmt::Display for CompareKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One row per Helix `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompareRow {
    pub id: String,
    pub code: String,
    pub kind: CompareKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<VerificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<VerificationStatus>,
    /// True only for [`CompareKind::NewFail`].
    pub regression: bool,
    /// Previous skip, current pass — must not be treated as a silent pass.
    pub skip_became_pass: bool,
}

/// Counts. Not a weighted score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompareSummary {
    pub new_fail: usize,
    pub fixed: usize,
    pub unchanged_fail: usize,
    pub unchanged_pass: usize,
    pub new_skip: usize,
    pub fixed_skip: usize,
    pub unchanged_skip: usize,
    pub added: usize,
    pub skip_became_pass: usize,
}

/// `helix compare` report. Not HelixTest `OverallReport`. Not HELIOS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompareReport {
    pub helix_version: String,
    pub previous_target: String,
    pub current_target: String,
    pub previous_identity: RunIdentity,
    pub current_identity: RunIdentity,
    pub identity_mismatches: Vec<IdentityMismatch>,
    /// Schema, profile, fixtures, target (and bench workload when present) match.
    pub same_measurement: bool,
    /// Helix or HelixTest pin differs. Still comparable by check id.
    pub suite_changed: bool,
    pub has_regression: bool,
    pub summary: CompareSummary,
    pub rows: Vec<CompareRow>,
}

impl CompareReport {
    /// Exit 1 when any `NEW_FAIL`. Existing failures do not change this.
    pub fn process_exit_code(&self) -> i32 {
        if self.has_regression {
            1
        } else {
            0
        }
    }
}

/// Classify a previous/current status pair. `None` = id absent from that run.
pub fn classify(
    previous: Option<VerificationStatus>,
    current: Option<VerificationStatus>,
) -> CompareKind {
    use VerificationStatus::{Error, Fail, Pass, Skip};
    match (previous, current) {
        (Some(Pass), Some(Pass)) => CompareKind::UnchangedPass,
        (Some(Pass), Some(Fail | Error)) => CompareKind::NewFail,
        (Some(Pass), Some(Skip) | None) => CompareKind::NewSkip,
        (Some(Fail | Error), Some(Pass)) => CompareKind::Fixed,
        (Some(Fail | Error), Some(Fail | Error)) => CompareKind::UnchangedFail,
        (Some(Fail | Error), Some(Skip) | None) => CompareKind::NewSkip,
        (Some(Skip), Some(Pass | Fail | Error)) => CompareKind::FixedSkip,
        (Some(Skip), Some(Skip) | None) => CompareKind::UnchangedSkip,
        (None, Some(_)) => CompareKind::Added,
        (None, None) => CompareKind::Added,
    }
}

pub fn compare_runs(
    previous: &VerificationRun,
    current: &VerificationRun,
) -> Result<CompareReport> {
    let prev_map = index_by_id(previous).context("previous verification run")?;
    let curr_map = index_by_id(current).context("current verification run")?;

    let mut ids: Vec<String> = prev_map.keys().chain(curr_map.keys()).cloned().collect();
    ids.sort();
    ids.dedup();

    let mut rows = Vec::with_capacity(ids.len());
    let mut summary = CompareSummary::default();

    for id in ids {
        let prev = prev_map.get(&id).copied();
        let curr = curr_map.get(&id).copied();
        let previous_status = prev.map(|r| r.status);
        let current_status = curr.map(|r| r.status);
        let kind = classify(previous_status, current_status);
        let skip_became_pass = matches!(previous_status, Some(VerificationStatus::Skip))
            && matches!(current_status, Some(VerificationStatus::Pass));
        let code = curr.or(prev).map(|r| r.code.clone()).unwrap_or_default();
        let row = CompareRow {
            id,
            code,
            kind,
            previous: previous_status,
            current: current_status,
            regression: kind.is_regression(),
            skip_became_pass,
        };
        bump_summary(&mut summary, &row);
        rows.push(row);
    }

    let prev_id = RunIdentity::from_verify(previous);
    let curr_id = RunIdentity::from_verify(current);
    let identity_mismatches = prev_id.mismatches(&curr_id);
    let same_measurement = prev_id.same_measurement(&curr_id);
    let suite_changed = prev_id.suite_changed(&curr_id);

    Ok(CompareReport {
        helix_version: helix_version().to_string(),
        previous_target: previous.target.url.clone(),
        current_target: current.target.url.clone(),
        previous_identity: prev_id,
        current_identity: curr_id,
        identity_mismatches,
        same_measurement,
        suite_changed,
        has_regression: summary.new_fail > 0,
        summary,
        rows,
    })
}

pub fn load_verification_run(path: &Path) -> Result<VerificationRun> {
    let raw =
        crate::http_safety::read_to_string_capped(path, crate::http_safety::MAX_COMPARE_FILE_BYTES)
            .with_context(|| format!("read {}", path.display()))?;
    parse_verification_run(&raw)
        .map_err(|e| anyhow::anyhow!("{}", crate::redact::redact_text(&format!("{e:#}"))))
        .with_context(|| format!("parse {}", path.display()))
}

pub fn parse_verification_run(raw: &str) -> Result<VerificationRun> {
    let value: serde_json::Value = serde_json::from_str(raw).context("invalid JSON")?;
    if value.get("services").is_some() && value.get("executed").is_none() {
        bail!(
            "JSON looks like HelixTest OverallReport (`services` without `executed`); \
             helix compare needs helix verify --format json (VerificationRun)"
        );
    }
    if value.get("passed").is_some() && value.get("executed").is_none() {
        bail!("JSON is not a Helix VerificationRun (has `passed`, no `executed`)");
    }
    serde_json::from_value(value).context("JSON is not a Helix VerificationRun")
}

pub fn compare_files(previous: &Path, current: &Path) -> Result<CompareReport> {
    let prev = load_verification_run(previous)?;
    let curr = load_verification_run(current)?;
    compare_runs(&prev, &curr)
}

fn index_by_id(run: &VerificationRun) -> Result<BTreeMap<String, &VerificationResult>> {
    let mut map = BTreeMap::new();
    for r in run.executed.iter().chain(run.skipped.iter()) {
        if r.id.is_empty() {
            bail!("verification result missing stable id");
        }
        if map.insert(r.id.clone(), r).is_some() {
            bail!("duplicate check id `{}` (compare is per stable id)", r.id);
        }
    }
    Ok(map)
}

fn bump_summary(summary: &mut CompareSummary, row: &CompareRow) {
    match row.kind {
        CompareKind::NewFail => summary.new_fail += 1,
        CompareKind::Fixed => summary.fixed += 1,
        CompareKind::UnchangedFail => summary.unchanged_fail += 1,
        CompareKind::UnchangedPass => summary.unchanged_pass += 1,
        CompareKind::NewSkip => summary.new_skip += 1,
        CompareKind::FixedSkip => summary.fixed_skip += 1,
        CompareKind::UnchangedSkip => summary.unchanged_skip += 1,
        CompareKind::Added => summary.added += 1,
    }
    if row.skip_became_pass {
        summary.skip_became_pass += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity;
    use crate::model::{Target, VerificationCheck, VerificationResult, VerificationRun};

    fn check(id: &str) -> VerificationCheck {
        VerificationCheck::from_spec(identity::spec(id)).with_profile("generic")
    }

    fn push(run: &mut VerificationRun, id: &str, status: VerificationStatus) {
        let c = check(id);
        match status {
            VerificationStatus::Pass => run.push_executed(VerificationResult::pass(c)),
            VerificationStatus::Fail => {
                run.push_executed(VerificationResult::fail(c, "assertion failed"))
            }
            VerificationStatus::Error => {
                run.push_executed(VerificationResult::error(c, "unreachable"))
            }
            VerificationStatus::Skip => {
                run.push_skipped(VerificationResult::skip(c, "not detected"))
            }
        }
    }

    fn run_named(url: &str, pairs: &[(&str, VerificationStatus)]) -> VerificationRun {
        let mut run = VerificationRun::new(Target::new(url));
        for (id, status) in pairs {
            push(&mut run, id, *status);
        }
        run
    }

    fn run_of(pairs: &[(&str, VerificationStatus)]) -> VerificationRun {
        run_named("http://127.0.0.1:9", pairs)
    }

    fn kind_for(
        prev: &[(&str, VerificationStatus)],
        curr: &[(&str, VerificationStatus)],
        id: &str,
    ) -> CompareKind {
        let report = compare_runs(&run_of(prev), &run_of(curr)).unwrap();
        report
            .rows
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("missing id {id} in {report:?}"))
            .kind
    }

    const NOT_FOUND: &str = "drs.object.not_found";
    const REACHABLE: &str = "drs.object.reachable";
    const SCHEMA: &str = "drs.object.schema";
    const SCATTER: &str = "wes.run.scatter_gather";

    #[test]
    fn pass_to_fail_is_new_fail_regression() {
        let prev = run_of(&[(NOT_FOUND, VerificationStatus::Pass)]);
        let curr = run_of(&[(NOT_FOUND, VerificationStatus::Fail)]);
        let report = compare_runs(&prev, &curr).unwrap();
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].id, NOT_FOUND);
        assert_eq!(report.rows[0].kind, CompareKind::NewFail);
        assert_eq!(report.rows[0].kind.as_str(), "NEW_FAIL");
        assert!(report.rows[0].regression);
        assert!(report.has_regression);
        assert_eq!(report.summary.new_fail, 1);
        assert_eq!(report.process_exit_code(), 1);
        assert_eq!(report.rows[0].previous, Some(VerificationStatus::Pass));
        assert_eq!(report.rows[0].current, Some(VerificationStatus::Fail));
    }

    #[test]
    fn fail_to_fail_is_unchanged_fail_not_regression() {
        let prev = run_of(&[(NOT_FOUND, VerificationStatus::Fail)]);
        let curr = run_of(&[(NOT_FOUND, VerificationStatus::Fail)]);
        let report = compare_runs(&prev, &curr).unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::UnchangedFail);
        assert!(!report.rows[0].regression);
        assert!(!report.has_regression);
        assert_eq!(report.summary.unchanged_fail, 1);
        assert_eq!(report.summary.new_fail, 0);
        assert_eq!(report.process_exit_code(), 0);
        assert!(report.rows[0].kind.is_existing_failure());
    }

    #[test]
    fn pass_to_pass_is_unchanged_pass() {
        assert_eq!(
            kind_for(
                &[(REACHABLE, VerificationStatus::Pass)],
                &[(REACHABLE, VerificationStatus::Pass)],
                REACHABLE
            ),
            CompareKind::UnchangedPass
        );
    }

    #[test]
    fn fail_to_pass_is_fixed_not_regression() {
        let report = compare_runs(
            &run_of(&[(SCHEMA, VerificationStatus::Fail)]),
            &run_of(&[(SCHEMA, VerificationStatus::Pass)]),
        )
        .unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::Fixed);
        assert!(!report.has_regression);
        assert_eq!(report.process_exit_code(), 0);
    }

    #[test]
    fn pass_to_skip_is_new_skip_not_regression() {
        let report = compare_runs(
            &run_of(&[(SCATTER, VerificationStatus::Pass)]),
            &run_of(&[(SCATTER, VerificationStatus::Skip)]),
        )
        .unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::NewSkip);
        assert!(!report.has_regression);
        assert_eq!(report.process_exit_code(), 0);
    }

    #[test]
    fn skip_to_pass_is_fixed_skip_never_unchanged_pass() {
        let report = compare_runs(
            &run_of(&[(SCATTER, VerificationStatus::Skip)]),
            &run_of(&[(SCATTER, VerificationStatus::Pass)]),
        )
        .unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::FixedSkip);
        assert_ne!(report.rows[0].kind, CompareKind::UnchangedPass);
        assert!(report.rows[0].skip_became_pass);
        assert_eq!(report.summary.skip_became_pass, 1);
        assert_eq!(report.summary.unchanged_pass, 0);
        assert!(!report.has_regression);
        assert_eq!(report.process_exit_code(), 0);
    }

    #[test]
    fn skip_to_fail_is_fixed_skip_not_new_fail() {
        let report = compare_runs(
            &run_of(&[(SCATTER, VerificationStatus::Skip)]),
            &run_of(&[(SCATTER, VerificationStatus::Fail)]),
        )
        .unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::FixedSkip);
        assert!(!report.rows[0].regression);
        assert!(!report.has_regression);
        assert!(!report.rows[0].skip_became_pass);
    }

    #[test]
    fn skip_to_skip_is_unchanged_skip() {
        assert_eq!(
            kind_for(
                &[(SCATTER, VerificationStatus::Skip)],
                &[(SCATTER, VerificationStatus::Skip)],
                SCATTER
            ),
            CompareKind::UnchangedSkip
        );
    }

    #[test]
    fn pass_to_error_is_new_fail() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Pass),
                Some(VerificationStatus::Error)
            ),
            CompareKind::NewFail
        );
    }

    #[test]
    fn error_to_pass_is_fixed() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Error),
                Some(VerificationStatus::Pass)
            ),
            CompareKind::Fixed
        );
    }

    #[test]
    fn error_to_error_is_unchanged_fail() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Error),
                Some(VerificationStatus::Error)
            ),
            CompareKind::UnchangedFail
        );
    }

    #[test]
    fn fail_to_error_is_unchanged_fail_not_new_regression() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Fail),
                Some(VerificationStatus::Error)
            ),
            CompareKind::UnchangedFail
        );
    }

    #[test]
    fn error_to_fail_is_unchanged_fail() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Error),
                Some(VerificationStatus::Fail)
            ),
            CompareKind::UnchangedFail
        );
    }

    #[test]
    fn fail_to_skip_is_new_skip() {
        assert_eq!(
            classify(
                Some(VerificationStatus::Fail),
                Some(VerificationStatus::Skip)
            ),
            CompareKind::NewSkip
        );
    }

    #[test]
    fn score_drop_from_new_skip_is_not_a_regression() {
        let prev = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Pass),
        ]);
        let curr = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Skip),
        ]);
        assert_eq!(prev.summary.passed, 2);
        assert_eq!(curr.summary.passed, 1);
        let report = compare_runs(&prev, &curr).unwrap();
        assert!(
            !report.has_regression,
            "passed count drop is not a regression"
        );
        assert_eq!(report.summary.new_fail, 0);
        assert_eq!(report.summary.new_skip, 1);
        assert_eq!(report.process_exit_code(), 0);
    }

    #[test]
    fn existing_fail_plus_new_fail_is_regression_on_the_pass_to_fail_id_only() {
        let prev = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (SCHEMA, VerificationStatus::Fail),
        ]);
        let curr = run_of(&[
            (REACHABLE, VerificationStatus::Fail),
            (SCHEMA, VerificationStatus::Fail),
        ]);
        let report = compare_runs(&prev, &curr).unwrap();
        assert!(report.has_regression);
        let by_id: BTreeMap<_, _> = report
            .rows
            .iter()
            .map(|r| (r.id.as_str(), r.kind))
            .collect();
        assert_eq!(by_id[REACHABLE], CompareKind::NewFail);
        assert_eq!(by_id[SCHEMA], CompareKind::UnchangedFail);
        assert_eq!(report.summary.new_fail, 1);
        assert_eq!(report.summary.unchanged_fail, 1);
    }

    #[test]
    fn added_failing_id_is_added_not_new_fail() {
        let prev = run_of(&[(REACHABLE, VerificationStatus::Pass)]);
        let curr = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Fail),
        ]);
        let report = compare_runs(&prev, &curr).unwrap();
        let row = report.rows.iter().find(|r| r.id == NOT_FOUND).unwrap();
        assert_eq!(row.kind, CompareKind::Added);
        assert!(!row.regression);
        assert!(!report.has_regression);
    }

    #[test]
    fn added_pass_is_added_not_fixed_or_unchanged_pass() {
        let prev = run_of(&[(REACHABLE, VerificationStatus::Pass)]);
        let curr = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Pass),
        ]);
        let report = compare_runs(&prev, &curr).unwrap();
        let row = report.rows.iter().find(|r| r.id == NOT_FOUND).unwrap();
        assert_eq!(row.kind, CompareKind::Added);
        assert_ne!(row.kind, CompareKind::Fixed);
        assert_ne!(row.kind, CompareKind::UnchangedPass);
    }

    #[test]
    fn removed_pass_is_new_skip() {
        let prev = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Pass),
        ]);
        let curr = run_of(&[(REACHABLE, VerificationStatus::Pass)]);
        let report = compare_runs(&prev, &curr).unwrap();
        let row = report.rows.iter().find(|r| r.id == NOT_FOUND).unwrap();
        assert_eq!(row.kind, CompareKind::NewSkip);
        assert_eq!(row.current, None);
        assert!(!report.has_regression);
    }

    #[test]
    fn removed_fail_is_new_skip_not_fixed() {
        let prev = run_of(&[(NOT_FOUND, VerificationStatus::Fail)]);
        let curr = run_of(&[]);
        let report = compare_runs(&prev, &curr).unwrap();
        assert_eq!(report.rows[0].kind, CompareKind::NewSkip);
        assert_ne!(report.rows[0].kind, CompareKind::Fixed);
    }

    #[test]
    fn duplicate_id_in_one_run_is_an_error() {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        push(&mut run, NOT_FOUND, VerificationStatus::Pass);
        run.executed
            .push(VerificationResult::fail(check(NOT_FOUND), "dup"));
        let result = compare_runs(&run, &run_of(&[(NOT_FOUND, VerificationStatus::Pass)]));
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("expected duplicate-id error"),
        };
        let msg = format!("{err:#}");
        assert!(msg.contains("duplicate check id"), "{msg}");
    }

    #[test]
    fn empty_runs_have_no_regression() {
        let report = compare_runs(&run_of(&[]), &run_of(&[])).unwrap();
        assert!(report.rows.is_empty());
        assert!(!report.has_regression);
        assert_eq!(report.process_exit_code(), 0);
    }

    #[test]
    fn rows_are_sorted_by_id() {
        let prev = run_of(&[
            (SCHEMA, VerificationStatus::Pass),
            (REACHABLE, VerificationStatus::Pass),
        ]);
        let curr = prev.clone();
        let report = compare_runs(&prev, &curr).unwrap();
        let ids: Vec<_> = report.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec![REACHABLE, SCHEMA]);
    }

    #[test]
    fn json_kinds_are_screaming_snake() {
        let report = compare_runs(
            &run_of(&[(NOT_FOUND, VerificationStatus::Pass)]),
            &run_of(&[(NOT_FOUND, VerificationStatus::Fail)]),
        )
        .unwrap();
        let v = serde_json::to_value(&report).unwrap();
        assert_eq!(v["rows"][0]["kind"].as_str(), Some("NEW_FAIL"));
        assert_eq!(v["has_regression"], true);
        assert!(v.get("overall_score").is_none());
        assert!(v.get("signature").is_none());
        assert!(v.get("ro_crate").is_none());
        assert!(v.get("previous_identity").is_some());
        assert_eq!(v["same_measurement"], true);
    }

    #[test]
    fn parse_rejects_overall_report() {
        let raw = r#"{"services":[],"enabled_services":[]}"#;
        let err = parse_verification_run(raw).unwrap_err();
        assert!(err.to_string().contains("OverallReport"), "{err}");
    }

    #[test]
    fn parse_roundtrip_verification_run() {
        let run = run_of(&[(NOT_FOUND, VerificationStatus::Pass)]);
        let json = serde_json::to_string_pretty(&run).unwrap();
        let loaded = parse_verification_run(&json).unwrap();
        assert_eq!(loaded.executed[0].id, NOT_FOUND);
        assert_eq!(loaded.executed[0].status, VerificationStatus::Pass);
    }

    #[test]
    fn classify_table_covers_six_named_kinds() {
        use CompareKind::*;
        use VerificationStatus::*;
        assert_eq!(classify(Some(Pass), Some(Fail)), NewFail);
        assert_eq!(classify(Some(Fail), Some(Pass)), Fixed);
        assert_eq!(classify(Some(Fail), Some(Fail)), UnchangedFail);
        assert_eq!(classify(Some(Pass), Some(Pass)), UnchangedPass);
        assert_eq!(classify(Some(Pass), Some(Skip)), NewSkip);
        assert_eq!(classify(Some(Skip), Some(Pass)), FixedSkip);
        assert_eq!(classify(Some(Skip), Some(Skip)), UnchangedSkip);
        assert_eq!(classify(None, Some(Pass)), Added);
    }

    #[test]
    fn compare_is_by_id_not_position() {
        let prev = run_of(&[
            (REACHABLE, VerificationStatus::Pass),
            (NOT_FOUND, VerificationStatus::Fail),
        ]);
        let curr = run_of(&[
            (NOT_FOUND, VerificationStatus::Fail),
            (REACHABLE, VerificationStatus::Pass),
        ]);
        let report = compare_runs(&prev, &curr).unwrap();
        assert_eq!(report.summary.unchanged_pass, 1);
        assert_eq!(report.summary.unchanged_fail, 1);
        assert_eq!(report.summary.new_fail, 0);
    }

    #[test]
    fn skip_became_pass_never_increments_unchanged_pass() {
        let report = compare_runs(
            &run_of(&[
                (REACHABLE, VerificationStatus::Pass),
                (SCATTER, VerificationStatus::Skip),
            ]),
            &run_of(&[
                (REACHABLE, VerificationStatus::Pass),
                (SCATTER, VerificationStatus::Pass),
            ]),
        )
        .unwrap();
        assert_eq!(report.summary.unchanged_pass, 1);
        assert_eq!(report.summary.fixed_skip, 1);
        assert_eq!(report.summary.skip_became_pass, 1);
    }

    #[test]
    fn targets_are_recorded() {
        let prev = run_named(
            "http://old.example",
            &[(REACHABLE, VerificationStatus::Pass)],
        );
        let curr = run_named(
            "http://new.example",
            &[(REACHABLE, VerificationStatus::Pass)],
        );
        let report = compare_runs(&prev, &curr).unwrap();
        assert_eq!(report.previous_target, "http://old.example");
        assert_eq!(report.current_target, "http://new.example");
        assert!(!report.same_measurement);
        assert!(report
            .identity_mismatches
            .iter()
            .any(|m| m.field == "target"));
    }

    #[test]
    fn identity_mismatch_is_not_a_regression() {
        let prev = run_named(
            "http://old.example",
            &[(REACHABLE, VerificationStatus::Pass)],
        );
        let curr = run_named(
            "http://new.example",
            &[(REACHABLE, VerificationStatus::Pass)],
        );
        let report = compare_runs(&prev, &curr).unwrap();
        assert!(!report.same_measurement);
        assert!(!report.has_regression);
        assert_eq!(report.process_exit_code(), 0);
        assert!(report.previous_identity.workload_id.is_none());
    }

    #[test]
    fn compare_file_oversize_is_rejected_without_dumping() {
        let p = std::env::temp_dir().join(format!(
            "helix-compare-oversize-{}.json",
            std::process::id()
        ));
        let blob = vec![b'{'; (crate::http_safety::MAX_COMPARE_FILE_BYTES as usize) + 1];
        std::fs::write(&p, &blob).unwrap();
        let err = format!("{:#}", load_verification_run(&p).unwrap_err());
        std::fs::remove_file(&p).ok();
        assert!(err.contains("bytes"), "{err}");
        assert!(!err.contains(&"{".repeat(20)), "{err}");
    }
}
