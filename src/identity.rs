// SPDX-License-Identifier: Apache-2.0
//! Stable Helix check identities (`id` + `code`) and HelixTest name mapping.
//!
//! HelixTest test names are not renamed. Changing an assigned `id` or `code`
//! is a compatibility change. See docs/TEST_IDENTITY.md.
//! Not HELIOS. Not certification.

use serde::{Deserialize, Serialize};

/// Default severity if this check fails. Not a compliance score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

/// Helix category. Not HelixTest `ComplianceLevel` or `TestCategory`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckCategory {
    Discovery,
    Schema,
    Lifecycle,
    Checksum,
    Robustness,
    Security,
    Performance,
    Other,
}

/// One catalog entry. `helixtest_names` are exact HelixTest `TestCaseResult.name` strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSpec {
    pub id: &'static str,
    pub code: &'static str,
    pub name: &'static str,
    pub service: &'static str,
    pub category: CheckCategory,
    pub severity: Severity,
    pub helixtest_names: &'static [&'static str],
}

impl CheckSpec {
    pub fn wraps_helixtest(&self) -> bool {
        !self.helixtest_names.is_empty()
    }
}

const fn s(
    id: &'static str,
    code: &'static str,
    name: &'static str,
    service: &'static str,
    category: CheckCategory,
    severity: Severity,
    helixtest_names: &'static [&'static str],
) -> CheckSpec {
    CheckSpec {
        id,
        code,
        name,
        service,
        category,
        severity,
        helixtest_names,
    }
}

/// Helix `id`s for the DRS verify suite (HelixTest wraps). Order is catalog order.
pub const DRS_VERIFY_IDS: [&str; 5] = [
    "drs.object.reachable",
    "drs.object.schema",
    "drs.object.checksum",
    "drs.object.range",
    "drs.object.not_found",
];

/// Helix `id`s for the WES verify suite (HelixTest wraps). Order is catalog order.
pub const WES_VERIFY_IDS: [&str; 8] = [
    "wes.service_info.reachable",
    "wes.service_info.schema",
    "wes.run.lifecycle_success",
    "wes.run.failure_state",
    "wes.run.missing_inputs",
    "wes.run.incompatible_type",
    "wes.run.invalid_workflow",
    "wes.run.scatter_gather",
];

/// Assigned catalog. Append only; do not reorder in a way that changes codes.
/// Frozen pairs are asserted in tests.
pub const SPECS: &[CheckSpec] = &[
    // DRS 001–005 (helix verify wrap)
    s(
        "drs.object.reachable",
        "HLX-DRS-001",
        "DRS object endpoint is reachable",
        "drs",
        CheckCategory::Robustness,
        Severity::Error,
        &["DRS object endpoint reachable"],
    ),
    s(
        "drs.object.schema",
        "HLX-DRS-002",
        "DRS object matches OpenAPI and has access methods",
        "drs",
        CheckCategory::Schema,
        Severity::Error,
        &["DRS DrsObject OpenAPI + access_methods"],
    ),
    s(
        "drs.object.checksum",
        "HLX-DRS-003",
        "DRS object checksum is correct",
        "drs",
        CheckCategory::Checksum,
        Severity::Error,
        &["DRS checksum correctness"],
    ),
    s(
        "drs.object.range",
        "HLX-DRS-004",
        "DRS object supports HTTP Range",
        "drs",
        CheckCategory::Robustness,
        Severity::Error,
        &["DRS HTTP Range support"],
    ),
    s(
        "drs.object.not_found",
        "HLX-DRS-005",
        "Unknown DRS object returns 404",
        "drs",
        CheckCategory::Robustness,
        Severity::Error,
        &["DRS invalid object id returns 404"],
    ),
    // WES 001–008
    s(
        "wes.service_info.reachable",
        "HLX-WES-001",
        "WES service-info is reachable",
        "wes",
        CheckCategory::Robustness,
        Severity::Error,
        &["WES service-info reachable"],
    ),
    s(
        "wes.service_info.schema",
        "HLX-WES-002",
        "WES service-info matches GA4GH schema",
        "wes",
        CheckCategory::Schema,
        Severity::Error,
        &["WES service-info schema (GA4GH official)"],
    ),
    s(
        "wes.run.lifecycle_success",
        "HLX-WES-003",
        "WES echo workflow reaches success",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES lifecycle success echo (API may show QUEUED/INITIALIZING/RUNNING before COMPLETE)"],
    ),
    s(
        "wes.run.failure_state",
        "HLX-WES-004",
        "WES reports failure for a bad workflow",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES failure state for bad workflow"],
    ),
    s(
        "wes.run.missing_inputs",
        "HLX-WES-005",
        "WES errors when inputs are missing",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES missing inputs leads to error state"],
    ),
    s(
        "wes.run.incompatible_type",
        "HLX-WES-006",
        "WES errors on incompatible workflow_type",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES incompatible workflow_type leads to error state"],
    ),
    s(
        "wes.run.invalid_workflow",
        "HLX-WES-007",
        "WES errors on an invalid workflow",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES invalid workflow leads to error state"],
    ),
    s(
        "wes.run.scatter_gather",
        "HLX-WES-008",
        "WES scatter/gather workflow",
        "wes",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["WES scatter/gather workflow"],
    ),
    // TES
    s(
        "tes.tasks.reachable",
        "HLX-TES-001",
        "TES /tasks is reachable",
        "tes",
        CheckCategory::Robustness,
        Severity::Error,
        &["TES /tasks reachable"],
    ),
    s(
        "tes.task.schema",
        "HLX-TES-002",
        "TES task create and status match schema",
        "tes",
        CheckCategory::Schema,
        Severity::Error,
        &["TES task schema (create + status)"],
    ),
    s(
        "tes.task.lifecycle_checksum",
        "HLX-TES-003",
        "TES task lifecycle and output checksum",
        "tes",
        CheckCategory::Checksum,
        Severity::Error,
        &["TES task lifecycle + checksum (non-terminal states allowed until terminal)"],
    ),
    // TRS
    s(
        "trs.tools.reachable",
        "HLX-TRS-001",
        "TRS /tools is reachable",
        "trs",
        CheckCategory::Robustness,
        Severity::Error,
        &["TRS /tools reachable"],
    ),
    s(
        "trs.tools.schema",
        "HLX-TRS-002",
        "TRS tools and versions match schema",
        "trs",
        CheckCategory::Schema,
        Severity::Error,
        &["TRS tools and versions schema"],
    ),
    s(
        "trs.descriptor.retrieve",
        "HLX-TRS-003",
        "TRS descriptor can be retrieved",
        "trs",
        CheckCategory::Robustness,
        Severity::Error,
        &["TRS descriptor retrieval"],
    ),
    // Beacon
    s(
        "beacon.query.reachable",
        "HLX-BEACON-001",
        "Beacon /query is reachable",
        "beacon",
        CheckCategory::Robustness,
        Severity::Error,
        &["Beacon /query reachable"],
    ),
    s(
        "beacon.query.boolean_schema",
        "HLX-BEACON-002",
        "Beacon boolean response matches schema",
        "beacon",
        CheckCategory::Schema,
        Severity::Error,
        &["Beacon boolean response (official schema)"],
    ),
    s(
        "beacon.variant.known_exists",
        "HLX-BEACON-003",
        "Beacon reports a known variant exists",
        "beacon",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["Beacon known variant exists"],
    ),
    s(
        "beacon.variant.negative_absent",
        "HLX-BEACON-004",
        "Beacon reports a negative variant as absent",
        "beacon",
        CheckCategory::Lifecycle,
        Severity::Error,
        &["Beacon negative variant not exists"],
    ),
    // htsget (007/009 accept generic and Ferrum-specific HelixTest names)
    s(
        "htsget.reads.service_info",
        "HLX-HTSGET-001",
        "htsget reads service-info is reachable",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget reads /reads/service-info (htsget 1.3.0)"],
    ),
    s(
        "htsget.variants.service_info",
        "HLX-HTSGET-002",
        "htsget variants service-info is reachable",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget variants /variants/service-info (htsget 1.3.0)"],
    ),
    s(
        "htsget.reads.ticket.get",
        "HLX-HTSGET-003",
        "htsget GET reads ticket",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget GET reads ticket (BAM + DRS stream URL)"],
    ),
    s(
        "htsget.variants.ticket.get",
        "HLX-HTSGET-004",
        "htsget GET variants ticket",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget GET variants ticket (VCF/BCF + DRS stream URL)"],
    ),
    s(
        "htsget.variants.wrong_object",
        "HLX-HTSGET-005",
        "htsget GET variants with a reads-only object is NotFound",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &["htsget GET variants with reads-only object → NotFound"],
    ),
    s(
        "htsget.reads.ticket.post",
        "HLX-HTSGET-006",
        "htsget POST reads ticket",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget POST reads ticket (JSON body, no query)"],
    ),
    s(
        "htsget.reads.ticket.post_regions",
        "HLX-HTSGET-007",
        "htsget POST reads ticket with regions",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &[
            "htsget POST reads ticket (JSON body with regions)",
            "htsget POST reads ticket with regions → InvalidInput (Ferrum does not slice)",
        ],
    ),
    s(
        "htsget.variants.ticket.post",
        "HLX-HTSGET-008",
        "htsget POST variants ticket",
        "htsget",
        CheckCategory::Schema,
        Severity::Error,
        &["htsget POST variants ticket (JSON body, no query)"],
    ),
    s(
        "htsget.variants.ticket.post_regions",
        "HLX-HTSGET-009",
        "htsget POST variants ticket with regions",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &[
            "htsget POST variants ticket (JSON body with regions)",
            "htsget POST variants ticket with regions → InvalidInput (Ferrum does not slice)",
        ],
    ),
    s(
        "htsget.reads.post_query_invalid",
        "HLX-HTSGET-010",
        "htsget POST reads with query params is InvalidInput",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &["htsget POST reads with query params → InvalidInput"],
    ),
    s(
        "htsget.reads.format_cram",
        "HLX-HTSGET-011",
        "htsget GET reads CRAM on a BAM object is UnsupportedFormat",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &["htsget GET reads ?format=CRAM on BAM object → UnsupportedFormat"],
    ),
    s(
        "htsget.reads.class_header",
        "HLX-HTSGET-012",
        "htsget GET reads class=header is InvalidInput",
        "htsget",
        CheckCategory::Robustness,
        Severity::Error,
        &["htsget GET reads ?class=header → InvalidInput"],
    ),
    s(
        "htsget.dataset.auth",
        "HLX-HTSGET-013",
        "htsget dataset auth fail-closed then allows a valid token",
        "htsget",
        CheckCategory::Security,
        Severity::Error,
        &["htsget dataset auth (403 without token, 200 with Passport/JWT)"],
    ),
    s(
        "htsget.suite.unresolved",
        "HLX-HTSGET-014",
        "htsget suite skipped when the service URL is unresolved",
        "htsget",
        CheckCategory::Other,
        Severity::Error,
        &["htsget suite (service-info, tickets, POST, errors)"],
    ),
    // Auth HelixTest HMAC wrap 001–006
    s(
        "auth.service_info.reachable",
        "HLX-AUTH-001",
        "Auth service-info is reachable",
        "auth",
        CheckCategory::Robustness,
        Severity::Error,
        &["Auth /service-info reachable (auth_url)"],
    ),
    s(
        "auth.token.valid",
        "HLX-AUTH-002",
        "Valid HMAC token grants DRS access",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &["Auth (HMAC JWT fixture): valid token grants DRS access"],
    ),
    s(
        "auth.token.expired",
        "HLX-AUTH-003",
        "Expired HMAC token is rejected",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &["Auth (HMAC JWT fixture): expired token rejected"],
    ),
    s(
        "auth.token.garbage",
        "HLX-AUTH-004",
        "Garbage bearer is rejected",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &["Auth (HMAC JWT fixture): garbage bearer rejected"],
    ),
    s(
        "auth.token.wrong_scope",
        "HLX-AUTH-005",
        "Wrong-scope HMAC token is denied",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &["Auth (HMAC JWT fixture): wrong scope denied"],
    ),
    s(
        "auth.token.missing",
        "HLX-AUTH-006",
        "Missing token returns 401",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &["Auth (HMAC JWT fixture): missing token returns 401"],
    ),
    // Helix-native security (not a HelixTest wrap)
    s(
        "auth.helix.token.valid",
        "HLX-AUTH-010",
        "Security: valid token grants access",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.token.expired",
        "HLX-AUTH-011",
        "Security: expired token rejected with 401",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.token.wrong_scope",
        "HLX-AUTH-012",
        "Security: wrong scope denied",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.token.manipulated",
        "HLX-AUTH-013",
        "Security: invalid or manipulated token rejected",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.token.wrong_audience",
        "HLX-AUTH-014",
        "Security: token for another service rejected",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.crypt4gh.header",
        "HLX-AUTH-050",
        "Security: Crypt4GH header structure is well-formed (no key material in output)",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.crypt4gh.invalid_rejected",
        "HLX-AUTH-053",
        "Crypt4GH: invalid envelope is rejected (layout only)",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    s(
        "auth.helix.crypt4gh.http_envelope",
        "HLX-AUTH-054",
        "Crypt4GH: HTTP body is a Crypt4GH envelope when magic is present",
        "auth",
        CheckCategory::Security,
        Severity::Error,
        &[],
    ),
    // Discovery (Helix-native)
    s(
        "discovery.drs",
        "HLX-DISCOVERY-001",
        "DRS is present under the target URL",
        "drs",
        CheckCategory::Discovery,
        Severity::Error,
        &[],
    ),
    s(
        "discovery.wes",
        "HLX-DISCOVERY-002",
        "WES is present under the target URL",
        "wes",
        CheckCategory::Discovery,
        Severity::Error,
        &[],
    ),
    s(
        "discovery.tes",
        "HLX-DISCOVERY-003",
        "TES is present under the target URL",
        "tes",
        CheckCategory::Discovery,
        Severity::Error,
        &[],
    ),
    s(
        "discovery.trs",
        "HLX-DISCOVERY-004",
        "TRS is present under the target URL",
        "trs",
        CheckCategory::Discovery,
        Severity::Error,
        &[],
    ),
    s(
        "discovery.htsget",
        "HLX-DISCOVERY-005",
        "htsget is present under the target URL",
        "htsget",
        CheckCategory::Discovery,
        Severity::Error,
        &[],
    ),
    // Bench (Helix-native)
    s(
        "bench.get.health",
        "HLX-BENCH-001",
        "GET /health",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.get.drs_service_info",
        "HLX-BENCH-002",
        "GET /ga4gh/drs/v1/service-info",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.get.drs_object",
        "HLX-BENCH-003",
        "GET /ga4gh/drs/v1/objects/test-object-1",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.wall_ms",
        "HLX-BENCH-010",
        "Client wall time (median of measured runs)",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.rss_kb",
        "HLX-BENCH-011",
        "Helix process RSS (Linux)",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.error_rate",
        "HLX-BENCH-012",
        "Request error rate",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.p95_ms",
        "HLX-BENCH-013",
        "Sample p95 wall time (reported at >= 20 measured runs)",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.min_ms",
        "HLX-BENCH-014",
        "Minimum measured wall time",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.max_ms",
        "HLX-BENCH-015",
        "Maximum measured wall time",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
    s(
        "bench.metric.bytes",
        "HLX-BENCH-016",
        "Response body bytes (measured runs)",
        "bench",
        CheckCategory::Performance,
        Severity::Warn,
        &[],
    ),
];

/// Allowed `HLX-<FAMILY>-NNN` families. Documented in TEST_IDENTITY.md.
pub const FAMILIES: &[&str] = &[
    "DRS",
    "WES",
    "TES",
    "TRS",
    "HTSGET",
    "BEACON",
    "AUTH",
    "BENCH",
    "DISCOVERY",
];

pub fn spec_by_id(id: &str) -> Option<&'static CheckSpec> {
    SPECS.iter().find(|s| s.id == id)
}

pub fn spec_by_code(code: &str) -> Option<&'static CheckSpec> {
    SPECS.iter().find(|s| s.code == code)
}

pub fn spec_by_helixtest_name(name: &str) -> Option<&'static CheckSpec> {
    SPECS.iter().find(|s| s.helixtest_names.contains(&name))
}

/// Panic if a catalog id is missing (known compile-time ids only).
pub fn spec(id: &str) -> &'static CheckSpec {
    spec_by_id(id).unwrap_or_else(|| panic!("unknown Helix check id: {id}"))
}

/// Catalog rows for `helix verify` DRS checks, in stable id order.
pub fn drs_verify_specs() -> impl Iterator<Item = &'static CheckSpec> {
    DRS_VERIFY_IDS.iter().map(|id| spec(id))
}

/// Catalog rows for `helix verify` WES checks, in stable id order.
pub fn wes_verify_specs() -> impl Iterator<Item = &'static CheckSpec> {
    WES_VERIFY_IDS.iter().map(|id| spec(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AUTH_CASE_NAMES, CRYPT4GH_CASE_NAME};
    use crate::verify::{DRS_CHECK_NAMES, WES_CHECK_NAMES};

    fn parse_code(code: &str) -> Option<(&str, u16)> {
        let rest = code.strip_prefix("HLX-")?;
        let (family, num) = rest.rsplit_once('-')?;
        let n: u16 = num.parse().ok()?;
        if num.len() != 3 {
            return None;
        }
        Some((family, n))
    }

    /// Frozen (id, code). Adding a new pair is fine; changing these is a compatibility break.
    const FROZEN_ID_CODE: &[(&str, &str)] = &[
        ("drs.object.reachable", "HLX-DRS-001"),
        ("drs.object.schema", "HLX-DRS-002"),
        ("drs.object.checksum", "HLX-DRS-003"),
        ("drs.object.range", "HLX-DRS-004"),
        ("drs.object.not_found", "HLX-DRS-005"),
        ("wes.service_info.reachable", "HLX-WES-001"),
        ("wes.service_info.schema", "HLX-WES-002"),
        ("wes.run.lifecycle_success", "HLX-WES-003"),
        ("wes.run.failure_state", "HLX-WES-004"),
        ("wes.run.missing_inputs", "HLX-WES-005"),
        ("wes.run.incompatible_type", "HLX-WES-006"),
        ("wes.run.invalid_workflow", "HLX-WES-007"),
        ("wes.run.scatter_gather", "HLX-WES-008"),
        ("tes.tasks.reachable", "HLX-TES-001"),
        ("tes.task.schema", "HLX-TES-002"),
        ("tes.task.lifecycle_checksum", "HLX-TES-003"),
        ("trs.tools.reachable", "HLX-TRS-001"),
        ("trs.tools.schema", "HLX-TRS-002"),
        ("trs.descriptor.retrieve", "HLX-TRS-003"),
        ("beacon.query.reachable", "HLX-BEACON-001"),
        ("beacon.query.boolean_schema", "HLX-BEACON-002"),
        ("beacon.variant.known_exists", "HLX-BEACON-003"),
        ("beacon.variant.negative_absent", "HLX-BEACON-004"),
        ("htsget.reads.service_info", "HLX-HTSGET-001"),
        ("htsget.variants.service_info", "HLX-HTSGET-002"),
        ("htsget.reads.ticket.get", "HLX-HTSGET-003"),
        ("htsget.variants.ticket.get", "HLX-HTSGET-004"),
        ("htsget.variants.wrong_object", "HLX-HTSGET-005"),
        ("htsget.reads.ticket.post", "HLX-HTSGET-006"),
        ("htsget.reads.ticket.post_regions", "HLX-HTSGET-007"),
        ("htsget.variants.ticket.post", "HLX-HTSGET-008"),
        ("htsget.variants.ticket.post_regions", "HLX-HTSGET-009"),
        ("htsget.reads.post_query_invalid", "HLX-HTSGET-010"),
        ("htsget.reads.format_cram", "HLX-HTSGET-011"),
        ("htsget.reads.class_header", "HLX-HTSGET-012"),
        ("htsget.dataset.auth", "HLX-HTSGET-013"),
        ("htsget.suite.unresolved", "HLX-HTSGET-014"),
        ("auth.service_info.reachable", "HLX-AUTH-001"),
        ("auth.token.valid", "HLX-AUTH-002"),
        ("auth.token.expired", "HLX-AUTH-003"),
        ("auth.token.garbage", "HLX-AUTH-004"),
        ("auth.token.wrong_scope", "HLX-AUTH-005"),
        ("auth.token.missing", "HLX-AUTH-006"),
        ("auth.helix.token.valid", "HLX-AUTH-010"),
        ("auth.helix.token.expired", "HLX-AUTH-011"),
        ("auth.helix.token.wrong_scope", "HLX-AUTH-012"),
        ("auth.helix.token.manipulated", "HLX-AUTH-013"),
        ("auth.helix.token.wrong_audience", "HLX-AUTH-014"),
        ("auth.helix.crypt4gh.header", "HLX-AUTH-050"),
        ("auth.helix.crypt4gh.invalid_rejected", "HLX-AUTH-053"),
        ("auth.helix.crypt4gh.http_envelope", "HLX-AUTH-054"),
        ("discovery.drs", "HLX-DISCOVERY-001"),
        ("discovery.wes", "HLX-DISCOVERY-002"),
        ("discovery.tes", "HLX-DISCOVERY-003"),
        ("discovery.trs", "HLX-DISCOVERY-004"),
        ("discovery.htsget", "HLX-DISCOVERY-005"),
        ("bench.get.health", "HLX-BENCH-001"),
        ("bench.get.drs_service_info", "HLX-BENCH-002"),
        ("bench.get.drs_object", "HLX-BENCH-003"),
        ("bench.metric.wall_ms", "HLX-BENCH-010"),
        ("bench.metric.rss_kb", "HLX-BENCH-011"),
        ("bench.metric.error_rate", "HLX-BENCH-012"),
        ("bench.metric.p95_ms", "HLX-BENCH-013"),
        ("bench.metric.min_ms", "HLX-BENCH-014"),
        ("bench.metric.max_ms", "HLX-BENCH-015"),
        ("bench.metric.bytes", "HLX-BENCH-016"),
    ];

    const FROZEN_HELIXTEST: &[(&str, &str)] = &[
        ("DRS object endpoint reachable", "drs.object.reachable"),
        (
            "DRS DrsObject OpenAPI + access_methods",
            "drs.object.schema",
        ),
        ("DRS checksum correctness", "drs.object.checksum"),
        ("DRS HTTP Range support", "drs.object.range"),
        ("DRS invalid object id returns 404", "drs.object.not_found"),
        ("WES service-info reachable", "wes.service_info.reachable"),
        (
            "WES service-info schema (GA4GH official)",
            "wes.service_info.schema",
        ),
        (
            "WES lifecycle success echo (API may show QUEUED/INITIALIZING/RUNNING before COMPLETE)",
            "wes.run.lifecycle_success",
        ),
        (
            "WES failure state for bad workflow",
            "wes.run.failure_state",
        ),
        (
            "WES missing inputs leads to error state",
            "wes.run.missing_inputs",
        ),
        (
            "WES incompatible workflow_type leads to error state",
            "wes.run.incompatible_type",
        ),
        (
            "WES invalid workflow leads to error state",
            "wes.run.invalid_workflow",
        ),
        ("WES scatter/gather workflow", "wes.run.scatter_gather"),
        ("TES /tasks reachable", "tes.tasks.reachable"),
        ("TES task schema (create + status)", "tes.task.schema"),
        (
            "TES task lifecycle + checksum (non-terminal states allowed until terminal)",
            "tes.task.lifecycle_checksum",
        ),
        ("TRS /tools reachable", "trs.tools.reachable"),
        ("TRS tools and versions schema", "trs.tools.schema"),
        ("TRS descriptor retrieval", "trs.descriptor.retrieve"),
        ("Beacon /query reachable", "beacon.query.reachable"),
        (
            "Beacon boolean response (official schema)",
            "beacon.query.boolean_schema",
        ),
        ("Beacon known variant exists", "beacon.variant.known_exists"),
        (
            "Beacon negative variant not exists",
            "beacon.variant.negative_absent",
        ),
        (
            "htsget POST reads ticket with regions → InvalidInput (Ferrum does not slice)",
            "htsget.reads.ticket.post_regions",
        ),
        (
            "htsget POST reads ticket (JSON body with regions)",
            "htsget.reads.ticket.post_regions",
        ),
        (
            "Auth (HMAC JWT fixture): valid token grants DRS access",
            "auth.token.valid",
        ),
    ];

    #[test]
    fn frozen_id_and_code_pairs_are_stable() {
        for (id, code) in FROZEN_ID_CODE {
            let spec = spec_by_id(id).unwrap_or_else(|| panic!("missing id {id}"));
            assert_eq!(
                spec.code, *code,
                "code changed for {id} — compatibility break"
            );
            assert_eq!(spec_by_code(code).map(|s| s.id), Some(*id));
        }
        assert_eq!(
            SPECS.len(),
            FROZEN_ID_CODE.len(),
            "catalog grew or shrank; append to FROZEN_ID_CODE (do not change old pairs)"
        );
    }

    #[test]
    fn ids_and_codes_are_unique() {
        let mut ids = std::collections::BTreeSet::new();
        let mut codes = std::collections::BTreeSet::new();
        let mut ht = std::collections::BTreeSet::new();
        for spec in SPECS {
            assert!(ids.insert(spec.id), "duplicate id {}", spec.id);
            assert!(codes.insert(spec.code), "duplicate code {}", spec.code);
            for name in spec.helixtest_names {
                assert!(ht.insert(*name), "HelixTest name mapped twice: {name}");
            }
        }
    }

    #[test]
    fn codes_match_family_ranges() {
        for spec in SPECS {
            let (family, n) =
                parse_code(spec.code).unwrap_or_else(|| panic!("malformed code {}", spec.code));
            assert!(
                FAMILIES.contains(&family),
                "unknown family in {}",
                spec.code
            );
            assert!(
                (1..=49).contains(&n) || (50..=99).contains(&n),
                "{} number {n} is outside assigned 001–049 / reserved-expansion 050–099",
                spec.code
            );
            if family == "AUTH" && spec.id.starts_with("auth.helix.crypt4gh") {
                assert!(
                    matches!(n, 50 | 53 | 54),
                    "{} Crypt4GH Helix-native codes are 050, 053, 054 (051–052 reserved for HelixTest secret-key HTTP)",
                    spec.code
                );
            }
        }
    }

    #[test]
    fn example_not_found_identity() {
        let spec = spec("drs.object.not_found");
        assert_eq!(spec.id, "drs.object.not_found");
        assert_eq!(spec.code, "HLX-DRS-005");
        assert_eq!(spec.name, "Unknown DRS object returns 404");
        assert_eq!(spec.service, "drs");
        assert_eq!(spec.category, CheckCategory::Robustness);
        assert_eq!(spec.severity, Severity::Error);
        assert_eq!(spec.helixtest_names, &["DRS invalid object id returns 404"]);
    }

    #[test]
    fn helix_verify_drs_names_map_to_helix_ids() {
        for name in DRS_CHECK_NAMES {
            let spec = spec_by_helixtest_name(name)
                .unwrap_or_else(|| panic!("no Helix id for HelixTest DRS name: {name}"));
            assert_eq!(spec.service, "drs");
            assert!(spec.wraps_helixtest());
        }
        assert_eq!(
            spec_by_helixtest_name("DRS invalid object id returns 404")
                .unwrap()
                .id,
            "drs.object.not_found"
        );
    }

    #[test]
    fn helix_verify_wes_names_map_to_helix_ids() {
        for name in WES_CHECK_NAMES {
            let spec = spec_by_helixtest_name(name)
                .unwrap_or_else(|| panic!("no Helix id for HelixTest WES name: {name}"));
            assert_eq!(spec.service, "wes");
            assert!(spec.wraps_helixtest());
        }
        assert_eq!(
            spec_by_helixtest_name("WES scatter/gather workflow")
                .unwrap()
                .id,
            "wes.run.scatter_gather"
        );
    }

    #[test]
    fn frozen_helixtest_name_mapping() {
        for (ht_name, id) in FROZEN_HELIXTEST {
            let spec = spec_by_helixtest_name(ht_name)
                .unwrap_or_else(|| panic!("lost mapping for HelixTest name: {ht_name}"));
            assert_eq!(spec.id, *id, "HelixTest name {ht_name} remapped");
        }
    }

    #[test]
    fn helix_security_cases_are_native_not_renames() {
        let ids = [
            "auth.helix.token.valid",
            "auth.helix.token.expired",
            "auth.helix.token.wrong_scope",
            "auth.helix.token.manipulated",
            "auth.helix.token.wrong_audience",
        ];
        for (name, id) in AUTH_CASE_NAMES.iter().zip(ids) {
            let spec = spec(id);
            assert_eq!(spec.name, *name);
            assert!(
                spec.helixtest_names.is_empty(),
                "{id} must not pretend to wrap HelixTest auth.rs"
            );
        }
        let c4 = spec("auth.helix.crypt4gh.header");
        assert_eq!(c4.name, CRYPT4GH_CASE_NAME);
        assert!(c4.helixtest_names.is_empty());
        for id in [
            "auth.helix.crypt4gh.invalid_rejected",
            "auth.helix.crypt4gh.http_envelope",
        ] {
            let s = spec(id);
            assert!(s.helixtest_names.is_empty(), "{id}");
            assert!(s.name.to_lowercase().contains("crypt4gh"));
            assert!(!s.name.to_lowercase().contains("secure"));
        }
        assert!(spec_by_helixtest_name(AUTH_CASE_NAMES[0]).is_none());
    }

    #[test]
    fn skip_is_not_encoded_in_identity() {
        for spec in SPECS {
            assert!(!spec.id.contains("pass"));
            assert!(!spec.code.contains("PASS"));
        }
    }
}
