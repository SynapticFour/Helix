// SPDX-License-Identifier: Apache-2.0
//! Evidence layer: schema vs behaviour vs security vs interoperability.
//!
//! A schema PASS must not be read as a behavioural PASS. Counts are not a
//! compliance score and must not be turned into a percentage.
//! Not HELIOS. Not GA4GH certification.

use serde::{Deserialize, Serialize};

use crate::identity::{spec_by_id, SPECS};

/// What kind of evidence a check produces. Distinct from claim taxonomy
/// (`traceability.category`) and domain (`executed[].category`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckLayer {
    Schema,
    Behavior,
    Security,
    Interoperability,
    /// Measurement only. Not a conformance layer; omitted from layer_summary.
    Benchmark,
}

impl CheckLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Behavior => "behavior",
            Self::Security => "security",
            Self::Interoperability => "interoperability",
            Self::Benchmark => "benchmark",
        }
    }

    pub fn is_conformance_layer(self) -> bool {
        !matches!(self, Self::Benchmark)
    }

    pub fn report_heading(self) -> &'static str {
        match self {
            Self::Schema => "SCHEMA",
            Self::Behavior => "BEHAVIOR",
            Self::Security => "SECURITY",
            Self::Interoperability => "INTEROPERABILITY",
            Self::Benchmark => "BENCHMARK",
        }
    }
}

/// Status token for layer counts. Kept here so this module does not import `model`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerOutcome {
    Pass,
    Fail,
    Skip,
    Error,
}

/// Per-layer counts. No percentage. No “compliant” field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayerCounts {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total: usize,
}

impl LayerCounts {
    pub fn add(&mut self, status: LayerOutcome) {
        self.total += 1;
        match status {
            LayerOutcome::Pass => self.passed += 1,
            LayerOutcome::Fail => self.failed += 1,
            LayerOutcome::Skip => self.skipped += 1,
            LayerOutcome::Error => self.errors += 1,
        }
    }

    /// NONE if this layer did not execute. All-skip is not PASS.
    pub fn verdict(&self) -> LayerVerdict {
        if self.errors > 0 {
            LayerVerdict::Error
        } else if self.failed > 0 {
            LayerVerdict::Fail
        } else if self.passed > 0 {
            LayerVerdict::Pass
        } else {
            LayerVerdict::None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerVerdict {
    Pass,
    Fail,
    Error,
    None,
}

impl LayerVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Error => "ERROR",
            Self::None => "NONE",
        }
    }
}

/// Run-level layer totals. SCHEMA PASS is not BEHAVIOR PASS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerSummary {
    pub schema: LayerCounts,
    pub behavior: LayerCounts,
    pub security: LayerCounts,
    pub interoperability: LayerCounts,
    pub note: String,
}

impl Default for LayerSummary {
    fn default() -> Self {
        Self {
            schema: LayerCounts::default(),
            behavior: LayerCounts::default(),
            security: LayerCounts::default(),
            interoperability: LayerCounts::default(),
            note: HONESTY_NOTE.to_string(),
        }
    }
}

impl LayerSummary {
    pub fn record(&mut self, layer: CheckLayer, status: LayerOutcome) {
        let counts = match layer {
            CheckLayer::Schema => &mut self.schema,
            CheckLayer::Behavior => &mut self.behavior,
            CheckLayer::Security => &mut self.security,
            CheckLayer::Interoperability => &mut self.interoperability,
            CheckLayer::Benchmark => return,
        };
        counts.add(status);
    }
}

pub const HONESTY_NOTE: &str = "SCHEMA PASS is not BEHAVIOR PASS. SECURITY PASS is not GA4GH conformance. INTEROPERABILITY PASS is not a MUST. Counts are not a score and must not be turned into a percentage.";

struct LayerRow {
    id: &'static str,
    layer: CheckLayer,
    request: Option<&'static str>,
}

/// Helix-native / HelixTest wrap classification. Not a GA4GH MUST list.
const LAYERS: &[LayerRow] = &[
    LayerRow {
        id: "drs.object.reachable",
        layer: CheckLayer::Interoperability,
        request: Some("GET {drs}/objects/{object_id}"),
    },
    LayerRow {
        id: "drs.object.schema",
        layer: CheckLayer::Schema,
        request: Some("GET {drs}/objects/{object_id}"),
    },
    LayerRow {
        id: "drs.object.schema.openapi",
        layer: CheckLayer::Schema,
        request: Some("GET {drs}/objects/{object_id}"),
    },
    LayerRow {
        id: "drs.object.checksum",
        layer: CheckLayer::Behavior,
        request: Some(
            "GET {drs}/objects/{object_id} then GET access_methods[0].access_url.url (no Range)",
        ),
    },
    LayerRow {
        id: "drs.object.range",
        layer: CheckLayer::Behavior,
        request: Some("GET access_url with Header Range: bytes=0-1023"),
    },
    LayerRow {
        id: "drs.object.not_found",
        layer: CheckLayer::Behavior,
        request: Some("GET {drs}/objects/{derived helix.unknown.* id}"),
    },
    LayerRow {
        id: "wes.service_info.reachable",
        layer: CheckLayer::Interoperability,
        request: Some("GET {wes}/service-info"),
    },
    LayerRow {
        id: "wes.service_info.schema",
        layer: CheckLayer::Schema,
        request: Some("GET {wes}/service-info"),
    },
    LayerRow {
        id: "wes.run.lifecycle_success",
        layer: CheckLayer::Behavior,
        request: Some(
            "POST {wes}/runs (HelixTest echo TRS URL) then poll GET {wes}/runs/{id}/status",
        ),
    },
    LayerRow {
        id: "wes.run.failure_state",
        layer: CheckLayer::Behavior,
        request: Some("POST {wes}/runs (HelixTest fail fixture) then poll status"),
    },
    LayerRow {
        id: "wes.run.missing_inputs",
        layer: CheckLayer::Behavior,
        request: Some("POST {wes}/runs (HelixTest missing-input fixture) then poll status"),
    },
    LayerRow {
        id: "wes.run.incompatible_type",
        layer: CheckLayer::Behavior,
        request: Some("POST {wes}/runs (incompatible workflow_type) then poll status"),
    },
    LayerRow {
        id: "wes.run.invalid_workflow",
        layer: CheckLayer::Behavior,
        request: Some("POST {wes}/runs (invalid workflow URL) then poll status"),
    },
    LayerRow {
        id: "wes.run.scatter_gather",
        layer: CheckLayer::Behavior,
        request: Some("POST {wes}/runs (scatter/gather fixture) then poll status"),
    },
    LayerRow {
        id: "tes.tasks.reachable",
        layer: CheckLayer::Interoperability,
        request: Some("GET {tes}/tasks (not executed by helix verify)"),
    },
    LayerRow {
        id: "tes.task.schema",
        layer: CheckLayer::Schema,
        request: Some("TES task JSON (not executed by helix verify)"),
    },
    LayerRow {
        id: "tes.task.lifecycle_checksum",
        layer: CheckLayer::Behavior,
        request: Some("TES lifecycle + checksum fixture (not executed by helix verify)"),
    },
    LayerRow {
        id: "trs.tools.reachable",
        layer: CheckLayer::Interoperability,
        request: Some("GET {trs}/tools (not executed by helix verify)"),
    },
    LayerRow {
        id: "trs.tools.schema",
        layer: CheckLayer::Schema,
        request: Some("TRS tools JSON (not executed by helix verify)"),
    },
    LayerRow {
        id: "trs.descriptor.retrieve",
        layer: CheckLayer::Behavior,
        request: Some("TRS descriptor retrieval (not executed by helix verify)"),
    },
    LayerRow {
        id: "beacon.query.reachable",
        layer: CheckLayer::Interoperability,
        request: Some("Beacon /query (not executed by helix verify)"),
    },
    LayerRow {
        id: "beacon.query.boolean_schema",
        layer: CheckLayer::Schema,
        request: Some("Beacon boolean response JSON (not executed by helix verify)"),
    },
    LayerRow {
        id: "beacon.variant.known_exists",
        layer: CheckLayer::Behavior,
        request: Some("Beacon known-variant fixture (not executed by helix verify)"),
    },
    LayerRow {
        id: "beacon.variant.negative_absent",
        layer: CheckLayer::Behavior,
        request: Some("Beacon negative-variant fixture (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.service_info",
        layer: CheckLayer::Interoperability,
        request: Some("htsget reads service-info (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.variants.service_info",
        layer: CheckLayer::Interoperability,
        request: Some("htsget variants service-info (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.ticket.get",
        layer: CheckLayer::Behavior,
        request: Some("GET htsget reads ticket (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.variants.ticket.get",
        layer: CheckLayer::Behavior,
        request: Some("GET htsget variants ticket (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.variants.wrong_object",
        layer: CheckLayer::Behavior,
        request: Some("htsget wrong-object (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.ticket.post",
        layer: CheckLayer::Behavior,
        request: Some("POST htsget reads ticket (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.ticket.post_regions",
        layer: CheckLayer::Behavior,
        request: Some("POST htsget reads regions (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.variants.ticket.post",
        layer: CheckLayer::Behavior,
        request: Some("POST htsget variants ticket (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.variants.ticket.post_regions",
        layer: CheckLayer::Behavior,
        request: Some("POST htsget variants regions (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.post_query_invalid",
        layer: CheckLayer::Behavior,
        request: Some("POST htsget reads with query params (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.format_cram",
        layer: CheckLayer::Behavior,
        request: Some("htsget CRAM-on-BAM (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.reads.class_header",
        layer: CheckLayer::Behavior,
        request: Some("htsget class=header (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.dataset.auth",
        layer: CheckLayer::Security,
        request: Some("htsget dataset auth (not executed by helix verify)"),
    },
    LayerRow {
        id: "htsget.suite.unresolved",
        layer: CheckLayer::Behavior,
        request: Some("htsget unresolved suite skip (not executed by helix verify)"),
    },
    LayerRow {
        id: "auth.service_info.reachable",
        layer: CheckLayer::Security,
        request: Some("GET auth /service-info (HelixTest wrap; not helix verify)"),
    },
    LayerRow {
        id: "auth.token.valid",
        layer: CheckLayer::Security,
        request: Some("DRS GET with valid dummy HMAC Bearer"),
    },
    LayerRow {
        id: "auth.token.expired",
        layer: CheckLayer::Security,
        request: Some("DRS GET with expired dummy HMAC Bearer"),
    },
    LayerRow {
        id: "auth.token.garbage",
        layer: CheckLayer::Security,
        request: Some("DRS GET with garbage Bearer"),
    },
    LayerRow {
        id: "auth.token.wrong_scope",
        layer: CheckLayer::Security,
        request: Some("DRS GET with wrong-scope dummy HMAC Bearer"),
    },
    LayerRow {
        id: "auth.token.missing",
        layer: CheckLayer::Security,
        request: Some("DRS GET with no Authorization"),
    },
    LayerRow {
        id: "auth.helix.token.valid",
        layer: CheckLayer::Security,
        request: Some("Helix security profile: valid dummy HMAC"),
    },
    LayerRow {
        id: "auth.helix.token.expired",
        layer: CheckLayer::Security,
        request: Some("Helix security profile: expired dummy token"),
    },
    LayerRow {
        id: "auth.helix.token.wrong_scope",
        layer: CheckLayer::Security,
        request: Some("Helix security profile: wrong-scope dummy token"),
    },
    LayerRow {
        id: "auth.helix.token.manipulated",
        layer: CheckLayer::Security,
        request: Some("Helix security profile: manipulated dummy token"),
    },
    LayerRow {
        id: "auth.helix.token.wrong_audience",
        layer: CheckLayer::Security,
        request: Some("Helix security profile: wrong-audience dummy token"),
    },
    LayerRow {
        id: "auth.helix.crypt4gh.header",
        layer: CheckLayer::Security,
        request: Some("Crypt4GH header layout (file or bytes)"),
    },
    LayerRow {
        id: "auth.helix.crypt4gh.invalid_rejected",
        layer: CheckLayer::Security,
        request: Some("Invalid Crypt4GH envelope bytes"),
    },
    LayerRow {
        id: "auth.helix.crypt4gh.http_envelope",
        layer: CheckLayer::Security,
        request: Some("HTTP body Crypt4GH magic (or skip)"),
    },
    LayerRow {
        id: "discovery.drs",
        layer: CheckLayer::Interoperability,
        request: Some("Helix discovery probe for DRS"),
    },
    LayerRow {
        id: "discovery.wes",
        layer: CheckLayer::Interoperability,
        request: Some("Helix discovery probe for WES"),
    },
    LayerRow {
        id: "discovery.tes",
        layer: CheckLayer::Interoperability,
        request: Some("Helix discovery probe for TES"),
    },
    LayerRow {
        id: "discovery.trs",
        layer: CheckLayer::Interoperability,
        request: Some("Helix discovery probe for TRS"),
    },
    LayerRow {
        id: "discovery.htsget",
        layer: CheckLayer::Interoperability,
        request: Some("Helix discovery probe for htsget"),
    },
    LayerRow {
        id: "bench.get.health",
        layer: CheckLayer::Benchmark,
        request: Some("GET /health (timed)"),
    },
    LayerRow {
        id: "bench.get.drs_service_info",
        layer: CheckLayer::Benchmark,
        request: Some("GET /ga4gh/drs/v1/service-info (timed)"),
    },
    LayerRow {
        id: "bench.get.drs_object",
        layer: CheckLayer::Benchmark,
        request: Some("GET /ga4gh/drs/v1/objects/{object_id} (timed)"),
    },
    LayerRow {
        id: "bench.metric.wall_ms",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.rss_kb",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.error_rate",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.p95_ms",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.min_ms",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.max_ms",
        layer: CheckLayer::Benchmark,
        request: None,
    },
    LayerRow {
        id: "bench.metric.bytes",
        layer: CheckLayer::Benchmark,
        request: None,
    },
];

fn row(id: &str) -> Option<&'static LayerRow> {
    LAYERS.iter().find(|r| r.id == id)
}

pub fn for_id(id: &str) -> CheckLayer {
    row(id)
        .map(|r| r.layer)
        .unwrap_or(CheckLayer::Interoperability)
}

pub fn request_for_id(id: &str) -> Option<&'static str> {
    row(id).and_then(|r| r.request)
}

pub fn validate_table() -> anyhow::Result<()> {
    if LAYERS.len() != SPECS.len() {
        anyhow::bail!(
            "layer table has {} rows, identity SPECS has {}",
            LAYERS.len(),
            SPECS.len()
        );
    }
    for spec in SPECS {
        let row =
            row(spec.id).ok_or_else(|| anyhow::anyhow!("check {} has no layer row", spec.id))?;
        if spec_by_id(row.id).is_none() {
            anyhow::bail!("layer row {} is not in SPECS", row.id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_covers_every_spec() {
        validate_table().expect("layer table");
    }

    #[test]
    fn schema_layer_is_not_behavior() {
        assert_eq!(for_id("drs.object.schema"), CheckLayer::Schema);
        assert_eq!(for_id("drs.object.checksum"), CheckLayer::Behavior);
        assert_eq!(for_id("drs.object.not_found"), CheckLayer::Behavior);
        assert_eq!(for_id("drs.object.range"), CheckLayer::Behavior);
        assert_eq!(for_id("wes.service_info.schema"), CheckLayer::Schema);
        assert_eq!(for_id("wes.run.lifecycle_success"), CheckLayer::Behavior);
    }

    #[test]
    fn schema_pass_does_not_make_behavior_pass() {
        let mut s = LayerSummary::default();
        s.record(CheckLayer::Schema, LayerOutcome::Pass);
        s.record(CheckLayer::Behavior, LayerOutcome::Fail);
        assert_eq!(s.schema.verdict(), LayerVerdict::Pass);
        assert_eq!(s.behavior.verdict(), LayerVerdict::Fail);
        assert_eq!(s.schema.passed, 1);
        assert_eq!(s.behavior.failed, 1);
        let json = serde_json::to_value(&s).unwrap();
        assert!(json.get("percent").is_none());
        assert!(json.get("compliant").is_none());
        assert!(json.get("score").is_none());
        assert!(s.note.contains("SCHEMA PASS is not BEHAVIOR PASS"));
    }

    #[test]
    fn empty_layer_is_none_not_pass() {
        let s = LayerSummary::default();
        assert_eq!(s.security.verdict(), LayerVerdict::None);
        assert_eq!(s.schema.verdict(), LayerVerdict::None);
    }
}
