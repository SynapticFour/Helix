// SPDX-License-Identifier: Apache-2.0
//! HelixTest conformance adapter.
//!
//! HelixTest already runs; this module productizes it behind one call site.
//! HelixTest stays a separate git root (D1). This crate path-depends on the
//! pinned sibling checkout; it does not vendor or merge HelixTest.
//!
//! Ferrum is not imported here. Ferrum may appear as a test *target* URL, a
//! fixture description, or a documentation reference — never as a crate.
//!
//! Not HELIOS (no signed evidence / RO-Crate / PDF).

use anyhow::Result;
use common::config::{AuthChecksConfig, ServiceConfig, SubsetConfig, TestConfig};
use common::http::HttpClient;
use common::report::ServiceReport;
use framework::drs::run_drs_checks;
use framework::wes::run_wes_checks;
use framework::{Features, Mode};

use crate::model::{Target, VerificationResult, VerificationRun, HELIXTEST_PIN, HELIXTEST_SHA};
use crate::profile::Capabilities;

mod translate;

pub use translate::{map_status, translate_service_report, translate_test_case};

/// Same names as HelixTest `framework::drs::run_drs_checks` / B1 mock.
pub const DRS_CHECK_NAMES: [&str; 5] = [
    "DRS object endpoint reachable",
    "DRS DrsObject OpenAPI + access_methods",
    "DRS checksum correctness",
    "DRS HTTP Range support",
    "DRS invalid object id returns 404",
];

/// Same names as HelixTest `framework::wes::run_wes_checks`. Scatter/gather is
/// skipped unless profile capabilities set `supports_scatter_gather`.
pub const WES_CHECK_NAMES: [&str; 8] = [
    "WES service-info reachable",
    "WES service-info schema (GA4GH official)",
    "WES lifecycle success echo (API may show QUEUED/INITIALIZING/RUNNING before COMPLETE)",
    "WES failure state for bad workflow",
    "WES missing inputs leads to error state",
    "WES incompatible workflow_type leads to error state",
    "WES invalid workflow leads to error state",
    "WES scatter/gather workflow",
];

fn empty_services() -> ServiceConfig {
    ServiceConfig {
        wes_url: String::new(),
        tes_url: String::new(),
        drs_url: String::new(),
        trs_url: String::new(),
        beacon_url: String::new(),
        auth_url: String::new(),
        htsget_url: None,
    }
}

fn features_from(caps: Capabilities) -> Features {
    Features {
        strict_drs_checksums: caps.strict_drs_checksums,
        supports_scatter_gather: caps.supports_scatter_gather,
        ..Default::default()
    }
}

/// Published HelixTest pin this adapter invokes ([VERSIONS.lock](../../VERSIONS.lock)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelixTestPin {
    pub tag: &'static str,
    pub sha: &'static str,
}

impl HelixTestPin {
    pub fn from_lockfile() -> Self {
        Self {
            tag: HELIXTEST_PIN,
            sha: HELIXTEST_SHA,
        }
    }
}

/// HelixTest `ServiceReport` plus Helix-native results for one adapter invocation.
#[derive(Debug, Clone)]
pub struct AdapterOutcome {
    pub pin: HelixTestPin,
    /// Original HelixTest report (adapter internal; `helix verify` JSON is VerificationRun).
    pub service_report: ServiceReport,
    /// Helix verification results. Skip is never pass.
    pub results: Vec<VerificationResult>,
}

impl AdapterOutcome {
    pub fn into_run(self, target: Target) -> VerificationRun {
        let mut run = VerificationRun::new(target);
        debug_assert_eq!(run.helixtest_version.as_deref(), Some(self.pin.tag));
        for r in self.results {
            run.push_executed(r);
        }
        run
    }
}

/// Intended seam ([ARCHITECTURE.md](../../docs/ARCHITECTURE.md) §5).
/// Only production impl is [`HelixTestAdapter`].
pub trait ConformanceAdapter: Send + Sync {
    fn pin(&self) -> HelixTestPin;

    fn run_drs(
        &self,
        base_url: &str,
    ) -> impl std::future::Future<Output = Result<AdapterOutcome>> + Send;

    fn run_wes(
        &self,
        base_url: &str,
    ) -> impl std::future::Future<Output = Result<AdapterOutcome>> + Send;
}

/// Invokes pinned HelixTest DRS and WES checks (`Mode::Generic`) and translates results.
///
/// This is the only `framework::*` call site for conformance execution.
/// Helix profile capabilities map to HelixTest `Features` only. Ferrum mode is never used.
pub struct HelixTestAdapter {
    pin: HelixTestPin,
    capabilities: Capabilities,
}

impl HelixTestAdapter {
    pub fn pinned() -> Self {
        Self {
            pin: HelixTestPin::from_lockfile(),
            capabilities: crate::profile::GENERIC.capabilities,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

impl ConformanceAdapter for HelixTestAdapter {
    fn pin(&self) -> HelixTestPin {
        self.pin
    }

    async fn run_drs(&self, base_url: &str) -> Result<AdapterOutcome> {
        let mut services = empty_services();
        services.drs_url = base_url.trim_end_matches('/').to_string();
        let cfg = TestConfig {
            services,
            subset: SubsetConfig::default(),
            auth_checks: AuthChecksConfig::default(),
        };
        // Always Generic: do not switch on WES `name`. Not Ferrum mode.
        let service_report = run_drs_checks(
            Mode::Generic,
            &features_from(self.capabilities),
            &cfg,
            &HttpClient::new(),
        )
        .await?;
        let results = translate_service_report(&service_report);
        Ok(AdapterOutcome {
            pin: self.pin,
            service_report,
            results,
        })
    }

    async fn run_wes(&self, base_url: &str) -> Result<AdapterOutcome> {
        let mut services = empty_services();
        services.wes_url = base_url.trim_end_matches('/').to_string();
        let cfg = TestConfig {
            services,
            subset: SubsetConfig::default(),
            auth_checks: AuthChecksConfig::default(),
        };
        // Public HTTP only. Mode is unused in HelixTest `run_wes_checks`.
        // Scatter/gather follows profile capabilities, not Ferrum mode.
        let service_report = run_wes_checks(
            Mode::Generic,
            &features_from(self.capabilities),
            &cfg,
            &HttpClient::new(),
        )
        .await?;
        let results = translate_service_report(&service_report);
        Ok(AdapterOutcome {
            pin: self.pin,
            service_report,
            results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_matches_versions_lock() {
        let pin = HelixTestPin::from_lockfile();
        assert_eq!(pin.tag, "v0.1.3");
        assert_eq!(pin.sha, "1832c043e1679ec283cb2113510ee33684317cce");
        assert_eq!(pin.tag, HELIXTEST_PIN);
        assert_eq!(pin.sha, HELIXTEST_SHA);
    }
}
