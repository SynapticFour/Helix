// SPDX-License-Identifier: Apache-2.0
//! Lightweight identity of a verification (or bench) run so two results can be compared.
//!
//! Not a signed audit trail. Not scientific reproducibility. Not HELIOS
//! (no signature, RO-Crate, PDF, evidence pack). See docs/RUN_IDENTITY.md.

use serde::{Deserialize, Serialize};

use crate::bench::BenchOutcome;
use crate::model::{VerificationRun, FIXTURE_VERSION, SCHEMA_VERSION};

/// Fields Helix records so `helix compare` can say whether two JSON files
/// are the same kind of measurement. Timestamp is recorded, not a signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunIdentity {
    pub helix_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helixtest_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helixtest_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Stable Helix check `id`s from this run (executed + skipped), sorted.
    pub check_ids: Vec<String>,
    pub target: String,
    pub fixture_version: String,
    pub schema_version: String,
    pub timestamp: String,
    /// Bench only (`http.drs.smoke.v1`). Omitted on `helix verify`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMismatch {
    pub field: String,
    pub previous: String,
    pub current: String,
}

impl RunIdentity {
    pub fn from_verify(run: &VerificationRun) -> Self {
        let mut check_ids: Vec<String> = run
            .executed
            .iter()
            .chain(run.skipped.iter())
            .map(|r| r.id.clone())
            .collect();
        check_ids.sort();
        check_ids.dedup();
        Self {
            helix_version: run.helix_version.clone(),
            helixtest_version: run.helixtest_version.clone(),
            helixtest_sha: run.helixtest_sha.clone(),
            profile: run.profile.clone(),
            check_ids,
            target: run.target.url.clone(),
            fixture_version: if run.fixture_version.is_empty() {
                FIXTURE_VERSION.to_string()
            } else {
                run.fixture_version.clone()
            },
            schema_version: if run.schema_version.is_empty() {
                SCHEMA_VERSION.to_string()
            } else {
                run.schema_version.clone()
            },
            timestamp: run.timestamp.clone(),
            workload_id: None,
            workload_version: None,
        }
    }

    pub fn from_bench(outcome: &BenchOutcome) -> Self {
        Self {
            helix_version: outcome.baseline.metadata.helix_version.clone(),
            helixtest_version: None,
            helixtest_sha: None,
            profile: None,
            check_ids: Vec::new(),
            target: format!(
                "{} vs {}",
                outcome.baseline.endpoint, outcome.candidate.endpoint
            ),
            fixture_version: FIXTURE_VERSION.to_string(),
            schema_version: SCHEMA_VERSION.to_string(),
            timestamp: String::new(),
            workload_id: Some(outcome.workload_id.clone()),
            workload_version: Some(outcome.workload_version.clone()),
        }
    }

    /// Same measurement class: schema, profile, fixtures, target.
    /// Helix/HelixTest version and timestamp may differ (suite upgrade / wall clock).
    /// Check-id set differences are catalog change, not this flag.
    pub fn same_measurement(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.profile == other.profile
            && self.fixture_version == other.fixture_version
            && self.target == other.target
            && self.workload_id == other.workload_id
            && self.workload_version == other.workload_version
    }

    pub fn suite_changed(&self, other: &Self) -> bool {
        self.helix_version != other.helix_version
            || self.helixtest_version != other.helixtest_version
            || self.helixtest_sha != other.helixtest_sha
    }

    pub fn catalog_changed(&self, other: &Self) -> bool {
        self.check_ids != other.check_ids
    }

    pub fn mismatches(&self, other: &Self) -> Vec<IdentityMismatch> {
        let mut out = Vec::new();
        push_mis(
            &mut out,
            "schema_version",
            &self.schema_version,
            &other.schema_version,
        );
        push_mis(
            &mut out,
            "helix_version",
            &self.helix_version,
            &other.helix_version,
        );
        push_opt(
            &mut out,
            "helixtest_version",
            &self.helixtest_version,
            &other.helixtest_version,
        );
        push_opt(
            &mut out,
            "helixtest_sha",
            &self.helixtest_sha,
            &other.helixtest_sha,
        );
        push_opt(&mut out, "profile", &self.profile, &other.profile);
        push_mis(&mut out, "target", &self.target, &other.target);
        push_mis(
            &mut out,
            "fixture_version",
            &self.fixture_version,
            &other.fixture_version,
        );
        if self.check_ids != other.check_ids {
            out.push(IdentityMismatch {
                field: "check_ids".into(),
                previous: self.check_ids.join(","),
                current: other.check_ids.join(","),
            });
        }
        push_opt(
            &mut out,
            "workload_id",
            &self.workload_id,
            &other.workload_id,
        );
        push_opt(
            &mut out,
            "workload_version",
            &self.workload_version,
            &other.workload_version,
        );
        out
    }
}

fn push_mis(out: &mut Vec<IdentityMismatch>, field: &str, a: &str, b: &str) {
    if a != b {
        out.push(IdentityMismatch {
            field: field.into(),
            previous: a.into(),
            current: b.into(),
        });
    }
}

fn push_opt(out: &mut Vec<IdentityMismatch>, field: &str, a: &Option<String>, b: &Option<String>) {
    if a != b {
        out.push(IdentityMismatch {
            field: field.into(),
            previous: a.clone().unwrap_or_default(),
            current: b.clone().unwrap_or_default(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity;
    use crate::model::{
        Target, VerificationCheck, VerificationResult, VerificationRun, HELIXTEST_PIN,
    };

    fn run_at(url: &str) -> VerificationRun {
        let mut run = VerificationRun::new(Target::new(url));
        run.profile = Some("generic".into());
        run.timestamp = "2026-09-04T12:00:00Z".into();
        run.push_executed(VerificationResult::pass(VerificationCheck::from_spec(
            identity::spec("drs.object.reachable"),
        )));
        run
    }

    #[test]
    fn from_verify_records_compare_fields_without_helios_keys() {
        let id = RunIdentity::from_verify(&run_at("http://127.0.0.1:9"));
        assert_eq!(id.helix_version, crate::model::helix_version());
        assert_eq!(id.helixtest_version.as_deref(), Some(HELIXTEST_PIN));
        assert_eq!(id.profile.as_deref(), Some("generic"));
        assert_eq!(id.check_ids, vec!["drs.object.reachable"]);
        assert_eq!(id.target, "http://127.0.0.1:9");
        assert_eq!(id.fixture_version, FIXTURE_VERSION);
        assert_eq!(id.schema_version, SCHEMA_VERSION);
        assert_eq!(id.timestamp, "2026-09-04T12:00:00Z");
        assert!(id.workload_id.is_none());
        assert!(id.workload_version.is_none());
        let v = serde_json::to_value(&id).unwrap();
        assert!(v.get("signature").is_none());
        assert!(v.get("ro_crate").is_none());
        assert!(v.get("pdf").is_none());
        assert!(v.get("audit_trail").is_none());
    }

    #[test]
    fn same_target_profile_fixtures_are_same_measurement_if_timestamp_differs() {
        let mut a = run_at("http://127.0.0.1:9");
        let mut b = a.clone();
        b.timestamp = "2026-09-04T13:00:00Z".into();
        let ia = RunIdentity::from_verify(&a);
        let ib = RunIdentity::from_verify(&b);
        assert!(ia.same_measurement(&ib));
        assert!(!ia.suite_changed(&ib));
        assert!(!ia.mismatches(&ib).iter().any(|m| m.field == "timestamp"));
        a.helix_version = "0.0.1".into();
        let ia = RunIdentity::from_verify(&a);
        assert!(ia.suite_changed(&ib));
        assert!(ia.same_measurement(&ib));
    }

    #[test]
    fn different_target_is_not_same_measurement() {
        let a = RunIdentity::from_verify(&run_at("http://127.0.0.1:8"));
        let b = RunIdentity::from_verify(&run_at("http://127.0.0.1:9"));
        assert!(!a.same_measurement(&b));
        assert_eq!(a.mismatches(&b)[0].field, "target");
    }

    #[test]
    fn catalog_change_is_recorded_and_is_not_a_signature() {
        let mut a = run_at("http://127.0.0.1:9");
        let mut b = a.clone();
        b.push_skipped(VerificationResult::skip(
            VerificationCheck::from_spec(identity::spec("wes.service_info.reachable")),
            "not detected",
        ));
        let ia = RunIdentity::from_verify(&a);
        let ib = RunIdentity::from_verify(&b);
        assert!(ia.same_measurement(&ib));
        assert!(ia.catalog_changed(&ib));
        a.profile = Some("ferrum".into());
        assert!(!RunIdentity::from_verify(&a).same_measurement(&ib));
    }
}
