// SPDX-License-Identifier: Apache-2.0
//! Reproducibility helpers. Wall-clock and bind ports are recorded, not hashed.
//!
//! This is not bit-for-bit file identity. It is not HELIOS. It is not
//! certification. Authorship is irrelevant: compare recorded fields.

use serde_json::Value;

use crate::model::{VerificationRun, VerificationStatus};

/// Sentinel used only when comparing two runs. Not written by `helix verify`.
pub const TIMESTAMP_PLACEHOLDER: &str = "1970-01-01T00:00:00Z";

/// Replace wall-clock `timestamp` so two otherwise identical runs compare equal.
/// Does not rewrite `target.url` (ephemeral mock ports are a known difference
/// across processes).
pub fn canonicalize_verify_json(v: &mut Value) {
    if v.get("timestamp").is_some() {
        v["timestamp"] = Value::String(TIMESTAMP_PLACEHOLDER.to_string());
    }
}

/// Stable check outcomes: id, status, code. Order is catalog sort (code, then id).
pub fn outcome_fingerprint(run: &VerificationRun) -> Vec<(String, String, String)> {
    let mut rows: Vec<_> = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .map(|r| {
            (
                r.id.clone(),
                status_token(r.status).to_string(),
                r.code.clone(),
            )
        })
        .collect();
    rows.sort();
    rows
}

/// Fail/error rows: id, diagnostic expected, diagnostic observed (or message).
pub fn failure_fingerprint(run: &VerificationRun) -> Vec<(String, String, String)> {
    let mut rows: Vec<_> = run
        .executed
        .iter()
        .filter(|r| {
            matches!(
                r.status,
                VerificationStatus::Fail | VerificationStatus::Error
            )
        })
        .map(|r| {
            let expected = r
                .diagnostic
                .as_ref()
                .map(|d| d.expected.clone())
                .unwrap_or_default();
            let observed = r
                .diagnostic
                .as_ref()
                .map(|d| d.observed.clone())
                .or_else(|| r.message.clone())
                .unwrap_or_default();
            (r.id.clone(), expected, observed)
        })
        .collect();
    rows.sort();
    rows
}

fn status_token(status: VerificationStatus) -> &'static str {
    match status {
        VerificationStatus::Pass => "pass",
        VerificationStatus::Fail => "fail",
        VerificationStatus::Skip => "skip",
        VerificationStatus::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_replaces_only_timestamp() {
        let mut v = serde_json::json!({
            "timestamp": "2026-09-05T03:00:00Z",
            "target": { "url": "http://127.0.0.1:9" }
        });
        canonicalize_verify_json(&mut v);
        assert_eq!(v["timestamp"], TIMESTAMP_PLACEHOLDER);
        assert_eq!(v["target"]["url"], "http://127.0.0.1:9");
    }
}
