# Who Helix is for

Helix is the **independence** of [HelixTest](https://github.com/SynapticFour/HelixTest): a GA4GH conformance / security-behaviour / regression runner you point at a running target. **Apache-2.0. Not a product SKU. Not a server.**

HelixTest already exists (public repo, CI, Ferrum pin, SF-TR-2026-001 / SF-TR-2026-002). Helix does not invent that suite. HelixTest stays a separate git root ([DECISIONS.md](DECISIONS.md) D1). Positioning: [HELIX_VISION.md](HELIX_VISION.md). Pin: [VERSIONS.lock](../VERSIONS.lock) **v0.1.3**.

## Audience

Anyone implementing or buying a GA4GH API — including Ferrum, including competitors.

**Not for:** deploying Beacon/DRS/WES (Ferrum), issuing Passports (ga4gh-infra), pipeline reproducibility or signed evidence packs (HELIOS), clinical consent (Solum).

## HELIOS boundary

[HELIOS](https://github.com/SynapticFour/HELIOS) (`helios-audit`, Apache-2.0, Early Access) covers reproducibility, signed audit trails, RO-Crate/PDF export. Helix must not duplicate that surface. Gate: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md).

## Ferrum honesty

Ferrum is BUSL-1.1, on-premise, Rust, tested. It has **no** real clinical pilot deployment (DIZ / genomDE). Do not write “production deployment” copy for Ferrum or Helix. Demos and CI ≠ pilot ≠ production. HelixTest / Helix results are not GA4GH certification.

## Standalone (today)

The CLI binary is still built from HelixTest:

```bash
git clone https://github.com/SynapticFour/HelixTest.git && cd HelixTest
make prove
# Against a stack you started:
helixtest --all --mode ferrum
```

This repo: `make prove` checks docs. See [PROVE.md](PROVE.md), [INVENTORY.md](../INVENTORY.md), [HELIX_ROADMAP.md](HELIX_ROADMAP.md) (Stage 0 exited: generic vs Ferrum decoupling).
