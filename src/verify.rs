// SPDX-License-Identifier: Apache-2.0
//! Run existing HelixTest checks for discovered APIs. Stage 1 order: DRS first.
//! WES / TES / TRS / htsget checks are not wired yet. Not HELIOS.

use anyhow::Result;
use common::config::{AuthChecksConfig, ServiceConfig, SubsetConfig, TestConfig};
use common::http::HttpClient;
use common::report::{
    ComplianceLevel, ServiceKind, ServiceReport, TestCaseResult, TestCategory, TestStatus,
};
use framework::drs::run_drs_checks;
use framework::{Features, Mode};

use crate::discover::{discover, http_client, Discovery, Ga4ghService};

/// Same names as HelixTest `framework::drs::run_drs_checks` / B1 mock `DRS_CHECK_NAMES`.
pub const DRS_CHECK_NAMES: [&str; 5] = [
    "DRS object endpoint reachable",
    "DRS DrsObject OpenAPI + access_methods",
    "DRS checksum correctness",
    "DRS HTTP Range support",
    "DRS invalid object id returns 404",
];

#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    pub discovery: Discovery,
    pub drs: ServiceReport,
}

impl VerifyOutcome {
    pub fn has_failures(&self) -> bool {
        self.drs.tests.iter().any(|t| t.status == TestStatus::Fail)
    }
}

/// Discover APIs under `endpoint`, then run HelixTest DRS checks when DRS is present.
pub async fn verify(endpoint: &str) -> Result<VerifyOutcome> {
    let client = http_client()?;
    let discovery = discover(endpoint, &client).await?;
    let drs = match discovery.get(Ga4ghService::Drs) {
        Some(svc) => run_discovered_drs(&svc.base_url).await?,
        None => ServiceReport {
            service: ServiceKind::Drs,
            tests: vec![TestCaseResult::fail(
                "DRS object endpoint reachable",
                ComplianceLevel::Level0,
                TestCategory::Other,
                "DRS not discovered under this URL (no 2xx/401/403 on DRS probes)",
            )],
        },
    };
    Ok(VerifyOutcome { discovery, drs })
}

async fn run_discovered_drs(base_url: &str) -> Result<ServiceReport> {
    let cfg = TestConfig {
        services: ServiceConfig {
            wes_url: String::new(),
            tes_url: String::new(),
            drs_url: base_url.trim_end_matches('/').to_string(),
            trs_url: String::new(),
            beacon_url: String::new(),
            auth_url: String::new(),
            htsget_url: None,
        },
        subset: SubsetConfig::default(),
        auth_checks: AuthChecksConfig::default(),
    };
    let features = Features {
        strict_drs_checksums: true,
        ..Default::default()
    };
    run_drs_checks(Mode::Generic, &features, &cfg, &HttpClient::new()).await
}
