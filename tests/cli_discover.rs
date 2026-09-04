// SPDX-License-Identifier: Apache-2.0
use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn helix_verify_json_reports_discovered_drs() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/objects/test-object-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "test-object-1"})),
        )
        .mount(&server)
        .await;

    Command::cargo_bin("helix")
        .unwrap()
        .args(["verify", &server.uri(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\": \"drs\""))
        .stdout(predicate::str::contains(server.uri()));
}
