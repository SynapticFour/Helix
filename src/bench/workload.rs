// SPDX-License-Identifier: Apache-2.0
//! Fixed workload `http.drs.smoke.v1`: three small GETs.
//! Same request count as Demo DRS micro `n=3`, not Demo hap.py / GIAB.

use reqwest::Client;
use serde::Serialize;
use std::time::Instant;

use super::rss::rss_kb;

/// Stable workload id. Bump the suffix when the request set changes.
pub const WORKLOAD_ID: &str = "http.drs.smoke.v1";
pub const WORKLOAD_VERSION: &str = "1";

/// Fixed tiny smoke (extend later). Paths work on Ferrum gateway and split mocks.
pub const SMOKE_REQUESTS: [(&str, &str); 3] = [
    ("GET", "/health"),
    ("GET", "/ga4gh/drs/v1/service-info"),
    ("GET", "/ga4gh/drs/v1/objects/test-object-1"),
];

/// Legacy alias used by prove greps and older comments.
pub const TINY_WORKLOAD: [(&str, &str); 3] = SMOKE_REQUESTS;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MeasuredRun {
    pub wall_ms: f64,
    pub bytes: u64,
    pub requests: u32,
    pub errors: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rss_kb: Option<u64>,
}

pub fn request_lines() -> Vec<String> {
    SMOKE_REQUESTS
        .iter()
        .map(|(m, p)| format!("{m} {p}"))
        .collect()
}

/// One execution of the fixed three-request workload.
pub async fn run_once(client: &Client, base: &str, collect_rss: bool) -> MeasuredRun {
    let start = Instant::now();
    let mut errors = 0u32;
    let mut bytes = 0u64;
    let mut peak_rss = if collect_rss { rss_kb() } else { None };

    for (method, path) in SMOKE_REQUESTS {
        debug_assert_eq!(method, "GET");
        let url = format!("{base}{path}");
        match client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                match crate::http_safety::read_body_capped(
                    resp,
                    crate::http_safety::MAX_RESPONSE_BYTES,
                )
                .await
                {
                    Ok(body) => {
                        bytes += body.len() as u64;
                        if !status.is_success() {
                            errors += 1;
                        }
                    }
                    Err(_) => {
                        errors += 1;
                    }
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
        if collect_rss {
            if let Some(now) = rss_kb() {
                peak_rss = Some(peak_rss.map_or(now, |p| p.max(now)));
            }
        }
    }

    MeasuredRun {
        wall_ms: start.elapsed().as_secs_f64() * 1000.0,
        bytes,
        requests: SMOKE_REQUESTS.len() as u32,
        errors,
        rss_kb: peak_rss,
    }
}
