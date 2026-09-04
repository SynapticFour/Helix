# Helix

Helix is the independence of **[HelixTest](https://github.com/SynapticFour/HelixTest)** — already one of five public GA4GH-stack repos (Ferrum, ga4gh-infra, Lab Kit, Demo, HelixTest), already in CI, already cited in SF-TR-2026-001 / SF-TR-2026-002.

Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency.

This is not a new test platform. It is the VERIFY brand around HelixTest, aligned with the Ferrum ecosystem, **separate git root** ([docs/DECISIONS.md](docs/DECISIONS.md) D1).

**Scope:** conformance, security behaviour, benchmark/regression.  
**Not in scope:** reproducibility, signed audit trails, RO-Crate/PDF evidence — that is **[HELIOS](https://github.com/SynapticFour/HELIOS)** (`helios-audit` on PyPI).  
**Not a server.** Point a runner at a target you started.  
**Not GA4GH certification.** Green runs are a technical signal.  
**Not a Ferrum production claim.** Ferrum is BUSL-1.1, on-prem, Rust, tested; there is no real clinical pilot (DIZ / genomDE) to cite.

Apache-2.0 — same licence as HelixTest. Not a Synaptic Four paid SKU.

### Current tree

`helix verify <url>` lives in this repo and runs **HelixTest DRS checks** (HelixTest stays a **separate git root**, pin in [VERSIONS.lock](VERSIONS.lock)). Stage 1 is not exited until DRS **and WES** report against Ferrum local. This repo also has:

- [INVENTORY.md](INVENTORY.md) — what HelixTest actually covers, how it is invoked, Ferrum coupling, licence, gaps
- [docs/HELIX_VISION.md](docs/HELIX_VISION.md) — VERIFY pillar, HELIOS split, audiences, 12-month non-goals
- [docs/HELIX_ROADMAP.md](docs/HELIX_ROADMAP.md) — scope stages 0–5 (not calendar dates); Stage 0 exited (generic vs Ferrum decoupling)
- [docs/HELIX_VS_HELIOS.md](docs/HELIX_VS_HELIOS.md) — feature-decision gate (do not overlap HELIOS)
- [docs/DECISIONS.md](docs/DECISIONS.md) · [docs/CLI_CONTRACT.md](docs/CLI_CONTRACT.md)
- Ecosystem docs matching sibling ambassadors (`docs/IDENTITY.md`, `ECOSYSTEM.md`, `DEPENDENCY.md`, `PROVE.md`)

```bash
make prove   # docs + cargo test (discovery + DRS vs B1 mock; no Ferrum)
cargo run --bin helix -- verify http://127.0.0.1:8080 --format json
```

Needs a sibling `../HelixTest` checkout ([docs/INSTALL.md](docs/INSTALL.md)). HelixTest remains usable directly:

```bash
# any GA4GH DRS (example: in-process mock in HelixTest CI)
DRS_URL=http://127.0.0.1:$PORT helixtest --all --mode generic --only drs --profile ga4gh-drs --report json

# Ferrum as reference target (start Ferrum first: `make up`)
helixtest --all --mode ferrum --only drs
```

---

**Synaptic Four** · [contact@synapticfour.com](mailto:contact@synapticfour.com) · [synapticfour.com](https://synapticfour.com) · Apache-2.0
