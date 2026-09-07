// SPDX-License-Identifier: Apache-2.0
//! Helix VERIFY CLI library. HelixTest already runs; this productizes discovery + `helix verify`.
//! Not HELIOS (no signed evidence / RO-Crate / PDF).

pub mod adapter;
pub mod authorization;
pub mod bench;
pub mod checker;
pub mod claim_integrity;
pub mod claims;
pub mod compare;
pub mod coverage;
pub mod diagnostics;
pub mod differential;
pub mod discover;
pub mod fixture;
pub mod guardrails;
pub mod http_safety;
pub mod identity;
pub mod independence;
pub mod interop;
pub mod layer;
pub mod live_evidence;
pub mod model;
pub mod mutation;
pub mod negative_control;
pub mod profile;
pub mod provenance;
pub mod redact;
pub mod report;
pub mod repro;
pub mod run_identity;
pub mod sanitize;
pub mod security;
pub mod standards;
pub mod target;
pub mod traceability;
pub mod verify;

/// HelixTest `HttpClient` installs tracing on first GET (default includes `common=debug`).
/// Helix's report is stdout. If `RUST_LOG` is unset, default to `error`.
pub fn default_client_log_filter() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "error");
    }
}
