// SPDX-License-Identifier: Apache-2.0
//! Canonical first-usable DRS 1.4.0 path against the in-process fixture.
//!
//! Not Ferrum. Not a live independent implementation. Not GA4GH certification.
//! `make verify-drs` → `cargo run --locked --offline --example verify-drs-140`

use std::path::PathBuf;

#[allow(dead_code)]
#[path = "../tests/support/mock_ga4gh_drs.rs"]
mod mock_ga4gh_drs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    helix::default_client_log_filter();
    let mock = mock_ga4gh_drs::start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    let out = std::env::var("HELIX_VERIFY_JSON").unwrap_or_else(|_| "verify.json".into());
    let out_path = PathBuf::from(&out);

    eprintln!("Helix verify — DRS 1.4.0 against in-process fixture (docs/FIXTURES.md §1).");
    eprintln!("Not Ferrum. Not HELIOS. Not GA4GH certification. Not independent evidence.");
    eprintln!("target: {url}");
    eprintln!("selected: drs 1.4.0");
    eprintln!();

    let outcome = helix::verify::verify_with_options(
        &url,
        helix::verify::VerifyOptions {
            selection: helix::verify::VerifySelection::Explicit {
                standard: "drs".into(),
                version: "1.4.0".into(),
                release_class: None,
            },
            declared_target: helix::target::DeclaredTarget {
                target_id: Some("helix-fixture-drs".into()),
                kind: helix::target::TargetKind::Fixture,
                implementation_name: Some("helix-in-process-mock".into()),
                implementation_version: Some("fixture".into()),
                ..helix::target::DeclaredTarget::default()
            },
            ..Default::default()
        },
    )
    .await?;

    print!(
        "{}",
        helix::report::format_verify_text(&outcome.run, helix::report::color_enabled())
    );

    std::fs::write(&out_path, helix::report::verify_json(&outcome.run)?)?;
    eprintln!("Wrote helix-verification-v1 to {}", out_path.display());
    eprintln!();
    print!("{}", helix::report::format_inspect_text(&outcome.run));

    if !outcome.is_success() {
        std::process::exit(1);
    }
    Ok(())
}
