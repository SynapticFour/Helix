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
use helix::standards::{
    format_list_text, format_show_text, list_json, load_path, show_json, validate_path,
    ReleaseClass,
};
use helix::target::{DeclaredTarget, TargetKind};
use helix::verify::{verify_with_options, VerifyOptions, VerifySelection};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "helix")]
#[command(
    version,
    about = "Helix — DRS/WES VERIFY CLI wrapping HelixTest. Not HELIOS. Not GA4GH certification."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover GA4GH APIs under a gateway-style URL, then run DRS and WES checks when TESTABLE.
    #[command(disable_version_flag = true)]
    Verify(VerifyArgs),
    /// Stage 3: Security Behavior Profile + Crypt4GH protocol layout (dummy fixtures only).
    Security(SecurityArgs),
    /// Stage 4: http.drs.smoke.v1 vs two endpoints; warn on >threshold% worse, never fail CI.
    Bench(BenchArgs),
    /// Compare two helix verify JSON runs at stable check id (PASS→FAIL = regression).
    Compare(CompareArgs),
    /// Target-neutral interop matrix from zero or more verify JSON files. External validation pending without independent runs.
    Matrix(MatrixArgs),
    /// Inspect pinned GA4GH specification provenance. Does not run verify. No network fetch.
    Standards(StandardsArgs),
}

#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
struct VerifyArgs {
    /// Gateway-style origin, e.g. http://127.0.0.1:8080
    endpoint: String,

    /// Helix profile. Default `generic`. Never inferred from the target.
    #[arg(long, value_enum, default_value_t = CliProfile::Generic)]
    profile: CliProfile,

    /// Restrict versioned selection to this GA4GH standard (`drs`, `wes`, …).
    #[arg(long)]
    standard: Option<String>,

    /// Exact GA4GH specification version (Mode 1). Requires `--standard`. Never substitutes.
    #[arg(
        long = "version",
        value_name = "GA4GH_VERSION",
        requires = "standard",
        conflicts_with = "all_supported_versions"
    )]
    ga4gh_version: Option<String>,

    /// Run every OfficialSupported pack for `--standard` (Mode 3). Conflicts with `--version`.
    #[arg(
        long = "all-supported-versions",
        requires = "standard",
        conflicts_with = "ga4gh_version"
    )]
    all_supported_versions: bool,

    /// Release class for Mode 1 (default official). `development` is never selectable.
    #[arg(long, value_enum, requires = "ga4gh_version")]
    release_class: Option<CliReleaseClass>,

    /// Operator-declared target id. Not inferred from headers, Docker names, or URLs.
    #[arg(long)]
    target_id: Option<String>,

    /// Operator-declared what this origin is. Default unspecified (not independent evidence).
    #[arg(long, value_enum)]
    target_kind: Option<CliTargetKind>,

    /// Operator-declared implementation name. Untrusted metadata. Not proof.
    #[arg(long)]
    implementation_name: Option<String>,

    /// Operator-declared implementation version. Untrusted. Never becomes verified_version.
    #[arg(long)]
    implementation_version: Option<String>,

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
struct StandardsArgs {
    #[command(subcommand)]
    command: StandardsCommand,
}

#[derive(Subcommand, Debug)]
enum StandardsCommand {
    /// List registry rows and OFFICIAL ∩ SUPPORTED discovery.
    List(StandardsListArgs),
    /// Show one standard version (exact match). Never substitutes another version.
    Show(StandardsShowArgs),
    /// Validate registry.yaml + vendor hashes. No network.
    Validate(StandardsValidateArgs),
    /// Follow one Helix check id back to kind, authority, and any related AVAILABLE pin.
    Trace(StandardsTraceArgs),
}

#[derive(Parser, Debug)]
struct StandardsListArgs {
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    /// Registry YAML (default: this crate's standards/registry.yaml).
    #[arg(long)]
    registry: Option<PathBuf>,
    /// Only print OFFICIAL ∩ SUPPORTED (default discovery set).
    #[arg(long)]
    supported_only: bool,
}

#[derive(Parser, Debug)]
struct StandardsShowArgs {
    /// Standard id (drs, wes, …).
    standard: String,
    /// GA4GH version string (e.g. 1.5.0). Exact match; no substitution.
    version: String,
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    #[arg(long)]
    registry: Option<PathBuf>,
    /// Disambiguate when the same version exists in more than one release class.
    #[arg(long, value_enum)]
    release_class: Option<CliReleaseClass>,
}

#[derive(Parser, Debug)]
struct StandardsValidateArgs {
    /// Registry YAML (default: this crate's standards/registry.yaml).
    registry: Option<PathBuf>,
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Parser, Debug)]
struct StandardsTraceArgs {
    /// Helix check id (e.g. drs.object.schema).
    check_id: String,
    #[arg(long, visible_alias = "report", value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum CliReleaseClass {
    Official,
    Ballot,
    Snapshot,
    Development,
}

impl From<CliReleaseClass> for ReleaseClass {
    fn from(c: CliReleaseClass) -> Self {
        match c {
            CliReleaseClass::Official => ReleaseClass::Official,
            CliReleaseClass::Ballot => ReleaseClass::Ballot,
            CliReleaseClass::Snapshot => ReleaseClass::Snapshot,
            CliReleaseClass::Development => ReleaseClass::Development,
        }
    }
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

#[derive(Parser, Debug)]
struct MatrixArgs {
    /// Recorded `helix verify --format json` as `id=path`. Repeatable. Omit for pending slots only.
    #[arg(long = "run", value_name = "ID=PATH")]
    run: Vec<String>,
    /// Kind for a `--run` id: `helix_fixture`, `reference_target`, `independent_implementation`, `unspecified`.
    #[arg(long = "kind", value_name = "ID=KIND")]
    kind: Vec<String>,
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

#[derive(ValueEnum, Debug, Clone, Copy)]
enum CliTargetKind {
    Mock,
    Fixture,
    SyntheticTarget,
    ReferenceImplementation,
    RealIndependentLocalImplementation,
    RealExternalImplementation,
    Unspecified,
}

impl From<CliTargetKind> for TargetKind {
    fn from(k: CliTargetKind) -> Self {
        match k {
            CliTargetKind::Mock => TargetKind::Mock,
            CliTargetKind::Fixture => TargetKind::Fixture,
            CliTargetKind::SyntheticTarget => TargetKind::SyntheticTarget,
            CliTargetKind::ReferenceImplementation => TargetKind::ReferenceImplementation,
            CliTargetKind::RealIndependentLocalImplementation => {
                TargetKind::RealIndependentLocalImplementation
            }
            CliTargetKind::RealExternalImplementation => TargetKind::RealExternalImplementation,
            CliTargetKind::Unspecified => TargetKind::Unspecified,
        }
    }
}

#[tokio::main]
async fn main() {
    helix::default_client_log_filter();
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli).await {
        eprintln!("{}", helix::sanitize::sanitize_untrusted(&format!("{e:#}")));
        std::process::exit(1);
    }
}

async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Verify(args) => verify_cmd(args).await,
        Commands::Security(args) => security_cmd(args).await,
        Commands::Bench(args) => bench_cmd(args).await,
        Commands::Compare(args) => compare_cmd(args),
        Commands::Matrix(args) => matrix_cmd(args),
        Commands::Standards(args) => standards_cmd(args),
    }
}

fn registry_path(explicit: Option<PathBuf>) -> PathBuf {
    explicit.unwrap_or_else(helix::standards::default_registry_path)
}

fn standards_cmd(args: StandardsArgs) -> Result<()> {
    match args.command {
        StandardsCommand::List(a) => standards_list(a),
        StandardsCommand::Show(a) => standards_show(a),
        StandardsCommand::Validate(a) => standards_validate(a),
        StandardsCommand::Trace(a) => standards_trace(a),
    }
}

fn standards_list(args: StandardsListArgs) -> Result<()> {
    let path = registry_path(args.registry);
    let reg = load_path(&path)?;
    if args.supported_only {
        let ids: Vec<String> = reg
            .official_supported()
            .iter()
            .map(|v| v.pack_id.clone())
            .collect();
        match args.format {
            OutputFormat::Json => {
                let v = serde_json::json!({
                    "schema_version": helix::standards::REGISTRY_SCHEMA_VERSION,
                    "official_supported": ids,
                    "substituted": false,
                    "default_discovery": "official_and_supported_only",
                });
                println!("{}", serde_json::to_string_pretty(&v)?);
            }
            OutputFormat::Text => {
                println!("HELIX STANDARDS REGISTRY");
                println!();
                println!("Default supported-version discovery (OFFICIAL ∩ SUPPORTED):");
                if ids.is_empty() {
                    println!("  (none)");
                } else {
                    for id in ids {
                        println!("  {id}");
                    }
                }
                println!();
                println!("Ballot and snapshot rows are never in this set.");
                println!("A GitHub tag alone does not make a version supported.");
                println!("Helix did not substitute another version.");
            }
        }
        return Ok(());
    }
    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&list_json(&reg))?),
        OutputFormat::Text => print!("{}", format_list_text(&reg)),
    }
    Ok(())
}

fn standards_show(args: StandardsShowArgs) -> Result<()> {
    let path = registry_path(args.registry);
    let reg = load_path(&path)?;
    let class = args.release_class.map(Into::into);
    let lookup = reg.lookup(&args.standard, &args.version, class);
    match args.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&show_json(&args.standard, &args.version, &lookup))?
            );
        }
        OutputFormat::Text => {
            print!(
                "{}",
                format_show_text(&args.standard, &args.version, &lookup)
            );
        }
    }
    match lookup {
        helix::standards::Lookup::Found(_) => Ok(()),
        _ => std::process::exit(1),
    }
}

fn standards_validate(args: StandardsValidateArgs) -> Result<()> {
    let path = registry_path(args.registry);
    match validate_path(&path) {
        Ok(reg) => match args.format {
            OutputFormat::Json => {
                let v = serde_json::json!({
                    "ok": true,
                    "path": path.display().to_string(),
                    "rows": reg.versions.len(),
                    "official_supported": reg.official_supported().len(),
                    "fetched": false,
                });
                println!("{}", serde_json::to_string_pretty(&v)?);
            }
            OutputFormat::Text => {
                println!("HELIX STANDARDS VALIDATE");
                println!("ok");
                println!("path: {}", path.display());
                println!("rows: {}", reg.versions.len());
                println!("official_supported: {}", reg.official_supported().len());
                println!("fetched: no");
            }
        },
        Err(e) => {
            match args.format {
                OutputFormat::Json => {
                    let v = serde_json::json!({
                        "ok": false,
                        "path": path.display().to_string(),
                        "kind": e.kind.as_str(),
                        "message": e.message,
                        "substituted": false,
                        "fetched": false,
                    });
                    println!("{}", serde_json::to_string_pretty(&v)?);
                }
                OutputFormat::Text => {
                    eprintln!("HELIX STANDARDS VALIDATE");
                    eprintln!("error: {}", e.message);
                    eprintln!("kind: {:?}", e.kind);
                    eprintln!("Helix did not substitute another version.");
                    eprintln!("Helix did not download a replacement spec.");
                }
            }
            std::process::exit(1);
        }
    }
    Ok(())
}

fn standards_trace(args: StandardsTraceArgs) -> Result<()> {
    match args.format {
        OutputFormat::Text => {
            print!(
                "{}",
                helix::traceability::format_trace_text(&args.check_id)?
            );
        }
        OutputFormat::Json => {
            let v = helix::traceability::format_trace_json(&args.check_id)?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
    }
    Ok(())
}

async fn verify_cmd(args: VerifyArgs) -> Result<()> {
    let declared_target = DeclaredTarget {
        target_id: args.target_id,
        kind: args
            .target_kind
            .map(Into::into)
            .unwrap_or(TargetKind::Unspecified),
        implementation_name: args.implementation_name,
        implementation_version: args.implementation_version,
        standard_version: None,
    };
    let selection = match args.standard {
        None => VerifySelection::Unversioned,
        Some(standard) => {
            if args.all_supported_versions {
                VerifySelection::Compatibility { standard }
            } else if let Some(version) = args.ga4gh_version {
                VerifySelection::Explicit {
                    standard,
                    version,
                    release_class: args.release_class.map(Into::into),
                }
            } else {
                VerifySelection::Automatic { standard }
            }
        }
    };
    let outcome = verify_with_options(
        &args.endpoint,
        VerifyOptions {
            profile: args.profile.into(),
            selection,
            registry: None,
            vendor_root: None,
            declared_target,
        },
    )
    .await?;
    match args.format {
        OutputFormat::Json => print_json(&outcome)?,
        OutputFormat::Text => print_text(&outcome)?,
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

fn matrix_cmd(args: MatrixArgs) -> Result<()> {
    use std::collections::BTreeMap;

    use helix::interop::{
        build_matrix, format_matrix_text, load_labeled_runs, matrix_json, parse_id_path,
        ImplementationKind,
    };

    let mut run_pairs = Vec::new();
    for spec in &args.run {
        run_pairs.push(parse_id_path(spec)?);
    }
    let mut kinds = BTreeMap::new();
    for spec in &args.kind {
        let (id, k) = parse_id_path(spec)?;
        kinds.insert(id, ImplementationKind::parse(&k)?);
    }
    for id in kinds.keys() {
        if !run_pairs.iter().any(|(i, _)| i == id) {
            anyhow::bail!("--kind `{id}` has no matching --run");
        }
    }
    let labeled = load_labeled_runs(&run_pairs, &kinds)?;
    let matrix = build_matrix(&labeled);
    match args.format {
        OutputFormat::Json => println!("{}", matrix_json(&matrix)?),
        OutputFormat::Text => print!("{}", format_matrix_text(&matrix)),
    }
    let code = matrix.process_exit_code();
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}
