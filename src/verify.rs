// SPDX-License-Identifier: Apache-2.0
//! DRS and WES verification: discover → testable? → HelixTest adapter → Helix results.
//!
//! Default `helix verify TARGET` is the unversioned HelixTest wrap (not a GA4GH
//! pack selection). `--standard` / `--version` / `--all-supported-versions` load
//! the standards registry and fail closed: never substitute another version,
//! never execute AVAILABLE-only rows. TES / TRS / htsget checks are not wired.
//! Not HELIOS.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Context, Result};

use crate::adapter::{ConformanceAdapter, HelixTestAdapter};
use crate::discover::{
    discover_for_drs_object, http_client, normalize_endpoint, Detection, Discovery, Ga4ghService,
    ServiceDiscovery, Testability, VERIFY_ORDER,
};
use crate::identity::{drs_verify_specs, wes_verify_specs, CheckSpec};
use crate::model::{
    DiscoveredService, StandardSelection, Target, VerificationCheck, VerificationResult,
    VerificationRun, HELIXTEST_PIN,
};
use crate::profile::{definition, Profile, ProfileId};
use crate::standards::{
    binding_id as compute_binding_id, catalog_id as compute_catalog_id, compare_spec_identity,
    contract_for, declared_checker_id, default_registry_path, execution_id, helix_repo_root,
    load_pack, load_path, select_all_official_supported, select_automatic, select_explicit,
    PackLoadError, PackRef, Registry, ReleaseClass, SelectionError, AVAILABLE_BUT_NOT_SUPPORTED,
    MULTIPLE_PACKS_NOT_EXECUTABLE, SELECTED,
};
use crate::target::{DeclaredTarget, TargetIdentity};

pub use crate::adapter::{DRS_CHECK_NAMES, WES_CHECK_NAMES};

const SKIP_STANDARD_NOT_SELECTED: &str = "standard not selected; checks not executed (not a pass)";

#[derive(Debug, Clone)]
pub enum VerifySelection {
    /// Default CLI: HelixTest wrap. Does not load the registry for selection.
    Unversioned,
    /// Mode 1: `--standard` + `--version`.
    Explicit {
        standard: String,
        version: String,
        release_class: Option<ReleaseClass>,
    },
    /// Mode 2: `--standard` without `--version`. Fail closed when OfficialSupported is empty.
    Automatic { standard: String },
    /// Mode 3: `--standard` + `--all-supported-versions`. OfficialSupported only.
    Compatibility { standard: String },
}

#[derive(Debug, Clone)]
pub struct VerifyOptions {
    pub profile: ProfileId,
    pub selection: VerifySelection,
    /// Override shipped `standards/registry.yaml` (tests). CLI uses the shipped file.
    pub registry: Option<Registry>,
    /// Tests: resolve `vendor_path` relative to this root. Default: crate directory.
    pub vendor_root: Option<std::path::PathBuf>,
    /// Operator-declared target labels. Untrusted. Never inferred from headers.
    pub declared_target: DeclaredTarget,
    /// Target-scoped DRS test object. Default catalog `test-object-1`.
    pub drs_fixture: crate::fixture::DrsVerifyFixture,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            profile: ProfileId::Generic,
            selection: VerifySelection::Unversioned,
            registry: None,
            vendor_root: None,
            declared_target: DeclaredTarget::default(),
            drs_fixture: crate::fixture::DrsVerifyFixture::default_catalog(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    pub discovery: Discovery,
    pub run: VerificationRun,
}

impl VerifyOutcome {
    pub fn has_failures(&self) -> bool {
        self.run.has_failures()
    }

    /// Success: at least one pass and no fail/error. Skip-only is not a pass.
    pub fn is_success(&self) -> bool {
        self.run.overall_status().is_pass()
    }
}

/// Discover APIs under `endpoint`, then run enabled HelixTest suites when TESTABLE.
/// Default profile is [`ProfileId::Generic`]. Unversioned: no registry pack.
pub async fn verify(endpoint: &str) -> Result<VerifyOutcome> {
    verify_with_profile(endpoint, ProfileId::Generic).await
}

pub async fn verify_with_profile(endpoint: &str, profile_id: ProfileId) -> Result<VerifyOutcome> {
    verify_with_options(
        endpoint,
        VerifyOptions {
            profile: profile_id,
            selection: VerifySelection::Unversioned,
            registry: None,
            vendor_root: None,
            declared_target: DeclaredTarget::default(),
            drs_fixture: crate::fixture::DrsVerifyFixture::default_catalog(),
        },
    )
    .await
}

pub async fn verify_with_options(endpoint: &str, options: VerifyOptions) -> Result<VerifyOutcome> {
    crate::checker::require_checker_pin()?;
    let mut outcome = match &options.selection {
        VerifySelection::Unversioned => verify_unversioned(endpoint, &options).await?,
        VerifySelection::Explicit { .. }
        | VerifySelection::Automatic { .. }
        | VerifySelection::Compatibility { .. } => verify_versioned(endpoint, options).await?,
    };
    crate::traceability::bind_run(&mut outcome.run)?;
    Ok(outcome)
}

/// Default path. Does not load the standards registry. Does not label HelixTest as a GA4GH pack.
async fn verify_unversioned(endpoint: &str, options: &VerifyOptions) -> Result<VerifyOutcome> {
    let mut outcome = execute_profile(endpoint, options).await?;
    stamp_unversioned(&mut outcome);
    Ok(outcome)
}

async fn verify_versioned(endpoint: &str, options: VerifyOptions) -> Result<VerifyOutcome> {
    let registry = match options.registry {
        Some(r) => r,
        None => load_path(&default_registry_path()).context("standards registry")?,
    };
    let (standard, requested, class, mode) = match &options.selection {
        VerifySelection::Explicit {
            standard,
            version,
            release_class,
        } => (
            standard.clone(),
            Some(version.clone()),
            *release_class,
            "explicit",
        ),
        VerifySelection::Automatic { standard } => (standard.clone(), None, None, "automatic"),
        VerifySelection::Compatibility { standard } => {
            (standard.clone(), None, None, "compatibility")
        }
        VerifySelection::Unversioned => unreachable!(),
    };

    let mut outcome = discover_only(
        endpoint,
        options.profile,
        &options.declared_target,
        &options.drs_fixture,
    )
    .await?;
    let detected = detected_for_standard(&outcome.discovery, &standard);

    let resolved = match &options.selection {
        VerifySelection::Explicit { version, .. } => {
            select_explicit(&registry, &standard, version, class).map(|p| vec![p])
        }
        VerifySelection::Automatic { .. } => {
            select_automatic(&registry, &standard, detected.as_deref()).map(|p| vec![p])
        }
        VerifySelection::Compatibility { .. } => {
            select_all_official_supported(&registry, &standard)
        }
        VerifySelection::Unversioned => unreachable!(),
    };

    match resolved {
        Ok(packs) if packs.len() == 1 => {
            let pack_ref = packs[0].clone();
            let version = registry
                .versions
                .iter()
                .find(|v| v.pack_id == pack_ref.pack_id)
                .expect("selected pack is in registry")
                .clone();
            let vendor_root = options.vendor_root.clone().unwrap_or_else(helix_repo_root);
            let join = execute_selected_pack(
                &mut outcome,
                options.profile,
                &version,
                &vendor_root,
                &options.drs_fixture,
            )
            .await?;
            stamp_selected(&mut outcome, requested.as_deref(), &pack_ref, join);
            Ok(outcome)
        }
        Ok(packs) if packs.is_empty() => {
            apply_selection_error(
                &mut outcome,
                options.profile,
                mode,
                &standard,
                requested.as_deref(),
                detected.as_deref(),
                SelectionError::NoOfficialSupported {
                    standard: standard.clone(),
                },
            );
            Ok(outcome)
        }
        Ok(packs) => {
            // HelixTest still compiles one OpenAPI. Do not pick one version or run AVAILABLE extras.
            skip_named_and_others(
                &mut outcome.run,
                definition(options.profile),
                &standard,
                &format!(
                    "Helix cannot execute multiple OfficialSupported {standard} packs in one process \
                     (shared HelixTest schema). Helix did not pick one version. Checks not executed (not a pass)."
                ),
            );
            outcome.run.sort_deterministic();
            let selection = StandardSelection {
                mode: "compatibility".into(),
                selection_status: MULTIPLE_PACKS_NOT_EXECUTABLE.into(),
                substituted: false,
                standard: Some(standard.clone()),
                requested_version: requested.clone(),
                detected_version: detected.clone(),
                selected_version: None,
                verified_version: None,
                standards_registry_entry: None,
                standards_source_commit: None,
                other_rows_not_selected: packs.iter().map(|p| p.pack_id.clone()).collect(),
                note: Some(
                    "Mode 3 requires per-pack schemas. AVAILABLE rows were not executed. \
                     Helix did not substitute another version."
                        .into(),
                ),
                integrity_validated: false,
                integrity_ok: None,
                pack_integrity_sha256: None,
                schema_document_sha256: None,
                schema_component_sha256: None,
                execution_id: None,
                target_execution_id: None,
                schema_entry: None,
                catalog_id: None,
                binding_id: None,
                checker_id: None,
                support_status: None,
                coverage_schema: None,
                coverage_behavior: None,
                coverage_security: None,
                coverage_interoperability: None,
            };
            stamp_outcome(
                &mut outcome,
                &VersionStamp {
                    standard: Some(standard),
                    requested_version: requested,
                    selected_version: None,
                    verified_version: None,
                    registry_entry: None,
                    commit: None,
                },
                selection,
            );
            Ok(outcome)
        }
        Err(err) => {
            apply_selection_error(
                &mut outcome,
                options.profile,
                mode,
                &standard,
                requested.as_deref(),
                detected.as_deref(),
                err,
            );
            Ok(outcome)
        }
    }
}

#[derive(Debug, Clone, Default)]
struct JoinRecord {
    integrity_validated: bool,
    integrity_ok: Option<bool>,
    pack_integrity_sha256: Option<String>,
    schema_document_sha256: Option<String>,
    schema_component_sha256: Option<String>,
    execution_id: Option<String>,
    schema_entry: Option<String>,
}

async fn execute_selected_pack(
    outcome: &mut VerifyOutcome,
    profile_id: ProfileId,
    pack: &crate::standards::StandardVersion,
    vendor_root: &std::path::Path,
    fixture: &crate::fixture::DrsVerifyFixture,
) -> Result<JoinRecord> {
    let profile = definition(profile_id);
    let standard = pack.standard.as_str();
    let Some(kind) = Ga4ghService::from_json_name(standard) else {
        skip_named_and_others(
            &mut outcome.run,
            profile,
            standard,
            &format!(
                "Helix has no executable suite for {standard}; pack {} was selected but not run.",
                pack.pack_id
            ),
        );
        outcome.run.sort_deterministic();
        return Ok(JoinRecord::default());
    };

    if kind != Ga4ghService::Drs {
        skip_named_and_others(
            &mut outcome.run,
            profile,
            standard,
            &format!(
                "pack {} is not executable: WES ServiceInfo is not a hashed local SpecSource",
                pack.pack_id
            ),
        );
        outcome.run.sort_deterministic();
        return Ok(JoinRecord::default());
    }

    let loaded = match load_pack(pack, vendor_root) {
        Ok(loaded) => loaded,
        Err(e) => {
            let msg = match &e {
                PackLoadError::Integrity { message } => {
                    format!("pack integrity failed; HelixTest was not invoked: {message}")
                }
                other => format!("pack load failed; HelixTest was not invoked: {other}"),
            };
            for result in profile_errors(kind, &msg) {
                outcome.run.push_executed(result);
            }
            skip_other_standards(&mut outcome.run, profile, standard);
            outcome.run.sort_deterministic();
            return Ok(JoinRecord {
                integrity_validated: true,
                integrity_ok: Some(false),
                pack_integrity_sha256: pack.pack_integrity.as_ref().map(|i| i.hex.clone()),
                schema_document_sha256: None,
                schema_component_sha256: None,
                execution_id: None,
                schema_entry: pack.schema_entry.clone(),
            });
        }
    };

    if !target_connectable(&outcome.discovery.endpoint) {
        for result in profile_errors(kind, &unreachable_message(kind)) {
            outcome.run.push_executed(result);
        }
        skip_other_standards(&mut outcome.run, profile, standard);
        outcome.run.sort_deterministic();
        return Ok(join_pack_loaded(&loaded));
    }

    let adapter = HelixTestAdapter::pinned().with_capabilities(profile.capabilities);
    let rec = outcome
        .discovery
        .record(kind)
        .expect("discovery always records VERIFY_ORDER services")
        .clone();
    match (rec.detection, rec.testability) {
        (Detection::Detected, Testability::Testable) => {
            let url = rec
                .base_url()
                .expect("DETECTED TESTABLE service has a base URL")
                .to_string();
            match adapter
                .run_drs_with_spec(&url, &loaded.spec, &fixture.to_helixtest())
                .await
            {
                Ok((out, returned)) => {
                    if let Err(e) = compare_spec_identity(&loaded.expected, &returned) {
                        for result in profile_errors(
                            kind,
                            &format!("SpecSource identity mismatch; results discarded: {e}"),
                        ) {
                            outcome.run.push_executed(result);
                        }
                        skip_other_standards(&mut outcome.run, profile, standard);
                        outcome.run.sort_deterministic();
                        return Ok(join_pack_loaded(&loaded));
                    }
                    outcome.run.helixtest_version = Some(out.pin.tag.to_string());
                    outcome.run.helixtest_sha = Some(out.pin.sha.to_string());
                    for r in out.results {
                        outcome.run.push_executed(r);
                    }
                    skip_other_standards(&mut outcome.run, profile, standard);
                    outcome.run.sort_deterministic();
                    return Ok(join_from_loaded(&loaded, Some(out.pin.sha)));
                }
                Err(e) => {
                    for result in profile_errors(kind, &format!("HelixTest adapter error: {e}")) {
                        outcome.run.push_executed(result);
                    }
                    skip_other_standards(&mut outcome.run, profile, standard);
                    outcome.run.sort_deterministic();
                    return Ok(join_pack_loaded(&loaded));
                }
            }
        }
        (Detection::Detected, Testability::NotTestable) => {
            let reason = rec
                .not_testable_reason
                .clone()
                .unwrap_or_else(|| skip_not_testable(kind));
            apply_missing(&mut outcome.run, profile, kind, &reason, true);
        }
        (Detection::NotDetected, _) => {
            apply_missing(
                &mut outcome.run,
                profile,
                kind,
                &skip_not_detected(kind),
                false,
            );
        }
    }
    skip_other_standards(&mut outcome.run, profile, standard);
    outcome.run.sort_deterministic();
    Ok(join_pack_loaded(&loaded))
}

/// Pack bytes verified; checker did not return a matching SpecCompileResult.
fn join_pack_loaded(loaded: &crate::standards::LoadedPack) -> JoinRecord {
    JoinRecord {
        integrity_validated: true,
        integrity_ok: Some(true),
        pack_integrity_sha256: Some(loaded.pack_integrity_sha256.clone()),
        schema_document_sha256: None,
        schema_component_sha256: None,
        execution_id: None,
        schema_entry: Some(loaded.spec.schema_entry.clone()),
    }
}

fn join_from_loaded(
    loaded: &crate::standards::LoadedPack,
    _helixtest_sha: Option<&str>,
) -> JoinRecord {
    let checker = crate::checker::executed_checker_id();
    let exec = execution_id(
        &loaded.pack_id,
        &loaded.pack_integrity_sha256,
        &loaded.expected.schema_document_sha256,
        &loaded.expected.schema_component_sha256,
        &checker,
        &loaded.spec.schema_entry,
        &loaded.spec.schema_component,
    );
    JoinRecord {
        integrity_validated: true,
        integrity_ok: Some(true),
        pack_integrity_sha256: Some(loaded.pack_integrity_sha256.clone()),
        schema_document_sha256: Some(loaded.expected.schema_document_sha256.clone()),
        schema_component_sha256: Some(loaded.expected.schema_component_sha256.clone()),
        execution_id: Some(exec),
        schema_entry: Some(loaded.spec.schema_entry.clone()),
    }
}

fn apply_selection_error(
    outcome: &mut VerifyOutcome,
    profile_id: ProfileId,
    mode: &str,
    standard: &str,
    requested: Option<&str>,
    detected: Option<&str>,
    err: SelectionError,
) {
    skip_named_and_others(
        &mut outcome.run,
        definition(profile_id),
        standard,
        &err.skip_message(),
    );
    outcome.run.sort_deterministic();

    let pack = err.registry_pack().cloned();
    let note = match err.status_code() {
        AVAILABLE_BUT_NOT_SUPPORTED => Some(
            "This version exists as a pinned GA4GH release in the Helix registry but Helix has no SUPPORTED pack. \
             Helix did not substitute another version. A GitHub tag alone does not make a version supported. \
             This is not a claim that the target declared this version."
                .into(),
        ),
        _ => Some(
            "Helix did not substitute another version. This is not a claim that the target declared a version."
                .into(),
        ),
    };
    let selection = StandardSelection {
        mode: mode.into(),
        selection_status: err.status_code().into(),
        substituted: false,
        standard: Some(standard.into()),
        requested_version: requested.map(str::to_string),
        detected_version: detected.map(str::to_string),
        selected_version: None,
        verified_version: None,
        standards_registry_entry: pack.as_ref().map(|p| p.pack_id.clone()),
        standards_source_commit: pack.as_ref().map(|p| p.commit.clone()),
        other_rows_not_selected: err.other_rows_not_selected(),
        note,
        integrity_validated: false,
        integrity_ok: None,
        pack_integrity_sha256: None,
        schema_document_sha256: None,
        schema_component_sha256: None,
        execution_id: None,
        schema_entry: None,
        ..StandardSelection::unversioned()
    };
    stamp_outcome(
        outcome,
        &VersionStamp {
            standard: Some(standard.into()),
            requested_version: requested.map(str::to_string),
            selected_version: None,
            verified_version: None,
            registry_entry: pack.as_ref().map(|p| p.pack_id.clone()),
            commit: pack.as_ref().map(|p| p.commit.clone()),
        },
        selection,
    );
}

fn skip_named_and_others(
    run: &mut VerificationRun,
    profile: Profile,
    standard: &str,
    named_reason: &str,
) {
    for kind in profile.enabled_services {
        if kind.json_name() == standard {
            for result in profile_skips(*kind, named_reason) {
                run.push_skipped(result);
            }
        } else {
            for result in profile_skips(*kind, SKIP_STANDARD_NOT_SELECTED) {
                run.push_skipped(result);
            }
        }
    }
}

fn skip_other_standards(run: &mut VerificationRun, profile: Profile, standard: &str) {
    for kind in profile.enabled_services {
        if kind.json_name() != standard {
            for result in profile_skips(*kind, SKIP_STANDARD_NOT_SELECTED) {
                run.push_skipped(result);
            }
        }
    }
}

struct VersionStamp {
    standard: Option<String>,
    requested_version: Option<String>,
    selected_version: Option<String>,
    verified_version: Option<String>,
    registry_entry: Option<String>,
    commit: Option<String>,
}

fn stamp_unversioned(outcome: &mut VerifyOutcome) {
    stamp_outcome(
        outcome,
        &VersionStamp {
            standard: None,
            requested_version: None,
            selected_version: None,
            verified_version: None,
            registry_entry: None,
            commit: None,
        },
        StandardSelection::unversioned(),
    );
}

fn stamp_selected(
    outcome: &mut VerifyOutcome,
    requested: Option<&str>,
    pack: &PackRef,
    join: JoinRecord,
) {
    let detected = detected_for_standard(&outcome.discovery, &pack.standard);
    let mut selection = StandardSelection {
        mode: if requested.is_some() {
            "explicit"
        } else {
            "automatic"
        }
        .into(),
        selection_status: SELECTED.into(),
        substituted: false,
        standard: Some(pack.standard.clone()),
        requested_version: requested.map(str::to_string),
        detected_version: detected,
        selected_version: Some(pack.version.clone()),
        verified_version: None,
        standards_registry_entry: Some(pack.pack_id.clone()),
        standards_source_commit: Some(pack.commit.clone()),
        other_rows_not_selected: Vec::new(),
        note: Some(
            "Helix selected this registry pack. Join success is not a VERIFIED claim. \
             verified_version stays empty until claim predicates permit a version sentence. \
             SUPPORTED is not VERIFIED."
                .into(),
        ),
        integrity_validated: join.integrity_validated,
        integrity_ok: join.integrity_ok,
        pack_integrity_sha256: join.pack_integrity_sha256.clone(),
        schema_document_sha256: join.schema_document_sha256.clone(),
        schema_component_sha256: join.schema_component_sha256.clone(),
        execution_id: join.execution_id.clone(),
        schema_entry: join.schema_entry.clone(),
        ..StandardSelection::unversioned()
    };
    apply_support_identities(&mut selection, pack, &join);
    stamp_outcome(
        outcome,
        &VersionStamp {
            standard: Some(pack.standard.clone()),
            requested_version: requested.map(str::to_string),
            selected_version: Some(pack.version.clone()),
            verified_version: None,
            registry_entry: Some(pack.pack_id.clone()),
            commit: Some(pack.commit.clone()),
        },
        selection,
    );
}

fn apply_support_identities(sel: &mut StandardSelection, pack: &PackRef, join: &JoinRecord) {
    let Some(contract) = contract_for(&pack.pack_id) else {
        return;
    };
    sel.catalog_id = Some(compute_catalog_id(contract));
    sel.checker_id = Some(declared_checker_id());
    sel.support_status = Some("supported".into());
    sel.coverage_schema = Some(contract.coverage.schema.as_str().into());
    sel.coverage_behavior = Some(contract.coverage.behavior.as_str().into());
    sel.coverage_security = Some(contract.coverage.security.as_str().into());
    sel.coverage_interoperability = Some(contract.coverage.interoperability.as_str().into());
    if let (Some(p), Some(d), Some(c)) = (
        join.pack_integrity_sha256.as_deref(),
        join.schema_document_sha256.as_deref(),
        join.schema_component_sha256.as_deref(),
    ) {
        sel.binding_id = Some(compute_binding_id(contract, p, d, c));
    }
}

fn stamp_outcome(
    outcome: &mut VerifyOutcome,
    stamp: &VersionStamp,
    mut selection: StandardSelection,
) {
    if selection.detected_version.is_none() {
        if let Some(std) = stamp.standard.as_deref() {
            selection.detected_version = detected_for_standard(&outcome.discovery, std);
        }
    }
    if let Some(identity) = outcome.run.target.identity.as_mut() {
        if identity.detected.standard_version.is_none() {
            identity.detected.standard_version = selection.detected_version.clone();
        }
    }
    if let Some(identity) = outcome.run.target.identity.as_ref() {
        selection.target_execution_id = Some(crate::target::target_execution_id(
            identity,
            &selection,
            outcome.run.drs_fixture.as_ref(),
        ));
    }
    let target_id = outcome
        .run
        .target
        .identity
        .as_ref()
        .map(|i| i.target_id.clone());
    outcome.run.standard_selection = Some(selection);
    for r in outcome
        .run
        .executed
        .iter_mut()
        .chain(outcome.run.skipped.iter_mut())
    {
        r.target_id = target_id.clone();
        r.standard = Some(r.service.clone());
        r.detected_version = detected_for_standard(&outcome.discovery, &r.service);
        if stamp.standard.as_deref() == Some(r.service.as_str()) {
            r.requested_version = stamp.requested_version.clone();
            r.selected_version = stamp.selected_version.clone();
            r.verified_version = stamp.verified_version.clone();
            r.standards_registry_entry = stamp.registry_entry.clone();
            r.standards_source_commit = stamp.commit.clone();
        } else {
            r.requested_version = None;
            r.selected_version = None;
            r.verified_version = None;
            r.standards_registry_entry = None;
            r.standards_source_commit = None;
        }
    }
}

fn detected_for_standard(discovery: &Discovery, standard: &str) -> Option<String> {
    let kind = Ga4ghService::from_json_name(standard)?;
    discovery.record(kind)?.detected_standard_version()
}

async fn execute_profile(endpoint: &str, options: &VerifyOptions) -> Result<VerifyOutcome> {
    let profile_id = options.profile;
    let mut outcome = discover_only(
        endpoint,
        profile_id,
        &options.declared_target,
        &options.drs_fixture,
    )
    .await?;
    if !target_connectable(&outcome.discovery.endpoint) {
        let profile = definition(profile_id);
        for kind in profile.enabled_services {
            for result in profile_errors(*kind, &unreachable_message(*kind)) {
                outcome.run.push_executed(result);
            }
        }
        outcome.run.sort_deterministic();
        return Ok(outcome);
    }

    let profile = definition(profile_id);
    let adapter = HelixTestAdapter::pinned().with_capabilities(profile.capabilities);
    for kind in profile.enabled_services {
        let rec = outcome
            .discovery
            .record(*kind)
            .expect("discovery always records VERIFY_ORDER services")
            .clone();
        match (rec.detection, rec.testability) {
            (Detection::Detected, Testability::Testable) => {
                let url = rec
                    .base_url()
                    .expect("DETECTED TESTABLE service has a base URL")
                    .to_string();
                match run_adapter(&adapter, *kind, &url, &options.drs_fixture).await {
                    Ok(out) => {
                        outcome.run.helixtest_version = Some(out.pin.tag.to_string());
                        outcome.run.helixtest_sha = Some(out.pin.sha.to_string());
                        for r in out.results {
                            outcome.run.push_executed(r);
                        }
                    }
                    Err(e) => {
                        for result in
                            profile_errors(*kind, &format!("HelixTest adapter error: {e}"))
                        {
                            outcome.run.push_executed(result);
                        }
                    }
                }
            }
            (Detection::Detected, Testability::NotTestable) => {
                let reason = rec
                    .not_testable_reason
                    .clone()
                    .unwrap_or_else(|| skip_not_testable(*kind));
                apply_missing(&mut outcome.run, profile, *kind, &reason, true);
            }
            (Detection::NotDetected, _) => {
                apply_missing(
                    &mut outcome.run,
                    profile,
                    *kind,
                    &skip_not_detected(*kind),
                    false,
                );
            }
        }
    }

    outcome.run.sort_deterministic();
    Ok(outcome)
}

async fn discover_only(
    endpoint: &str,
    profile_id: ProfileId,
    declared: &DeclaredTarget,
    fixture: &crate::fixture::DrsVerifyFixture,
) -> Result<VerifyOutcome> {
    let profile = definition(profile_id);
    let endpoint = normalize_endpoint(endpoint)?;
    let identity = TargetIdentity::from_declared(&endpoint, declared);
    let mut run = VerificationRun::new(Target::from_identity(identity));
    run.profile = Some(profile.id.as_str().to_string());
    run.helixtest_version = Some(HELIXTEST_PIN.to_string());
    run.helixtest_sha = Some(crate::checker::executed_checker_source_sha256().to_string());
    run.drs_fixture = Some(fixture.clone());

    if !target_connectable(&endpoint) {
        let discovery = Discovery {
            endpoint: endpoint.clone(),
            services: VERIFY_ORDER
                .iter()
                .map(|k| ServiceDiscovery::not_detected(*k))
                .collect(),
        };
        run.discovery = model_discovery(&discovery);
        return Ok(VerifyOutcome { discovery, run });
    }

    let client = http_client()?;
    let discovery = discover_for_drs_object(&endpoint, &client, &fixture.object_id).await?;
    run.discovery = model_discovery(&discovery);
    Ok(VerifyOutcome { discovery, run })
}

async fn run_adapter(
    adapter: &HelixTestAdapter,
    kind: Ga4ghService,
    url: &str,
    fixture: &crate::fixture::DrsVerifyFixture,
) -> Result<crate::adapter::AdapterOutcome> {
    match kind {
        Ga4ghService::Drs => adapter.run_drs(url, &fixture.to_helixtest()).await,
        Ga4ghService::Wes => adapter.run_wes(url).await,
        other => anyhow::bail!("helix verify does not execute {}", other.as_str()),
    }
}

/// Expected-but-missing → fail. Otherwise skip. Skip is never pass.
fn apply_missing(
    run: &mut VerificationRun,
    profile: Profile,
    kind: Ga4ghService,
    skip_reason: &str,
    detected_not_testable: bool,
) {
    if profile.expects(kind) {
        let msg = if detected_not_testable {
            format!(
                "{} expected by profile {} but not TESTABLE",
                kind.as_str(),
                profile.id.as_str()
            )
        } else {
            format!(
                "{} expected by profile {} but not detected",
                kind.as_str(),
                profile.id.as_str()
            )
        };
        for result in profile_fails(kind, &msg) {
            run.push_executed(result);
        }
    } else {
        for result in profile_skips(kind, skip_reason) {
            run.push_skipped(result);
        }
    }
}

fn skip_not_detected(kind: Ga4ghService) -> String {
    format!(
        "{} not detected; {} checks not executed (not a pass)",
        kind.as_str(),
        kind.as_str()
    )
}

fn skip_not_testable(kind: Ga4ghService) -> String {
    format!(
        "{} detected but not TESTABLE; checks not executed (DETECTED is not a pass)",
        kind.as_str()
    )
}

fn unreachable_message(kind: Ga4ghService) -> String {
    format!("target unreachable; {} checks not executed", kind.as_str())
}

fn model_discovery(d: &Discovery) -> Vec<DiscoveredService> {
    d.services.iter().map(to_model_service).collect()
}

fn to_model_service(rec: &ServiceDiscovery) -> DiscoveredService {
    let name = rec.kind.json_name();
    match rec.detection {
        Detection::NotDetected => DiscoveredService::missing(name),
        Detection::Detected => {
            if rec.testability == Testability::Testable {
                DiscoveredService::found(name, rec.base_url.clone().unwrap_or_default())
            } else {
                DiscoveredService::detected_not_testable(
                    name,
                    rec.base_url.clone().unwrap_or_default(),
                    rec.not_testable_reason
                        .clone()
                        .unwrap_or_else(|| skip_not_testable(rec.kind)),
                )
            }
        }
    }
}

fn helix_check(spec: &CheckSpec) -> VerificationCheck {
    VerificationCheck::from_spec(spec).with_profile("generic")
}

fn with_original_name(mut result: VerificationResult, spec: &CheckSpec) -> VerificationResult {
    if let Some(ht) = spec.helixtest_names.first() {
        result = result.with_helixtest_name(*ht);
    }
    result
}

fn verify_specs(kind: Ga4ghService) -> Vec<&'static CheckSpec> {
    match kind {
        Ga4ghService::Drs => drs_verify_specs().collect(),
        Ga4ghService::Wes => wes_verify_specs().collect(),
        _ => Vec::new(),
    }
}

fn profile_skips(kind: Ga4ghService, reason: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::skip(helix_check(s), reason), s))
        .collect()
}

fn profile_fails(kind: Ga4ghService, message: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::fail(helix_check(s), message), s))
        .collect()
}

fn profile_errors(kind: Ga4ghService, message: &str) -> Vec<VerificationResult> {
    verify_specs(kind)
        .into_iter()
        .map(|s| with_original_name(VerificationResult::error(helix_check(s), message), s))
        .collect()
}

/// TCP connect to the origin host:port. Distinct from HTTP 404 (missing API).
pub fn target_connectable(endpoint: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(endpoint) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    let Ok(mut addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    let timeout = Duration::from_millis(400);
    addrs.any(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::VerificationStatus;

    #[test]
    fn skipped_drs_rows_are_never_pass() {
        for r in profile_skips(Ga4ghService::Drs, &skip_not_detected(Ga4ghService::Drs)) {
            assert_eq!(r.status, VerificationStatus::Skip);
            assert!(!r.is_pass());
            assert!(r.id.starts_with("drs.object."));
            assert!(r.code.starts_with("HLX-DRS-"));
            assert_eq!(r.service, "drs");
        }
        assert_eq!(profile_skips(Ga4ghService::Drs, "x").len(), 5);
    }

    #[test]
    fn skipped_wes_rows_are_never_pass() {
        for r in profile_skips(Ga4ghService::Wes, &skip_not_detected(Ga4ghService::Wes)) {
            assert_eq!(r.status, VerificationStatus::Skip);
            assert!(!r.is_pass());
            assert!(r.id.starts_with("wes."));
            assert!(r.code.starts_with("HLX-WES-"));
            assert_eq!(r.service, "wes");
        }
        assert_eq!(profile_skips(Ga4ghService::Wes, "x").len(), 8);
        assert_eq!(profile_fails(Ga4ghService::Wes, "expected").len(), 8);
        for r in profile_fails(Ga4ghService::Wes, "expected") {
            assert_eq!(r.status, VerificationStatus::Fail);
            assert!(!r.is_pass());
        }
    }
}
