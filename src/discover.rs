// SPDX-License-Identifier: Apache-2.0
//! Probe a gateway-style origin for public GA4GH HTTP APIs.
//!
//! Discovery is not conformance. DETECTED is not a pass. TESTABLE means Helix
//! will execute checks for that service in `helix verify`, not that they passed.
//! See docs/DISCOVERY.md.

use anyhow::{bail, Result};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

/// Stage 1 verify order. Not Beacon / africa / infra.
pub const VERIFY_ORDER: [Ga4ghService; 5] = [
    Ga4ghService::Drs,
    Ga4ghService::Wes,
    Ga4ghService::Tes,
    Ga4ghService::Trs,
    Ga4ghService::Htsget,
];

/// Services whose checks `helix verify` actually runs today.
pub const VERIFY_EXECUTABLE: [Ga4ghService; 2] = [Ga4ghService::Drs, Ga4ghService::Wes];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Ga4ghService {
    Drs,
    Wes,
    Tes,
    Trs,
    Htsget,
}

impl Ga4ghService {
    pub fn as_str(self) -> &'static str {
        match self {
            Ga4ghService::Drs => "DRS",
            Ga4ghService::Wes => "WES",
            Ga4ghService::Tes => "TES",
            Ga4ghService::Trs => "TRS",
            Ga4ghService::Htsget => "htsget",
        }
    }

    /// Open service string for [`crate::model::DiscoveredService`].
    pub fn json_name(self) -> &'static str {
        match self {
            Ga4ghService::Drs => "drs",
            Ga4ghService::Wes => "wes",
            Ga4ghService::Tes => "tes",
            Ga4ghService::Trs => "trs",
            Ga4ghService::Htsget => "htsget",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Detection {
    NotDetected,
    Detected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Testability {
    Testable,
    NotTestable,
}

/// How the winning probe identified the service. Not a capability claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    Ga4ghDrsObject,
    Ga4ghDrsServiceInfo,
    SplitDrsObject,
    Ga4ghWesServiceInfo,
    SplitWesServiceInfo,
    Ga4ghTesServiceInfo,
    Ga4ghTesTasks,
    Ga4ghTrsServiceInfo,
    Ga4ghTrsTools,
    Ga4ghHtsgetReadsServiceInfo,
}

impl DiscoveryMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ga4ghDrsObject => "ga4gh_drs_object",
            Self::Ga4ghDrsServiceInfo => "ga4gh_drs_service_info",
            Self::SplitDrsObject => "split_drs_object",
            Self::Ga4ghWesServiceInfo => "ga4gh_wes_service_info",
            Self::SplitWesServiceInfo => "split_wes_service_info",
            Self::Ga4ghTesServiceInfo => "ga4gh_tes_service_info",
            Self::Ga4ghTesTasks => "ga4gh_tes_tasks",
            Self::Ga4ghTrsServiceInfo => "ga4gh_trs_service_info",
            Self::Ga4ghTrsTools => "ga4gh_trs_tools",
            Self::Ga4ghHtsgetReadsServiceInfo => "ga4gh_htsget_reads_service_info",
        }
    }

    fn is_service_info(self) -> bool {
        matches!(
            self,
            Self::Ga4ghDrsServiceInfo
                | Self::Ga4ghWesServiceInfo
                | Self::SplitWesServiceInfo
                | Self::Ga4ghTesServiceInfo
                | Self::Ga4ghTrsServiceInfo
                | Self::Ga4ghHtsgetReadsServiceInfo
        )
    }
}

/// Fields copied from a 2xx service-info JSON body only. Never inferred from the URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ServiceInfoSnapshot {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceDiscovery {
    pub kind: Ga4ghService,
    pub detection: Detection,
    pub testability: Testability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_testable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_method: Option<DiscoveryMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    pub service_info: ServiceInfoSnapshot,
}

impl ServiceDiscovery {
    pub fn not_detected(kind: Ga4ghService) -> Self {
        Self {
            kind,
            detection: Detection::NotDetected,
            testability: Testability::NotTestable,
            not_testable_reason: Some("not detected; nothing to test".into()),
            base_url: None,
            discovery_method: None,
            http_status: None,
            service_info: ServiceInfoSnapshot::default(),
        }
    }

    pub fn is_detected(&self) -> bool {
        self.detection == Detection::Detected
    }

    /// HelixTest-style base URL when DETECTED.
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Discovery {
    pub endpoint: String,
    pub services: Vec<ServiceDiscovery>,
}

impl Discovery {
    /// DETECTED record for `kind`, if any. Not a pass.
    pub fn get(&self, kind: Ga4ghService) -> Option<&ServiceDiscovery> {
        self.services
            .iter()
            .find(|s| s.kind == kind && s.is_detected())
    }

    pub fn record(&self, kind: Ga4ghService) -> Option<&ServiceDiscovery> {
        self.services.iter().find(|s| s.kind == kind)
    }
}

/// Strip trailing slash; require http(s).
pub fn normalize_endpoint(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("endpoint URL is empty");
    }
    let url =
        reqwest::Url::parse(trimmed).map_err(|e| anyhow::anyhow!("invalid endpoint URL: {e}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => bail!("endpoint URL must be http or https, got {other}"),
    }
    if url.host_str().is_none() {
        bail!("endpoint URL must include a host");
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("endpoint URL must not include userinfo (credentials in the URL are rejected)");
    }
    let mut s = url.to_string();
    if s.ends_with('/') {
        s.pop();
    }
    Ok(s)
}

fn status_means_api(code: u16) -> bool {
    (200..300).contains(&code) || code == 401 || code == 403
}

struct ProbeHit {
    method: DiscoveryMethod,
    base_url: String,
    probe_url: String,
    status: u16,
    body: Vec<u8>,
}

async fn get_probe(client: &Client, url: &str) -> Option<(u16, Vec<u8>)> {
    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            match crate::http_safety::read_body_capped(resp, crate::http_safety::MAX_RESPONSE_BYTES)
                .await
            {
                Ok(body) => Some((status, body)),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

struct ProbeSpec<'a> {
    method: DiscoveryMethod,
    probe_url: &'a str,
    base_url: &'a str,
}

async fn first_present(client: &Client, candidates: &[ProbeSpec<'_>]) -> Option<ProbeHit> {
    for c in candidates {
        if let Some((status, body)) = get_probe(client, c.probe_url).await {
            if status_means_api(status) {
                return Some(ProbeHit {
                    method: c.method,
                    base_url: c.base_url.to_string(),
                    probe_url: c.probe_url.to_string(),
                    status,
                    body,
                });
            }
        }
    }
    None
}

fn executable_in_verify(kind: Ga4ghService) -> bool {
    VERIFY_EXECUTABLE.contains(&kind)
}

fn testability_for(kind: Ga4ghService) -> (Testability, Option<String>) {
    if executable_in_verify(kind) {
        (Testability::Testable, None)
    } else {
        (
            Testability::NotTestable,
            Some(format!(
                "Helix Stage 1 does not execute {} checks; DETECTED is not a pass",
                kind.as_str()
            )),
        )
    }
}

fn snapshot_from_http(status: u16, body: &[u8]) -> ServiceInfoSnapshot {
    let mut snap = ServiceInfoSnapshot {
        available: (200..300).contains(&status),
        http_status: Some(status),
        ..ServiceInfoSnapshot::default()
    };
    if !snap.available {
        return snap;
    }
    let Ok(v) = serde_json::from_slice::<Value>(body) else {
        return snap;
    };
    let Some(obj) = v.as_object() else {
        return snap;
    };
    snap.id = obj.get("id").and_then(|x| x.as_str()).map(str::to_string);
    snap.name = obj.get("name").and_then(|x| x.as_str()).map(str::to_string);
    snap.version = obj
        .get("version")
        .and_then(|x| x.as_str())
        .map(str::to_string);
    if let Some(t) = obj.get("type").and_then(|x| x.as_object()) {
        snap.type_artifact = t
            .get("artifact")
            .and_then(|x| x.as_str())
            .map(str::to_string);
        snap.type_version = t
            .get("version")
            .and_then(|x| x.as_str())
            .map(str::to_string);
    }
    snap
}

fn service_info_url(kind: Ga4ghService, method: DiscoveryMethod, base_url: &str) -> Option<String> {
    match (kind, method) {
        (Ga4ghService::Drs, DiscoveryMethod::SplitDrsObject) => {
            // Do not use origin /service-info (often WES). Only a DRS-prefixed path.
            Some(format!("{base_url}/ga4gh/drs/v1/service-info"))
        }
        (Ga4ghService::Drs, _) => Some(format!("{base_url}/service-info")),
        (Ga4ghService::Wes, _) => Some(format!("{base_url}/service-info")),
        (Ga4ghService::Tes, _) => Some(format!("{base_url}/service-info")),
        (Ga4ghService::Trs, _) => Some(format!("{base_url}/service-info")),
        (Ga4ghService::Htsget, _) => Some(format!("{base_url}/reads/service-info")),
    }
}

async fn enrich_service_info(
    client: &Client,
    kind: Ga4ghService,
    hit: &ProbeHit,
) -> ServiceInfoSnapshot {
    let Some(info_url) = service_info_url(kind, hit.method, &hit.base_url) else {
        return ServiceInfoSnapshot::default();
    };
    if hit.method.is_service_info() && hit.probe_url == info_url {
        return snapshot_from_http(hit.status, &hit.body);
    }
    match get_probe(client, &info_url).await {
        Some((status, body)) => snapshot_from_http(status, &body),
        None => ServiceInfoSnapshot::default(),
    }
}

async fn record_from_hit(client: &Client, kind: Ga4ghService, hit: ProbeHit) -> ServiceDiscovery {
    let (testability, reason) = testability_for(kind);
    let service_info = enrich_service_info(client, kind, &hit).await;
    ServiceDiscovery {
        kind,
        detection: Detection::Detected,
        testability,
        not_testable_reason: reason,
        base_url: Some(hit.base_url),
        discovery_method: Some(hit.method),
        http_status: Some(hit.status),
        service_info,
    }
}

/// Discover which Stage 1 GA4GH APIs answer under `endpoint`.
/// Lightweight probes only — does not run HelixTest checks.
pub async fn discover(endpoint: &str, client: &Client) -> Result<Discovery> {
    let endpoint = normalize_endpoint(endpoint)?;
    let mut services = Vec::with_capacity(VERIFY_ORDER.len());

    for kind in VERIFY_ORDER {
        let hit = match kind {
            Ga4ghService::Drs => discover_drs(client, &endpoint).await,
            Ga4ghService::Wes => discover_wes(client, &endpoint).await,
            Ga4ghService::Tes => discover_tes(client, &endpoint).await,
            Ga4ghService::Trs => discover_trs(client, &endpoint).await,
            Ga4ghService::Htsget => discover_htsget(client, &endpoint).await,
        };
        let rec = match hit {
            Some(hit) => record_from_hit(client, kind, hit).await,
            None => ServiceDiscovery::not_detected(kind),
        };
        services.push(rec);
    }

    Ok(Discovery { endpoint, services })
}

async fn discover_drs(client: &Client, endpoint: &str) -> Option<ProbeHit> {
    let gw = format!("{endpoint}/ga4gh/drs/v1");
    let gw_obj = format!("{gw}/objects/test-object-1");
    let gw_info = format!("{gw}/service-info");
    let split_obj = format!("{endpoint}/objects/test-object-1");
    first_present(
        client,
        &[
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghDrsObject,
                probe_url: &gw_obj,
                base_url: gw.as_str(),
            },
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghDrsServiceInfo,
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeSpec {
                method: DiscoveryMethod::SplitDrsObject,
                probe_url: &split_obj,
                base_url: endpoint,
            },
        ],
    )
    .await
}

async fn discover_wes(client: &Client, endpoint: &str) -> Option<ProbeHit> {
    let gw = format!("{endpoint}/ga4gh/wes/v1");
    let gw_info = format!("{gw}/service-info");
    let split_info = format!("{endpoint}/service-info");
    first_present(
        client,
        &[
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghWesServiceInfo,
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeSpec {
                method: DiscoveryMethod::SplitWesServiceInfo,
                probe_url: &split_info,
                base_url: endpoint,
            },
        ],
    )
    .await
}

async fn discover_tes(client: &Client, endpoint: &str) -> Option<ProbeHit> {
    let gw = format!("{endpoint}/ga4gh/tes/v1");
    let gw_info = format!("{gw}/service-info");
    let gw_tasks = format!("{gw}/tasks");
    first_present(
        client,
        &[
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghTesServiceInfo,
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghTesTasks,
                probe_url: &gw_tasks,
                base_url: gw.as_str(),
            },
        ],
    )
    .await
}

async fn discover_trs(client: &Client, endpoint: &str) -> Option<ProbeHit> {
    let gw = format!("{endpoint}/ga4gh/trs/v2");
    let gw_info = format!("{gw}/service-info");
    let gw_tools = format!("{gw}/tools");
    first_present(
        client,
        &[
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghTrsServiceInfo,
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeSpec {
                method: DiscoveryMethod::Ga4ghTrsTools,
                probe_url: &gw_tools,
                base_url: gw.as_str(),
            },
        ],
    )
    .await
}

async fn discover_htsget(client: &Client, endpoint: &str) -> Option<ProbeHit> {
    let gw = format!("{endpoint}/ga4gh/htsget/v1");
    let reads = format!("{gw}/reads/service-info");
    first_present(
        client,
        &[ProbeSpec {
            method: DiscoveryMethod::Ga4ghHtsgetReadsServiceInfo,
            probe_url: &reads,
            base_url: gw.as_str(),
        }],
    )
    .await
}

pub fn http_client() -> Result<Client> {
    crate::http_safety::http_client()
}

/// Human discovery table. Never says "found" or "verified". No green PASS.
pub fn format_discovery_report(d: &Discovery) -> String {
    let mut out = String::new();
    out.push_str("Helix verify — GA4GH discovery (not conformance, not certification)\n");
    out.push_str(&format!("endpoint: {}\n", d.endpoint));
    out.push_str("Helix tests behavior against the GA4GH spec, independent of implementation.\n");
    out.push_str("Ferrum is used as a reference target, not a dependency.\n");
    out.push_str(
        "DETECTED is not a pass. TESTABLE means Helix will run checks, not that they passed.\n",
    );
    out.push('\n');
    for rec in &d.services {
        out.push_str(&format_discovery_row(rec));
        out.push('\n');
    }
    if d.services.iter().all(|s| !s.is_detected()) {
        out.push_str("\nNo Stage 1 APIs (DRS, WES, TES, TRS, htsget) answered.\n");
    }
    crate::redact::redact_text(&out)
}

pub fn format_discovery_row(rec: &ServiceDiscovery) -> String {
    let svc = rec.kind.as_str();
    match rec.detection {
        Detection::NotDetected => format!("{svc:<8} NOT_DETECTED"),
        Detection::Detected => {
            let test_col = match rec.testability {
                Testability::Testable => "TESTABLE".to_string(),
                Testability::NotTestable => "NOT_TESTABLE".to_string(),
            };
            let mut line = format!("{svc:<8} DETECTED     {test_col}");
            if let Some(reason) = &rec.not_testable_reason {
                if rec.testability == Testability::NotTestable {
                    line.push_str("  ");
                    line.push_str(reason);
                }
            }
            let mut detail = Vec::new();
            if let Some(m) = rec.discovery_method {
                detail.push(format!("method={}", m.as_str()));
            }
            if let Some(st) = rec.http_status {
                detail.push(format!("http={st}"));
            }
            if let Some(base) = &rec.base_url {
                detail.push(format!("base={base}"));
            }
            if rec.service_info.available {
                detail.push("service-info=yes".into());
            } else if rec.service_info.http_status.is_some() {
                detail.push(format!(
                    "service-info=no({})",
                    rec.service_info.http_status.unwrap()
                ));
            } else {
                detail.push("service-info=not_read".into());
            }
            if let Some(v) = &rec.service_info.version {
                detail.push(format!("version={v}"));
            } else if let Some(v) = &rec.service_info.type_version {
                detail.push(format!("type.version={v}"));
            }
            if !detail.is_empty() {
                line.push('\n');
                line.push_str(&format!("         {}", detail.join("  ")));
            }
            line
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client() -> Client {
        http_client().unwrap()
    }

    #[test]
    fn normalize_strips_slash_and_rejects_non_http() {
        assert_eq!(
            normalize_endpoint("http://127.0.0.1:8080/").unwrap(),
            "http://127.0.0.1:8080"
        );
        assert_eq!(
            normalize_endpoint("https://example.org/ga4gh").unwrap(),
            "https://example.org/ga4gh"
        );
        assert!(normalize_endpoint("ftp://x").is_err());
        assert!(normalize_endpoint("not-a-url").is_err());
        assert!(normalize_endpoint("").is_err());
        let with_userinfo = normalize_endpoint("http://alice:s3cret@example.org/ga4gh")
            .unwrap_err()
            .to_string();
        assert!(with_userinfo.contains("userinfo"), "{with_userinfo}");
        assert!(
            !with_userinfo.contains("s3cret"),
            "normalize error must not echo the password: {with_userinfo}"
        );
        assert!(
            !with_userinfo.contains("alice"),
            "normalize error must not echo URL userinfo: {with_userinfo}"
        );
        assert!(normalize_endpoint("http://:s3cret@example.org/").is_err());
        assert!(normalize_endpoint("http://alice@example.org/").is_err());
    }

    #[test]
    fn verify_order_is_drs_first() {
        assert_eq!(VERIFY_ORDER[0], Ga4ghService::Drs);
        assert_eq!(VERIFY_ORDER[1], Ga4ghService::Wes);
        assert_eq!(VERIFY_ORDER[2], Ga4ghService::Tes);
        assert_eq!(VERIFY_ORDER[3], Ga4ghService::Trs);
        assert_eq!(VERIFY_ORDER[4], Ga4ghService::Htsget);
    }

    #[test]
    fn discovery_row_never_says_found_or_verified() {
        let rec = ServiceDiscovery {
            kind: Ga4ghService::Tes,
            detection: Detection::Detected,
            testability: Testability::NotTestable,
            not_testable_reason: Some(
                "Helix Stage 1 does not execute TES checks; DETECTED is not a pass".into(),
            ),
            base_url: Some("http://127.0.0.1:9/ga4gh/tes/v1".into()),
            discovery_method: Some(DiscoveryMethod::Ga4ghTesTasks),
            http_status: Some(200),
            service_info: ServiceInfoSnapshot {
                available: true,
                http_status: Some(200),
                ..ServiceInfoSnapshot::default()
            },
        };
        let row = format_discovery_row(&rec);
        assert!(row.contains("DETECTED"));
        assert!(row.contains("NOT_TESTABLE"));
        assert!(row.contains("TES"));
        assert!(!row.to_lowercase().contains("found"));
        assert!(!row.to_lowercase().contains("verified"));
        assert!(!row.contains("PASS"));
        let missing = format_discovery_row(&ServiceDiscovery::not_detected(Ga4ghService::Trs));
        assert_eq!(missing, "TRS      NOT_DETECTED");
        assert!(!missing.contains("found"));
    }

    #[tokio::test]
    async fn discovers_split_drs_like_b1_mock() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":"test-object-1"})),
            )
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let drs = d.get(Ga4ghService::Drs).expect("DRS");
        assert_eq!(drs.detection, Detection::Detected);
        assert_eq!(drs.testability, Testability::Testable);
        assert_eq!(drs.discovery_method, Some(DiscoveryMethod::SplitDrsObject));
        assert_eq!(drs.http_status, Some(200));
        assert_eq!(drs.base_url.as_ref(), Some(&server.uri()));
        assert!(!drs.service_info.available);
        let wes = d.record(Ga4ghService::Wes).unwrap();
        assert_eq!(wes.detection, Detection::NotDetected);
        assert!(d.get(Ga4ghService::Wes).is_none());
    }

    #[tokio::test]
    async fn discovers_gateway_drs_before_split() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let drs = d.get(Ga4ghService::Drs).unwrap();
        let expected = format!("{}/ga4gh/drs/v1", server.uri());
        assert_eq!(drs.base_url.as_deref(), Some(expected.as_str()));
        assert_eq!(drs.discovery_method, Some(DiscoveryMethod::Ga4ghDrsObject));
    }

    #[tokio::test]
    async fn gateway_drs_service_info_counts_when_object_missing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/service-info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "org.ga4gh.drs",
                "name": "Example DRS",
                "version": "1.2.0",
                "type": { "group": "org.ga4gh", "artifact": "drs", "version": "1.2.0" }
            })))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let drs = d.get(Ga4ghService::Drs).unwrap();
        let expected = format!("{}/ga4gh/drs/v1", server.uri());
        assert_eq!(drs.base_url.as_deref(), Some(expected.as_str()));
        assert_eq!(
            drs.discovery_method,
            Some(DiscoveryMethod::Ga4ghDrsServiceInfo)
        );
        assert_eq!(drs.testability, Testability::Testable);
        assert!(drs.service_info.available);
        assert_eq!(drs.service_info.version.as_deref(), Some("1.2.0"));
        assert_eq!(drs.service_info.type_artifact.as_deref(), Some("drs"));
        assert_eq!(drs.service_info.id.as_deref(), Some("org.ga4gh.drs"));
    }

    #[tokio::test]
    async fn unmatched_404_is_not_drs() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        assert_eq!(
            d.record(Ga4ghService::Drs).unwrap().detection,
            Detection::NotDetected
        );
        assert!(d.get(Ga4ghService::Drs).is_none());
    }

    #[tokio::test]
    async fn discovers_wes_detected_testable() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/service-info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "wes",
                "type": { "artifact": "wes", "version": "1.0.0" }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/htsget/v1/reads/service-info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let wes = d.get(Ga4ghService::Wes).unwrap();
        assert_eq!(wes.detection, Detection::Detected);
        assert_eq!(wes.testability, Testability::Testable);
        assert!(wes.not_testable_reason.is_none());
        assert_eq!(wes.service_info.type_artifact.as_deref(), Some("wes"));
        assert_eq!(wes.service_info.type_version.as_deref(), Some("1.0.0"));
        assert!(wes.service_info.version.is_none());
        let hts = d.get(Ga4ghService::Htsget).unwrap();
        assert_eq!(hts.detection, Detection::Detected);
        assert_eq!(hts.testability, Testability::NotTestable);
        assert_eq!(
            d.record(Ga4ghService::Drs).unwrap().detection,
            Detection::NotDetected
        );
        let text = format_discovery_report(&d);
        assert!(text.contains("WES      DETECTED     TESTABLE"));
        assert!(text.contains("htsget   DETECTED     NOT_TESTABLE"));
        assert!(text.contains("DRS      NOT_DETECTED"));
        assert!(!text.contains(" found"));
        assert!(text.contains("DETECTED is not a pass"));
    }

    #[tokio::test]
    async fn does_not_invent_version_from_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/service-info"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let wes = d.get(Ga4ghService::Wes).unwrap();
        assert!(wes.service_info.available);
        assert!(wes.service_info.version.is_none());
        assert!(wes.service_info.type_version.is_none());
        assert!(wes.service_info.type_artifact.is_none());
    }

    #[tokio::test]
    async fn auth_challenge_is_detected_not_verified() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/service-info"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let wes = d.get(Ga4ghService::Wes).unwrap();
        assert_eq!(wes.detection, Detection::Detected);
        assert_eq!(wes.http_status, Some(401));
        assert!(!wes.service_info.available);
        assert_eq!(wes.service_info.http_status, Some(401));
        assert_eq!(wes.testability, Testability::Testable);
    }

    #[tokio::test]
    async fn empty_origin_is_all_not_detected() {
        let server = MockServer::start().await;
        let d = discover(&server.uri(), &client()).await.unwrap();
        assert_eq!(d.services.len(), 5);
        assert!(d
            .services
            .iter()
            .all(|s| s.detection == Detection::NotDetected));
        assert!(d.get(Ga4ghService::Drs).is_none());
        let text = format_discovery_report(&d);
        assert!(text.contains("NOT_DETECTED"));
        assert!(!text.contains(" DETECTED"));
        assert!(text.contains("DETECTED is not a pass"));
    }

    #[tokio::test]
    async fn tes_tasks_probe_is_detected_not_testable() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/tes/v1/tasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        let tes = d.get(Ga4ghService::Tes).unwrap();
        assert_eq!(tes.detection, Detection::Detected);
        assert_eq!(tes.testability, Testability::NotTestable);
        assert_eq!(tes.discovery_method, Some(DiscoveryMethod::Ga4ghTesTasks));
        assert!(!tes.service_info.available);
    }

    #[tokio::test]
    async fn redirect_is_not_followed_and_not_detected() {
        let internal = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "leaked"})),
            )
            .expect(0)
            .mount(&internal)
            .await;

        let public = MockServer::start().await;
        let location = format!("{}/objects/test-object-1", internal.uri());
        Mock::given(method("GET"))
            .and(path("/ga4gh/drs/v1/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(302).insert_header("Location", location.as_str()))
            .mount(&public)
            .await;

        let d = discover(&public.uri(), &client()).await.unwrap();
        assert_eq!(
            d.record(Ga4ghService::Drs).unwrap().detection,
            Detection::NotDetected
        );
        assert!(d.get(Ga4ghService::Drs).is_none());
    }

    #[tokio::test]
    async fn oversized_probe_body_is_not_detected() {
        let server = MockServer::start().await;
        let big = vec![b'x'; crate::http_safety::MAX_RESPONSE_BYTES + 1];
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(big))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        assert_eq!(
            d.record(Ga4ghService::Drs).unwrap().detection,
            Detection::NotDetected
        );
    }
}
