// SPDX-License-Identifier: Apache-2.0
//! Standards registry validation and lookup. Does not run `helix verify`.
//! Not HELIOS. Not certification.

use helix::standards::{
    confined_vendor_file, default_registry_path, hex_sha256, load_path, validate_loaded,
    validate_path, validate_yaml, BindingKind, LocatorType, Lookup, SupportStatus, TestBinding,
    ValidationKind, VersionCitation,
};
use jsonschema::JSONSchema;
use serde_json::Value;

fn commit() -> &'static str {
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}

fn base_record(standard: &str, version: &str, pack_id: &str) -> String {
    let c = commit();
    format!(
        r#"
  - schema_version: helix-standard-version-v1
    pack_id: {pack_id}
    standard: {standard}
    product: Test Product
    version: "{version}"
    release_class: official
    support_status: available
    repository: https://github.com/ga4gh/data-repository-service-schemas
    release_ref: "{version}"
    commit: "{c}"
    retrieved_at: "2026-09-04"
    normative_sources:
      - path: openapi.yaml
        source_url: https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/{c}/openapi.yaml
        role: openapi
        integrity:
          algorithm: sha256
          hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
"#
    )
}

fn wrap(records: &str) -> String {
    format!("schema_version: helix-standards-registry-v1\nversions:\n{records}")
}

#[test]
fn shipped_registry_is_valid() {
    let path = default_registry_path();
    let reg = validate_path(&path).expect("shipped registry must validate");
    assert_eq!(reg.schema_version, "helix-standards-registry-v1");
    let supported = reg.official_supported();
    assert_eq!(supported.len(), 1);
    assert_eq!(supported[0].pack_id, "ga4gh.drs.1.4.0");
    assert_eq!(supported[0].support_status, SupportStatus::Supported);
    assert!(reg.versions.iter().all(|v| v.commit.len() == 40));
    assert!(reg
        .versions
        .iter()
        .all(|v| v.repository.starts_with("https://github.com/ga4gh/")));
    assert!(reg
        .versions
        .iter()
        .any(|v| v.pack_id == "ga4gh.drs.1.5.0" && v.support_status == SupportStatus::Available));
    assert!(reg
        .versions
        .iter()
        .any(|v| v.pack_id == "ga4gh.wes.1.1.0" && v.support_status == SupportStatus::Available));
}

#[test]
fn shipped_records_match_json_schema() {
    let path = default_registry_path();
    let text = std::fs::read_to_string(&path).unwrap();
    let doc: Value = serde_yaml::from_str(&text).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/helix-standard-version-v1.json")).unwrap();
    let leaked: &'static Value = Box::leak(Box::new(schema));
    let compiled = JSONSchema::compile(leaked).unwrap();
    for rec in doc["versions"].as_array().unwrap() {
        if let Err(errors) = compiled.validate(rec) {
            let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            panic!("schema rejected {msgs:?}");
        }
    }
}

#[test]
fn valid_minimal_registry() {
    let yaml = wrap(&base_record("drs", "1.4.0", "ga4gh.drs.1.4.0"));
    validate_yaml(&yaml, None).expect("valid registry");
}

#[test]
fn invalid_registry_not_yaml() {
    let err = validate_yaml("::::", None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
}

#[test]
fn missing_commit() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0").replace(
        "    commit: \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
        "",
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::MissingCommit);
    assert!(err.message.contains("commit"));
}

#[test]
fn unsupported_status_is_available_not_official_supported() {
    let yaml = wrap(&format!(
        "{}{}",
        base_record("drs", "1.5.0", "ga4gh.drs.1.5.0"),
        base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
    ));
    let reg = validate_yaml(&yaml, None).unwrap();
    assert!(reg.official_supported().is_empty());
    match reg.lookup("drs", "1.5.0", None) {
        Lookup::Found(v) => {
            assert_eq!(v.version, "1.5.0");
            assert!(!v.is_supported());
            assert_eq!(v.pack_id, "ga4gh.drs.1.5.0");
        }
        other => panic!("must not substitute: {other:?}"),
    }
}

#[test]
fn duplicate_version() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0");
    let err = validate_yaml(&wrap(&format!("{rec}{rec}")), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::DuplicateVersion);
}

#[test]
fn unknown_release_class() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
        .replace("release_class: official", "release_class: experimental");
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::UnknownReleaseClass);
}

#[test]
fn invalid_source_ferrum_repo() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0").replace(
        "https://github.com/ga4gh/data-repository-service-schemas",
        "https://github.com/SynapticFour/Ferrum",
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidSource);
}

#[test]
fn mismatched_integrity_hash() {
    let root = std::env::temp_dir().join(format!(
        "helix-std-hash-{}-{}",
        std::process::id(),
        "mismatch"
    ));
    let standards = root.join("standards");
    let vendor = standards.join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    let file = vendor.join("spec.yaml");
    std::fs::write(&file, b"not-the-pinned-bytes\n").unwrap();
    let actual = hex_sha256(b"not-the-pinned-bytes\n");
    assert_ne!(
        actual,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    let c = commit();
    let yaml = format!(
        r#"
schema_version: helix-standards-registry-v1
versions:
  - schema_version: helix-standard-version-v1
    pack_id: ga4gh.drs.1.4.0
    standard: drs
    product: Test Product
    version: "1.4.0"
    release_class: official
    support_status: available
    repository: https://github.com/ga4gh/data-repository-service-schemas
    release_ref: "1.4.0"
    commit: "{c}"
    retrieved_at: "2026-09-04"
    normative_sources:
      - path: openapi.yaml
        source_url: https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/{c}/openapi.yaml
        role: openapi
        vendor_path: standards/vendor/spec.yaml
        integrity:
          algorithm: sha256
          hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
"#
    );
    let path = standards.join("registry.yaml");
    std::fs::write(&path, yaml).unwrap();
    let err = validate_path(&path).unwrap_err();
    assert_eq!(err.kind, ValidationKind::IntegrityMismatch);
    assert!(err.message.contains("hash mismatch"));
}

#[test]
fn lookup_unknown_does_not_substitute() {
    let path = default_registry_path();
    let reg = load_path(&path).unwrap();
    match reg.lookup("drs", "1.3.0", None) {
        Lookup::Unknown {
            version, others, ..
        } => {
            assert_eq!(version, "1.3.0");
            assert!(others.iter().any(|s| s.contains("1.4.0")));
            assert!(others.iter().any(|s| s.contains("1.5.0")));
        }
        Lookup::Found(v) => panic!("substituted {}", v.pack_id),
        Lookup::Ambiguous { .. } => panic!("ambiguous"),
    }
}

#[test]
fn shipped_drs_1_5_0_is_available_not_supported() {
    let reg = load_path(&default_registry_path()).unwrap();
    match reg.lookup("drs", "1.5.0", None) {
        Lookup::Found(v) => {
            assert_eq!(v.release_ref, "drs-1.5.0");
            assert!(!v.is_supported());
            assert!(!v.in_default_discovery());
            assert_eq!(v.commit, "fe25c3953ae3398a31054d3f9f040d5e27aad517");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn verify_default_path_is_unversioned() {
    let src = include_str!("../src/verify.rs");
    assert!(
        src.contains("VerifySelection::Unversioned"),
        "default helix verify must remain the unversioned wrap"
    );
    assert!(
        src.contains("AVAILABLE_BUT_NOT_SUPPORTED") || src.contains("select_explicit"),
        "versioned verify must fail closed through the registry selector"
    );
}

#[test]
fn normative_binding_source_file_must_be_in_the_pin() {
    let mut reg = load_path(&default_registry_path()).unwrap();
    for v in &mut reg.versions {
        if v.pack_id == "ga4gh.drs.1.4.0" {
            v.support_status = SupportStatus::Supported;
            v.fixture_catalog = Some("helix-fixtures-v1".into());
            v.test_bindings = Some(vec![TestBinding {
                id: "drs.object.schema".into(),
                code: "HLX-DRS-002".into(),
                kind: BindingKind::Normative,
                citation: Some(VersionCitation {
                    source_path: "openapi/missing.yaml".into(),
                    locator_type: LocatorType::SchemaName,
                    locator: "DrsObject".into(),
                    excerpt: None,
                }),
            }]);
        }
    }
    let err =
        validate_loaded(&reg, Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("not in normative_sources"),
        "{}",
        err.message
    );
}

#[test]
fn vendor_path_cannot_escape_the_repo() {
    let root = std::path::Path::new("/tmp/helix-registry-root");
    assert!(confined_vendor_file(root, "standards/vendor/spec.yaml").is_ok());
    let err = confined_vendor_file(root, "../etc/passwd").unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidSource);
    assert!(err.message.contains("vendor_path"), "{}", err.message);
    let abs = confined_vendor_file(root, "/etc/passwd").unwrap_err();
    assert_eq!(abs.kind, ValidationKind::InvalidSource);
}

fn supported_bindings() -> String {
    r#"
    fixture_catalog: helix-fixtures-v1
    test_bindings:
      - id: drs.object.schema
        code: HLX-DRS-002
        kind: fixture
"#
    .into()
}

fn with_vendor_path(rec: &str) -> String {
    rec.replace(
        "        role: openapi\n",
        "        role: openapi\n        vendor_path: standards/vendor/ga4gh.drs.1.4.0/data_repository_service.openapi.yaml\n",
    )
}

#[test]
fn supported_requires_bindings_and_fixture_catalog() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
        .replace("support_status: available", "support_status: supported");
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("test_bindings")
            || err.message.contains("fixture_catalog")
            || err.message.contains("catalog_id")
            || err.message.contains("binding_id")
            || err.message.contains("coverage")
            || err.message.contains("support gate"),
        "{}",
        err.message
    );
}

#[test]
fn development_cannot_be_supported() {
    let rec = with_vendor_path(
        &base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
            .replace("release_class: official", "release_class: development")
            .replace("support_status: available", "support_status: supported"),
    ) + &supported_bindings();
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.to_lowercase().contains("development") || err.message.contains("supported"),
        "{}",
        err.message
    );
}

#[test]
fn official_release_ref_cannot_be_head() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
        .replace("release_ref: \"1.4.0\"", "release_ref: HEAD");
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("HEAD")
            || err.message.contains("immutable")
            || err.message.contains("release_ref"),
        "{}",
        err.message
    );
}

#[test]
fn source_url_cannot_be_branch_head() {
    let c = commit();
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0").replace(
        &format!(
            "https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/{c}/openapi.yaml"
        ),
        "https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/HEAD/openapi.yaml",
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidSource);
    assert!(
        err.message.contains("HEAD")
            || err.message.contains("commit")
            || err.message.contains("source_url"),
        "{}",
        err.message
    );
}

#[test]
fn supported_requires_vendor_path() {
    let rec = format!(
        "{}{}",
        base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
            .replace("support_status: available", "support_status: supported"),
        supported_bindings()
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("vendor_path")
            || err.message.contains("schema")
            || err.message.contains("catalog_id")
            || err.message.contains("support gate")
            || err.message.contains("supported"),
        "{}",
        err.message
    );
}

#[test]
fn source_url_cannot_use_main_branch() {
    let c = commit();
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0").replace(
        &format!(
            "https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/{c}/openapi.yaml"
        ),
        "https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/main/openapi.yaml",
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidSource);
    assert!(
        err.message.contains("HEAD")
            || err.message.contains("main")
            || err.message.contains("commit")
            || err.message.contains("source_url"),
        "{}",
        err.message
    );
}

#[test]
fn source_url_must_contain_the_pinned_commit() {
    let c = commit();
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0").replace(
        &format!(
            "https://raw.githubusercontent.com/ga4gh/data-repository-service-schemas/{c}/openapi.yaml"
        ),
        "https://ga4gh.github.io/data-repository-service-schemas/preview/release/drs-1.4.0/openapi.yaml",
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidSource);
    assert!(
        err.message.contains("commit") || err.message.contains("source_url"),
        "{}",
        err.message
    );
}

#[test]
fn official_release_ref_cannot_be_main() {
    let rec = base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
        .replace("release_ref: \"1.4.0\"", "release_ref: main");
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("main")
            || err.message.contains("immutable")
            || err.message.contains("release_ref"),
        "{}",
        err.message
    );
}

#[test]
fn helios_field_on_registry_record_is_rejected() {
    let rec = format!(
        "{}    ro_crate: {{}}\n",
        base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
    );
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
}

#[test]
fn normative_binding_requires_citation() {
    let rec = with_vendor_path(
        &base_record("drs", "1.4.0", "ga4gh.drs.1.4.0")
            .replace("support_status: available", "support_status: supported"),
    ) + r#"
    fixture_catalog: helix-fixtures-v1
    test_bindings:
      - id: drs.object.schema
        code: HLX-DRS-002
        kind: normative
"#;
    let err = validate_yaml(&wrap(&rec), None).unwrap_err();
    assert_eq!(err.kind, ValidationKind::InvalidRegistry);
    assert!(
        err.message.contains("citation") || err.message.contains("schema"),
        "{}",
        err.message
    );
}
