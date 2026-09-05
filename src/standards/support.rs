// SPDX-License-Identifier: Apache-2.0
//! Executable support contract. YAML `support_status: supported` is not sufficient.
//!
//! A pack is SUPPORTED only when this module's predicates hold. DRS 1.4.0 is the
//! only compiled contract in B3. Not HELIOS. Not GA4GH certification.

use anyhow::Result;
use std::path::Path;

use crate::identity::spec_by_id;
use crate::layer::CheckLayer;
use crate::model::{HELIXTEST_PIN, HELIXTEST_SHA};
use crate::standards::model::{
    BindingKind, ClaimScope, CoverageLevel, LocatorType, PackCoverage, ReleaseClass,
    StandardVersion, SupportStatus, TestBinding, VersionCitation,
};
use crate::standards::pack::{checker_id, load_pack};
use common::spec_source::sha256_hex;

pub const DRS_140_PACK_ID: &str = "ga4gh.drs.1.4.0";
pub const DRS_140_VERSION: &str = "1.4.0";
pub const DRS_140_COMMIT: &str = "36145d389e0a454428d1dac5c4a30870995fdd7c";
pub const DRS_140_SCHEMA_ENTRY: &str = "openapi/components/schemas/DrsObject.yaml";
pub const DRS_140_SCHEMA_COMPONENT: &str = "DrsObject";
pub const DRS_140_FIXTURE_CATALOG: &str = "helix-fixtures-v1";

pub const DRS_OPENAPI_SPECSOURCE_CHECK: &str = "DRS DrsObject OpenAPI SpecSource";

#[derive(Debug, Clone, Copy)]
pub struct SupportCheckDecl {
    pub id: &'static str,
    pub code: &'static str,
    pub kind: BindingKind,
    pub layer: CheckLayer,
    pub helixtest_name: &'static str,
    pub source_path: &'static str,
    pub locator_type: LocatorType,
    pub locator: &'static str,
    pub excerpt: Option<&'static str>,
}

impl SupportCheckDecl {
    pub fn claim_scope(self) -> ClaimScope {
        self.kind.claim_scope()
    }

    pub fn citation(self) -> Option<VersionCitation> {
        if self.kind != BindingKind::Normative {
            return None;
        }
        Some(VersionCitation {
            source_path: self.source_path.into(),
            locator_type: self.locator_type,
            locator: self.locator.into(),
            excerpt: self.excerpt.map(str::to_string),
        })
    }

    pub fn binding(self) -> TestBinding {
        TestBinding {
            id: self.id.into(),
            code: self.code.into(),
            kind: self.kind,
            citation: self.citation(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SupportContract {
    pub pack_id: &'static str,
    pub standard: &'static str,
    pub version: &'static str,
    pub release_commit: &'static str,
    pub schema_entry: &'static str,
    pub schema_component: &'static str,
    pub fixture_catalog: &'static str,
    pub checks: &'static [SupportCheckDecl],
    pub coverage: PackCoverageRef,
}

#[derive(Debug, Clone, Copy)]
pub struct PackCoverageRef {
    pub schema: CoverageLevel,
    pub behavior: CoverageLevel,
    pub security: CoverageLevel,
    pub interoperability: CoverageLevel,
    pub notes: &'static str,
}

impl PackCoverageRef {
    pub fn owned(self) -> PackCoverage {
        PackCoverage {
            schema: self.schema,
            behavior: self.behavior,
            security: self.security,
            interoperability: self.interoperability,
            notes: Some(self.notes.into()),
        }
    }
}

const DRS_140_CHECKS: &[SupportCheckDecl] = &[
    SupportCheckDecl {
        id: "drs.object.schema.openapi",
        code: "HLX-DRS-006",
        kind: BindingKind::Normative,
        layer: CheckLayer::Schema,
        helixtest_name: DRS_OPENAPI_SPECSOURCE_CHECK,
        source_path: "openapi/components/schemas/DrsObject.yaml",
        locator_type: LocatorType::SchemaName,
        locator: "DrsObject",
        excerpt: Some(
            "GET /objects/{object_id} 200 content schema is DrsObject (required: id, self_uri, size, created_time, checksums)",
        ),
    },
    SupportCheckDecl {
        id: "drs.object.schema",
        code: "HLX-DRS-002",
        kind: BindingKind::Fixture,
        layer: CheckLayer::Schema,
        helixtest_name: "DRS DrsObject OpenAPI + access_methods",
        source_path: "openapi/components/schemas/DrsObject.yaml",
        locator_type: LocatorType::SchemaName,
        locator: "DrsObject",
        excerpt: None,
    },
    SupportCheckDecl {
        id: "drs.object.reachable",
        code: "HLX-DRS-001",
        kind: BindingKind::Fixture,
        layer: CheckLayer::Interoperability,
        helixtest_name: "DRS object endpoint reachable",
        source_path: "openapi/paths/objects@{object_id}.yaml",
        locator_type: LocatorType::HttpPath,
        locator: "/objects/{object_id}",
        excerpt: None,
    },
    SupportCheckDecl {
        id: "drs.object.checksum",
        code: "HLX-DRS-003",
        kind: BindingKind::Fixture,
        layer: CheckLayer::Behavior,
        helixtest_name: "DRS checksum correctness",
        source_path: "openapi/components/schemas/DrsObject.yaml",
        locator_type: LocatorType::SchemaName,
        locator: "checksums",
        excerpt: None,
    },
    SupportCheckDecl {
        id: "drs.object.range",
        code: "HLX-DRS-004",
        kind: BindingKind::Fixture,
        layer: CheckLayer::Behavior,
        helixtest_name: "DRS HTTP Range support",
        source_path: "openapi/components/schemas/AccessURL.yaml",
        locator_type: LocatorType::SchemaName,
        locator: "AccessURL",
        excerpt: None,
    },
    SupportCheckDecl {
        id: "drs.object.not_found",
        code: "HLX-DRS-005",
        kind: BindingKind::Fixture,
        layer: CheckLayer::Behavior,
        helixtest_name: "DRS invalid object id returns 404",
        source_path: "openapi/paths/objects@{object_id}.yaml",
        locator_type: LocatorType::HttpPath,
        locator: "404",
        excerpt: None,
    },
];

const DRS_140_COVERAGE_NOTES: &str = "\
covered: GET /objects/{object_id} 200 JSON against the pinned DrsObject schema (versioned SpecSource).\n\
partially covered: HelixTest fixture probes (reachable, extras, checksum download, Range 206, unknown-id 404) are executed but are not GA4GH MUSTs.\n\
not covered: bulk objects, /access, passports, OPTIONS authorizations, service-info, bundles/contents, every optional DrsObject property as MUST.\n\
SCHEMA PASS is not BEHAVIOR coverage. Fixture PASS is not normative PASS.";

pub const DRS_140_CONTRACT: SupportContract = SupportContract {
    pack_id: DRS_140_PACK_ID,
    standard: "drs",
    version: DRS_140_VERSION,
    release_commit: DRS_140_COMMIT,
    schema_entry: DRS_140_SCHEMA_ENTRY,
    schema_component: DRS_140_SCHEMA_COMPONENT,
    fixture_catalog: DRS_140_FIXTURE_CATALOG,
    checks: DRS_140_CHECKS,
    coverage: PackCoverageRef {
        schema: CoverageLevel::Partial,
        behavior: CoverageLevel::None,
        security: CoverageLevel::None,
        interoperability: CoverageLevel::Partial,
        notes: DRS_140_COVERAGE_NOTES,
    },
};

const CONTRACTS: &[SupportContract] = &[DRS_140_CONTRACT];

pub fn contract_for(pack_id: &str) -> Option<&'static SupportContract> {
    CONTRACTS.iter().find(|c| c.pack_id == pack_id)
}

pub fn catalog_id(contract: &SupportContract) -> String {
    let mut buf = String::from("helix-catalog-v1\n");
    buf.push_str(&format!(
        "pack={} version={} commit={}\n",
        contract.pack_id, contract.version, contract.release_commit
    ));
    for c in contract.checks {
        buf.push_str(&format!(
            "check={}|{}|{}|{}|{}|{}|{}\n",
            c.id,
            c.code,
            c.kind.as_str(),
            c.layer.as_str(),
            c.helixtest_name,
            c.source_path,
            c.locator
        ));
    }
    buf.push_str(&format!(
        "coverage schema={} behavior={} security={} interoperability={}\n",
        contract.coverage.schema.as_str(),
        contract.coverage.behavior.as_str(),
        contract.coverage.security.as_str(),
        contract.coverage.interoperability.as_str()
    ));
    sha256_hex(buf.as_bytes())
}

pub fn declared_checker_id() -> String {
    checker_id(HELIXTEST_PIN, HELIXTEST_SHA)
}

pub fn binding_id(
    contract: &SupportContract,
    pack_integrity_sha256: &str,
    schema_document_sha256: &str,
    schema_component_sha256: &str,
) -> String {
    let mut buf = String::from("helix-binding-v1\n");
    buf.push_str(&format!("catalog_id={}\n", catalog_id(contract)));
    buf.push_str(&format!("pack_id={}\n", contract.pack_id));
    buf.push_str(&format!("commit={}\n", contract.release_commit));
    buf.push_str(&format!("pack_integrity_sha256={pack_integrity_sha256}\n"));
    buf.push_str(&format!(
        "schema_document_sha256={schema_document_sha256}\n"
    ));
    buf.push_str(&format!(
        "schema_component_sha256={schema_component_sha256}\n"
    ));
    buf.push_str(&format!("checker_id={}\n", declared_checker_id()));
    sha256_hex(buf.as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportVerdict {
    pub supported: bool,
    pub catalog_id: String,
    pub binding_id: Option<String>,
    pub checker_id: String,
    pub reasons: Vec<String>,
}

impl SupportVerdict {
    pub fn fail(reasons: Vec<String>) -> Self {
        Self {
            supported: false,
            catalog_id: String::new(),
            binding_id: None,
            checker_id: declared_checker_id(),
            reasons,
        }
    }
}

/// Machine gate. YAML `supported` is an input, not the conclusion.
pub fn evaluate_support(v: &StandardVersion, repo_root: Option<&Path>) -> SupportVerdict {
    match evaluate_support_inner(v, repo_root) {
        Ok(verdict) => verdict,
        Err(e) => SupportVerdict::fail(vec![e.to_string()]),
    }
}

fn evaluate_support_inner(v: &StandardVersion, repo_root: Option<&Path>) -> Result<SupportVerdict> {
    let mut reasons: Vec<String> = Vec::new();

    if v.support_status != SupportStatus::Supported {
        reasons.push(format!(
            "registry support_status is {} (not supported)",
            v.support_status.as_str()
        ));
    }

    let Some(contract) = contract_for(&v.pack_id) else {
        reasons.push(format!(
            "no compiled support contract for pack {} (YAML cannot create SUPPORTED)",
            v.pack_id
        ));
        return Ok(SupportVerdict::fail(reasons));
    };

    let cid = catalog_id(contract);

    if v.standard != contract.standard {
        reasons.push(format!(
            "standard {} != contract {}",
            v.standard, contract.standard
        ));
    }
    if v.version != contract.version {
        reasons.push(format!(
            "version {} != contract {}",
            v.version, contract.version
        ));
    }
    if v.commit != contract.release_commit {
        reasons.push(format!(
            "commit {} != contract {}",
            v.commit, contract.release_commit
        ));
    }
    if v.release_class != ReleaseClass::Official {
        reasons.push(format!(
            "release_class {} is not official",
            v.release_class.as_str()
        ));
    }
    if v.schema_entry.as_deref() != Some(contract.schema_entry) {
        reasons.push("schema_entry does not match the contract".into());
    }
    if v.schema_component.as_deref() != Some(contract.schema_component) {
        reasons.push("schema_component does not match the contract".into());
    }
    if v.pack_integrity.is_none() {
        reasons.push("missing pack_integrity".into());
    }
    if v.fixture_catalog.as_deref() != Some(contract.fixture_catalog) {
        reasons.push("fixture_catalog does not match the contract".into());
    }

    if contract.checks.is_empty() {
        reasons.push("support contract catalog is empty".into());
    }

    let normative: Vec<_> = contract
        .checks
        .iter()
        .filter(|c| c.kind == BindingKind::Normative)
        .collect();
    if normative.is_empty() {
        reasons.push("fixture-only catalog cannot satisfy normative support".into());
    }
    for c in &normative {
        if c.source_path.is_empty() || c.locator.is_empty() {
            reasons.push(format!("{} normative check lacks provenance", c.id));
        }
        if !v.normative_sources.iter().any(|s| s.path == c.source_path) {
            reasons.push(format!(
                "{} source file {} is not in normative_sources",
                c.id, c.source_path
            ));
        }
        if c.kind.claim_scope() != ClaimScope::Ga4ghRequirement {
            reasons.push(format!("{} normative claim_scope mismatch", c.id));
        }
        if c.layer == CheckLayer::Schema && contract.coverage.schema == CoverageLevel::None {
            reasons.push("schema check present but coverage.schema is none".into());
        }
        if c.layer == CheckLayer::Behavior && contract.coverage.schema != CoverageLevel::None {
            // schema coverage must not be described as behavior; checked below
        }
    }

    if contract.coverage.schema != CoverageLevel::None
        && contract.coverage.behavior == CoverageLevel::Complete
        && !contract
            .checks
            .iter()
            .any(|c| c.kind == BindingKind::Normative && c.layer == CheckLayer::Behavior)
    {
        reasons
            .push("behavior coverage claimed complete without a normative behavior check".into());
    }

    match &v.test_bindings {
        None => reasons.push("missing test_bindings".into()),
        Some(bindings) if bindings.is_empty() => reasons.push("empty test_bindings".into()),
        Some(bindings) => {
            if bindings.len() != contract.checks.len() {
                reasons.push(format!(
                    "test_bindings len {} != contract catalog {}",
                    bindings.len(),
                    contract.checks.len()
                ));
            }
            for (i, (got, want)) in bindings.iter().zip(contract.checks.iter()).enumerate() {
                if got.id != want.id || got.code != want.code || got.kind != want.kind {
                    reasons.push(format!(
                        "test_bindings[{i}] {}/{} {:?} != contract {}/{} {:?}",
                        got.id, got.code, got.kind, want.id, want.code, want.kind
                    ));
                }
                if want.kind == BindingKind::Normative {
                    match &got.citation {
                        None => reasons.push(format!("{} missing normative citation", want.id)),
                        Some(cit) => {
                            if cit.source_path != want.source_path || cit.locator != want.locator {
                                reasons.push(format!(
                                    "{} citation does not match contract provenance",
                                    want.id
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    for c in contract.checks {
        let Some(spec) = spec_by_id(c.id) else {
            reasons.push(format!("{} is not in the Helix identity catalog", c.id));
            continue;
        };
        if spec.code != c.code {
            reasons.push(format!(
                "{} code {} != identity {}",
                c.id, c.code, spec.code
            ));
        }
        if spec.helixtest_names.is_empty() {
            reasons.push(format!("{} has no executable HelixTest name", c.id));
        } else if !spec.helixtest_names.contains(&c.helixtest_name) {
            reasons.push(format!(
                "{} HelixTest name {} is not bound in identity SPECS",
                c.id, c.helixtest_name
            ));
        }
        if c.kind == BindingKind::Normative && !spec.wraps_helixtest() {
            reasons.push(format!("{} normative check has no HelixTest binding", c.id));
        }
    }

    match &v.coverage {
        None => reasons.push("missing coverage declaration".into()),
        Some(cov) => {
            if cov.schema != contract.coverage.schema
                || cov.behavior != contract.coverage.behavior
                || cov.security != contract.coverage.security
                || cov.interoperability != contract.coverage.interoperability
            {
                reasons.push("declared coverage does not match the support contract".into());
            }
        }
    }

    if v.catalog_id.as_deref() != Some(cid.as_str()) {
        reasons.push(format!(
            "catalog_id mismatch: registry {:?} computed {cid}",
            v.catalog_id
        ));
    }

    let mut computed_binding: Option<String> = None;
    if let Some(root) = repo_root {
        match load_pack(v, root) {
            Ok(loaded) => {
                let bid = binding_id(
                    contract,
                    &loaded.pack_integrity_sha256,
                    &loaded.expected.schema_document_sha256,
                    &loaded.expected.schema_component_sha256,
                );
                computed_binding = Some(bid.clone());
                if v.binding_id.as_deref() != Some(bid.as_str()) {
                    reasons.push(format!(
                        "binding_id mismatch: registry {:?} computed {bid} (catalog/pack/checker identity changed without a new binding id)",
                        v.binding_id
                    ));
                }
            }
            Err(e) => reasons.push(format!("pack is not executable: {e}")),
        }
    } else if v.binding_id.as_deref().unwrap_or("").len() != 64 {
        reasons.push("binding_id missing or not a sha256 hex".into());
    }

    if v.support_status != SupportStatus::Supported {
        return Ok(SupportVerdict::fail(reasons));
    }
    if !reasons.is_empty() {
        return Ok(SupportVerdict::fail(reasons));
    }

    Ok(SupportVerdict {
        supported: true,
        catalog_id: cid,
        binding_id: computed_binding.or_else(|| v.binding_id.clone()),
        checker_id: declared_checker_id(),
        reasons: Vec::new(),
    })
}

/// Fail-closed for validate_loaded: YAML supported ⇒ gate must pass.
pub fn require_supported(v: &StandardVersion, repo_root: Option<&Path>) -> Result<(), String> {
    if v.support_status != SupportStatus::Supported {
        return Ok(());
    }
    let verdict = evaluate_support(v, repo_root);
    if verdict.supported {
        return Ok(());
    }
    Err(format!(
        "{} declared supported but the support gate failed: {}",
        v.pack_id,
        verdict.reasons.join("; ")
    ))
}

/// Selection-time gate: YAML supported is necessary; the compiled contract must exist.
pub fn yaml_supported_is_executable(v: &StandardVersion) -> bool {
    v.support_status == SupportStatus::Supported && contract_for(&v.pack_id).is_some()
}

pub fn expected_bindings(contract: &SupportContract) -> Vec<TestBinding> {
    contract.checks.iter().map(|c| c.binding()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standards::{default_registry_path, helix_repo_root, load_path};

    #[test]
    fn catalog_id_is_stable_hex() {
        let id = catalog_id(&DRS_140_CONTRACT);
        assert_eq!(id.len(), 64);
        assert_eq!(id, catalog_id(&DRS_140_CONTRACT));
    }

    #[test]
    fn empty_catalog_cannot_be_the_drs_140_contract() {
        assert!(!DRS_140_CONTRACT.checks.is_empty());
        assert!(DRS_140_CONTRACT
            .checks
            .iter()
            .any(|c| c.kind == BindingKind::Normative));
    }

    #[test]
    fn shipped_drs_140_passes_the_support_gate() {
        let reg = load_path(&default_registry_path()).unwrap();
        let v = reg
            .versions
            .iter()
            .find(|v| v.pack_id == DRS_140_PACK_ID)
            .unwrap();
        let verdict = evaluate_support(v, Some(&helix_repo_root()));
        assert!(
            verdict.supported,
            "DRS 1.4.0 support gate failed: {:?}",
            verdict.reasons
        );
        assert_eq!(verdict.catalog_id, catalog_id(&DRS_140_CONTRACT));
        assert_eq!(verdict.checker_id, declared_checker_id());
    }

    #[test]
    fn yaml_supported_without_contract_is_not_supported() {
        let mut reg = load_path(&default_registry_path()).unwrap();
        let v = reg
            .versions
            .iter_mut()
            .find(|v| v.pack_id == "ga4gh.drs.1.5.0")
            .unwrap();
        v.support_status = SupportStatus::Supported;
        v.fixture_catalog = Some("helix-fixtures-v1".into());
        v.test_bindings = Some(expected_bindings(&DRS_140_CONTRACT));
        let verdict = evaluate_support(v, Some(&helix_repo_root()));
        assert!(!verdict.supported, "{:?}", verdict.reasons);
        assert!(verdict
            .reasons
            .iter()
            .any(|r| r.contains("no compiled support contract")));
    }
}
