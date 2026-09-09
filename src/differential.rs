// SPDX-License-Identifier: Apache-2.0
//! Check-level differential of two `VerificationRun`s. Descriptive only.
//!
//! Does not create verification. Does not stamp `verified_version`.
//! Does not rank implementations. Not HELIOS. Not certification.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::claims::{evaluate, ClaimKind, ClaimStatus};
use crate::compare::load_verification_run;
use crate::coverage::CoverageReport;
use crate::evidence::{classify_evidence, EvidenceStanding};
use crate::independence::{reviewed_record, run_counts_as_independent};
use crate::model::{helix_version, VerificationResult, VerificationRun, VerificationStatus};
use crate::target::{FailureAttribution, TargetKind};

pub const DIFFERENTIAL_SCHEMA_VERSION: &str = "helix-differential-v1";

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
    "market comparison",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceClass {
    SameBehavior,
    TargetBehaviorDifference,
    FixtureCapabilityDifference,
    TargetConfigurationDifference,
    EnvironmentDifference,
    VerificationExecutionDifference,
    InsufficientEvidence,
}

impl DifferenceClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SameBehavior => "same_behavior",
            Self::TargetBehaviorDifference => "target_behavior_difference",
            Self::FixtureCapabilityDifference => "fixture_capability_difference",
            Self::TargetConfigurationDifference => "target_configuration_difference",
            Self::EnvironmentDifference => "environment_difference",
            Self::VerificationExecutionDifference => "verification_execution_difference",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckSnapshot {
    pub check_id: String,
    pub code: String,
    pub status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<FailureAttribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialTarget {
    pub target_id: String,
    pub target_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_artifact_identity: Option<String>,
    pub independent_evidence: bool,
    /// Computed standing of this input. Not stored on the verify JSON. Not HELIOS.
    pub evidence_standing: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_version: Option<String>,
    pub ga4gh_requirement: String,
    pub checks: Vec<CheckSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckDifferential {
    pub check_id: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_status: Option<VerificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_status: Option<VerificationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_attribution: Option<FailureAttribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_attribution: Option<FailureAttribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_diagnostic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b_diagnostic: Option<String>,
    pub class: DifferenceClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialReport {
    pub schema_version: String,
    pub helix_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_version: Option<String>,
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
    pub coverage_id: Option<String>,
    pub same_verifier: bool,
    pub same_specification: bool,
    pub independent_implementation_evidence: bool,
    /// Always false. A differential artifact is descriptive.
    pub creates_verification: bool,
    pub ranking_semantics: String,
    pub targets: Vec<DifferentialTarget>,
    pub differences: Vec<CheckDifferential>,
}

impl DifferentialReport {
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

pub fn classify_difference(
    a: Option<&VerificationResult>,
    b: Option<&VerificationResult>,
) -> DifferenceClass {
    match (a, b) {
        (None, None) | (None, Some(_)) | (Some(_), None) => DifferenceClass::InsufficientEvidence,
        (Some(a), Some(b)) => {
            if a.status == b.status && a.attribution == b.attribution {
                return DifferenceClass::SameBehavior;
            }
            if a.attribution == Some(FailureAttribution::HelixExecutionFailure)
                || b.attribution == Some(FailureAttribution::HelixExecutionFailure)
            {
                return DifferenceClass::VerificationExecutionDifference;
            }
            if a.attribution == Some(FailureAttribution::TransportFailure)
                || b.attribution == Some(FailureAttribution::TransportFailure)
            {
                return DifferenceClass::EnvironmentDifference;
            }
            if fixture_skip(a) != fixture_skip(b) {
                return DifferenceClass::FixtureCapabilityDifference;
            }
            if config_skip(a) != config_skip(b) {
                return DifferenceClass::TargetConfigurationDifference;
            }
            DifferenceClass::TargetBehaviorDifference
        }
    }
}

fn fixture_skip(r: &VerificationResult) -> bool {
    r.status == VerificationStatus::Skip
        && r.message
            .as_deref()
            .is_some_and(|m| m.contains(framework::drs::FIXTURE_UNAVAILABLE))
}

fn config_skip(r: &VerificationResult) -> bool {
    r.attribution == Some(FailureAttribution::TargetConfigurationFailure) && !fixture_skip(r)
}

fn diagnostic_of(r: &VerificationResult) -> Option<String> {
    r.message.clone()
}

fn status_label(status: VerificationStatus) -> &'static str {
    match status {
        VerificationStatus::Pass => "PASS",
        VerificationStatus::Fail => "FAIL",
        VerificationStatus::Skip => "SKIP",
        VerificationStatus::Error => "ERROR",
    }
}

fn index_results(run: &VerificationRun) -> std::collections::BTreeMap<String, &VerificationResult> {
    let mut map = std::collections::BTreeMap::new();
    for r in run.executed.iter().chain(run.skipped.iter()) {
        map.insert(r.id.clone(), r);
    }
    map
}

fn snapshot_checks(run: &VerificationRun) -> Vec<CheckSnapshot> {
    let mut out: Vec<CheckSnapshot> = run
        .executed
        .iter()
        .chain(run.skipped.iter())
        .map(|r| CheckSnapshot {
            check_id: r.id.clone(),
            code: r.code.clone(),
            status: r.status,
            attribution: r.attribution,
            diagnostic: diagnostic_of(r),
        })
        .collect();
    out.sort_by(|a, b| a.check_id.cmp(&b.check_id));
    out
}

fn target_view(run: &VerificationRun) -> DifferentialTarget {
    let identity = run.target.identity.as_ref();
    let target_id = identity
        .map(|i| i.target_id.clone())
        .unwrap_or_else(|| format!("endpoint:{}", run.target.url));
    let kind = identity
        .map(|i| i.target_kind)
        .unwrap_or(TargetKind::Unspecified);
    let reviewed = reviewed_record(&target_id);
    let sel = run.standard_selection.as_ref();
    let ga4gh = evaluate(run).get(ClaimKind::Ga4ghRequirement).status;
    let coverage = CoverageReport::from_run(run);
    DifferentialTarget {
        target_id,
        target_kind: kind.as_str().to_string(),
        implementation_name: identity.and_then(|i| i.implementation_name.clone()),
        reviewed_classification: reviewed.map(|r| r.classification.as_str().to_string()),
        reviewed_artifact: reviewed.map(|r| r.artifact.clone()),
        reviewed_artifact_identity: reviewed.and_then(|r| r.artifact_identity.clone()),
        independent_evidence: run_counts_as_independent(run),
        evidence_standing: classify_evidence(run).as_str().to_string(),
        coverage_state: Some(coverage.state.as_str().to_string()),
        execution_id: sel.and_then(|s| s.execution_id.clone()),
        target_execution_id: sel.and_then(|s| s.target_execution_id.clone()),
        selected_version: sel.and_then(|s| s.selected_version.clone()),
        declared_version: identity.and_then(|i| i.declared.standard_version.clone()),
        detected_version: sel
            .and_then(|s| s.detected_version.clone())
            .or_else(|| identity.and_then(|i| i.detected.standard_version.clone())),
        verified_version: sel.and_then(|s| s.verified_version.clone()),
        ga4gh_requirement: match ga4gh {
            ClaimStatus::Verified => "verified".into(),
            ClaimStatus::NotVerified => "not_verified".into(),
        },
        checks: snapshot_checks(run),
    }
}

fn same_opt(a: Option<&String>, b: Option<&String>) -> bool {
    a.is_some() && a == b
}

/// Descriptive comparison. Does not evaluate or stamp claims onto either run.
pub fn differential_from_runs(a: &VerificationRun, b: &VerificationRun) -> DifferentialReport {
    let sa = a.standard_selection.as_ref();
    let sb = b.standard_selection.as_ref();
    let same_verifier = same_opt(
        sa.and_then(|s| s.checker_id.as_ref()),
        sb.and_then(|s| s.checker_id.as_ref()),
    );
    let same_specification = same_opt(
        sa.and_then(|s| s.pack_integrity_sha256.as_ref()),
        sb.and_then(|s| s.pack_integrity_sha256.as_ref()),
    ) && same_opt(
        sa.and_then(|s| s.schema_document_sha256.as_ref()),
        sb.and_then(|s| s.schema_document_sha256.as_ref()),
    ) && same_opt(
        sa.and_then(|s| s.schema_component_sha256.as_ref()),
        sb.and_then(|s| s.schema_component_sha256.as_ref()),
    ) && same_opt(
        sa.and_then(|s| s.binding_id.as_ref()),
        sb.and_then(|s| s.binding_id.as_ref()),
    ) && same_opt(
        sa.and_then(|s| s.catalog_id.as_ref()),
        sb.and_then(|s| s.catalog_id.as_ref()),
    ) && same_opt(
        sa.and_then(|s| s.standards_source_commit.as_ref()),
        sb.and_then(|s| s.standards_source_commit.as_ref()),
    );
    let ia = index_results(a);
    let ib = index_results(b);
    let mut ids: BTreeSet<String> = BTreeSet::new();
    ids.extend(ia.keys().cloned());
    ids.extend(ib.keys().cloned());
    let mut differences = Vec::new();
    for id in ids {
        let ra = ia.get(&id).copied();
        let rb = ib.get(&id).copied();
        let code = ra
            .or(rb)
            .map(|r| r.code.clone())
            .unwrap_or_else(|| id.clone());
        differences.push(CheckDifferential {
            check_id: id,
            code,
            a_status: ra.map(|r| r.status),
            b_status: rb.map(|r| r.status),
            a_attribution: ra.and_then(|r| r.attribution),
            b_attribution: rb.and_then(|r| r.attribution),
            a_diagnostic: ra.and_then(diagnostic_of),
            b_diagnostic: rb.and_then(diagnostic_of),
            class: classify_difference(ra, rb),
        });
    }
    DifferentialReport {
        schema_version: DIFFERENTIAL_SCHEMA_VERSION.into(),
        helix_version: helix_version().to_string(),
        standard: sa.and_then(|s| s.standard.clone()),
        selected_version: sa.and_then(|s| s.selected_version.clone()),
        release_commit: sa.and_then(|s| s.standards_source_commit.clone()),
        pack_integrity_sha256: sa.and_then(|s| s.pack_integrity_sha256.clone()),
        schema_document_sha256: sa.and_then(|s| s.schema_document_sha256.clone()),
        schema_component_sha256: sa.and_then(|s| s.schema_component_sha256.clone()),
        checker_id: sa.and_then(|s| s.checker_id.clone()),
        binding_id: sa.and_then(|s| s.binding_id.clone()),
        catalog_id: sa.and_then(|s| s.catalog_id.clone()),
        coverage_id: CoverageReport::from_run(a).coverage_id,
        same_verifier,
        same_specification,
        independent_implementation_evidence: run_counts_as_independent(a)
            && run_counts_as_independent(b),
        creates_verification: false,
        ranking_semantics: "absent".into(),
        targets: vec![target_view(a), target_view(b)],
        differences,
    }
}

/// Fail closed when two artifacts are not the same verification contract, or
/// when either side is invalid. Does not restamp either run. Not HELIOS.
pub fn assert_comparable_runs(a: &VerificationRun, b: &VerificationRun) -> Result<()> {
    match classify_evidence(a) {
        EvidenceStanding::Invalid => {
            bail!(
                "first artifact is invalid evidence; helix differential will not compare it as trustworthy"
            );
        }
        EvidenceStanding::HistoricalObservation | EvidenceStanding::CurrentVerifierEvidence => {}
    }
    match classify_evidence(b) {
        EvidenceStanding::Invalid => {
            bail!(
                "second artifact is invalid evidence; helix differential will not compare it as trustworthy"
            );
        }
        EvidenceStanding::HistoricalObservation | EvidenceStanding::CurrentVerifierEvidence => {}
    }

    let a_sel = a.standard_selection.as_ref();
    let b_sel = b.standard_selection.as_ref();
    let a_std = a_sel.and_then(|s| s.standard.as_deref());
    let b_std = b_sel.and_then(|s| s.standard.as_deref());
    match (a_std, b_std) {
        (Some(x), Some(y)) if x != y => {
            bail!(
                "artifacts select different standards ({x} vs {y}); they are not the same verification contract"
            );
        }
        (Some(_), None) | (None, Some(_)) => {
            bail!(
                "artifacts do not share a selected standard; they are not the same verification contract"
            );
        }
        _ => {}
    }
    let a_ver = a_sel.and_then(|s| s.selected_version.as_deref());
    let b_ver = b_sel.and_then(|s| s.selected_version.as_deref());
    match (a_ver, b_ver) {
        (Some(x), Some(y)) if x != y => {
            bail!(
                "artifacts select different versions ({x} vs {y}); they are not the same verification contract"
            );
        }
        (Some(_), None) | (None, Some(_)) => {
            bail!(
                "artifacts do not share a selected version; they are not the same verification contract"
            );
        }
        _ => {}
    }
    let a_exec = a_sel.and_then(|s| s.execution_id.as_deref());
    let b_exec = b_sel.and_then(|s| s.execution_id.as_deref());
    match (a_exec, b_exec) {
        (Some(x), Some(y)) if x != y => {
            bail!("execution_id values differ; these are not the same verification contract");
        }
        (Some(_), None) | (None, Some(_)) => {
            bail!(
                "execution_id is recorded on only one artifact; they are not the same verification contract"
            );
        }
        _ => {}
    }
    Ok(())
}

pub fn differential_files(a: &Path, b: &Path) -> Result<DifferentialReport> {
    let ra = load_verification_run(a)?;
    let rb = load_verification_run(b)?;
    assert_comparable_runs(&ra, &rb)?;
    let report = differential_from_runs(&ra, &rb);
    if report.contains_ranking_semantics() {
        bail!("differential output must not contain ranking semantics");
    }
    if report.creates_verification {
        bail!("differential artifact must not create verification");
    }
    crate::guardrails::forbid_helios_keys(&serde_json::to_value(&report)?)?;
    Ok(report)
}

fn ga4gh_label(status: &str) -> &'static str {
    if status == "verified" {
        "VERIFIED"
    } else {
        "NOT_VERIFIED"
    }
}

fn status_cell(
    status: Option<VerificationStatus>,
    attribution: Option<FailureAttribution>,
) -> String {
    match status {
        None => "ABSENT".into(),
        Some(s) => match attribution {
            Some(a) if s != VerificationStatus::Pass => {
                format!("{} ({})", status_label(s), a.as_str())
            }
            _ => status_label(s).to_string(),
        },
    }
}

pub fn format_differential_text(report: &DifferentialReport) -> String {
    let mut out = String::new();
    out.push_str("HELIX TARGET DIFFERENTIAL\n");
    out.push('\n');
    out.push_str(
        "This interprets two helix-verification-v1 artifacts under one verification contract.\n",
    );
    out.push_str("It does not create verification. It does not rank implementations.\n");
    out.push_str("PASS is not ga4gh_requirement VERIFIED. A check difference is behavioural, not a ranking.\n");
    out.push_str("It is not GA4GH certification.\n");
    out.push('\n');

    out.push_str("Verification contract\n");
    out.push_str(&format!(
        "  standard: {}\n",
        report.standard.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!(
        "  selected_version: {}\n",
        report.selected_version.as_deref().unwrap_or("(none)")
    ));
    let a_exec = report
        .targets
        .first()
        .and_then(|t| t.execution_id.as_deref());
    let b_exec = report
        .targets
        .get(1)
        .and_then(|t| t.execution_id.as_deref());
    let same_exec = a_exec.is_some() && a_exec == b_exec;
    out.push_str(&format!("  execution_id: {}\n", a_exec.unwrap_or("(none)")));
    out.push_str(&format!(
        "  same_execution_id: {}\n",
        if same_exec { "yes" } else { "no" }
    ));
    out.push_str(&format!(
        "  coverage_id: {}\n",
        report.coverage_id.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!(
        "  pack_integrity: {}\n",
        report.pack_integrity_sha256.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!(
        "  checker_id: {}\n",
        report.checker_id.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!(
        "  same_verifier: {}\n",
        if report.same_verifier { "yes" } else { "no" }
    ));
    out.push_str(&format!(
        "  same_specification: {}\n",
        if report.same_specification {
            "yes"
        } else {
            "no"
        }
    ));
    out.push_str(&format!(
        "  independent_implementation_evidence: {}\n",
        if report.independent_implementation_evidence {
            "yes"
        } else {
            "no"
        }
    ));
    out.push('\n');

    for t in &report.targets {
        out.push_str(&format!("Target {}\n", t.target_id));
        out.push_str(&format!(
            "  implementation: {}\n",
            t.implementation_name.as_deref().unwrap_or("(undeclared)")
        ));
        out.push_str(&format!("  target_kind: {}\n", t.target_kind));
        out.push_str(&format!("  evidence_standing: {}\n", t.evidence_standing));
        out.push_str(&format!(
            "  ga4gh_requirement: {}\n",
            ga4gh_label(&t.ga4gh_requirement)
        ));
        out.push_str(&format!(
            "  coverage: {}\n",
            t.coverage_state.as_deref().unwrap_or("(none)")
        ));
        out.push_str(&format!(
            "  target_execution_id: {}\n",
            t.target_execution_id.as_deref().unwrap_or("(none)")
        ));
        out.push_str(&format!(
            "  verified_version: {}\n",
            t.verified_version.as_deref().unwrap_or("(none)")
        ));
        out.push_str(&format!(
            "  declared_version: {}\n",
            t.declared_version.as_deref().unwrap_or("(none)")
        ));
        out.push_str(&format!(
            "  detected_version: {}\n",
            t.detected_version.as_deref().unwrap_or("(none)")
        ));
        out.push_str(&format!(
            "  independent_evidence: {}\n",
            t.independent_evidence
        ));
        out.push('\n');
    }

    let a_id = report
        .targets
        .first()
        .map(|t| t.target_id.as_str())
        .unwrap_or("target-0");
    let b_id = report
        .targets
        .get(1)
        .map(|t| t.target_id.as_str())
        .unwrap_or("target-1");
    let a_te = report
        .targets
        .first()
        .and_then(|t| t.target_execution_id.as_deref());
    let b_te = report
        .targets
        .get(1)
        .and_then(|t| t.target_execution_id.as_deref());
    let distinct_te = a_te.is_some() && b_te.is_some() && a_te != b_te;

    out.push_str("Check comparison (joined by check_id; missing is ABSENT, not SKIP)\n");
    out.push_str(&format!("  left:  {a_id}\n"));
    out.push_str(&format!("  right: {b_id}\n"));
    for d in &report.differences {
        let as_ = status_cell(d.a_status, d.a_attribution);
        let bs = status_cell(d.b_status, d.b_attribution);
        out.push_str(&format!("  {}  {}  {as_} | {bs}\n", d.check_id, d.code));
    }
    out.push('\n');

    out.push_str("Attribution (existing model; not every non-PASS is target_failure)\n");
    for d in report
        .differences
        .iter()
        .filter(|d| d.class != DifferenceClass::SameBehavior)
    {
        out.push_str(&format!("  {}: {}\n", d.check_id, d.class.as_str()));
        if let Some(attr) = d.a_attribution {
            out.push_str(&format!("    {a_id} attribution: {}\n", attr.as_str()));
        }
        if let Some(attr) = d.b_attribution {
            out.push_str(&format!("    {b_id} attribution: {}\n", attr.as_str()));
        }
        if let Some(msg) = &d.a_diagnostic {
            out.push_str(&format!("    {a_id}: {msg}\n"));
        }
        if let Some(msg) = &d.b_diagnostic {
            out.push_str(&format!("    {b_id}: {msg}\n"));
        }
    }
    out.push('\n');

    let mut same = 0usize;
    let mut behavior = 0usize;
    let mut fixture = 0usize;
    let mut config = 0usize;
    let mut env = 0usize;
    let mut exec = 0usize;
    let mut absent = 0usize;
    for d in &report.differences {
        match d.class {
            DifferenceClass::SameBehavior => same += 1,
            DifferenceClass::TargetBehaviorDifference => behavior += 1,
            DifferenceClass::FixtureCapabilityDifference => fixture += 1,
            DifferenceClass::TargetConfigurationDifference => config += 1,
            DifferenceClass::EnvironmentDifference => env += 1,
            DifferenceClass::VerificationExecutionDifference => exec += 1,
            DifferenceClass::InsufficientEvidence => absent += 1,
        }
    }
    out.push_str("Interpretation\n");
    out.push_str(&format!("  same_behavior: {same}\n"));
    out.push_str(&format!("  target_behavior_difference: {behavior}\n"));
    out.push_str(&format!("  fixture_capability_difference: {fixture}\n"));
    out.push_str(&format!("  target_configuration_difference: {config}\n"));
    out.push_str(&format!("  environment_difference: {env}\n"));
    out.push_str(&format!("  verification_execution_difference: {exec}\n"));
    out.push_str(&format!(
        "  insufficient_evidence (ABSENT on one side): {absent}\n"
    ));
    out.push_str(&format!(
        "  distinct_target_execution_id: {}\n",
        if distinct_te { "yes" } else { "no" }
    ));
    out.push_str("  VERIFIED / NOT_VERIFIED is the derived claim, not a count of PASS rows.\n");
    out.push_str("  Observed differences are behavioural. Helix does not rank implementations.\n");
    out.push_str("  creates_verification: no\n");
    out.push_str("  ranking_semantics: absent\n");
    out
}

pub fn differential_json(report: &DifferentialReport) -> Result<String> {
    if report.contains_ranking_semantics() {
        bail!("differential output must not contain ranking semantics");
    }
    crate::guardrails::forbid_helios_keys(&serde_json::to_value(report)?)?;
    Ok(crate::redact::redact_text(&serde_json::to_string_pretty(
        report,
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity;
    use crate::model::{Target, VerificationCheck, VerificationResult, VerificationRun};
    use crate::target::{DeclaredTarget, TargetIdentity, TargetKind};

    fn check(id: &str) -> VerificationCheck {
        VerificationCheck::from_spec(identity::spec(id)).with_profile("generic")
    }

    fn run(id: &str, kind: TargetKind) -> VerificationRun {
        VerificationRun::new(Target::from_identity(TargetIdentity::from_declared(
            "http://127.0.0.1:9",
            &DeclaredTarget {
                target_id: Some(id.into()),
                kind,
                ..DeclaredTarget::default()
            },
        )))
    }

    #[test]
    fn skip_vs_pass_is_fixture_capability_when_fixture_unavailable() {
        let mut ra = VerificationResult::skip(
            check("drs.object.checksum"),
            "fixture_unavailable: no access_url",
        );
        crate::target::attach_attribution(&mut ra);
        let mut rb = VerificationResult::pass(check("drs.object.checksum"));
        crate::target::attach_attribution(&mut rb);
        assert_eq!(
            classify_difference(Some(&ra), Some(&rb)),
            DifferenceClass::FixtureCapabilityDifference
        );
    }

    #[test]
    fn fail_vs_pass_is_target_behavior() {
        let mut ra = VerificationResult::fail(check("drs.object.schema"), "access_methods missing");
        crate::target::attach_attribution(&mut ra);
        let mut rb = VerificationResult::pass(check("drs.object.schema"));
        crate::target::attach_attribution(&mut rb);
        assert_eq!(
            classify_difference(Some(&ra), Some(&rb)),
            DifferenceClass::TargetBehaviorDifference
        );
    }

    #[test]
    fn differential_does_not_create_verification() {
        let a = run("ga4gh-starter-kit-drs-0.3.2", TargetKind::Mock);
        let b = run("bento-drs-0.21.5", TargetKind::Mock);
        let report = differential_from_runs(&a, &b);
        assert!(!report.creates_verification);
        assert_eq!(report.ranking_semantics, "absent");
        assert!(!report.contains_ranking_semantics());
        assert!(!report.independent_implementation_evidence);
    }
}
