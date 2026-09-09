// SPDX-License-Identifier: Apache-2.0
//! Distribution compare of two measured series.
//!
//! **Measurement**, **warning**, and **regression** are separate. A bench
//! warning is not a verification failure and does not mean the implementation
//! is incorrect. Not a significance test. Not `helix compare` NEW_FAIL.

use serde::Serialize;

use super::stats::{median, percent_change};
use super::{EnvironmentComparison, Sample};

/// Both series need at least this many measured runs to compare distributions.
/// A single wall-clock sample is **measurement** only.
pub const MIN_DISTRIBUTION_RUNS: usize = 2;

pub const MEASUREMENT_MEANS: &str = "A recorded series of repeated runs and the distribution deltas computed from them. Not a verdict.";

pub const WARNING_MEANS: &str = "Performance changed enough to merit human inspection.";

pub const WARNING_DOES_NOT_MEAN: &str = "Implementation is incorrect.";

pub const REGRESSION_MEANS: &str = "The candidate latency distribution (median of measured runs) is worse than baseline beyond the inspect threshold, from a repeated-measurement compare. Not a verification failure and not a claim that the implementation is incorrect.";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DistributionChange {
    pub metric: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct: Option<f64>,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omitted_reason: Option<String>,
    /// Worse than the inspect threshold on a comparable distribution compare.
    pub worse: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BenchAnalysis {
    /// Always true after a compare: facts were recorded.
    pub measurement: bool,
    /// Inspect-threshold crossed on a comparable distribution compare.
    pub warning: bool,
    /// Median latency distribution is worse beyond the threshold (requires
    /// repeated measurements). Never a verification failure.
    pub regression: bool,
    /// Always false. Bench does not fail `helix verify`.
    pub verification_failure: bool,
    pub distribution_compare: bool,
    pub measurement_means: String,
    pub warning_means: String,
    pub warning_does_not_mean: String,
    pub regression_means: String,
    pub changes: Vec<DistributionChange>,
}

pub fn analyze(
    baseline: &Sample,
    candidate: &Sample,
    environment: &EnvironmentComparison,
    threshold_pct: f64,
) -> BenchAnalysis {
    let distribution_compare = baseline.runs.len() >= MIN_DISTRIBUTION_RUNS
        && candidate.runs.len() >= MIN_DISTRIBUTION_RUNS;
    let apply = environment.comparable && distribution_compare;

    let median_change = compared(
        "median_ms",
        Some(baseline.latency.median_ms),
        Some(candidate.latency.median_ms),
        None,
        threshold_pct,
        apply,
    );
    let p95_change = match (baseline.latency.p95_ms, candidate.latency.p95_ms) {
        (Some(b), Some(c)) => compared("p95_ms", Some(b), Some(c), None, threshold_pct, apply),
        _ => omitted(
            "p95_ms",
            "sample p95 compared only when both series have >= 20 measured runs (not a significance test)",
        ),
    };
    let error_change = compared(
        "error_rate",
        Some(baseline.error_rate),
        Some(candidate.error_rate),
        None,
        threshold_pct,
        apply,
    );
    let rss_change = match (median_rss(baseline), median_rss(candidate)) {
        (Some(b), Some(c)) => compared("rss_kb", Some(b), Some(c), None, threshold_pct, apply),
        _ => omitted(
            "rss_kb",
            "optional resource metric: median per-run Helix process RSS compared only when both series recorded it",
        ),
    };

    let changes = vec![median_change, p95_change, error_change, rss_change];
    let warning = apply && changes.iter().any(|c| c.worse);
    let regression = apply && changes.iter().any(|c| c.metric == "median_ms" && c.worse);

    BenchAnalysis {
        measurement: true,
        warning,
        regression,
        verification_failure: false,
        distribution_compare,
        measurement_means: MEASUREMENT_MEANS.to_string(),
        warning_means: WARNING_MEANS.to_string(),
        warning_does_not_mean: WARNING_DOES_NOT_MEAN.to_string(),
        regression_means: REGRESSION_MEANS.to_string(),
        changes,
    }
}

fn median_rss(sample: &Sample) -> Option<f64> {
    let mut vals: Vec<f64> = sample
        .runs
        .iter()
        .filter_map(|r| r.rss_kb.map(|v| v as f64))
        .collect();
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(median(&vals))
}

fn omitted(metric: &str, reason: &str) -> DistributionChange {
    DistributionChange {
        metric: metric.into(),
        baseline: None,
        candidate: None,
        pct: None,
        available: false,
        omitted_reason: Some(reason.into()),
        worse: false,
    }
}

fn compared(
    metric: &str,
    baseline: Option<f64>,
    candidate: Option<f64>,
    omitted_reason: Option<String>,
    threshold_pct: f64,
    apply: bool,
) -> DistributionChange {
    let (b, c) = match (baseline, candidate) {
        (Some(b), Some(c)) => (b, c),
        _ => {
            return omitted(
                metric,
                omitted_reason
                    .as_deref()
                    .unwrap_or("metric unavailable on one or both series"),
            );
        }
    };
    let (pct, rose) = percent_change(b, c);
    let over = match pct {
        Some(p) => p > threshold_pct,
        None => rose,
    };
    DistributionChange {
        metric: metric.into(),
        baseline: Some(b),
        candidate: Some(c),
        pct,
        available: true,
        omitted_reason: None,
        worse: apply && over,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::engine::aggregate;
    use crate::bench::metadata::{
        RuntimeInfo, HTTP_CONNECT_TIMEOUT_SECS, HTTP_REQUEST_TIMEOUT_SECS,
    };
    use crate::bench::workload::{MeasuredRun, WORKLOAD_ID, WORKLOAD_VERSION};
    use crate::bench::{compare_environments, BenchMetadata};

    fn meta(label: &str, os: &str, n: u32) -> BenchMetadata {
        BenchMetadata {
            helix_version: "0.1.0".into(),
            workload_id: WORKLOAD_ID.into(),
            workload_version: WORKLOAD_VERSION.into(),
            target_url: "http://127.0.0.1:9".into(),
            target_label: label.into(),
            timestamp: "2026-09-04T00:00:00Z".into(),
            os: os.into(),
            arch: "x86_64".into(),
            runtime: RuntimeInfo {
                rust_msrv: "1.88".into(),
                http_request_timeout_secs: HTTP_REQUEST_TIMEOUT_SECS,
                http_connect_timeout_secs: HTTP_CONNECT_TIMEOUT_SECS,
                rss_source: "unavailable".into(),
            },
            repetitions: n,
            warmup: 0,
        }
    }

    fn run(wall: f64, errors: u32, rss: Option<u64>) -> MeasuredRun {
        MeasuredRun {
            wall_ms: wall,
            bytes: 10,
            requests: 3,
            errors,
            rss_kb: rss,
        }
    }

    fn series(label: &str, walls: &[f64], errors: &[u32], rss: &[Option<u64>], os: &str) -> Sample {
        assert_eq!(walls.len(), errors.len());
        assert_eq!(walls.len(), rss.len());
        let runs: Vec<MeasuredRun> = walls
            .iter()
            .zip(errors.iter())
            .zip(rss.iter())
            .map(|((w, e), r)| run(*w, *e, *r))
            .collect();
        let n = runs.len() as u32;
        aggregate(meta(label, os, n), runs)
    }

    fn env(a: &Sample, b: &Sample) -> EnvironmentComparison {
        compare_environments(&a.metadata, &b.metadata)
    }

    #[test]
    fn identical_distributions_are_measurement_only() {
        let walls = [10.0, 12.0, 11.0, 10.5, 11.5];
        let z = [0, 0, 0, 0, 0];
        let rss = [Some(100), Some(100), Some(100), Some(100), Some(100)];
        let a = series("vX", &walls, &z, &rss, "linux");
        let b = series("vY", &walls, &z, &rss, "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.measurement);
        assert!(out.distribution_compare);
        assert!(!out.warning);
        assert!(!out.regression);
        assert!(!out.verification_failure);
        assert_eq!(out.warning_means, WARNING_MEANS);
        assert_eq!(out.warning_does_not_mean, WARNING_DOES_NOT_MEAN);
        for c in &out.changes {
            assert!(!c.worse, "{}", c.metric);
        }
        let median = out
            .changes
            .iter()
            .find(|c| c.metric == "median_ms")
            .unwrap();
        assert!(median.available);
        assert_eq!(median.pct, Some(0.0));
    }

    #[test]
    fn median_twenty_percent_slower_is_warning_and_regression_not_verify_fail() {
        let base = [100.0, 100.0, 100.0, 100.0, 100.0];
        let cand = [120.0, 120.0, 120.0, 120.0, 120.0];
        let z = [0, 0, 0, 0, 0];
        let rss = [None; 5];
        let a = series("vX", &base, &z, &rss, "linux");
        let b = series("vY", &cand, &z, &rss, "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.warning);
        assert!(out.regression);
        assert!(!out.verification_failure);
        assert_eq!(
            out.warning_means,
            "Performance changed enough to merit human inspection."
        );
        assert_eq!(out.warning_does_not_mean, "Implementation is incorrect.");
        let median = out
            .changes
            .iter()
            .find(|c| c.metric == "median_ms")
            .unwrap();
        assert!((median.pct.unwrap() - 20.0).abs() < 1e-9);
        assert!(median.worse);
        assert!(
            !out.changes
                .iter()
                .find(|c| c.metric == "p95_ms")
                .unwrap()
                .available
        );
    }

    #[test]
    fn five_percent_median_shift_is_not_a_warning() {
        let base = [100.0, 100.0, 100.0, 100.0, 100.0];
        let cand = [105.0, 105.0, 105.0, 105.0, 105.0];
        let z = [0; 5];
        let rss = [None; 5];
        let a = series("vX", &base, &z, &rss, "linux");
        let b = series("vY", &cand, &z, &rss, "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(!out.warning);
        assert!(!out.regression);
    }

    #[test]
    fn p95_tail_shift_warns_without_median_regression() {
        let base = vec![100.0; 20];
        let mut cand = vec![100.0; 18];
        cand.push(200.0);
        cand.push(200.0);
        let z = vec![0; 20];
        let rss = vec![None; 20];
        let a = series("vX", &base, &z, &rss, "linux");
        let b = series("vY", &cand, &z, &rss, "linux");
        assert!((a.latency.median_ms - 100.0).abs() < 1e-9);
        assert!((b.latency.median_ms - 100.0).abs() < 1e-9);
        assert_eq!(a.latency.p95_ms, Some(100.0));
        assert_eq!(b.latency.p95_ms, Some(200.0));
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.warning);
        assert!(!out.regression);
        assert!(!out.verification_failure);
        let p95 = out.changes.iter().find(|c| c.metric == "p95_ms").unwrap();
        assert!(p95.available);
        assert!(p95.worse);
        let median = out
            .changes
            .iter()
            .find(|c| c.metric == "median_ms")
            .unwrap();
        assert!(!median.worse);
    }

    #[test]
    fn error_rate_rise_warns_without_latency_regression() {
        let walls = [10.0, 10.0, 10.0, 10.0, 10.0];
        let a = series("vX", &walls, &[0; 5], &[None; 5], "linux");
        let b = series("vY", &walls, &[1, 1, 1, 1, 1], &[None; 5], "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.warning);
        assert!(!out.regression);
        let err = out
            .changes
            .iter()
            .find(|c| c.metric == "error_rate")
            .unwrap();
        assert!(err.worse);
        assert!(err.available);
    }

    #[test]
    fn rss_median_rise_warns_where_available() {
        let walls = [10.0, 10.0, 10.0, 10.0, 10.0];
        let z = [0; 5];
        let a = series("vX", &walls, &z, &[Some(100); 5], "linux");
        let b = series("vY", &walls, &z, &[Some(150); 5], "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.warning);
        assert!(!out.regression);
        let rss = out.changes.iter().find(|c| c.metric == "rss_kb").unwrap();
        assert!(rss.available);
        assert!((rss.pct.unwrap() - 50.0).abs() < 1e-9);
        assert!(rss.worse);
    }

    #[test]
    fn single_wall_clock_sample_is_not_a_distribution_compare() {
        let a = series("vX", &[100.0], &[0], &[None], "linux");
        let b = series("vY", &[200.0], &[0], &[None], "linux");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(out.measurement);
        assert!(!out.distribution_compare);
        assert!(!out.warning);
        assert!(!out.regression);
        let median = out
            .changes
            .iter()
            .find(|c| c.metric == "median_ms")
            .unwrap();
        assert!(median.available);
        assert!((median.pct.unwrap() - 100.0).abs() < 1e-9);
        assert!(
            !median.worse,
            "must not treat a single sample as regression"
        );
    }

    #[test]
    fn incomparable_environments_do_not_claim_regression() {
        let walls = [100.0; 5];
        let cand = [200.0; 5];
        let z = [0; 5];
        let a = series("vX", &walls, &z, &[None; 5], "linux");
        let b = series("vY", &cand, &z, &[None; 5], "macos");
        let out = analyze(&a, &b, &env(&a, &b), 10.0);
        assert!(!out.warning);
        assert!(!out.regression);
        assert!(!out.verification_failure);
        let median = out
            .changes
            .iter()
            .find(|c| c.metric == "median_ms")
            .unwrap();
        assert!(median.available);
        assert!(!median.worse);
    }
}
