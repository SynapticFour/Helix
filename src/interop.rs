// SPDX-License-Identifier: Apache-2.0
//! Target-neutral interoperability matrix.
//!
//! The same `helix verify` suite (profile `generic`) is the contract. Labels
//! on `--run` are operator names, not branches in verify. An in-process mock
//! is not an independent implementation.
//!
//! External multi-implementation validation is **pending** until an operator
//! supplies a non-fixture run **and** a run labeled
//! `independent_implementation`. Not HELIOS. Not GA4GH certification.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::compare::load_verification_run;
use crate::identity::spec_by_id;
use crate::model::{VerificationRun, VerificationStatus};
use crate::sanitize::sanitize_untrusted;

pub const MATRIX_SCHEMA_VERSION: &str = "helix-interop-matrix-v1";

/// Operator-declared what a run JSON is. Not inferred from URL or service-info name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationKind {
    /// In-process Helix/HelixTest mock. Never independent evidence.
    HelixFixture,
    /// Live reference stack (e.g. Ferrum). Real target, not a second independent impl by itself.
    ReferenceTarget,
    /// Operator asserts this JSON came from a distinct implementation.
    IndependentImplementation,
    /// Kind not stated. Fail closed: does not count as independent evidence.
    Unspecified,
}

impl ImplementationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HelixFixture => "helix_fixture",
            Self::ReferenceTarget => "reference_target",
            Self::IndependentImplementation => "independent_implementation",
            Self::Unspecified => "unspecified",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "helix_fixture" => Ok(Self::HelixFixture),
            "reference_target" => Ok(Self::ReferenceTarget),
            "independent_implementation" => Ok(Self::IndependentImplementation),
            "unspecified" => Ok(Self::Unspecified),
            other => bail!(
                "unknown implementation kind `{other}` (helix_fixture|reference_target|independent_implementation|unspecified)"
            ),
        }
    }

    pub fn counts_as_independent_evidence(self) -> bool {
        matches!(self, Self::IndependentImplementation)
    }

    pub fn is_fixture(self) -> bool {
        matches!(self, Self::HelixFixture)
    }
}

/// How the external target contract classifies the check (not a score).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractClass {
    /// Behaviour the GA4GH DRS/WES spec requires under the external contract.
    Standard,
    /// Helix-defined fixture id or workflow URL. Not a MUST in the spec text.
    Fixture,
    /// Spec permits absence or variation.
    Optional,
    /// Today's HelixTest runner is stricter than the spec.
    RunnerExtra,
}

impl ContractClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Fixture => "fixture",
            Self::Optional => "optional",
            Self::RunnerExtra => "runner_extra",
        }
    }
}

/// Whether two implementations must produce the same executed outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossImpl {
    /// Spec requires the same observable (once the contract fixture is mounted).
    MustAgree,
    /// Spec permits difference. Must not be reported as a spec failure.
    MayDiffer,
}

impl CrossImpl {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MustAgree => "must_agree",
            Self::MayDiffer => "may_differ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    Pass,
    PassOrSkip,
}

impl ExpectedOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::PassOrSkip => "pass_or_skip",
        }
    }
}

/// Per-row result. `contract_violation` only when `must_agree` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatrixResult {
    Pending,
    MeetsContract,
    ContractViolation,
    FixtureUnsatisfied,
    RunnerStricter,
    NotExecuted,
}

impl MatrixResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::MeetsContract => "meets_contract",
            Self::ContractViolation => "contract_violation",
            Self::FixtureUnsatisfied => "fixture_unsatisfied",
            Self::RunnerStricter => "runner_stricter",
            Self::NotExecuted => "not_executed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscrepancyClass {
    Agree,
    ImplementationSpecific,
    UnresolvedDiscrepancy,
    Pending,
    NotComparable,
}

impl DiscrepancyClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agree => "agree",
            Self::ImplementationSpecific => "implementation_specific",
            Self::UnresolvedDiscrepancy => "unresolved_discrepancy",
            Self::Pending => "pending",
            Self::NotComparable => "not_comparable",
        }
    }
}

/// Hypothesis for an unresolved must-agree disagreement. Not auto-assigned as fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hypothesis {
    HelixBug,
    ImplementationBug,
    AmbiguousSpec,
}

impl Hypothesis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HelixBug => "helix_bug",
            Self::ImplementationBug => "implementation_bug",
            Self::AmbiguousSpec => "ambiguous_spec",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub standard: &'static str,
    pub contract: ContractClass,
    pub cross_impl: CrossImpl,
    pub expected: ExpectedOutcome,
    pub rationale: &'static str,
}

/// Checks the generic `helix verify` suite actually executes.
pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: "drs.object.reachable",
        standard: "drs",
        contract: ContractClass::Standard,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "GetObject 200 for the documented known id once that fixture is mounted",
    },
    CatalogEntry {
        id: "drs.object.schema",
        standard: "drs",
        contract: ContractClass::RunnerExtra,
        cross_impl: CrossImpl::MayDiffer,
        expected: ExpectedOutcome::Pass,
        rationale: "DrsObject required properties are standard; HelixTest also requires name and non-empty access_methods",
    },
    CatalogEntry {
        id: "drs.object.checksum",
        standard: "drs",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "Checksums are standard; the sha256/blob pair is the Helix fixture. Implementations that mount the fixture must match bytes",
    },
    CatalogEntry {
        id: "drs.object.range",
        standard: "drs",
        contract: ContractClass::Optional,
        cross_impl: CrossImpl::MayDiffer,
        expected: ExpectedOutcome::PassOrSkip,
        rationale: "HTTP Range on access_url is not required by DRS GetObject",
    },
    CatalogEntry {
        id: "drs.object.not_found",
        standard: "drs",
        contract: ContractClass::Standard,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "GetObject unknown id → 404",
    },
    CatalogEntry {
        id: "wes.service_info.reachable",
        standard: "wes",
        contract: ContractClass::Standard,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "GET service-info 200 when WES is present",
    },
    CatalogEntry {
        id: "wes.service_info.schema",
        standard: "wes",
        contract: ContractClass::RunnerExtra,
        cross_impl: CrossImpl::MayDiffer,
        expected: ExpectedOutcome::Pass,
        rationale: "ServiceInfo shape is standard; HelixTest extra equality on supported_wes_versions 1.0|1.1 is not",
    },
    CatalogEntry {
        id: "wes.run.lifecycle_success",
        standard: "wes",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MayDiffer,
        expected: ExpectedOutcome::Pass,
        rationale: "Echo TRS URL is a Helix fixture. Requiring a pre-terminal state before COMPLETE is runner_extra",
    },
    CatalogEntry {
        id: "wes.run.failure_state",
        standard: "wes",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "Documented fail workflow must end EXECUTOR_ERROR or SYSTEM_ERROR when mounted",
    },
    CatalogEntry {
        id: "wes.run.missing_inputs",
        standard: "wes",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "Documented missing-input fixture",
    },
    CatalogEntry {
        id: "wes.run.incompatible_type",
        standard: "wes",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "Documented WDL-vs-CWL fixture",
    },
    CatalogEntry {
        id: "wes.run.invalid_workflow",
        standard: "wes",
        contract: ContractClass::Fixture,
        cross_impl: CrossImpl::MustAgree,
        expected: ExpectedOutcome::Pass,
        rationale: "Documented invalid workflow URL fixture",
    },
    CatalogEntry {
        id: "wes.run.scatter_gather",
        standard: "wes",
        contract: ContractClass::Optional,
        cross_impl: CrossImpl::MayDiffer,
        expected: ExpectedOutcome::PassOrSkip,
        rationale: "Scatter/gather is not a WES-required workflow. Profile generic skips it",
    },
];

pub fn catalog_entry(id: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|c| c.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationSlot {
    pub id: String,
    pub kind: ImplementationKind,
    pub status: SlotStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotStatus {
    Recorded,
    Pending,
}

impl SlotStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recorded => "recorded",
            Self::Pending => "pending",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixRow {
    pub standard: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub check: String,
    pub implementation: String,
    pub expected: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
    pub result: MatrixResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discrepancy {
    pub check: String,
    pub standard: String,
    pub cross_impl: CrossImpl,
    pub classification: DiscrepancyClass,
    pub observed_by_implementation: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hypotheses: Vec<Hypothesis>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteropMatrix {
    pub schema_version: String,
    pub helix_version: String,
    /// Always false unless an independent_implementation run and another non-fixture run are both recorded.
    pub independent_evidence: bool,
    pub external_validation: ExternalValidation,
    pub note: String,
    pub implementations: Vec<ImplementationSlot>,
    pub rows: Vec<MatrixRow>,
    pub discrepancies: Vec<Discrepancy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalValidation {
    Pending,
    RecordedWithoutIndependent,
    RecordedWithIndependent,
}

impl ExternalValidation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::RecordedWithoutIndependent => "recorded_without_independent",
            Self::RecordedWithIndependent => "recorded_with_independent",
        }
    }
}

impl InteropMatrix {
    pub fn unresolved_count(&self) -> usize {
        self.discrepancies
            .iter()
            .filter(|d| d.classification == DiscrepancyClass::UnresolvedDiscrepancy)
            .count()
    }

    /// 1 when must-agree executed outcomes disagree. Pending-only is 0.
    pub fn process_exit_code(&self) -> i32 {
        if self.unresolved_count() > 0 {
            1
        } else {
            0
        }
    }
}

pub const PENDING_FERRUM: &str = "ferrum";
pub const PENDING_INDEPENDENT: &str = "independent";

#[derive(Debug, Clone)]
pub struct LabeledRun {
    pub implementation: String,
    pub kind: ImplementationKind,
    pub run: VerificationRun,
}

pub fn parse_id_path(spec: &str) -> Result<(String, String)> {
    let (id, path) = spec.split_once('=').context(
        "--run and --kind need ID=VALUE (example: --run ferrum=./ferrum.json --kind ferrum=reference_target)",
    )?;
    if id.is_empty() || path.is_empty() {
        bail!("empty id or value in `{spec}`");
    }
    Ok((id.to_string(), path.to_string()))
}

pub fn load_labeled_runs(
    runs: &[(String, String)],
    kinds: &BTreeMap<String, ImplementationKind>,
) -> Result<Vec<LabeledRun>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (id, path) in runs {
        if !seen.insert(id.clone()) {
            bail!("duplicate --run id `{id}`");
        }
        let run = load_verification_run(Path::new(path))?;
        let kind = kinds
            .get(id)
            .copied()
            .unwrap_or(ImplementationKind::Unspecified);
        out.push(LabeledRun {
            implementation: id.clone(),
            kind,
            run,
        });
    }
    Ok(out)
}

/// Build the matrix. Empty `runs` → pending Ferrum + independent slots, no observations.
pub fn build_matrix(runs: &[LabeledRun]) -> InteropMatrix {
    let mut slots: Vec<ImplementationSlot> = runs
        .iter()
        .map(|r| ImplementationSlot {
            id: r.implementation.clone(),
            kind: r.kind,
            status: SlotStatus::Recorded,
        })
        .collect();
    let recorded_ids: BTreeSet<String> = slots.iter().map(|s| s.id.clone()).collect();
    for (id, kind) in [
        (PENDING_FERRUM, ImplementationKind::ReferenceTarget),
        (
            PENDING_INDEPENDENT,
            ImplementationKind::IndependentImplementation,
        ),
    ] {
        if !recorded_ids.contains(id) {
            slots.push(ImplementationSlot {
                id: id.to_string(),
                kind,
                status: SlotStatus::Pending,
            });
        }
    }

    let independent_evidence = has_independent_evidence(runs);
    let external_validation = if independent_evidence {
        ExternalValidation::RecordedWithIndependent
    } else if runs.iter().any(|r| !r.kind.is_fixture()) {
        ExternalValidation::RecordedWithoutIndependent
    } else {
        ExternalValidation::Pending
    };

    let mut rows = Vec::new();
    for slot in &slots {
        let run = runs.iter().find(|r| r.implementation == slot.id);
        for cat in CATALOG {
            rows.push(row_for(cat, slot, run.map(|r| &r.run)));
        }
    }

    let discrepancies = compare_recorded(runs);
    InteropMatrix {
        schema_version: MATRIX_SCHEMA_VERSION.to_string(),
        helix_version: crate::model::helix_version().to_string(),
        independent_evidence,
        external_validation,
        note: note_for(external_validation),
        implementations: slots,
        rows,
        discrepancies,
    }
}

fn has_independent_evidence(runs: &[LabeledRun]) -> bool {
    let real: Vec<&LabeledRun> = runs
        .iter()
        .filter(|r| {
            matches!(
                r.kind,
                ImplementationKind::ReferenceTarget | ImplementationKind::IndependentImplementation
            )
        })
        .collect();
    real.len() >= 2 && real.iter().any(|r| r.kind.counts_as_independent_evidence())
}

fn note_for(v: ExternalValidation) -> String {
    match v {
        ExternalValidation::Pending => "External multi-implementation validation is pending. \
             In-process fixtures are not independent evidence. \
             Not GA4GH certification."
            .into(),
        ExternalValidation::RecordedWithoutIndependent => {
            "Recorded run(s) are not independent evidence \
             (need --kind independent_implementation plus another non-fixture run). \
             Not GA4GH certification."
                .into()
        }
        ExternalValidation::RecordedWithIndependent => {
            "Operator supplied an independent_implementation run and another non-fixture run. \
             That is recorded comparison, not GA4GH certification, not a Helix product claim."
                .into()
        }
    }
}

fn row_for(
    cat: &CatalogEntry,
    slot: &ImplementationSlot,
    run: Option<&VerificationRun>,
) -> MatrixRow {
    let (observed_status, version) = match run {
        None => (None, None),
        Some(run) => (result_status(run, cat.id), version_of(run)),
    };
    let observed = observed_status.map(status_wire);
    let result = match observed_status {
        None => MatrixResult::Pending,
        Some(st) => classify_observed(cat, st),
    };
    MatrixRow {
        standard: cat.standard.to_string(),
        version,
        check: cat.id.to_string(),
        implementation: slot.id.clone(),
        expected: cat.expected.as_str().to_string(),
        observed: observed.map(str::to_string),
        result,
    }
}

fn version_of(run: &VerificationRun) -> Option<String> {
    let sel = run.standard_selection.as_ref()?;
    sel.verified_version
        .clone()
        .or_else(|| sel.selected_version.clone())
        .filter(|s| !s.is_empty())
}

fn result_status(run: &VerificationRun, id: &str) -> Option<VerificationStatus> {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id)
        .map(|r| r.status)
}

fn classify_observed(cat: &CatalogEntry, st: VerificationStatus) -> MatrixResult {
    match st {
        VerificationStatus::Skip => MatrixResult::NotExecuted,
        VerificationStatus::Pass => MatrixResult::MeetsContract,
        VerificationStatus::Fail | VerificationStatus::Error => match cat.contract {
            ContractClass::Standard if cat.cross_impl == CrossImpl::MustAgree => {
                MatrixResult::ContractViolation
            }
            ContractClass::Optional => MatrixResult::MeetsContract,
            ContractClass::RunnerExtra => MatrixResult::RunnerStricter,
            ContractClass::Fixture => MatrixResult::FixtureUnsatisfied,
            ContractClass::Standard => MatrixResult::MeetsContract,
        },
    }
}

fn status_wire(st: VerificationStatus) -> &'static str {
    match st {
        VerificationStatus::Pass => "pass",
        VerificationStatus::Fail => "fail",
        VerificationStatus::Skip => "skip",
        VerificationStatus::Error => "error",
    }
}

fn executed_status(st: VerificationStatus) -> bool {
    matches!(
        st,
        VerificationStatus::Pass | VerificationStatus::Fail | VerificationStatus::Error
    )
}

fn compare_recorded(runs: &[LabeledRun]) -> Vec<Discrepancy> {
    if runs.len() < 2 {
        return CATALOG
            .iter()
            .map(|cat| Discrepancy {
                check: cat.id.to_string(),
                standard: cat.standard.to_string(),
                cross_impl: cat.cross_impl,
                classification: DiscrepancyClass::Pending,
                observed_by_implementation: BTreeMap::new(),
                hypotheses: Vec::new(),
                note: "Fewer than two recorded implementations.".into(),
            })
            .collect();
    }
    CATALOG
        .iter()
        .map(|cat| discrepancy_for(cat, runs))
        .collect()
}

fn discrepancy_for(cat: &CatalogEntry, runs: &[LabeledRun]) -> Discrepancy {
    let mut observed = BTreeMap::new();
    for r in runs {
        if let Some(st) = result_status(&r.run, cat.id) {
            observed.insert(r.implementation.clone(), status_wire(st).to_string());
        }
    }
    let executed: Vec<(&str, VerificationStatus)> = runs
        .iter()
        .filter_map(|r| {
            result_status(&r.run, cat.id)
                .filter(|st| executed_status(*st))
                .map(|st| (r.implementation.as_str(), st))
        })
        .collect();

    if executed.len() < 2 {
        return Discrepancy {
            check: cat.id.to_string(),
            standard: cat.standard.to_string(),
            cross_impl: cat.cross_impl,
            classification: if observed.len() < 2 {
                DiscrepancyClass::Pending
            } else {
                DiscrepancyClass::NotComparable
            },
            observed_by_implementation: observed,
            hypotheses: Vec::new(),
            note:
                "Need two executed (pass/fail/error) outcomes to compare. Skip is not comparable."
                    .into(),
        };
    }

    let first = executed[0].1;
    let all_equal = executed.iter().all(|(_, st)| *st == first);
    if all_equal {
        return Discrepancy {
            check: cat.id.to_string(),
            standard: cat.standard.to_string(),
            cross_impl: cat.cross_impl,
            classification: DiscrepancyClass::Agree,
            observed_by_implementation: observed,
            hypotheses: Vec::new(),
            note: cat.rationale.to_string(),
        };
    }

    match cat.cross_impl {
        CrossImpl::MayDiffer => Discrepancy {
            check: cat.id.to_string(),
            standard: cat.standard.to_string(),
            cross_impl: cat.cross_impl,
            classification: DiscrepancyClass::ImplementationSpecific,
            observed_by_implementation: observed,
            hypotheses: Vec::new(),
            note: format!(
                "Difference is permitted. {}. Not a spec failure.",
                cat.rationale
            ),
        },
        CrossImpl::MustAgree => Discrepancy {
            check: cat.id.to_string(),
            standard: cat.standard.to_string(),
            cross_impl: cat.cross_impl,
            classification: DiscrepancyClass::UnresolvedDiscrepancy,
            observed_by_implementation: observed,
            hypotheses: vec![
                Hypothesis::HelixBug,
                Hypothesis::ImplementationBug,
                Hypothesis::AmbiguousSpec,
            ],
            note: format!(
                "Executed outcomes disagree on a must_agree check. {}. \
                 Helix does not auto-assign helix_bug vs implementation_bug.",
                cat.rationale
            ),
        },
    }
}

pub fn matrix_json(matrix: &InteropMatrix) -> Result<String> {
    Ok(crate::redact::redact_text(&serde_json::to_string_pretty(
        matrix,
    )?))
}

pub fn format_matrix_text(matrix: &InteropMatrix) -> String {
    let mut out = String::new();
    out.push_str("HELIX INTEROP MATRIX\n");
    out.push('\n');
    out.push_str(&format!(
        "External multi-implementation validation: {}\n",
        matrix.external_validation.as_str().to_uppercase()
    ));
    out.push_str(&format!(
        "Independent evidence: {}\n",
        if matrix.independent_evidence {
            "yes (operator-labeled; not certification)"
        } else {
            "no"
        }
    ));
    out.push_str(&sanitize_untrusted(&matrix.note));
    out.push('\n');
    out.push('\n');
    out.push_str("Implementations:\n");
    for s in &matrix.implementations {
        out.push_str(&format!(
            "  {}  {}  {}\n",
            s.id,
            s.kind.as_str(),
            s.status.as_str()
        ));
    }
    out.push('\n');
    out.push_str("Matrix:\n");
    out.push_str("  standard  version  check  implementation  expected  observed  result\n");
    for r in &matrix.rows {
        out.push_str(&format!(
            "  {}  {}  {}  {}  {}  {}  {}\n",
            r.standard,
            r.version.as_deref().unwrap_or("unversioned"),
            r.check,
            r.implementation,
            r.expected,
            r.observed.as_deref().unwrap_or("—"),
            r.result.as_str()
        ));
    }
    out.push('\n');
    out.push_str("Comparisons:\n");
    for d in &matrix.discrepancies {
        out.push_str(&format!(
            "  {}  {}  {}\n",
            d.check,
            d.cross_impl.as_str(),
            d.classification.as_str()
        ));
        if !d.hypotheses.is_empty() {
            let h: Vec<&str> = d.hypotheses.iter().map(|x| x.as_str()).collect();
            out.push_str(&format!("    hypotheses: {}\n", h.join(", ")));
        }
        out.push_str(&format!("    {}\n", d.note));
    }
    out.push('\n');
    out.push_str("It is not GA4GH certification.\n");
    out.push_str("In-process fixtures are not independent evidence.\n");
    out
}

/// Catalog ids must stay assigned in identity.rs.
pub fn catalog_ids_are_assigned() -> Result<()> {
    for c in CATALOG {
        if spec_by_id(c.id).is_none() {
            bail!("interop catalog id `{}` is not in identity::SPECS", c.id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::spec;
    use crate::model::{Target, VerificationCheck, VerificationResult};

    fn run_with(id: &str, st: VerificationStatus) -> VerificationRun {
        let mut run = VerificationRun::new(Target::new("http://127.0.0.1:9"));
        run.timestamp = "2026-09-05T00:00:00Z".into();
        let c = VerificationCheck::from_spec(spec(id));
        match st {
            VerificationStatus::Pass => run.push_executed(VerificationResult::pass(c)),
            VerificationStatus::Fail => {
                run.push_executed(VerificationResult::fail(c, "fixture"));
            }
            VerificationStatus::Skip => run.push_skipped(VerificationResult::skip(c, "skip")),
            VerificationStatus::Error => {
                run.push_executed(VerificationResult::from_check(c, VerificationStatus::Error));
            }
        }
        run
    }

    fn labeled(name: &str, kind: ImplementationKind, run: VerificationRun) -> LabeledRun {
        LabeledRun {
            implementation: name.into(),
            kind,
            run,
        }
    }

    #[test]
    fn pending_matrix_has_no_independent_evidence() {
        let m = build_matrix(&[]);
        assert!(!m.independent_evidence);
        assert_eq!(m.external_validation, ExternalValidation::Pending);
        assert!(m.implementations.iter().any(|s| s.id == PENDING_FERRUM
            && s.status == SlotStatus::Pending
            && s.kind == ImplementationKind::ReferenceTarget));
        assert!(m.implementations.iter().any(|s| {
            s.id == PENDING_INDEPENDENT
                && s.status == SlotStatus::Pending
                && s.kind == ImplementationKind::IndependentImplementation
        }));
        assert!(m.rows.iter().all(|r| r.result == MatrixResult::Pending));
        assert!(m
            .discrepancies
            .iter()
            .all(|d| d.classification == DiscrepancyClass::Pending));
        assert_eq!(m.process_exit_code(), 0);
        assert!(!m.note.to_lowercase().contains("validated against multiple"));
    }

    #[test]
    fn two_fixture_runs_are_not_independent_evidence() {
        let a = labeled(
            "mock_a",
            ImplementationKind::HelixFixture,
            run_with("drs.object.not_found", VerificationStatus::Pass),
        );
        let b = labeled(
            "mock_b",
            ImplementationKind::HelixFixture,
            run_with("drs.object.not_found", VerificationStatus::Pass),
        );
        let m = build_matrix(&[a, b]);
        assert!(!m.independent_evidence);
        assert_eq!(m.external_validation, ExternalValidation::Pending);
    }

    #[test]
    fn range_disagreement_is_implementation_specific() {
        let a = labeled(
            "a",
            ImplementationKind::HelixFixture,
            run_with("drs.object.range", VerificationStatus::Pass),
        );
        let b = labeled(
            "b",
            ImplementationKind::HelixFixture,
            run_with("drs.object.range", VerificationStatus::Fail),
        );
        let m = build_matrix(&[a, b]);
        let d = m
            .discrepancies
            .iter()
            .find(|d| d.check == "drs.object.range")
            .unwrap();
        assert_eq!(d.cross_impl, CrossImpl::MayDiffer);
        assert_eq!(d.classification, DiscrepancyClass::ImplementationSpecific);
        assert!(d.hypotheses.is_empty());
        let fail_row = m
            .rows
            .iter()
            .find(|r| r.check == "drs.object.range" && r.implementation == "b")
            .unwrap();
        assert_eq!(fail_row.result, MatrixResult::MeetsContract);
        assert_eq!(m.process_exit_code(), 0);
    }

    #[test]
    fn not_found_disagreement_is_unresolved_with_hypotheses() {
        let a = labeled(
            "a",
            ImplementationKind::ReferenceTarget,
            run_with("drs.object.not_found", VerificationStatus::Pass),
        );
        let b = labeled(
            "b",
            ImplementationKind::IndependentImplementation,
            run_with("drs.object.not_found", VerificationStatus::Fail),
        );
        let m = build_matrix(&[a, b]);
        assert!(m.independent_evidence);
        let d = m
            .discrepancies
            .iter()
            .find(|d| d.check == "drs.object.not_found")
            .unwrap();
        assert_eq!(d.classification, DiscrepancyClass::UnresolvedDiscrepancy);
        assert!(d.hypotheses.contains(&Hypothesis::HelixBug));
        assert!(d.hypotheses.contains(&Hypothesis::ImplementationBug));
        assert!(d.hypotheses.contains(&Hypothesis::AmbiguousSpec));
        assert_eq!(m.process_exit_code(), 1);
        let fail = m
            .rows
            .iter()
            .find(|r| r.check == "drs.object.not_found" && r.implementation == "b")
            .unwrap();
        assert_eq!(fail.result, MatrixResult::ContractViolation);
    }

    #[test]
    fn scatter_skip_vs_pass_is_not_comparable_or_specific_not_a_must_fail() {
        let a = labeled(
            "generic",
            ImplementationKind::HelixFixture,
            run_with("wes.run.scatter_gather", VerificationStatus::Skip),
        );
        let b = labeled(
            "scatter_on",
            ImplementationKind::HelixFixture,
            run_with("wes.run.scatter_gather", VerificationStatus::Pass),
        );
        let m = build_matrix(&[a, b]);
        let d = m
            .discrepancies
            .iter()
            .find(|d| d.check == "wes.run.scatter_gather")
            .unwrap();
        assert_eq!(d.cross_impl, CrossImpl::MayDiffer);
        assert_ne!(d.classification, DiscrepancyClass::UnresolvedDiscrepancy);
        assert_eq!(m.process_exit_code(), 0);
    }

    #[test]
    fn catalog_matches_identity() {
        catalog_ids_are_assigned().unwrap();
    }

    #[test]
    fn verify_source_has_no_implementation_name_branches() {
        let verify = include_str!("verify.rs");
        assert!(!verify.contains("Ferrum Gateway"));
        assert!(!verify.to_lowercase().contains("elixir"));
        assert!(!verify.to_lowercase().contains("cromwell"));
        let adapter = include_str!("adapter/mod.rs");
        assert!(!adapter.contains("if name"));
        assert!(!adapter.contains("Ferrum Gateway"));
        assert!(!adapter.contains("ferrum::"));
        let target = include_str!("target.rs");
        assert!(!target.contains("use ferrum"));
        assert!(!target.contains("ferrum::"));
    }

    #[test]
    fn schema_check_fail_is_runner_stricter_not_contract_violation() {
        let r = classify_observed(
            catalog_entry("drs.object.schema").unwrap(),
            VerificationStatus::Fail,
        );
        assert_eq!(r, MatrixResult::RunnerStricter);
    }

    #[test]
    fn text_pending_does_not_claim_validation() {
        let text = format_matrix_text(&build_matrix(&[]));
        assert!(text.contains("PENDING"));
        assert!(text.contains("not independent evidence"));
        assert!(
            text.contains("standard  version  check  implementation  expected  observed  result")
        );
        assert!(!text
            .to_lowercase()
            .contains("multi-implementation validation complete"));
    }
}
