// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use helix::bench::{
    run_bench, BenchRequest, MeasureConfig, DEFAULT_REPETITIONS, DEFAULT_THRESHOLD_PCT,
    DEFAULT_WARMUP,
};
use helix::compare::compare_files;
use helix::profile::ProfileId;
use helix::report::{
    print_bench_json, print_bench_text, print_compare_json, print_compare_text, print_json,
    print_security_json, print_security_text, print_text,
};
use helix::security::{load_hmac_secret, run_security};
use helix::verify::verify_with_profile;
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
    /// Discover GA4GH APIs under a gateway-style URL, then run DRS and WES checks when TESTABLE.
    Verify(VerifyArgs),
    /// Stage 3: Security Behavior Profile + Crypt4GH protocol layout (dummy fixtures only).
    Security(SecurityArgs),
    /// Stage 4: http.drs.smoke.v1 vs two endpoints; warn on >threshold% worse, never fail CI.
    Bench(BenchArgs),
    /// Compare two helix verify JSON runs at stable check id (PASS→FAIL = regression).
    Compare(CompareArgs),
}

#[derive(Parser, Debug)]
struct VerifyArgs {
    /// Gateway-style origin, e.g. http://127.0.0.1:8080
    endpoint: String,

    /// Helix profile. Default `generic`. Never inferred from the target.
    #[arg(long, value_enum, default_value_t = CliProfile::Generic)]
    profile: CliProfile,

    /// text (default) or json (Helix VerificationRun). `--report` is an alias.
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

    /// Override Crypt4GH well-formed fixture for HLX-AUTH-050 (default: embedded test header). Never a private key.
    #[arg(long)]
    crypt4gh_file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct BenchArgs {
    /// Baseline gateway origin (e.g. Ferrum vX or Demo).
    #[arg(long)]
    baseline: String,

    /// Candidate gateway origin (e.g. Ferrum vY).
    #[arg(long)]
    candidate: String,

    #[arg(long, default_value = "baseline")]
    baseline_label: String,

    #[arg(long, default_value = "candidate")]
    candidate_label: String,

    /// Warn if a metric is this many percent worse. Does not fail the process.
    #[arg(long, default_value_t = DEFAULT_THRESHOLD_PCT)]
    threshold: f64,

    /// Discarded runs before measured repetitions (not included in stats).
    #[arg(long, default_value_t = DEFAULT_WARMUP)]
    warmup: u32,

    /// Measured repetitions of http.drs.smoke.v1. Sample p95 needs 20.
    #[arg(long, default_value_t = DEFAULT_REPETITIONS)]
    repetitions: u32,

    /// Skip optional Linux VmRSS of this Helix process.
    #[arg(long)]
    no_rss: bool,

    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Parser, Debug)]
struct CompareArgs {
    /// Previous `helix verify --format json` file.
    previous: PathBuf,
    /// Current `helix verify --format json` file.
    current: PathBuf,
    /// text (default) or json (Helix CompareReport). `--report` is an alias.
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
enum CliProfile {
    #[default]
    Generic,
    Ferrum,
}

impl From<CliProfile> for ProfileId {
    fn from(p: CliProfile) -> Self {
        match p {
            CliProfile::Generic => ProfileId::Generic,
            CliProfile::Ferrum => ProfileId::Ferrum,
        }
    }
}

#[tokio::main]
async fn main() {
    helix::default_client_log_filter();
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli).await {
        eprintln!("{}", helix::redact::redact_text(&format!("{e:#}")));
        std::process::exit(1);
    }
}

async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Verify(args) => verify_cmd(args).await,
        Commands::Security(args) => security_cmd(args).await,
        Commands::Bench(args) => bench_cmd(args).await,
        Commands::Compare(args) => compare_cmd(args),
    }
}

async fn verify_cmd(args: VerifyArgs) -> Result<()> {
    let outcome = verify_with_profile(&args.endpoint, args.profile.into()).await?;
    match args.format {
        OutputFormat::Json => print_json(&outcome)?,
        OutputFormat::Text => print_text(&outcome),
    }
    if !outcome.is_success() {
        std::process::exit(1);
    }
    Ok(())
}

fn resolve_hmac_secret(file: Option<PathBuf>) -> Result<Option<String>> {
    if let Ok(s) = std::env::var("HELIX_HMAC_SECRET") {
        let t = s.trim();
        if !t.is_empty() {
            if t.len() as u64 > helix::http_safety::MAX_SECRET_FILE_BYTES {
                anyhow::bail!(
                    "HELIX_HMAC_SECRET exceeds {} bytes (value not printed)",
                    helix::http_safety::MAX_SECRET_FILE_BYTES
                );
            }
            return Ok(Some(t.to_string()));
        }
    }
    let explicit = file.is_some();
    let path = file.unwrap_or_else(|| PathBuf::from("test-fixtures/hmac/shared-secret.txt"));
    match load_hmac_secret(&path) {
        Ok(s) => Ok(Some(s)),
        Err(_) if !explicit && !path.exists() => Ok(None),
        Err(e) => Err(e),
    }
}

async fn security_cmd(args: SecurityArgs) -> Result<()> {
    let secret = resolve_hmac_secret(args.hmac_secret_file)?;
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

async fn bench_cmd(args: BenchArgs) -> Result<()> {
    let outcome = run_bench(&BenchRequest {
        baseline_url: args.baseline,
        candidate_url: args.candidate,
        baseline_label: args.baseline_label,
        candidate_label: args.candidate_label,
        threshold_pct: args.threshold,
        config: MeasureConfig {
            warmup: args.warmup,
            repetitions: args.repetitions,
            collect_rss: !args.no_rss,
        },
    })
    .await?;
    match args.format {
        OutputFormat::Json => print_bench_json(&outcome)?,
        OutputFormat::Text => print_bench_text(&outcome),
    }
    // Warnings are for humans / helix-action comments. Never fail the build.
    Ok(())
}

fn compare_cmd(args: CompareArgs) -> Result<()> {
    let report = compare_files(&args.previous, &args.current)?;
    match args.format {
        OutputFormat::Json => print_compare_json(&report)?,
        OutputFormat::Text => print_compare_text(&report),
    }
    let code = report.process_exit_code();
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}
