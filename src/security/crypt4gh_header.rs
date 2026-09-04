// SPDX-License-Identifier: Apache-2.0
//! Crypt4GH unencrypted header layout (spec magic/version/packets).
//! Does not decrypt, does not load private keys, does not print key material.

use anyhow::Result;
use common::report::{ComplianceLevel, TestCaseResult, TestCategory};
use std::path::Path;

const MAGIC: &[u8; 8] = b"crypt4gh";

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

/// Parse unencrypted header structure only. Packet bodies are not interpreted (they may be encrypted).
pub fn validate_crypt4gh_header(bytes: &[u8]) -> Result<(), Crypt4ghHeaderError> {
    if bytes.len() < 16 {
        return Err(Crypt4ghHeaderError::TooShort);
    }
    if &bytes[0..8] != MAGIC {
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

pub fn crypt4gh_header_case(path: Option<&Path>) -> Result<TestCaseResult> {
    const NAME: &str =
        "Security: Crypt4GH header structure is well-formed (no key material in output)";
    // Covers: rejecting files that are not Crypt4GH envelopes so a gateway does not
    // treat arbitrary blobs as encrypted payloads, without ever printing keys.
    let bytes = match path {
        Some(path) => std::fs::read(path)?,
        None => include_bytes!("../../test-fixtures/crypt4gh/well-formed.c4gh").to_vec(),
    };
    let res = validate_crypt4gh_header(&bytes).map_err(|e| e.to_string());
    Ok(TestCaseResult::from_outcome(
        NAME,
        ComplianceLevel::Level5,
        TestCategory::Security,
        res,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn truncated_fails() {
        let err = validate_crypt4gh_header(&fixture("truncated.c4gh")).unwrap_err();
        assert_eq!(err, Crypt4ghHeaderError::TooShort);
    }
}
