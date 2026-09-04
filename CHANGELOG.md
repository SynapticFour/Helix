# Changelog

## Unreleased

- GitHub CI clippy (`-D warnings`, rustc 1.91.1) failed on `clippy::single_match` in `helix bench`; local `make prove` did not run clippy. Fixed with `if let`.
- Local build verification 2026-09-04 (this machine): HelixTest `cargo check --workspace --locked` exit 0; `make prove` exit 0 (53 tests passed, 1 ignored, live-stack crates excluded). Helix `cargo check --locked` exit 0; `make prove` exit 0 (30 tests passed). Compiler actually used: Homebrew **rustc 1.97.1**, not `rust-toolchain.toml` **1.91.1** (Homebrew `cargo` is first on PATH). Sibling HelixTest was local HEAD `29472d2c…` plus uncommitted files, not the VERSIONS.lock pin `1832c043…`. Full command log: `local/BUILD_VERIFICATION.md` (gitignored). This is a compile signal, not certification.
- Initial public repository: HelixTest inventory and Ferrum-ecosystem ambassador scaffolding. The runnable CLI remains HelixTest v0.1.3.
- Stage 1 started: `helix verify <url>` discovers DRS → WES → TES → TRS → htsget under a gateway-style origin (no HelixTest checks yet).
- `helix verify` runs existing HelixTest DRS checks when DRS is discovered (generic mode, `strict_drs_checksums`). WES/TES/TRS/htsget checks are not wired yet.
- `--format json` (alias `--report json`) emits HelixTest `OverallReport`. Terminal PASS/FAIL is colored on a TTY. Exit 0 if no FAIL, 1 otherwise. Skips are not passes.
- Stage 2 pilot: sibling repo `helix-action` (PR comment + fail only on PASS → FAIL). Ferrum `main` is unchanged; test branch is `ci/helix-verify-pilot`. HelixTest stays a separate git root (D1).
- Stage 3 started: `helix security <url>` — five black-box auth cases + Crypt4GH header check. Dummy secrets only (`test-fixtures/`, NICHT FÜR PRODUKTION). Not HELIOS.
- Stage 4 started: `helix bench --baseline <url> --candidate <url>` — 3 small GETs, wall time / optional RSS / error rate, percent diff. Default >10% worse is a warning; the process still exits 0. Not Demo hap.py, not GIAB, not HELIOS.
- README rewritten for a first-time GitHub visitor: one-sentence scope, early-stage DRS coverage (WES not executed yet), `helix verify` sample, HELIOS split, Issues/Discussions.
- `docs/HELIX_VISION.md` — VERIFY pillar, HELIOS split, audiences, 12-month non-goals; HelixTest stays a separate git root (D1).
- `docs/HELIX_ROADMAP.md` — scope stages 0–5; Stage 0 started as docs, then exited (generic vs Ferrum decoupling).
- `docs/HELIX_VS_HELIOS.md` — feature-decision table and rule of thumb; ISO 15189 / AI Act stay HELIOS orientation, not Helix.
- `docs/DECISIONS.md` — HelixTest stays separate; Stage 0 fixture is in-tree HTTP, not unverified mock images.
- `docs/CLI_CONTRACT.md` + `VERSIONS.lock` — pin HelixTest v0.1.3; `helix verify` wraps that JSON.
