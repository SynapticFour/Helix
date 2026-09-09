// SPDX-License-Identifier: Apache-2.0
//! Repeatable-run metadata. Helix version, workload, target, host, runtime.

use serde::Serialize;

use super::rss::rss_source;
use super::workload::{WORKLOAD_ID, WORKLOAD_VERSION};
use super::MeasureConfig;

/// HTTP client timeouts used for bench (same as `http_safety::http_client`).
pub use crate::http_safety::{HTTP_CONNECT_TIMEOUT_SECS, HTTP_REQUEST_TIMEOUT_SECS};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeInfo {
    pub rust_msrv: String,
    pub http_request_timeout_secs: u64,
    pub http_connect_timeout_secs: u64,
    pub rss_source: String,
}

impl RuntimeInfo {
    pub fn current(collect_rss: bool) -> Self {
        Self {
            rust_msrv: env!("CARGO_PKG_RUST_VERSION").to_string(),
            http_request_timeout_secs: HTTP_REQUEST_TIMEOUT_SECS,
            http_connect_timeout_secs: HTTP_CONNECT_TIMEOUT_SECS,
            rss_source: if collect_rss {
                rss_source().to_string()
            } else {
                "disabled".to_string()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BenchMetadata {
    pub helix_version: String,
    pub workload_id: String,
    pub workload_version: String,
    pub target_url: String,
    pub target_label: String,
    pub timestamp: String,
    pub os: String,
    pub arch: String,
    pub runtime: RuntimeInfo,
    pub repetitions: u32,
    pub warmup: u32,
}

impl BenchMetadata {
    pub fn capture(target_url: &str, target_label: &str, cfg: &MeasureConfig) -> Self {
        Self {
            helix_version: env!("CARGO_PKG_VERSION").to_string(),
            workload_id: WORKLOAD_ID.to_string(),
            workload_version: WORKLOAD_VERSION.to_string(),
            target_url: target_url.to_string(),
            target_label: target_label.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            runtime: RuntimeInfo::current(cfg.collect_rss),
            repetitions: cfg.repetitions,
            warmup: cfg.warmup,
        }
    }
}

/// Whether two samples may be read as a same-environment compare.
/// Different OS/arch/runtime is never a silent apples-to-apples diff.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvironmentComparison {
    pub comparable: bool,
    pub basis: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomparable_reason: Option<String>,
}

pub fn compare_environments(a: &BenchMetadata, b: &BenchMetadata) -> EnvironmentComparison {
    let mut mismatches = Vec::new();
    if a.os != b.os {
        mismatches.push(format!("os {} vs {}", a.os, b.os));
    }
    if a.arch != b.arch {
        mismatches.push(format!("arch {} vs {}", a.arch, b.arch));
    }
    if a.helix_version != b.helix_version {
        mismatches.push(format!(
            "helix_version {} vs {}",
            a.helix_version, b.helix_version
        ));
    }
    if a.workload_id != b.workload_id || a.workload_version != b.workload_version {
        mismatches.push(format!(
            "workload {}@{} vs {}@{}",
            a.workload_id, a.workload_version, b.workload_id, b.workload_version
        ));
    }
    if a.runtime.http_request_timeout_secs != b.runtime.http_request_timeout_secs
        || a.runtime.http_connect_timeout_secs != b.runtime.http_connect_timeout_secs
    {
        mismatches.push("HTTP client timeouts differ".to_string());
    }
    if mismatches.is_empty() {
        EnvironmentComparison {
            comparable: true,
            basis: format!(
                "same Helix {}, workload {}, os {}, arch {}, HTTP timeouts {}/{}s",
                a.helix_version,
                a.workload_id,
                a.os,
                a.arch,
                a.runtime.http_request_timeout_secs,
                a.runtime.http_connect_timeout_secs
            ),
            incomparable_reason: None,
        }
    } else {
        EnvironmentComparison {
            comparable: false,
            basis:
                "explicitly marked incomparable; do not read percent diffs as same-host regression"
                    .to_string(),
            incomparable_reason: Some(mismatches.join("; ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(os: &str, arch: &str) -> BenchMetadata {
        BenchMetadata {
            helix_version: "0.1.0".into(),
            workload_id: WORKLOAD_ID.into(),
            workload_version: WORKLOAD_VERSION.into(),
            target_url: "http://127.0.0.1:9".into(),
            target_label: "x".into(),
            timestamp: "2026-09-04T00:00:00Z".into(),
            os: os.into(),
            arch: arch.into(),
            runtime: RuntimeInfo {
                rust_msrv: "1.88".into(),
                http_request_timeout_secs: HTTP_REQUEST_TIMEOUT_SECS,
                http_connect_timeout_secs: HTTP_CONNECT_TIMEOUT_SECS,
                rss_source: "unavailable".into(),
            },
            repetitions: 5,
            warmup: 1,
        }
    }

    #[test]
    fn same_host_is_comparable() {
        let env = compare_environments(&meta("linux", "x86_64"), &meta("linux", "x86_64"));
        assert!(env.comparable);
        assert!(env.incomparable_reason.is_none());
    }

    #[test]
    fn different_os_is_marked_not_silent() {
        let env = compare_environments(&meta("linux", "x86_64"), &meta("macos", "x86_64"));
        assert!(!env.comparable);
        assert!(env
            .incomparable_reason
            .as_deref()
            .unwrap_or("")
            .contains("os"));
    }
}
