# Helix

Helix is a CLI that checks whether a running GA4GH HTTP stack behaves (conformance, selected security behaviour, performance regression). It is [HelixTest](https://github.com/SynapticFour/HelixTest) becoming a standalone binary (separate git root, pin **v0.1.3**), not a new test platform.

Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency.

**Not HELIOS** (no signed evidence, RO-Crate, or PDF). **Not GA4GH certification.** **Not a Ferrum clinical pilot.** Early stage: `helix verify` executes **DRS** and **WES** checks; TES/TRS/htsget are discovered only. One maintainer.

Start here: [docs/FOR-EVALUATORS.md](docs/FOR-EVALUATORS.md) (five minutes). Pack without a Synaptic Four conversation: [docs/evaluator-pack/README.md](docs/evaluator-pack/README.md). First-clone pitfalls: [docs/EVALUATOR_JOURNEY.md](docs/EVALUATOR_JOURNEY.md).

[![CI](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml/badge.svg)](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-early--stage-orange.svg)](https://github.com/SynapticFour/Helix)

These public repositories are maintained by the same organisation and are designed to work together. Each repository keeps its own version and license. For details on roles, maturity, and how the components relate to one another, see [SUITE-OVERVIEW](https://github.com/SynapticFour/.github/blob/main/profile/SUITE-OVERVIEW.md).

## What this is not

- **Not a server.** Point `helix` at a target you already started (or use `make verify-fixture`).
- **Not HELIOS.** No signed audit trails, RO-Crate, PDF export, or reproducibility envelope. Binary name is `helix`, never `helios`.
- **Not GA4GH certification.** Green CI and a green `helix verify` are a technical signal.
- **Not a Ferrum production claim.** Ferrum is BUSL-1.1, on-prem, Rust, tested; there is no real clinical pilot (DIZ / genomDE). Demos and CI ≠ pilot ≠ production.
- **Not a paid SKU.** Apache-2.0, same licence as HelixTest.

## Helix vs HELIOS

Helix answers *whether* a running system behaves. HELIOS answers *what* ran and *how* to reproduce it. Full gate: [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md). HELIOS CLI / PyPI: [`helios-audit`](https://github.com/SynapticFour/HELIOS) (Early Access).

## Quick start

Needs a sibling [HelixTest](https://github.com/SynapticFour/HelixTest) checkout at the SHA in [VERSIONS.lock](VERSIONS.lock) (path `../HelixTest`). Rust **1.91.1** via rustup ([docs/INSTALL.md](docs/INSTALL.md)). Helix does not start Ferrum.

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make prove
make verify-fixture
```

`make prove` is docs + `cargo test --locked --all-targets` against deterministic in-process fixtures ([docs/FIXTURES.md](docs/FIXTURES.md)). It does not need Ferrum, Docker, or credentials.

`make verify-fixture` runs **`helix verify`** against that DRS fixture and prints `HELIX VERIFICATION`. DETECTED is not a pass. Skip is never pass. Layout: [docs/REPORT.md](docs/REPORT.md).

### Optional: install the `helix` binary

```bash
make install    # cargo install --path . --locked; still needs sibling HelixTest to compile
helix verify http://127.0.0.1:8080   # only if you already started a stack
```

### Optional: live origin you started

Ferrum-style gateway on `http://127.0.0.1:8080` (`cd ../Ferrum && make up`) is a **reference** live target, not required for prove.

```bash
NO_COLOR=1 cargo run --bin helix -- verify http://127.0.0.1:8080
# or: make test-live HELIX_LIVE_URL=http://127.0.0.1:8080
```

`--format json` (alias `--report json`) prints Helix `VerificationRun` on stdout. Default profile is `generic`. `--profile ferrum` is opt-in and never inferred from the target ([docs/PROFILES.md](docs/PROFILES.md)). Exit `0` if overall status is pass; exit `1` on FAIL, ERROR, skip-only, unreachable target, or runtime error.

The broader HelixTest ladder (TES, TRS, Beacon, htsget, HMAC auth, …) still lives in HelixTest — [INVENTORY.md](INVENTORY.md). WES fixture assumptions: [docs/WES.md](docs/WES.md).

### Other commands (scaffolds)

Need a running origin (not `verify-fixture`):

```bash
# Stage 3 — dummy HMAC only (test-fixtures/, NICHT FÜR PRODUKTION). Not a security audit.
cargo run --bin helix -- security http://127.0.0.1:8080 \
  --hmac-secret-file test-fixtures/hmac/shared-secret.txt

# Stage 4 — 3-GET smoke (`http.drs.smoke.v1`); >10% worse warns, does not fail the process
cargo run --bin helix -- bench --baseline http://127.0.0.1:8080 --candidate http://127.0.0.1:8080

# Compare two verify JSON files (PASS→FAIL at stable id = regression; not a score)
cargo run --bin helix -- compare previous.json current.json --format json
```

CLI contract: [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md). Roadmap (scope stages, not dates): [docs/HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md).

## Documentation

- [docs/FOR-EVALUATORS.md](docs/FOR-EVALUATORS.md) — what it is / is not, how to run, what a result means, Ferrum, HELIOS, how to report
- [docs/evaluator-pack/README.md](docs/evaluator-pack/README.md) — install, one-page explanation, target/fixtures, commands, example JSON, interpretation, failure template
- [docs/EXTERNAL_TARGET_CONTRACT.md](docs/EXTERNAL_TARGET_CONTRACT.md) — what a generic DRS/WES origin must expose (`helix verify <url>`)
- [docs/EVALUATOR_JOURNEY.md](docs/EVALUATOR_JOURNEY.md) — first-clone obstacles
- [docs/INSTALL.md](docs/INSTALL.md) · [docs/PROVE.md](docs/PROVE.md) · [docs/FIXTURES.md](docs/FIXTURES.md) · [docs/REPORT.md](docs/REPORT.md)
- [INVENTORY.md](INVENTORY.md) — what HelixTest actually runs
- [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md) · [docs/RUN_IDENTITY.md](docs/RUN_IDENTITY.md) · [docs/OPEN_SOURCE_RELEASE_CHECKLIST.md](docs/OPEN_SOURCE_RELEASE_CHECKLIST.md) · [docs/DECISIONS.md](docs/DECISIONS.md) · [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md)
- [docs/DRS_PROFILE.md](docs/DRS_PROFILE.md) · [docs/WES.md](docs/WES.md) · [docs/SECURITY_PROFILE.md](docs/SECURITY_PROFILE.md) · [docs/CRYPT4GH.md](docs/CRYPT4GH.md) · [docs/BENCHMARKS.md](docs/BENCHMARKS.md) · [docs/DIAGNOSTICS.md](docs/DIAGNOSTICS.md) · [docs/SCHEMA.md](docs/SCHEMA.md) · [docs/REGRESSION.md](docs/REGRESSION.md) · [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) · [docs/HELIX_VISION.md](docs/HELIX_VISION.md) · [docs/HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md)
- CI comment wrapper: [helix-action](https://github.com/SynapticFour/helix-action) (pilot; fail only on `NEW_FAIL` at stable id; not a required Ferrum check)

## Contributing

- [Open an issue](https://github.com/SynapticFour/Helix/issues) for bugs, missing coverage, or a `helix verify` / `make verify-fixture` run. That is the reporting path.
- Small PRs after an issue: [CONTRIBUTING.md](CONTRIBUTING.md).
- Do not add HELIOS-style evidence (RO-Crate, PDF, signatures) here. Do not claim Ferrum production or clinical deployments.

## License

Apache License 2.0 — see [LICENSE](LICENSE). Same licence as HelixTest.

**Synaptic Four** · [contact@synapticfour.com](mailto:contact@synapticfour.com) · [synapticfour.com](https://synapticfour.com)
