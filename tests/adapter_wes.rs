// SPDX-License-Identifier: Apache-2.0
//! Integration: HelixTest adapter against the in-process generic WES fixture.
//! Not Ferrum. Does not rewrite HelixTest.

use helix::adapter::{ConformanceAdapter, HelixTestAdapter, WES_CHECK_NAMES};
use helix::identity::{spec_by_helixtest_name, WES_VERIFY_IDS};
use helix::model::{VerificationStatus, HELIXTEST_PIN, HELIXTEST_SHA};

mod support;

use support::mock_ga4gh_wes::start_mock_ga4gh_wes;

#[tokio::test]
async fn adapter_runs_helixtest_wes_against_in_process_generic_fixture() {
    let mock = start_mock_ga4gh_wes().await;
    let adapter = HelixTestAdapter::pinned();
    let pin = adapter.pin();
    assert_eq!(pin.tag, HELIXTEST_PIN);
    assert_eq!(pin.sha, HELIXTEST_SHA);
    assert_eq!(pin.tag, "v0.1.3");

    let out = adapter
        .run_wes(&mock.wes_url())
        .await
        .expect("adapter run_wes");

    assert_eq!(out.pin.tag, "v0.1.3");
    assert_eq!(out.pin.sha, HELIXTEST_SHA);
    assert_eq!(out.results.len(), 8, "eight HelixTest WES checks");
    assert_eq!(out.service_report.tests.len(), 8);

    for (i, r) in out.results.iter().enumerate() {
        assert_eq!(r.id, WES_VERIFY_IDS[i]);
        assert_eq!(r.helixtest_name.as_deref(), Some(WES_CHECK_NAMES[i]));
        let spec = spec_by_helixtest_name(WES_CHECK_NAMES[i]).expect("mapped");
        assert_eq!(r.id, spec.id);
        assert_eq!(r.code, spec.code);
        assert_eq!(r.service, "wes");
        if r.id == "wes.run.scatter_gather" {
            assert_eq!(r.status, VerificationStatus::Skip);
            assert!(!r.is_pass());
            assert!(
                r.message
                    .as_deref()
                    .is_some_and(|m| m.contains("supports_scatter_gather=false")),
                "scatter skip reason from HelixTest: {:?}",
                r.message
            );
        } else {
            assert_eq!(r.status, VerificationStatus::Pass, "{}", r.id);
        }
    }
}
