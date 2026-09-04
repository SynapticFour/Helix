// SPDX-License-Identifier: Apache-2.0
//! Three small GETs. Same count as Demo DRS micro `n=3`, not Demo's 7.6 MiB /stream.

use anyhow::Result;
use reqwest::Client;
use std::time::Instant;

use super::rss::rss_kb;
use super::Sample;

/// Fixed tiny workload (extend later). Paths work on Ferrum gateway and split mocks.
pub const TINY_WORKLOAD: [(&str, &str); 3] = [
    ("GET", "/health"),
    ("GET", "/ga4gh/drs/v1/service-info"),
    ("GET", "/ga4gh/drs/v1/objects/test-object-1"),
];

pub async fn run_sample(client: &Client, endpoint: &str, label: &str) -> Result<Sample> {
    let base = crate::discover::normalize_endpoint(endpoint)?;
    let start = Instant::now();
    let mut errors = 0u32;
    let mut bytes = 0u64;
    let mut peak_rss = rss_kb();

    for (method, path) in TINY_WORKLOAD {
        anyhow::ensure!(method == "GET", "tiny workload is GET-only");
        let url = format!("{base}{path}");
        match client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.bytes().await.unwrap_or_default();
                bytes += body.len() as u64;
                if !status.is_success() {
                    errors += 1;
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
        if let Some(now) = rss_kb() {
            peak_rss = Some(peak_rss.map_or(now, |p| p.max(now)));
        }
    }

    let n = TINY_WORKLOAD.len() as u32;
    Ok(Sample {
        label: label.to_string(),
        endpoint: base,
        wall_ms: start.elapsed().as_secs_f64() * 1000.0,
        rss_kb: peak_rss,
        requests: n,
        errors,
        error_rate: errors as f64 / f64::from(n),
        bytes,
    })
}
