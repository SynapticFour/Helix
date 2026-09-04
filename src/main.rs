// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use helix::report::{print_json, print_security_json, print_security_text, print_text};
use helix::security::{load_hmac_secret, run_security};
use helix::verify::verify;
use std::path::PathBuf;

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
    /// Stage 3: black-box auth behaviour + Crypt4GH header structure (dummy fixtures only).
    Security(SecurityArgs),
}

#[derive(Parser, Debug)]
struct VerifyArgs {
    /// Gateway-style origin, e.g. http://127.0.0.1:8080
    endpoint: String,

    /// text (default) or json (HelixTest OverallReport). `--report` is an alias.
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Parser, Debug)]
struct SecurityArgs {
    /// Gateway-style origin with an auth-protected DRS (e.g. http://127.0.0.1:8080)
    endpoint: String,

    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Dummy HMAC secret file (default: test-fixtures/hmac/shared-secret.txt). NICHT FÜR PRODUKTION.
    #[arg(long)]
    hmac_secret_file: Option<PathBuf>,

    /// Override Crypt4GH header fixture (default: embedded well-formed test header).
    #[arg(long)]
    crypt4gh_file: Option<PathBuf>,
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
        Commands::Security(args) => security_cmd(args).await,
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

fn resolve_hmac_secret(file: Option<PathBuf>) -> Option<String> {
    if let Ok(s) = std::env::var("HELIX_HMAC_SECRET") {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let path = file.unwrap_or_else(|| PathBuf::from("test-fixtures/hmac/shared-secret.txt"));
    load_hmac_secret(&path).ok()
}

async fn security_cmd(args: SecurityArgs) -> Result<()> {
    let secret = resolve_hmac_secret(args.hmac_secret_file);
    let outcome = run_security(
        &args.endpoint,
        secret.as_deref(),
        args.crypt4gh_file.as_deref(),
    )
    .await?;
    match args.format {
        OutputFormat::Json => print_security_json(&outcome)?,
        OutputFormat::Text => print_security_text(&outcome),
    }
    if outcome.has_failures() {
        std::process::exit(1);
    }
    Ok(())
}
