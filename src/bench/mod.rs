// SPDX-License-Identifier: Apache-2.0
//! Stage 4: repeatable client-side HTTP measurement vs two endpoints.
//! Not a publication benchmark, not HELIOS, not Ferrum-GA4GH-Demo hap.py / GIAB.
//! HelixTest stays separate. Percent diffs are not statistical significance.

mod analysis;
mod engine;
mod metadata;
mod rss;
mod stats;
mod workload;

use serde::Serialize;

pub use analysis::{
    analyze, BenchAnalysis, DistributionChange, MEASUREMENT_MEANS, MIN_DISTRIBUTION_RUNS,
    REGRESSION_MEANS, WARNING_DOES_NOT_MEAN, WARNING_MEANS,
};
pub use metadata::{
    compare_environments, BenchMetadata, EnvironmentComparison, RuntimeInfo,
    HTTP_CONNECT_TIMEOUT_SECS, HTTP_REQUEST_TIMEOUT_SECS,
};
pub use stats::{LatencyStats, P95_MIN_REPETITIONS};
pub use workload::{
    request_lines, run_once, MeasuredRun, SMOKE_REQUESTS, TINY_WORKLOAD, WORKLOAD_ID,
    WORKLOAD_VERSION,
};

use engine::measure;

/// Default: warn in output/CI comment if a metric is more than 10% worse. Never fail the job.
pub const DEFAULT_THRESHOLD_PCT: f64 = 10.0;
pub const DEFAULT_WARMUP: u32 = 1;
pub const DEFAULT_REPETITIONS: u32 = 5;

#[derive(Debug, Clone)]
pub struct MeasureConfig {
    pub warmup: u32,
    pub repetitions: u32,
    pub collect_rss: bool,
}

impl Default for MeasureConfig {
    fn default() -> Self {
        Self {
            warmup: DEFAULT_WARMUP,
            repetitions: DEFAULT_REPETITIONS,
            collect_rss: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchRequest {
    pub baseline_url: String,
    pub candidate_url: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub threshold_pct: f64,
    pub config: MeasureConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub label: String,
    pub endpoint: String,
    /// Primary wall time: **median** of measured runs (ms). Kept as `wall_ms`
    /// so helix-action comments stay readable.
    pub wall_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rss_kb: Option<u64>,
    pub requests: u32,
    pub errors: u32,
    pub error_rate: f64,
    pub bytes: u64,
    pub metadata: BenchMetadata,
    pub latency: LatencyStats,
    pub runs: Vec<MeasuredRun>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricDiff {
    pub name: String,
    /// Percent change candidate vs baseline. `None` if baseline was 0 and candidate is not.
    pub pct: Option<f64>,
    pub worse: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchOutcome {
    pub workload_id: String,
    pub workload_version: String,
    pub workload: Vec<String>,
    pub threshold_pct: f64,
    pub warning: bool,
    pub environment: EnvironmentComparison,
    pub analysis: BenchAnalysis,
    pub baseline: Sample,
    pub candidate: Sample,
    pub diff: Vec<MetricDiff>,
    pub warnings: Vec<String>,
    pub note: String,
}

pub async fn run_bench(req: &BenchRequest) -> anyhow::Result<BenchOutcome> {
    let client = crate::discover::http_client()?;
    let baseline = measure(&client, &req.baseline_url, &req.baseline_label, &req.config).await?;
    let candidate = measure(
        &client,
        &req.candidate_url,
        &req.candidate_label,
        &req.config,
    )
    .await?;
    Ok(compare(baseline, candidate, req.threshold_pct))
}

pub fn compare(baseline: Sample, candidate: Sample, threshold_pct: f64) -> BenchOutcome {
    let environment = compare_environments(&baseline.metadata, &candidate.metadata);
    let analysis = analyze(&baseline, &candidate, &environment, threshold_pct);
    let mut warnings = Vec::new();

    if !environment.comparable {
        warnings.push(format!(
            "environments are marked incomparable ({}) — percent diffs are shown, not a same-host regression",
            environment
                .incomparable_reason
                .as_deref()
                .unwrap_or("unspecified")
        ));
    }

    let mut diff = Vec::new();
    for c in &analysis.changes {
        if c.metric == "median_ms" && c.available {
            diff.push(MetricDiff {
                name: "wall_ms".into(),
                pct: c.pct,
                worse: c.worse,
            });
        }
        if c.available {
            diff.push(MetricDiff {
                name: c.metric.clone(),
                pct: c.pct,
                worse: c.worse,
            });
        }
        if c.worse {
            match c.pct {
                Some(p) => warnings.push(format!(
                    "{metric} {p:+.1}% exceeds {threshold_pct}% inspect threshold (warning: human inspection, not a verification failure)",
                    metric = c.metric
                )),
                None => warnings.push(format!(
                    "{metric} rose from 0 (baseline) to {cand} — warning for inspection, not a fail",
                    metric = c.metric,
                    cand = c.candidate.unwrap_or(0.0)
                )),
            }
        }
    }

    BenchOutcome {
        workload_id: WORKLOAD_ID.to_string(),
        workload_version: WORKLOAD_VERSION.to_string(),
        workload: request_lines(),
        threshold_pct,
        warning: analysis.warning || !environment.comparable,
        environment,
        analysis,
        baseline,
        candidate,
        diff,
        warnings,
        note: "Distribution compare of http.drs.smoke.v1 (median / p95 where available / error-rate / optional RSS). Measurement, warning, and regression are separate. A warning means performance changed enough to merit human inspection; it does not mean the implementation is incorrect. Not a verification failure, not a significance test, not GIAB/Demo hap.py, not HELIOS. Threshold warnings do not fail CI.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::engine::aggregate;
    use crate::bench::metadata::RuntimeInfo;

    fn dist(label: &str, wall: f64, rss: Option<u64>, errors: u32, os: &str) -> Sample {
        let metadata = BenchMetadata {
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
            repetitions: 5,
            warmup: 0,
        };
        aggregate(
            metadata,
            (0..5)
                .map(|_| MeasuredRun {
                    wall_ms: wall,
                    bytes: 100,
                    requests: 3,
                    errors,
                    rss_kb: rss,
                })
                .collect(),
        )
    }

    #[test]
    fn twenty_percent_slower_warns_does_not_imply_fail() {
        let out = compare(
            dist("vX", 100.0, Some(1000), 0, "linux"),
            dist("vY", 120.0, Some(1000), 0, "linux"),
            10.0,
        );
        assert!(out.warning);
        assert!(out.analysis.regression);
        assert!(!out.analysis.verification_failure);
        assert_eq!(out.analysis.warning_means, WARNING_MEANS);
        assert!(out.environment.comparable);
        assert!(out.warnings.iter().any(|w| w.contains("median_ms")));
        let wall = out.diff.iter().find(|d| d.name == "wall_ms").unwrap();
        assert!((wall.pct.unwrap() - 20.0).abs() < 1e-9);
        assert!(wall.worse);
        assert_eq!(out.workload_id, WORKLOAD_ID);
        assert!(!out.note.contains("statistically significant"));
    }

    #[test]
    fn five_percent_slower_is_quiet() {
        let out = compare(
            dist("vX", 100.0, Some(1000), 0, "linux"),
            dist("vY", 105.0, Some(1000), 0, "linux"),
            10.0,
        );
        assert!(!out.warning);
        assert!(!out.analysis.regression);
        assert!(out.analysis.measurement);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn new_errors_from_zero_warn_without_latency_regression() {
        let out = compare(
            dist("vX", 100.0, None, 0, "linux"),
            dist("vY", 100.0, None, 1, "linux"),
            10.0,
        );
        assert!(out.warning);
        assert!(!out.analysis.regression);
        assert!(!out.analysis.verification_failure);
        assert!(out.warnings.iter().any(|w| w.contains("error_rate")));
    }

    #[test]
    fn different_os_is_marked_and_threshold_is_not_applied() {
        let out = compare(
            dist("vX", 100.0, Some(1000), 0, "linux"),
            dist("vY", 120.0, Some(1000), 0, "macos"),
            10.0,
        );
        assert!(!out.environment.comparable);
        assert!(out.warning);
        assert!(!out.analysis.regression);
        assert!(out.warnings.iter().any(|w| w.contains("incomparable")));
        let wall = out.diff.iter().find(|d| d.name == "wall_ms").unwrap();
        assert!((wall.pct.unwrap() - 20.0).abs() < 1e-9);
        assert!(
            !wall.worse,
            "threshold must not fire across marked environments"
        );
        assert!(!out.warnings.iter().any(|w| w.contains("exceeds")));
    }
}
