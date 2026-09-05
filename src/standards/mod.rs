// SPDX-License-Identifier: Apache-2.0
//! Helix GA4GH standards registry.
//!
//! Provenance pins for GA4GH specification releases.
//!
//! `helix verify` without `--standard` stays the unversioned HelixTest wrap.
//! `--standard` / `--version` / `--all-supported-versions` load this registry
//! for fail-closed selection. Runtime must not fetch GA4GH files.
//! Not HELIOS. Not certification. A GitHub tag alone does not make a version supported.

mod model;
mod pack;
mod select;
mod support;
mod validate;

pub use model::{
    BindingKind, ClaimScope, CoverageLevel, Integrity, LocatorType, PackCoverage, ReleaseClass,
    SourceRole, StandardVersion, SupportStatus, TestBinding, VersionCitation, VersionSource,
};
pub use pack::{
    checker_id, compare_spec_identity, copy_pack_tree, execution_id, helix_repo_root, load_pack,
    pack_integrity_hex, LoadedPack, PackLoadError, DRS_OBJECT_CLOSURE, MANIFEST_ALG,
};
pub use select::{
    select_all_official_supported, select_automatic, select_explicit, PackRef, SelectionError,
    AMBIGUOUS, AVAILABLE_BUT_NOT_SUPPORTED, DEVELOPMENT_NOT_SELECTABLE, INSUFFICIENT,
    MULTIPLE_PACKS_NOT_EXECUTABLE, NEEDS_RELEASE_CLASS, NOT_SUPPORTED, NO_OFFICIAL_SUPPORTED,
    SELECTED, UNKNOWN_TO_HELIX, UNVERSIONED,
};
pub use support::{
    binding_id, catalog_id, contract_for, declared_checker_id, evaluate_support, expected_bindings,
    yaml_supported_is_executable, SupportContract, SupportVerdict, DRS_140_CONTRACT,
    DRS_140_PACK_ID, DRS_OPENAPI_SPECSOURCE_CHECK,
};
pub use validate::{
    confined_vendor_file, hex_sha256, validate_loaded, validate_path, validate_yaml,
    ValidationError, ValidationKind,
};

use std::path::{Path, PathBuf};

pub const REGISTRY_SCHEMA_VERSION: &str = "helix-standards-registry-v1";
pub const VERSION_SCHEMA_VERSION: &str = "helix-standard-version-v1";

/// Shipped registry in this crate (`standards/registry.yaml`).
pub fn default_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("standards/registry.yaml")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: String,
    pub versions: Vec<StandardVersion>,
}

impl Registry {
    /// Rows default automation may use: OFFICIAL ∩ SUPPORTED.
    /// Ballot/snapshot/development are never included.
    pub fn official_supported(&self) -> Vec<&StandardVersion> {
        self.versions
            .iter()
            .filter(|v| {
                v.release_class == ReleaseClass::Official
                    && v.support_status == SupportStatus::Supported
                    && crate::standards::evaluate_support(v, None).supported
            })
            .collect()
    }

    /// Exact `(standard, version)` lookup. Never returns a different version.
    pub fn lookup(
        &self,
        standard: &str,
        version: &str,
        release_class: Option<ReleaseClass>,
    ) -> Lookup<'_> {
        let mut hits: Vec<&StandardVersion> = self
            .versions
            .iter()
            .filter(|v| v.standard == standard && v.version == version)
            .collect();
        if let Some(class) = release_class {
            hits.retain(|v| v.release_class == class);
        }
        match hits.len() {
            0 => Lookup::Unknown {
                standard: standard.to_string(),
                version: version.to_string(),
                others: self
                    .versions
                    .iter()
                    .filter(|v| v.standard == standard)
                    .map(|v| v.summary_label())
                    .collect(),
            },
            1 => Lookup::Found(hits[0]),
            _ => Lookup::Ambiguous { matches: hits },
        }
    }

    pub fn other_versions(&self, standard: &str, except_version: &str) -> Vec<&StandardVersion> {
        self.versions
            .iter()
            .filter(|v| v.standard == standard && v.version != except_version)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum Lookup<'a> {
    Found(&'a StandardVersion),
    Unknown {
        standard: String,
        version: String,
        others: Vec<String>,
    },
    Ambiguous {
        matches: Vec<&'a StandardVersion>,
    },
}

impl Lookup<'_> {
    pub fn substituted(&self) -> bool {
        false
    }
}

pub fn load_path(path: &Path) -> Result<Registry, ValidationError> {
    validate_path(path)
}

pub fn repo_root_from_registry(registry: &Path) -> PathBuf {
    registry
        .parent()
        .and_then(|standards_dir| standards_dir.parent())
        .unwrap_or_else(|| registry.parent().unwrap_or(Path::new(".")))
        .to_path_buf()
}

pub fn format_list_text(reg: &Registry) -> String {
    let mut out = String::from("HELIX STANDARDS REGISTRY\n\n");
    out.push_str("Default supported-version discovery (OFFICIAL ∩ SUPPORTED):\n");
    let supported = reg.official_supported();
    if supported.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for v in &supported {
            out.push_str(&format!("  {}\n", v.pack_id));
        }
    }
    out.push_str(
        "\nA GitHub tag alone does not make a version supported.\n\
         Ballot and snapshot rows are never in default discovery.\n\
         Default helix verify does not select a registry pack.\n\
         Helix does not download GA4GH files at runtime.\n\
         Not GA4GH certification. Not HELIOS.\n\n\
         All rows:\n",
    );
    for v in &reg.versions {
        out.push('\n');
        out.push_str(&format_version_text(v));
    }
    out
}

pub fn format_version_text(v: &StandardVersion) -> String {
    let mut out = format!("{}\n", v.pack_id);
    out.push_str(&format!("  standard:        {}\n", v.standard));
    out.push_str(&format!("  product:         {}\n", v.product));
    out.push_str(&format!("  version:         {}\n", v.version));
    out.push_str(&format!(
        "  release_class:   {}\n",
        v.release_class.as_str()
    ));
    out.push_str(&format!(
        "  support_status:  {}\n",
        v.support_status.as_str()
    ));
    out.push_str(&format!(
        "  supported:       {}\n",
        if evaluate_support(v, None).supported {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(&format!("  repository:      {}\n", v.repository));
    out.push_str(&format!("  release_ref:     {}\n", v.release_ref));
    out.push_str(&format!("  commit:          {}\n", v.commit));
    out.push_str(&format!("  retrieved_at:    {}\n", v.retrieved_at));
    if let Some(pi) = &v.pack_integrity {
        out.push_str(&format!(
            "  pack_integrity:  {}: {}\n",
            pi.algorithm, pi.hex
        ));
    }
    if let Some(se) = &v.schema_entry {
        out.push_str(&format!("  schema_entry:    {se}\n"));
    }
    if let Some(sc) = &v.schema_component {
        out.push_str(&format!("  schema_component: {sc}\n"));
    }
    if let Some(cid) = &v.catalog_id {
        out.push_str(&format!("  catalog_id:      {cid}\n"));
    }
    if let Some(bid) = &v.binding_id {
        out.push_str(&format!("  binding_id:      {bid}\n"));
    }
    out.push_str(&format!("  checker_id:      {}\n", declared_checker_id()));
    if let Some(cov) = &v.coverage {
        out.push_str(&format!(
            "  coverage:        schema={} behavior={} security={} interoperability={}\n",
            cov.schema.as_str(),
            cov.behavior.as_str(),
            cov.security.as_str(),
            cov.interoperability.as_str()
        ));
        if let Some(notes) = &cov.notes {
            out.push_str(&format!("  coverage_notes:  {notes}\n"));
        }
    }
    if let Some(bindings) = &v.test_bindings {
        for b in bindings {
            out.push_str(&format!(
                "  binding:         {} {} {}\n",
                b.id,
                b.code,
                b.kind.as_str()
            ));
        }
    }
    for src in &v.normative_sources {
        out.push_str(&format!("  source:          {}\n", src.path));
        out.push_str(&format!("  source_url:      {}\n", src.source_url));
        out.push_str(&format!(
            "  integrity:       {}: {}\n",
            src.integrity.algorithm, src.integrity.hex
        ));
        if let Some(vp) = &src.vendor_path {
            out.push_str(&format!("  vendor:          {vp}\n"));
        }
    }
    if let Some(notes) = &v.notes {
        out.push_str(&format!("  notes:           {notes}\n"));
    }
    out
}

pub fn format_show_text(standard: &str, version: &str, lookup: &Lookup<'_>) -> String {
    let mut out = String::from("HELIX STANDARDS\n\n");
    out.push_str(&format!("query:        {standard} {version}\n"));
    match lookup {
        Lookup::Found(v) => {
            let result = if evaluate_support(v, None).supported {
                "supported"
            } else {
                "available_not_supported"
            };
            out.push_str(&format!("result:       {result}\n"));
            out.push_str(&format!(
                "supported:    {}\n",
                if evaluate_support(v, None).supported {
                    "yes"
                } else {
                    "no"
                }
            ));
            out.push_str("substituted:  no\n\n");
            out.push_str(&format_version_text(v));
            if !evaluate_support(v, None).supported {
                out.push_str(
                    "\nThis version is not SUPPORTED in Helix.\n\
                     A repository tag alone does not make a version supported.\n\
                     Helix did not substitute another version.\n",
                );
            }
        }
        Lookup::Unknown { others, .. } => {
            out.push_str("result:       unknown_to_helix\n");
            out.push_str("supported:    no\n");
            out.push_str("substituted:  no\n\n");
            out.push_str(
                "This version is not in the Helix registry.\n\
                 A repository tag alone does not make a version supported.\n\
                 Helix did not substitute another version.\n",
            );
            if !others.is_empty() {
                out.push_str("\nOther rows for this standard (not selected):\n");
                for label in others {
                    out.push_str(&format!("  {label}\n"));
                }
            }
        }
        Lookup::Ambiguous { matches } => {
            out.push_str("result:       ambiguous\n");
            out.push_str("supported:    no\n");
            out.push_str("substituted:  no\n\n");
            out.push_str(
                "Multiple registry rows match this standard and version.\n\
                 Pass --release-class. Helix did not substitute another version.\n",
            );
            for v in matches {
                out.push_str(&format!("  {}\n", v.summary_label()));
            }
        }
    }
    out
}

pub fn list_json(reg: &Registry) -> serde_json::Value {
    serde_json::json!({
        "schema_version": REGISTRY_SCHEMA_VERSION,
        "official_supported": reg.official_supported().iter().map(|v| v.pack_id.clone()).collect::<Vec<_>>(),
        "substituted": false,
        "versions": reg.versions,
        "notes": [
            "Default supported-version discovery is OFFICIAL intersect SUPPORTED only.",
            "A GitHub tag alone does not make a version supported.",
            "Default helix verify does not select a registry pack.",
            "Helix does not download GA4GH files at runtime.",
        ]
    })
}

pub fn show_json(standard: &str, version: &str, lookup: &Lookup<'_>) -> serde_json::Value {
    match lookup {
        Lookup::Found(v) => serde_json::json!({
            "query": { "standard": standard, "version": version },
            "result": if evaluate_support(v, None).supported { "supported" } else { "available_not_supported" },
            "supported": evaluate_support(v, None).supported,
            "substituted": false,
            "record": v,
        }),
        Lookup::Unknown { others, .. } => serde_json::json!({
            "query": { "standard": standard, "version": version },
            "result": "unknown_to_helix",
            "supported": false,
            "substituted": false,
            "record": serde_json::Value::Null,
            "other_rows_not_selected": others,
        }),
        Lookup::Ambiguous { matches } => serde_json::json!({
            "query": { "standard": standard, "version": version },
            "result": "ambiguous",
            "supported": false,
            "substituted": false,
            "record": serde_json::Value::Null,
            "matches": matches.iter().map(|v| v.pack_id.clone()).collect::<Vec<_>>(),
        }),
    }
}
