// SPDX-License-Identifier: Apache-2.0
//! B2 versioned DRS execution join. Not SUPPORTED. Not VERIFIED. Not HELIOS.

use common::spec_source::{reset_schema_call_counters, validate_with_spec, SpecSource};
use helix::claims::{evaluate, ClaimStatus};
use helix::profile::ProfileId;
use helix::standards::{
    compare_spec_identity, copy_pack_tree, default_registry_path, helix_repo_root, hex_sha256,
    load_pack, load_path, pack_integrity_hex, DRS_OBJECT_CLOSURE,
};
use helix::verify::{VerifyOptions, VerifySelection};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn shipped_drs(pack_id: &str) -> helix::standards::StandardVersion {
    load_path(&default_registry_path())
        .unwrap()
        .versions
        .into_iter()
        .find(|v| v.pack_id == pack_id)
        .unwrap()
}

fn fixture_supported_drs_140() -> helix::standards::Registry {
    load_path(&default_registry_path()).unwrap()
}

const DRS_OBJECT_YAML: &str = "openapi/components/schemas/DrsObject.yaml";
const REQUIRED_ID: &str = "required:\n  - id\n";
const REQUIRED_INJECTED: &str = "required:\n  - deliberately_injected_field\n  - id\n";

fn inject_required_field(yaml: &str) -> String {
    let out = yaml.replacen(REQUIRED_ID, REQUIRED_INJECTED, 1);
    assert_ne!(
        out, yaml,
        "DrsObject.yaml must contain the required-id block"
    );
    out
}

fn rehash_drs_140_from_disk(reg: &mut helix::standards::Registry, repo_root: &Path) {
    let v = reg
        .versions
        .iter_mut()
        .find(|v| v.pack_id == "ga4gh.drs.1.4.0")
        .expect("ga4gh.drs.1.4.0");
    let mut files: BTreeMap<String, Arc<[u8]>> = BTreeMap::new();
    for src in &mut v.normative_sources {
        let vp = src.vendor_path.as_ref().expect("vendor_path");
        let bytes = std::fs::read(repo_root.join(vp)).unwrap();
        src.integrity.hex = hex_sha256(&bytes);
        files.insert(src.path.clone(), Arc::from(bytes));
    }
    v.pack_integrity.as_mut().expect("pack_integrity").hex = pack_integrity_hex(&files);
}

fn fixture_supported_drs_140_at(vendor_root: &Path) -> helix::standards::Registry {
    let mut reg = load_path(&default_registry_path()).unwrap();
    rehash_drs_140_from_disk(&mut reg, vendor_root);
    // Only 1.4.0 is copied into `vendor_root`. Do not validate_loaded the full registry.
    let pack = reg
        .versions
        .iter()
        .find(|v| v.pack_id == "ga4gh.drs.1.4.0")
        .expect("ga4gh.drs.1.4.0");
    load_pack(pack, vendor_root).expect("mutated 1.4.0 pack must load after rehash");
    reg
}

fn explicit_drs_140(
    registry: helix::standards::Registry,
    vendor_root: Option<PathBuf>,
) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        registry: Some(registry),
        vendor_root,
        declared_target: helix::target::DeclaredTarget::default(),
    }
}

fn ok_payload() -> serde_json::Value {
    json!({
        "id": "test-object-1",
        "self_uri": "drs://example.org/test-object-1",
        "size": 12,
        "created_time": "2026-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": "abc" }]
    })
}

#[test]
fn test1_corrupt_vendor_byte_does_not_compile_spec() {
    let tmp = tempfile::tempdir().unwrap();
    copy_pack_tree(&helix_repo_root(), "ga4gh.drs.1.4.0", tmp.path()).unwrap();
    let target = tmp
        .path()
        .join("standards/vendor/ga4gh.drs.1.4.0/openapi/components/schemas/DrsObject.yaml");
    let mut bytes = std::fs::read(&target).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&target, bytes).unwrap();
    reset_schema_call_counters();
    let err = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), tmp.path()).unwrap_err();
    assert!(err.to_string().contains("hash mismatch"), "{err}");
    // Integrity failure is before SpecSource compile / HelixTest.
}

#[test]
fn test2_missing_each_drs_closure_file_rejects_load() {
    for rel in DRS_OBJECT_CLOSURE {
        let tmp = tempfile::tempdir().unwrap();
        copy_pack_tree(&helix_repo_root(), "ga4gh.drs.1.4.0", tmp.path()).unwrap();
        let path = tmp
            .path()
            .join("standards/vendor/ga4gh.drs.1.4.0")
            .join(rel);
        std::fs::remove_file(&path).unwrap();
        reset_schema_call_counters();
        let err = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), tmp.path()).unwrap_err();
        assert!(
            err.to_string().contains("missing") || err.to_string().contains("hash"),
            "{rel}: {err}"
        );
    }
}

#[test]
fn test3_mutated_schema_bytes_change_checker_behavior() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    validate_with_spec(&loaded.spec, &ok_payload()).expect("original DrsObject accepts payload");
    let mut spec = loaded.spec.clone();
    let key = "openapi/components/schemas/DrsObject.yaml";
    let yaml = String::from_utf8(spec.files.get(key).unwrap().to_vec()).unwrap();
    let yaml = yaml.replacen(
        "required:\n  - id\n",
        "required:\n  - deliberately_injected_field\n  - id\n",
        1,
    );
    spec.files.insert(key.into(), Arc::from(yaml.into_bytes()));
    let err = validate_with_spec(&spec, &ok_payload())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("deliberately_injected_field") || err.contains("required"),
        "{err}"
    );
}

#[test]
fn test4_checker_returns_schema_identity() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let returned = validate_with_spec(&loaded.spec, &ok_payload()).unwrap();
    assert_eq!(returned.schema_document_sha256.len(), 64);
    assert_eq!(returned.schema_component_sha256.len(), 64);
    assert!(!returned.files_opened.is_empty());
    compare_spec_identity(&loaded.expected, &returned).unwrap();
}

#[test]
fn test5_lying_checker_identity_fails_closed() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let mut lie = loaded.expected.clone();
    lie.schema_document_sha256 = "0".repeat(64);
    let err = compare_spec_identity(&loaded.expected, &lie)
        .unwrap_err()
        .to_string();
    assert!(err.contains("schema_document_sha256 mismatch"), "{err}");
    lie = loaded.expected.clone();
    lie.schema_component_sha256 = "1".repeat(64);
    let err = compare_spec_identity(&loaded.expected, &lie)
        .unwrap_err()
        .to_string();
    assert!(err.contains("schema_component_sha256 mismatch"), "{err}");
    let verify = include_str!("../src/verify.rs");
    assert!(verify.contains("compare_spec_identity(&loaded.expected, &returned)"));
    assert!(verify.contains("SpecSource identity mismatch; results discarded"));
    assert!(verify.contains("return Ok(join_pack_loaded(&loaded))"));
}

#[test]
fn test6_versioned_adapter_does_not_call_bundled_drs() {
    let adapter = include_str!("../src/adapter/mod.rs");
    let start = adapter
        .find("    /// Versioned DRS path")
        .expect("versioned DRS adapter comment");
    let rest = &adapter[start..];
    let end = rest
        .find("\nimpl ConformanceAdapter")
        .expect("ConformanceAdapter impl follows run_drs_with_spec");
    let body = &rest[..end];
    assert!(body.contains("run_drs_checks_with_spec"));
    assert!(
        !body.contains("run_drs_checks("),
        "versioned adapter must not call bundled run_drs_checks"
    );
    assert!(!body.contains("validate_drs_object("));
    let verify = include_str!("../src/verify.rs");
    let exec = verify
        .find("async fn execute_selected_pack")
        .expect("execute_selected_pack");
    let exec_body = &verify[exec..];
    let join_at = exec_body.find("fn join_from_loaded").unwrap();
    assert!(exec_body.contains("run_drs_with_spec"));
    assert!(!exec_body[..join_at].contains("run_adapter("));
}

#[test]
fn test7_outside_closure_mutation_changes_pack_not_schema_document() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let pack_before = loaded.pack_integrity_sha256.clone();
    let doc_before = loaded.expected.schema_document_sha256.clone();
    let mut files = loaded.spec.files.clone();
    let tag = "openapi/tags/Introduction.md";
    let mut t = files.get(tag).unwrap().to_vec();
    t.push(b'\n');
    files.insert(tag.into(), Arc::from(t));
    let pack_after = pack_integrity_hex(&files);
    assert_ne!(pack_before, pack_after);
    let spec = SpecSource {
        schema_entry: loaded.spec.schema_entry.clone(),
        schema_component: loaded.spec.schema_component.clone(),
        files,
    };
    let after = common::spec_source::resolve_schema_value(&spec).unwrap().1;
    assert_eq!(doc_before, after.schema_document_sha256);
}

#[test]
fn test8_inside_closure_mutation_changes_identities() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let mut files = loaded.spec.files.clone();
    let key = "openapi/components/schemas/DrsObject.yaml";
    let mut yaml = String::from_utf8(files.get(key).unwrap().to_vec()).unwrap();
    yaml = yaml.replacen(REQUIRED_ID, REQUIRED_INJECTED, 1);
    files.insert(key.into(), Arc::from(yaml.into_bytes()));
    let pack_after = pack_integrity_hex(&files);
    assert_ne!(loaded.pack_integrity_sha256, pack_after);
    let spec = SpecSource {
        schema_entry: loaded.spec.schema_entry.clone(),
        schema_component: loaded.spec.schema_component.clone(),
        files,
    };
    let after = common::spec_source::resolve_schema_value(&spec).unwrap().1;
    assert_ne!(
        loaded.expected.schema_document_sha256,
        after.schema_document_sha256
    );
    assert_ne!(
        loaded.expected.schema_component_sha256,
        after.schema_component_sha256
    );
}

#[test]
fn test9_two_specsources_do_not_share_name_only_cache() {
    reset_schema_call_counters();
    let a = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let mut bspec = a.spec.clone();
    let key = "openapi/components/schemas/DrsObject.yaml";
    let yaml = String::from_utf8(bspec.files.get(key).unwrap().to_vec()).unwrap();
    let yaml = yaml.replacen(
        "required:\n  - id\n",
        "required:\n  - deliberately_injected_field\n  - id\n",
        1,
    );
    bspec.files.insert(key.into(), Arc::from(yaml.into_bytes()));
    validate_with_spec(&a.spec, &ok_payload()).unwrap();
    assert!(validate_with_spec(&bspec, &ok_payload()).is_err());
    validate_with_spec(&a.spec, &ok_payload()).unwrap();
}

#[test]
fn test10_http_ref_fails_closed_and_local_pack_needs_no_network() {
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    validate_with_spec(&loaded.spec, &ok_payload()).unwrap();
    let mut spec = loaded.spec.clone();
    spec.files.insert(
        "openapi/components/schemas/Checksum.yaml".into(),
        Arc::from(&b"$ref: 'https://example.invalid/schema.yaml'\n"[..]),
    );
    let err = common::spec_source::resolve_schema_value(&spec)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("local relative") || err.contains("$ref"),
        "{err}"
    );
    let spec_src = include_str!("../../HelixTest/helixtest/crates/common/src/spec_source.rs");
    assert!(!spec_src.contains("reqwest"));
    assert!(!spec_src.contains("HttpClient"));
    let pack_src = include_str!("../src/standards/pack.rs");
    assert!(!pack_src.contains("reqwest"));
}

#[test]
fn test11_wes_remains_not_executable() {
    let err = load_pack(&shipped_drs("ga4gh.wes.1.1.0"), &helix_repo_root()).unwrap_err();
    match err {
        helix::standards::PackLoadError::NotExecutable { pack_id, reason } => {
            assert_eq!(pack_id, "ga4gh.wes.1.1.0");
            assert!(reason.to_lowercase().contains("wes") || reason.contains("HTTPS"));
        }
        other => panic!("{other}"),
    }
}

#[test]
fn test12_and_14_supported_metadata_does_not_set_verified_or_verified_claim() {
    let set = evaluate(&{
        // Constructed later via async test below; this unit checks claim engine.
        let mut sel = helix::model::StandardSelection::unversioned();
        sel.mode = "explicit".into();
        sel.selection_status = helix::standards::SELECTED.into();
        sel.selected_version = Some("1.4.0".into());
        sel.verified_version = None;
        sel.integrity_validated = true;
        sel.integrity_ok = Some(true);
        sel.pack_integrity_sha256 = Some("a".repeat(64));
        sel.schema_document_sha256 = Some("b".repeat(64));
        sel.schema_component_sha256 = Some("c".repeat(64));
        let mut run =
            helix::model::VerificationRun::new(helix::model::Target::new("http://127.0.0.1:9"));
        run.standard_selection = Some(sel);
        run.push_executed(helix::model::VerificationResult::from_check(
            helix::model::VerificationCheck::from_spec(helix::identity::spec("drs.object.schema")),
            helix::model::VerificationStatus::Pass,
        ));
        helix::traceability::bind_run(&mut run).unwrap();
        run.executed[0].selected_version = Some("1.4.0".into());
        run.executed[0].verified_version = None;
        run
    });
    assert!(!set.any_verified());
    assert_eq!(
        set.get(helix::claims::ClaimKind::Ga4ghRequirement).status,
        ClaimStatus::NotVerified
    );
}

#[test]
fn test6_bundled_path_still_exists_for_unversioned() {
    assert!(include_str!("../src/adapter/mod.rs").contains("run_drs_checks("));
}

mod support;
use common::config::{AuthChecksConfig, ServiceConfig, SubsetConfig, TestConfig};
use common::ga4gh_schemas::validate_drs_object;
use common::http::HttpClient;
use common::report::TestStatus;
use common::spec_source::bundled_drs_validate_calls;
use framework::drs::{
    reset_with_spec_calls, run_drs_checks_with_spec, set_lie_spec_document_hash, with_spec_calls,
};
use framework::{Features, Mode};
use helix::model::VerificationStatus;
use helix::verify::{verify, verify_with_options, VerifyOutcome};
use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

/// Serializes versioned adapter tests that share process-wide HelixTest hooks.
static VERSIONED_JOIN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn schema_row(outcome: &VerifyOutcome) -> &helix::model::VerificationResult {
    outcome
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema")
        .expect("drs.object.schema")
}

fn executed_messages(outcome: &VerifyOutcome) -> String {
    outcome
        .run
        .executed
        .iter()
        .filter_map(|r| r.message.as_deref())
        .collect::<Vec<_>>()
        .join("\n")
}

fn drs_test_cfg(drs_url: &str) -> TestConfig {
    TestConfig {
        services: ServiceConfig {
            wes_url: String::new(),
            tes_url: String::new(),
            drs_url: drs_url.to_string(),
            trs_url: String::new(),
            beacon_url: String::new(),
            auth_url: String::new(),
            htsget_url: None,
        },
        subset: SubsetConfig::default(),
        auth_checks: AuthChecksConfig::default(),
    }
}

#[tokio::test]
async fn test13_unversioned_verify_does_not_select_or_hash_pack() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify(&mock.drs_url()).await.expect("unversioned");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.mode, "unversioned");
    assert!(sel.selected_version.is_none());
    assert!(sel.verified_version.is_none());
    assert!(!sel.integrity_validated);
    assert!(sel.integrity_ok.is_none());
    assert!(sel.pack_integrity_sha256.is_none());
}

#[tokio::test]
async fn test14_supported_drs_140_is_not_automatically_verified() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        explicit_drs_140(fixture_supported_drs_140(), None),
    )
    .await
    .expect("join execute");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    assert_eq!(schema_row(&outcome).status, VerificationStatus::Pass);
    let openapi = outcome
        .run
        .executed
        .iter()
        .find(|r| r.id == "drs.object.schema.openapi")
        .expect("normative OpenAPI check");
    assert_eq!(openapi.status, VerificationStatus::Pass);
    assert_eq!(
        openapi.traceability.as_ref().unwrap().category,
        helix::standards::BindingKind::Normative
    );
    let claims = evaluate(&outcome.run);
    assert!(!claims.any_verified());
}

/// F2 mutation: vendor pack bytes that change the schema must change the checker
/// verdict on the real `verify_with_options` → `execute_selected_pack` path.
///
/// Wrong implementation: hashes/`SpecSource` updated but checker still uses bundled
/// OpenAPI. That previously passed test3 (`validate_with_spec` only). Now the
/// mock payload is accepted by the honest pack (test14) and rejected by the
/// mutated pack; `drs.object.schema` must be Fail.
#[tokio::test]
async fn test_versioned_mutated_schema_changes_checker_behavior() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    copy_pack_tree(&helix_repo_root(), "ga4gh.drs.1.4.0", tmp.path()).unwrap();
    let target = tmp
        .path()
        .join("standards/vendor/ga4gh.drs.1.4.0")
        .join(DRS_OBJECT_YAML);
    let yaml = inject_required_field(&std::fs::read_to_string(&target).unwrap());
    std::fs::write(&target, yaml).unwrap();

    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        explicit_drs_140(
            fixture_supported_drs_140_at(tmp.path()),
            Some(tmp.path().to_path_buf()),
        ),
    )
    .await
    .expect("mutated pack must load after rehash");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    assert_eq!(
        schema_row(&outcome).status,
        VerificationStatus::Fail,
        "mutated required field must fail the real versioned checker; Pass means bundled schema was used: {:?}",
        schema_row(&outcome).message
    );
    let msg = schema_row(&outcome).message.clone().unwrap_or_default();
    assert!(
        msg.contains("deliberately_injected_field") || msg.contains("required"),
        "{msg}"
    );
}

/// F2 corrupt: a declared vendor byte change must fail at the versioned verify
/// boundary (`execute_selected_pack` → `load_pack`) and never enter HelixTest.
///
/// Wrong implementation: `load_pack` errors but HelixTest was already invoked,
/// or the test only called `load_pack` directly (test1). Counter must stay put.
#[tokio::test]
async fn test_versioned_corrupt_pack_never_reaches_checker() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    let before = with_spec_calls();

    let tmp = tempfile::tempdir().unwrap();
    copy_pack_tree(&helix_repo_root(), "ga4gh.drs.1.4.0", tmp.path()).unwrap();
    let target = tmp
        .path()
        .join("standards/vendor/ga4gh.drs.1.4.0")
        .join(DRS_OBJECT_YAML);
    let mut bytes = std::fs::read(&target).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&target, bytes).unwrap();

    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        explicit_drs_140(fixture_supported_drs_140(), Some(tmp.path().to_path_buf())),
    )
    .await
    .expect("integrity failure is a recorded outcome, not a panic");
    assert_eq!(
        with_spec_calls(),
        before,
        "corrupt pack must not call run_drs_checks_with_spec"
    );
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.integrity_ok, Some(false));
    assert!(sel.integrity_validated);
    assert!(sel.execution_id.is_none());
    assert!(sel.schema_document_sha256.is_none());
    assert!(sel.verified_version.is_none());
    let msgs = executed_messages(&outcome);
    assert!(
        msgs.contains("HelixTest was not invoked"),
        "expected pack-load fail-closed message, got: {msgs}"
    );
}

/// F2 identity: a lying checker/spec identity on the real adapter join must
/// discard results (`execution_id == None`). Not a unit test of
/// `compare_spec_identity` alone (test5).
///
/// Wrong implementation: ignore mismatch and call `join_from_loaded`. That
/// previously passed test5 (function + source grep). Now `execution_id` would
/// be Some.
#[tokio::test]
async fn test_versioned_lied_identity_fails_closed() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    reset_with_spec_calls();
    set_lie_spec_document_hash(true);
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        explicit_drs_140(fixture_supported_drs_140(), None),
    )
    .await
    .expect("mismatch is fail-closed, not a panic");
    let sel = outcome.run.standard_selection.as_ref().unwrap();
    assert_eq!(sel.selected_version.as_deref(), Some("1.4.0"));
    assert!(sel.verified_version.is_none());
    assert!(
        sel.execution_id.is_none(),
        "identity mismatch must not produce a successful join"
    );
    assert!(sel.schema_document_sha256.is_none());
    assert!(sel.schema_component_sha256.is_none());
    assert_eq!(schema_row(&outcome).status, VerificationStatus::Error);
    let msgs = executed_messages(&outcome);
    assert!(
        msgs.contains("identity mismatch"),
        "expected identity-mismatch discard, got: {msgs}"
    );
}

/// F1 lock at `run_drs_checks_with_spec` with real vendor 1.4.0 bytes.
/// Bundled OpenAPI accepts the mock payload; mutated SpecSource rejects it.
///
/// Wrong implementation: `compile_identity(spec); validate_drs_object(...)`.
/// Hashes would still come from SpecSource; schema would Pass. This test
/// requires the schema check to Fail.
#[tokio::test]
async fn test_with_spec_cannot_fallback_to_bundled_schema() {
    let _guard = VERSIONED_JOIN_LOCK.lock().await;
    let loaded = load_pack(&shipped_drs("ga4gh.drs.1.4.0"), &helix_repo_root()).unwrap();
    let mut spec = loaded.spec.clone();
    let yaml = String::from_utf8(spec.files.get(DRS_OBJECT_YAML).unwrap().to_vec()).unwrap();
    spec.files.insert(
        DRS_OBJECT_YAML.into(),
        Arc::from(inject_required_field(&yaml).into_bytes()),
    );

    let mock = start_mock_ga4gh_drs().await;
    let client = HttpClient::new();
    let cfg = drs_test_cfg(&mock.drs_url());
    let features = Features {
        strict_drs_checksums: true,
        ..Features::default()
    };
    let url = format!("{}/objects/test-object-1", mock.drs_url());
    let payload = client.get_json(&url).await.expect("mock DrsObject");
    validate_drs_object(&payload).expect("bundled schema accepts mock payload");

    reset_schema_call_counters();
    let bundled_before = bundled_drs_validate_calls();
    let (report, _compile) =
        run_drs_checks_with_spec(Mode::Generic, &features, &cfg, &client, &spec)
            .await
            .expect("with_spec");
    assert_eq!(
        bundled_drs_validate_calls(),
        bundled_before,
        "run_drs_checks_with_spec must not call bundled validate_drs_object"
    );
    let schema = report
        .tests
        .iter()
        .find(|t| t.name == "DRS DrsObject OpenAPI + access_methods")
        .expect("schema check");
    assert_eq!(
        schema.status,
        TestStatus::Fail,
        "mutated vendor SpecSource must reject; Pass means fallback to bundled OpenAPI: {:?}",
        schema.error
    );
    let err = schema.error.as_deref().unwrap_or("");
    assert!(
        err.contains("deliberately_injected_field") || err.contains("required"),
        "{err}"
    );
}
