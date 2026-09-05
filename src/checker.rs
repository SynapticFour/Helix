// SPDX-License-Identifier: Apache-2.0
//! Executed DRS checker identity. Not VERSIONS.lock git SHA. Not HELIOS.

use std::sync::Mutex;

static LOCK_DIGEST_LIE: Mutex<Option<String>> = Mutex::new(None);

/// Test-only: next pin check uses this expected digest instead of VERSIONS.lock.
/// Production never sets this. Does not change the executed checker identity.
pub fn set_lie_lock_checker_digest(hex: Option<&str>) {
    *LOCK_DIGEST_LIE.lock().expect("lock digest lie mutex") = hex.map(str::to_string);
}

pub fn executed_checker_source_sha256() -> &'static str {
    framework::drs::executed_checker_source_sha256()
}

pub fn executed_checker_id() -> String {
    framework::drs::executed_checker_id()
}

pub fn expected_checker_source_sha256() -> String {
    if let Some(lie) = LOCK_DIGEST_LIE
        .lock()
        .expect("lock digest lie mutex")
        .clone()
    {
        return lie;
    }
    env!("HELIX_EXPECTED_CHECKER_SOURCE_SHA256").to_string()
}

/// When VERSIONS.lock (or a test lie) does not match the compiled checker.
pub fn checker_pin_mismatch() -> Option<String> {
    let exec = executed_checker_source_sha256();
    let exp = expected_checker_source_sha256();
    if exec == exp {
        None
    } else {
        Some(format!(
            "checker identity mismatch: executed helixtest-drs:{exec} != VERSIONS.lock HELIXTEST_CHECKER_SOURCE_SHA256={exp}"
        ))
    }
}

pub fn require_checker_pin() -> anyhow::Result<()> {
    if let Some(msg) = checker_pin_mismatch() {
        anyhow::bail!("{msg}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executed_digest_is_sha256_hex() {
        let d = executed_checker_source_sha256();
        assert_eq!(d.len(), 64);
        assert!(d.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(executed_checker_id().starts_with("helixtest-drs:"));
        assert_eq!(expected_checker_source_sha256(), d);
    }

    #[test]
    fn lie_lock_digest_is_detected() {
        set_lie_lock_checker_digest(Some(&"0".repeat(64)));
        let msg = checker_pin_mismatch().expect("mismatch");
        set_lie_lock_checker_digest(None);
        assert!(msg.contains("checker identity mismatch"), "{msg}");
        assert!(checker_pin_mismatch().is_none());
    }
}
