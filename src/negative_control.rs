// SPDX-License-Identifier: Apache-2.0
//! Versioned DRS 1.4.0 mutation / negative-control evidence.
//!
//! Binds to an existing [`VerificationRun`]. Does not stamp `verified_version`,
//! does not override `claims[]` / `claim_join`, and does not rank targets.
//! Mutations belong to the HTTP target, not to Helix. Not HELIOS. Not certification.
//!
//! Distinct from the unversioned `HLX-MUT-*` corpus in [`crate::mutation`].

use serde::{Deserialize, Serialize};

use crate::claims::{evaluate, ClaimKind, ClaimStatus};
use crate::model::{VerificationResult, VerificationRun, VerificationStatus};
use crate::target::FailureAttribution;

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

/// Whether the mutated behavior sits inside the current DRS catalog closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationScope {
    /// Behavior exercised by a shipped DRS 1.4.0 catalog check.
    InContract,
    /// Deliberate change Helix does not currently test.
    OutsideClosure,
    /// Semantic-preserving control (must not create a false failure).
    Benign,
    /// Harness self-check: field unused by the selected check.
    HarnessIneffective,
}

impl MutationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InContract => "in_contract",
            Self::OutsideClosure => "outside_closure",
            Self::Benign => "benign",
            Self::HarnessIneffective => "harness_ineffective",
        }
    }
}

/// Observed coverage classification. Distinct from “Helix is broken.”
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageClassification {
    /// In-scope mutation changed the expected check.
    MutationDetected,
    /// Change is not part of the current verification contract.
    MutationOutsideClosure,
    /// Semantic-preserving change; no false target_failure.
    BenignNoChange,
    /// Selected check did not use the mutated field.
    NoBehavioralChange,
}

impl CoverageClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MutationDetected => "mutation_detected",
            Self::MutationOutsideClosure => "mutation_outside_closure",
            Self::BenignNoChange => "benign_no_change",
            Self::NoBehavioralChange => "no_behavioral_change",
        }
    }
}

/// Oracle used by HLX-DRS-003. Advertised digest is not independent evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumOracle {
    AdvertisedConsistency,
    OperatorDigest,
    NotApplicable,
}

impl ChecksumOracle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdvertisedConsistency => "advertised_consistency",
            Self::OperatorDigest => "operator_digest",
            Self::NotApplicable => "not_applicable",
        }
    }
}

/// Spec/checker join copied from the run. Not a second execution identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_integrity_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_document_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_component_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
}

impl VerifierIdentity {
    pub fn from_run(run: &VerificationRun) -> Self {
        let sel = run.standard_selection.as_ref();
        Self {
            pack_id: sel.and_then(|s| s.standards_registry_entry.clone()),
            release_commit: sel.and_then(|s| s.standards_source_commit.clone()),
            pack_integrity_sha256: sel.and_then(|s| s.pack_integrity_sha256.clone()),
            schema_document_sha256: sel.and_then(|s| s.schema_document_sha256.clone()),
            schema_component_sha256: sel.and_then(|s| s.schema_component_sha256.clone()),
            checker_id: sel.and_then(|s| s.checker_id.clone()),
            binding_id: sel.and_then(|s| s.binding_id.clone()),
            catalog_id: sel.and_then(|s| s.catalog_id.clone()),
            execution_id: sel.and_then(|s| s.execution_id.clone()),
        }
    }
}

/// Inputs for [`MutationEvidence::bind`]. Not a verification claim.
pub struct MutationBind<'a> {
    pub mutation_id: &'a str,
    pub mutation_description: &'a str,
    pub mutation_scope: MutationScope,
    pub expected_check_ids: &'a [&'a str],
    pub checksum_oracle: ChecksumOracle,
    pub baseline: &'a VerificationRun,
    pub mutated: &'a VerificationRun,
}

/// Record of one controlled target mutation. Descriptive only.
///
/// `observed_claim_state` is copied from [`evaluate`] of the mutated run at
/// bind time. Presentation must still recompute claims through `evaluate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationEvidence {
    pub mutation_id: String,
    pub mutation_description: String,
    pub mutation_scope: MutationScope,
    pub expected_check_ids: Vec<String>,
    pub observed_check_ids: Vec<String>,
    pub expected_claim_state: ClaimStatus,
    pub observed_claim_state: ClaimStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutated_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_target_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutated_target_execution_id: Option<String>,
    pub verifier_identity: VerifierIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<FailureAttribution>,
    pub coverage_classification: CoverageClassification,
    pub checksum_oracle: ChecksumOracle,
    pub ranking_semantics: String,
    pub creates_verification: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_artifact_identity: Option<String>,
}

impl MutationEvidence {
    /// Bind mutation metadata to two already-executed runs. Does not restamp claims.
    pub fn bind(input: MutationBind<'_>) -> Self {
        let observed_claim = evaluate(input.mutated)
            .get(ClaimKind::Ga4ghRequirement)
            .status;
        let expected_claim = match input.mutation_scope {
            MutationScope::InContract => ClaimStatus::NotVerified,
            MutationScope::OutsideClosure
            | MutationScope::Benign
            | MutationScope::HarnessIneffective => {
                evaluate(input.baseline)
                    .get(ClaimKind::Ga4ghRequirement)
                    .status
            }
        };
        let observed_check_ids: Vec<String> = input
            .expected_check_ids
            .iter()
            .filter(|id| {
                check(input.mutated, id).is_some_and(|r| r.status != VerificationStatus::Pass)
            })
            .map(|id| (*id).to_string())
            .collect();
        let coverage = classify_coverage(
            input.mutation_scope,
            input.expected_check_ids,
            input.baseline,
            input.mutated,
        );
        let attribution = input
            .expected_check_ids
            .iter()
            .find_map(|id| check(input.mutated, id).and_then(|r| r.attribution));
        let sel_b = input.baseline.standard_selection.as_ref();
        let sel_m = input.mutated.standard_selection.as_ref();
        Self {
            mutation_id: input.mutation_id.to_string(),
            mutation_description: input.mutation_description.to_string(),
            mutation_scope: input.mutation_scope,
            expected_check_ids: input
                .expected_check_ids
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            observed_check_ids,
            expected_claim_state: expected_claim,
            observed_claim_state: observed_claim,
            baseline_execution_id: sel_b.and_then(|s| s.execution_id.clone()),
            mutated_execution_id: sel_m.and_then(|s| s.execution_id.clone()),
            baseline_target_execution_id: sel_b.and_then(|s| s.target_execution_id.clone()),
            mutated_target_execution_id: sel_m.and_then(|s| s.target_execution_id.clone()),
            verifier_identity: VerifierIdentity::from_run(input.mutated),
            attribution,
            coverage_classification: coverage,
            checksum_oracle: input.checksum_oracle,
            ranking_semantics: "absent".into(),
            creates_verification: false,
            base_target_id: input
                .baseline
                .target
                .identity
                .as_ref()
                .map(|t| t.target_id.clone()),
            base_artifact_identity: None,
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
}

fn classify_coverage(
    scope: MutationScope,
    expected_ids: &[&str],
    baseline: &VerificationRun,
    mutated: &VerificationRun,
) -> CoverageClassification {
    match scope {
        MutationScope::OutsideClosure => CoverageClassification::MutationOutsideClosure,
        MutationScope::Benign => CoverageClassification::BenignNoChange,
        MutationScope::HarnessIneffective => CoverageClassification::NoBehavioralChange,
        MutationScope::InContract => {
            let detected = expected_ids.iter().any(|id| {
                let before = check(baseline, id).map(|r| r.status);
                let after = check(mutated, id).map(|r| r.status);
                before == Some(VerificationStatus::Pass) && after == Some(VerificationStatus::Fail)
            });
            if detected {
                CoverageClassification::MutationDetected
            } else {
                CoverageClassification::NoBehavioralChange
            }
        }
    }
}

/// Catalog row by Helix `id` (executed or skipped).
pub fn check<'a>(run: &'a VerificationRun, id: &str) -> Option<&'a VerificationResult> {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id)
}

pub fn check_status(run: &VerificationRun, id: &str) -> Option<VerificationStatus> {
    check(run, id).map(|r| r.status)
}

pub fn ga4gh_requirement(run: &VerificationRun) -> ClaimStatus {
    evaluate(run).get(ClaimKind::Ga4ghRequirement).status
}

pub fn verified_version(run: &VerificationRun) -> Option<&str> {
    run.standard_selection
        .as_ref()
        .and_then(|s| s.verified_version.as_deref())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ranking_fields_are_absent_by_construction() {
        let json = serde_json::json!({
            "ranking_semantics": "absent",
            "creates_verification": false
        });
        assert!(json.get("score").is_none());
        assert!(json.get("overall_score").is_none());
    }
}
