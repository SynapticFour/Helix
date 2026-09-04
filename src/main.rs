// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use common::report::TestStatus;
use helix::discover::{Discovery, VERIFY_ORDER};
use helix::verify::{verify, VerifyOutcome};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(name = "helix")]
#[command(
    version,
    about = "Helix — GA4GH VERIFY CLI (HelixTest heritage). Not HELIOS."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover GA4GH APIs under a gateway-style URL, then run HelixTest checks (DRS first).
    Verify(VerifyArgs),
}

#[derive(Parser, Debug)]
struct VerifyArgs {
    /// Gateway-style origin, e.g. http://127.0.0.1:8080
    endpoint: String,

    /// text (default) or json
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Serialize)]
struct VerifyJson<'a> {
    discovery: &'a Discovery,
    services: &'a [common::report::ServiceReport],
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Verify(args) => verify_cmd(args).await,
    }
}

async fn verify_cmd(args: VerifyArgs) -> Result<()> {
    let outcome = verify(&args.endpoint).await?;
    match args.format {
        OutputFormat::Json => print_json(&outcome)?,
        OutputFormat::Text => print_text(&outcome),
    }
    if outcome.has_failures() {
        std::process::exit(1);
    }
    Ok(())
}

fn print_json(outcome: &VerifyOutcome) -> Result<()> {
    let body = VerifyJson {
        discovery: &outcome.discovery,
        services: std::slice::from_ref(&outcome.drs),
    };
    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}

fn print_text(outcome: &VerifyOutcome) {
    print_discovery_text(&outcome.discovery);
    println!();
    println!("DRS (HelixTest checks; not certification)");
    for t in &outcome.drs.tests {
        let mark = match t.status {
            TestStatus::Pass => "PASS",
            TestStatus::Fail => "FAIL",
            TestStatus::Skip => "SKIP",
        };
        match &t.error {
            Some(err) if t.status != TestStatus::Pass => {
                println!("  {mark}  {} — {err}", t.name);
            }
            _ => println!("  {mark}  {}", t.name),
        }
    }
}

fn print_discovery_text(d: &Discovery) {
    println!("Helix verify — GA4GH discovery (not certification)");
    println!("endpoint: {}", d.endpoint);
    println!("Helix tests behavior against the GA4GH spec, independent of implementation.");
    println!("Ferrum is used as a reference target, not a dependency.");
    println!();
    for kind in VERIFY_ORDER {
        match d.get(kind) {
            Some(s) => println!("{:<8} found   {}", kind.as_str(), s.base_url),
            None => println!("{:<8} missing", kind.as_str()),
        }
    }
    if d.found.is_empty() {
        println!("\nNo Stage 1 APIs (DRS, WES, TES, TRS, htsget) answered.");
    }
}
