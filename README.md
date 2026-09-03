# Helix

Helix is the independence of **[HelixTest](https://github.com/SynapticFour/HelixTest)** — already one of five public GA4GH-stack repos (Ferrum, ga4gh-infra, Lab Kit, Demo, HelixTest), already in CI, already cited in SF-TR-2026-001 / SF-TR-2026-002.

This is not a new test platform. It is HelixTest becoming its own product-shaped repo, aligned with the Ferrum ecosystem, still a separate git root.

**Scope:** conformance, security behaviour, benchmark/regression.  
**Not in scope:** reproducibility, signed audit trails, RO-Crate/PDF evidence — that is **[HELIOS](https://github.com/SynapticFour/HELIOS)** (`helios-audit` on PyPI).  
**Not a server.** Point a runner at a target you started.  
**Not GA4GH certification.** Green runs are a technical signal.  
**Not a Ferrum production claim.** Ferrum is BUSL-1.1, on-prem, Rust, tested; there is no real clinical pilot (DIZ / genomDE) to cite.

Apache-2.0 — same licence as HelixTest. Not a Synaptic Four paid SKU.

### Current tree

The running CLI is still **HelixTest** (tag **v0.1.3**, SHA `1832c043…` as pinned by Ferrum / Lab Kit / ga4gh-infra). This repo starts with:

- [INVENTORY.md](INVENTORY.md) — what HelixTest actually covers, how it is invoked, Ferrum coupling, licence, gaps
- Ecosystem docs matching sibling ambassadors (`docs/IDENTITY.md`, `ECOSYSTEM.md`, `DEPENDENCY.md`, `PROVE.md`)

```bash
make prove   # docs integrity; no live stack
```

Live suite (still HelixTest):

```bash
# start a target first, e.g. Ferrum `make up`
helixtest --all --mode ferrum
```

---

**Synaptic Four** · [contact@synapticfour.com](mailto:contact@synapticfour.com) · [synapticfour.com](https://synapticfour.com) · Apache-2.0
