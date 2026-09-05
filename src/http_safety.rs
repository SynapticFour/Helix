// SPDX-License-Identifier: Apache-2.0
//! Limits for Helix-owned HTTP and local file reads.
//!
//! Not a WAF. HelixTest’s client is separate (redirects / gzip / unbounded body).
//! See docs/THREAT_MODEL.md.

use anyhow::{bail, Context, Result};
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

/// Same values as recorded on `helix bench` metadata.
pub const HTTP_REQUEST_TIMEOUT_SECS: u64 = 5;
pub const HTTP_CONNECT_TIMEOUT_SECS: u64 = 3;

/// Helix-owned GET bodies (discovery, security HTTP, Crypt4GH probe, bench).
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

pub const MAX_COMPARE_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_SECRET_FILE_BYTES: u64 = 64 * 1024;
pub const MAX_CRYPT4GH_FILE_BYTES: u64 = 1024 * 1024;

/// Helix-owned client: rustls (default webpki roots), no redirect follow, no gzip/brotli
/// (reqwest `default-features = false`). Invalid certificates fail the request (not DETECTED).
/// Helix does not pin certificates and does not disable TLS verification.
pub fn http_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(HTTP_CONNECT_TIMEOUT_SECS))
        .redirect(Policy::none())
        .build()
        .map_err(Into::into)
}

/// Read at most `max` bytes. Does not keep the rest. Content-Length over `max` is refused without buffering.
pub async fn read_body_capped(mut resp: Response, max: usize) -> Result<Vec<u8>> {
    if let Some(len) = resp.content_length() {
        if len > max as u64 {
            bail!("response exceeds {max} bytes");
        }
    }
    let mut out = Vec::new();
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                if out.len().saturating_add(chunk.len()) > max {
                    bail!("response exceeds {max} bytes");
                }
                out.extend_from_slice(&chunk);
            }
            Ok(None) => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(out)
}

pub fn read_file_capped(path: &Path, max: u64) -> Result<Vec<u8>> {
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buf = Vec::new();
    let n = f.take(max + 1).read_to_end(&mut buf)?;
    if n as u64 > max {
        bail!(
            "{} exceeds {max} bytes (refusing to load; contents not printed)",
            path.display()
        );
    }
    Ok(buf)
}

pub fn read_to_string_capped(path: &Path, max: u64) -> Result<String> {
    let bytes = read_file_capped(path, max)?;
    String::from_utf8(bytes)
        .map_err(|_| anyhow::anyhow!("{} is not UTF-8 (contents not printed)", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn body_capped_refuses_oversized_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 1000]))
            .mount(&server)
            .await;
        let client = http_client().unwrap();
        let resp = client.get(server.uri()).send().await.unwrap();
        let err = read_body_capped(resp, 64).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("64"), "{msg}");
        assert!(!msg.contains(&"a".repeat(20)), "{msg}");
    }

    #[tokio::test]
    async fn redirects_are_not_followed() {
        let internal = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("internal-secret-body"))
            .expect(0)
            .mount(&internal)
            .await;

        let public = MockServer::start().await;
        let location = internal.uri();
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(302).insert_header("Location", location.as_str()))
            .mount(&public)
            .await;

        let client = http_client().unwrap();
        let resp = client.get(public.uri()).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 302);
        let body = read_body_capped(resp, MAX_RESPONSE_BYTES)
            .await
            .unwrap_or_default();
        let text = String::from_utf8_lossy(&body);
        assert!(!text.contains("internal-secret-body"), "{text}");
    }

    #[test]
    fn file_capped_refuses_oversized_without_dumping() {
        let p = std::env::temp_dir().join(format!(
            "helix-file-cap-{}-{}",
            std::process::id(),
            "oversize"
        ));
        let mut f = File::create(&p).unwrap();
        f.write_all(&[b'Z'; 200]).unwrap();
        drop(f);
        let err = read_file_capped(&p, 64).unwrap_err();
        let msg = err.to_string();
        std::fs::remove_file(&p).ok();
        assert!(msg.contains("64"), "{msg}");
        assert!(!msg.contains(&"Z".repeat(20)), "{msg}");
    }
}
