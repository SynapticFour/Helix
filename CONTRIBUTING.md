# Contributing

Thank you for your interest in contributing to this project.

## How to contribute

- [Open an issue](https://github.com/SynapticFour/Helix/issues). A question about a `helix verify` run is enough; you do not need a patch first. [Discussions](https://github.com/SynapticFour/Helix/discussions) are the same bar once that tab is enabled.
- Open an issue to discuss significant changes before starting implementation.
- Use focused branches and keep pull requests small and reviewable.
- Add or update tests for behavior changes.
- Ensure local linting, formatting, and tests pass before opening a PR. Same gates as GitHub CI: `pre-commit install` (fmt + clippy `-D warnings` + `make prove`). Needs a sibling `HelixTest` checkout, like CI.
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
