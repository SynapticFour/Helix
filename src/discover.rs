// SPDX-License-Identifier: Apache-2.0
//! Probe a gateway-style origin for public GA4GH HTTP APIs.
//!
//! Order matches Helix Stage 1: DRS → WES → TES → TRS → htsget.
//! Ferrum is a reference target, not a dependency. Paths are the published
//! GA4GH prefixes (`/ga4gh/drs/v1`, …) plus split-port `/objects/{id}` for DRS.

use anyhow::{bail, Result};
use reqwest::Client;
use serde::Serialize;

/// Stage 1 verify order. Not Beacon / africa / infra.
pub const VERIFY_ORDER: [Ga4ghService; 5] = [
    Ga4ghService::Drs,
    Ga4ghService::Wes,
    Ga4ghService::Tes,
    Ga4ghService::Trs,
    Ga4ghService::Htsget,
];

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredService {
    pub kind: Ga4ghService,
    /// Base URL HelixTest uses (no trailing slash), e.g. `http://host/ga4gh/drs/v1`.
    pub base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Discovery {
    pub endpoint: String,
    pub found: Vec<DiscoveredService>,
    pub missing: Vec<Ga4ghService>,
}

impl Discovery {
    pub fn get(&self, kind: Ga4ghService) -> Option<&DiscoveredService> {
        self.found.iter().find(|s| s.kind == kind)
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
    let mut s = url.to_string();
    if s.ends_with('/') {
        s.pop();
    }
    Ok(s)
}

fn status_means_api(code: u16) -> bool {
    (200..300).contains(&code) || code == 401 || code == 403
}

async fn get_status(client: &Client, url: &str) -> Option<u16> {
    match client.get(url).send().await {
        Ok(resp) => Some(resp.status().as_u16()),
        Err(_) => None,
    }
}

struct ProbeCandidate<'a> {
    probe_url: &'a str,
    base_url: &'a str,
}

async fn first_present(client: &Client, candidates: &[ProbeCandidate<'_>]) -> Option<String> {
    for c in candidates {
        if let Some(code) = get_status(client, c.probe_url).await {
            if status_means_api(code) {
                return Some(c.base_url.to_string());
            }
        }
    }
    None
}

/// Discover which Stage 1 GA4GH APIs answer under `endpoint`.
pub async fn discover(endpoint: &str, client: &Client) -> Result<Discovery> {
    let endpoint = normalize_endpoint(endpoint)?;
    let mut found = Vec::new();
    let mut missing = Vec::new();

    for kind in VERIFY_ORDER {
        let base = match kind {
            Ga4ghService::Drs => discover_drs(client, &endpoint).await,
            Ga4ghService::Wes => discover_wes(client, &endpoint).await,
            Ga4ghService::Tes => discover_tes(client, &endpoint).await,
            Ga4ghService::Trs => discover_trs(client, &endpoint).await,
            Ga4ghService::Htsget => discover_htsget(client, &endpoint).await,
        };
        match base {
            Some(base_url) => found.push(DiscoveredService { kind, base_url }),
            None => missing.push(kind),
        }
    }

    Ok(Discovery {
        endpoint,
        found,
        missing,
    })
}

async fn discover_drs(client: &Client, endpoint: &str) -> Option<String> {
    let gw = format!("{endpoint}/ga4gh/drs/v1");
    let gw_obj = format!("{gw}/objects/test-object-1");
    let gw_info = format!("{gw}/service-info");
    let split_obj = format!("{endpoint}/objects/test-object-1");
    first_present(
        client,
        &[
            ProbeCandidate {
                probe_url: &gw_obj,
                base_url: gw.as_str(),
            },
            ProbeCandidate {
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeCandidate {
                probe_url: &split_obj,
                base_url: endpoint,
            },
        ],
    )
    .await
}

async fn discover_wes(client: &Client, endpoint: &str) -> Option<String> {
    let gw = format!("{endpoint}/ga4gh/wes/v1");
    let gw_info = format!("{gw}/service-info");
    let split_info = format!("{endpoint}/service-info");
    first_present(
        client,
        &[
            ProbeCandidate {
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeCandidate {
                probe_url: &split_info,
                base_url: endpoint,
            },
        ],
    )
    .await
}

async fn discover_tes(client: &Client, endpoint: &str) -> Option<String> {
    let gw = format!("{endpoint}/ga4gh/tes/v1");
    let gw_info = format!("{gw}/service-info");
    let gw_tasks = format!("{gw}/tasks");
    first_present(
        client,
        &[
            ProbeCandidate {
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeCandidate {
                probe_url: &gw_tasks,
                base_url: gw.as_str(),
            },
        ],
    )
    .await
}

async fn discover_trs(client: &Client, endpoint: &str) -> Option<String> {
    let gw = format!("{endpoint}/ga4gh/trs/v2");
    let gw_info = format!("{gw}/service-info");
    let gw_tools = format!("{gw}/tools");
    first_present(
        client,
        &[
            ProbeCandidate {
                probe_url: &gw_info,
                base_url: gw.as_str(),
            },
            ProbeCandidate {
                probe_url: &gw_tools,
                base_url: gw.as_str(),
            },
        ],
    )
    .await
}

async fn discover_htsget(client: &Client, endpoint: &str) -> Option<String> {
    let gw = format!("{endpoint}/ga4gh/htsget/v1");
    let reads = format!("{gw}/reads/service-info");
    first_present(
        client,
        &[ProbeCandidate {
            probe_url: &reads,
            base_url: gw.as_str(),
        }],
    )
    .await
}

pub fn http_client() -> Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .connect_timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(Into::into)
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
    }

    #[test]
    fn verify_order_is_drs_first() {
        assert_eq!(VERIFY_ORDER[0], Ga4ghService::Drs);
        assert_eq!(VERIFY_ORDER[1], Ga4ghService::Wes);
        assert_eq!(VERIFY_ORDER[2], Ga4ghService::Tes);
        assert_eq!(VERIFY_ORDER[3], Ga4ghService::Trs);
        assert_eq!(VERIFY_ORDER[4], Ga4ghService::Htsget);
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
        assert_eq!(drs.base_url, server.uri());
        assert!(d.get(Ga4ghService::Wes).is_none());
        assert!(d.missing.contains(&Ga4ghService::Wes));
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
        assert_eq!(
            d.get(Ga4ghService::Drs).unwrap().base_url,
            format!("{}/ga4gh/drs/v1", server.uri())
        );
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
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":"drs"})))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        assert_eq!(
            d.get(Ga4ghService::Drs).unwrap().base_url,
            format!("{}/ga4gh/drs/v1", server.uri())
        );
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
        assert!(d.get(Ga4ghService::Drs).is_none());
    }

    #[tokio::test]
    async fn discovers_wes_and_htsget_on_gateway() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/service-info"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"name":"wes"})),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/htsget/v1/reads/service-info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        assert_eq!(
            d.get(Ga4ghService::Wes).unwrap().base_url,
            format!("{}/ga4gh/wes/v1", server.uri())
        );
        assert_eq!(
            d.get(Ga4ghService::Htsget).unwrap().base_url,
            format!("{}/ga4gh/htsget/v1", server.uri())
        );
        assert!(d.get(Ga4ghService::Drs).is_none());
    }

    #[tokio::test]
    async fn auth_challenge_counts_as_present() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ga4gh/wes/v1/service-info"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let d = discover(&server.uri(), &client()).await.unwrap();
        assert!(d.get(Ga4ghService::Wes).is_some());
    }

    #[tokio::test]
    async fn empty_origin_finds_nothing() {
        let server = MockServer::start().await;
        let d = discover(&server.uri(), &client()).await.unwrap();
        assert!(d.found.is_empty());
        assert_eq!(d.missing.len(), 5);
    }
}
