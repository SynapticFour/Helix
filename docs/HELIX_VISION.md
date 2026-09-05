# Helix vision

**Synaptic Four builds the infrastructure. Helix proves it works.**

Helix is the VERIFY pillar next to BUILD (Ferrum) and PROTECT (Solum). It is not a new invention: it is the product-shaped name for work that already runs as [HelixTest](https://github.com/SynapticFour/HelixTest). [HELIOS](https://github.com/SynapticFour/HELIOS) stays a separate brand for reproducibility and audit evidence. It is not merged into Helix.

This document is a positioning note, not a capability claim. HelixTest results are a technical signal, not GA4GH certification. Ferrum has no real clinical pilot (DIZ / genomDE). Do not read “proves it works” as production-proven.

Sources: [INVENTORY.md](../INVENTORY.md), [IDENTITY.md](IDENTITY.md), [HELIX_ROADMAP.md](HELIX_ROADMAP.md), SF-TR-2026-001 / SF-TR-2026-002 (cite HelixTest, not this repo name).

---

## 1. One-sentence vision

Synaptic Four builds the infrastructure. Helix proves it works.

---

## 2. Why now

HelixTest already exists as one of five public GA4GH-stack repos (Ferrum, ga4gh-infra, Lab Kit, Demo, HelixTest). It is already in CI. Ferrum, Lab Kit, and ga4gh-infra pin git tag **v0.1.3** (`1832c043…`). SF-TR-2026-001 and SF-TR-2026-002 cite it (`@helixtest2026`).

That is evidence of demand and of first substance — not of a finished VERIFY product. The gap Helix is meant to close is naming and scope around that suite (conformance + security behaviour + benchmark/regression), not a green-field test framework.

---

## 3. Helix vs HELIOS

Canonical split for **new features:** [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md). HELIOS (`helios-audit`, Apache-2.0, Early Access) remains the reproducibility / signed-evidence tool. Helix does not absorb it.

| Question | Helix | HELIOS |
|----------|--------|--------|
| Does Helix run documented DRS/WES checks on a live HTTP origin? | Yes (HelixTest wrap; DRS 1.4.0 SUPPORTED within declared coverage, not VERIFIED) | No |
| Does Helix verify a named GA4GH release? | **No** (registry AVAILABLE only) | No |
| Does auth fail closed on dummy HMAC fixtures (401/403 class)? | Selected `helix security` cases; not a pentest | No |
| Did this version regress against a known fixture? | `helix compare` at stable id (not a score). `helix bench` is warn-only smoke | No |
| Is this pipeline run attestably reproducible? | No | Yes |
| Is there a signed audit trail / RO-Crate / PDF export of the run? | No | Yes |
| Does it orchestrate Ferrum or Solum? | No | No (file ingest / WES artefact ids only) |
| Is a green result official certification? | No | No |

Both are ambassadors today (not paid SKUs). They answer different questions on purpose.

---

## 4. Scope

**In:**

- **Conformance** — HelixTest heritage: DRS, WES, TES, TRS, Beacon, htsget, HMAC/Passport auth as implemented, plus opt-in Ferrum-africa / ferrum+infra modes. See [INVENTORY.md](../INVENTORY.md) for what the code actually runs.
- **Security behaviour** — fail-closed checks (missing/garbage token, wrong scope, dataset-gated htsget when configured). Not a threat model for HELIOS, not Solum consent.
- **Benchmark / regression** — repeatable fail-level, JSON reports, pins that Ferrum CI already consumes.

**Out (stays HELIOS):**

- Reproducibility of a scientific run
- Signed evidence, audit trails, RO-Crate, PDF export
- Wrapping Nextflow / Snakemake as an evidence envelope

**Also out:** shipping Beacon/DRS/WES (Ferrum), issuing Passports (ga4gh-infra), clinical data (Solum). Helix is a runner you point at a target you started. It is not a server.

---

## 5. Audiences (priority order)

1. **Ferrum as reference implementation** — first consumer today (`VERSIONS.lock`, NON-PILOT demo CI, optional ferrum+infra / pilot-auth). Helix exists so Ferrum can keep proving its own stack without Helix becoming a Ferrum-only tool.
2. **GA4GH community / implementers** — HelixTest already advertises generic mode and competitor-usable Apache-2.0. Profiles besides `ferrum.toml` exist (`generic.toml`, `bioresearch-assistant.toml`). This audience is real in the code; it is not yet a fielded design-partner programme.
3. **Later: external infrastructure operators as design partners** — DIZ / genomDE-class operators, if and when they exist. Not a current deployment. Do not write outreach as if they already run Helix or Ferrum in production.

Single-steward capacity: this order is scope, not a calendar. Execution order: [HELIX_ROADMAP.md](HELIX_ROADMAP.md) Stages 0–5.

---

## 6. Non-goals (next 12 months)

- No SaaS dashboard
- No Synaptic Four-hosted cloud for other people’s stacks
- No new standard (Helix consumes published GA4GH OpenAPI; it does not replace GA4GH)
- No certification product or “Helix-certified” mark (green CI remains a technical signal)

Also not in those 12 months: claiming Ferrum production deployments, absorbing HELIOS, or treating HelixTest JSON as legal evidence.

---

## 7. HelixTest: keep separate

**Decided (2026-09-03):** Keep HelixTest as the tagged, pinable CLI. Keep Helix as the VERIFY brand and docs (and later, a wrapper). Do not merge git histories for Stages 0–1. Detail: [DECISIONS.md](DECISIONS.md) D1.

Why not absorb now:

- Ferrum, Lab Kit, and ga4gh-infra pin **HelixTest v0.1.3** on every relevant CI path. Absorption is a lockfile and citation blast radius (SF-TR `@helixtest2026`, `helixtest-action`, `lab-kit conformance run`), not a docs rename.
- HelixTest already has a public identity as an ambassador. Helix currently has inventory and vision, not a second implementation. Merging now would put an unreleased brand on a tagged artefact that Ferrum PR CI depends on.
- Generic vs Ferrum coupling (Stage 0) is implemented **in HelixTest**, not by rewriting history.
- Two names can coexist the way HELIOS / `helios-audit` already do: brand Helix, binary `helixtest`, repo HelixTest until a deliberate redirect.

Why absorb later might still be right:

- VERIFY should not stay two public ambassadors that explain the same runner.
- Security-behaviour and benchmark/regression that do not fit the current `helixtest --all` ladder may belong in Helix as extra crates/binaries, with HelixTest remaining the conformance core — or the core can move once Helix has a release train of its own.

Revisit when at least one of these is true: Helix ships a capability HelixTest does not; Ferrum can bump `HELIXTEST_REF` in one `VERSIONS.lock` change; SF-TR successor text can point at Helix without silently rewriting 001/002. Until then Helix **depends on** HelixTest ([VERSIONS.lock](../VERSIONS.lock)). It does not vendor it.
