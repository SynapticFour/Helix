# Who Helix is for

Helix is the **independence** of [HelixTest](https://github.com/SynapticFour/HelixTest): a command-line runner you point at a running GA4GH HTTP target. **Apache-2.0. Not a product SKU. Not a server.**

HelixTest already exists (public repo, CI, pin **v0.1.3**). Helix does not invent that suite. HelixTest stays a separate git root ([DECISIONS.md](DECISIONS.md) D1). Positioning: [HELIX_VISION.md](HELIX_VISION.md). Pin: [VERSIONS.lock](../VERSIONS.lock).

## Who maintains and reviews

One steward (Synaptic Four). There is no GA4GH-appointed mapping board and no CODEOWNERS file in this tree. A check becomes `normative` only under [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) §10.1. Stewardship is not evidence that a result is correct ([TRUST.md](TRUST.md)).

## Audience

Anyone implementing or buying a GA4GH API — including Ferrum, including competitors.

**Not for:** deploying Beacon/DRS/WES (Ferrum), issuing Passports (ga4gh-infra), pipeline reproducibility or signed evidence packs (HELIOS), clinical consent (Solum).

## HELIOS boundary

[HELIOS](https://github.com/SynapticFour/HELIOS) (`helios-audit`, Apache-2.0, Early Access) covers reproducibility, signed audit trails, RO-Crate/PDF export. Helix must not duplicate that surface. Helix run identity (so two verify JSON files can be compared) is not HELIOS evidence ([RUN_IDENTITY.md](RUN_IDENTITY.md), [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

## Ferrum honesty

Ferrum is BUSL-1.1, on-premise, Rust, tested. It has **no** real clinical pilot deployment with German hospital data-integration centres (DIZ) or the genomDE programme. Do not write “production deployment” copy for Ferrum or Helix. Demos and CI ≠ pilot ≠ production. HelixTest / Helix results are not GA4GH certification.

## Standalone (today)

Ferrum CI still clones the tagged HelixTest CLI (**v0.1.3**). This repo’s `helix` binary wraps HelixTest **DRS and WES** checks (`helix verify`) and also ships `helix security`, `helix bench`, `helix compare`, `helix matrix`, and `helix standards`. HelixTest stays its own git root. `make prove` here is docs greps plus `cargo test --locked --offline --all-targets` after `make fetch`. GitHub CI also runs clippy `-D warnings` on **1.91.1**. Independent verification: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md). Evaluator briefing: [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

```bash
# Tagged runner (what Ferrum VERSIONS.lock clones)
git clone https://github.com/SynapticFour/HelixTest.git && cd HelixTest
git checkout v0.1.3
make prove
helixtest --all --mode ferrum

# This repo (sibling HelixTest at VERSIONS.lock SHA — see docs/INSTALL.md)
cd Helix && make fetch && make prove && make verify-fixture

# Optional live origin you started (not prove):
# cargo run --bin helix -- verify http://127.0.0.1:8080
```

See [PROVE.md](PROVE.md), [INVENTORY.md](../INVENTORY.md), [HELIX_ROADMAP.md](HELIX_ROADMAP.md) (Stage 0 exited: generic vs Ferrum decoupling).
