// SPDX-License-Identifier: Apache-2.0
//! Claim ↔ execution join. A VERIFIED sentence is derived from executed
//! predicates bound to recorded spec/target/fixture identities.
//!
//! Not a replay engine. Not a signed archive. Not HELIOS.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::claims::{evaluate, ClaimKind, ClaimStatus};
use crate::model::{StandardSelection, VerificationResult, VerificationRun, VerificationStatus};
use crate::standards::{
    binding_id, catalog_id, contract_for, execution_id, DRS_140_PACK_ID, SELECTED,
};
use crate::target::FailureAttribution;

/// Machine-checkable join from a claim to the execution that produced it.
/// IDs and check statuses only — not duplicated response blobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimJoin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixture_object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixture_expected_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_id: Option<String>,
    /// Helix git commit recorded on the run. Bound into the join so pasting
    /// `helix_git_sha` onto a historical file without restamping the join is Invalid.
    /// Does not enter `execution_id` or `coverage_id`. Not HELIOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helix_git_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helix_git_dirty: Option<bool>,
    pub checks: Vec<ClaimJoinCheck>,
    pub claims: Vec<ClaimJoinState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimJoinCheck {
    pub id: String,
    pub code: String,
    pub status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<FailureAttribution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimJoinState {
    pub kind: ClaimKind,
    pub status: ClaimStatus,
}

/// Inputs that must reproduce the same spec/target execution identities.
/// Excludes timestamp, prose, and machine-local paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReproductionTuple {
    pub standard: Option<String>,
    pub selected_version: Option<String>,
    pub pack_id: Option<String>,
    pub release_commit: Option<String>,
    pub pack_integrity_sha256: Option<String>,
    pub schema_document_sha256: Option<String>,
    pub schema_component_sha256: Option<String>,
    pub checker_id: Option<String>,
    pub binding_id: Option<String>,
    pub catalog_id: Option<String>,
    pub target_id: Option<String>,
    pub endpoint: Option<String>,
    pub fixture_object_id: Option<String>,
    pub fixture_expected_sha256: Option<String>,
}

impl ClaimJoin {
    pub fn from_run(run: &VerificationRun) -> Self {
        let sel = run.standard_selection.as_ref();
        let claims = evaluate(run);
        let mut checks: Vec<ClaimJoinCheck> = run
            .executed
            .iter()
            .chain(run.skipped.iter())
            .map(ClaimJoinCheck::from_result)
            .collect();
        checks.sort_by(|a, b| a.code.cmp(&b.code).then(a.id.cmp(&b.id)));
        Self {
            standard: sel.and_then(|s| s.standard.clone()),
            selected_version: sel.and_then(|s| s.selected_version.clone()),
            verified_version: sel.and_then(|s| s.verified_version.clone()),
            release_commit: sel.and_then(|s| s.standards_source_commit.clone()),
            pack_id: sel.and_then(|s| s.standards_registry_entry.clone()),
            pack_integrity_sha256: sel.and_then(|s| s.pack_integrity_sha256.clone()),
            schema_document_sha256: sel.and_then(|s| s.schema_document_sha256.clone()),
            schema_component_sha256: sel.and_then(|s| s.schema_component_sha256.clone()),
            checker_id: sel.and_then(|s| s.checker_id.clone()),
            binding_id: sel.and_then(|s| s.binding_id.clone()),
            catalog_id: sel.and_then(|s| s.catalog_id.clone()),
            execution_id: sel.and_then(|s| s.execution_id.clone()),
            target_execution_id: sel.and_then(|s| s.target_execution_id.clone()),
            fixture_object_id: run.drs_fixture.as_ref().map(|f| f.object_id.clone()),
            fixture_expected_sha256: run
                .drs_fixture
                .as_ref()
                .and_then(|f| f.expected_sha256.clone()),
            coverage_id: crate::coverage::CoverageReport::from_run(run).coverage_id,
            helix_git_sha: run.helix_git_sha.clone(),
            helix_git_dirty: run.helix_git_dirty,
            checks,
            claims: claims
                .items
                .iter()
                .map(|c| ClaimJoinState {
                    kind: c.kind,
                    status: c.status,
                })
                .collect(),
        }
    }

    pub fn ga4gh_requirement(&self) -> Option<ClaimStatus> {
        self.claims
            .iter()
            .find(|c| c.kind == ClaimKind::Ga4ghRequirement)
            .map(|c| c.status)
    }
}

impl ClaimJoinCheck {
    fn from_result(r: &VerificationResult) -> Self {
        Self {
            id: r.id.clone(),
            code: r.code.clone(),
            status: r.status,
            attribution: r.attribution,
        }
    }
}

pub fn reproduction_tuple(run: &VerificationRun) -> ReproductionTuple {
    let sel = run.standard_selection.as_ref();
    let id = run.target.identity.as_ref();
    ReproductionTuple {
        standard: sel.and_then(|s| s.standard.clone()),
        selected_version: sel.and_then(|s| s.selected_version.clone()),
        pack_id: sel.and_then(|s| s.standards_registry_entry.clone()),
        release_commit: sel.and_then(|s| s.standards_source_commit.clone()),
        pack_integrity_sha256: sel.and_then(|s| s.pack_integrity_sha256.clone()),
        schema_document_sha256: sel.and_then(|s| s.schema_document_sha256.clone()),
        schema_component_sha256: sel.and_then(|s| s.schema_component_sha256.clone()),
        checker_id: sel.and_then(|s| s.checker_id.clone()),
        binding_id: sel.and_then(|s| s.binding_id.clone()),
        catalog_id: sel.and_then(|s| s.catalog_id.clone()),
        target_id: id.map(|i| i.target_id.clone()),
        endpoint: id.map(|i| i.endpoint.clone()),
        fixture_object_id: run.drs_fixture.as_ref().map(|f| f.object_id.clone()),
        fixture_expected_sha256: run
            .drs_fixture
            .as_ref()
            .and_then(|f| f.expected_sha256.clone()),
    }
}

/// Single stamp point: `verified_version` is set only when `ga4gh_requirement` is Verified.
pub fn stamp_verified_version_if_justified(run: &mut VerificationRun) {
    let Some(sel) = run.standard_selection.as_ref() else {
        return;
    };
    if sel.selection_status != SELECTED {
        return;
    }
    let Some(selected) = sel.selected_version.clone().filter(|s| !s.is_empty()) else {
        return;
    };
    if sel.verified_version.is_some() {
        return;
    }

    let mut trial = run.clone();
    apply_verified_version(&mut trial, &selected);
    if evaluate(&trial).get(ClaimKind::Ga4ghRequirement).status != ClaimStatus::Verified {
        return;
    }
    apply_verified_version(run, &selected);
}

fn apply_verified_version(run: &mut VerificationRun, version: &str) {
    if let Some(sel) = run.standard_selection.as_mut() {
        sel.verified_version = Some(version.to_string());
    }
    for r in run.executed.iter_mut().chain(run.skipped.iter_mut()) {
        if r.selected_version.as_deref() == Some(version) {
            r.verified_version = Some(version.to_string());
        }
    }
}

/// Attach derived join after the semantic result exists. Presentation must not restamp.
pub fn finalize_run(run: &mut VerificationRun) {
    stamp_verified_version_if_justified(run);
    run.coverage = Some(crate::coverage::CoverageReport::from_run(run));
    run.claim_join = Some(ClaimJoin::from_run(run));
}

/// Internal consistency of observations, derived claims, coverage, and
/// spec/target identities. Does **not** require `helix_git_sha` to match this
/// binary. Use [`validate_claim_integrity`] before treating a run as *current*
/// verification from this verifier.
pub fn validate_artifact_consistency(run: &VerificationRun) -> Result<()> {
    let expected = ClaimJoin::from_run(run);
    if let Some(join) = &run.claim_join {
        if join != &expected {
            bail!("claim_join does not match the recorded execution");
        }
    }

    let expected_cov = crate::coverage::CoverageReport::from_run(run);
    if expected_cov.contains_ranking_semantics() {
        bail!("coverage contains ranking semantics");
    }
    if let Some(cov) = &run.coverage {
        if cov != &expected_cov {
            bail!("coverage does not match the recorded execution");
        }
        if cov.contains_ranking_semantics() {
            bail!("coverage contains ranking semantics");
        }
    }
    if let Some(join) = &run.claim_join {
        if join.coverage_id != expected_cov.coverage_id {
            bail!("claim_join.coverage_id does not match derived coverage");
        }
    }

    let claims = evaluate(run);
    crate::claims::check_set(&claims)?;

    let verified = run
        .standard_selection
        .as_ref()
        .and_then(|s| s.verified_version.clone())
        .filter(|s| !s.is_empty());
    if let Some(ver) = verified {
        if run.claim_join.is_none() {
            bail!("verified_version={ver} requires claim_join");
        }
        if claims.get(ClaimKind::Ga4ghRequirement).status != ClaimStatus::Verified {
            bail!("verified_version={ver} but ga4gh_requirement is not verified");
        }
        if run.coverage.is_none() {
            bail!("verified_version={ver} requires coverage");
        }
        if !expected_cov.required_complete {
            bail!("verified_version={ver} but coverage required_complete is false");
        }
        let selected = run
            .standard_selection
            .as_ref()
            .and_then(|s| s.selected_version.as_deref())
            .unwrap_or("");
        if selected != ver {
            bail!("verified_version ({ver}) != selected_version ({selected})");
        }
    }

    validate_recorded_identities(run)?;
    validate_helix_git_shape(run)?;
    Ok(())
}

/// Fail closed if a recorded VERIFIED / join / identity cannot be re-derived
/// *and* Helix git provenance does not match this binary.
pub fn validate_claim_integrity(run: &VerificationRun) -> Result<()> {
    validate_artifact_consistency(run)?;
    validate_helix_git_matches_this_build(run)?;
    Ok(())
}

fn validate_helix_git_shape(run: &VerificationRun) -> Result<()> {
    if let Some(recorded) = run.helix_git_sha.as_deref() {
        if crate::provenance::parse_git_sha(recorded).is_none() {
            bail!("helix_git_sha is not a 40-char lowercase git SHA");
        }
    }
    if run.helix_git_dirty == Some(false) && run.helix_git_sha.is_none() {
        bail!("helix_git_dirty=false requires helix_git_sha");
    }
    Ok(())
}

fn validate_helix_git_matches_this_build(run: &VerificationRun) -> Result<()> {
    if let Some(recorded) = run.helix_git_sha.as_deref() {
        match crate::provenance::helix_git_sha() {
            Some(built) if recorded != built => {
                bail!("helix_git_sha does not match this verifier build");
            }
            None => {
                bail!("helix_git_sha is recorded but this verifier build has no git provenance")
            }
            Some(_) => {}
        }
    }
    if let Some(recorded_dirty) = run.helix_git_dirty {
        match crate::provenance::helix_git_dirty() {
            Some(built) if recorded_dirty != built => {
                bail!("helix_git_dirty does not match this verifier build");
            }
            None => {
                bail!("helix_git_dirty is recorded but this verifier build has no git dirty state")
            }
            Some(_) => {}
        }
    }
    Ok(())
}

fn validate_recorded_identities(run: &VerificationRun) -> Result<()> {
    let Some(sel) = run.standard_selection.as_ref() else {
        return Ok(());
    };

    if let Some(id) = sel.checker_id.as_deref() {
        let executed = crate::checker::executed_checker_id();
        if id != executed {
            bail!("checker_id does not match the executed checker");
        }
    }

    if let Some(pack_id) = sel.standards_registry_entry.as_deref() {
        if let Some(contract) = contract_for(pack_id) {
            if let Some(cat) = sel.catalog_id.as_deref() {
                let expected = catalog_id(contract);
                if cat != expected {
                    bail!("catalog_id does not match the compiled support catalog");
                }
            }
            if let (Some(p), Some(d), Some(c)) = (
                sel.pack_integrity_sha256.as_deref(),
                sel.schema_document_sha256.as_deref(),
                sel.schema_component_sha256.as_deref(),
            ) {
                if let Some(bid) = sel.binding_id.as_deref() {
                    if bid != binding_id(contract, p, d, c) {
                        bail!("binding_id does not match pack/schema/checker identity");
                    }
                }
                if let Some(eid) = sel.execution_id.as_deref() {
                    let checker = sel
                        .checker_id
                        .clone()
                        .unwrap_or_else(crate::checker::executed_checker_id);
                    let entry = sel.schema_entry.as_deref().unwrap_or(contract.schema_entry);
                    let expected =
                        execution_id(pack_id, p, d, c, &checker, entry, contract.schema_component);
                    if eid != expected {
                        bail!("execution_id does not match spec-join inputs");
                    }
                }
            }
        } else if pack_id == DRS_140_PACK_ID {
            bail!("ga4gh.drs.1.4.0 pack is recorded but no support contract is compiled");
        }
    }

    if let (Some(identity), Some(recorded)) = (
        run.target.identity.as_ref(),
        sel.target_execution_id.as_deref(),
    ) {
        let expected = crate::target::target_execution_id(identity, sel, run.drs_fixture.as_ref());
        if recorded != expected {
            bail!("target_execution_id does not match target/fixture/spec identity");
        }
    }
    Ok(())
}

/// Test helper: mutate a recorded identity field without re-running checks.
pub fn tamper_selection(run: &mut VerificationRun, f: impl FnOnce(&mut StandardSelection)) {
    if let Some(sel) = run.standard_selection.as_mut() {
        f(sel);
    }
}

/// `ga4gh_requirement` VERIFIED is the only gate that may set `verified_version`.
pub fn ga4gh_requirement_is_verified(run: &VerificationRun) -> bool {
    evaluate(run).get(ClaimKind::Ga4ghRequirement).status == ClaimStatus::Verified
}
