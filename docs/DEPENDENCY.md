# Dependency care (ambassador)

Dependabot and Renovate are **off by choice**, matching HelixTest / Ferrum-group ambassadors.

Until this repo contains a Rust lockfile:

- Docs-only `make prove`
- HelixTest pin: [VERSIONS.lock](../VERSIONS.lock) (tag **v0.1.3**)
- GitHub **Dependency Review** on PRs (non-fatal until a graph exists)
- No Dependabot smoke job on `main`

Stage 1 wrapper (if any) uses that pin. Do not vendor HelixTest. If a HelixTest CLI is ever moved here: commit `Cargo.lock`; MSRV and `rust-toolchain.toml` should match Ferrum / Lab Kit / ga4gh-infra / HelixTest (**toolchain 1.91.1**, HelixTest `rust-version` **1.88**).
