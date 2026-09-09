// SPDX-License-Identifier: Apache-2.0
//! Run `helix verify` against the in-process DRS fixture (docs/FIXTURES.md §1).
//! Not Ferrum. Not a live stack. Not GA4GH certification. `make verify-fixture`.

#[allow(dead_code)] // file also mounts the invalid-object fixture used only by tests
#[path = "../tests/support/mock_ga4gh_drs.rs"]
mod mock_ga4gh_drs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    helix::default_client_log_filter();
    let mock = mock_ga4gh_drs::start_mock_ga4gh_drs().await;
    let url = mock.drs_url();
    eprintln!("Helix verify — in-process DRS fixture (docs/FIXTURES.md §1).");
    eprintln!("Not Ferrum. Not HELIOS. Not GA4GH certification.");
    eprintln!("target: {url}");
    eprintln!();
    let outcome = helix::verify::verify(&url).await?;
    print!(
        "{}",
        helix::report::format_verify_text(&outcome.run, helix::report::color_enabled())
    );
    if !outcome.is_success() {
        std::process::exit(1);
    }
    Ok(())
}
