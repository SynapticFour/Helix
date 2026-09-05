// SPDX-License-Identifier: Apache-2.0
//! Fail-closed GA4GH pack selection for `helix verify`.
//!
//! Never substitutes another version. A GitHub tag is not SUPPORTED.
//! AVAILABLE rows are never executed. Development is never selected.
//! Default `helix verify` (no --standard) does not use this module.
//! Not HELIOS. Not certification.

#![allow(clippy::result_large_err)]

use super::model::{ReleaseClass, StandardVersion, SupportStatus};
use super::{Lookup, Registry};

/// Exact status string required when a registry row exists but Helix has no pack.
pub const AVAILABLE_BUT_NOT_SUPPORTED: &str = "AVAILABLE_BUT_NOT_SUPPORTED";
pub const UNKNOWN_TO_HELIX: &str = "UNKNOWN_TO_HELIX";
pub const NO_OFFICIAL_SUPPORTED: &str = "NO_OFFICIAL_SUPPORTED";
pub const DEVELOPMENT_NOT_SELECTABLE: &str = "DEVELOPMENT_NOT_SELECTABLE";
pub const AMBIGUOUS: &str = "AMBIGUOUS";
pub const NEEDS_RELEASE_CLASS: &str = "NEEDS_RELEASE_CLASS";
pub const SELECTED: &str = "SELECTED";
pub const INSUFFICIENT: &str = "INSUFFICIENT";
pub const UNVERSIONED: &str = "UNVERSIONED";
pub const MULTIPLE_PACKS_NOT_EXECUTABLE: &str = "MULTIPLE_PACKS_NOT_EXECUTABLE";
pub const NOT_SUPPORTED: &str = "NOT_SUPPORTED";

/// Owned copy of the registry row involved in a selection decision.
/// Presence of this struct does **not** mean Helix selected or verified it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRef {
    pub pack_id: String,
    pub standard: String,
    pub version: String,
    pub commit: String,
    pub support_status: SupportStatus,
    pub release_class: ReleaseClass,
}

impl PackRef {
    pub fn from_version(v: &StandardVersion) -> Self {
        Self {
            pack_id: v.pack_id.clone(),
            standard: v.standard.clone(),
            version: v.version.clone(),
            commit: v.commit.clone(),
            support_status: v.support_status,
            release_class: v.release_class,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    AvailableButNotSupported {
        pack: PackRef,
        others_not_selected: Vec<String>,
    },
    Unknown {
        standard: String,
        version: String,
        others_not_selected: Vec<String>,
    },
    NoOfficialSupported {
        standard: String,
    },
    DevelopmentNotSelectable {
        standard: String,
        version: Option<String>,
    },
    Ambiguous {
        standard: String,
        version: String,
        pack_ids: Vec<String>,
    },
    NeedsReleaseClass {
        standard: String,
        version: String,
        pack_ids: Vec<String>,
    },
    Insufficient {
        standard: String,
    },
    NotSupported {
        pack: PackRef,
        reasons: Vec<String>,
        others_not_selected: Vec<String>,
    },
}

impl SelectionError {
    pub fn status_code(&self) -> &'static str {
        match self {
            Self::AvailableButNotSupported { .. } => AVAILABLE_BUT_NOT_SUPPORTED,
            Self::Unknown { .. } => UNKNOWN_TO_HELIX,
            Self::NoOfficialSupported { .. } => NO_OFFICIAL_SUPPORTED,
            Self::DevelopmentNotSelectable { .. } => DEVELOPMENT_NOT_SELECTABLE,
            Self::Ambiguous { .. } => AMBIGUOUS,
            Self::NeedsReleaseClass { .. } => NEEDS_RELEASE_CLASS,
            Self::Insufficient { .. } => INSUFFICIENT,
            Self::NotSupported { .. } => NOT_SUPPORTED,
        }
    }

    pub fn substituted(&self) -> bool {
        false
    }

    /// Registry row Helix looked at, if any. Not a selected/verified pack.
    pub fn registry_pack(&self) -> Option<&PackRef> {
        match self {
            Self::AvailableButNotSupported { pack, .. } | Self::NotSupported { pack, .. } => {
                Some(pack)
            }
            _ => None,
        }
    }

    pub fn other_rows_not_selected(&self) -> Vec<String> {
        match self {
            Self::AvailableButNotSupported {
                others_not_selected,
                ..
            }
            | Self::NotSupported {
                others_not_selected,
                ..
            }
            | Self::Unknown {
                others_not_selected,
                ..
            } => others_not_selected.clone(),
            Self::Ambiguous { pack_ids, .. } | Self::NeedsReleaseClass { pack_ids, .. } => {
                pack_ids.clone()
            }
            _ => Vec::new(),
        }
    }

    pub fn skip_message(&self) -> String {
        match self {
            Self::AvailableButNotSupported { pack, .. } => format!(
                "GA4GH {} {} is AVAILABLE in the Helix registry but not SUPPORTED ({AVAILABLE_BUT_NOT_SUPPORTED}). \
                 Helix did not substitute another version. A GitHub tag alone does not make a version supported. \
                 Checks not executed (not a pass).",
                pack.standard, pack.version
            ),
            Self::Unknown {
                standard, version, ..
            } => format!(
                "GA4GH {standard} {version} is unknown to Helix ({UNKNOWN_TO_HELIX}). \
                 Helix did not substitute another version. Checks not executed (not a pass)."
            ),
            Self::NoOfficialSupported { standard } => format!(
                "Helix has no official SUPPORTED pack for {standard} ({NO_OFFICIAL_SUPPORTED}). \
                 AVAILABLE registry rows are not executed. Helix did not substitute another version. \
                 Checks not executed (not a pass)."
            ),
            Self::DevelopmentNotSelectable { standard, version } => {
                let v = version.as_deref().unwrap_or("");
                format!(
                    "Development release class is never selectable ({DEVELOPMENT_NOT_SELECTABLE}) \
                     ({standard} {v}). Helix did not substitute another version. Checks not executed (not a pass)."
                )
            }
            Self::Ambiguous {
                standard, version, ..
            } => format!(
                "Multiple registry rows match {standard} {version} ({AMBIGUOUS}). \
                 Pass --release-class. Helix did not substitute another version. Checks not executed (not a pass)."
            ),
            Self::NeedsReleaseClass {
                standard, version, ..
            } => format!(
                "{standard} {version} is not an official registry row. Pass --release-class ({NEEDS_RELEASE_CLASS}). \
                 Helix did not substitute another version. Checks not executed (not a pass)."
            ),
            Self::Insufficient { standard } => format!(
                "Target did not declare a usable {standard} version ({INSUFFICIENT}). \
                 Helix did not select an official SUPPORTED pack. Use --standard and --version. \
                 Checks not executed (not a pass)."
            ),
            Self::NotSupported { pack, reasons, .. } => format!(
                "GA4GH {} {} is declared supported in YAML but the executable support gate failed ({NOT_SUPPORTED}): {}. \
                 Helix did not substitute another version. Checks not executed (not a pass).",
                pack.standard,
                pack.version,
                reasons.join("; ")
            ),
        }
    }
}

impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.skip_message())
    }
}

impl std::error::Error for SelectionError {}

/// Mode 1: exact standard + version. Default class is official. No substitution.
pub fn select_explicit(
    reg: &Registry,
    standard: &str,
    version: &str,
    release_class: Option<ReleaseClass>,
) -> Result<PackRef, SelectionError> {
    if release_class == Some(ReleaseClass::Development) {
        return Err(SelectionError::DevelopmentNotSelectable {
            standard: standard.to_string(),
            version: Some(version.to_string()),
        });
    }
    let class = release_class.unwrap_or(ReleaseClass::Official);
    match reg.lookup(standard, version, Some(class)) {
        Lookup::Found(v) => pack_if_supported(reg, v),
        Lookup::Unknown { others, .. } => {
            // A ballot/snapshot row for the same version is not a silent official substitute.
            match reg.lookup(standard, version, None) {
                Lookup::Found(other) if other.release_class != class => {
                    Err(SelectionError::NeedsReleaseClass {
                        standard: standard.to_string(),
                        version: version.to_string(),
                        pack_ids: vec![other.pack_id.clone()],
                    })
                }
                Lookup::Ambiguous { matches } => Err(SelectionError::Ambiguous {
                    standard: standard.to_string(),
                    version: version.to_string(),
                    pack_ids: matches.iter().map(|m| m.pack_id.clone()).collect(),
                }),
                _ => Err(SelectionError::Unknown {
                    standard: standard.to_string(),
                    version: version.to_string(),
                    others_not_selected: others,
                }),
            }
        }
        Lookup::Ambiguous { matches } => Err(SelectionError::Ambiguous {
            standard: standard.to_string(),
            version: version.to_string(),
            pack_ids: matches.iter().map(|m| m.pack_id.clone()).collect(),
        }),
    }
}

fn pack_if_supported(reg: &Registry, v: &StandardVersion) -> Result<PackRef, SelectionError> {
    if v.release_class == ReleaseClass::Development {
        return Err(SelectionError::DevelopmentNotSelectable {
            standard: v.standard.clone(),
            version: Some(v.version.clone()),
        });
    }
    if v.support_status != SupportStatus::Supported {
        return Err(SelectionError::AvailableButNotSupported {
            pack: PackRef::from_version(v),
            others_not_selected: other_labels(reg, &v.standard, &v.version),
        });
    }
    let verdict = super::support::evaluate_support(v, None);
    if !verdict.supported {
        return Err(SelectionError::NotSupported {
            pack: PackRef::from_version(v),
            reasons: verdict.reasons,
            others_not_selected: other_labels(reg, &v.standard, &v.version),
        });
    }
    Ok(PackRef::from_version(v))
}

fn other_labels(reg: &Registry, standard: &str, except_version: &str) -> Vec<String> {
    reg.other_versions(standard, except_version)
        .iter()
        .map(|v| v.summary_label())
        .collect()
}

/// Mode 3: every OfficialSupported pack for one standard. AVAILABLE rows are omitted.
pub fn select_all_official_supported(
    reg: &Registry,
    standard: &str,
) -> Result<Vec<PackRef>, SelectionError> {
    let packs: Vec<PackRef> = reg
        .official_supported()
        .into_iter()
        .filter(|v| v.standard == standard)
        .map(PackRef::from_version)
        .collect();
    if packs.is_empty() {
        return Err(SelectionError::NoOfficialSupported {
            standard: standard.to_string(),
        });
    }
    Ok(packs)
}

/// Mode 2: unique OfficialSupported pack whose version equals detected evidence.
/// Empty OfficialSupported is an error, not a licence to run AVAILABLE rows.
/// Insufficient evidence does not pick “latest.”
pub fn select_automatic(
    reg: &Registry,
    standard: &str,
    detected_version: Option<&str>,
) -> Result<PackRef, SelectionError> {
    let official: Vec<&StandardVersion> = reg
        .official_supported()
        .into_iter()
        .filter(|v| v.standard == standard)
        .collect();
    if official.is_empty() {
        return Err(SelectionError::NoOfficialSupported {
            standard: standard.to_string(),
        });
    }
    let Some(detected) = detected_version.map(str::trim).filter(|s| !s.is_empty()) else {
        return Err(SelectionError::Insufficient {
            standard: standard.to_string(),
        });
    };
    if let Some(hit) = official.iter().find(|v| v.version == detected) {
        return Ok(PackRef::from_version(hit));
    }
    match reg.lookup(standard, detected, Some(ReleaseClass::Official)) {
        Lookup::Found(v) if v.support_status == SupportStatus::Available => {
            Err(SelectionError::AvailableButNotSupported {
                pack: PackRef::from_version(v),
                others_not_selected: other_labels(reg, standard, detected),
            })
        }
        _ => Err(SelectionError::Unknown {
            standard: standard.to_string(),
            version: detected.to_string(),
            others_not_selected: official.iter().map(|v| v.summary_label()).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standards::{default_registry_path, load_path};

    fn shipped() -> Registry {
        load_path(&default_registry_path()).expect("shipped registry")
    }

    #[test]
    fn shipped_drs_1_5_0_is_available_but_not_supported() {
        let err = select_explicit(&shipped(), "drs", "1.5.0", None).unwrap_err();
        assert_eq!(err.status_code(), AVAILABLE_BUT_NOT_SUPPORTED);
        assert!(!err.substituted());
        let pack = err.registry_pack().expect("looked-up row");
        assert_eq!(pack.pack_id, "ga4gh.drs.1.5.0");
        assert_eq!(pack.version, "1.5.0");
        assert_eq!(pack.commit, "fe25c3953ae3398a31054d3f9f040d5e27aad517");
        let others = err.other_rows_not_selected();
        assert!(
            others.iter().any(|s| s.contains("ga4gh.drs.1.4.0")),
            "{others:?}"
        );
    }

    #[test]
    fn shipped_drs_1_4_0_is_not_a_substitute_for_1_5_0() {
        let err = select_explicit(&shipped(), "drs", "1.5.0", None).unwrap_err();
        match err {
            SelectionError::AvailableButNotSupported { pack, .. } => {
                assert_ne!(pack.version, "1.4.0");
                assert_eq!(pack.version, "1.5.0");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unknown_version_does_not_select_1_4_0_or_1_5_0() {
        let err = select_explicit(&shipped(), "drs", "1.3.0", None).unwrap_err();
        assert_eq!(err.status_code(), UNKNOWN_TO_HELIX);
        assert!(err.registry_pack().is_none());
        assert!(!err.substituted());
        let others = err.other_rows_not_selected();
        assert!(others.iter().any(|s| s.contains("1.4.0")), "{others:?}");
        assert!(others.iter().any(|s| s.contains("1.5.0")), "{others:?}");
    }

    #[test]
    fn all_supported_selects_only_official_supported_drs_1_4_0() {
        let packs = select_all_official_supported(&shipped(), "drs").expect("1.4.0 supported");
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].pack_id, "ga4gh.drs.1.4.0");
        assert_eq!(packs[0].version, "1.4.0");
        assert!(!packs.iter().any(|p| p.version == "1.5.0"));
    }

    #[test]
    fn automatic_does_not_pick_supported_1_4_0_when_detected_is_missing() {
        let err = select_automatic(&shipped(), "drs", None).unwrap_err();
        assert_eq!(err.status_code(), INSUFFICIENT);
    }

    #[test]
    fn development_class_is_never_selected() {
        let err = select_explicit(&shipped(), "drs", "1.5.0", Some(ReleaseClass::Development))
            .unwrap_err();
        assert_eq!(err.status_code(), DEVELOPMENT_NOT_SELECTABLE);
        assert!(!err.substituted());
    }

    #[test]
    fn wes_1_1_0_is_available_but_not_supported() {
        let err = select_explicit(&shipped(), "wes", "1.1.0", None).unwrap_err();
        assert_eq!(err.status_code(), AVAILABLE_BUT_NOT_SUPPORTED);
        assert_eq!(err.registry_pack().unwrap().pack_id, "ga4gh.wes.1.1.0");
    }

    fn registry_with_drs_140_supported() -> Registry {
        shipped()
    }

    #[test]
    fn explicit_1_5_0_does_not_downgrade_to_supported_1_4_0() {
        let err =
            select_explicit(&registry_with_drs_140_supported(), "drs", "1.5.0", None).unwrap_err();
        assert_eq!(err.status_code(), AVAILABLE_BUT_NOT_SUPPORTED);
        assert_eq!(err.registry_pack().unwrap().version, "1.5.0");
        assert_ne!(err.registry_pack().unwrap().version, "1.4.0");
    }

    #[test]
    fn explicit_supported_1_4_0_selects_that_pack() {
        let pack = select_explicit(&registry_with_drs_140_supported(), "drs", "1.4.0", None)
            .expect("supported");
        assert_eq!(pack.pack_id, "ga4gh.drs.1.4.0");
        assert_eq!(pack.version, "1.4.0");
    }

    #[test]
    fn automatic_insufficient_does_not_pick_supported_1_4_0() {
        let err = select_automatic(&registry_with_drs_140_supported(), "drs", None).unwrap_err();
        assert_eq!(err.status_code(), INSUFFICIENT);
        assert!(err.registry_pack().is_none());
    }

    #[test]
    fn automatic_detected_1_5_0_does_not_fall_back_to_1_4_0() {
        let err =
            select_automatic(&registry_with_drs_140_supported(), "drs", Some("1.5.0")).unwrap_err();
        assert_eq!(err.status_code(), AVAILABLE_BUT_NOT_SUPPORTED);
        assert_eq!(err.registry_pack().unwrap().version, "1.5.0");
    }

    #[test]
    fn automatic_exact_match_selects_supported_pack() {
        let pack = select_automatic(&registry_with_drs_140_supported(), "drs", Some("1.4.0"))
            .expect("exact");
        assert_eq!(pack.version, "1.4.0");
    }

    #[test]
    fn all_supported_runs_only_official_supported_not_1_5_0() {
        let packs = select_all_official_supported(&registry_with_drs_140_supported(), "drs")
            .expect("one pack");
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].version, "1.4.0");
        assert!(!packs.iter().any(|p| p.version == "1.5.0"));
    }
}
