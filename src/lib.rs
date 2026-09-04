// SPDX-License-Identifier: Apache-2.0
//! Helix VERIFY CLI library. HelixTest already runs; this productizes discovery + `helix verify`.
//! Not HELIOS (no signed evidence / RO-Crate / PDF).

pub mod adapter;
pub mod bench;
pub mod compare;
pub mod diagnostics;
pub mod discover;
pub mod http_safety;
pub mod identity;
pub mod model;
pub mod profile;
pub mod redact;
pub mod report;
pub mod run_identity;
pub mod security;
pub mod verify;

/// HelixTest `HttpClient` installs tracing on first GET (default includes `common=debug`).
/// Helix's report is stdout. If `RUST_LOG` is unset, default to `error`.
pub fn default_client_log_filter() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "error");
    }
}
