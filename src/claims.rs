// SPDX-License-Identifier: Apache-2.0
//! Explicit verification claims. Human-readable VERIFIED / NOT_VERIFIED text
//! is generated only from this model.
//!
//! A claim is `verified` only when every required predicate holds. Missing
//! evidence does not produce a claim. Fixture FAIL is not a GA4GH MUST fail.
//! Empty normative sets are not vacuously verified.
//!
//! HelixTest already runs the checks; this module productizes what a run is
//! allowed to *say*. Not HELIOS. Not GA4GH certification.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::layer::CheckLayer;
use crate::model::{StandardSelection, VerificationResult, VerificationRun, VerificationStatus};
use crate::sanitize::sanitize_untrusted;
use crate::standards::{
    BindingKind, ClaimScope, AMBIGUOUS, AVAILABLE_BUT_NOT_SUPPORTED, DEVELOPMENT_NOT_SELECTABLE,
    INSUFFICIENT, MULTIPLE_PACKS_NOT_EXECUTABLE, NEEDS_RELEASE_CLASS, NOT_SUPPORTED,
    NO_OFFICIAL_SUPPORTED, SELECTED, UNKNOWN_TO_HELIX, UNVERSIONED,
};
use crate::traceability::Authority;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

/// Six claims, always, in this order. Never collapsed into one sentence.
pub const CLAIM_KINDS: [ClaimKind; 6] = [
    ClaimKind::Ga4ghRequirement,
    ClaimKind::Schema,
    ClaimKind::Behavior,
    ClaimKind::Interoperability,
    ClaimKind::Security,
    ClaimKind::Benchmark,
];

/// What a VERIFIED row is allowed to mean. Separate kinds stay separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Ga4ghRequirement,
    Schema,
    Behavior,
    Interoperability,
    Security,
    Benchmark,
}

impl ClaimKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ga4ghRequirement => "ga4gh_requirement",
            Self::Schema => "schema",
            Self::Behavior => "behavior",
            Self::Interoperability => "interoperability",
            Self::Security => "security",
            Self::Benchmark => "benchmark",
        }
    }
}

/// Issued only when every required predicate for that kind holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Verified,
    NotVerified,
}

impl ClaimStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::NotVerified => "not_verified",
        }
    }

    pub fn report_mark(self) -> &'static str {
        match self {
            Self::Verified => "VERIFIED",
            Self::NotVerified => "NOT_VERIFIED",
        }
    }
}

/// Predicates that must all hold for a verifiable kind to be VERIFIED.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimPredicate {
    ExactStandardIdentified,
    SupportedReleaseSelected,
    PinnedSpecificationSource,
    IntegrityValidationSuccessful,
    SelectedEqualsTested,
    RequiredNormativeChecksExecuted,
    RequiredNormativeChecksPassed,
    NoBlockingNormativeFailures,
    CoverageRequirementsSatisfied,
    EvidenceRecorded,
    NoSubstitution,
}

impl ClaimPredicate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactStandardIdentified => "exact_standard_identified",
            Self::SupportedReleaseSelected => "supported_release_selected",
            Self::PinnedSpecificationSource => "pinned_specification_source",
            Self::IntegrityValidationSuccessful => "integrity_validation_successful",
            Self::SelectedEqualsTested => "selected_equals_tested",
            Self::RequiredNormativeChecksExecuted => "required_normative_checks_executed",
            Self::RequiredNormativeChecksPassed => "required_normative_checks_passed",
            Self::NoBlockingNormativeFailures => "no_blocking_normative_failures",
            Self::CoverageRequirementsSatisfied => "coverage_requirements_satisfied",
            Self::EvidenceRecorded => "evidence_recorded",
            Self::NoSubstitution => "no_substitution",
        }
    }

    pub fn required_for(kind: ClaimKind) -> &'static [ClaimPredicate] {
        match kind {
            ClaimKind::Ga4ghRequirement
            | ClaimKind::Schema
            | ClaimKind::Behavior
            | ClaimKind::Security => ALL_VERIFIED_PREDICATES,
            ClaimKind::Interoperability | ClaimKind::Benchmark => &[],
        }
    }
}

const ALL_VERIFIED_PREDICATES: &[ClaimPredicate] = &[
    ClaimPredicate::ExactStandardIdentified,
    ClaimPredicate::SupportedReleaseSelected,
    ClaimPredicate::PinnedSpecificationSource,
    ClaimPredicate::IntegrityValidationSuccessful,
    ClaimPredicate::SelectedEqualsTested,
    ClaimPredicate::RequiredNormativeChecksExecuted,
    ClaimPredicate::RequiredNormativeChecksPassed,
    ClaimPredicate::NoBlockingNormativeFailures,
    ClaimPredicate::CoverageRequirementsSatisfied,
    ClaimPredicate::EvidenceRecorded,
    ClaimPredicate::NoSubstitution,
];

/// Why a claim is NOT_VERIFIED. Codes are an enum, not free text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimBlockCode {
    UnversionedRun,
    NoVersionSelected,
    StandardNotIdentified,
    AvailableButNotSupported,
    NotSupported,
    UnknownVersion,
    NoOfficialSupported,
    DevelopmentNotSelectable,
    AmbiguousSelection,
    NeedsReleaseClass,
    InsufficientSelection,
    MultiplePacksNotExecutable,
    SelectedNeVerified,
    SubstitutionDetected,
    ProvenanceMissing,
    IntegrityValidationNotRecorded,
    IntegrityMismatch,
    NoNormativeChecks,
    IncompleteNormativeCoverage,
    NormativeCheckFailed,
    NormativeCheckError,
    NoChecksRecorded,
    InteroperabilityIsNotAGa4ghRequirement,
    BenchmarkIsMeasurementOnly,
}

impl ClaimBlockCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnversionedRun => "unversioned_run",
            Self::NoVersionSelected => "no_version_selected",
            Self::StandardNotIdentified => "standard_not_identified",
            Self::AvailableButNotSupported => "available_but_not_supported",
            Self::NotSupported => "not_supported",
            Self::UnknownVersion => "unknown_version",
            Self::NoOfficialSupported => "no_official_supported",
            Self::DevelopmentNotSelectable => "development_not_selectable",
            Self::AmbiguousSelection => "ambiguous_selection",
            Self::NeedsReleaseClass => "needs_release_class",
            Self::InsufficientSelection => "insufficient_selection",
            Self::MultiplePacksNotExecutable => "multiple_packs_not_executable",
            Self::SelectedNeVerified => "selected_ne_verified",
            Self::SubstitutionDetected => "substitution_detected",
            Self::ProvenanceMissing => "provenance_missing",
            Self::IntegrityValidationNotRecorded => "integrity_validation_not_recorded",
            Self::IntegrityMismatch => "integrity_mismatch",
            Self::NoNormativeChecks => "no_normative_checks",
            Self::IncompleteNormativeCoverage => "incomplete_normative_coverage",
            Self::NormativeCheckFailed => "normative_check_failed",
            Self::NormativeCheckError => "normative_check_error",
            Self::NoChecksRecorded => "no_checks_recorded",
            Self::InteroperabilityIsNotAGa4ghRequirement => {
                "interoperability_is_not_a_ga4gh_requirement"
            }
            Self::BenchmarkIsMeasurementOnly => "benchmark_is_measurement_only",
        }
    }
}

/// One recorded field or check that justifies a block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEvidence {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// One blocking predicate plus the evidence that fired it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBlock {
    pub code: ClaimBlockCode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ClaimEvidence>,
}

/// One of the six claims. Status is computed; it is not a free-text label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub kind: ClaimKind,
    pub status: ClaimStatus,
    pub satisfied: Vec<ClaimPredicate>,
    pub blocks: Vec<ClaimBlock>,
}

impl Claim {
    pub fn block_codes(&self) -> Vec<ClaimBlockCode> {
        self.blocks.iter().map(|b| b.code).collect()
    }

    pub fn has_block(&self, code: ClaimBlockCode) -> bool {
        self.blocks.iter().any(|b| b.code == code)
    }
}

/// Always six items ([`CLAIM_KINDS`] order). Serialized as a JSON array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClaimSet {
    pub items: Vec<Claim>,
}

impl ClaimSet {
    pub fn get(&self, kind: ClaimKind) -> &Claim {
        self.items
            .iter()
            .find(|c| c.kind == kind)
            .expect("ClaimSet always contains every ClaimKind")
    }

    pub fn verified_count(&self) -> usize {
        self.items
            .iter()
            .filter(|c| c.status == ClaimStatus::Verified)
            .count()
    }

    pub fn any_verified(&self) -> bool {
        self.verified_count() > 0
    }
}

struct Accum {
    satisfied: BTreeSet<ClaimPredicate>,
    blocks: BTreeMap<ClaimBlockCode, Vec<ClaimEvidence>>,
}

impl Accum {
    fn new() -> Self {
        Self {
            satisfied: BTreeSet::new(),
            blocks: BTreeMap::new(),
        }
    }

    fn satisfy(&mut self, p: ClaimPredicate) {
        self.satisfied.insert(p);
    }

    fn block(&mut self, code: ClaimBlockCode, evidence: Vec<ClaimEvidence>) {
        self.blocks.entry(code).or_default().extend(evidence);
    }

    fn finish(mut self, kind: ClaimKind) -> Claim {
        for ev in self.blocks.values_mut() {
            ev.sort_by(|a, b| {
                a.field
                    .cmp(&b.field)
                    .then(a.check_id.cmp(&b.check_id))
                    .then(a.observed.cmp(&b.observed))
            });
        }
        let required = ClaimPredicate::required_for(kind);
        let all_required = required.iter().all(|p| self.satisfied.contains(p));
        let never_verified = matches!(kind, ClaimKind::Interoperability | ClaimKind::Benchmark);
        let status = if !never_verified && self.blocks.is_empty() && all_required {
            ClaimStatus::Verified
        } else {
            ClaimStatus::NotVerified
        };
        Claim {
            kind,
            status,
            satisfied: self.satisfied.into_iter().collect(),
            blocks: self
                .blocks
                .into_iter()
                .map(|(code, evidence)| ClaimBlock { code, evidence })
                .collect(),
        }
    }
}

/// Compute claims from recorded run fields only. Deterministic. No PASS/FAIL string search.
pub fn evaluate(run: &VerificationRun) -> ClaimSet {
    let selection = run
        .standard_selection
        .clone()
        .unwrap_or_else(StandardSelection::unversioned);
    let set = ClaimSet {
        items: CLAIM_KINDS
            .iter()
            .map(|kind| evaluate_kind(run, &selection, *kind))
            .collect(),
    };
    debug_assert!(
        check_set(&set).is_ok(),
        "claim engine emitted an internally inconsistent set: {:?}",
        check_set(&set).err()
    );
    set
}

/// A VERIFIED row must have every required predicate and no blocks.
/// Interoperability and benchmark must never be VERIFIED.
pub fn check_set(set: &ClaimSet) -> anyhow::Result<()> {
    use anyhow::bail;
    if set.items.len() != CLAIM_KINDS.len() {
        bail!("claim set must contain exactly {} kinds", CLAIM_KINDS.len());
    }
    for (i, kind) in CLAIM_KINDS.iter().enumerate() {
        if set.items[i].kind != *kind {
            bail!("claim set order must match CLAIM_KINDS");
        }
    }
    for c in &set.items {
        if matches!(c.kind, ClaimKind::Interoperability | ClaimKind::Benchmark)
            && c.status == ClaimStatus::Verified
        {
            bail!(
                "{} cannot be VERIFIED (not a GA4GH requirement / measurement only)",
                c.kind.as_str()
            );
        }
        if c.status == ClaimStatus::Verified {
            if !c.blocks.is_empty() {
                bail!(
                    "VERIFIED claim {} has blocks; predicates did not hold",
                    c.kind.as_str()
                );
            }
            for p in ClaimPredicate::required_for(c.kind) {
                if !c.satisfied.contains(p) {
                    bail!(
                        "VERIFIED claim {} missing predicate {}",
                        c.kind.as_str(),
                        p.as_str()
                    );
                }
            }
        }
    }
    Ok(())
}

fn evaluate_kind(run: &VerificationRun, selection: &StandardSelection, kind: ClaimKind) -> Claim {
    let mut acc = Accum::new();
    match kind {
        ClaimKind::Interoperability => {
            acc.block(
                ClaimBlockCode::InteroperabilityIsNotAGa4ghRequirement,
                vec![ev(
                    "claim.kind",
                    Some("interoperability"),
                    Some("not a GA4GH MUST"),
                )],
            );
            apply_selection_predicates(&mut acc, selection);
            apply_evidence_recorded(&mut acc, run);
        }
        ClaimKind::Benchmark => {
            acc.block(
                ClaimBlockCode::BenchmarkIsMeasurementOnly,
                vec![ev(
                    "claim.kind",
                    Some("benchmark"),
                    Some("measurement only; never a verification claim"),
                )],
            );
            apply_evidence_recorded(&mut acc, run);
        }
        ClaimKind::Ga4ghRequirement => {
            apply_selection_predicates(&mut acc, selection);
            apply_evidence_recorded(&mut acc, run);
            apply_normative_predicates(&mut acc, run, None);
        }
        ClaimKind::Schema => {
            apply_selection_predicates(&mut acc, selection);
            apply_evidence_recorded(&mut acc, run);
            apply_normative_predicates(&mut acc, run, Some(CheckLayer::Schema));
        }
        ClaimKind::Behavior => {
            apply_selection_predicates(&mut acc, selection);
            apply_evidence_recorded(&mut acc, run);
            apply_normative_predicates(&mut acc, run, Some(CheckLayer::Behavior));
        }
        ClaimKind::Security => {
            apply_selection_predicates(&mut acc, selection);
            apply_evidence_recorded(&mut acc, run);
            apply_normative_predicates(&mut acc, run, Some(CheckLayer::Security));
        }
    }
    acc.finish(kind)
}

fn apply_selection_predicates(acc: &mut Accum, selection: &StandardSelection) {
    if nonempty(&selection.standard) {
        acc.satisfy(ClaimPredicate::ExactStandardIdentified);
    } else {
        acc.block(
            ClaimBlockCode::StandardNotIdentified,
            vec![ev(
                "standard_selection.standard",
                Some("null"),
                Some("identified GA4GH product id"),
            )],
        );
    }

    if selection.mode == "unversioned" || selection.selection_status == UNVERSIONED {
        acc.block(
            ClaimBlockCode::UnversionedRun,
            vec![ev(
                "standard_selection.mode",
                Some(selection.mode.as_str()),
                Some("explicit or automatic with SELECTED"),
            )],
        );
    }

    match selection.selection_status.as_str() {
        s if s == SELECTED => acc.satisfy(ClaimPredicate::SupportedReleaseSelected),
        s if s == AVAILABLE_BUT_NOT_SUPPORTED => acc.block(
            ClaimBlockCode::AvailableButNotSupported,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == NOT_SUPPORTED => acc.block(
            ClaimBlockCode::NotSupported,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == UNKNOWN_TO_HELIX => acc.block(
            ClaimBlockCode::UnknownVersion,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == NO_OFFICIAL_SUPPORTED => acc.block(
            ClaimBlockCode::NoOfficialSupported,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == DEVELOPMENT_NOT_SELECTABLE => acc.block(
            ClaimBlockCode::DevelopmentNotSelectable,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == AMBIGUOUS => acc.block(
            ClaimBlockCode::AmbiguousSelection,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == NEEDS_RELEASE_CLASS => acc.block(
            ClaimBlockCode::NeedsReleaseClass,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == INSUFFICIENT => acc.block(
            ClaimBlockCode::InsufficientSelection,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == MULTIPLE_PACKS_NOT_EXECUTABLE => acc.block(
            ClaimBlockCode::MultiplePacksNotExecutable,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
        s if s == UNVERSIONED => {}
        s => acc.block(
            ClaimBlockCode::NoVersionSelected,
            vec![ev(
                "standard_selection.selection_status",
                Some(s),
                Some(SELECTED),
            )],
        ),
    }

    if !nonempty(&selection.selected_version) {
        acc.block(
            ClaimBlockCode::NoVersionSelected,
            vec![ev(
                "standard_selection.selected_version",
                Some("null"),
                Some("exact supported release"),
            )],
        );
    }

    if nonempty(&selection.standards_registry_entry) && nonempty(&selection.standards_source_commit)
    {
        acc.satisfy(ClaimPredicate::PinnedSpecificationSource);
    } else {
        acc.block(
            ClaimBlockCode::ProvenanceMissing,
            vec![ev(
                "standard_selection.standards_registry_entry",
                selection
                    .standards_registry_entry
                    .as_deref()
                    .or(Some("null")),
                Some("pack_id and source commit"),
            )],
        );
    }

    match selection.integrity_ok {
        Some(false) => acc.block(
            ClaimBlockCode::IntegrityMismatch,
            vec![ev(
                "standard_selection.integrity_ok",
                Some("false"),
                Some("true"),
            )],
        ),
        Some(true) if selection.integrity_validated => {
            acc.satisfy(ClaimPredicate::IntegrityValidationSuccessful);
        }
        _ => acc.block(
            ClaimBlockCode::IntegrityValidationNotRecorded,
            vec![ev(
                "standard_selection.integrity_validated",
                Some(if selection.integrity_validated {
                    "true"
                } else {
                    "false"
                }),
                Some("true with integrity_ok true"),
            )],
        ),
    }

    match (
        selection.selected_version.as_deref(),
        selection.verified_version.as_deref(),
    ) {
        (Some(a), Some(b)) if a == b && !a.is_empty() => {
            acc.satisfy(ClaimPredicate::SelectedEqualsTested);
        }
        (None, None) | (Some(""), Some("")) => {}
        (sel, ver) => acc.block(
            ClaimBlockCode::SelectedNeVerified,
            vec![ClaimEvidence {
                field: "standard_selection.selected_version".into(),
                check_id: None,
                observed: Some(format!(
                    "selected={} verified={}",
                    sel.unwrap_or("null"),
                    ver.unwrap_or("null")
                )),
                expected: Some("selected_version equals verified_version".into()),
                value: None,
            }],
        ),
    }

    if selection.substituted {
        acc.block(
            ClaimBlockCode::SubstitutionDetected,
            vec![ev(
                "standard_selection.substituted",
                Some("true"),
                Some("false"),
            )],
        );
    } else {
        acc.satisfy(ClaimPredicate::NoSubstitution);
    }
}

fn apply_evidence_recorded(acc: &mut Accum, run: &VerificationRun) {
    let any = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .any(|r| !r.id.is_empty());
    if any {
        acc.satisfy(ClaimPredicate::EvidenceRecorded);
    } else {
        acc.block(
            ClaimBlockCode::NoChecksRecorded,
            vec![ev(
                "executed|skipped",
                Some("empty"),
                Some("at least one recorded check"),
            )],
        );
    }
}

fn apply_normative_predicates(acc: &mut Accum, run: &VerificationRun, layer: Option<CheckLayer>) {
    let rows: Vec<&VerificationResult> = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .filter(|r| is_normative_requirement(r) && layer_matches(r, layer))
        .collect();

    if rows.is_empty() {
        acc.block(
            ClaimBlockCode::NoNormativeChecks,
            vec![ev(
                "traceability.category",
                Some("no BindingKind::Normative rows"),
                Some("at least one executed normative check"),
            )],
        );
        return;
    }

    let skipped: Vec<&VerificationResult> = rows
        .iter()
        .copied()
        .filter(|r| r.status == VerificationStatus::Skip)
        .collect();
    let executed: Vec<&VerificationResult> = rows
        .iter()
        .copied()
        .filter(|r| r.status != VerificationStatus::Skip)
        .collect();

    if !skipped.is_empty() {
        let evidence = skipped
            .iter()
            .map(|r| ClaimEvidence {
                field: "skipped[].status".into(),
                check_id: Some(r.id.clone()),
                observed: Some(status_wire(r.status).into()),
                expected: Some(status_wire(VerificationStatus::Pass).into()),
                value: None,
            })
            .collect();
        acc.block(ClaimBlockCode::IncompleteNormativeCoverage, evidence);
    }

    if executed.is_empty() {
        return;
    }

    acc.satisfy(ClaimPredicate::RequiredNormativeChecksExecuted);

    let mut failed = Vec::new();
    let mut errored = Vec::new();
    let mut all_passed = true;
    for r in &executed {
        match r.status {
            VerificationStatus::Pass => {}
            VerificationStatus::Fail => {
                all_passed = false;
                failed.push(*r);
            }
            VerificationStatus::Error => {
                all_passed = false;
                errored.push(*r);
            }
            VerificationStatus::Skip => {}
        }
    }

    if failed.is_empty() && errored.is_empty() {
        acc.satisfy(ClaimPredicate::NoBlockingNormativeFailures);
    }
    if all_passed && failed.is_empty() && errored.is_empty() {
        acc.satisfy(ClaimPredicate::RequiredNormativeChecksPassed);
    }
    if skipped.is_empty() && all_passed {
        acc.satisfy(ClaimPredicate::CoverageRequirementsSatisfied);
    }

    if !failed.is_empty() {
        acc.block(
            ClaimBlockCode::NormativeCheckFailed,
            failed.iter().map(|r| check_status_ev(r)).collect(),
        );
    }
    if !errored.is_empty() {
        acc.block(
            ClaimBlockCode::NormativeCheckError,
            errored.iter().map(|r| check_status_ev(r)).collect(),
        );
    }
}

/// Structured taxonomy only. Never inspects `message` for PASS/FAIL.
fn is_normative_requirement(r: &VerificationResult) -> bool {
    let Some(t) = r.traceability.as_ref() else {
        return false;
    };
    t.category == BindingKind::Normative
        && t.check_kind == BindingKind::Normative
        && t.claim_scope == ClaimScope::Ga4ghRequirement
        && t.authority == Authority::Ga4gh
}

fn layer_matches(r: &VerificationResult, filter: Option<CheckLayer>) -> bool {
    match filter {
        None => true,
        Some(want) => result_layer(r) == want,
    }
}

fn result_layer(r: &VerificationResult) -> CheckLayer {
    r.layer
        .or_else(|| r.traceability.as_ref().map(|t| t.layer))
        .unwrap_or_else(|| crate::layer::for_id(&r.id))
}

fn status_wire(status: VerificationStatus) -> &'static str {
    match status {
        VerificationStatus::Pass => "pass",
        VerificationStatus::Fail => "fail",
        VerificationStatus::Skip => "skip",
        VerificationStatus::Error => "error",
    }
}

fn nonempty(v: &Option<String>) -> bool {
    v.as_deref().map(|s| !s.is_empty()).unwrap_or(false)
}

fn ev(field: &str, observed: Option<&str>, expected: Option<&str>) -> ClaimEvidence {
    ClaimEvidence {
        field: field.to_string(),
        check_id: None,
        observed: observed.map(str::to_string),
        expected: expected.map(str::to_string),
        value: None,
    }
}

fn check_status_ev(r: &VerificationResult) -> ClaimEvidence {
    ClaimEvidence {
        field: "executed[].status".into(),
        check_id: Some(r.id.clone()),
        observed: Some(status_wire(r.status).into()),
        expected: Some(status_wire(VerificationStatus::Pass).into()),
        value: None,
    }
}

/// Human Claims section. Generated only from [`evaluate`]. Not a PASS/FAIL grep.
pub fn format_claims_section(run: &VerificationRun, color: bool) -> String {
    format_claim_set(&evaluate(run), color)
}

pub fn format_claim_set(set: &ClaimSet, color: bool) -> String {
    let mut out = String::new();
    out.push_str("Claims (predicates; not GA4GH certification):\n");
    if set.any_verified() {
        out.push_str(&format!(
            "  {} VERIFIED claim(s) justified by recorded predicates. It is not GA4GH certification.\n",
            set.verified_count()
        ));
    } else {
        out.push_str("  No VERIFIED claim is justified by this run.\n");
    }
    out.push('\n');
    for claim in &set.items {
        let mark = paint_status(claim.status, color);
        out.push_str(&format!("  {}  {mark}\n", claim.kind.as_str()));
        if claim.status == ClaimStatus::Verified {
            out.push_str("    Why verified:\n");
            for p in &claim.satisfied {
                out.push_str(&format!("      - {}\n", p.as_str()));
            }
        } else {
            out.push_str("    Why not verified:\n");
            if claim.blocks.is_empty() {
                out.push_str("      - (no predicate satisfied; fail closed)\n");
            }
            for block in &claim.blocks {
                out.push_str(&format!("      - {}\n", block.code.as_str()));
                for e in &block.evidence {
                    out.push_str(&format!(
                        "          field: {}\n",
                        sanitize_untrusted(&e.field)
                    ));
                    if let Some(id) = &e.check_id {
                        out.push_str(&format!("          check_id: {}\n", sanitize_untrusted(id)));
                    }
                    if let Some(obs) = &e.observed {
                        out.push_str(&format!(
                            "          observed: {}\n",
                            sanitize_untrusted(obs)
                        ));
                    }
                    if let Some(exp) = &e.expected {
                        out.push_str(&format!(
                            "          expected: {}\n",
                            sanitize_untrusted(exp)
                        ));
                    }
                    if let Some(val) = &e.value {
                        out.push_str(&format!("          value: {}\n", sanitize_untrusted(val)));
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}

fn paint_status(status: ClaimStatus, color: bool) -> String {
    let plain = status.report_mark();
    if !color {
        return plain.to_string();
    }
    let paint = match status {
        ClaimStatus::Verified => GREEN,
        ClaimStatus::NotVerified => RED,
    };
    format!("{paint}{plain}{RESET}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::spec;
    use crate::model::{Target, VerificationCheck, VerificationResult};

    fn run_blank() -> VerificationRun {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        run.timestamp = "2026-09-05T00:00:00Z".into();
        run
    }

    fn selected_ok() -> StandardSelection {
        StandardSelection {
            mode: "explicit".into(),
            selection_status: SELECTED.into(),
            substituted: false,
            standard: Some("drs".into()),
            requested_version: Some("1.4.0".into()),
            detected_version: Some("1.4.0".into()),
            selected_version: Some("1.4.0".into()),
            verified_version: Some("1.4.0".into()),
            standards_registry_entry: Some("ga4gh.drs.1.4.0".into()),
            standards_source_commit: Some("abc123def456".into()),
            other_rows_not_selected: Vec::new(),
            note: None,
            integrity_validated: true,
            integrity_ok: Some(true),
            pack_integrity_sha256: Some("a".repeat(64)),
            schema_document_sha256: Some("b".repeat(64)),
            schema_component_sha256: Some("c".repeat(64)),
            execution_id: Some("d".repeat(64)),
            schema_entry: Some("openapi/components/schemas/DrsObject.yaml".into()),
            ..crate::model::StandardSelection::unversioned()
        }
    }

    fn mark_normative(r: &mut VerificationResult, layer: CheckLayer) {
        r.layer = Some(layer);
        if let Some(t) = r.traceability.as_mut() {
            t.category = BindingKind::Normative;
            t.check_kind = BindingKind::Normative;
            t.claim_scope = ClaimScope::Ga4ghRequirement;
            t.authority = Authority::Ga4gh;
            t.untraceable_reason = None;
            t.layer = layer;
            t.version = Some("1.4.0".into());
            t.registry_entry = Some("ga4gh.drs.1.4.0".into());
            t.source_commit = Some("abc123def456".into());
        }
    }

    fn normative_pass(id: &str, layer: CheckLayer) -> VerificationResult {
        let mut r = VerificationResult::pass(VerificationCheck::from_spec(spec(id)));
        mark_normative(&mut r, layer);
        r
    }

    fn with_selection(mut run: VerificationRun, sel: StandardSelection) -> VerificationRun {
        run.standard_selection = Some(sel);
        run
    }

    #[test]
    fn six_claims_always_in_fixed_order() {
        let set = evaluate(&run_blank());
        let kinds: Vec<_> = set.items.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, CLAIM_KINDS.to_vec());
        assert_eq!(set.items.len(), 6);
    }

    #[test]
    fn empty_run_is_not_verified() {
        let set = evaluate(&run_blank());
        assert!(!set.any_verified());
        let ga4gh = set.get(ClaimKind::Ga4ghRequirement);
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::UnversionedRun));
        assert!(ga4gh.has_block(ClaimBlockCode::NoVersionSelected));
        assert!(ga4gh.has_block(ClaimBlockCode::NoNormativeChecks));
        assert!(ga4gh.has_block(ClaimBlockCode::NoChecksRecorded));
        assert!(!ga4gh.has_block(ClaimBlockCode::NormativeCheckFailed));
    }

    #[test]
    fn no_version_yields_no_verification_claim() {
        let mut run = run_blank();
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            spec("drs.object.schema"),
        )));
        let set = evaluate(&run);
        assert!(!set.any_verified());
        assert!(set
            .get(ClaimKind::Ga4ghRequirement)
            .has_block(ClaimBlockCode::UnversionedRun));
        assert!(set
            .get(ClaimKind::Schema)
            .has_block(ClaimBlockCode::NoVersionSelected));
    }

    #[test]
    fn available_but_unsupported_yields_no_verification_claim() {
        let mut sel = selected_ok();
        sel.selection_status = AVAILABLE_BUT_NOT_SUPPORTED.into();
        sel.selected_version = None;
        sel.verified_version = None;
        sel.integrity_validated = false;
        sel.integrity_ok = None;
        let mut run = with_selection(run_blank(), sel);
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::AvailableButNotSupported));
        assert!(!ga4gh.has_block(ClaimBlockCode::NormativeCheckFailed));
    }

    #[test]
    fn selected_ne_tested_is_not_verified() {
        let mut sel = selected_ok();
        sel.verified_version = Some("1.5.0".into());
        let mut run = with_selection(run_blank(), sel);
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::SelectedNeVerified));
    }

    #[test]
    fn fixture_failure_is_not_a_normative_failure_claim() {
        let mut run = run_blank();
        run.push_executed(VerificationResult::fail(
            VerificationCheck::from_spec(spec("drs.object.schema")),
            "FAIL: this text must not be grepped into a MUST",
        ));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert!(!ga4gh.has_block(ClaimBlockCode::NormativeCheckFailed));
        assert!(!ga4gh.has_block(ClaimBlockCode::NormativeCheckError));
        assert!(ga4gh.has_block(ClaimBlockCode::NoNormativeChecks));
        let schema = evaluate(&run).get(ClaimKind::Schema).clone();
        assert!(!schema.has_block(ClaimBlockCode::NormativeCheckFailed));
    }

    #[test]
    fn incomplete_normative_coverage_blocks_full_verification() {
        let mut run = with_selection(run_blank(), selected_ok());
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let mut skipped = VerificationResult::skip(
            VerificationCheck::from_spec(spec("drs.object.not_found")),
            "not executed",
        );
        mark_normative(&mut skipped, CheckLayer::Behavior);
        run.push_skipped(skipped);
        let set = evaluate(&run);
        let ga4gh = set.get(ClaimKind::Ga4ghRequirement);
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::IncompleteNormativeCoverage));
        let schema = set.get(ClaimKind::Schema);
        assert_eq!(schema.status, ClaimStatus::Verified);
        let behavior = set.get(ClaimKind::Behavior);
        assert_eq!(behavior.status, ClaimStatus::NotVerified);
        assert!(behavior.has_block(ClaimBlockCode::IncompleteNormativeCoverage));
    }

    #[test]
    fn provenance_missing_blocks_verification() {
        let mut sel = selected_ok();
        sel.standards_registry_entry = None;
        sel.standards_source_commit = None;
        let mut run = with_selection(run_blank(), sel);
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::ProvenanceMissing));
    }

    #[test]
    fn integrity_mismatch_blocks_verification() {
        let mut sel = selected_ok();
        sel.integrity_ok = Some(false);
        let mut run = with_selection(run_blank(), sel);
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::IntegrityMismatch));
    }

    #[test]
    fn integrity_not_recorded_blocks_verification() {
        let mut sel = selected_ok();
        sel.integrity_validated = false;
        sel.integrity_ok = None;
        let mut run = with_selection(run_blank(), sel);
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert_eq!(ga4gh.status, ClaimStatus::NotVerified);
        assert!(ga4gh.has_block(ClaimBlockCode::IntegrityValidationNotRecorded));
    }

    #[test]
    fn synthetic_predicates_can_issue_verified_without_shipping_catalog_normative() {
        let mut run = with_selection(run_blank(), selected_ok());
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let set = evaluate(&run);
        assert_eq!(
            set.get(ClaimKind::Ga4ghRequirement).status,
            ClaimStatus::Verified
        );
        assert_eq!(set.get(ClaimKind::Schema).status, ClaimStatus::Verified);
        assert!(set.get(ClaimKind::Ga4ghRequirement).blocks.is_empty());
        assert_eq!(
            set.get(ClaimKind::Behavior).status,
            ClaimStatus::NotVerified
        );
        assert!(set
            .get(ClaimKind::Behavior)
            .has_block(ClaimBlockCode::NoNormativeChecks));
        assert_eq!(
            set.get(ClaimKind::Security).status,
            ClaimStatus::NotVerified
        );
        assert_eq!(
            set.get(ClaimKind::Interoperability).status,
            ClaimStatus::NotVerified
        );
        assert!(set
            .get(ClaimKind::Interoperability)
            .has_block(ClaimBlockCode::InteroperabilityIsNotAGa4ghRequirement));
        assert_eq!(
            set.get(ClaimKind::Benchmark).status,
            ClaimStatus::NotVerified
        );
        assert!(set
            .get(ClaimKind::Benchmark)
            .has_block(ClaimBlockCode::BenchmarkIsMeasurementOnly));
    }

    #[test]
    fn normative_fail_blocks_with_check_evidence() {
        let mut run = with_selection(run_blank(), selected_ok());
        let mut r = VerificationResult::fail(
            VerificationCheck::from_spec(spec("drs.object.schema")),
            "body mismatch",
        );
        mark_normative(&mut r, CheckLayer::Schema);
        run.push_executed(r);
        let ga4gh = evaluate(&run).get(ClaimKind::Ga4ghRequirement).clone();
        assert!(ga4gh.has_block(ClaimBlockCode::NormativeCheckFailed));
        let ev = &ga4gh
            .blocks
            .iter()
            .find(|b| b.code == ClaimBlockCode::NormativeCheckFailed)
            .unwrap()
            .evidence[0];
        assert_eq!(ev.check_id.as_deref(), Some("drs.object.schema"));
        assert_eq!(ev.observed.as_deref(), Some("fail"));
        assert_eq!(ev.expected.as_deref(), Some("pass"));
    }

    #[test]
    fn claims_are_not_collapsed() {
        let mut run = with_selection(run_blank(), selected_ok());
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        let set = evaluate(&run);
        assert_ne!(
            set.get(ClaimKind::Schema).status,
            set.get(ClaimKind::Behavior).status
        );
        assert_ne!(
            set.get(ClaimKind::Ga4ghRequirement).kind,
            set.get(ClaimKind::Security).kind
        );
    }

    #[test]
    fn human_text_comes_from_claim_model_not_result_marks() {
        let mut run = run_blank();
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            spec("drs.object.schema"),
        )));
        let text = format_claims_section(&run, false);
        assert!(text.contains("NOT_VERIFIED"));
        assert!(text.contains("Why not verified:"));
        assert!(text.contains("unversioned_run"));
        assert!(text.contains("No VERIFIED claim is justified by this run."));
        assert!(!text.contains("  schema  VERIFIED"));
        assert!(!text.contains("  ga4gh_requirement  VERIFIED"));
        let set = evaluate(&run);
        assert!(!set.any_verified());
        for claim in &set.items {
            assert_eq!(claim.status, ClaimStatus::NotVerified);
            assert_eq!(claim.status.report_mark(), "NOT_VERIFIED");
        }
    }

    #[test]
    fn evaluate_is_deterministic() {
        let mut run = with_selection(run_blank(), selected_ok());
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        assert_eq!(evaluate(&run), evaluate(&run));
    }

    #[test]
    fn check_set_rejects_verified_without_predicates() {
        let mut set = evaluate(&run_blank());
        set.items[0].status = ClaimStatus::Verified;
        set.items[0].blocks.clear();
        set.items[0].satisfied.clear();
        let err = check_set(&set).unwrap_err().to_string();
        assert!(err.contains("missing predicate"), "{err}");
    }

    #[test]
    fn check_set_rejects_verified_interoperability() {
        let mut set = evaluate(&run_blank());
        let interop = set
            .items
            .iter_mut()
            .find(|c| c.kind == ClaimKind::Interoperability)
            .unwrap();
        interop.status = ClaimStatus::Verified;
        interop.blocks.clear();
        let err = check_set(&set).unwrap_err().to_string();
        assert!(err.contains("interoperability"), "{err}");
    }

    #[test]
    fn evaluate_output_always_passes_check_set() {
        check_set(&evaluate(&run_blank())).expect("blank");
        let mut run = with_selection(run_blank(), selected_ok());
        run.push_executed(normative_pass("drs.object.schema", CheckLayer::Schema));
        check_set(&evaluate(&run)).expect("constructed");
    }
}
