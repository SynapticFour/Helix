// SPDX-License-Identifier: Apache-2.0
//! SCHEMA PASS must not imply BEHAVIOR PASS. Known-bad fixtures fail
//! for the recorded reason. Not certification.

use assert_cmd::Command;
use serde_json::Value;

mod support;

use support::mock_ga4gh_drs::{
    start_mock_schema_ok_checksum_wrong, start_mock_schema_ok_unknown_id_200,
};

fn helix() -> Command {
    Command::cargo_bin("helix").unwrap()
}

fn verify_json(url: &str) -> Value {
    let out = helix()
        .env("RUST_LOG", "error")
        .args(["verify", url, "--format", "json"])
        .output()
        .expect("helix verify");
    serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        );
    })
}

fn row<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["executed"]
        .as_array()
        .unwrap()
        .iter()
        .chain(v["skipped"].as_array().unwrap())
        .find(|r| r["id"] == id)
        .unwrap_or_else(|| panic!("missing {id} in {v}"))
}

#[tokio::test]
async fn schema_pass_checksum_fail_is_not_behavior_pass() {
    let mock = start_mock_schema_ok_checksum_wrong().await;
    let v = verify_json(&mock.drs_url());
    assert_eq!(row(&v, "drs.object.schema")["status"], "pass");
    assert_eq!(row(&v, "drs.object.schema")["layer"], "schema");
    assert_eq!(row(&v, "drs.object.checksum")["status"], "fail");
    assert_eq!(row(&v, "drs.object.checksum")["layer"], "behavior");
    assert_eq!(v["layer_summary"]["schema"]["passed"], 1);
    assert!(v["layer_summary"]["behavior"]["failed"].as_u64().unwrap() >= 1);
    assert!(v["layer_summary"]["note"]
        .as_str()
        .unwrap()
        .contains("SCHEMA PASS is not BEHAVIOR PASS"));
    assert!(v["layer_summary"].get("percent").is_none());
    assert!(v["layer_summary"].get("compliant").is_none());
    assert!(v.get("score").is_none());
    let msg = row(&v, "drs.object.checksum")["message"]
        .as_str()
        .unwrap_or("");
    assert!(
        msg.contains("checksum") || msg.contains("mismatch"),
        "known-bad must fail for checksum, got {msg}"
    );
}

#[tokio::test]
async fn schema_pass_unknown_id_200_fails_behavior_404() {
    let mock = start_mock_schema_ok_unknown_id_200().await;
    let v = verify_json(&mock.drs_url());
    assert_eq!(row(&v, "drs.object.schema")["status"], "pass");
    assert_eq!(row(&v, "drs.object.not_found")["status"], "fail");
    assert_eq!(row(&v, "drs.object.not_found")["layer"], "behavior");
    assert_eq!(v["layer_summary"]["schema"]["passed"], 1);
    assert!(v["layer_summary"]["behavior"]["failed"].as_u64().unwrap() >= 1);
    let observed = row(&v, "drs.object.not_found")
        .get("observed_response")
        .and_then(|x| x.as_str())
        .or_else(|| row(&v, "drs.object.not_found")["diagnostic"]["observed"].as_str())
        .unwrap_or("");
    assert!(
        observed.contains("200") || observed.contains("success"),
        "404 probe must record HTTP 200 as observed, got {observed}"
    );
}
