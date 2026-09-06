// SPDX-License-Identifier: Apache-2.0
//! Machine-enforced DRS 1.4.0 verification boundary.
//!
//! Derived from the compiled support catalog plus pinned OpenAPI operations
//! that have no Helix binding. Not a coverage percentage. Not HELIOS.
//! Not GA4GH certification. Catalog identity is unchanged: this module does
//! not feed `catalog_id`.
//!
//! `UNEVALUATED` ≠ `OUT_OF_SCOPE` ≠ `PASS` ≠ `VERIFIED`.

use serde::{Deserialize, Serialize};

use crate::model::{VerificationResult, VerificationRun, VerificationStatus};
use crate::standards::{
    binding_id, catalog_id, contract_for, BindingKind, SupportCheckDecl, SupportContract,
    DRS_140_CONTRACT, SELECTED,
};
use crate::target::FailureAttribution;
use common::spec_source::sha256_hex;

const RANKING_PHRASES: &[&str] = &[
    "winner",
    "leaderboard",
    "ranks second",
    "ranked first",
    "best implementation",
    "worst implementation",
    "more compliant",
    "less compliant",
    "compliance percentage",
    "compliance %",
    "overall score",
    "quality percentage",
    "implementation grade",
    "market comparison",
];

fn ranking_absent() -> String {
    "absent".into()
}

/// Contract role of one coverage row. Not a result status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageClass {
    Normative,
    EvaluatedNonNormative,
    Unevaluated,
    OutOfScope,
}

impl CoverageClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normative => "normative",
            Self::EvaluatedNonNormative => "evaluated_non_normative",
            Self::Unevaluated => "unevaluated",
            Self::OutOfScope => "out_of_scope",
        }
    }
}

/// Helix-contract coverage state for a selected pack. Not a DRS completeness score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    /// No selected supported pack; coverage contract does not apply.
    Inapplicable,
    /// A required Helix-contract row is missing, skipped, failed, or errored.
    Blocked,
    /// Required Helix-contract rows PASSed; pinned-standard operations remain unevaluated.
    Partial,
    /// Required rows PASSed and the compiled contract lists no unevaluated remainder.
    Complete,
}

impl CoverageState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inapplicable => "inapplicable",
            Self::Blocked => "blocked",
            Self::Partial => "partial",
            Self::Complete => "complete",
        }
    }
}

/// One compiled boundary row. Data, not executable config.
#[derive(Debug, Clone, Copy)]
pub struct CoverageDecl {
    pub id: &'static str,
    pub code: Option<&'static str>,
    pub class: CoverageClass,
    pub required_for_ga4gh: bool,
    pub required_for_schema: bool,
    pub execution_binding: Option<&'static str>,
    pub evidence_requirement: &'static str,
    pub standard_role: &'static str,
    pub claim_contribution: &'static str,
}

/// DRS 1.4.0 operations present in the pinned OpenAPI with no Helix catalog binding.
const DRS_140_UNEVALUATED: &[CoverageDecl] = &[
    CoverageDecl {
        id: "drs.op.service_info",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; GET /service-info is not a catalog check",
        standard_role: "pinned OpenAPI GetServiceInfo",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.op.objects_bulk",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; GET /objects (bulk) is not executed",
        standard_role: "pinned OpenAPI bulk GetObject",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.op.access",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; GET /objects/{id}/access/{access_id} is not executed",
        standard_role: "pinned OpenAPI GetAccessURL",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.op.access_bulk",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; POST /objects/access is not executed",
        standard_role: "pinned OpenAPI bulk access",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.op.object_options",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; OPTIONS authorizations are not executed",
        standard_role: "pinned OpenAPI OptionsObject",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.op.object_post_passport",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; POST GetObject with Passport is not executed",
        standard_role: "pinned OpenAPI PostObject / PassportAuth",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.security.authorization",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; DRS securitySchemes are not a catalog check",
        standard_role: "pinned OpenAPI BasicAuth / BearerAuth / PassportAuth",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.checksum.types.other",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "HelixTest advertised lookup is type sha256 (ASCII); IANA sha-256 and other Checksum.yaml types are not evaluated",
        standard_role: "pinned Checksum.yaml type string (example sha-256)",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "drs.object.bundle_contents",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; DrsObject.contents (bundles) is not a catalog check",
        standard_role: "pinned DrsObject.yaml optional contents",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "http.tls.as_drs_requirement",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; TLS is transport, not a DRS catalog check",
        standard_role: "HTTP transport",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "http.client.timeouts",
        code: None,
        class: CoverageClass::Unevaluated,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "none; client timeouts are not a DRS MUST in the catalog",
        standard_role: "HTTP transport",
        claim_contribution: "none",
    },
];

/// Explicit Helix verification-contract exclusions. Not a hiding place for missing tests.
const DRS_140_OUT_OF_SCOPE: &[CoverageDecl] = &[
    CoverageDecl {
        id: "helix.helios",
        code: None,
        class: CoverageClass::OutOfScope,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "excluded; HELIOS owns signed evidence / RO-Crate / PDF",
        standard_role: "not a GA4GH DRS MUST",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "helix.bench.as_verification",
        code: None,
        class: CoverageClass::OutOfScope,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "excluded; helix bench is measurement only",
        standard_role: "not part of helix verify",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "target.non_drs_http",
        code: None,
        class: CoverageClass::OutOfScope,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement:
            "excluded; paths outside the DRS catalog (e.g. /internal/not-in-drs-catalog)",
        standard_role: "not a DRS path",
        claim_contribution: "none",
    },
    CoverageDecl {
        id: "http.discovery.redirect_follow",
        code: None,
        class: CoverageClass::OutOfScope,
        required_for_ga4gh: false,
        required_for_schema: false,
        execution_binding: None,
        evidence_requirement: "excluded; Helix-owned discovery client uses redirect Policy::none",
        standard_role: "Helix HTTP safety, not a DRS catalog check",
        claim_contribution: "none",
    },
];

/// Result observed for a compiled row on this run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageRow {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub classification: CoverageClass,
    pub required_for_ga4gh_requirement: bool,
    pub required_for_schema_claim: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_binding: Option<String>,
    pub evidence_requirement: String,
    pub standard_role: String,
    pub claim_contribution: String,
    pub executed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<VerificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<FailureAttribution>,
}

/// Derived coverage evaluation. Presentation must recompute this; it is not a declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    pub state: CoverageState,
    /// Derived. Never a stamp input. Integrity recomputes it.
    pub required_complete: bool,
    pub rows: Vec<CoverageRow>,
    pub unevaluated: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub required_missing: Vec<String>,
    #[serde(default = "ranking_absent")]
    pub ranking_semantics: String,
    #[serde(default)]
    pub creates_verification: bool,
}

impl CoverageReport {
    pub fn from_run(run: &VerificationRun) -> Self {
        let sel = run.standard_selection.as_ref();
        let pack_id = sel.and_then(|s| s.standards_registry_entry.as_deref());
        let selected = sel.map(|s| s.selection_status.as_str()) == Some(SELECTED);
        let contract = pack_id.and_then(contract_for);
        if !selected || contract.is_none() {
            return Self::inapplicable(sel);
        }
        let Some(contract) = contract else {
            return Self::inapplicable(sel);
        };
        let checker = sel
            .and_then(|s| s.checker_id.clone())
            .unwrap_or_else(crate::checker::executed_checker_id);
        let catalog = catalog_id(contract);
        let binding = match (
            sel.and_then(|s| s.pack_integrity_sha256.as_deref()),
            sel.and_then(|s| s.schema_document_sha256.as_deref()),
            sel.and_then(|s| s.schema_component_sha256.as_deref()),
        ) {
            (Some(p), Some(d), Some(c)) => Some(binding_id(contract, p, d, c)),
            _ => sel.and_then(|s| s.binding_id.clone()),
        };
        let coverage_id = coverage_id(
            contract,
            &checker,
            binding.as_deref(),
            sel.and_then(|s| s.pack_integrity_sha256.as_deref()),
        );
        let decls = compiled_decls(contract);
        let mut rows = Vec::new();
        let mut unevaluated = Vec::new();
        let mut out_of_scope = Vec::new();
        let mut required_missing = Vec::new();
        let mut required_complete = true;
        for decl in decls {
            let observed = match decl.class {
                CoverageClass::Unevaluated | CoverageClass::OutOfScope => None,
                CoverageClass::Normative | CoverageClass::EvaluatedNonNormative => {
                    find_check(run, decl.id, decl.code)
                }
            };
            let result = observed.map(|r| r.status);
            if decl.required_for_ga4gh && result != Some(VerificationStatus::Pass) {
                required_complete = false;
                required_missing.push(decl.id.to_string());
            }
            match decl.class {
                CoverageClass::Unevaluated => unevaluated.push(decl.id.to_string()),
                CoverageClass::OutOfScope => out_of_scope.push(decl.id.to_string()),
                CoverageClass::Normative | CoverageClass::EvaluatedNonNormative => {}
            }
            let executed = matches!(
                result,
                Some(VerificationStatus::Pass)
                    | Some(VerificationStatus::Fail)
                    | Some(VerificationStatus::Error)
            );
            rows.push(CoverageRow {
                id: decl.id.to_string(),
                code: decl.code.map(str::to_string),
                classification: decl.class,
                required_for_ga4gh_requirement: decl.required_for_ga4gh,
                required_for_schema_claim: decl.required_for_schema,
                execution_binding: decl.execution_binding.map(str::to_string),
                evidence_requirement: decl.evidence_requirement.to_string(),
                standard_role: decl.standard_role.to_string(),
                claim_contribution: decl.claim_contribution.to_string(),
                executed,
                result,
                attribution: observed.and_then(|r| r.attribution),
            });
        }
        let has_unevaluated = !unevaluated.is_empty();
        let state = if !required_complete {
            CoverageState::Blocked
        } else if has_unevaluated {
            CoverageState::Partial
        } else {
            CoverageState::Complete
        };
        Self {
            coverage_id: Some(coverage_id),
            standard: sel.and_then(|s| s.standard.clone()),
            selected_version: sel.and_then(|s| s.selected_version.clone()),
            release_commit: sel.and_then(|s| s.standards_source_commit.clone()),
            pack_id: pack_id.map(str::to_string),
            checker_id: Some(checker),
            binding_id: binding,
            catalog_id: Some(catalog),
            execution_id: sel.and_then(|s| s.execution_id.clone()),
            state,
            required_complete,
            rows,
            unevaluated,
            out_of_scope,
            required_missing,
            ranking_semantics: "absent".into(),
            creates_verification: false,
        }
    }

    fn inapplicable(sel: Option<&crate::model::StandardSelection>) -> Self {
        Self {
            coverage_id: None,
            standard: sel.and_then(|s| s.standard.clone()),
            selected_version: sel.and_then(|s| s.selected_version.clone()),
            release_commit: sel.and_then(|s| s.standards_source_commit.clone()),
            pack_id: sel.and_then(|s| s.standards_registry_entry.clone()),
            checker_id: sel.and_then(|s| s.checker_id.clone()),
            binding_id: sel.and_then(|s| s.binding_id.clone()),
            catalog_id: sel.and_then(|s| s.catalog_id.clone()),
            execution_id: sel.and_then(|s| s.execution_id.clone()),
            state: CoverageState::Inapplicable,
            required_complete: false,
            rows: Vec::new(),
            unevaluated: Vec::new(),
            out_of_scope: Vec::new(),
            required_missing: Vec::new(),
            ranking_semantics: "absent".into(),
            creates_verification: false,
        }
    }

    pub fn contains_ranking_semantics(&self) -> bool {
        let Ok(text) = serde_json::to_string(self) else {
            return true;
        };
        let lower = text.to_lowercase();
        RANKING_PHRASES.iter().any(|p| lower.contains(p))
            || self.ranking_semantics != "absent"
            || self.creates_verification
    }

    pub fn row(&self, id: &str) -> Option<&CoverageRow> {
        self.rows.iter().find(|r| r.id == id)
    }
}

fn find_check<'a>(
    run: &'a VerificationRun,
    id: &str,
    code: Option<&str>,
) -> Option<&'a VerificationResult> {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id || code.is_some_and(|c| r.code == c))
}

fn catalog_decl(check: &SupportCheckDecl) -> CoverageDecl {
    let normative = check.kind == BindingKind::Normative;
    CoverageDecl {
        id: check.id,
        code: Some(check.code),
        class: if normative {
            CoverageClass::Normative
        } else {
            CoverageClass::EvaluatedNonNormative
        },
        required_for_ga4gh: true,
        required_for_schema: normative && check.layer == crate::layer::CheckLayer::Schema,
        execution_binding: Some(check.helixtest_name),
        evidence_requirement: if normative {
            "PASS against pinned SpecSource DrsObject"
        } else {
            "PASS HelixTest fixture probe (not a GA4GH MUST)"
        },
        standard_role: check.locator,
        claim_contribution: if normative {
            "ga4gh_requirement+schema"
        } else {
            "ga4gh_requirement_catalog"
        },
    }
}

fn compiled_decls(contract: &SupportContract) -> Vec<CoverageDecl> {
    let mut out: Vec<CoverageDecl> = contract.checks.iter().map(catalog_decl).collect();
    if contract.pack_id == DRS_140_CONTRACT.pack_id {
        out.extend_from_slice(DRS_140_UNEVALUATED);
        out.extend_from_slice(DRS_140_OUT_OF_SCOPE);
    }
    out
}

/// Canonical coverage-contract bytes. No target URL, timestamp, path, or PID.
pub fn coverage_canonical(
    contract: &SupportContract,
    checker_id: &str,
    binding_id: Option<&str>,
    pack_integrity_sha256: Option<&str>,
) -> String {
    let mut buf = String::from("helix-coverage-v1\n");
    buf.push_str(&format!(
        "pack_id={}\nversion={}\ncommit={}\ncatalog_id={}\nchecker_id={}\n",
        contract.pack_id,
        contract.version,
        contract.release_commit,
        catalog_id(contract),
        checker_id
    ));
    buf.push_str(&format!(
        "binding_id={}\npack_integrity_sha256={}\n",
        binding_id.unwrap_or(""),
        pack_integrity_sha256.unwrap_or("")
    ));
    for d in compiled_decls(contract) {
        buf.push_str(&format!(
            "row={}|{}|{}|{}|{}|{}\n",
            d.id,
            d.class.as_str(),
            d.required_for_ga4gh,
            d.required_for_schema,
            d.execution_binding.unwrap_or(""),
            d.claim_contribution
        ));
    }
    buf
}

/// Deterministic coverage identity. Derived from [`coverage_canonical`].
pub fn coverage_id(
    contract: &SupportContract,
    checker_id: &str,
    binding_id: Option<&str>,
    pack_integrity_sha256: Option<&str>,
) -> String {
    sha256_hex(
        coverage_canonical(contract, checker_id, binding_id, pack_integrity_sha256).as_bytes(),
    )
}

pub fn required_ga4gh_ids(contract: &SupportContract) -> Vec<&'static str> {
    compiled_decls(contract)
        .into_iter()
        .filter(|d| d.required_for_ga4gh)
        .map(|d| d.id)
        .collect()
}

pub fn compiled_contract_is_sound(contract: &SupportContract) -> Result<(), String> {
    for d in compiled_decls(contract) {
        if d.required_for_ga4gh
            && matches!(
                d.class,
                CoverageClass::Unevaluated | CoverageClass::OutOfScope
            )
        {
            return Err(format!(
                "required row {} cannot be {}",
                d.id,
                d.class.as_str()
            ));
        }
    }
    Ok(())
}

pub fn format_coverage_section(run: &VerificationRun) -> String {
    let cov = CoverageReport::from_run(run);
    let mut out = String::from("Coverage (Helix DRS 1.4.0 contract; not a score; not full DRS):\n");
    out.push_str(&format!(
        "  coverage_id: {}\n",
        cov.coverage_id.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!("  state: {}\n", cov.state.as_str()));
    out.push_str(&format!(
        "  required_complete: {} (derived)\n",
        cov.required_complete
    ));
    out.push_str("  UNEVALUATED is not OUT_OF_SCOPE. PASS is not VERIFIED.\n");
    if !cov.unevaluated.is_empty() {
        out.push_str("  unevaluated:\n");
        for id in &cov.unevaluated {
            out.push_str(&format!("    - {id}\n"));
        }
    }
    if !cov.out_of_scope.is_empty() {
        out.push_str("  out_of_scope:\n");
        for id in &cov.out_of_scope {
            out.push_str(&format!("    - {id}\n"));
        }
    }
    if !cov.required_missing.is_empty() {
        out.push_str("  required_missing:\n");
        for id in &cov.required_missing {
            out.push_str(&format!("    - {id}\n"));
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_drs_140_required_rows_are_not_unevaluated_or_out_of_scope() {
        compiled_contract_is_sound(&DRS_140_CONTRACT).unwrap();
    }

    #[test]
    fn coverage_id_is_deterministic() {
        let checker = crate::checker::executed_checker_id();
        let a = coverage_id(&DRS_140_CONTRACT, &checker, Some("b"), Some("p"));
        let b = coverage_id(&DRS_140_CONTRACT, &checker, Some("b"), Some("p"));
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn coverage_id_excludes_target_url() {
        let checker = crate::checker::executed_checker_id();
        let canon = coverage_canonical(&DRS_140_CONTRACT, &checker, None, None);
        assert!(!canon.contains("http://"));
        assert!(!canon.contains("https://"));
        assert!(!canon.contains("127.0.0.1"));
        assert!(!canon.to_lowercase().contains("timestamp"));
        assert!(!canon.contains("pid"));
    }

    #[test]
    fn changing_canonical_content_changes_coverage_id() {
        let checker = crate::checker::executed_checker_id();
        let a = coverage_canonical(&DRS_140_CONTRACT, &checker, Some("b"), Some("p"));
        let mut b = a.clone();
        b.push_str("row=forged.out_of_scope|out_of_scope|false|false||none\n");
        assert_ne!(sha256_hex(a.as_bytes()), sha256_hex(b.as_bytes()));
    }
}
