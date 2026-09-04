// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use helix::report::{print_json, print_text};
use helix::verify::verify;

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

    /// text (default) or json (HelixTest OverallReport). `--report` is an alias.
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
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
