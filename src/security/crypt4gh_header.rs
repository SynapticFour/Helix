// SPDX-License-Identifier: Apache-2.0
//! Crypt4GH **protocol layout** checks. After the five HTTP Security Behavior
//! Profile cases. Not encryption, not a key-management system, not “secure”.
//!
//! Helix does **not** implement X25519 or ChaCha20-Poly1305. HelixTest already
//! uses the GA4GH `crypt4gh` crate for env-gated DRS rewrap/decrypt (needs a
//! client **secret** key). Helix does not call that path.
//!
//! Private keys are never loaded or printed. `dummy-x25519.placeholder` is not read.

use anyhow::Result;
use common::report::{ComplianceLevel, ServiceKind, ServiceReport, TestCaseResult, TestCategory};
use std::path::Path;

use crate::discover::{discover, http_client, Ga4ghService};
use crate::identity;
use crate::security::http_cases::protected_object_url;

const MAGIC: &[u8; 8] = b"crypt4gh";

pub const CRYPT4GH_CASE_IDS: [&str; 3] = [
    "auth.helix.crypt4gh.header",
    "auth.helix.crypt4gh.invalid_rejected",
    "auth.helix.crypt4gh.http_envelope",
];

const WELL_FORMED: &[u8] = include_bytes!("../../test-fixtures/crypt4gh/well-formed.c4gh");
const WRONG_MAGIC: &[u8] = include_bytes!("../../test-fixtures/crypt4gh/wrong-magic.c4gh");
const TRUNCATED: &[u8] = include_bytes!("../../test-fixtures/crypt4gh/truncated.c4gh");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Crypt4ghHeaderError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u32),
    NoPackets,
    TruncatedPacket,
}

impl std::fmt::Display for Crypt4ghHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Crypt4ghHeaderError::TooShort => write!(f, "Crypt4GH header shorter than 16 bytes"),
            Crypt4ghHeaderError::BadMagic => {
                write!(f, "Crypt4GH magic is not 'crypt4gh' (not dumping bytes)")
            }
            Crypt4ghHeaderError::UnsupportedVersion(v) => {
                write!(f, "Crypt4GH version {v} is not 1")
            }
            Crypt4ghHeaderError::NoPackets => write!(f, "Crypt4GH header has zero packets"),
            Crypt4ghHeaderError::TruncatedPacket => {
                write!(f, "Crypt4GH header packet is truncated")
            }
        }
    }
}

/// Parse unencrypted header structure only. Packet bodies are not interpreted
/// (they may be encrypted). This is protocol framing, not cryptography.
pub fn validate_crypt4gh_header(bytes: &[u8]) -> Result<(), Crypt4ghHeaderError> {
    if bytes.len() < 16 {
        return Err(Crypt4ghHeaderError::TooShort);
    }
    if bytes[0..8] != MAGIC[..] {
        return Err(Crypt4ghHeaderError::BadMagic);
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if version != 1 {
        return Err(Crypt4ghHeaderError::UnsupportedVersion(version));
    }
    let npackets = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
    if npackets == 0 || npackets > 1024 {
        return Err(Crypt4ghHeaderError::NoPackets);
    }
    let mut off = 16usize;
    for _ in 0..npackets {
        if off + 4 > bytes.len() {
            return Err(Crypt4ghHeaderError::TruncatedPacket);
        }
        let plen = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        if plen < 8 || off + plen > bytes.len() {
            return Err(Crypt4ghHeaderError::TruncatedPacket);
        }
        off += plen;
    }
    Ok(())
}

fn looks_like_crypt4gh_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes[0..8] == MAGIC[..]
}

fn case_name(id: &str) -> &'static str {
    identity::spec(id).name
}

fn case_code(id: &str) -> &'static str {
    identity::spec(id).code
}

/// Well-formed envelope is recognized (`HLX-AUTH-050`). Safe fixture or `--crypt4gh-file`.
fn case_well_formed(path: Option<&Path>) -> Result<TestCaseResult> {
    let id = "auth.helix.crypt4gh.header";
    let bytes = match path {
        Some(path) => {
            crate::http_safety::read_file_capped(path, crate::http_safety::MAX_CRYPT4GH_FILE_BYTES)?
        }
        None => WELL_FORMED.to_vec(),
    };
    let res = validate_crypt4gh_header(&bytes).map_err(|e| {
        format!(
            "{code}: well-formed Crypt4GH layout expected — {e}",
            code = case_code(id)
        )
    });
    Ok(TestCaseResult::from_outcome(
        case_name(id),
        ComplianceLevel::Level5,
        TestCategory::Security,
        res,
    ))
}

/// Invalid envelopes are rejected (`HLX-AUTH-053`). Embedded negative fixtures.
/// Pass means Helix **rejected** them. Accepting garbage is a fail.
fn case_invalid_rejected() -> TestCaseResult {
    let id = "auth.helix.crypt4gh.invalid_rejected";
    let res = (|| {
        expect_err(WRONG_MAGIC, Crypt4ghHeaderError::BadMagic, "wrong-magic")?;
        expect_err(TRUNCATED, Crypt4ghHeaderError::TooShort, "truncated")?;
        let mut ver2 = WELL_FORMED.to_vec();
        ver2[8..12].copy_from_slice(&2u32.to_le_bytes());
        expect_err(
            &ver2,
            Crypt4ghHeaderError::UnsupportedVersion(2),
            "version-2",
        )?;
        let mut empty = WELL_FORMED.to_vec();
        empty[12..16].copy_from_slice(&0u32.to_le_bytes());
        expect_err(&empty, Crypt4ghHeaderError::NoPackets, "zero-packets")?;
        Ok::<(), String>(())
    })();
    TestCaseResult::from_outcome(
        case_name(id),
        ComplianceLevel::Level5,
        TestCategory::Security,
        res.map_err(|e| format!("{code}: {e}", code = case_code(id))),
    )
}

fn expect_err(bytes: &[u8], want: Crypt4ghHeaderError, label: &str) -> Result<(), String> {
    match validate_crypt4gh_header(bytes) {
        Err(got) if got == want => Ok(()),
        Err(got) => Err(format!(
            "{label}: expected {want}, got {got} (not dumping bytes)"
        )),
        Ok(()) => Err(format!(
            "{label}: invalid envelope was accepted (not dumping bytes)"
        )),
    }
}

/// Black-box: if the target returns Crypt4GH magic, the envelope layout must hold
/// (`HLX-AUTH-054`). Otherwise skip. Never sends a client secret key.
async fn case_http_envelope(endpoint: &str) -> Result<TestCaseResult> {
    let id = "auth.helix.crypt4gh.http_envelope";
    let name = case_name(id);
    let code = case_code(id);
    let skip = |reason: String| {
        TestCaseResult::skip(
            name,
            ComplianceLevel::Level5,
            TestCategory::Security,
            crate::redact::redact_text(&reason),
        )
    };

    let client = http_client()?;
    let discovery = discover(endpoint, &client).await?;
    let Some(drs) = discovery.get(Ga4ghService::Drs) else {
        return Ok(skip(format!(
            "{code}: DRS not discovered; no object URL to probe for a Crypt4GH envelope"
        )));
    };
    let url = protected_object_url(drs.base_url().expect("DETECTED DRS has a base URL"));
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(skip(format!(
                "{code}: could not GET object ({e}); skip, not a Crypt4GH fail"
            )));
        }
    };
    let status = resp.status();
    if !status.is_success() {
        return Ok(skip(format!(
            "{code}: GET {url} returned {status}; target did not offer a 2xx body to inspect"
        )));
    }
    let bytes =
        match crate::http_safety::read_body_capped(resp, crate::http_safety::MAX_RESPONSE_BYTES)
            .await
        {
            Ok(b) => b,
            Err(_) => {
                return Ok(skip(format!(
                    "{code}: response exceeds size limit; skip, not a Crypt4GH fail"
                )));
            }
        };
    if !looks_like_crypt4gh_magic(&bytes) {
        return Ok(skip(format!(
            "{code}: 2xx body has no Crypt4GH magic; skip (plaintext DRS is not a fail)"
        )));
    }
    let res = validate_crypt4gh_header(&bytes)
        .map_err(|e| format!("{code}: body starts with Crypt4GH magic but layout failed — {e}"));
    Ok(TestCaseResult::from_outcome(
        name,
        ComplianceLevel::Level5,
        TestCategory::Security,
        res,
    ))
}

/// Run Crypt4GH protocol cases after the HTTP Security Behavior Profile.
pub async fn run_crypt4gh_cases(
    endpoint: &str,
    crypt4gh_path: Option<&Path>,
) -> Result<ServiceReport> {
    Ok(ServiceReport {
        service: ServiceKind::Crypt4gh,
        tests: vec![
            case_well_formed(crypt4gh_path)?,
            case_invalid_rejected(),
            case_http_envelope(endpoint).await?,
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::report::TestStatus;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn fixture(name: &str) -> Vec<u8> {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test-fixtures/crypt4gh")
            .join(name);
        std::fs::read(p).unwrap()
    }

    #[test]
    fn well_formed_fixture_passes() {
        validate_crypt4gh_header(&fixture("well-formed.c4gh")).unwrap();
    }

    #[test]
    fn wrong_magic_fails_without_dumping() {
        let err = validate_crypt4gh_header(&fixture("wrong-magic.c4gh")).unwrap_err();
        assert_eq!(err, Crypt4ghHeaderError::BadMagic);
        assert!(!err.to_string().contains("NOTC4GH"));
        let raw = fixture("wrong-magic.c4gh");
        let dump: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        assert!(!err.to_string().contains(&dump));
    }

    #[test]
    fn truncated_fails() {
        let err = validate_crypt4gh_header(&fixture("truncated.c4gh")).unwrap_err();
        assert_eq!(err, Crypt4ghHeaderError::TooShort);
    }

    #[test]
    fn invalid_rejected_case_passes_on_negative_fixtures() {
        let t = case_invalid_rejected();
        assert_eq!(t.status, TestStatus::Pass, "{:?}", t.error);
        assert!(t.error.is_none());
    }

    #[test]
    fn well_formed_case_fails_when_pointed_at_wrong_magic() {
        let p =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/crypt4gh/wrong-magic.c4gh");
        let t = case_well_formed(Some(&p)).unwrap();
        assert_eq!(t.status, TestStatus::Fail);
        assert!(t.error.as_deref().unwrap_or("").contains("HLX-AUTH-050"));
        assert!(!t.error.as_deref().unwrap_or("").contains("NOTC4GH"));
    }

    #[test]
    fn crypt4gh_file_oversize_is_rejected_without_dumping() {
        let p = std::env::temp_dir().join(format!("helix-c4gh-oversize-{}", std::process::id()));
        let blob = vec![b'C'; (crate::http_safety::MAX_CRYPT4GH_FILE_BYTES as usize) + 1];
        std::fs::write(&p, &blob).unwrap();
        let err = case_well_formed(Some(&p)).unwrap_err().to_string();
        std::fs::remove_file(&p).ok();
        assert!(err.contains("bytes"), "{err}");
        assert!(!err.contains(&"C".repeat(20)), "{err}");
    }

    #[tokio::test]
    async fn http_envelope_skips_plaintext_json() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"test-object-1"}"#))
            .mount(&server)
            .await;
        let t = case_http_envelope(&server.uri()).await.unwrap();
        assert_eq!(t.status, TestStatus::Skip, "{:?}", t.error);
        assert!(t.error.as_deref().unwrap_or("").contains("HLX-AUTH-054"));
    }

    #[tokio::test]
    async fn http_envelope_passes_on_well_formed_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(WELL_FORMED.to_vec()))
            .mount(&server)
            .await;
        let t = case_http_envelope(&server.uri()).await.unwrap();
        assert_eq!(t.status, TestStatus::Pass, "{:?}", t.error);
    }

    #[tokio::test]
    async fn http_envelope_fails_when_magic_present_but_layout_broken() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/objects/test-object-1"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(TRUNCATED.to_vec()))
            .mount(&server)
            .await;
        let t = case_http_envelope(&server.uri()).await.unwrap();
        assert_eq!(t.status, TestStatus::Fail, "{:?}", t.error);
        assert!(t.error.as_deref().unwrap_or("").contains("HLX-AUTH-054"));
    }
}
