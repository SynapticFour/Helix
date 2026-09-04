// SPDX-License-Identifier: Apache-2.0
//! DRS and WES verification: discover → testable? → HelixTest adapter → Helix results.
//!
//! TES / TRS / htsget checks are not wired. Discovery of those APIs is not a pass.
//! Not HELIOS.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::Result;

use crate::adapter::{ConformanceAdapter, HelixTestAdapter};
use crate::discover::{
    discover, http_client, normalize_endpoint, Detection, Discovery, Ga4ghService,
    ServiceDiscovery, Testability, VERIFY_ORDER,
};
use crate::identity::{drs_verify_specs, wes_verify_specs, CheckSpec};
use crate::model::{
    DiscoveredService, Target, VerificationCheck, VerificationResult, VerificationRun,
    HELIXTEST_PIN, HELIXTEST_SHA,
};
use crate::profile::{definition, Profile, ProfileId};

pub use crate::adapter::{DRS_CHECK_NAMES, WES_CHECK_NAMES};

#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    pub discovery: Discovery,
    pub run: VerificationRun,
}

impl VerifyOutcome {
    pub fn has_failures(&self) -> bool {
        self.run.has_failures()
    }

    /// Success: at least one pass and no fail/error. Skip-only is not a pass.
    pub fn is_success(&self) -> bool {
        self.run.overall_status().is_pass()
    }
}

/// Discover APIs under `endpoint`, then run enabled HelixTest suites when TESTABLE.
/// Default profile is [`ProfileId::Generic`].
pub async fn verify(endpoint: &str) -> Result<VerifyOutcome> {
    verify_with_profile(endpoint, ProfileId::Generic).await
}

pub async fn verify_with_profile(endpoint: &str, profile_id: ProfileId) -> Result<VerifyOutcome> {
    let profile = definition(profile_id);
    let endpoint = normalize_endpoint(endpoint)?;
    let mut run = VerificationRun::new(Target::new(&endpoint));
    run.profile = Some(profile.id.as_str().to_string());
    run.helixtest_version = Some(HELIXTEST_PIN.to_string());
    run.helixtest_sha = Some(HELIXTEST_SHA.to_string());

    if !target_connectable(&endpoint) {
        let discovery = Discovery {
            endpoint: endpoint.clone(),
            services: VERIFY_ORDER
                .iter()
                .map(|k| ServiceDiscovery::not_detected(*k))
                .collect(),
        };
        run.discovery = model_discovery(&discovery);
        for kind in profile.enabled_services {
            for result in profile_errors(*kind, &unreachable_message(*kind)) {
                run.push_executed(result);
            }
        }
        run.sort_deterministic();
        return Ok(VerifyOutcome { discovery, run });
    }

    let client = http_client()?;
    let discovery = discover(&endpoint, &client).await?;
    run.discovery = model_discovery(&discovery);

    let adapter = HelixTestAdapter::pinned().with_capabilities(profile.capabilities);

    for kind in profile.enabled_services {
        let rec = discovery
            .record(*kind)
            .expect("discovery always records VERIFY_ORDER services");
        match (rec.detection, rec.testability) {
            (Detection::Detected, Testability::Testable) => {
                let url = rec
                    .base_url()
                    .expect("DETECTED TESTABLE service has a base URL");
                match run_adapter(&adapter, *kind, url).await {
                    Ok(out) => {
                        run.helixtest_version = Some(out.pin.tag.to_string());
                        run.helixtest_sha = Some(out.pin.sha.to_string());
                        for r in out.results {
                            run.push_executed(r);
                        }
                    }
                    Err(e) => {
                        for result in
                            profile_errors(*kind, &format!("HelixTest adapter error: {e}"))
                        {
                            run.push_executed(result);
                        }
                    }
                }
            }
            (Detection::Detected, Testability::NotTestable) => {
                let reason = rec
                    .not_testable_reason
                    .clone()
                    .unwrap_or_else(|| skip_not_testable(*kind));
                apply_missing(&mut run, profile, *kind, &reason, true);
            }
            (Detection::NotDetected, _) => {
                apply_missing(&mut run, profile, *kind, &skip_not_detected(*kind), false);
            }
        }
    }

    run.sort_deterministic();
    Ok(VerifyOutcome { discovery, run })
}

async fn run_adapter(
    adapter: &HelixTestAdapter,
    kind: Ga4ghService,
    url: &str,
) -> Result<crate::adapter::AdapterOutcome> {
    match kind {
        Ga4ghService::Drs => adapter.run_drs(url).await,
        Ga4ghService::Wes => adapter.run_wes(url).await,
        other => anyhow::bail!("helix verify does not execute {}", other.as_str()),
    }
}

/// Expected-but-missing → fail. Otherwise skip. Skip is never pass.
fn apply_missing(
    run: &mut VerificationRun,
    profile: Profile,
    kind: Ga4ghService,
    skip_reason: &str,
    detected_not_testable: bool,
) {
    if profile.expects(kind) {
        let msg = if detected_not_testable {
            format!(
                "{} expected by profile {} but not TESTABLE",
                kind.as_str(),
                profile.id.as_str()
            )
        } else {
            format!(
                "{} expected by profile {} but not detected",
                kind.as_str(),
                profile.id.as_str()
            )
        };
        for result in profile_fails(kind, &msg) {
            run.push_executed(result);
        }
    } else {
        for result in profile_skips(kind, skip_reason) {
            run.push_skipped(result);
        }
    }
}

fn skip_not_detected(kind: Ga4ghService) -> String {
    format!(
        "{} not detected; {} checks not executed (not a pass)",
        kind.as_str(),
        kind.as_str()
    )
}

fn skip_not_testable(kind: Ga4ghService) -> String {
    format!(
        "{} detected but not TESTABLE; checks not executed (DETECTED is not a pass)",
        kind.as_str()
    )
}

fn unreachable_message(kind: Ga4ghService) -> String {
    format!("target unreachable; {} checks not executed", kind.as_str())
}

fn model_discovery(d: &Discovery) -> Vec<DiscoveredService> {
    d.services.iter().map(to_model_service).collect()
}

fn to_model_service(rec: &ServiceDiscovery) -> DiscoveredService {
    let name = rec.kind.json_name();
    match rec.detection {
        Detection::NotDetected => DiscoveredService::missing(name),
        Detection::Detected => {
            if rec.testability == Testability::Testable {
                DiscoveredService::found(name, rec.base_url.clone().unwrap_or_default())
            } else {
                DiscoveredService::detected_not_testable(
                    name,
                    rec.base_url.clone().unwrap_or_default(),
                    rec.not_testable_reason
                        .clone()
                        .unwrap_or_else(|| skip_not_testable(rec.kind)),
                )
            }
        }
    }
}

fn helix_check(spec: &CheckSpec) -> VerificationCheck {
    VerificationCheck::from_spec(spec).with_profile("generic")
}

fn with_original_name(mut result: VerificationResult, spec: &CheckSpec) -> VerificationResult {
    if let Some(ht) = spec.helixtest_names.first() {
        result = result.with_helixtest_name(*ht);
    }
    result
}

fn verify_specs(kind: Ga4ghService) -> Vec<&'static CheckSpec> {
    match kind {
        Ga4ghService::Drs => drs_verify_specs().collect(),
        Ga4ghService::Wes => wes_verify_specs().collect(),
        _ => Vec::new(),
    }
}

fn profile_skips(kind: Ga4ghService, reason: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::skip(helix_check(s), reason), s))
        .collect()
}

fn profile_fails(kind: Ga4ghService, message: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::fail(helix_check(s), message), s))
        .collect()
}

fn profile_errors(kind: Ga4ghService, message: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::error(helix_check(s), message), s))
        .collect()
}

/// TCP connect to the origin host:port. Distinct from HTTP 404 (missing API).
pub fn target_connectable(endpoint: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(endpoint) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    let Ok(mut addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    let timeout = Duration::from_millis(400);
    addrs.any(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::VerificationStatus;

    #[test]
    fn skipped_drs_rows_are_never_pass() {
        for r in profile_skips(Ga4ghService::Drs, &skip_not_detected(Ga4ghService::Drs)) {
            assert_eq!(r.status, VerificationStatus::Skip);
            assert!(!r.is_pass());
            assert!(r.id.starts_with("drs.object."));
            assert!(r.code.starts_with("HLX-DRS-"));
            assert_eq!(r.service, "drs");
        }
        assert_eq!(profile_skips(Ga4ghService::Drs, "x").len(), 5);
    }

    #[test]
    fn skipped_wes_rows_are_never_pass() {
        for r in profile_skips(Ga4ghService::Wes, &skip_not_detected(Ga4ghService::Wes)) {
            assert_eq!(r.status, VerificationStatus::Skip);
            assert!(!r.is_pass());
            assert!(r.id.starts_with("wes."));
            assert!(r.code.starts_with("HLX-WES-"));
            assert_eq!(r.service, "wes");
        }
        assert_eq!(profile_skips(Ga4ghService::Wes, "x").len(), 8);
        assert_eq!(profile_fails(Ga4ghService::Wes, "expected").len(), 8);
        for r in profile_fails(Ga4ghService::Wes, "expected") {
            assert_eq!(r.status, VerificationStatus::Fail);
            assert!(!r.is_pass());
        }
    }
}
