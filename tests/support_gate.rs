// SPDX-License-Identifier: Apache-2.0
//! B3 executable support gate. YAML `support_status: supported` is not sufficient.
//! Not VERIFIED. Not HELIOS. Not GA4GH certification.

use helix::identity::spec_by_helixtest_name;
use helix::standards::{
    binding_id, catalog_id, contract_for, default_registry_path, evaluate_support,
    expected_bindings, helix_repo_root, load_path, BindingKind, LocatorType, PackCoverage,
    SupportStatus, TestBinding, DRS_140_CONTRACT, DRS_140_PACK_ID, DRS_OPENAPI_SPECSOURCE_CHECK,
};

fn shipped_140() -> helix::standards::StandardVersion {
    load_path(&default_registry_path())
        .unwrap()
        .versions
        .into_iter()
        .find(|v| v.pack_id == DRS_140_PACK_ID)
        .unwrap()
}

fn root() -> std::path::PathBuf {
    helix_repo_root()
}

#[test]
fn a_supported_registry_entry_with_missing_pack_fails() {
    let v = shipped_140();
    let empty = tempfile::tempdir().unwrap();
    let verdict = evaluate_support(&v, Some(empty.path()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(
        verdict
            .reasons
            .iter()
            .any(|r| r.contains("pack is not executable") || r.contains("vendor")),
        "{:?}",
        verdict.reasons
    );
}

#[test]
fn b_supported_registry_entry_with_corrupted_pack_fails() {
    let v = shipped_140();
    let tmp = tempfile::tempdir().unwrap();
    helix::standards::copy_pack_tree(&root(), DRS_140_PACK_ID, tmp.path()).unwrap();
    let target = tmp
        .path()
        .join("standards/vendor")
        .join(DRS_140_PACK_ID)
        .join("openapi/components/schemas/DrsObject.yaml");
    let mut bytes = std::fs::read(&target).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&target, bytes).unwrap();
    let verdict = evaluate_support(&v, Some(tmp.path()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
}

#[test]
fn c_supported_pack_without_checker_binding_fails() {
    let mut v = shipped_140();
    v.test_bindings = None;
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(
        verdict.reasons.iter().any(|r| r.contains("test_bindings")),
        "{:?}",
        verdict.reasons
    );
}

#[test]
fn d_supported_pack_with_empty_catalog_fails() {
    let mut v = shipped_140();
    v.test_bindings = Some(Vec::new());
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(
        verdict
            .reasons
            .iter()
            .any(|r| r.contains("empty") || r.contains("len")),
        "{:?}",
        verdict.reasons
    );
}

#[test]
fn e_catalog_check_missing_authoritative_provenance_fails() {
    let mut v = shipped_140();
    let mut bindings = expected_bindings(&DRS_140_CONTRACT);
    bindings[0].citation = None;
    v.test_bindings = Some(bindings);
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(
        verdict
            .reasons
            .iter()
            .any(|r| r.contains("citation") || r.contains("provenance")),
        "{:?}",
        verdict.reasons
    );
}

#[test]
fn f_fixture_only_catalog_cannot_satisfy_normative_support() {
    let mut v = shipped_140();
    v.test_bindings = Some(vec![TestBinding {
        id: "drs.object.schema".into(),
        code: "HLX-DRS-002".into(),
        kind: BindingKind::Fixture,
        citation: None,
    }]);
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
}

#[test]
fn g_binding_pointing_to_nonexistent_implementation_fails() {
    assert!(spec_by_helixtest_name(DRS_OPENAPI_SPECSOURCE_CHECK).is_some());
    assert!(spec_by_helixtest_name("this HelixTest function does not exist").is_none());
    let spec = spec_by_helixtest_name(DRS_OPENAPI_SPECSOURCE_CHECK).unwrap();
    assert_eq!(spec.id, "drs.object.schema.openapi");
}

#[test]
fn h_binding_modified_without_binding_identity_change_fails() {
    let mut v = shipped_140();
    let original = v.binding_id.clone();
    v.catalog_id = Some("ff".repeat(32));
    assert_eq!(v.binding_id, original);
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(
        verdict
            .reasons
            .iter()
            .any(|r| r.contains("catalog_id") || r.contains("binding_id")),
        "{:?}",
        verdict.reasons
    );

    let contract = contract_for(DRS_140_PACK_ID).unwrap();
    let a = binding_id(contract, &"a".repeat(64), &"b".repeat(64), &"c".repeat(64));
    let b = binding_id(contract, &"d".repeat(64), &"b".repeat(64), &"c".repeat(64));
    assert_ne!(a, b, "pack hash change must change binding_id");
}

#[test]
fn i_drs_1_5_0_remains_unsupported() {
    let mut reg = load_path(&default_registry_path()).unwrap();
    let v = reg
        .versions
        .iter_mut()
        .find(|v| v.pack_id == "ga4gh.drs.1.5.0")
        .unwrap();
    assert_eq!(v.support_status, SupportStatus::Available);
    let available = evaluate_support(v, Some(&root()));
    assert!(!available.supported);
    v.support_status = SupportStatus::Supported;
    v.fixture_catalog = Some("helix-fixtures-v1".into());
    v.test_bindings = Some(expected_bindings(&DRS_140_CONTRACT));
    v.coverage = Some(PackCoverage {
        schema: helix::standards::CoverageLevel::Partial,
        behavior: helix::standards::CoverageLevel::None,
        security: helix::standards::CoverageLevel::None,
        interoperability: helix::standards::CoverageLevel::Partial,
        notes: None,
    });
    v.catalog_id = Some(catalog_id(&DRS_140_CONTRACT));
    v.binding_id = Some("ab".repeat(32));
    let verdict = evaluate_support(v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
    assert!(verdict
        .reasons
        .iter()
        .any(|r| r.contains("no compiled support contract")));
}

#[test]
fn j_wes_remains_non_executable() {
    let v = load_path(&default_registry_path())
        .unwrap()
        .versions
        .into_iter()
        .find(|v| v.pack_id == "ga4gh.wes.1.1.0")
        .unwrap();
    assert_eq!(v.support_status, SupportStatus::Available);
    assert!(contract_for(&v.pack_id).is_none());
    let mut lying = v.clone();
    lying.support_status = SupportStatus::Supported;
    lying.fixture_catalog = Some("helix-fixtures-v1".into());
    lying.test_bindings = Some(expected_bindings(&DRS_140_CONTRACT));
    assert!(!helix::standards::yaml_supported_is_executable(&lying));
    let verdict = evaluate_support(&lying, Some(&root()));
    assert!(!verdict.supported);
    assert!(verdict
        .reasons
        .iter()
        .any(|r| r.contains("no compiled support contract")));
}

#[test]
fn positive_shipped_drs_140_passes_the_support_gate() {
    let v = shipped_140();
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(
        verdict.supported,
        "DRS 1.4.0 support gate failed: {:?}",
        verdict.reasons
    );
    let loaded = helix::standards::load_pack(&v, &root()).unwrap();
    assert_eq!(
        loaded.pack_integrity_sha256,
        "c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89"
    );
    assert_eq!(
        loaded.expected.schema_document_sha256,
        "3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608"
    );
    assert_eq!(
        loaded.expected.schema_component_sha256,
        "b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003"
    );
    assert_eq!(verdict.catalog_id, catalog_id(&DRS_140_CONTRACT));
    assert_eq!(verdict.binding_id.as_deref(), v.binding_id.as_deref());
    assert!(!verdict.catalog_id.is_empty());
    assert_eq!(verdict.binding_id.as_ref().unwrap().len(), 64);
}

#[test]
fn registry_cannot_lie_by_flipping_only_support_status() {
    let mut v = shipped_140();
    v.support_status = SupportStatus::Supported;
    v.test_bindings = None;
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
}

#[test]
fn catalog_cannot_lie_when_binding_implementation_is_absent() {
    let mut v = shipped_140();
    let mut bindings = expected_bindings(&DRS_140_CONTRACT);
    bindings[0].id = "drs.object.does_not_exist".into();
    v.test_bindings = Some(bindings);
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported, "{:?}", verdict.reasons);
}

#[test]
fn normative_entries_must_have_pinned_drs_140_provenance() {
    let contract = contract_for(DRS_140_PACK_ID).unwrap();
    let normative: Vec<_> = contract
        .checks
        .iter()
        .filter(|c| c.kind == BindingKind::Normative)
        .collect();
    assert!(!normative.is_empty());
    for c in normative {
        assert_eq!(c.source_path, "openapi/components/schemas/DrsObject.yaml");
        assert_eq!(c.locator, "DrsObject");
        assert_eq!(c.locator_type, LocatorType::SchemaName);
        assert!(!c.source_path.is_empty());
        assert!(!c.locator.is_empty());
        let spec = helix::identity::spec_by_id(c.id).expect("identity");
        assert!(spec.helixtest_names.contains(&c.helixtest_name));
        assert_eq!(
            c.kind.claim_scope(),
            helix::standards::ClaimScope::Ga4ghRequirement
        );
    }
}

#[test]
fn yaml_supported_without_executable_predicates_is_not_supported() {
    let mut v = shipped_140();
    assert_eq!(v.support_status, SupportStatus::Supported);
    v.coverage = None;
    let verdict = evaluate_support(&v, Some(&root()));
    assert!(!verdict.supported);
}

#[test]
fn binding_id_changes_when_catalog_id_changes() {
    let contract = &DRS_140_CONTRACT;
    let cid = catalog_id(contract);
    let bid = binding_id(contract, &"a".repeat(64), &"b".repeat(64), &"c".repeat(64));
    assert_eq!(cid.len(), 64);
    assert_eq!(bid.len(), 64);
    assert_ne!(cid, bid);
}
