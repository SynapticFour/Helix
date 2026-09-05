// SPDX-License-Identifier: Apache-2.0
//! Local-only GA4GH execution pack loader.
//!
//! Never fetches specifications. Never calls HelixTest. Integrity failure must
//! stop before `*_with_spec`. Not HELIOS. Not certification.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use common::spec_source::{
    resolve_schema_value, sha256_hex, sha256_manifest_v1, SpecCompileResult, SpecSource,
};

use super::model::StandardVersion;
use super::validate::{confined_vendor_file, hex_sha256};

/// Files required to compile DRS `DrsObject` from the vendored openapi/ tree.
pub const DRS_OBJECT_CLOSURE: &[&str] = &[
    "openapi/components/schemas/DrsObject.yaml",
    "openapi/components/schemas/Checksum.yaml",
    "openapi/components/schemas/AccessMethod.yaml",
    "openapi/components/schemas/AccessURL.yaml",
    "openapi/components/schemas/Authorizations.yaml",
    "openapi/components/schemas/ContentsObject.yaml",
];

pub const MANIFEST_ALG: &str = "sha256-manifest-v1";

#[derive(Debug, Clone)]
pub struct LoadedPack {
    pub pack_id: String,
    pub version: String,
    pub commit: String,
    pub pack_integrity_sha256: String,
    pub spec: SpecSource,
    pub expected: SpecCompileResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackLoadError {
    NotExecutable { pack_id: String, reason: String },
    Integrity { message: String },
    Incomplete { message: String },
    Path { message: String },
}

impl std::fmt::Display for PackLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotExecutable { pack_id, reason } => {
                write!(f, "pack {pack_id} is not executable: {reason}")
            }
            Self::Integrity { message } | Self::Incomplete { message } | Self::Path { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for PackLoadError {}

/// Load a pack from local vendor bytes. Does not invoke HelixTest.
pub fn load_pack(v: &StandardVersion, repo_root: &Path) -> Result<LoadedPack, PackLoadError> {
    let schema_entry = v.schema_entry.as_deref().unwrap_or("");
    let schema_component = v.schema_component.as_deref().unwrap_or("");
    if v.standard != "drs"
        || schema_entry != "openapi/components/schemas/DrsObject.yaml"
        || schema_component != "DrsObject"
    {
        return Err(PackLoadError::NotExecutable {
            pack_id: v.pack_id.clone(),
            reason: if v.standard == "wes" {
                "WES ServiceInfo has an HTTPS $ref to ga4gh-service-info; that is not a hashed local pin"
                    .into()
            } else {
                "schema_entry/schema_component do not identify a local DrsObject closure".into()
            },
        });
    }
    let Some(declared) = v.pack_integrity.as_ref() else {
        return Err(PackLoadError::NotExecutable {
            pack_id: v.pack_id.clone(),
            reason: "missing pack_integrity".into(),
        });
    };
    if declared.algorithm != MANIFEST_ALG {
        return Err(PackLoadError::Integrity {
            message: format!(
                "{} pack_integrity.algorithm must be {MANIFEST_ALG}",
                v.pack_id
            ),
        });
    }

    let mut files: BTreeMap<String, Arc<[u8]>> = BTreeMap::new();
    for src in &v.normative_sources {
        let Some(vendor_path) = src.vendor_path.as_deref() else {
            return Err(PackLoadError::Incomplete {
                message: format!("{} source {} has no vendor_path", v.pack_id, src.path),
            });
        };
        let file = confined_vendor_file(repo_root, vendor_path)
            .map_err(|e| PackLoadError::Path { message: e.message })?;
        let bytes = std::fs::read(&file).map_err(|e| PackLoadError::Incomplete {
            message: format!("{} missing vendor file {}: {e}", v.pack_id, file.display()),
        })?;
        let actual = hex_sha256(&bytes);
        if actual != src.integrity.hex {
            return Err(PackLoadError::Integrity {
                message: format!(
                    "{} vendor {} hash mismatch: registry {} file {actual}",
                    v.pack_id, vendor_path, src.integrity.hex
                ),
            });
        }
        if files.insert(src.path.clone(), Arc::from(bytes)).is_some() {
            return Err(PackLoadError::Incomplete {
                message: format!("{} duplicate source path {}", v.pack_id, src.path),
            });
        }
    }

    let computed = pack_integrity_hex(&files);
    if computed != declared.hex {
        return Err(PackLoadError::Integrity {
            message: format!(
                "{} pack_integrity mismatch: registry {} computed {computed}",
                v.pack_id, declared.hex
            ),
        });
    }

    for req in DRS_OBJECT_CLOSURE {
        if !files.contains_key(*req) {
            return Err(PackLoadError::Incomplete {
                message: format!("{} missing DrsObject closure file {req}", v.pack_id),
            });
        }
    }

    let spec = SpecSource {
        schema_entry: schema_entry.to_string(),
        schema_component: schema_component.to_string(),
        files,
    };
    let (_value, expected) =
        resolve_schema_value(&spec).map_err(|e| PackLoadError::Incomplete {
            message: format!("{} SpecSource resolve failed: {e}", v.pack_id),
        })?;

    Ok(LoadedPack {
        pack_id: v.pack_id.clone(),
        version: v.version.clone(),
        commit: v.commit.clone(),
        pack_integrity_sha256: computed,
        spec,
        expected,
    })
}

pub fn pack_integrity_hex(files: &BTreeMap<String, Arc<[u8]>>) -> String {
    sha256_manifest_v1(files.iter().map(|(p, b)| (p.as_str(), b.as_ref())))
}

/// Fail closed if the checker did not compile the bytes Helix passed.
pub fn compare_spec_identity(
    expected: &SpecCompileResult,
    returned: &SpecCompileResult,
) -> Result<()> {
    if expected.schema_document_sha256 != returned.schema_document_sha256 {
        bail!(
            "schema_document_sha256 mismatch: expected {} returned {}",
            expected.schema_document_sha256,
            returned.schema_document_sha256
        );
    }
    if expected.schema_component_sha256 != returned.schema_component_sha256 {
        bail!(
            "schema_component_sha256 mismatch: expected {} returned {}",
            expected.schema_component_sha256,
            returned.schema_component_sha256
        );
    }
    if expected.files_opened != returned.files_opened {
        bail!(
            "files_opened mismatch: expected {:?} returned {:?}",
            expected.files_opened,
            returned.files_opened
        );
    }
    Ok(())
}

/// Deterministic spec-join identity. No timestamps, paths, or target URLs.
/// Target-scoped run identity is `target_execution_id` (`src/target.rs`).
pub fn execution_id(
    pack_id: &str,
    pack_integrity_sha256: &str,
    schema_document_sha256: &str,
    schema_component_sha256: &str,
    checker_id: &str,
    schema_entry: &str,
    schema_component: &str,
) -> String {
    let canonical = format!(
        "pack_id={pack_id}\n\
         pack_integrity_sha256={pack_integrity_sha256}\n\
         schema_document_sha256={schema_document_sha256}\n\
         schema_component_sha256={schema_component_sha256}\n\
         checker_id={checker_id}\n\
         schema_entry={schema_entry}\n\
         schema_component={schema_component}\n"
    );
    sha256_hex(canonical.as_bytes())
}

pub fn checker_id(tag: &str, sha: &str) -> String {
    format!("{tag}:{sha}")
}

pub fn helix_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Copy a vendor pack tree into `dest_root/standards/vendor/{pack_id}/` for mutation tests.
pub fn copy_pack_tree(repo_root: &Path, pack_id: &str, dest_root: &Path) -> Result<PathBuf> {
    let src = repo_root.join("standards/vendor").join(pack_id);
    let dest = dest_root.join("standards/vendor").join(pack_id);
    copy_dir(&src, &dest)?;
    Ok(dest)
}

fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).with_context(|| format!("mkdir {}", dest.display()))?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standards::load_path;

    fn shipped_drs(pack_id: &str) -> StandardVersion {
        let reg = load_path(&crate::standards::default_registry_path()).unwrap();
        reg.versions
            .into_iter()
            .find(|v| v.pack_id == pack_id)
            .unwrap()
    }

    #[test]
    fn drs_1_4_0_loads_from_vendor() {
        let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
        assert_eq!(loaded.pack_id, "ga4gh.drs.1.4.0");
        assert_eq!(loaded.spec.schema_component, "DrsObject");
        for f in DRS_OBJECT_CLOSURE {
            assert!(loaded.spec.files.contains_key(*f), "{f}");
        }
        assert!(loaded
            .expected
            .files_opened
            .iter()
            .any(|p| p.ends_with("DrsObject.yaml")));
        let mut expected_closure: Vec<String> = DRS_OBJECT_CLOSURE
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        expected_closure.sort();
        assert_eq!(loaded.expected.files_opened, expected_closure);
    }

    #[test]
    fn drs_1_5_0_loads_from_vendor() {
        load_pack(&shipped_drs("ga4gh.drs.1.5.0"), &helix_repo_root()).unwrap();
    }

    #[test]
    fn wes_is_not_executable() {
        let err = load_pack(&shipped_drs("ga4gh.wes.1.1.0"), &helix_repo_root()).unwrap_err();
        match err {
            PackLoadError::NotExecutable { pack_id, reason } => {
                assert_eq!(pack_id, "ga4gh.wes.1.1.0");
                assert!(
                    reason.contains("HTTPS") || reason.contains("WES"),
                    "{reason}"
                );
            }
            other => panic!("{other}"),
        }
    }

    #[test]
    fn execution_id_is_deterministic_and_sensitive() {
        let a = execution_id("p", "1", "2", "3", "c", "e", "DrsObject");
        let b = execution_id("p", "1", "2", "3", "c", "e", "DrsObject");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        let c = execution_id("p", "1", "2", "9", "c", "e", "DrsObject");
        assert_ne!(a, c);
    }
}
