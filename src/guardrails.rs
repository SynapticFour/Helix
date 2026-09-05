// SPDX-License-Identifier: Apache-2.0
//! Trust-model guardrails. Prefer these over contributor instructions.
//!
//! Helix-produced `VerificationRun` values must pass [`check_run`] before a
//! report is printed. Registry rows must pass [`crate::standards::validate_yaml`].
//! Source scans live in `tests/guardrails.rs`.
//!
//! Not HELIOS. Not GA4GH certification.

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::claims::{check_set, evaluate, ClaimStatus};
use crate::model::{StandardSelection, VerificationRun};
use crate::traceability::validate_result;

const FORBIDDEN_HELIOS_KEYS: &[&str] = &[
    "signature",
    "ro_crate",
    "pdf",
    "evidence_pack",
    "audit_trail",
];

/// How strictly to apply run-level rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckMode {
    /// Helix is about to emit this run (`helix verify` JSON/text).
    Emit,
    /// Operator-supplied JSON (`helix compare` / `helix matrix`).
    /// Missing traceability on old files is allowed; mislabeled taxonomy is not.
    Load,
}

/// Fail closed if this run would let a reviewer infer substitution, a MUST, or HELIOS.
pub fn check_run(run: &VerificationRun) -> Result<()> {
    check_run_with(run, CheckMode::Emit)
}

pub fn check_run_with(run: &VerificationRun, mode: CheckMode) -> Result<()> {
    let value = serde_json::to_value(run).context("serialize run for integrity check")?;
    forbid_helios_keys(&value)?;
    check_selection(run.standard_selection.as_ref())?;
    for r in run.executed.iter().chain(run.skipped.iter()) {
        match mode {
            CheckMode::Emit => {
                if let Err(e) = validate_result(r) {
                    bail!("check {} failed emit guardrail: {e}", r.id);
                }
            }
            CheckMode::Load => {
                if r.traceability.is_some() {
                    if let Err(e) = validate_result(r) {
                        bail!("check {} failed load guardrail: {e}", r.id);
                    }
                }
            }
        }
        if let (Some(sel), Some(ver)) = (&r.selected_version, &r.verified_version) {
            if sel != ver {
                bail!(
                    "check {}: selected_version ({sel}) != verified_version ({ver})",
                    r.id
                );
            }
        }
        if r.verified_version.is_some() && r.selected_version.is_none() {
            bail!("check {}: verified_version requires selected_version", r.id);
        }
    }
    if mode == CheckMode::Emit {
        check_set(&evaluate(run)).context("VERIFIED claim is not justified by predicates")?;
    }
    Ok(())
}

fn check_selection(sel: Option<&StandardSelection>) -> Result<()> {
    let Some(sel) = sel else {
        return Ok(());
    };
    if sel.substituted {
        bail!("standard_selection.substituted must be false (silent substitution is forbidden)");
    }
    match (
        nonempty(sel.selected_version.as_deref()),
        nonempty(sel.verified_version.as_deref()),
    ) {
        (Some(a), Some(b)) if a != b => {
            bail!("selected_version ({a}) != verified_version ({b})");
        }
        (None, Some(v)) => {
            bail!("verified_version ({v}) requires selected_version");
        }
        (Some(_), Some(_)) => {
            require_join_hashes_for_verified(sel)?;
        }
        (Some(_), None) | (None, None) => {}
    }
    Ok(())
}

fn require_join_hashes_for_verified(sel: &StandardSelection) -> Result<()> {
    if sel
        .pack_integrity_sha256
        .as_deref()
        .unwrap_or("")
        .is_empty()
        || sel
            .schema_document_sha256
            .as_deref()
            .unwrap_or("")
            .is_empty()
        || sel
            .schema_component_sha256
            .as_deref()
            .unwrap_or("")
            .is_empty()
        || sel.integrity_ok != Some(true)
    {
        bail!(
            "verified_version requires pack_integrity_sha256, schema_document_sha256, \
             schema_component_sha256, and integrity_ok=true"
        );
    }
    Ok(())
}

fn nonempty(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

pub fn forbid_helios_keys(v: &Value) -> Result<()> {
    forbid_helios_keys_rec(v, "$")
}

fn forbid_helios_keys_rec(v: &Value, path: &str) -> Result<()> {
    match v {
        Value::Object(map) => {
            for key in FORBIDDEN_HELIOS_KEYS {
                if map.contains_key(*key) {
                    bail!("{path}.{key} is a HELIOS field and must not appear on a Helix run");
                }
            }
            for (k, child) in map {
                forbid_helios_keys_rec(child, &format!("{path}.{k}"))?;
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                forbid_helios_keys_rec(child, &format!("{path}[{i}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Claims already on a JSON object (if any) must not be VERIFIED unless evaluate agrees.
pub fn check_serialized_claims(run: &VerificationRun, json: &Value) -> Result<()> {
    let expected = evaluate(run);
    check_set(&expected)?;
    if let Some(arr) = json.get("claims").and_then(|c| c.as_array()) {
        for item in arr {
            let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            let status = item.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status == ClaimStatus::Verified.as_str() {
                let got = expected
                    .items
                    .iter()
                    .find(|c| c.kind.as_str() == kind)
                    .ok_or_else(|| anyhow::anyhow!("unknown claim kind {kind}"))?;
                if got.status != ClaimStatus::Verified {
                    bail!(
                        "JSON claims {kind} as VERIFIED but predicates do not hold ({})",
                        got.block_codes()
                            .iter()
                            .map(|c| c.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::spec;
    use crate::model::{
        VerificationCheck, VerificationResult, VerificationRun, VerificationStatus,
    };
    use crate::standards::{BindingKind, ClaimScope};
    use crate::traceability::Authority;

    fn honest_run() -> VerificationRun {
        let mut run = VerificationRun::new(crate::model::Target::new("http://127.0.0.1:9"));
        run.push_executed(VerificationResult::from_check(
            VerificationCheck::from_spec(spec("drs.object.reachable")),
            VerificationStatus::Pass,
        ));
        crate::traceability::bind_run(&mut run).expect("bind");
        run
    }

    #[test]
    fn honest_unversioned_run_passes_emit() {
        check_run(&honest_run()).expect("honest fixture run");
    }

    #[test]
    fn substituted_true_is_rejected() {
        let mut run = honest_run();
        run.standard_selection = Some(StandardSelection {
            substituted: true,
            ..StandardSelection::unversioned()
        });
        let err = check_run(&run).unwrap_err().to_string();
        assert!(err.contains("substituted"), "{err}");
    }

    #[test]
    fn selected_ne_verified_is_rejected() {
        let mut run = honest_run();
        let mut sel = StandardSelection::unversioned();
        sel.selected_version = Some("1.4.0".into());
        sel.verified_version = Some("1.5.0".into());
        run.standard_selection = Some(sel);
        let err = check_run(&run).unwrap_err().to_string();
        assert!(err.contains("1.4.0"), "{err}");
        assert!(err.contains("1.5.0"), "{err}");
    }

    #[test]
    fn fixture_serialized_as_normative_is_rejected() {
        let mut run = honest_run();
        let t = run.executed[0].traceability.as_mut().unwrap();
        t.category = BindingKind::Fixture;
        t.check_kind = BindingKind::Normative;
        t.claim_scope = ClaimScope::Ga4ghRequirement;
        t.authority = Authority::Ga4gh;
        t.untraceable_reason = None;
        let err = check_run(&run).unwrap_err().to_string();
        assert!(
            err.contains("cannot be serialized as a normative requirement"),
            "{err}"
        );
    }

    #[test]
    fn helios_key_on_run_json_is_rejected() {
        let run = honest_run();
        let mut v = serde_json::to_value(&run).unwrap();
        v["ro_crate"] = serde_json::json!({"id": "no"});
        let err = forbid_helios_keys(&v).unwrap_err().to_string();
        assert!(err.contains("ro_crate"), "{err}");
    }

    #[test]
    fn normative_without_provenance_is_rejected() {
        let mut run = honest_run();
        let t = run.executed[0].traceability.as_mut().unwrap();
        t.category = BindingKind::Normative;
        t.check_kind = BindingKind::Normative;
        t.claim_scope = ClaimScope::Ga4ghRequirement;
        t.authority = Authority::Ga4gh;
        t.untraceable_reason = None;
        t.source_file = Some("openapi.yaml".into());
        t.version = Some("1.4.0".into());
        t.registry_entry = Some("ga4gh.drs.1.4.0".into());
        t.source_commit = None;
        let err = check_run(&run).unwrap_err().to_string();
        assert!(err.contains("no source commit"), "{err}");
    }

    #[test]
    fn result_verified_without_selected_is_rejected() {
        let mut run = honest_run();
        run.executed[0].selected_version = None;
        run.executed[0].verified_version = Some("1.4.0".into());
        let err = check_run(&run).unwrap_err().to_string();
        assert!(
            err.contains("verified_version requires selected_version"),
            "{err}"
        );
    }

    #[test]
    fn selected_without_verified_is_allowed() {
        let mut run = honest_run();
        let mut sel = StandardSelection::unversioned();
        sel.mode = "explicit".into();
        sel.selection_status = crate::standards::SELECTED.into();
        sel.standard = Some("drs".into());
        sel.selected_version = Some("1.4.0".into());
        sel.verified_version = None;
        run.standard_selection = Some(sel);
        run.executed[0].selected_version = Some("1.4.0".into());
        run.executed[0].verified_version = None;
        check_run(&run).expect("B2 selected_version without verified_version");
    }

    #[test]
    fn verified_without_join_hashes_is_rejected() {
        let mut run = honest_run();
        let mut sel = StandardSelection::unversioned();
        sel.mode = "explicit".into();
        sel.selection_status = crate::standards::SELECTED.into();
        sel.selected_version = Some("1.4.0".into());
        sel.verified_version = Some("1.4.0".into());
        run.standard_selection = Some(sel);
        let err = check_run(&run).unwrap_err().to_string();
        assert!(err.contains("verified_version requires"), "{err}");
    }

    #[test]
    fn load_mode_accepts_missing_traceability() {
        let mut run = VerificationRun::new(crate::model::Target::new("http://127.0.0.1:9"));
        run.push_executed(VerificationResult::from_check(
            VerificationCheck::from_spec(spec("drs.object.reachable")),
            VerificationStatus::Pass,
        ));
        run.executed[0].traceability = None;
        check_run_with(&run, CheckMode::Load).expect("old JSON without traceability");
        let err = check_run_with(&run, CheckMode::Emit)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no provenance"), "{err}");
    }
}
