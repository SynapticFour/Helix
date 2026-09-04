# Helix

Helix tests whether a running GA4GH HTTP stack behaves: API conformance, security behaviour, and performance regression. It does not attest reproducibility or produce signed evidence — that is [HELIOS](https://github.com/SynapticFour/HELIOS).

Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency.

**Maturity: Early stage.** Built alongside Ferrum. `helix verify` currently runs **DRS** checks (the five HelixTest DRS names in [INVENTORY.md](INVENTORY.md)). WES, TES, TRS, and htsget are discovered when they answer; those checks are **not** executed yet. `helix security` and `helix bench` are started scaffolds, not stage exits. One maintainer. Results are not GA4GH certification.

This is [HelixTest](https://github.com/SynapticFour/HelixTest) becoming a standalone VERIFY CLI (separate git root, pin **v0.1.3**). Not a new test platform.

[![CI](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml/badge.svg)](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-early--stage-orange.svg)](https://github.com/SynapticFour/Helix)

These public repositories are maintained by the same organisation and are designed to work together. Each repository keeps its own version and license. For details on roles, maturity, and how the components relate to one another, see [SUITE-OVERVIEW](https://github.com/SynapticFour/.github/blob/main/profile/SUITE-OVERVIEW.md).

## What this is not

- **Not a server.** Point `helix` at a target you already started.
- **Not HELIOS.** No signed audit trails, RO-Crate, PDF export, or reproducibility envelope.
- **Not GA4GH certification.** Green CI and a green `helix verify` are a technical signal.
- **Not a Ferrum production claim.** Ferrum is BUSL-1.1, on-prem, Rust, tested; there is no real clinical pilot (DIZ / genomDE). Demos and CI ≠ pilot ≠ production.
- **Not a paid SKU.** Apache-2.0 ambassador, same licence as HelixTest.

## Helix vs HELIOS

Helix answers *whether* a running system behaves. HELIOS answers *what* ran and *how* to reproduce it. Full gate: [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md). HELIOS CLI / PyPI: [`helios-audit`](https://github.com/SynapticFour/HELIOS) (Early Access). Never name this binary `helios`.

## Quick start

Needs a sibling [HelixTest](https://github.com/SynapticFour/HelixTest) checkout (path dependency `../HelixTest`) and a **running** target. Helix does not start Ferrum.

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
cd Helix
make prove
# start a target first, e.g. from Ferrum: make up
NO_COLOR=1 cargo run --bin helix -- verify http://127.0.0.1:8080
```

Install notes: [docs/INSTALL.md](docs/INSTALL.md). Rust: see `rust-toolchain` / CI (1.91.1). `make prove` is docs + `cargo test` against in-process mocks; it does not need Ferrum.

### Example: `helix verify <url>`

Against a local Ferrum-style gateway (`http://127.0.0.1:8080`) that exposes DRS (and typically WES/TES/TRS/htsget). Text mode, `NO_COLOR=1`. Discovery `found` is not a pass. Only the DRS block below is executed today.

```text
Helix verify — GA4GH discovery (not certification)
endpoint: http://127.0.0.1:8080
Helix tests behavior against the GA4GH spec, independent of implementation.
Ferrum is used as a reference target, not a dependency.

DRS      found   http://127.0.0.1:8080/ga4gh/drs/v1
WES      found   http://127.0.0.1:8080/ga4gh/wes/v1
TES      found   http://127.0.0.1:8080/ga4gh/tes/v1
TRS      found   http://127.0.0.1:8080/ga4gh/trs/v2
htsget   found   http://127.0.0.1:8080/ga4gh/htsget/v1

DRS (HelixTest checks; not certification)
  PASS  DRS object endpoint reachable
  PASS  DRS DrsObject OpenAPI + access_methods
  PASS  DRS checksum correctness
  PASS  DRS HTTP Range support
  PASS  DRS invalid object id returns 404
```

`--format json` (alias `--report json`) prints HelixTest `OverallReport` on stdout. Exit `0` if no executed FAIL; exit `1` if any FAIL, if DRS is missing, or on a usage/runtime error. Skip is never pass.

The broader HelixTest ladder (WES lifecycle, TES, TRS, Beacon, htsget, HMAC auth, …) still lives in HelixTest — [INVENTORY.md](INVENTORY.md).

### Other commands (scaffolds)

```bash
# Stage 3 — dummy HMAC only (test-fixtures/, NICHT FÜR PRODUKTION)
cargo run --bin helix -- security http://127.0.0.1:8080 \
  --hmac-secret-file test-fixtures/hmac/shared-secret.txt

# Stage 4 — 3 small GETs vs two origins; >10% worse warns, does not fail the process
cargo run --bin helix -- bench --baseline http://127.0.0.1:8080 --candidate http://127.0.0.1:8080
```

CLI contract: [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md). Roadmap (scope stages, not dates): [docs/HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md).

## Documentation

- [INVENTORY.md](INVENTORY.md) — what HelixTest actually runs
- [docs/HELIX_VISION.md](docs/HELIX_VISION.md) · [docs/HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md)
- [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md) · [docs/DECISIONS.md](docs/DECISIONS.md)
- [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md) · [docs/INSTALL.md](docs/INSTALL.md)
- [docs/FOR-EVALUATORS.md](docs/FOR-EVALUATORS.md) · [docs/PROVE.md](docs/PROVE.md)
- CI comment wrapper: [helix-action](https://github.com/SynapticFour/helix-action) (pilot; fail only on PASS → FAIL)

## Contributing

The useful first step is a question, not a large PR.

- [Open an issue](https://github.com/SynapticFour/Helix/issues) for bugs, missing coverage, “does this match the spec?”, or a first `helix verify` run. That is the current entry.
- [Discussions](https://github.com/SynapticFour/Helix/discussions) are the same bar for run reports once that tab is enabled on the repo.
- Small, reviewable PRs are welcome after an issue. See [CONTRIBUTING.md](CONTRIBUTING.md).

Do not add HELIOS-style evidence (RO-Crate, PDF, signatures) here. Do not claim Ferrum production or clinical deployments.

## License

Apache License 2.0 — see [LICENSE](LICENSE). Same licence as HelixTest. Not a Synaptic Four paid SKU.

**Synaptic Four** · [contact@synapticfour.com](mailto:contact@synapticfour.com) · [synapticfour.com](https://synapticfour.com) · Apache-2.0
