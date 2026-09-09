// SPDX-License-Identifier: Apache-2.0
//! Sample statistics for one measured series. Not a significance test.

use serde::Serialize;

/// Sample p95 is reported only when this many measured runs exist.
/// That is a sample-size floor so the percentile is not a single noisy point.
/// It is **not** a claim of statistical significance.
pub const P95_MIN_REPETITIONS: u32 = 20;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LatencyStats {
    pub median_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    /// Sample 95th percentile. Omitted below [`P95_MIN_REPETITIONS`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95_ms: Option<f64>,
    /// Always present when p95 is omitted or included: this is a sample
    /// percentile, not a confidence interval and not a significance test.
    pub p95_note: String,
}

impl LatencyStats {
    pub fn from_samples(values_ms: &[f64]) -> Self {
        assert!(
            !values_ms.is_empty(),
            "latency stats need at least one measured run"
        );
        let mut sorted = values_ms.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len() as u32;
        let (p95_ms, p95_note) = if n >= P95_MIN_REPETITIONS {
            (
                Some(percentile_nearest_rank(&sorted, 0.95)),
                "sample percentile of this series; not a significance test".to_string(),
            )
        } else {
            (
                None,
                format!(
                    "omitted: sample p95 is reported only at >= {P95_MIN_REPETITIONS} measured runs (not a significance test)"
                ),
            )
        };
        Self {
            median_ms: median(&sorted),
            min_ms: sorted[0],
            max_ms: *sorted.last().expect("non-empty"),
            p95_ms,
            p95_note,
        }
    }
}

/// Nearest-rank percentile on a non-empty sorted slice. Rank = ceil(p * n).
pub(crate) fn percentile_nearest_rank(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    let rank = ((p * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1]
}

pub(crate) fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// Percent change candidate vs baseline. `None` if baseline is 0 and candidate is not.
pub(crate) fn percent_change(base: f64, cand: f64) -> (Option<f64>, bool) {
    if base == 0.0 {
        if cand == 0.0 {
            (Some(0.0), false)
        } else {
            (None, true)
        }
    } else {
        let pct = (cand - base) / base * 100.0;
        (Some(pct), pct > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_and_even() {
        let odd = LatencyStats::from_samples(&[3.0, 1.0, 2.0]);
        assert!((odd.median_ms - 2.0).abs() < 1e-12);
        assert!((odd.min_ms - 1.0).abs() < 1e-12);
        assert!((odd.max_ms - 3.0).abs() < 1e-12);
        assert!(odd.p95_ms.is_none());
        assert!(odd.p95_note.contains("not a significance test"));
        assert!(!odd.p95_note.contains("statistically significant"));

        let even = LatencyStats::from_samples(&[1.0, 2.0, 3.0, 4.0]);
        assert!((even.median_ms - 2.5).abs() < 1e-12);
    }

    #[test]
    fn p95_only_at_twenty_runs() {
        let small: Vec<f64> = (1..=19).map(|i| i as f64).collect();
        assert!(LatencyStats::from_samples(&small).p95_ms.is_none());

        let twenty: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let s = LatencyStats::from_samples(&twenty);
        // nearest-rank: ceil(0.95 * 20) = 19 → value 19
        assert_eq!(s.p95_ms, Some(19.0));
        assert!(s.p95_note.contains("not a significance test"));
        assert!(!s.p95_note.contains("statistically significant"));
    }
}
