// SPDX-License-Identifier: Apache-2.0
//! Per-check provenance: kind, authority, and (only when justified) a GA4GH locator.
//!
//! HelixTest checks are **not** labeled normative. They run HelixTest-vendored
//! OpenAPI plus fixture extras, not `standards/vendor` bytes. A related locator
//! in an AVAILABLE pin is an audit hint, not a MUST mapping.
//!
//! Not HELIOS. Not GA4GH certification.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::identity::{spec_by_id, CheckSpec, SPECS};
use crate::model::VerificationResult;
use crate::standards::{
    default_registry_path, load_path, BindingKind, ClaimScope, LocatorType, Registry,
    StandardVersion,
};

/// Who asserted the behaviour this check encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    Ga4gh,
    Helix,
    Helixtest,
}

impl Authority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ga4gh => "ga4gh",
            Self::Helix => "helix",
            Self::Helixtest => "helixtest",
        }
    }
}

/// Related structure in an AVAILABLE pin. Not a claim Helix executed that pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedSource {
    pub registry_entry: String,
    pub standard: String,
    pub version: String,
    pub release_class: String,
    pub source_repository: String,
    pub source_commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    pub source_file: String,
    pub locator_type: String,
    pub locator: String,
    /// Why this locator is not a complete normative binding.
    pub limitation: String,
}

/// Machine-readable provenance stamped on every `VerificationResult`.
///
/// `category` is the claim taxonomy ([`BindingKind`]). It is **not** the
/// domain field `VerificationResult.category` (schema, lifecycle, …).
/// `check_kind` must equal `category` (compat name). `claim_scope` must
/// equal `category.claim_scope()`.
///
/// Pack identity fields (`version`, `source_commit`, …) are filled **only**
/// when `category` is `normative` and a pack actually executed. Related
/// AVAILABLE pins live under `related_source` and must not be quoted as
/// verified-against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckTraceability {
    pub check_id: String,
    /// Claim taxonomy. Same value as `check_kind`.
    pub category: BindingKind,
    pub check_kind: BindingKind,
    pub claim_scope: ClaimScope,
    pub authority: Authority,
    pub expected_behavior: String,
    pub implementation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub untraceable_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_entry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_source: Option<RelatedSource>,
    pub layer: crate::layer::CheckLayer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
}

impl CheckTraceability {
    #[cfg(test)]
    fn apply_taxonomy(&mut self, kind: BindingKind) {
        self.category = kind;
        self.check_kind = kind;
        self.claim_scope = kind.claim_scope();
    }
}

struct CatalogRow {
    id: &'static str,
    kind: BindingKind,
    authority: Authority,
    expected: &'static str,
    implementation: &'static str,
    untraceable: Option<&'static str>,
    related_pack: Option<&'static str>,
    locator_type: Option<LocatorType>,
    locator: Option<&'static str>,
    limitation: Option<&'static str>,
}

const HT_UNBOUND: &str = "On the unversioned helix verify path this check uses HelixTest-vendored OpenAPI and Helix/HelixTest fixtures, not Helix standards/vendor bytes. It is not a GA4GH MUST. Versioned DRS 1.4.0 execution uses SpecSource only for drs.object.schema.openapi.";

const DRS_PATH_LIMIT: &str = "Fixture probe related to GET /objects/{object_id}. The DRS 1.4.0 vendor pack is complete locally; this locator is the path key only. Schema MUST coverage is the Normative check drs.object.schema.openapi, not this fixture.";

const DRS_OPENAPI_LIMIT: &str = "Covers GET /objects/{object_id} 200 JSON against the pinned DRS 1.4.0 DrsObject schema via SpecSource. Does not cover bulk objects, /access, passports, OPTIONS authorizations, service-info, bundles/contents, or optional DrsObject properties as MUST. SCHEMA PASS is not BEHAVIOR coverage.";

const WES_SI_LIMIT: &str = "operationId GetServiceInfo exists in the pinned WES 1.1.0 vendor file. HelixTest does not load those bytes. Reachability is not the same as implementing the operation. Not a MUST mapping.";

const WES_SCHEMA_LIMIT: &str = "components.schemas.ServiceInfo exists in the pinned WES 1.1.0 file. HelixTest validates a different vendored copy and additionally requires supported_wes_versions to contain 1.0 or 1.1 (HelixTest policy, not a locator). Not a MUST mapping.";

const WES_RUN_LIMIT: &str = "operationId RunWorkflow exists in the pinned WES 1.1.0 vendor file. This check submits HelixTest fixture workflow URLs (trs://test-tool/…) and extra lifecycle assertions. Not a MUST mapping.";

const NO_PACK: &str = "Helix has no registry row for this standard. helix verify does not execute this suite. No GA4GH commit is bound.";

const CATALOG: &[CatalogRow] = &[
    // DRS verify (HelixTest wrap)
    CatalogRow {
        id: "drs.object.reachable",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "GET objects/test-object-1 returns an HTTP success from the HelixTest reachable probe",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level0_reachable",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.drs.1.4.0"),
        locator_type: Some(LocatorType::HttpPath),
        locator: Some("/objects/{object_id}"),
        limitation: Some(DRS_PATH_LIMIT),
    },
    CatalogRow {
        id: "drs.object.schema",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "JSON body validates against the HelixTest-vendored DrsObject schema and HelixTest extras (id=test-object-1, self_uri, name, non-empty access_methods)",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level1_basic_schema_and_fields",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.drs.1.4.0"),
        locator_type: Some(LocatorType::HttpPath),
        locator: Some("/objects/{object_id}"),
        limitation: Some(DRS_PATH_LIMIT),
    },
    CatalogRow {
        id: "drs.object.schema.openapi",
        kind: BindingKind::Normative,
        authority: Authority::Ga4gh,
        expected: "GET /objects/{object_id} 200 JSON validates against the pinned DRS 1.4.0 DrsObject schema (SpecSource; no HelixTest extras)",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level1_openapi_specsource via run_drs_checks_with_spec",
        untraceable: None,
        related_pack: Some("ga4gh.drs.1.4.0"),
        locator_type: Some(LocatorType::SchemaName),
        locator: Some("DrsObject"),
        limitation: Some(DRS_OPENAPI_LIMIT),
    },
    CatalogRow {
        id: "drs.object.checksum",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "sha256 in DrsObject.checksums matches a download of access_methods[0].access_url.url for test-object-1",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level2_checksum_correctness",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.drs.1.4.0"),
        locator_type: Some(LocatorType::HttpPath),
        locator: Some("/objects/{object_id}"),
        limitation: Some(DRS_PATH_LIMIT),
    },
    CatalogRow {
        id: "drs.object.range",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HTTP 206 Partial Content with a valid Content-Range for Range: bytes=0-1023 on the fixture object bytes URL",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level2_range_request",
        untraceable: Some(
            "HTTP Range is a HelixTest probe. The pinned DRS 1.4.0 entry OpenAPI does not contain a Range requirement Helix can cite (path $refs are unvendored). Not a GA4GH MUST in Helix.",
        ),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "drs.object.not_found",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HTTP 404 for the Helix unknown object id fixture",
        implementation: "HelixTest helixtest/crates/framework/src/drs.rs level5_invalid_id_404",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.drs.1.4.0"),
        locator_type: Some(LocatorType::HttpPath),
        locator: Some("/objects/{object_id}"),
        limitation: Some(DRS_PATH_LIMIT),
    },
    // WES verify
    CatalogRow {
        id: "wes.service_info.reachable",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "GET service-info returns an HTTP success from the HelixTest reachable probe",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level0_service_info_reachable",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("GetServiceInfo"),
        limitation: Some(WES_SI_LIMIT),
    },
    CatalogRow {
        id: "wes.service_info.schema",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "JSON body validates against the HelixTest-vendored ServiceInfo schema and HelixTest policy that supported_wes_versions contains 1.0 or 1.1",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level1_service_info_schema",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::SchemaName),
        locator: Some("ServiceInfo"),
        limitation: Some(WES_SCHEMA_LIMIT),
    },
    CatalogRow {
        id: "wes.run.lifecycle_success",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest echo fixture (trs://test-tool/echo/1.0) reaches COMPLETE with echo_out and a pre-terminal state",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level2_lifecycle_success",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    CatalogRow {
        id: "wes.run.failure_state",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest fail fixture reaches EXECUTOR_ERROR or SYSTEM_ERROR",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level2_failure_state",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    CatalogRow {
        id: "wes.run.missing_inputs",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest missing-input fixture reaches EXECUTOR_ERROR or SYSTEM_ERROR",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level2_missing_inputs_error_state",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    CatalogRow {
        id: "wes.run.incompatible_type",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest incompatible workflow_type fixture reaches an error state",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level2_incompatible_type_error_state",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    CatalogRow {
        id: "wes.run.invalid_workflow",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest invalid workflow fixture reaches an error state",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level3_invalid_workflow",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    CatalogRow {
        id: "wes.run.scatter_gather",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "HelixTest scatter/gather fixture completes, or skip when supports_scatter_gather=false (skip is never pass)",
        implementation: "HelixTest helixtest/crates/framework/src/wes.rs level2_scatter_gather",
        untraceable: Some(HT_UNBOUND),
        related_pack: Some("ga4gh.wes.1.1.0"),
        locator_type: Some(LocatorType::OperationId),
        locator: Some("RunWorkflow"),
        limitation: Some(WES_RUN_LIMIT),
    },
    // TES / TRS / Beacon / htsget — catalogued, not executed by helix verify
    CatalogRow {
        id: "tes.tasks.reachable",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "TES /tasks answers (HelixTest wrap; not executed by helix verify)",
        implementation: "HelixTest TES wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "tes.task.schema",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "TES task create/status match HelixTest-vendored schema (not executed by helix verify)",
        implementation: "HelixTest TES wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "tes.task.lifecycle_checksum",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "TES lifecycle + checksum fixture (not executed by helix verify)",
        implementation: "HelixTest TES wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "trs.tools.reachable",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "TRS /tools answers (not executed by helix verify)",
        implementation: "HelixTest TRS wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "trs.tools.schema",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "TRS tools/versions match HelixTest-vendored schema (not executed by helix verify)",
        implementation: "HelixTest TRS wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "trs.descriptor.retrieve",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "TRS descriptor retrieval fixture (not executed by helix verify)",
        implementation: "HelixTest TRS wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "beacon.query.reachable",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "Beacon /query answers (not executed by helix verify)",
        implementation: "HelixTest Beacon wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "beacon.query.boolean_schema",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "Beacon boolean response matches HelixTest-vendored schema (not executed by helix verify)",
        implementation: "HelixTest Beacon wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "beacon.variant.known_exists",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "Beacon known-variant fixture (not executed by helix verify)",
        implementation: "HelixTest Beacon wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "beacon.variant.negative_absent",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "Beacon negative-variant fixture (not executed by helix verify)",
        implementation: "HelixTest Beacon wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.service_info",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "htsget reads service-info (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.variants.service_info",
        kind: BindingKind::Interoperability,
        authority: Authority::Helixtest,
        expected: "htsget variants service-info (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.ticket.get",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget GET reads ticket fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.variants.ticket.get",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget GET variants ticket fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.variants.wrong_object",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget wrong-object NotFound fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.ticket.post",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget POST reads ticket fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.ticket.post_regions",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget POST reads regions fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.variants.ticket.post",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget POST variants ticket fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.variants.ticket.post_regions",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget POST variants regions fixture (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.post_query_invalid",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget POST reads with query params is InvalidInput (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.format_cram",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget CRAM-on-BAM is UnsupportedFormat (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.reads.class_header",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget class=header is InvalidInput (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.dataset.auth",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "htsget dataset auth fail-closed then allows a valid token (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "htsget.suite.unresolved",
        kind: BindingKind::Fixture,
        authority: Authority::Helixtest,
        expected: "htsget suite skipped when the service URL is unresolved (not executed by helix verify)",
        implementation: "HelixTest htsget wrap (not called from src/verify.rs)",
        untraceable: Some(NO_PACK),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    // Auth HelixTest HMAC wrap
    CatalogRow {
        id: "auth.service_info.reachable",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Auth /service-info answers (HelixTest HMAC wrap; not helix verify)",
        implementation: "HelixTest auth wrap",
        untraceable: Some("Helix-owned security behaviour uses HLX-AUTH-010–014 / 050–054. This HelixTest wrap is not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.token.valid",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Valid HMAC fixture token grants DRS access",
        implementation: "HelixTest auth wrap",
        untraceable: Some("HMAC JWT fixture behaviour. Not a GA4GH MUST. Dummy secret only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.token.expired",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Expired HMAC fixture token is rejected",
        implementation: "HelixTest auth wrap",
        untraceable: Some("HMAC JWT fixture behaviour. Not a GA4GH MUST. Dummy secret only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.token.garbage",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Garbage bearer is rejected",
        implementation: "HelixTest auth wrap",
        untraceable: Some("HMAC JWT fixture behaviour. Not a GA4GH MUST. Dummy secret only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.token.wrong_scope",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Wrong-scope HMAC fixture token is denied",
        implementation: "HelixTest auth wrap",
        untraceable: Some("HMAC JWT fixture behaviour. Not a GA4GH MUST. Dummy secret only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.token.missing",
        kind: BindingKind::Security,
        authority: Authority::Helixtest,
        expected: "Missing token returns 401",
        implementation: "HelixTest auth wrap",
        untraceable: Some("HMAC JWT fixture behaviour. Not a GA4GH MUST. Dummy secret only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    // Helix-native security
    CatalogRow {
        id: "auth.helix.token.valid",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Valid dummy HMAC token is accepted by the Helix security profile (not a proof the target is secure)",
        implementation: "src/security/mod.rs AUTH_CASE_NAMES / src/security/http_cases.rs",
        untraceable: Some("Helix security-behaviour profile. Not a GA4GH specification requirement. Dummy fixtures only."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.token.expired",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Expired dummy token is rejected with 401",
        implementation: "src/security/mod.rs / src/security/http_cases.rs",
        untraceable: Some("Helix security-behaviour profile. Not a GA4GH specification requirement."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.token.wrong_scope",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Wrong-scope dummy token is denied",
        implementation: "src/security/mod.rs / src/security/http_cases.rs",
        untraceable: Some("Helix security-behaviour profile. Not a GA4GH specification requirement."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.token.manipulated",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Manipulated dummy token is rejected",
        implementation: "src/security/mod.rs / src/security/http_cases.rs",
        untraceable: Some("Helix security-behaviour profile. Not a GA4GH specification requirement."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.token.wrong_audience",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Token for another service is rejected",
        implementation: "src/security/mod.rs / src/security/http_cases.rs",
        untraceable: Some("Helix security-behaviour profile. Not a GA4GH specification requirement."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.crypt4gh.header",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Crypt4GH header layout is well-formed (magic/version/packet count). Not encryption correctness.",
        implementation: "src/security/crypt4gh_header.rs validate_crypt4gh_header",
        untraceable: Some("Helix protocol-layout check. Crypt4GH is not a GA4GH DRS/WES MUST in the Helix registry. A pass is not 'secure'."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.crypt4gh.invalid_rejected",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "Invalid Crypt4GH envelope is rejected (layout only)",
        implementation: "src/security/crypt4gh_header.rs",
        untraceable: Some("Helix protocol-layout check. Not a GA4GH DRS/WES MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "auth.helix.crypt4gh.http_envelope",
        kind: BindingKind::Security,
        authority: Authority::Helix,
        expected: "HTTP body is a Crypt4GH envelope when magic is present, or skip",
        implementation: "src/security/crypt4gh_header.rs",
        untraceable: Some("Helix protocol-layout check. Not a GA4GH DRS/WES MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    // Discovery
    CatalogRow {
        id: "discovery.drs",
        kind: BindingKind::Interoperability,
        authority: Authority::Helix,
        expected: "Helix probe records DETECTED or NOT_DETECTED for DRS. DETECTED is not a pass.",
        implementation: "src/discover.rs",
        untraceable: Some("Helix discovery probe. Endpoint existence is not a GA4GH requirement identifier."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "discovery.wes",
        kind: BindingKind::Interoperability,
        authority: Authority::Helix,
        expected: "Helix probe records DETECTED or NOT_DETECTED for WES. DETECTED is not a pass.",
        implementation: "src/discover.rs",
        untraceable: Some("Helix discovery probe. Endpoint existence is not a GA4GH requirement identifier."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "discovery.tes",
        kind: BindingKind::Interoperability,
        authority: Authority::Helix,
        expected: "Helix probe records DETECTED or NOT_DETECTED for TES. DETECTED is not a pass.",
        implementation: "src/discover.rs",
        untraceable: Some("Helix discovery probe. Endpoint existence is not a GA4GH requirement identifier."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "discovery.trs",
        kind: BindingKind::Interoperability,
        authority: Authority::Helix,
        expected: "Helix probe records DETECTED or NOT_DETECTED for TRS. DETECTED is not a pass.",
        implementation: "src/discover.rs",
        untraceable: Some("Helix discovery probe. Endpoint existence is not a GA4GH requirement identifier."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "discovery.htsget",
        kind: BindingKind::Interoperability,
        authority: Authority::Helix,
        expected: "Helix probe records DETECTED or NOT_DETECTED for htsget. DETECTED is not a pass.",
        implementation: "src/discover.rs",
        untraceable: Some("Helix discovery probe. Endpoint existence is not a GA4GH requirement identifier."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    // Bench
    CatalogRow {
        id: "bench.get.health",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "GET /health is timed as part of http.drs.smoke.v1. Not a verification assertion.",
        implementation: "src/bench/workload.rs",
        untraceable: Some("Helix smoke measurement. Not a GA4GH MUST. Thresholds do not fail CI."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.get.drs_service_info",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "GET /ga4gh/drs/v1/service-info is timed. Not a verification assertion.",
        implementation: "src/bench/workload.rs",
        untraceable: Some("Helix smoke measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.get.drs_object",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "GET /ga4gh/drs/v1/objects/test-object-1 is timed. Fixture object id.",
        implementation: "src/bench/workload.rs",
        untraceable: Some("Helix smoke measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.wall_ms",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Client wall-time median of measured runs (warning, not a verification fail)",
        implementation: "src/bench/analysis.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.rss_kb",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Optional Linux RSS sample",
        implementation: "src/bench/rss.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.error_rate",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Request error rate of measured runs",
        implementation: "src/bench/analysis.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.p95_ms",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Sample p95 wall time when enough repetitions exist",
        implementation: "src/bench/stats.rs",
        untraceable: Some("Helix measurement. Not a significance test. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.min_ms",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Minimum measured wall time",
        implementation: "src/bench/stats.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.max_ms",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Maximum measured wall time",
        implementation: "src/bench/stats.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
    CatalogRow {
        id: "bench.metric.bytes",
        kind: BindingKind::Benchmark,
        authority: Authority::Helix,
        expected: "Response body bytes of measured runs",
        implementation: "src/bench/engine.rs",
        untraceable: Some("Helix measurement. Not a GA4GH MUST."),
        related_pack: None,
        locator_type: None,
        locator: None,
        limitation: None,
    },
];

fn row(id: &str) -> Option<&'static CatalogRow> {
    CATALOG.iter().find(|r| r.id == id)
}

fn load_default_registry() -> Option<Registry> {
    load_path(&default_registry_path()).ok()
}

fn pack_by_id<'a>(reg: &'a Registry, pack_id: &str) -> Option<&'a StandardVersion> {
    reg.versions.iter().find(|v| v.pack_id == pack_id)
}

fn locator_in_vendor<'a>(
    pack: &'a StandardVersion,
    locator: &str,
    locator_type: LocatorType,
) -> Result<&'a crate::standards::VersionSource> {
    if pack.normative_sources.is_empty() {
        bail!(
            "pack {} has no normative_sources; cannot cover locator {locator}",
            pack.pack_id
        );
    }
    let root = default_registry_path()
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_registry_path);
    let mut hits = Vec::new();
    for src in &pack.normative_sources {
        let Some(vendor) = src.vendor_path.as_ref() else {
            continue;
        };
        let path = root.join(vendor);
        let bytes = std::fs::read(&path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        if text.contains(locator) {
            hits.push(src);
        }
    }
    if hits.is_empty() {
        bail!(
            "locator {locator:?} not found in any vendored file for pack {} (related source is not covered by pinned bytes)",
            pack.pack_id
        );
    }
    if locator_type == LocatorType::SchemaName {
        let suffix = format!("/schemas/{locator}.yaml");
        if let Some(hit) = hits.iter().find(|s| s.path.ends_with(&suffix)) {
            return Ok(*hit);
        }
        let exact = format!("{locator}.yaml");
        if let Some(hit) = hits
            .iter()
            .find(|s| s.path.rsplit('/').next() == Some(exact.as_str()))
        {
            return Ok(*hit);
        }
    }
    if let Some(entry) = hits.iter().find(|s| {
        s.path.ends_with("data_repository_service.openapi.yaml")
            || s.path.ends_with("workflow_execution_service.openapi.yaml")
    }) {
        return Ok(*entry);
    }
    Ok(hits[0])
}

fn related_from_row(reg: &Registry, row: &CatalogRow) -> Result<Option<RelatedSource>> {
    let Some(pack_id) = row.related_pack else {
        return Ok(None);
    };
    let Some(locator) = row.locator else {
        bail!(
            "{} related_pack {pack_id} needs a locator (do not invent requirement ids)",
            row.id
        );
    };
    let locator_type = row
        .locator_type
        .ok_or_else(|| anyhow::anyhow!("{} related locator needs locator_type", row.id))?;
    let pack = pack_by_id(reg, pack_id).ok_or_else(|| {
        anyhow::anyhow!("{} related_pack {pack_id} is not in the registry", row.id)
    })?;
    let src = locator_in_vendor(pack, locator, locator_type)?;
    Ok(Some(RelatedSource {
        registry_entry: pack.pack_id.clone(),
        standard: pack.standard.clone(),
        version: pack.version.clone(),
        release_class: pack.release_class.as_str().to_string(),
        source_repository: pack.repository.clone(),
        source_commit: pack.commit.clone(),
        source_sha256: Some(src.integrity.hex.clone()),
        source_file: src.path.clone(),
        locator_type: match locator_type {
            LocatorType::OperationId => "operation_id".into(),
            LocatorType::SchemaName => "schema_name".into(),
            LocatorType::JsonPointer => "json_pointer".into(),
            LocatorType::HttpPath => "http_path".into(),
            LocatorType::StatusCode => "status_code".into(),
            LocatorType::Quote => "quote".into(),
        },
        locator: locator.to_string(),
        limitation: row
            .limitation
            .unwrap_or("Related locator only; not a normative binding.")
            .to_string(),
    }))
}

fn from_row(row: &CatalogRow, related: Option<RelatedSource>) -> CheckTraceability {
    CheckTraceability {
        check_id: row.id.to_string(),
        category: row.kind,
        check_kind: row.kind,
        claim_scope: row.kind.claim_scope(),
        authority: row.authority,
        expected_behavior: row.expected.to_string(),
        implementation: row.implementation.to_string(),
        untraceable_reason: row.untraceable.map(str::to_string),
        standard: spec_by_id(row.id).map(|s| s.service.to_string()),
        version: None,
        release_class: None,
        registry_entry: None,
        source_repository: None,
        source_commit: None,
        source_sha256: None,
        source_file: None,
        source_location: None,
        related_source: related,
        layer: crate::layer::for_id(row.id),
        request: crate::layer::request_for_id(row.id).map(str::to_string),
    }
}

fn unmapped(id: &str) -> CheckTraceability {
    CheckTraceability {
        check_id: id.to_string(),
        category: BindingKind::Fixture,
        check_kind: BindingKind::Fixture,
        claim_scope: ClaimScope::HelixFixture,
        authority: Authority::Helixtest,
        expected_behavior: "Unknown HelixTest name; not in the Helix catalog.".into(),
        implementation: "src/adapter/translate.rs".into(),
        untraceable_reason: Some(
            "Unmapped HelixTest name. Not a GA4GH requirement. Not a Helix catalog id.".into(),
        ),
        standard: None,
        version: None,
        release_class: None,
        registry_entry: None,
        source_repository: None,
        source_commit: None,
        source_sha256: None,
        source_file: None,
        source_location: None,
        related_source: None,
        layer: crate::layer::CheckLayer::Interoperability,
        request: None,
    }
}

/// Provenance for a catalog id (or an unmapped adapter row).
pub fn for_id(id: &str) -> CheckTraceability {
    let Some(row) = row(id) else {
        return unmapped(id);
    };
    let related = load_default_registry()
        .and_then(|reg| related_from_row(&reg, row).ok())
        .flatten();
    from_row(row, related)
}

pub fn for_spec(spec: &CheckSpec) -> CheckTraceability {
    for_id(spec.id)
}

/// Fail-closed catalog + result invariants. Does not download GA4GH.
pub fn validate_catalog(reg: &Registry) -> Result<()> {
    if CATALOG.len() != SPECS.len() {
        bail!(
            "traceability catalog has {} rows, identity SPECS has {}",
            CATALOG.len(),
            SPECS.len()
        );
    }
    for spec in SPECS {
        let row = row(spec.id)
            .ok_or_else(|| anyhow::anyhow!("check {} has no traceability catalog row", spec.id))?;
        validate_row(row, reg)?;
    }
    crate::layer::validate_table()?;
    Ok(())
}

fn validate_row(row: &CatalogRow, reg: &Registry) -> Result<()> {
    if row.kind == BindingKind::Normative {
        if row.untraceable.is_some() {
            bail!(
                "{} is labeled normative but has untraceable_reason (do not fabricate provenance)",
                row.id
            );
        }
        if row.authority != Authority::Ga4gh {
            bail!("{} normative checks must have authority=ga4gh", row.id);
        }
        if row.related_pack.is_none() || row.locator.is_none() {
            bail!("{} normative check has no provenance locator", row.id);
        }
        let related = related_from_row(reg, row)?
            .ok_or_else(|| anyhow::anyhow!("{} normative related_source missing", row.id))?;
        if related.source_commit.is_empty() {
            bail!("{} normative source commit is missing", row.id);
        }
        let pack = pack_by_id(reg, &related.registry_entry).unwrap();
        if pack.version != related.version {
            bail!(
                "{} registry version {} does not match check metadata {}",
                row.id,
                pack.version,
                related.version
            );
        }
        match pack.release_class {
            crate::standards::ReleaseClass::Official
            | crate::standards::ReleaseClass::Ballot
            | crate::standards::ReleaseClass::Snapshot
            | crate::standards::ReleaseClass::Development => {}
        }
    } else if row.kind.may_claim_ga4gh_requirement() {
        bail!("{} kind must not claim a GA4GH requirement", row.id);
    }
    if row.kind == BindingKind::Guidance {
        bail!(
            "{} is labeled guidance but Helix has no official GA4GH implementation-guidance pin; HelixTest policy is fixture, not guidance",
            row.id
        );
    }
    if row.related_pack.is_some() {
        related_from_row(reg, row)?;
    }
    Ok(())
}

/// Result-level invariants after version stamps.
pub fn validate_result(r: &VerificationResult) -> Result<()> {
    let Some(t) = &r.traceability else {
        bail!("check {} has no provenance (traceability missing)", r.id);
    };
    if t.check_id != r.id && r.id != "helixtest.unmapped" {
        bail!(
            "traceability.check_id {} does not match result id {}",
            t.check_id,
            r.id
        );
    }
    if t.category == BindingKind::Fixture
        && (t.check_kind == BindingKind::Normative || t.claim_scope == ClaimScope::Ga4ghRequirement)
    {
        bail!(
            "fixture check {} cannot be serialized as a normative requirement",
            r.id
        );
    }
    if t.category != t.check_kind {
        bail!(
            "check {}: taxonomy category {} does not match check_kind {}",
            r.id,
            t.category.as_str(),
            t.check_kind.as_str()
        );
    }
    if t.claim_scope != t.category.claim_scope() {
        bail!(
            "check {}: claim_scope {} does not match category {}",
            r.id,
            t.claim_scope.as_str(),
            t.category.as_str()
        );
    }
    if t.category != BindingKind::Normative && t.claim_scope == ClaimScope::Ga4ghRequirement {
        bail!(
            "check {} claim_scope ga4gh_requirement is not allowed for category {}",
            r.id,
            t.category.as_str()
        );
    }
    if t.check_kind == BindingKind::Normative {
        if t.source_commit.as_deref().unwrap_or("").is_empty() {
            bail!("normative check {} has no source commit", r.id);
        }
        if t.source_file.as_deref().unwrap_or("").is_empty() {
            bail!("normative check {} has no source file", r.id);
        }
        if t.version.as_deref().unwrap_or("").is_empty() {
            bail!("normative check {} has no version", r.id);
        }
        if t.registry_entry.as_deref().unwrap_or("").is_empty() {
            bail!("normative check {} has no registry_entry", r.id);
        }
        if t.authority != Authority::Ga4gh {
            bail!("normative check {} authority is not ga4gh", r.id);
        }
        if t.untraceable_reason.is_some() {
            bail!("normative check {} must not be untraceable", r.id);
        }
        if let Some(rc) = t.release_class.as_deref() {
            if !matches!(rc, "official" | "ballot" | "snapshot" | "development") {
                bail!("check {} has invalid release_class {rc}", r.id);
            }
        } else {
            bail!("normative check {} has no release_class", r.id);
        }
        if let (Some(entry), Some(file), Some(ver)) =
            (&t.registry_entry, &t.source_file, &t.version)
        {
            if let Some(reg) = load_default_registry() {
                if let Some(pack) = pack_by_id(&reg, entry) {
                    if pack.version != *ver {
                        bail!(
                            "check {}: registry version {} does not match check metadata {ver}",
                            r.id,
                            pack.version
                        );
                    }
                    if !pack.normative_sources.iter().any(|s| s.path == *file) {
                        bail!(
                            "check {}: source file {file} is not covered by the pinned source material",
                            r.id
                        );
                    }
                }
            }
        }
    }
    if let (Some(claimed), Some(selected)) = (&t.version, &r.selected_version) {
        if claimed != selected {
            bail!(
                "check {} claims version {claimed} but executed pack is {selected}",
                r.id
            );
        }
    }
    if let (Some(claimed), Some(verified)) = (&t.version, &r.verified_version) {
        if claimed != verified {
            bail!(
                "check {} claims version {claimed} but verified_version is {verified}",
                r.id
            );
        }
    }
    Ok(())
}

pub fn bind_result(r: &mut VerificationResult) {
    if r.layer.is_none() {
        r.layer = Some(crate::layer::for_id(&r.id));
    }
    if r.traceability.is_none() {
        r.traceability = Some(for_id(&r.id));
    }
    if let Some(t) = r.traceability.as_mut() {
        if t.category == BindingKind::Normative {
            t.version = r.selected_version.clone();
            t.registry_entry = r.standards_registry_entry.clone();
            t.source_commit = r.standards_source_commit.clone();
            t.standard = r.standard.clone();
            if let Some(rel) = &t.related_source {
                if t.source_file.is_none() {
                    t.source_file = Some(rel.source_file.clone());
                }
                if t.source_sha256.is_none() {
                    t.source_sha256 = rel.source_sha256.clone();
                }
                if t.source_repository.is_none() {
                    t.source_repository = Some(rel.source_repository.clone());
                }
                if t.release_class.is_none() {
                    t.release_class = Some(rel.release_class.clone());
                }
                if t.source_location.is_none() {
                    t.source_location = Some(format!("{} {}", rel.locator_type, rel.locator));
                }
            }
        }
    }
}

pub fn bind_run(run: &mut crate::model::VerificationRun) -> Result<()> {
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        bind_result(r);
    }
    crate::guardrails::check_run(run)?;
    run.recompute_summary();
    Ok(())
}

pub fn format_trace_text(id: &str) -> Result<String> {
    let spec = spec_by_id(id).ok_or_else(|| anyhow::anyhow!("unknown Helix check id: {id}"))?;
    let t = for_spec(spec);
    let mut out = String::from("HELIX TRACEABILITY\n\n");
    out.push_str("This is not GA4GH certification.\n");
    out.push_str("A related AVAILABLE pin is not a verified-against claim.\n\n");
    out.push_str(&format!("check_id:            {}\n", t.check_id));
    out.push_str(&format!("code:                {}\n", spec.code));
    out.push_str(&format!("category:            {}\n", t.category.as_str()));
    out.push_str(&format!("layer:               {}\n", t.layer.as_str()));
    out.push_str(&format!("check_kind:          {}\n", t.check_kind.as_str()));
    out.push_str(&format!(
        "claim_scope:         {}\n",
        t.claim_scope.as_str()
    ));
    out.push_str(&format!("authority:           {}\n", t.authority.as_str()));
    out.push_str(&format!(
        "conformance_claim:    {}\n",
        if t.claim_scope.may_support_conformance_claim() {
            "allowed only if this check is normative and the executed pack matches"
        } else {
            "no (PASS is not a GA4GH MUST)"
        }
    ));
    out.push_str(&format!("expected_behavior:   {}\n", t.expected_behavior));
    if let Some(req) = &t.request {
        out.push_str(&format!("request:             {req}\n"));
    }
    out.push_str(&format!("implementation:      {}\n", t.implementation));
    if let Some(reason) = &t.untraceable_reason {
        out.push_str(&format!("untraceable_reason:  {reason}\n"));
    }
    out.push_str(&format!(
        "normative_version:   {}\n",
        t.version
            .as_deref()
            .unwrap_or("(none; not a normative binding)")
    ));
    out.push_str(&format!(
        "source_commit:       {}\n",
        t.source_commit.as_deref().unwrap_or("(none)")
    ));
    if let Some(rel) = &t.related_source {
        if t.category == BindingKind::Normative {
            out.push_str("\nAuthoritative GA4GH pin (versioned SpecSource path):\n");
        } else {
            out.push_str("\nRelated registry pin (not a MUST unless this check is normative):\n");
        }
        out.push_str(&format!("  registry_entry:    {}\n", rel.registry_entry));
        out.push_str(&format!("  standard:          {}\n", rel.standard));
        out.push_str(&format!("  version:           {}\n", rel.version));
        out.push_str(&format!("  release_class:     {}\n", rel.release_class));
        out.push_str(&format!("  source_repository: {}\n", rel.source_repository));
        out.push_str(&format!("  source_commit:     {}\n", rel.source_commit));
        if let Some(h) = &rel.source_sha256 {
            out.push_str(&format!("  source_sha256:     {h}\n"));
        }
        out.push_str(&format!("  source_file:       {}\n", rel.source_file));
        out.push_str(&format!(
            "  source_location:   {} {}\n",
            rel.locator_type, rel.locator
        ));
        out.push_str(&format!("  limitation:        {}\n", rel.limitation));
        out.push_str("\nManual follow-back:\n");
        out.push_str("  1. helix standards show ");
        out.push_str(&rel.standard);
        out.push(' ');
        out.push_str(&rel.version);
        out.push('\n');
        out.push_str("  2. Open the vendor copy listed as vendor_path (hash must match).\n");
        out.push_str("  3. Search the file for the locator above.\n");
        out.push_str("  4. On the versioned path, HelixTest run_drs_checks_with_spec compiles those bytes; the unversioned path does not.\n");
        if t.category == BindingKind::Normative {
            if let Some(contract) = crate::standards::contract_for(&rel.registry_entry) {
                out.push_str("\nSupport identities (machine):\n");
                out.push_str(&format!(
                    "  catalog_id:        {}\n",
                    crate::standards::catalog_id(contract)
                ));
                out.push_str(&format!(
                    "  checker_id:        {}\n",
                    crate::standards::declared_checker_id()
                ));
                out.push_str(&format!("  pack_id:           {}\n", contract.pack_id));
                out.push_str(&format!(
                    "  release_commit:    {}\n",
                    contract.release_commit
                ));
            }
        }
    } else {
        out.push_str("\nNo related GA4GH locator. This check is Helix- or HelixTest-defined.\n");
        out.push_str("Docs: docs/TRACEABILITY.md\n");
    }
    Ok(out)
}

pub fn format_trace_json(id: &str) -> Result<serde_json::Value> {
    let spec = spec_by_id(id).ok_or_else(|| anyhow::anyhow!("unknown Helix check id: {id}"))?;
    let t = for_spec(spec);
    Ok(serde_json::json!({
        "schema_version": "helix-traceability-v1",
        "certification": false,
        "check": {
            "id": spec.id,
            "code": spec.code,
            "name": spec.name,
            "service": spec.service,
        },
        "traceability": t,
        "notes": [
            "Not GA4GH certification.",
            "related_source is a registry pin. For normative checks it is the authoritative GA4GH locator; it is not a VERIFIED claim by itself.",
            "drs.object.schema.openapi is the only shipped normative check (DRS 1.4.0 DrsObject via SpecSource).",
            "category is the claim taxonomy; result.category is the domain (schema, lifecycle, …).",
            "claim_scope other than ga4gh_requirement is never a conformance claim.",
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{VerificationCheck, VerificationStatus};
    use crate::standards::LocatorType;

    fn registry() -> Registry {
        load_path(&default_registry_path()).expect("shipped registry")
    }

    fn fabricate_normative(t: &mut CheckTraceability) {
        t.apply_taxonomy(BindingKind::Normative);
        t.authority = Authority::Ga4gh;
        t.untraceable_reason = None;
    }

    #[test]
    fn catalog_covers_every_spec() {
        validate_catalog(&registry()).expect("catalog");
        for spec in SPECS {
            let t = for_spec(spec);
            assert_eq!(t.category, t.check_kind, "{}", spec.id);
            assert_eq!(t.claim_scope, t.category.claim_scope(), "{}", spec.id);
            assert_eq!(t.layer, crate::layer::for_id(spec.id), "{}", spec.id);
            if spec.id == "drs.object.schema.openapi" {
                assert_eq!(t.category, BindingKind::Normative, "{}", spec.id);
                assert_eq!(t.authority, Authority::Ga4gh, "{}", spec.id);
                assert!(t.untraceable_reason.is_none(), "{}", spec.id);
                assert!(t.claim_scope.may_support_conformance_claim(), "{}", spec.id);
                let rel = t.related_source.as_ref().expect("normative related_source");
                assert_eq!(rel.registry_entry, "ga4gh.drs.1.4.0");
                assert_eq!(rel.version, "1.4.0");
                assert_eq!(rel.source_file, "openapi/components/schemas/DrsObject.yaml");
                assert_eq!(rel.locator, "DrsObject");
                continue;
            }
            assert_ne!(
                t.category,
                BindingKind::Normative,
                "{} must not be normative without a provenance-backed SpecSource binding",
                spec.id
            );
            assert_ne!(
                t.category,
                BindingKind::Guidance,
                "{}: HelixTest extras are not official GA4GH implementation guidance",
                spec.id
            );
            assert_ne!(t.authority, Authority::Ga4gh, "{}", spec.id);
            assert!(
                !t.claim_scope.may_support_conformance_claim(),
                "{}",
                spec.id
            );
            assert!(
                t.untraceable_reason.is_some(),
                "{} needs an untraceable_reason",
                spec.id
            );
            assert!(
                t.version.is_none(),
                "{} must not claim a spec version",
                spec.id
            );
        }
    }

    #[test]
    fn mixed_schema_checks_are_fixture_not_interoperability() {
        let schema = for_id("drs.object.schema");
        assert_eq!(schema.category, BindingKind::Fixture);
        assert_eq!(schema.claim_scope, ClaimScope::HelixFixture);
        let wes = for_id("wes.service_info.schema");
        assert_eq!(wes.category, BindingKind::Fixture);
        assert_eq!(wes.claim_scope, ClaimScope::HelixFixture);
    }

    #[test]
    fn fixture_json_cannot_serialize_as_normative() {
        let t = for_id("drs.object.range");
        let v = serde_json::to_value(&t).expect("serialize");
        assert_eq!(v["category"], "fixture");
        assert_eq!(v["check_kind"], "fixture");
        assert_eq!(v["claim_scope"], "helix_fixture");
        assert_ne!(v["category"], "normative");
        assert_ne!(v["check_kind"], "normative");
        assert_ne!(v["claim_scope"], "ga4gh_requirement");
        let text = serde_json::to_string(&t).unwrap();
        assert!(
            !text.contains("\"normative\""),
            "fixture catalog row must not emit the string normative: {text}"
        );
    }

    #[test]
    fn related_locators_are_in_vendor_files() {
        let reg = registry();
        for row in CATALOG {
            if row.related_pack.is_some() {
                related_from_row(&reg, row).unwrap();
            }
        }
    }

    #[test]
    fn fixture_mislabeled_normative_fails_validate_result() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.range")),
            VerificationStatus::Pass,
        );
        r.traceability.as_mut().unwrap().check_kind = BindingKind::Normative;
        r.traceability.as_mut().unwrap().claim_scope = ClaimScope::Ga4ghRequirement;
        r.traceability.as_mut().unwrap().authority = Authority::Ga4gh;
        r.traceability.as_mut().unwrap().untraceable_reason = None;
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(
            err.contains("cannot be serialized as a normative requirement"),
            "{err}"
        );
    }

    #[test]
    fn fixture_claim_scope_ga4gh_requirement_fails() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.range")),
            VerificationStatus::Pass,
        );
        r.traceability.as_mut().unwrap().claim_scope = ClaimScope::Ga4ghRequirement;
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(
            err.contains("cannot be serialized as a normative requirement"),
            "{err}"
        );
    }

    #[test]
    fn category_must_match_check_kind() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("bench.get.health")),
            VerificationStatus::Pass,
        );
        r.traceability.as_mut().unwrap().check_kind = BindingKind::Security;
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(err.contains("does not match check_kind"), "{err}");
    }

    #[test]
    fn interoperability_cannot_take_ga4gh_requirement_scope() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("wes.service_info.reachable")),
            VerificationStatus::Pass,
        );
        r.traceability.as_mut().unwrap().claim_scope = ClaimScope::Ga4ghRequirement;
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(
            err.contains("claim_scope") || err.contains("ga4gh_requirement"),
            "{err}"
        );
    }

    #[test]
    fn normative_without_commit_fails() {
        let mut t = for_id("drs.object.schema");
        fabricate_normative(&mut t);
        t.version = Some("1.4.0".into());
        t.registry_entry = Some("ga4gh.drs.1.4.0".into());
        t.source_file = Some("openapi/data_repository_service.openapi.yaml".into());
        t.source_commit = None;
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.schema")),
            VerificationStatus::Pass,
        );
        r.traceability = Some(t);
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(err.contains("source commit"), "{err}");
    }

    #[test]
    fn claimed_version_must_match_executed_pack() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.schema")),
            VerificationStatus::Pass,
        );
        r.selected_version = Some("1.4.0".into());
        r.verified_version = Some("1.4.0".into());
        r.traceability.as_mut().unwrap().version = Some("1.5.0".into());
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(err.contains("1.5.0"), "{err}");
        assert!(err.contains("1.4.0"), "{err}");
    }

    #[test]
    fn missing_traceability_fails() {
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.reachable")),
            VerificationStatus::Pass,
        );
        r.traceability = None;
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(err.contains("no provenance"), "{err}");
    }

    #[test]
    fn source_file_not_in_normative_sources_fails() {
        let mut t = for_id("drs.object.schema");
        fabricate_normative(&mut t);
        t.version = Some("1.4.0".into());
        t.release_class = Some("official".into());
        t.registry_entry = Some("ga4gh.drs.1.4.0".into());
        t.source_repository =
            Some("https://github.com/ga4gh/data-repository-service-schemas".into());
        t.source_commit = Some("36145d389e0a454428d1dac5c4a30870995fdd7c".into());
        t.source_file = Some("openapi/not-in-the-pin.yaml".into());
        t.source_location = Some("DrsObject".into());
        // Catalog validate is about related locators in vendor files.
        // A fabricated normative result with a file not in the pin is still
        // a result-level error only when we add a coverage check here.
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.schema")),
            VerificationStatus::Pass,
        );
        r.traceability = Some(t);
        r.selected_version = Some("1.4.0".into());
        r.verified_version = Some("1.4.0".into());
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(
            err.contains("not covered by the pinned source material"),
            "{err}"
        );
    }

    #[test]
    fn invalid_release_class_fails() {
        let mut t = for_id("drs.object.schema");
        fabricate_normative(&mut t);
        t.version = Some("1.4.0".into());
        t.release_class = Some("latest".into());
        t.registry_entry = Some("ga4gh.drs.1.4.0".into());
        t.source_repository =
            Some("https://github.com/ga4gh/data-repository-service-schemas".into());
        t.source_commit = Some("36145d389e0a454428d1dac5c4a30870995fdd7c".into());
        t.source_file = Some("openapi/data_repository_service.openapi.yaml".into());
        t.source_location = Some("/objects/{object_id}".into());
        let mut r = VerificationResult::from_check(
            VerificationCheck::from_spec(crate::identity::spec("drs.object.schema")),
            VerificationStatus::Pass,
        );
        r.traceability = Some(t);
        r.selected_version = Some("1.4.0".into());
        r.verified_version = Some("1.4.0".into());
        let err = validate_result(&r).unwrap_err().to_string();
        assert!(err.contains("invalid release_class"), "{err}");
    }

    #[test]
    fn related_pack_must_exist_and_cover_locator() {
        let row = CatalogRow {
            id: "drs.object.reachable",
            kind: BindingKind::Fixture,
            authority: Authority::Helixtest,
            expected: "x",
            implementation: "x",
            untraceable: Some("x"),
            related_pack: Some("ga4gh.does-not-exist"),
            locator_type: Some(LocatorType::HttpPath),
            locator: Some("/objects/{object_id}"),
            limitation: Some("x"),
        };
        let err = related_from_row(&registry(), &row).unwrap_err().to_string();
        assert!(err.contains("not in the registry"), "{err}");
    }

    #[test]
    fn wes_get_service_info_locator_is_in_vendor() {
        let yaml = include_str!(
            "../standards/vendor/ga4gh.wes.1.1.0/workflow_execution_service.openapi.yaml"
        );
        assert!(yaml.contains("operationId: GetServiceInfo"));
        assert!(yaml.contains("ServiceInfo"));
        assert!(yaml.contains("operationId: RunWorkflow"));
    }

    #[test]
    fn drs_path_key_is_in_vendor_but_operation_id_is_not() {
        let yaml = include_str!(
            "../standards/vendor/ga4gh.drs.1.4.0/openapi/data_repository_service.openapi.yaml"
        );
        assert!(yaml.contains("/objects/{object_id}"));
        assert!(
            !yaml.contains("operationId:"),
            "do not claim an operationId that is not in the pinned entry file"
        );
    }
}
