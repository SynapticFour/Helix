// SPDX-License-Identifier: Apache-2.0
//! Stage 4: tiny client-side HTTP workload vs two endpoints. Not a publication
//! benchmark, not HELIOS, not Ferrum-GA4GH-Demo hap.py / GIAB. HelixTest stays separate.

mod rss;
mod workload;

use serde::Serialize;

pub use workload::{run_sample, TINY_WORKLOAD};

/// Default: warn in output/CI comment if a metric is more than 10% worse. Never fail the job.
pub const DEFAULT_THRESHOLD_PCT: f64 = 10.0;

#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub label: String,
    pub endpoint: String,
    pub wall_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rss_kb: Option<u64>,
    pub requests: u32,
    pub errors: u32,
    pub error_rate: f64,
    pub bytes: u64,
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
    pub workload: Vec<String>,
    pub threshold_pct: f64,
    pub warning: bool,
    pub baseline: Sample,
    pub candidate: Sample,
    pub diff: Vec<MetricDiff>,
    pub warnings: Vec<String>,
    pub note: String,
}

pub async fn run_bench(
    baseline_url: &str,
    candidate_url: &str,
    baseline_label: &str,
    candidate_label: &str,
    threshold_pct: f64,
) -> anyhow::Result<BenchOutcome> {
    let client = crate::discover::http_client()?;
    let baseline = run_sample(&client, baseline_url, baseline_label).await?;
    let candidate = run_sample(&client, candidate_url, candidate_label).await?;
    Ok(compare(baseline, candidate, threshold_pct))
}

pub fn compare(baseline: Sample, candidate: Sample, threshold_pct: f64) -> BenchOutcome {
    let mut diff = Vec::new();
    let mut warnings = Vec::new();

    push_diff(
        &mut diff,
        &mut warnings,
        "wall_ms",
        baseline.wall_ms,
        candidate.wall_ms,
        threshold_pct,
    );
    if let (Some(b), Some(c)) = (baseline.rss_kb, candidate.rss_kb) {
        push_diff(
            &mut diff,
            &mut warnings,
            "rss_kb",
            b as f64,
            c as f64,
            threshold_pct,
        );
    }
    push_diff(
        &mut diff,
        &mut warnings,
        "error_rate",
        baseline.error_rate,
        candidate.error_rate,
        threshold_pct,
    );

    BenchOutcome {
        workload: TINY_WORKLOAD
            .iter()
            .map(|(m, p)| format!("{m} {p}"))
            .collect(),
        threshold_pct,
        warning: !warnings.is_empty(),
        baseline,
        candidate,
        diff,
        warnings,
        note: "Client-side timing of 3 small GETs. Not a GIAB/Demo hap.py benchmark, not clinical throughput, not HELIOS. Warnings do not fail CI."
            .into(),
    }
}

fn push_diff(
    diff: &mut Vec<MetricDiff>,
    warnings: &mut Vec<String>,
    name: &str,
    base: f64,
    cand: f64,
    threshold_pct: f64,
) {
    let (pct, worse) = if base == 0.0 {
        if cand == 0.0 {
            (Some(0.0), false)
        } else {
            (None, true)
        }
    } else {
        let pct = (cand - base) / base * 100.0;
        (Some(pct), pct > threshold_pct)
    };
    if worse {
        match pct {
            Some(p) => warnings.push(format!(
                "{name} {p:+.1}% exceeds {threshold_pct}% threshold"
            )),
            None => warnings.push(format!(
                "{name} rose from 0 (baseline) to {cand} — treat as warning, not a fail"
            )),
        }
    }
    diff.push(MetricDiff {
        name: name.into(),
        pct,
        worse,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(label: &str, wall: f64, rss: Option<u64>, errors: u32) -> Sample {
        Sample {
            label: label.into(),
            endpoint: "http://127.0.0.1:9".into(),
            wall_ms: wall,
            rss_kb: rss,
            requests: 3,
            errors,
            error_rate: errors as f64 / 3.0,
            bytes: 100,
        }
    }

    #[test]
    fn twenty_percent_slower_warns_does_not_imply_fail() {
        let out = compare(
            sample("vX", 100.0, Some(1000), 0),
            sample("vY", 120.0, Some(1000), 0),
            10.0,
        );
        assert!(out.warning);
        assert!(out.warnings.iter().any(|w| w.contains("wall_ms")));
        let wall = out.diff.iter().find(|d| d.name == "wall_ms").unwrap();
        assert!((wall.pct.unwrap() - 20.0).abs() < 1e-9);
        assert!(wall.worse);
    }

    #[test]
    fn five_percent_slower_is_quiet() {
        let out = compare(
            sample("vX", 100.0, Some(1000), 0),
            sample("vY", 105.0, Some(1000), 0),
            10.0,
        );
        assert!(!out.warning);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn new_errors_from_zero_warn() {
        let out = compare(
            sample("vX", 100.0, None, 0),
            sample("vY", 100.0, None, 1),
            10.0,
        );
        assert!(out.warning);
        assert!(out.warnings.iter().any(|w| w.contains("error_rate")));
    }
}
