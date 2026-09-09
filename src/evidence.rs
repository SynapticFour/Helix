// SPDX-License-Identifier: Apache-2.0
//! Durability of persisted `helix verify` JSON as a Helix evidence object.
//!
//! Claims, coverage, and `verified_version` are derived. JSON formatting is
//! not identity. Helix git provenance distinguishes *current* verifier evidence
//! from *historical* observations. Not a cache. Not HELIOS signing.

use serde::{Deserialize, Serialize};

use crate::claim_integrity::{
    validate_artifact_consistency, validate_claim_integrity, ClaimJoin, ReproductionTuple,
};
use crate::claims::{evaluate, ClaimSet};
use crate::coverage::CoverageReport;
use crate::model::VerificationRun;
use crate::provenance::cites_this_verifier_build;

/// Standing of a persisted run relative to **this** verifier binary.
/// Computed on inspect. Not a JSON field. Not stored. Not HELIOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStanding {
    /// Join, coverage, claims, or identities cannot be re-derived.
    Invalid,
    /// Internally consistent; not attributed to this binary (no SHA, or SHA/dirty mismatch).
    HistoricalObservation,
    /// Internally consistent and cites this verifier build.
    CurrentVerifierEvidence,
}

impl EvidenceStanding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::HistoricalObservation => "historical_observation",
            Self::CurrentVerifierEvidence => "current_verifier_evidence",
        }
    }

    pub fn is_current(self) -> bool {
        self == Self::CurrentVerifierEvidence
    }
}

/// Revalidation result. Claims and coverage are always recomputed.
#[derive(Debug, Clone)]
pub struct EvidenceRevalidation {
    pub standing: EvidenceStanding,
    pub claims: ClaimSet,
    pub coverage: CoverageReport,
    pub join: ClaimJoin,
    pub reproduction: ReproductionTuple,
}

/// Classify persisted evidence. Never upgrades Historical → Current by trusting JSON claims.
/// Standing is computed: it is not a JSON field. Pasting `helix_git_sha` onto a
/// historical file without restamping `claim_join` is Invalid, not Current.
pub fn classify_evidence(run: &VerificationRun) -> EvidenceStanding {
    if validate_artifact_consistency(run).is_err() {
        return EvidenceStanding::Invalid;
    }
    if cites_this_verifier_build(run) {
        EvidenceStanding::CurrentVerifierEvidence
    } else {
        EvidenceStanding::HistoricalObservation
    }
}

/// True only when this binary may treat the run as current verification.
pub fn is_current_verification(run: &VerificationRun) -> bool {
    classify_evidence(run) == EvidenceStanding::CurrentVerifierEvidence
        && validate_claim_integrity(run).is_ok()
}

/// Load-time revalidation: recompute derived fields; classify standing.
/// Claims and coverage are never taken from JSON as authority.
pub fn revalidate_evidence(run: &VerificationRun) -> EvidenceRevalidation {
    EvidenceRevalidation {
        standing: classify_evidence(run),
        claims: evaluate(run),
        coverage: CoverageReport::from_run(run),
        join: ClaimJoin::from_run(run),
        reproduction: crate::claim_integrity::reproduction_tuple(run),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Target, VerificationRun};

    #[test]
    fn empty_run_is_not_invalid() {
        let run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        // New() stamps this build's SHA when git is available → may be Current
        // for a never-executed run. Standing must not be Invalid solely from emptiness.
        let standing = classify_evidence(&run);
        assert_ne!(standing, EvidenceStanding::Invalid);
    }
}
