# Helix

Helix is verification infrastructure for genomic and clinical computing.

You point it at a [GA4GH](https://www.ga4gh.org/) HTTP origin you already run. It tests that implementation against a documented contract and records what was tested, what can be claimed, and what was left unevaluated.

The check bodies come from [HelixTest](https://github.com/SynapticFour/HelixTest) (separate git repository). Helix is that engine as a standalone `helix` binary, not a new test platform. The exact HelixTest source is the SHA in [VERSIONS.lock](VERSIONS.lock). Tag **v0.1.3** is HelixTest release lineage, not the commit Helix compiles.

Helix runs the same documented DRS and WES checks against any HTTP origin that implements those GA4GH paths. Ferrum is used as a reference target, not a dependency. Helix supports technical verification checks for GA4GH DRS 1.4.0 within the declared coverage boundary. That is not a VERIFIED claim and not GA4GH certification. Default `helix verify TARGET` stays unversioned.

**Current baseline (this branch, not a claim that `origin/main` already contains it):** versioned **GA4GH DRS 1.4.0** technical verification; declared **partial** coverage; persisted `helix-verification-v1`; `helix inspect`; optional `helix differential`. WES is not a published Supported standard. Authorization verification is deferred. **Not HELIOS** (no signed evidence, RO-Crate, or PDF). **Not GA4GH certification.** **Not complete DRS coverage.**

Do not take this page on trust. Inspect pins, tests, and recorded JSON: [docs/TRUST.md](docs/TRUST.md).

Start here: [docs/HELIX_PRODUCT.md](docs/HELIX_PRODUCT.md) (what Helix can do today). Clone-and-run: [docs/FOR-EVALUATORS.md](docs/FOR-EVALUATORS.md). Operator path: [docs/OPERATOR_VERIFY.md](docs/OPERATOR_VERIFY.md).

[![CI](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml/badge.svg)](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-early--stage-orange.svg)](https://github.com/SynapticFour/Helix)

## Names a first-time reader will hit

| Name | What it is here |
|------|-----------------|
| **GA4GH** | Global Alliance for Genomics and Health. Publishes HTTP API specifications (DRS, WES, …). Helix is not GA4GH and is not certified by GA4GH. |
| **HelixTest** | Existing test engine. Helix wraps it. Required as a sibling checkout at the SHA in [VERSIONS.lock](VERSIONS.lock). |
| **Ferrum** | Optional live reference implementation (BUSL-1.1). Not a Helix dependency. Not required for `make prove`. |
| **HELIOS** | Different product (`helios-audit`): signed evidence, RO-Crate, PDF. Not this binary. |

Related public repositories: [SUITE-OVERVIEW](https://github.com/SynapticFour/.github/blob/main/profile/SUITE-OVERVIEW.md). You do not need that overview to run Helix.

## What this is not

- **Not a server.** Point `helix` at a target you already started (or use `make verify-fixture` / `make verify-drs`).
- **Not HELIOS.** No signed audit trails, RO-Crate, PDF, or scientific-reproducibility envelope. Binary name is `helix`, never `helios`.
- **Not GA4GH certification.** Green CI and a green `helix verify` are a technical signal. Inspect pins and JSON: [docs/TRUST.md](docs/TRUST.md).
- **Not a Ferrum production claim.** Ferrum is on-prem, Rust, tested; demos and CI ≠ pilot ≠ production.
- **Not a ranking of implementations.** `helix differential` compares two artifacts under the same contract. In-process mocks are not a second implementation.

## Helix vs HELIOS

Helix answers *whether* a running system behaves on a documented check suite. HELIOS answers *what* ran and *how* to reproduce a pipeline. Full gate: [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md). HELIOS CLI / PyPI: [`helios-audit`](https://github.com/SynapticFour/HELIOS) (Early Access).

## Quick start

Needs a sibling [HelixTest](https://github.com/SynapticFour/HelixTest) checkout at the SHA in [VERSIONS.lock](VERSIONS.lock) (path `../HelixTest`). Rust **1.91.1** via rustup ([docs/INSTALL.md](docs/INSTALL.md)). Helix does not start Ferrum.

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch
make prove
make verify-drs
```

`make fetch` is crates.io at [Cargo.lock](Cargo.lock) checksums (explicit network, not a GA4GH download). `make prove` is docs + `cargo test --locked --offline --all-targets` against in-process fixtures ([docs/FIXTURES.md](docs/FIXTURES.md)). It does not need Ferrum, Docker, or credentials.

`make verify-drs` is the canonical DRS 1.4.0 path against the in-process fixture: human report, `verify.json`, and `helix inspect`. It is not independent-implementation evidence. Unversioned `make verify-fixture` still exists. Against a DRS you started: [docs/OPERATOR_VERIFY.md](docs/OPERATOR_VERIFY.md). Two independent DRS implementations (Starter Kit and Bento): [docs/INDEPENDENT_DRS.md](docs/INDEPENDENT_DRS.md). `helix inspect` classifies current vs historical evidence and does not rewrite the file. `helix differential` compares two artifacts under the same contract and does not rank implementations.

### Optional: install the `helix` binary

```bash
make install    # cargo install --path . --locked; still needs sibling HelixTest to compile
helix verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
helix inspect verify.json
```

### Optional: live origin you started

A gateway on `http://127.0.0.1:8080` (for example Ferrum `cd ../Ferrum && make up`) is a **reference** live target, not required for prove.

```bash
NO_COLOR=1 cargo run --bin helix -- verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
# or: make test-live HELIX_LIVE_URL=http://127.0.0.1:8080
```

`--format json` (alias `--report json`) prints Helix `VerificationRun` on stdout. Default profile is `generic`. `--profile ferrum` is opt-in and never inferred from the target ([docs/PROFILES.md](docs/PROFILES.md)). Exit `0` if overall status is pass; exit `1` on FAIL, ERROR, skip-only, unreachable target, or runtime error.

TES, TRS, Beacon, htsget, HMAC auth, and the rest of the HelixTest ladder still live in HelixTest — [INVENTORY.md](INVENTORY.md). WES fixture assumptions: [docs/WES.md](docs/WES.md). Authorization: [docs/AUTHORIZATION.md](docs/AUTHORIZATION.md) (UNEVALUATED; B13 deferred).

### Shipped commands

Need a running origin except `inspect` / `differential` / `compare` / `matrix` / `standards` of saved files. Dummy HMAC files are **not for production**.

```bash
# Versioned DRS 1.4.0 (canonical). Unversioned: helix verify URL
cargo run --bin helix -- verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json

# Classify standing of retained verify JSON (does not rewrite the file)
cargo run --bin helix -- inspect verify.json

# Check-level comparison of two verify JSON files. Same contract; not a ranking.
cargo run --bin helix -- differential first.json second.json

# Compare two verify JSON files (PASS→FAIL at stable id = regression; not a score)
cargo run --bin helix -- compare previous.json current.json --format json

# Pinned GA4GH spec provenance (does not run verify; no network).
# DRS 1.4.0 is SUPPORTED for technical verification within declared coverage.
# Default `list` includes AVAILABLE rows; `--supported-only` lists DRS 1.4.0.
# Not GA4GH certification. Not complete DRS coverage.
cargo run --bin helix -- standards list
cargo run --bin helix -- standards list --supported-only
cargo run --bin helix -- standards show drs 1.5.0
cargo run --bin helix -- standards validate
cargo run --bin helix -- standards trace drs.object.schema.openapi

# Selected dummy-HMAC behaviour checks. Not a security audit.
cargo run --bin helix -- security http://127.0.0.1:8080 \
  --hmac-secret-file test-fixtures/hmac/shared-secret.txt

# 3-GET smoke (`http.drs.smoke.v1`); >10% worse warns, does not fail the process
cargo run --bin helix -- bench --baseline http://127.0.0.1:8080 --candidate http://127.0.0.1:8080

# Interop matrix — pending without independent run JSON; mocks are not a second implementation
cargo run --bin helix -- matrix --format json
```

CLI contract: [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md). Product: [docs/HELIX_PRODUCT.md](docs/HELIX_PRODUCT.md). Trust: [docs/TRUST.md](docs/TRUST.md).

## Documentation

**Start.** [HELIX_PRODUCT.md](docs/HELIX_PRODUCT.md) · [FOR-EVALUATORS.md](docs/FOR-EVALUATORS.md) · [OPERATOR_VERIFY.md](docs/OPERATOR_VERIFY.md) · [TRUST.md](docs/TRUST.md) · [evaluator-pack](docs/evaluator-pack/README.md)

**Run.** [INSTALL.md](docs/INSTALL.md) · [PROVE.md](docs/PROVE.md) · [FIXTURES.md](docs/FIXTURES.md) · [INDEPENDENT_VERIFICATION.md](docs/INDEPENDENT_VERIFICATION.md)

**Interpret a result.** [REPORT.md](docs/REPORT.md) · [CLAIMS.md](docs/CLAIMS.md) · [TAXONOMY.md](docs/TAXONOMY.md) · [TRACEABILITY.md](docs/TRACEABILITY.md) · [BEHAVIOR.md](docs/BEHAVIOR.md) · [SCHEMA.md](docs/SCHEMA.md) · [COVERAGE.md](docs/COVERAGE.md)

**Standards.** [STANDARDS_REGISTRY.md](docs/STANDARDS_REGISTRY.md) · [STANDARD_VERSIONING.md](docs/STANDARD_VERSIONING.md) — default `helix verify` is unversioned; `--standard` / `--version` fail closed when a pack is not SUPPORTED.

**Limits.** [EXTERNAL_TARGET_CONTRACT.md](docs/EXTERNAL_TARGET_CONTRACT.md) · [INTEROP.md](docs/INTEROP.md) · [HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md) · [THREAT_MODEL.md](docs/THREAT_MODEL.md) · [AUTHORIZATION.md](docs/AUTHORIZATION.md)

**More.** [INVENTORY.md](INVENTORY.md) · [DRS_PROFILE.md](docs/DRS_PROFILE.md) · [WES.md](docs/WES.md) · [SECURITY_PROFILE.md](docs/SECURITY_PROFILE.md) · [CRYPT4GH.md](docs/CRYPT4GH.md) · [BENCHMARKS.md](docs/BENCHMARKS.md) · [DIAGNOSTICS.md](docs/DIAGNOSTICS.md) · [REGRESSION.md](docs/REGRESSION.md) · [DECISIONS.md](docs/DECISIONS.md) · [CLI_CONTRACT.md](docs/CLI_CONTRACT.md) · [HELIX_VISION.md](docs/HELIX_VISION.md) · [HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md) · [IDENTITY.md](docs/IDENTITY.md) · [INDEPENDENT_DRS.md](docs/INDEPENDENT_DRS.md)

## Contributing

- [Open an issue](https://github.com/SynapticFour/Helix/issues) for bugs, missing coverage, or a `helix verify` / `make verify-fixture` run.
- Small PRs after an issue: [CONTRIBUTING.md](CONTRIBUTING.md). Who reviews a normative mapping: [STANDARDS_REGISTRY.md](docs/STANDARDS_REGISTRY.md) §10.1.
- Do not add HELIOS-style evidence (RO-Crate, PDF, signatures) here. Do not claim Ferrum production or clinical deployments.

## License

Apache License 2.0 — see [LICENSE](LICENSE). Same licence as HelixTest.

**Synaptic Four** · [contact@synapticfour.com](mailto:contact@synapticfour.com) · [synapticfour.com](https://synapticfour.com)
