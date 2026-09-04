# Changelog

## Unreleased

- Initial public repository: HelixTest inventory and Ferrum-ecosystem ambassador scaffolding. The runnable CLI remains HelixTest v0.1.3.
- Stage 1 started: `helix verify <url>` discovers DRS → WES → TES → TRS → htsget under a gateway-style origin (no HelixTest checks yet).
- `helix verify` runs existing HelixTest DRS checks when DRS is discovered (generic mode, `strict_drs_checksums`). WES/TES/TRS/htsget checks are not wired yet.
- `docs/HELIX_VISION.md` — VERIFY pillar, HELIOS split, audiences, 12-month non-goals; HelixTest stays a separate git root (D1).
- `docs/HELIX_ROADMAP.md` — scope stages 0–5; Stage 0 started as docs, then exited (generic vs Ferrum decoupling).
- `docs/HELIX_VS_HELIOS.md` — feature-decision table and rule of thumb; ISO 15189 / AI Act stay HELIOS orientation, not Helix.
- `docs/DECISIONS.md` — HelixTest stays separate; Stage 0 fixture is in-tree HTTP, not unverified mock images.
- `docs/CLI_CONTRACT.md` + `VERSIONS.lock` — pin HelixTest v0.1.3; `helix verify` wraps that JSON.
