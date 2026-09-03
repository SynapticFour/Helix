# Dependency care (ambassador)

Dependabot and Renovate are **off by choice**, matching HelixTest / Ferrum-group ambassadors.

Until this repo contains a lockfile:

- Docs-only `make prove`
- GitHub **Dependency Review** on PRs (non-fatal until a graph exists)
- No Dependabot smoke job on `main`

When the HelixTest CLI moves here: commit `Cargo.lock`; MSRV and `rust-toolchain.toml` should match Ferrum / Lab Kit / ga4gh-infra / HelixTest (**toolchain 1.91.1**, HelixTest `rust-version` **1.88**).
