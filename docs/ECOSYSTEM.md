# Synaptic Four — this repo in the portfolio

Four **products**, two free **ambassadors**, Ferrum **companions**, and **proof** repos. Glue is GA4GH; Solum extends into clinical data. **Not a bundle SKU.** Canonical public map: [Ferrum PORTFOLIO.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/PORTFOLIO.md).

**You are here:** [Helix](https://github.com/SynapticFour/Helix) — independence of HelixTest. Conformance / security-behaviour / regression. Not a product SKU. Not a server. Not HELIOS.

The **running** CLI today is still [HelixTest](https://github.com/SynapticFour/HelixTest). HelixTest stays a **separate git root** ([DECISIONS.md](DECISIONS.md) D1). This repo holds vision, inventory, VERIFY positioning, and a HelixTest pin ([VERSIONS.lock](../VERSIONS.lock)). HELIOS is not dissolved into Helix.

## Repositories

| Kind | Repository | Role | License |
|------|------------|------|---------|
| Ambassador (this) | **Helix** | Independence of HelixTest | Apache-2.0 |
| Ambassador | [HelixTest](https://github.com/SynapticFour/HelixTest) | Conformance CLI (`helixtest`) | Apache-2.0 |
| Ambassador | [HELIOS](https://github.com/SynapticFour/HELIOS) | Pipeline audit evidence | Apache-2.0 |
| Product | [Ferrum](https://github.com/SynapticFour/Ferrum) | GA4GH data/compute | BUSL-1.1 |
| Product | [ga4gh-infra](https://github.com/SynapticFour/ga4gh-infra) | Identity plane | Apache-2.0 |
| With Ferrum | [Ferrum-Lab-Kit](https://github.com/SynapticFour/Ferrum-Lab-Kit) | Subset install | BUSL-1.1 |
| Proof | [Ferrum-GA4GH-Demo](https://github.com/SynapticFour/Ferrum-GA4GH-Demo) | Local `./run` smoke | Apache-2.0 |

## Ownership boundaries

| Layer | Owner | Notes |
|--------|--------|--------|
| Identity | **ga4gh-infra** | Broker, visas, DUO, ADS, service registry |
| Data/compute | **Ferrum** | DRS, WES/TES, TRS, Beacon; built-in passports in standalone mode |
| Deployment | **Ferrum-Lab-Kit** | Selective GA4GH surfaces for labs; does not fork Ferrum |
| Demo/benchmark | **Ferrum-GA4GH-Demo** | Reproducible GIAB smoke; optional `--with-infra` |
| Conformance / regression | **Helix** (brand) / **HelixTest** (tagged CLI) | Validates implementations; does not ship GA4GH services. Git roots stay separate (D1). |
| Reproducibility / evidence | **HELIOS** | Signed trails, RO-Crate/PDF — not Helix. Decision table: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md). |

Helix/HelixTest **validate** implementations. Ferrum runs HelixTest in CI. Pin: Ferrum `VERSIONS.lock` HelixTest **v0.1.3**.

## Default co-deploy ports

Same as HelixTest / Ferrum `docs/ECOSYSTEM.md`:

| Service | Standalone Ferrum | Co-deploy (demo / lab) |
|---------|-------------------|-------------------------|
| Ferrum gateway | 8080 | 18080 (demo) or 8080 (lab) |
| AAI broker | — | 8180 |
| Visa registry | — | 8181 |
| DUO | — | 8182 |
| Service registry | — | 8183 |
| ADS | — | 8190 |
| mock-idp | — | 9100 |

## Local lifecycle

Repos that run a local Docker stack share `up` / `down` / `destroy`. Helix does **not** start servers (same as HelixTest).

```bash
# Target (example)
cd ../Ferrum && make up

# Suite (today)
helixtest --all --mode ferrum
helixtest --all --mode ferrum+infra --profile ferrum-infra
```

## CI

GitHub Actions on this repo run `make prove` (docs) plus secret-scan. Live Ferrum proof stays in HelixTest and Ferrum workflows until the CLI lives here. Dependabot/Renovate off — [DEPENDENCY.md](DEPENDENCY.md).
