# Dependency care (ambassador)

Dependabot and Renovate are **off by choice**, matching HelixTest / Ferrum-group ambassadors.

This repo **has** a Rust lockfile ([Cargo.lock](../Cargo.lock)). Tests and prove use `cargo test --locked --offline` after an explicit `make fetch` (`cargo fetch --locked`). Crate versions are lockfile checksums, not “latest on crates.io”. Helix does not fetch GA4GH specification files at verify time.

Also:

- HelixTest pin: [VERSIONS.lock](../VERSIONS.lock) (tag **v0.1.3**)
- GitHub **Dependency Review** on PRs (non-fatal)
- No Dependabot smoke job on `main`
- MSRV `rust-version` **1.88**; CI / `rust-toolchain.toml` **1.91.1**

Do not vendor HelixTest (D1 path dep). Independent verification: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md).
