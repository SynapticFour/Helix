// SPDX-License-Identifier: Apache-2.0
//! Reviewed target independence. Operator CLI metadata is not proof.
//!
//! `independent_implementation_evidence` is true only when a shipped reviewed
//! record classifies the `target_id` as a real independent implementation **and**
//! the operator kind is also `real_*`. A caller cannot turn a mock or Ferrum
//! into independent evidence by changing `--target-kind`.
//!
//! Helix does not authenticate the process behind a URL. These records are
//! reviewed identity, not a cryptographic bind of live bytes. Not HELIOS.
//! Not certification. Not ranking.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::model::VerificationRun;
use crate::target::TargetKind;

const YAML: &str = include_str!("../targets/independence.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceRegistry {
    pub schema_version: String,
    pub records: Vec<IndependenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceRecord {
    pub target_id: String,
    pub classification: TargetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub lineage: String,
    pub artifact_kind: String,
    pub artifact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    /// Project-declared DRS/WES version in source. Untrusted. Not verified_version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_standard_version_in_project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl IndependenceRecord {
    pub fn qualifies_as_independent_implementation(&self) -> bool {
        self.classification
            .qualifies_as_independent_implementation()
    }
}

fn registry() -> &'static IndependenceRegistry {
    static REG: OnceLock<IndependenceRegistry> = OnceLock::new();
    REG.get_or_init(|| {
        let reg: IndependenceRegistry =
            serde_yaml::from_str(YAML).expect("targets/independence.yaml must parse");
        assert_eq!(
            reg.schema_version, "helix-independence-v1",
            "independence registry schema_version"
        );
        let mut seen = std::collections::BTreeSet::new();
        for r in &reg.records {
            assert!(
                seen.insert(r.target_id.clone()),
                "duplicate independence target_id {}",
                r.target_id
            );
        }
        reg
    })
}

pub fn reviewed_record(target_id: &str) -> Option<&'static IndependenceRecord> {
    registry().records.iter().find(|r| r.target_id == target_id)
}

/// Operator kind + reviewed record. Neither alone is sufficient.
pub fn run_counts_as_independent(run: &VerificationRun) -> bool {
    let Some(identity) = run.target.identity.as_ref() else {
        return false;
    };
    if !identity
        .target_kind
        .qualifies_as_independent_implementation()
    {
        return false;
    }
    let Some(record) = reviewed_record(&identity.target_id) else {
        return false;
    };
    if !record.qualifies_as_independent_implementation() {
        return false;
    }
    if let (Some(declared), Some(reviewed)) = (
        identity.implementation_name.as_deref(),
        record.implementation_name.as_deref(),
    ) {
        if declared != reviewed {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Target;
    use crate::target::{DeclaredTarget, TargetIdentity};

    fn run_with(id: &str, kind: TargetKind, name: Option<&str>) -> VerificationRun {
        VerificationRun::new(Target::from_identity(TargetIdentity::from_declared(
            "http://127.0.0.1:9",
            &DeclaredTarget {
                target_id: Some(id.into()),
                kind,
                implementation_name: name.map(str::to_string),
                ..DeclaredTarget::default()
            },
        )))
    }

    #[test]
    fn yaml_parses_and_starter_kit_is_independent_lineage() {
        let sk = reviewed_record("ga4gh-starter-kit-drs-0.3.2").expect("starter kit record");
        assert!(sk.qualifies_as_independent_implementation());
        assert!(sk
            .artifact_identity
            .as_deref()
            .unwrap()
            .starts_with("sha256:"));
        let bento = reviewed_record("bento-drs-0.21.5").expect("bento record");
        assert!(bento.qualifies_as_independent_implementation());
        assert_eq!(
            bento.artifact_identity.as_deref(),
            Some("1dc55ebea90185b1fec2c78c8c52909dd0ca889e")
        );
        assert_eq!(
            bento.declared_standard_version_in_project.as_deref(),
            Some("1.4.0")
        );
        assert_ne!(sk.lineage, bento.lineage);
        let ferrum = reviewed_record("ferrum").expect("ferrum record");
        assert!(!ferrum.qualifies_as_independent_implementation());
        assert_eq!(ferrum.classification, TargetKind::ReferenceImplementation);
        let mock = reviewed_record("helix-in-process-mock").expect("mock record");
        assert!(!mock.qualifies_as_independent_implementation());
        assert_eq!(mock.classification, TargetKind::Mock);
    }

    #[test]
    fn operator_kind_cannot_upgrade_mock_record() {
        let run = run_with(
            "helix-in-process-mock",
            TargetKind::RealIndependentLocalImplementation,
            Some("helix-wiremock"),
        );
        assert!(!run_counts_as_independent(&run));
    }

    #[test]
    fn operator_kind_cannot_upgrade_reference_record() {
        let run = run_with(
            "ferrum",
            TargetKind::RealIndependentLocalImplementation,
            Some("Ferrum"),
        );
        assert!(!run_counts_as_independent(&run));
    }

    #[test]
    fn real_kind_without_reviewed_record_is_not_independent() {
        let run = run_with(
            "forged-independent",
            TargetKind::RealIndependentLocalImplementation,
            None,
        );
        assert!(!run_counts_as_independent(&run));
    }

    #[test]
    fn reviewed_independent_with_mock_kind_is_not_independent() {
        let run = run_with(
            "ga4gh-starter-kit-drs-0.3.2",
            TargetKind::Mock,
            Some("ga4gh-starter-kit-drs"),
        );
        assert!(!run_counts_as_independent(&run));
    }

    #[test]
    fn reviewed_independent_with_matching_real_kind_counts() {
        let run = run_with(
            "bento-drs-0.21.5",
            TargetKind::RealIndependentLocalImplementation,
            Some("bento_drs"),
        );
        assert!(run_counts_as_independent(&run));
    }

    #[test]
    fn implementation_name_mismatch_fails_closed() {
        let run = run_with(
            "bento-drs-0.21.5",
            TargetKind::RealIndependentLocalImplementation,
            Some("not-bento"),
        );
        assert!(!run_counts_as_independent(&run));
    }
}
