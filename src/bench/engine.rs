// SPDX-License-Identifier: Apache-2.0
//! Repeatable measurement: warmup (discarded) then measured runs. Not GIAB.

use anyhow::{ensure, Result};
use reqwest::Client;

use super::metadata::BenchMetadata;
use super::stats::LatencyStats;
use super::workload::{run_once, MeasuredRun};
use super::{MeasureConfig, Sample};

pub async fn measure(
    client: &Client,
    endpoint: &str,
    label: &str,
    cfg: &MeasureConfig,
) -> Result<Sample> {
    ensure!(cfg.repetitions >= 1, "--repetitions must be >= 1");
    let base = crate::discover::normalize_endpoint(endpoint)?;
    let metadata = BenchMetadata::capture(&base, label, cfg);

    for _ in 0..cfg.warmup {
        let _ = run_once(client, &base, cfg.collect_rss).await;
    }

    let mut runs = Vec::with_capacity(cfg.repetitions as usize);
    for _ in 0..cfg.repetitions {
        runs.push(run_once(client, &base, cfg.collect_rss).await);
    }
    Ok(aggregate(metadata, runs))
}

pub(crate) fn aggregate(metadata: BenchMetadata, runs: Vec<MeasuredRun>) -> Sample {
    let walls: Vec<f64> = runs.iter().map(|r| r.wall_ms).collect();
    let latency = LatencyStats::from_samples(&walls);
    let errors: u32 = runs.iter().map(|r| r.errors).sum();
    let requests: u32 = runs.iter().map(|r| r.requests).sum();
    let bytes: u64 = runs.iter().map(|r| r.bytes).sum();
    let rss_kb = runs.iter().filter_map(|r| r.rss_kb).max();
    Sample {
        label: metadata.target_label.clone(),
        endpoint: metadata.target_url.clone(),
        wall_ms: latency.median_ms,
        rss_kb,
        requests,
        errors,
        error_rate: if requests == 0 {
            0.0
        } else {
            f64::from(errors) / f64::from(requests)
        },
        bytes,
        metadata,
        latency,
        runs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::metadata::RuntimeInfo;
    use crate::bench::workload::{WORKLOAD_ID, WORKLOAD_VERSION};
    use crate::bench::MeasureConfig;

    fn meta() -> BenchMetadata {
        BenchMetadata {
            helix_version: "0.1.0".into(),
            workload_id: WORKLOAD_ID.into(),
            workload_version: WORKLOAD_VERSION.into(),
            target_url: "http://127.0.0.1:9".into(),
            target_label: "fixture".into(),
            timestamp: "2026-09-04T00:00:00Z".into(),
            os: "linux".into(),
            arch: "x86_64".into(),
            runtime: RuntimeInfo {
                rust_msrv: "1.88".into(),
                http_request_timeout_secs: 5,
                http_connect_timeout_secs: 3,
                rss_source: "unavailable".into(),
            },
            repetitions: 3,
            warmup: 1,
        }
    }

    fn run(wall: f64, errors: u32, bytes: u64) -> MeasuredRun {
        MeasuredRun {
            wall_ms: wall,
            bytes,
            requests: 3,
            errors,
            rss_kb: Some(100),
        }
    }

    #[test]
    fn aggregate_uses_median_and_totals() {
        let s = aggregate(
            meta(),
            vec![run(10.0, 0, 10), run(30.0, 1, 20), run(20.0, 0, 30)],
        );
        assert!((s.wall_ms - 20.0).abs() < 1e-12);
        assert_eq!(s.requests, 9);
        assert_eq!(s.errors, 1);
        assert!((s.error_rate - 1.0 / 9.0).abs() < 1e-12);
        assert_eq!(s.bytes, 60);
        assert_eq!(s.latency.min_ms, 10.0);
        assert_eq!(s.latency.max_ms, 30.0);
        assert!(s.latency.p95_ms.is_none());
        assert_eq!(s.metadata.workload_id, WORKLOAD_ID);
        assert_eq!(s.runs.len(), 3);
    }

    #[test]
    fn measure_config_defaults() {
        let c = MeasureConfig::default();
        assert_eq!(c.warmup, 1);
        assert_eq!(c.repetitions, 5);
        assert!(c.collect_rss);
    }

    #[tokio::test]
    async fn warmup_runs_are_not_in_stats() {
        use crate::discover::http_client;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        for p in ["/health", "/ga4gh/drs/v1/service-info"] {
            Mock::given(method("GET"))
                .and(path(p))
                .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(500).set_body_string("warm"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string("obj"))
            .mount(&server)
            .await;

        let client = http_client().unwrap();
        let cfg = MeasureConfig {
            warmup: 1,
            repetitions: 1,
            collect_rss: false,
        };
        let s = measure(&client, &server.uri(), "w", &cfg).await.unwrap();
        assert_eq!(s.errors, 0, "warmup 500 must not count: {:?}", s.runs);
        assert_eq!(s.requests, 3);
        assert_eq!(s.metadata.warmup, 1);
        assert_eq!(s.metadata.repetitions, 1);
        assert_eq!(s.metadata.workload_id, WORKLOAD_ID);
    }
}
