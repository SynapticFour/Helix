# Decisions (Helix)

Recorded 2026-09-03. HelixTest Stage 0 decoupling (generic vs Ferrum) is implemented in HelixTest: no WES-name auto-switch, in-tree mock DRS in CI (D2).

## D1 — Keep HelixTest as its own git root

**Decision:** Do not merge HelixTest into Helix until the revisit criteria in [HELIX_VISION.md](HELIX_VISION.md) §7 are met.

**Why:** Ferrum / Lab Kit / ga4gh-infra pin HelixTest **v0.1.3**. SF-TR-2026-001/002 cite HelixTest. Helix is VERIFY docs + later wrapper, not a second suite.

**Consequence:** Stage 0 (generic vs Ferrum coupling) is implemented **in HelixTest**. Stage 1 `helix verify` may live in Helix and invoke the pinned `helixtest` binary. Ferrum `HELIXTEST_REF` stays until Stage 2 explicitly moves it.

## D2 — Non-Ferrum Stage 0 target

**Decision:** Stage 0 exit is an in-tree HTTP fixture (wiremock or equivalent in HelixTest CI), **not** `helixtest/docker/docker-compose.yml` (`ghcr.io/example/mock-*`) until those images are proven to exist and run.

**Why:** [INVENTORY.md](../INVENTORY.md) marked those images UNKLAR. A CI job that needs Ferrum GHCR is not a non-Ferrum proof.

## D3 — Report contract

**Decision:** Helix JSON is HelixTest JSON. Skips are not passes. No HELIOS fields (signatures, RO-Crate, PDF). See [CLI_CONTRACT.md](CLI_CONTRACT.md).

## D4 — HELIOS

**Decision:** Unchanged. [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) is the feature gate.
