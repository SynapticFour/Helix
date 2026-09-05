# Contributing

Thank you for your interest in contributing to this project.

## How to contribute

- [Open an issue](https://github.com/SynapticFour/Helix/issues). Include command, Helix commit, HelixTest SHA, and stdout/stderr. You do not need a patch first.
- Open an issue to discuss significant changes before starting implementation.
- Use focused branches and keep pull requests small and reviewable.
- Add or update tests for behavior changes.
- Ensure local linting, formatting, and tests pass before opening a PR. Same gates as GitHub CI: `pre-commit install` (fmt + clippy `-D warnings` + `make prove` + `make independent-verify`). First time: `make fetch` (`Cargo.lock`; explicit network). Needs a sibling `HelixTest` checkout, like CI. `make prove` uses in-process fixtures ([docs/FIXTURES.md](docs/FIXTURES.md)) and `cargo test --locked --offline`; it does not need Ferrum. Do not `#[ignore]` those tests. Live verify against a stack you started is `make test-live`, not prove. Clone-and-run: [docs/INDEPENDENT_VERIFICATION.md](docs/INDEPENDENT_VERIFICATION.md).
- Do not add HELIOS-style evidence/RO-Crate/signed-export features here. Gate: [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md). Trust-model rules are encoded in [docs/ARCHITECTURE_GUARDRAILS.md](docs/ARCHITECTURE_GUARDRAILS.md), schemas, `src/guardrails.rs`, and `tests/guardrails.rs`. Do not weaken those tests to match an implementation.
- Do not claim Ferrum production or clinical pilot deployments.

## Pull request checklist

- Clear problem statement and motivation
- Tests added or updated (when code exists)
- Documentation updated where relevant
- No unrelated refactors bundled in the same PR
- Honesty: demos ≠ pilots ≠ production; Helix results ≠ GA4GH certification
- Normative mappings and SUPPORTED packs: follow [docs/STANDARDS_REGISTRY.md](docs/STANDARDS_REGISTRY.md) §4 and §10.1. There is no GA4GH-appointed board. Do not mark a check `normative` without the TRACEABILITY §7 chain. Checkout HelixTest at `HELIXTEST_SHA` in [VERSIONS.lock](VERSIONS.lock) or CI and your tree will diverge (`scripts/require-helixtest.sh` warns).

## Code review expectations

We value precise, respectful, and actionable feedback. Please keep discussions technical and reproducible.

## License

By contributing, you agree that your contributions are licensed under this repository's license (Apache-2.0).

New first-party Rust files (when added) start with `// SPDX-License-Identifier: Apache-2.0`.
