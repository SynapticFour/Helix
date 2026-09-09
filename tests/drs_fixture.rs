// SPDX-License-Identifier: Apache-2.0
//! B6: DRS fixture is target-scoped test input, not a GA4GH requirement.
//! Mocks remain mocks. Not VERIFIED. Not HELIOS.

mod support;

use common::util::sha256_bytes;
use helix::fixture::{DrsVerifyFixture, DEFAULT_DRS_OBJECT_ID};
use helix::model::VerificationStatus;
use helix::profile::ProfileId;
use helix::target::{DeclaredTarget, FailureAttribution, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use serde_json::json;
use support::mock_ga4gh_drs::{
    mount_ga4gh_drs_service_info, start_mock_ga4gh_drs, BLOB_LEN, TEST_OBJECT_ID,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

static JOIN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn versioned(fixture: DrsVerifyFixture) -> VerifyOptions {
    VerifyOptions {
        profile: ProfileId::Generic,
        selection: VerifySelection::Explicit {
            standard: "drs".into(),
            version: "1.4.0".into(),
            release_class: None,
        },
        declared_target: DeclaredTarget {
            target_id: Some("fixture-target".into()),
            kind: TargetKind::Mock,
            ..DeclaredTarget::default()
        },
        drs_fixture: fixture,
        ..Default::default()
    }
}

struct BytesWithOptionalRange {
    body: Vec<u8>,
}

impl wiremock::Respond for BytesWithOptionalRange {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let total = self.body.len() as u64;
        let range_hdr = request
            .headers
            .get("range")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if let Some(spec) = range_hdr.strip_prefix("bytes=") {
            let (start_s, end_s) = spec.split_once('-').unwrap_or(("0", ""));
            let start: usize = start_s.parse().unwrap_or(0);
            let end: usize = if end_s.is_empty() {
                self.body.len().saturating_sub(1)
            } else {
                end_s.parse().unwrap_or(self.body.len().saturating_sub(1))
            };
            let end = end.min(self.body.len().saturating_sub(1));
            let start = start.min(end);
            let slice = self.body[start..=end].to_vec();
            return ResponseTemplate::new(206)
                .insert_header("Content-Range", format!("bytes {start}-{end}/{total}"))
                .insert_header("Content-Type", "application/octet-stream")
                .set_body_bytes(slice);
        }
        ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/octet-stream")
            .set_body_bytes(self.body.clone())
    }
}

async fn mock_object(id: &str, blob: Vec<u8>, json_checksum: &str) -> MockServer {
    let server = MockServer::start().await;
    let access_url = format!("{}/bytes/{id}", server.uri());
    let object = json!({
        "id": id,
        "name": id,
        "self_uri": format!("drs://example.invalid/{id}"),
        "size": blob.len(),
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": json_checksum }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": access_url }
        }]
    });
    Mock::given(method("GET"))
        .and(path(format!("/objects/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/bytes/{id}")))
        .respond_with(BytesWithOptionalRange { body: blob })
        .mount(&server)
        .await;
    server
}

async fn mount_valid_object(server: &MockServer, id: &str) {
    let blob = vec![b'A'; BLOB_LEN];
    let sha = sha256_bytes(&blob);
    let access_url = format!("{}/bytes/{id}", server.uri());
    let object = json!({
        "id": id,
        "name": id,
        "self_uri": format!("drs://example.invalid/{id}"),
        "size": blob.len(),
        "created_time": "2020-01-01T00:00:00Z",
        "checksums": [{ "type": "sha256", "checksum": sha }],
        "access_methods": [{
            "type": "https",
            "access_url": { "url": access_url }
        }]
    });
    Mock::given(method("GET"))
        .and(path(format!("/objects/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(object))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/bytes/{id}")))
        .respond_with(BytesWithOptionalRange { body: blob })
        .mount(server)
        .await;
}

fn status_of(run: &helix::model::VerificationRun, id: &str) -> VerificationStatus {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id)
        .map(|r| r.status)
        .expect(id)
}

fn row<'a>(
    run: &'a helix::model::VerificationRun,
    id: &str,
) -> &'a helix::model::VerificationResult {
    run.executed
        .iter()
        .chain(run.skipped.iter())
        .find(|r| r.id == id)
        .expect(id)
}

/// T1 — changing the fixture changes the request object id (not only a constant).
#[tokio::test]
async fn t1_fixture_changes_request_path() {
    let _g = JOIN_LOCK.lock().await;
    let blob = vec![b'A'; BLOB_LEN];
    let sha = sha256_bytes(&blob);
    let server = mock_object("portable-object", blob, &sha).await;
    let fx = DrsVerifyFixture::operator_declared("portable-object".into(), None).unwrap();
    let outcome = verify_with_options(&server.uri(), versioned(fx))
        .await
        .expect("verify");
    assert_eq!(
        outcome.run.drs_fixture.as_ref().unwrap().object_id,
        "portable-object"
    );
    assert_eq!(
        status_of(&outcome.run, "drs.object.schema.openapi"),
        VerificationStatus::Pass
    );
    assert_ne!(
        outcome.run.drs_fixture.as_ref().unwrap().object_id,
        DEFAULT_DRS_OBJECT_ID
    );
}

/// T2/T3 — fixture is target-scoped; spec execution_id unchanged.
#[tokio::test]
async fn t2_t3_fixture_scopes_target_execution_not_spec_join() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    mount_valid_object(&mock.server, "other-object").await;
    let a = verify_with_options(
        &mock.drs_url(),
        versioned(DrsVerifyFixture::default_catalog()),
    )
    .await
    .expect("a");
    let other = DrsVerifyFixture::operator_declared("other-object".into(), None).unwrap();
    let b = verify_with_options(&mock.drs_url(), versioned(other))
        .await
        .expect("b");
    assert!(
        a.run
            .standard_selection
            .as_ref()
            .unwrap()
            .execution_id
            .is_some(),
        "checker must still join the pack"
    );
    assert_eq!(
        status_of(&a.run, "drs.object.schema.openapi"),
        VerificationStatus::Pass
    );
    assert_eq!(
        status_of(&b.run, "drs.object.schema.openapi"),
        VerificationStatus::Pass
    );
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id, "T3 spec-join unchanged");
    assert_ne!(
        sa.target_execution_id, sb.target_execution_id,
        "T2 fixture must change target_execution_id"
    );
}

/// T8 — operator expected SHA256 is fixture identity, not spec-join.
#[tokio::test]
async fn t8_expected_sha256_changes_target_execution_id_not_spec_join() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let none = DrsVerifyFixture::operator_declared(DEFAULT_DRS_OBJECT_ID.into(), None).unwrap();
    let digest =
        DrsVerifyFixture::operator_declared(DEFAULT_DRS_OBJECT_ID.into(), Some("ab".repeat(32)))
            .unwrap();
    let a = verify_with_options(&mock.drs_url(), versioned(none))
        .await
        .expect("a");
    let b = verify_with_options(&mock.drs_url(), versioned(digest))
        .await
        .expect("b");
    let sa = a.run.standard_selection.as_ref().unwrap();
    let sb = b.run.standard_selection.as_ref().unwrap();
    assert_eq!(sa.execution_id, sb.execution_id);
    assert_ne!(sa.target_execution_id, sb.target_execution_id);
    assert_eq!(
        a.run.drs_fixture.as_ref().unwrap().checksum_mode,
        helix::fixture::ChecksumMode::AdvertisedConsistency
    );
    assert_eq!(
        b.run.drs_fixture.as_ref().unwrap().checksum_mode,
        helix::fixture::ChecksumMode::OperatorDigest
    );
}

/// T4 — missing configured object is fixture_unavailable, not spec_failure.
#[tokio::test]
async fn t4_missing_object_is_fixture_unavailable() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    mount_ga4gh_drs_service_info(&mock.server).await;
    let fx = DrsVerifyFixture::operator_declared("does-not-exist-on-mock".into(), None).unwrap();
    let outcome = verify_with_options(&mock.drs_url(), versioned(fx))
        .await
        .expect("verify");
    let schema = row(&outcome.run, "drs.object.schema.openapi");
    assert_eq!(schema.status, VerificationStatus::Skip);
    assert_eq!(
        schema.attribution,
        Some(FailureAttribution::TargetConfigurationFailure)
    );
    let msg = schema.message.as_deref().unwrap_or("");
    assert!(msg.contains(framework::drs::FIXTURE_UNAVAILABLE), "{msg}");
    assert_eq!(
        status_of(&outcome.run, "drs.object.not_found"),
        VerificationStatus::Pass
    );
}

/// T6 — returned object is validated against pinned DRS 1.4.0 SpecSource.
#[tokio::test]
async fn t6_normative_schema_uses_specsource() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        versioned(DrsVerifyFixture::default_catalog()),
    )
    .await
    .expect("verify");
    let schema = row(&outcome.run, "drs.object.schema.openapi");
    assert_eq!(schema.status, VerificationStatus::Pass);
    let t = schema.traceability.as_ref().expect("traceability");
    assert_eq!(t.check_kind.as_str(), "normative");
    assert!(t.implementation.contains("run_drs_checks_with_spec"));
}

/// T7 — unknown id is derived from the positive fixture and is not that fixture.
#[tokio::test]
async fn t7_unknown_id_independent_of_positive_fixture() {
    let a = DrsVerifyFixture::default_catalog();
    let b = DrsVerifyFixture::operator_declared("portable-object".into(), None).unwrap();
    assert_ne!(a.object_id, a.unknown_object_id);
    assert_ne!(b.object_id, b.unknown_object_id);
    assert_ne!(a.unknown_object_id, b.unknown_object_id);
    assert!(!a.unknown_object_id.contains(TEST_OBJECT_ID));
}

/// T8 — fixture digest, not GetObject.checksums, is the expected checksum.
#[tokio::test]
async fn t8_checksum_uses_fixture_digest_not_getobject_json() {
    let _g = JOIN_LOCK.lock().await;
    let blob = vec![b'A'; BLOB_LEN];
    let real = sha256_bytes(&blob);
    let lying = sha256_bytes(&vec![b'B'; BLOB_LEN]);
    assert_ne!(real, lying);
    let server = mock_object("digest-object", blob, &lying).await;
    let fx =
        DrsVerifyFixture::operator_declared("digest-object".into(), Some(real.clone())).unwrap();
    let outcome = verify_with_options(&server.uri(), versioned(fx))
        .await
        .expect("verify");
    assert_eq!(
        status_of(&outcome.run, "drs.object.checksum"),
        VerificationStatus::Pass,
        "lying GetObject.checksums must not become expected when fixture digest is set"
    );
}

/// T9 — range expected semantics are HTTP 206/Content-Range, not a second download compared to itself.
#[test]
fn t9_range_is_protocol_not_tautological_redownload() {
    let drs = include_str!("../../HelixTest/helixtest/crates/framework/src/drs.rs");
    assert!(drs.contains("Range\", \"bytes=0-1023\"") || drs.contains("bytes=0-1023"));
    assert!(drs.contains("Expected 206 Partial Content"));
    assert!(
        !drs.contains("expected = target.download") && !drs.contains("expected_bytes = body"),
        "range must not set expected from the same response body"
    );
}

/// T10 — no implementation-name / starter-kit / localhost branches.
#[test]
fn t10_no_target_specific_branch() {
    let adapter = include_str!("../src/adapter/mod.rs");
    let verify = include_str!("../src/verify.rs");
    let fixture = include_str!("../src/fixture.rs");
    let drs = include_str!("../../HelixTest/helixtest/crates/framework/src/drs.rs");
    for src in [adapter, verify, fixture, drs] {
        let lower = src.to_lowercase();
        assert!(!lower.contains("ga4gh-starter-kit"));
        assert!(!lower.contains("starter_kit"));
        assert!(!src.contains("127.0.0.1:4500"));
        assert!(!src.contains("localhost:4500"));
    }
    assert!(adapter.contains("run_drs_checks_with_spec_and_fixture"));
    assert!(adapter.contains("run_drs_checks_with_fixture"));
}

/// T11 — Ferrum is not a mandatory target; default catalog id remains the Ferrum demo object.
#[test]
fn t11_ferrum_is_optional_default_catalog_id() {
    assert_eq!(DEFAULT_DRS_OBJECT_ID, "test-object-1");
    let makefile = include_str!("../Makefile");
    assert!(makefile.contains("test-live"));
    assert!(
        !makefile.contains("--drs-object-id"),
        "live Ferrum path must keep working with the default catalog id"
    );
}

/// T12 — default catalog still passes the in-process mock.
#[tokio::test]
async fn t12_default_mock_catalog_still_passes() {
    let _g = JOIN_LOCK.lock().await;
    let mock = start_mock_ga4gh_drs().await;
    let outcome = verify_with_options(
        &mock.drs_url(),
        versioned(DrsVerifyFixture::default_catalog()),
    )
    .await
    .expect("verify");
    assert_eq!(
        status_of(&outcome.run, "drs.object.schema.openapi"),
        VerificationStatus::Pass
    );
    assert_eq!(
        status_of(&outcome.run, "drs.object.reachable"),
        VerificationStatus::Pass
    );
    assert_eq!(
        status_of(&outcome.run, "drs.object.not_found"),
        VerificationStatus::Pass
    );
    assert_eq!(
        outcome.run.drs_fixture.as_ref().unwrap().source,
        helix::fixture::FixtureSource::DefaultCatalog
    );
    assert_eq!(
        outcome.run.drs_fixture.as_ref().unwrap().checksum_mode,
        helix::fixture::ChecksumMode::AdvertisedConsistency
    );
}
