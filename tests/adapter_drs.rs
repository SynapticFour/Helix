// SPDX-License-Identifier: Apache-2.0
//! Integration: HelixTest adapter against the in-process generic DRS fixture.
//! Same B1 mock as HelixTest CI. Not Ferrum. Does not rewrite HelixTest.

use helix::adapter::{ConformanceAdapter, HelixTestAdapter, DRS_CHECK_NAMES};
use helix::identity::spec_by_helixtest_name;
use helix::model::{VerificationStatus, HELIXTEST_PIN, HELIXTEST_SHA};

mod support;

use support::mock_ga4gh_drs::start_mock_ga4gh_drs;

#[tokio::test]
async fn adapter_runs_helixtest_drs_against_in_process_generic_fixture() {
    let mock = start_mock_ga4gh_drs().await;
    let adapter = HelixTestAdapter::pinned();
    let pin = adapter.pin();
    assert_eq!(pin.tag, HELIXTEST_PIN);
    assert_eq!(pin.sha, HELIXTEST_SHA);
    assert_eq!(pin.tag, "v0.1.3");

    let out = adapter
        .run_drs(&mock.drs_url())
        .await
        .expect("adapter run_drs");

    assert_eq!(out.pin.tag, "v0.1.3");
    assert_eq!(out.pin.sha, HELIXTEST_SHA);
    assert_eq!(out.results.len(), 5, "five HelixTest DRS checks");
    assert_eq!(out.service_report.tests.len(), 5);

    let ht_names: Vec<&str> = out
        .service_report
        .tests
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(ht_names, DRS_CHECK_NAMES);

    let expected_ids = [
        "drs.object.reachable",
        "drs.object.schema",
        "drs.object.checksum",
        "drs.object.range",
        "drs.object.not_found",
    ];
    let expected_codes = [
        "HLX-DRS-001",
        "HLX-DRS-002",
        "HLX-DRS-003",
        "HLX-DRS-004",
        "HLX-DRS-005",
    ];

    for (i, r) in out.results.iter().enumerate() {
        assert_eq!(r.id, expected_ids[i], "{}", r.id);
        assert_eq!(r.code, expected_codes[i]);
        assert_eq!(r.status, VerificationStatus::Pass, "{}", r.id);
        assert!(r.is_pass());
        assert_ne!(r.status, VerificationStatus::Skip);
        let ht_name = r
            .helixtest_name
            .as_deref()
            .expect("original HelixTest name");
        assert_eq!(ht_name, DRS_CHECK_NAMES[i]);
        let spec = spec_by_helixtest_name(ht_name).expect("catalog wrap");
        assert_eq!(spec.id, r.id);
        assert_eq!(spec.code, r.code);
        assert_eq!(r.name, spec.name);
        assert!(r.failure.is_none());
    }

    for t in &out.service_report.tests {
        assert_eq!(
            t.status,
            common::report::TestStatus::Pass,
            "fixture must pass; skip is never pass: {}",
            t.name
        );
    }
}
