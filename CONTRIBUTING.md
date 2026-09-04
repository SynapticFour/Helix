# Contributing

Thank you for your interest in contributing to this project.

## How to contribute

- [Open an issue](https://github.com/SynapticFour/Helix/issues). Include command, Helix commit, HelixTest SHA, and stdout/stderr. You do not need a patch first.
- Open an issue to discuss significant changes before starting implementation.
- Use focused branches and keep pull requests small and reviewable.
- Add or update tests for behavior changes.
- Ensure local linting, formatting, and tests pass before opening a PR. Same gates as GitHub CI: `pre-commit install` (fmt + clippy `-D warnings` + `make prove`). Needs a sibling `HelixTest` checkout, like CI. `make prove` uses in-process fixtures ([docs/FIXTURES.md](docs/FIXTURES.md)); it does not need Ferrum. Do not `#[ignore]` those tests. Live verify against a stack you started is `make test-live`, not prove.
- Do not add HELIOS-style evidence/RO-Crate/signed-export features here. Gate: [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md).
- Do not claim Ferrum production or clinical pilot deployments.

## Pull request checklist

- Clear problem statement and motivation
- Tests added or updated (when code exists)
- Documentation updated where relevant
- No unrelated refactors bundled in the same PR
- Honesty: demos ≠ pilots ≠ production; Helix results ≠ GA4GH certification

## Code review expectations

We value precise, respectful, and actionable feedback. Please keep discussions technical and reproducible.

## License

By contributing, you agree that your contributions are licensed under this repository's license (Apache-2.0).

New first-party Rust files (when added) start with `// SPDX-License-Identifier: Apache-2.0`.
