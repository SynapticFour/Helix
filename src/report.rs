// SPDX-License-Identifier: Apache-2.0
//! Human and JSON reporting.
//!
//! `helix verify` JSON is Helix `VerificationRun` (DRS + WES when TESTABLE).
//! `helix compare` JSON is Helix `CompareReport` (stable-id regression).
//! `helix security` JSON is HelixTest `OverallReport`.
//! `helix bench` JSON is Helix-owned (`BenchOutcome`).
//! Skips are not passes. Discovery is not a pass. Not HELIOS.

use std::io::{self, IsTerminal};

use common::report::{OverallReport, ServiceKind, TestStatus};

use crate::bench::BenchOutcome;
use crate::compare::{CompareKind, CompareReport};
use crate::layer::{CheckLayer, LayerSummary};
use crate::model::{VerificationResult, VerificationRun, VerificationStatus};
use crate::security::SecurityOutcome;
use crate::standards::BindingKind;
use crate::verify::VerifyOutcome;

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";

/// Color when stdout is a TTY and `NO_COLOR` is unset.
pub fn color_enabled() -> bool {
    io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// PASS / FAIL / SKIP. Skip is never painted as PASS.
pub fn status_mark(status: TestStatus, color: bool) -> String {
    let (plain, paint) = match status {
        TestStatus::Pass => ("PASS", GREEN),
        TestStatus::Fail => ("FAIL", RED),
        TestStatus::Skip => ("SKIP", YELLOW),
    };
    if color {
        format!("{paint}{plain}{RESET}")
    } else {
        plain.to_string()
    }
}

/// Helix verification status. Skip is never green. Error is not pass.
pub fn helix_status_mark(status: VerificationStatus, color: bool) -> String {
    let (plain, paint) = match status {
        VerificationStatus::Pass => ("PASS", GREEN),
        VerificationStatus::Fail => ("FAIL", RED),
        VerificationStatus::Skip => ("SKIP", YELLOW),
        VerificationStatus::Error => ("ERROR", RED),
    };
    if color {
        format!("{paint}{plain}{RESET}")
    } else {
        plain.to_string()
    }
}

/// Deterministic pretty JSON for `helix verify` (`VerificationRun`).
/// `claims` is always recomputed from the rest of the document.
pub fn verify_json(run: &VerificationRun) -> anyhow::Result<String> {
    crate::guardrails::check_run(run)?;
    let mut value = serde_json::to_value(run)?;
    value["claims"] = serde_json::to_value(crate::claims::evaluate(run))?;
    crate::guardrails::check_serialized_claims(run, &value)?;
    Ok(crate::redact::redact_text(&serde_json::to_string_pretty(
        &value,
    )?))
}

pub fn print_json(outcome: &VerifyOutcome) -> anyhow::Result<()> {
    println!("{}", verify_json(&outcome.run)?);
    Ok(())
}

pub fn compare_json(report: &CompareReport) -> anyhow::Result<String> {
    Ok(crate::redact::redact_text(&serde_json::to_string_pretty(
        report,
    )?))
}

pub fn print_compare_json(report: &CompareReport) -> anyhow::Result<()> {
    println!("{}", compare_json(report)?);
    Ok(())
}

pub fn print_compare_text(report: &CompareReport) {
    print!("{}", format_compare_text(report, color_enabled()));
}

/// Human compare report. Same facts as [`CompareReport`] JSON. Not a score.
pub fn format_compare_text(report: &CompareReport, color: bool) -> String {
    let mut out = String::new();
    out.push_str("HELIX VERIFICATION COMPARE\n");
    out.push('\n');
    out.push_str("This is a technical verification signal.\n");
    out.push_str("It is not GA4GH certification.\n");
    out.push('\n');
    out.push_str("Previous:\n");
    out.push_str(&format!("  {}\n", report.previous_target));
    out.push_str("Current:\n");
    out.push_str(&format!("  {}\n", report.current_target));
    out.push('\n');
    out.push_str("Identity:\n");
    if report.same_measurement {
        out.push_str("  same measurement: yes\n");
    } else {
        out.push_str("  same measurement: no\n");
    }
    if report.suite_changed {
        out.push_str("  suite changed: yes (Helix or HelixTest pin differs; not a signed trail)\n");
    } else {
        out.push_str("  suite changed: no\n");
    }
    if report.identity_mismatches.is_empty() {
        out.push_str("  mismatches: none\n");
    } else {
        out.push_str("  mismatches:\n");
        for m in &report.identity_mismatches {
            out.push_str(&format!(
                "    {} : {} -> {}\n",
                m.field, m.previous, m.current
            ));
        }
    }
    out.push_str("  Not a signed audit trail. Not HELIOS.\n");
    out.push('\n');
    out.push_str("Changes:\n");
    let changed: Vec<_> = report
        .rows
        .iter()
        .filter(|r| {
            !matches!(
                r.kind,
                CompareKind::UnchangedPass
                    | CompareKind::UnchangedFail
                    | CompareKind::UnchangedSkip
            )
        })
        .collect();
    if changed.is_empty() {
        out.push_str("  None.\n");
    } else {
        for row in changed {
            out.push_str(&format_compare_row(row, color));
        }
    }
    out.push('\n');
    out.push_str("Unchanged:\n");
    let unchanged: Vec<_> = report
        .rows
        .iter()
        .filter(|r| {
            matches!(
                r.kind,
                CompareKind::UnchangedPass
                    | CompareKind::UnchangedFail
                    | CompareKind::UnchangedSkip
            )
        })
        .collect();
    if unchanged.is_empty() {
        out.push_str("  None.\n");
    } else {
        for row in unchanged {
            out.push_str(&format_compare_row(row, color));
        }
    }
    out.push('\n');
    let s = &report.summary;
    out.push_str("Summary:\n");
    out.push_str(&format!("  {} NEW_FAIL\n", s.new_fail));
    out.push_str(&format!("  {} UNCHANGED_FAIL\n", s.unchanged_fail));
    out.push_str(&format!("  {} FIXED\n", s.fixed));
    out.push_str(&format!("  {} UNCHANGED_PASS\n", s.unchanged_pass));
    out.push_str(&format!("  {} NEW_SKIP\n", s.new_skip));
    out.push_str(&format!(
        "  {} FIXED_SKIP (SKIP→PASS: {})\n",
        s.fixed_skip, s.skip_became_pass
    ));
    out.push_str(&format!("  {} UNCHANGED_SKIP\n", s.unchanged_skip));
    out.push_str(&format!("  {} ADDED\n", s.added));
    out.push('\n');
    out.push_str("Skip is never pass. SKIP→PASS is FIXED_SKIP, never UNCHANGED_PASS.\n");
    if report.has_regression {
        out.push_str("Result: REGRESSION\n");
    } else {
        out.push_str("Result: NO_NEW_REGRESSION\n");
    }
    crate::redact::redact_text(&out)
}

fn format_compare_row(row: &crate::compare::CompareRow, color: bool) -> String {
    let mark = compare_kind_mark(row.kind, color);
    let prev = status_opt(row.previous);
    let curr = status_opt(row.current);
    if row.skip_became_pass {
        format!(
            "  {mark}  {}  {}  {prev} → {curr}  (SKIP must not silently become PASS)\n",
            row.id, row.code
        )
    } else {
        format!("  {mark}  {}  {}  {prev} → {curr}\n", row.id, row.code)
    }
}

fn status_opt(status: Option<VerificationStatus>) -> &'static str {
    match status {
        Some(VerificationStatus::Pass) => "pass",
        Some(VerificationStatus::Fail) => "fail",
        Some(VerificationStatus::Skip) => "skip",
        Some(VerificationStatus::Error) => "error",
        None => "absent",
    }
}

fn compare_kind_mark(kind: CompareKind, color: bool) -> String {
    let (plain, paint) = match kind {
        CompareKind::NewFail => ("NEW_FAIL", RED),
        CompareKind::Fixed => ("FIXED", GREEN),
        CompareKind::UnchangedFail => ("UNCHANGED_FAIL", YELLOW),
        CompareKind::UnchangedPass => ("UNCHANGED_PASS", GREEN),
        CompareKind::NewSkip => ("NEW_SKIP", YELLOW),
        CompareKind::FixedSkip => ("FIXED_SKIP", YELLOW),
        CompareKind::UnchangedSkip => ("UNCHANGED_SKIP", YELLOW),
        CompareKind::Added => ("ADDED", YELLOW),
    };
    if color {
        format!("{paint}{plain}{RESET}")
    } else {
        format!("{plain:<16}")
    }
}

pub fn print_text(outcome: &VerifyOutcome) -> anyhow::Result<()> {
    crate::guardrails::check_run(&outcome.run)?;
    print!("{}", format_verify_text(&outcome.run, color_enabled()));
    Ok(())
}

/// Human verify report. Same facts as [`VerificationRun`] JSON. Not HELIOS.
pub fn format_verify_text(run: &VerificationRun, color: bool) -> String {
    let mut out = String::new();
    out.push_str("HELIX VERIFICATION\n");
    out.push('\n');
    out.push_str("This is a technical verification signal.\n");
    out.push_str("It is not GA4GH certification.\n");
    out.push('\n');
    out.push_str(&crate::claims::format_claims_section(run, color));
    out.push_str("What:\n");
    out.push_str(
        "  DRS and WES checks (HelixTest wrap). TES/TRS/htsget discovered only, not executed.\n",
    );
    out.push('\n');
    out.push_str("Target:\n");
    out.push_str(&format!("  {}\n", run.target.url));
    if let Some(id) = &run.target.identity {
        out.push_str(&format!("  target_id: {}\n", id.target_id));
        out.push_str(&format!("  target_kind: {}\n", id.target_kind.as_str()));
        out.push_str(&format!(
            "  implementation_name: {}\n",
            id.implementation_name.as_deref().unwrap_or("(undeclared)")
        ));
        out.push_str(&format!(
            "  implementation_version: {} (declared, untrusted)\n",
            id.implementation_version
                .as_deref()
                .unwrap_or("(undeclared)")
        ));
        if let Some(sel) = &run.standard_selection {
            out.push_str(&format!(
                "  target_execution_id: {}\n",
                sel.target_execution_id.as_deref().unwrap_or("(none)")
            ));
        }
    }
    if let Some(fx) = &run.drs_fixture {
        out.push('\n');
        out.push_str("DRS fixture (test input, not a GA4GH requirement):\n");
        out.push_str(&format!("  object_id: {}\n", fx.object_id));
        out.push_str(&format!("  unknown_object_id: {}\n", fx.unknown_object_id));
        out.push_str(&format!("  source: {}\n", fx.source.as_str()));
        if let Some(h) = fx.expected_sha256.as_deref() {
            out.push_str(&format!("  expected_sha256: {h}\n"));
        }
        out.push_str(&format!(
            "  checksum_mode: {} ({})\n",
            fx.checksum_mode.as_str(),
            match fx.checksum_mode {
                crate::fixture::ChecksumMode::OperatorDigest => {
                    "operator digest vs downloaded bytes; advertised GetObject sha256 cannot manufacture a PASS"
                }
                crate::fixture::ChecksumMode::AdvertisedConsistency => {
                    "advertised GetObject sha256 vs downloaded bytes; not an independent blob-integrity oracle"
                }
            }
        ));
    }
    out.push('\n');
    out.push_str("Helix:\n");
    out.push_str(&format!("  {}\n", run.helix_version));
    out.push_str(&format!("  schema {}\n", run.schema_version));
    if let Some(profile) = run.profile.as_deref() {
        out.push_str(&format!("  profile {profile}\n"));
    }
    out.push_str(&format!("  fixtures {}\n", run.fixture_version));
    out.push_str(&format!("  {}\n", run.timestamp));
    out.push('\n');
    out.push_str("Test suite:\n");
    match (
        run.helixtest_version.as_deref(),
        run.helixtest_sha.as_deref(),
    ) {
        (Some(tag), Some(sha)) => {
            out.push_str(&format!("  HelixTest tag {tag}\n"));
            out.push_str(&format!(
                "  git checkout pin: {}\n",
                crate::model::HELIXTEST_SHA
            ));
            out.push_str(&format!("  executed checker: helixtest-drs:{sha}\n"));
        }
        (Some(tag), None) => out.push_str(&format!("  HelixTest {tag}\n")),
        _ => out.push_str("  HelixTest pin not recorded on this run\n"),
    }
    out.push('\n');
    out.push_str(&format_standards_section(run));
    out.push_str("Services:\n");
    if run.discovery.is_empty() {
        out.push_str("  (none recorded)\n");
    } else {
        for d in &run.discovery {
            out.push_str("  ");
            out.push_str(&format_service_line(d));
            out.push('\n');
        }
    }
    out.push('\n');
    out.push_str("Results:\n");
    let grouped = grouped_results(run);
    if grouped.is_empty() {
        out.push_str("  (no checks)\n");
    } else {
        for (service, rows) in grouped {
            out.push('\n');
            out.push_str(&format!("{}\n", display_service(&service)));
            for r in rows {
                out.push_str(&format_result_block(r, color));
            }
        }
    }
    out.push('\n');
    let s = &run.summary;
    out.push_str("Summary:\n");
    out.push_str(&format!("  {} PASS\n", s.passed));
    out.push_str(&format!("  {} FAIL\n", s.failed));
    out.push_str(&format!("  {} ERROR\n", s.errors));
    out.push_str(&format!("  {} SKIP\n", s.skipped));
    out.push('\n');
    if let Some(layers) = &run.layer_summary {
        out.push_str(&format_layers_section(layers, run));
    }
    out.push_str(&format_evidence_section(run));
    out.push_str("Changes:\n");
    out.push_str("  Not compared. This report is a single run.\n");
    out.push_str("  What changed: helix compare <previous.json> <current.json>\n");
    out.push('\n');
    out.push_str("Discovery is not conformance. DETECTED is not a pass. Skip is never pass.\n");
    crate::redact::redact_text(&out)
}

fn format_layers_section(summary: &LayerSummary, run: &VerificationRun) -> String {
    let mut out = String::from("Layers (not a score; SCHEMA PASS is not BEHAVIOR PASS):\n");
    for (layer, counts) in [
        (CheckLayer::Schema, &summary.schema),
        (CheckLayer::Behavior, &summary.behavior),
        (CheckLayer::Security, &summary.security),
        (CheckLayer::Interoperability, &summary.interoperability),
    ] {
        let verdict = counts.verdict();
        out.push_str(&format!(
            "  {} {}\n",
            layer.report_heading(),
            verdict.as_str()
        ));
        out.push_str(&format!(
            "    pass={} fail={} error={} skip={}\n",
            counts.passed, counts.failed, counts.errors, counts.skipped
        ));
        if matches!(
            verdict,
            crate::layer::LayerVerdict::Fail | crate::layer::LayerVerdict::Error
        ) {
            for r in run.executed.iter() {
                let l = r.layer.unwrap_or_else(|| crate::layer::for_id(&r.id));
                if l == layer && r.status.is_blocking() {
                    let mark = match r.status {
                        VerificationStatus::Fail => "fail",
                        VerificationStatus::Error => "error",
                        _ => "blocking",
                    };
                    out.push_str(&format!("    - {} ({mark})\n", r.id));
                }
            }
        }
    }
    out.push_str("  Benchmark rows are not a conformance layer.\n");
    out.push_str("  NONE means this layer did not execute; that is not PASS.\n");
    out.push('\n');
    out
}

fn format_evidence_section(run: &VerificationRun) -> String {
    let mut counts = [0u32; 6];
    let mut unlabeled = 0u32;
    for r in run.executed.iter().chain(run.skipped.iter()) {
        match &r.traceability {
            Some(t) => {
                if let Some(i) = BindingKind::ALL.iter().position(|k| *k == t.category) {
                    counts[i] += 1;
                }
            }
            None => unlabeled += 1,
        }
    }
    let mut out = String::from("Evidence (classification, not a score):\n");
    for (kind, n) in BindingKind::ALL.iter().zip(counts.iter()) {
        out.push_str(&format!("  {n} {}\n", kind.as_str()));
    }
    if unlabeled > 0 {
        out.push_str(&format!("  {unlabeled} unlabeled (invalid)\n"));
    }
    if counts[0] == 0 {
        out.push_str("  No check in this run is a GA4GH MUST.\n");
    } else {
        out.push_str("  Only category=normative rows may support a GA4GH requirement sentence.\n");
    }
    out.push('\n');
    out
}

fn format_standards_section(run: &VerificationRun) -> String {
    let mut out = String::from("Standards:\n");
    match &run.standard_selection {
        None => {
            out.push_str("  mode: unversioned\n");
            out.push_str("  selected_version: (none)\n");
            out.push_str("  verified_version: (none)\n");
            out.push_str("  helix verify did not select a GA4GH registry pack.\n");
        }
        Some(sel) => {
            out.push_str(&format!("  mode: {}\n", sel.mode));
            out.push_str(&format!("  selection_status: {}\n", sel.selection_status));
            out.push_str(&format!(
                "  standard: {}\n",
                sel.standard.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  requested_version: {}\n",
                sel.requested_version.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  detected_version: {}\n",
                sel.detected_version.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  selected_version: {}\n",
                sel.selected_version.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  verified_version: {}\n",
                sel.verified_version.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  support_status: {}\n",
                sel.support_status
                    .as_deref()
                    .unwrap_or(if sel.selection_status == crate::standards::SELECTED {
                        "SUPPORTED"
                    } else {
                        "(not selected)"
                    })
                    .to_uppercase()
            ));
            out.push_str(&format!(
                "  schema: {}\n",
                sel.schema_entry.as_deref().unwrap_or("(none)")
            ));
            if let Some(h) = &sel.schema_document_sha256 {
                out.push_str(&format!("  schema_document_sha256: {h}\n"));
            }
            if let Some(h) = &sel.schema_component_sha256 {
                out.push_str(&format!("  schema_component_sha256: {h}\n"));
            }
            out.push_str(&format!(
                "  checker: {}\n",
                sel.checker_id.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  binding: {}\n",
                sel.binding_id.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  catalog: {}\n",
                sel.catalog_id.as_deref().unwrap_or("(none)")
            ));
            if sel.coverage_schema.is_some() {
                out.push_str(&format!(
                    "  coverage: schema={} behavior={} security={} interoperability={}\n",
                    sel.coverage_schema.as_deref().unwrap_or("(none)"),
                    sel.coverage_behavior.as_deref().unwrap_or("(none)"),
                    sel.coverage_security.as_deref().unwrap_or("(none)"),
                    sel.coverage_interoperability.as_deref().unwrap_or("(none)")
                ));
            }
            let target = if sel.selection_status != crate::standards::SELECTED {
                "NOT_VERIFIED"
            } else if run.executed.iter().any(|r| {
                r.status == crate::model::VerificationStatus::Fail
                    || r.status == crate::model::VerificationStatus::Error
            }) {
                "FAIL"
            } else if run
                .executed
                .iter()
                .any(|r| r.status == crate::model::VerificationStatus::Pass)
            {
                "PASS"
            } else {
                "NOT_VERIFIED"
            };
            out.push_str(&format!("  target_result: {target}\n"));
            out.push_str("  verification_claim: NOT_VERIFIED unless claims[] says otherwise\n");
            out.push_str("  Technical Verification only. Not GA4GH certification.\n");
            out.push_str(&format!(
                "  standards_registry_entry: {}\n",
                sel.standards_registry_entry.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  standards_source_commit: {}\n",
                sel.standards_source_commit.as_deref().unwrap_or("(none)")
            ));
            out.push_str(&format!(
                "  substituted: {}\n",
                if sel.substituted { "yes" } else { "no" }
            ));
            if let Some(note) = &sel.note {
                out.push_str(&format!("  {note}\n"));
            }
            if !sel.other_rows_not_selected.is_empty() {
                out.push_str("  other rows (not selected):\n");
                for row in &sel.other_rows_not_selected {
                    out.push_str(&format!("    {row}\n"));
                }
            }
        }
    }
    out.push('\n');
    out
}

fn display_service(json_name: &str) -> String {
    match json_name {
        "drs" => "DRS".into(),
        "wes" => "WES".into(),
        "tes" => "TES".into(),
        "trs" => "TRS".into(),
        "htsget" => "htsget".into(),
        other => other.to_string(),
    }
}

fn format_service_line(d: &crate::model::DiscoveredService) -> String {
    let svc = display_service(&d.service);
    if !d.present {
        format!("{svc:<8} NOT_DETECTED")
    } else if d.testable {
        let mut line = format!("{svc:<8} DETECTED     TESTABLE");
        if let Some(base) = &d.base_url {
            if !base.is_empty() {
                line.push_str("  ");
                line.push_str(base);
            }
        }
        line
    } else {
        let mut line = format!("{svc:<8} DETECTED     NOT_TESTABLE");
        if let Some(reason) = &d.not_testable_reason {
            line.push_str("  ");
            line.push_str(reason);
        }
        line
    }
}

fn grouped_results(run: &VerificationRun) -> Vec<(String, Vec<&VerificationResult>)> {
    const ORDER: &[&str] = &["drs", "wes", "tes", "trs", "htsget"];
    let mut by_service: std::collections::BTreeMap<String, Vec<&VerificationResult>> =
        std::collections::BTreeMap::new();
    for r in run.executed.iter().chain(run.skipped.iter()) {
        by_service.entry(r.service.clone()).or_default().push(r);
    }
    for rows in by_service.values_mut() {
        rows.sort_by(|a, b| a.code.cmp(&b.code).then(a.id.cmp(&b.id)));
    }
    let mut out = Vec::new();
    for key in ORDER {
        if let Some(rows) = by_service.remove(*key) {
            if !rows.is_empty() {
                out.push(((*key).to_string(), rows));
            }
        }
    }
    for (k, rows) in by_service {
        if !rows.is_empty() {
            out.push((k, rows));
        }
    }
    out
}

fn format_result_block(r: &VerificationResult, color: bool) -> String {
    let mut out = String::new();
    let mark = helix_status_mark(r.status, color);
    match &r.message {
        Some(msg) if r.status != VerificationStatus::Pass => {
            let msg = crate::sanitize::sanitize_untrusted(msg);
            out.push_str(&format!(
                "  {mark}  {}  {}  {} — {msg}\n",
                r.id, r.code, r.name
            ));
        }
        _ => {
            out.push_str(&format!("  {mark}  {}  {}  {}\n", r.id, r.code, r.name));
        }
    }
    if let Some(d) = &r.diagnostic {
        out.push_str(&format!("        expected: {}\n", d.expected));
        out.push_str(&format!(
            "        observed: {}\n",
            crate::sanitize::sanitize_untrusted(&d.observed)
        ));
        out.push_str(&format!(
            "        category: {}\n",
            d.likely_category.as_str()
        ));
        out.push_str(&format!("        hint: {}\n", d.hint));
        out.push_str("        possible causes:\n");
        for c in &d.possible_causes {
            out.push_str(&format!("          - {c}\n"));
        }
    }
    if let Some(t) = &r.traceability {
        out.push_str(&format!(
            "        layer: {}  kind: {}  claim_scope: {}  authority: {}\n",
            t.layer.as_str(),
            t.category.as_str(),
            t.claim_scope.as_str(),
            t.authority.as_str()
        ));
        if let Some(req) = &t.request {
            out.push_str(&format!("        request: {req}\n"));
        }
        if t.claim_scope.may_support_conformance_claim() {
            if let Some(v) = &t.version {
                out.push_str(&format!("        normative_version: {v}\n"));
            }
        } else {
            out.push_str("        not a GA4GH MUST  (PASS is not a conformance claim)\n");
        }
    }
    out
}

pub fn print_security_json(outcome: &SecurityOutcome) -> anyhow::Result<()> {
    let mut report = OverallReport {
        services: vec![outcome.auth.clone(), outcome.crypt4gh.clone()],
        enabled_services: vec![ServiceKind::Auth, ServiceKind::Crypt4gh],
        skipped_services: Vec::new(),
        executed_test_modules: vec![ServiceKind::Auth, ServiceKind::Crypt4gh],
        diagnostics: None,
    };
    report.sort_services_canonical();
    println!(
        "{}",
        crate::redact::redact_text(&serde_json::to_string_pretty(&report)?)
    );
    Ok(())
}

pub fn print_security_text(outcome: &SecurityOutcome) {
    let color = color_enabled();
    println!("{}", crate::security::SECURITY_BEHAVIOR_DISCLAIMER);
    println!("Helix security — selected Security Behavior Profile (not HELIOS)");
    println!(
        "Helix runs documented DRS and WES checks. Ferrum is a reference target, not a dependency."
    );
    println!("This is not a verification against a SUPPORTED GA4GH release. Not certification.");
    println!("Tokens from test-fixtures/ only. Not for production.");
    println!("Passing these checks does not prove the implementation is secure.");
    println!();
    println!("Auth (black-box HTTP)");
    print_tests(&outcome.auth.tests, color);
    println!();
    println!("Crypt4GH (protocol layout only; not encryption, not secure)");
    print_tests(&outcome.crypt4gh.tests, color);
}

pub fn print_bench_json(outcome: &BenchOutcome) -> anyhow::Result<()> {
    println!(
        "{}",
        crate::redact::redact_text(&serde_json::to_string_pretty(outcome)?)
    );
    Ok(())
}

pub fn print_bench_text(outcome: &BenchOutcome) {
    let color = color_enabled();
    println!(
        "Helix bench — {} (not Demo hap.py, not GIAB, not HELIOS)",
        outcome.workload_id
    );
    println!(
        "Helix runs documented DRS and WES checks. Ferrum is a reference target, not a dependency."
    );
    println!("This is not a verification against a SUPPORTED GA4GH release. Not certification.");
    println!("Sample percentiles of this series. Not a significance test.");
    println!("A warning means performance changed enough to merit human inspection.");
    println!("It does not mean the implementation is incorrect.");
    println!(
        "A bench warning is not a verification failure. This command does not fail the build."
    );
    println!();
    println!(
        "workload_id: {}  version: {}",
        outcome.workload_id, outcome.workload_version
    );
    println!("requests: {}", outcome.workload.join(", "));
    println!(
        "warmup: {}  repetitions: {}  (wall_ms is the median of measured runs)",
        outcome.baseline.metadata.warmup, outcome.baseline.metadata.repetitions
    );
    if outcome.environment.comparable {
        println!("environment: comparable — {}", outcome.environment.basis);
    } else {
        println!(
            "environment: INCOMPARABLE — {}",
            outcome
                .environment
                .incomparable_reason
                .as_deref()
                .unwrap_or(&outcome.environment.basis)
        );
    }
    println!();
    print_sample_text("baseline", &outcome.baseline);
    print_sample_text("candidate", &outcome.candidate);
    println!();
    println!(
        "analysis: measurement={}  warning={}  regression={}  verification_failure={}",
        outcome.analysis.measurement,
        outcome.analysis.warning,
        outcome.analysis.regression,
        outcome.analysis.verification_failure
    );
    println!("  measurement: {}", outcome.analysis.measurement_means);
    println!("  warning: {}", outcome.analysis.warning_means);
    println!(
        "  warning does not mean: {}",
        outcome.analysis.warning_does_not_mean
    );
    println!("  regression: {}", outcome.analysis.regression_means);
    if !outcome.analysis.distribution_compare {
        println!("  distribution_compare: false (single-run series are measurement only)");
    }
    println!();
    for c in &outcome.analysis.changes {
        if c.available {
            let pct = c
                .pct
                .map(|p| format!("{p:+.1}%"))
                .unwrap_or_else(|| "n/a (baseline 0)".into());
            println!(
                "  change  {}  baseline={:.4}  candidate={:.4}  {pct}",
                c.metric,
                c.baseline.unwrap_or(0.0),
                c.candidate.unwrap_or(0.0)
            );
        } else {
            println!(
                "  change  {}  omitted — {}",
                c.metric,
                c.omitted_reason.as_deref().unwrap_or("unavailable")
            );
        }
    }
    println!();
    for d in &outcome.diff {
        let pct = d
            .pct
            .map(|p| format!("{p:+.1}%"))
            .unwrap_or_else(|| "n/a (baseline 0)".into());
        let mark = if d.worse {
            if color {
                "\x1b[33mWARN\x1b[0m"
            } else {
                "WARN"
            }
        } else if !outcome.environment.comparable {
            "n/c"
        } else {
            "ok"
        };
        println!("  {mark}  {} {pct}", d.name);
    }
    if outcome.warning {
        println!();
        println!(
            "threshold {:+}% — human inspection, not a red X, not a verification failure (does not fail CI):",
            outcome.threshold_pct
        );
        for w in &outcome.warnings {
            println!("  - {w}");
        }
    }
}

fn print_sample_text(role: &str, s: &crate::bench::Sample) {
    let p95 = s
        .latency
        .p95_ms
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "n/a".into());
    println!(
        "{role}  {}  {}  helix={}  {}/{}",
        s.label, s.endpoint, s.metadata.helix_version, s.metadata.os, s.metadata.arch
    );
    println!(
        "         wall_ms median={:.1} min={:.1} max={:.1} p95={}  rss_kb={}  error_rate={:.2} ({}/{})  bytes={}",
        s.latency.median_ms,
        s.latency.min_ms,
        s.latency.max_ms,
        p95,
        s.rss_kb
            .map(|v| v.to_string())
            .unwrap_or_else(|| "n/a".into()),
        s.error_rate,
        s.errors,
        s.requests,
        s.bytes
    );
    println!("         recorded {}", s.metadata.timestamp);
}

fn print_tests(tests: &[common::report::TestCaseResult], color: bool) {
    for t in tests {
        let mark = status_mark(t.status, color);
        match &t.error {
            Some(err) if t.status != TestStatus::Pass => {
                let err = crate::redact::redact_text(err);
                println!("  {mark}  {} — {err}", t.name);
            }
            _ => println!("  {mark}  {}", t.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::{
        Detection, Discovery, Ga4ghService, ServiceDiscovery, ServiceInfoSnapshot, Testability,
        VERIFY_ORDER,
    };
    use crate::model::{
        Target, VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
    };

    #[test]
    fn skip_is_never_green() {
        let skip = status_mark(TestStatus::Skip, true);
        assert!(skip.contains("SKIP"));
        assert!(!skip.contains("32m"), "skip must not use green: {skip}");
        assert_eq!(status_mark(TestStatus::Pass, false), "PASS");
        assert!(status_mark(TestStatus::Pass, true).contains("32m"));
        assert!(status_mark(TestStatus::Fail, true).contains("31m"));
        let hskip = helix_status_mark(VerificationStatus::Skip, true);
        assert!(hskip.contains("SKIP"));
        assert!(!hskip.contains("32m"), "skip must not use green: {hskip}");
        assert!(helix_status_mark(VerificationStatus::Error, true).contains("31m"));
    }

    #[test]
    fn compare_fixed_skip_is_never_green() {
        let skip = compare_kind_mark(CompareKind::FixedSkip, true);
        assert!(skip.contains("FIXED_SKIP"));
        assert!(
            !skip.contains("32m"),
            "SKIP→PASS must not look like PASS: {skip}"
        );
        assert!(compare_kind_mark(CompareKind::NewFail, true).contains("31m"));
    }

    #[test]
    fn json_shape_is_helix_verification_run() {
        let mut services: Vec<ServiceDiscovery> = VERIFY_ORDER
            .iter()
            .map(|k| ServiceDiscovery::not_detected(*k))
            .collect();
        services[0] = ServiceDiscovery {
            kind: Ga4ghService::Drs,
            detection: Detection::Detected,
            testability: Testability::Testable,
            not_testable_reason: None,
            base_url: Some("http://127.0.0.1:9".into()),
            discovery_method: None,
            http_status: Some(200),
            service_info: ServiceInfoSnapshot::default(),
        };
        services[1] = ServiceDiscovery {
            kind: Ga4ghService::Wes,
            detection: Detection::Detected,
            testability: Testability::NotTestable,
            not_testable_reason: Some("not wired".into()),
            base_url: Some("http://127.0.0.1:9/ga4gh/wes/v1".into()),
            discovery_method: None,
            http_status: Some(200),
            service_info: ServiceInfoSnapshot::default(),
        };
        let mut run = VerificationRun::drs_profile(Target::new("http://127.0.0.1:9"));
        run.timestamp = "2026-09-04T12:00:00Z".into();
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            crate::identity::spec("drs.object.reachable"),
        )));
        run.sort_deterministic();
        let outcome = VerifyOutcome {
            discovery: Discovery {
                endpoint: "http://127.0.0.1:9".into(),
                services,
            },
            run,
        };
        let json = verify_json(&outcome.run).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("services").is_none(), "not HelixTest OverallReport");
        assert_eq!(v["profile"].as_str(), Some("drs"));
        assert_eq!(v["target"]["url"].as_str(), Some("http://127.0.0.1:9"));
        assert_eq!(
            v["helix_version"].as_str(),
            Some(crate::model::helix_version())
        );
        assert_eq!(v["helixtest_version"].as_str(), Some("v0.1.3"));
        assert!(v.get("passed").is_none());
        assert_eq!(
            v["executed"][0]["id"].as_str(),
            Some("drs.object.reachable")
        );
        assert_eq!(v["executed"][0]["code"].as_str(), Some("HLX-DRS-001"));
        assert_eq!(v["executed"][0]["status"].as_str(), Some("pass"));
        assert!(v.get("signature").is_none());
        assert!(v.get("ro_crate").is_none());
    }

    #[test]
    fn verify_text_answers_operator_questions_from_the_same_run() {
        use crate::model::{DiscoveredService, Target, HELIXTEST_PIN};

        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:8080"));
        run.timestamp = "2026-09-04T12:00:00Z".into();
        run.profile = Some("generic".into());
        run.discovery = vec![
            DiscoveredService::found("drs", "http://127.0.0.1:8080/ga4gh/drs/v1"),
            DiscoveredService::missing("wes"),
            DiscoveredService::detected_not_testable(
                "tes",
                "http://127.0.0.1:8080/ga4gh/tes/v1",
                "Helix Stage 1 does not execute TES checks; DETECTED is not a pass",
            ),
        ];
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            crate::identity::spec("drs.object.reachable"),
        )));
        run.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found")),
            "expected 404, got 200",
        ));
        run.push_skipped(VerificationResult::skip(
            VerificationCheck::from_spec(crate::identity::spec("wes.service_info.reachable")),
            "WES not detected; WES checks not executed (not a pass)",
        ));

        let text = format_verify_text(&run, false);
        assert!(text.starts_with("HELIX VERIFICATION\n"));
        assert!(text.contains("This is a technical verification signal."));
        assert!(text.contains("It is not GA4GH certification."));
        assert!(text.contains("Claims (predicates; not GA4GH certification):"));
        assert!(text.contains("No VERIFIED claim is justified by this run."));
        assert!(text.contains("ga4gh_requirement  NOT_VERIFIED"));
        assert!(text.contains("Why not verified:"));
        assert!(text.contains("unversioned_run"));
        assert!(!text.contains("ga4gh_requirement  VERIFIED"));
        assert!(text.contains("What:\n  DRS and WES checks (HelixTest wrap)"));
        assert!(text.contains("Target:\n  http://127.0.0.1:8080"));
        assert!(text.contains(&format!("Helix:\n  {}", crate::model::helix_version())));
        assert!(text.contains("schema helix-verification-v1"));
        assert!(text.contains("profile generic"));
        assert!(text.contains("fixtures helix-fixtures-v1"));
        assert!(text.contains(&format!("HelixTest tag {HELIXTEST_PIN}")));
        assert!(text.contains(&format!(
            "git checkout pin: {}",
            crate::model::HELIXTEST_SHA
        )));
        assert!(text.contains(&format!(
            "executed checker: helixtest-drs:{}",
            crate::checker::executed_checker_source_sha256()
        )));
        assert!(text.contains("Standards:"));
        assert!(text.contains("unversioned") || text.contains("did not select"));
        assert!(text.contains("DRS      DETECTED     TESTABLE"));
        assert!(text.contains("WES      NOT_DETECTED"));
        assert!(text.contains("TES      DETECTED     NOT_TESTABLE"));
        assert!(text.contains("\nDRS\n"));
        assert!(text.contains("PASS  drs.object.reachable  HLX-DRS-001"));
        assert!(text.contains("layer: interoperability  kind: fixture"));
        assert!(text.contains("layer: behavior  kind: fixture"));
        assert!(text.contains("Layers (not a score; SCHEMA PASS is not BEHAVIOR PASS)"));
        assert!(text.contains("SCHEMA"));
        assert!(text.contains("BEHAVIOR"));
        assert!(text.contains("not a GA4GH MUST"));
        assert!(text.contains("Evidence (classification, not a score):"));
        assert!(text.contains("No check in this run is a GA4GH MUST."));
        assert!(text.contains("FAIL  drs.object.not_found  HLX-DRS-005"));
        assert!(text.contains("expected 404, got 200"));
        assert!(text.contains("possible causes:"));
        assert!(text.contains("\nWES\n"));
        assert!(text.contains("SKIP  wes.service_info.reachable  HLX-WES-001"));
        assert!(text.contains("Summary:\n  1 PASS\n  1 FAIL\n  0 ERROR\n  1 SKIP\n"));
        assert!(text.contains("Not compared. This report is a single run."));
        assert!(text.contains("helix compare"));
        assert!(text.contains("DETECTED is not a pass"));
        assert!(text.contains("not conformance"));
        assert!(!text.contains("RO-Crate"));
        assert!(!text.contains("signature"));
        assert!(!text.to_lowercase().contains("pdf"));
        assert!(!text.contains("Cause:"));
        assert!(
            !text.to_lowercase().split_whitespace().any(|w| w == "found"),
            "text must not say found as verified: {text}"
        );

        let json = verify_json(&run).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["target"]["url"], "http://127.0.0.1:8080");
        assert_eq!(v["helix_version"], crate::model::helix_version());
        assert_eq!(v["helixtest_version"], HELIXTEST_PIN);
        assert_eq!(v["summary"]["passed"], 1);
        assert_eq!(v["summary"]["failed"], 1);
        assert_eq!(v["summary"]["skipped"], 1);
        assert_eq!(v["discovery"][0]["present"], true);
        assert_eq!(v["discovery"][0]["testable"], true);
        assert_eq!(v["discovery"][1]["present"], false);
        assert_eq!(v["claims"][0]["status"], "not_verified");
        assert!(v["claims"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] == "not_verified"));
    }

    #[test]
    fn compare_text_answers_what_changed() {
        use crate::compare::{compare_runs, CompareKind};
        use crate::model::Target;

        let mut previous = VerificationRun::new(Target::new("http://127.0.0.1:8"));
        previous.timestamp = "2026-09-04T11:00:00Z".into();
        previous.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            crate::identity::spec("drs.object.reachable"),
        )));
        previous.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found")),
            "expected 404, got 200",
        ));
        let mut current = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        current.timestamp = "2026-09-04T12:00:00Z".into();
        current.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.reachable")),
            "unreachable",
        ));
        current.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found")),
            "expected 404, got 200",
        ));
        let report = compare_runs(&previous, &current).unwrap();
        assert!(report.rows.iter().any(|r| r.kind == CompareKind::NewFail));
        let text = format_compare_text(&report, false);
        assert!(text.starts_with("HELIX VERIFICATION COMPARE\n"));
        assert!(text.contains("It is not GA4GH certification."));
        assert!(text.contains("Previous:\n  http://127.0.0.1:8"));
        assert!(text.contains("Current:\n  http://127.0.0.1:9"));
        assert!(text.contains("Identity:\n"));
        assert!(text.contains("same measurement: no"));
        assert!(text.contains("Not a signed audit trail. Not HELIOS."));
        assert!(text.contains("Changes:\n"));
        assert!(text.contains("NEW_FAIL"));
        assert!(text.contains("Unchanged:\n"));
        assert!(text.contains("UNCHANGED_FAIL"));
        assert!(text.contains("Result: REGRESSION"));
        assert!(!text.contains("RO-Crate"));
    }

    #[test]
    fn verify_json_and_text_redact_authorization() {
        let jwt =
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        run.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found")),
            format!("Authorization: Bearer {jwt}"),
        ));
        let json = verify_json(&run).unwrap();
        assert!(!json.contains(jwt), "{json}");
        assert!(!json.contains("Bearer eyJ"), "{json}");
        serde_json::from_str::<serde_json::Value>(&json).expect("redacted JSON stays valid JSON");
        let text = format_verify_text(&run, false);
        assert!(!text.contains(jwt), "{text}");
        assert!(!text.contains("s3cret"));
    }

    #[test]
    fn text_report_strips_ansi_and_forged_newlines_from_target_text() {
        let mut run = VerificationRun::new(crate::model::Target::new("http://127.0.0.1:9"));
        run.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.not_found")),
            "got 200\x1b[32mPASS\x1b[0m\nHELIX VERIFICATION\n  5 PASS",
        ));
        let text = format_verify_text(&run, false);
        assert!(!text.contains('\u{1b}'), "{text:?}");
        assert!(text.starts_with("HELIX VERIFICATION\n"));
        assert!(
            !text.contains("\nHELIX VERIFICATION\n"),
            "target must not inject an extra report header:\n{text}"
        );
        assert!(
            !text.contains("\n  5 PASS\n"),
            "target must not inject a forged summary line:\n{text}"
        );
        let json = verify_json(&run).unwrap();
        assert!(!json.contains('\u{1b}'), "{json}");
    }
}
