// SPDX-License-Identifier: Apache-2.0
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use jsonschema::JSONSchema;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::model::{BindingKind, ReleaseClass, StandardVersion, SupportStatus, VersionSource};
use super::{repo_root_from_registry, Registry, REGISTRY_SCHEMA_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationKind {
    InvalidRegistry,
    MissingCommit,
    DuplicateVersion,
    UnknownReleaseClass,
    InvalidSource,
    IntegrityMismatch,
}

impl ValidationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRegistry => "invalid_registry",
            Self::MissingCommit => "missing_commit",
            Self::DuplicateVersion => "duplicate_version",
            Self::UnknownReleaseClass => "unknown_release_class",
            Self::InvalidSource => "invalid_source",
            Self::IntegrityMismatch => "integrity_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub kind: ValidationKind,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

fn version_schema() -> &'static JSONSchema {
    static SCHEMA: OnceLock<JSONSchema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let json: Value =
            serde_json::from_str(include_str!("../../schemas/helix-standard-version-v1.json"))
                .expect("helix-standard-version-v1.json");
        let leaked: &'static Value = Box::leak(Box::new(json));
        JSONSchema::compile(leaked).expect("helix-standard-version-v1 compiles")
    })
}

pub fn validate_path(registry_path: &Path) -> Result<Registry, ValidationError> {
    let text = std::fs::read_to_string(registry_path).map_err(|e| ValidationError {
        kind: ValidationKind::InvalidRegistry,
        message: format!("cannot read {}: {e}", registry_path.display()),
    })?;
    let root = repo_root_from_registry(registry_path);
    validate_yaml(&text, Some(&root))
}

pub fn validate_yaml(text: &str, repo_root: Option<&Path>) -> Result<Registry, ValidationError> {
    let doc: Value = serde_yaml::from_str(text).map_err(|e| ValidationError {
        kind: ValidationKind::InvalidRegistry,
        message: format!("registry YAML is not valid: {e}"),
    })?;
    validate_document(&doc, repo_root)
}

pub fn validate_loaded(reg: &Registry, repo_root: Option<&Path>) -> Result<(), ValidationError> {
    let value = serde_json::to_value(reg).map_err(|e| ValidationError {
        kind: ValidationKind::InvalidRegistry,
        message: e.to_string(),
    })?;
    validate_document(&value, repo_root).map(|_| ())
}

fn validate_document(doc: &Value, repo_root: Option<&Path>) -> Result<Registry, ValidationError> {
    if doc.get("schema_version").and_then(|v| v.as_str()) != Some(REGISTRY_SCHEMA_VERSION) {
        return Err(ValidationError {
            kind: ValidationKind::InvalidRegistry,
            message: format!("registry schema_version must be {REGISTRY_SCHEMA_VERSION}"),
        });
    }
    let Some(versions) = doc.get("versions").and_then(|v| v.as_array()) else {
        return Err(ValidationError {
            kind: ValidationKind::InvalidRegistry,
            message: "registry must have a versions array".into(),
        });
    };
    for (i, rec) in versions.iter().enumerate() {
        if rec.get("commit").is_none() {
            return Err(ValidationError {
                kind: ValidationKind::MissingCommit,
                message: format!("versions[{i}] missing commit (releases must be pinned)"),
            });
        }
        if let Some(class) = rec.get("release_class") {
            if let Some(s) = class.as_str() {
                if !matches!(s, "official" | "ballot" | "snapshot" | "development") {
                    return Err(ValidationError {
                        kind: ValidationKind::UnknownReleaseClass,
                        message: format!(
                            "versions[{i}] unknown release_class {s:?} (official|ballot|snapshot|development)"
                        ),
                    });
                }
            }
        }
        if let Err(errors) = version_schema().validate(rec) {
            let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(map_schema_errors(i, rec, &msgs));
        }
    }

    let registry: Registry = serde_json::from_value(doc.clone())
        .map_err(|e| classify_deserialize(doc, &e.to_string()))?;
    extra_checks(&registry, repo_root)?;
    Ok(registry)
}

fn classify_deserialize(doc: &Value, serde_msg: &str) -> ValidationError {
    let versions = doc.get("versions").and_then(|v| v.as_array());
    if let Some(arr) = versions {
        for (i, v) in arr.iter().enumerate() {
            if v.get("commit").is_none() {
                return ValidationError {
                    kind: ValidationKind::MissingCommit,
                    message: format!("versions[{i}] missing commit"),
                };
            }
            if let Some(class) = v.get("release_class").and_then(|c| c.as_str()) {
                if !matches!(class, "official" | "ballot" | "snapshot" | "development") {
                    return ValidationError {
                        kind: ValidationKind::UnknownReleaseClass,
                        message: format!("versions[{i}] unknown release_class {class:?}"),
                    };
                }
            }
        }
    }
    ValidationError {
        kind: ValidationKind::InvalidRegistry,
        message: serde_msg.to_string(),
    }
}

fn map_schema_errors(index: usize, rec: &Value, msgs: &[String]) -> ValidationError {
    let joined = msgs.join("; ");
    if rec.get("commit").is_none() || (joined.contains("commit") && joined.contains("required")) {
        return ValidationError {
            kind: ValidationKind::MissingCommit,
            message: format!("versions[{index}] missing commit: {joined}"),
        };
    }
    if joined.contains("release_class") {
        return ValidationError {
            kind: ValidationKind::UnknownReleaseClass,
            message: format!("versions[{index}] {joined}"),
        };
    }
    if joined.contains("repository") {
        return ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!("versions[{index}] {joined}"),
        };
    }
    ValidationError {
        kind: ValidationKind::InvalidRegistry,
        message: format!("versions[{index}] schema: {joined}"),
    }
}

fn extra_checks(reg: &Registry, repo_root: Option<&Path>) -> Result<(), ValidationError> {
    let mut pack_ids = HashSet::new();
    let mut triples = HashSet::new();
    for v in &reg.versions {
        if !pack_ids.insert(v.pack_id.clone()) {
            return Err(ValidationError {
                kind: ValidationKind::DuplicateVersion,
                message: format!("duplicate pack_id {}", v.pack_id),
            });
        }
        let triple = (
            v.standard.clone(),
            v.version.clone(),
            v.release_class.as_str().to_string(),
        );
        if !triples.insert(triple) {
            return Err(ValidationError {
                kind: ValidationKind::DuplicateVersion,
                message: format!(
                    "duplicate version {} {} {}",
                    v.standard,
                    v.version,
                    v.release_class.as_str()
                ),
            });
        }
        check_version(v, repo_root)?;
    }
    Ok(())
}

fn check_version(v: &StandardVersion, repo_root: Option<&Path>) -> Result<(), ValidationError> {
    if !is_authoritative_repository(&v.repository) {
        return Err(ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!(
                "{} repository is not an authoritative GA4GH spec repo: {}",
                v.pack_id, v.repository
            ),
        });
    }
    if v.release_class != ReleaseClass::Development && is_forbidden_release_ref(&v.release_ref) {
        return Err(ValidationError {
            kind: ValidationKind::InvalidRegistry,
            message: format!(
                "{} release_ref {:?} is not an immutable release (no HEAD/main/develop)",
                v.pack_id, v.release_ref
            ),
        });
    }
    if v.release_class == ReleaseClass::Development && v.support_status == SupportStatus::Supported
    {
        return Err(ValidationError {
            kind: ValidationKind::InvalidRegistry,
            message: format!("{} DEVELOPMENT cannot be supported", v.pack_id),
        });
    }
    if v.support_status == SupportStatus::Supported {
        let bindings = v.test_bindings.as_ref().map(|b| b.len()).unwrap_or(0);
        if bindings == 0 || v.fixture_catalog.is_none() {
            return Err(ValidationError {
                kind: ValidationKind::InvalidRegistry,
                message: format!(
                    "{} is supported but missing test_bindings/fixture_catalog (a tag is not enough)",
                    v.pack_id
                ),
            });
        }
        for src in &v.normative_sources {
            if src.vendor_path.as_deref().unwrap_or("").is_empty() {
                return Err(ValidationError {
                    kind: ValidationKind::InvalidRegistry,
                    message: format!(
                        "{} is supported but normative source {} has no vendor_path (Helix must load pinned local bytes)",
                        v.pack_id, src.path
                    ),
                });
            }
        }
        for b in v.test_bindings.as_deref().unwrap_or(&[]) {
            if b.kind == BindingKind::Normative {
                let Some(c) = &b.citation else {
                    return Err(ValidationError {
                        kind: ValidationKind::InvalidRegistry,
                        message: format!(
                            "{} normative binding {} needs a citation",
                            v.pack_id, b.id
                        ),
                    });
                };
                if !v.normative_sources.iter().any(|s| s.path == c.source_path) {
                    return Err(ValidationError {
                        kind: ValidationKind::InvalidRegistry,
                        message: format!(
                            "{} normative binding {} source file {} is not in normative_sources",
                            v.pack_id, b.id, c.source_path
                        ),
                    });
                }
            }
        }
        if let Err(msg) = super::support::require_supported(v, repo_root) {
            return Err(ValidationError {
                kind: ValidationKind::InvalidRegistry,
                message: msg,
            });
        }
    }
    for src in &v.normative_sources {
        check_source(v, src, repo_root)?;
    }
    if v.schema_entry.is_some() {
        if v.pack_integrity.is_none() || v.schema_component.as_deref() != Some("DrsObject") {
            return Err(ValidationError {
                kind: ValidationKind::InvalidRegistry,
                message: format!(
                    "{} schema_entry requires pack_integrity and schema_component DrsObject",
                    v.pack_id
                ),
            });
        }
        if let Some(root) = repo_root {
            super::pack::load_pack(v, root).map_err(|e| ValidationError {
                kind: ValidationKind::IntegrityMismatch,
                message: e.to_string(),
            })?;
        }
    }
    Ok(())
}

fn check_source(
    v: &StandardVersion,
    src: &VersionSource,
    repo_root: Option<&Path>,
) -> Result<(), ValidationError> {
    if !src.source_url.starts_with("https://") {
        return Err(ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!("{} source_url must be https", v.pack_id),
        });
    }
    if is_mutable_source_url(&src.source_url) {
        return Err(ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!(
                "{} source_url must not fetch HEAD/main/develop (got {})",
                v.pack_id, src.source_url
            ),
        });
    }
    if !src.source_url.contains(&v.commit) {
        return Err(ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!(
                "{} source_url does not contain pinned commit {} (unpinned Pages/HEAD is not allowed)",
                v.pack_id, v.commit
            ),
        });
    }
    if src.integrity.algorithm != "sha256" || src.integrity.hex.len() != 64 {
        return Err(ValidationError {
            kind: ValidationKind::IntegrityMismatch,
            message: format!(
                "{} integrity must be sha256 with 64 lowercase hex chars",
                v.pack_id
            ),
        });
    }
    let Some(vendor_path) = &src.vendor_path else {
        return Ok(());
    };
    let Some(root) = repo_root else {
        return Ok(());
    };
    let file = confined_vendor_file(root, vendor_path)?;
    let bytes = std::fs::read(&file).map_err(|e| ValidationError {
        kind: ValidationKind::IntegrityMismatch,
        message: format!("{} missing vendor file {}: {e}", v.pack_id, file.display()),
    })?;
    let actual = hex_sha256(&bytes);
    if actual != src.integrity.hex {
        return Err(ValidationError {
            kind: ValidationKind::IntegrityMismatch,
            message: format!(
                "{} vendor {} hash mismatch: registry {} file {actual}. Helix did not fetch a replacement.",
                v.pack_id,
                vendor_path,
                src.integrity.hex
            ),
        });
    }
    Ok(())
}

/// Registry `vendor_path` is local provenance, not a target URL. Still refuse `..` /
/// absolute paths so a malicious registry cannot make Helix read arbitrary files.
pub fn confined_vendor_file(root: &Path, vendor_path: &str) -> Result<PathBuf, ValidationError> {
    let p = Path::new(vendor_path);
    if p.is_absolute() {
        return Err(ValidationError {
            kind: ValidationKind::InvalidSource,
            message: format!("vendor_path must be relative (got {vendor_path})"),
        });
    }
    for c in p.components() {
        match c {
            Component::Normal(_) | Component::CurDir => {}
            _ => {
                return Err(ValidationError {
                    kind: ValidationKind::InvalidSource,
                    message: format!(
                        "vendor_path must not contain '..' or other non-normal components (got {vendor_path})"
                    ),
                });
            }
        }
    }
    Ok(root.join(p))
}

pub fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn is_authoritative_repository(url: &str) -> bool {
    url == "https://github.com/samtools/hts-specs"
        || url.starts_with("https://github.com/ga4gh/")
        || url.starts_with("https://github.com/ga4gh-beacon/")
}

fn is_forbidden_release_ref(r: &str) -> bool {
    matches!(
        r,
        "HEAD"
            | "head"
            | "main"
            | "master"
            | "develop"
            | "origin/main"
            | "origin/master"
            | "origin/develop"
    )
}

fn is_mutable_source_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("/head/")
        || lower.contains("/head?")
        || lower.ends_with("/head")
        || lower.contains("/main/")
        || lower.contains("/master/")
        || lower.contains("/develop/")
}
