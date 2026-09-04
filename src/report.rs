// SPDX-License-Identifier: Apache-2.0
//! Human and JSON reporting.
//!
//! `helix verify` / `helix security` JSON is HelixTest `OverallReport` (D3).
//! `helix bench` JSON is Helix-owned (`BenchOutcome`), not that shape.
//! Skips are not passes. Not HELIOS (no RO-Crate / PDF / signatures).

use std::io::{self, IsTerminal};

use common::report::{OverallReport, ServiceKind, SkippedService, TestStatus};

use crate::bench::BenchOutcome;
use crate::discover::{Ga4ghService, VERIFY_ORDER};
use crate::security::SecurityOutcome;
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

fn to_service_kind(kind: Ga4ghService) -> ServiceKind {
    match kind {
        Ga4ghService::Drs => ServiceKind::Drs,
        Ga4ghService::Wes => ServiceKind::Wes,
        Ga4ghService::Tes => ServiceKind::Tes,
        Ga4ghService::Trs => ServiceKind::Trs,
        Ga4ghService::Htsget => ServiceKind::Htsget,
    }
}

/// HelixTest JSON shape. Discovered but unwired APIs are skipped, not passed.
pub fn overall_report(outcome: &VerifyOutcome) -> OverallReport {
    let mut skipped_services = Vec::new();
    for kind in VERIFY_ORDER {
        if kind == Ga4ghService::Drs {
            continue;
        }
        if outcome.discovery.get(kind).is_some() {
            skipped_services.push(SkippedService {
                service: to_service_kind(kind),
                reason:
                    "discovered; Helix Stage 1 runs DRS first (WES/TES/TRS/htsget checks not wired)"
                        .to_string(),
            });
        }
    }
    OverallReport {
        services: vec![outcome.drs.clone()],
        enabled_services: vec![ServiceKind::Drs],
        skipped_services,
        executed_test_modules: vec![ServiceKind::Drs],
        diagnostics: None,
    }
}

pub fn print_json(outcome: &VerifyOutcome) -> anyhow::Result<()> {
    let mut report = overall_report(outcome);
    report.sort_services_canonical();
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn print_text(outcome: &VerifyOutcome) {
    let color = color_enabled();
    print_discovery_text(&outcome.discovery);
    println!();
    println!("DRS (HelixTest checks; not certification)");
    print_tests(&outcome.drs.tests, color);
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
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn print_security_text(outcome: &SecurityOutcome) {
    let color = color_enabled();
    println!("Helix security — GA4GH auth behaviour (not certification, not HELIOS)");
    println!("Helix tests behavior against the GA4GH spec, independent of implementation.");
    println!("Ferrum is used as a reference target, not a dependency.");
    println!("Tokens from test-fixtures/ only. NICHT FÜR PRODUKTION.");
    println!();
    println!("Auth (black-box HTTP)");
    print_tests(&outcome.auth.tests, color);
    println!();
    println!("Crypt4GH (header structure only; no keys in output)");
    print_tests(&outcome.crypt4gh.tests, color);
}

pub fn print_bench_json(outcome: &BenchOutcome) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(outcome)?);
    Ok(())
}

pub fn print_bench_text(outcome: &BenchOutcome) {
    let color = color_enabled();
    println!("Helix bench — 3 small GETs (not Demo hap.py, not GIAB, not HELIOS)");
    println!("Helix tests behavior against the GA4GH spec, independent of implementation.");
    println!("Ferrum is used as a reference target, not a dependency. Not certification.");
    println!("Warnings are for humans; this command does not fail the build.");
    println!();
    println!("workload: {}", outcome.workload.join(", "));
    println!(
        "baseline  {}  wall_ms={:.1}  rss_kb={}  error_rate={:.2} ({}/{})",
        outcome.baseline.label,
        outcome.baseline.wall_ms,
        outcome
            .baseline
            .rss_kb
            .map(|v| v.to_string())
            .unwrap_or_else(|| "n/a".into()),
        outcome.baseline.error_rate,
        outcome.baseline.errors,
        outcome.baseline.requests
    );
    println!(
        "candidate {}  wall_ms={:.1}  rss_kb={}  error_rate={:.2} ({}/{})",
        outcome.candidate.label,
        outcome.candidate.wall_ms,
        outcome
            .candidate
            .rss_kb
            .map(|v| v.to_string())
            .unwrap_or_else(|| "n/a".into()),
        outcome.candidate.error_rate,
        outcome.candidate.errors,
        outcome.candidate.requests
    );
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
        } else {
            "ok"
        };
        println!("  {mark}  {} {pct}", d.name);
    }
    if outcome.warning {
        println!();
        println!(
            "threshold {:+}% — human review, not a red X:",
            outcome.threshold_pct
        );
        for w in &outcome.warnings {
            println!("  - {w}");
        }
    }
}

fn print_tests(tests: &[common::report::TestCaseResult], color: bool) {
    for t in tests {
        let mark = status_mark(t.status, color);
        match &t.error {
            Some(err) if t.status != TestStatus::Pass => {
                println!("  {mark}  {} — {err}", t.name);
            }
            _ => println!("  {mark}  {}", t.name),
        }
    }
}

fn print_discovery_text(d: &crate::discover::Discovery) {
    println!("Helix verify — GA4GH discovery (not certification)");
    println!("endpoint: {}", d.endpoint);
    println!("Helix tests behavior against the GA4GH spec, independent of implementation.");
    println!("Ferrum is used as a reference target, not a dependency.");
    println!();
    for kind in VERIFY_ORDER {
        match d.get(kind) {
            Some(s) => println!("{:<8} found   {}", kind.as_str(), s.base_url),
            None => println!("{:<8} missing", kind.as_str()),
        }
    }
    if d.found.is_empty() {
        println!("\nNo Stage 1 APIs (DRS, WES, TES, TRS, htsget) answered.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::{DiscoveredService, Discovery};
    use common::report::{ComplianceLevel, ServiceReport, TestCaseResult, TestCategory};

    fn pass_drs() -> ServiceReport {
        ServiceReport {
            service: ServiceKind::Drs,
            tests: vec![TestCaseResult::pass(
                "DRS object endpoint reachable",
                ComplianceLevel::Level0,
                TestCategory::Other,
            )],
        }
    }

    #[test]
    fn skip_is_never_green() {
        let skip = status_mark(TestStatus::Skip, true);
        assert!(skip.contains("SKIP"));
        assert!(!skip.contains("32m"), "skip must not use green: {skip}");
        assert_eq!(status_mark(TestStatus::Pass, false), "PASS");
        assert!(status_mark(TestStatus::Pass, true).contains("32m"));
        assert!(status_mark(TestStatus::Fail, true).contains("31m"));
    }

    #[test]
    fn json_shape_is_helixtest_overall_report() {
        let outcome = VerifyOutcome {
            discovery: Discovery {
                endpoint: "http://127.0.0.1:9".into(),
                found: vec![
                    DiscoveredService {
                        kind: Ga4ghService::Drs,
                        base_url: "http://127.0.0.1:9".into(),
                    },
                    DiscoveredService {
                        kind: Ga4ghService::Wes,
                        base_url: "http://127.0.0.1:9/ga4gh/wes/v1".into(),
                    },
                ],
                missing: vec![Ga4ghService::Tes, Ga4ghService::Trs, Ga4ghService::Htsget],
            },
            drs: pass_drs(),
        };
        let report = overall_report(&outcome);
        let v = serde_json::to_value(&report).unwrap();
        assert!(v.get("discovery").is_none());
        assert!(v.get("services").is_some());
        assert_eq!(v["skipped_services"][0]["service"].as_str(), Some("Wes"));
        assert_eq!(
            v["services"][0]["tests"][0]["status"].as_str(),
            Some("pass")
        );
        assert_eq!(v["services"][0]["tests"][0]["passed"].as_bool(), Some(true));
        assert!(!report.has_failures());
    }
}
