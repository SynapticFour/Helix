# Helix roadmap

Scope stages, not calendar dates. Capacity is single-steward; a stage is done when its exit criterion is met, not when a month ends.

**Synaptic Four builds the infrastructure. Helix proves it works.** Positioning: [HELIX_VISION.md](HELIX_VISION.md). What the suite actually runs today: [INVENTORY.md](../INVENTORY.md). HELIOS is not on this ladder.

**Current position (2026-09-05):** Stage 0 is **exited**. Stage 1 **CLI** ships DRS and WES execution (`helix verify`); the Stage 1 **exit criterion** (recorded DRS+WES against Ferrum local) is **not** a public artefact in this repo. Stage 2 remains a **pilot only**: [helix-action](https://github.com/SynapticFour/helix-action). Not on Ferrum `main`. Stage 3 and 4 are **started, not exited**. DRS 1.4.0 is SUPPORTED for technical verification within declared coverage; that is not VERIFIED. External multi-implementation validation is **pending**. HelixTest stays a **separate git root** ([DECISIONS.md](DECISIONS.md) D1).

Stages are sequential. Do not start *n+1* until *n* has exited. Skipping a stage to chase visibility (5) or a dashboard is out of order.

---

## Stage 0 — Decoupling and docs

**Goal:** Helix (via HelixTest) is technically usable without Ferrum-specific behaviour leaking into generic runs. Docs name the VERIFY pillar honestly: [HELIX_VISION.md](HELIX_VISION.md), this file, README.

**Concrete result:**

- Coupling listed in [INVENTORY.md](../INVENTORY.md) §3 is either removed or gated so `--mode generic` does not auto-switch to Ferrum when WES `name` contains `"Ferrum"`, and generic htsget/auth paths do not require `ferrum_like` forks.
- `--start-ferrum` is either renamed to a generic compose start or documented as Ferrum-only and unused on the generic path.
- README states the vision sentence, current stage, that the live CLI is still HelixTest v0.1.3 until Stage 1, and that results are not certification.
- A recorded run of HelixTest against a **non-Ferrum** HTTP target with `--mode generic` (or equivalent) producing a JSON report. Target: in-tree fixture in HelixTest CI ([DECISIONS.md](DECISIONS.md) D2). Do not use `ghcr.io/example/mock-*` until those images are proven.

**Exit criterion:** HelixTest can run against a non-Ferrum endpoint. Evidence: one JSON report in-tree or in CI, command line + target described, no Ferrum image required for that job.

**Status:** Exited 2026-09-03. Proof: HelixTest `make prove` runs `generic_drs_mock` / `generic_drs_independence` against an in-process wiremock DRS (`helixtest/testing/mock_ga4gh_drs.rs`). Command: `helixtest --all --mode generic --only drs --profile ga4gh-drs --report json` with `DRS_URL` pointing at that fixture. Same five DRS checks as against Ferrum DRS. `--start-compose` is the generic alias; `--start-ferrum` remains as the historical name and is unused on the generic CI path. Opt-in Ferrum HTTP (`crypt4gh_ferrum_http`, `africa`, `infra`, `ferrum_like` htsget) stays behind `--mode ferrum*`.

**Not in this stage:**

- `helix verify` binary (Stage 1)
- Changing Ferrum PR CI or posting verification comments (Stage 2)
- New Passport/OIDC/Crypt4GH cases (Stage 3)
- Absorbing HelixTest into this repo (D1: keep separate)
- Using unverified `ghcr.io/example/mock-*` as the Stage 0 proof (D2)
- HELIOS features (signed evidence, RO-Crate, PDF)
- Any claim that generic mode already “just works” without the proof above (`generic.toml` exists; that is not the exit)

---

## Stage 1 — CLI core (`helix verify`)

**Goal:** Existing HelixTest checks behind a small Helix CLI. Service order: DRS → WES → TES → TRS → htsget. Not all surfaces at once (Beacon, E2E, africa, infra wait).

**Concrete result:**

- A `helix` (or `helix verify`) command in **this** repo, or a clearly versioned wrapper that invokes the pinned HelixTest CLI. Git-merging HelixTest is still not required.
- `helix verify <url>` maps a single gateway-style base URL onto DRS and WES (TES/TRS/htsget may be flags or follow-on in the same stage if cheap; they are not the exit).
- Terminal table + JSON report (same honesty as HelixTest: skips are not passes).

**Exit criterion:** `helix verify <url>` produces a terminal and JSON report for **at least DRS and WES** against **Ferrum local** (`make up` / demo stack). Documented command in README/INSTALL. Not certification; local Ferrum is the reference target, not a clinical site.

**Not in this stage:**

- Beacon, africa, ferrum+infra, Crypt4GH HTTP as required surfaces
- GitHub Action on Ferrum PRs (Stage 2)
- Replacing `helixtest` in Ferrum `VERSIONS.lock` (optional later; pin can stay HelixTest)
- HELIOS export formats
- SaaS, cloud, or a web UI

---

## Stage 2 — CI visibility

**Goal:** A GitHub Action that Ferrum PRs can run, posting a PR comment with verification **regressions at stable Helix id** (not an X/Y score).

Ferrum **already** clones HelixTest on every PR (`conformance.yml`, NON-PILOT, `HELIXTEST_SKIP_AUTH=true`, TES noop + stubs). This stage is **id-level comments + fewer false alarms**, not “HelixTest appears in Ferrum CI for the first time.”

**Concrete result:**

- Reusable action: sibling repo [`helix-action`](https://github.com/SynapticFour/helix-action) (composite; parallel to `helixtest-action`). Ferrum pilot: branch `ci/helix-verify-pilot` only — not `main`.
- PR comment leads with **new regressions**, then **fixed failures**, then **existing failures** (`VerificationRun` / `helix compare`; skips are not passes). Job fails only on `NEW_FAIL` (PASS → FAIL/ERROR at stable id) vs last successful run of that workflow, or on explicit runtime errors. Known failures, skips, bench warnings, and identified infra do not fail the job.
- Do not make Helix a required Ferrum check on `main` until false-alarm rate is known. Stage 2 is **not exited**.

**Exit criterion:** Runs reliably in Ferrum’s own CI with **no false alarms** — meaning: no red X caused by Helix infra flakes, skip-as-green, or scoring stubs as real compute. A week of PR runs (or equivalent dispatch sample) without a known-bad comment is enough evidence; not a statistical SLA.

**Not in this stage:**

- Required status check that can block Ferrum merges (only after the comment path is boring)
- Public leaderboard or comparison of other implementations
- Security-module expansion (Stage 3) as a PR gate
- Nightly Passport co-deploy as a PR requirement (Ferrum’s `helixtest-ferrum-infra.yml` is dispatch-only today)

---

## Stage 3 — Security module

**Goal:** Behaviour tests for OIDC / Passport / Crypt4GH, on top of ga4gh-infra — as **API behaviour**, not HELIOS evidence.

HelixTest already has HMAC JWT fixtures, `--mode ferrum+infra` Passport-on-DRS, and env-gated Crypt4GH HTTP. This stage makes a **documented, reproducible set of at least five cases**, not a from-scratch auth product.

**Concrete result:**

- A named module or `--only` surface (docs + CLI) with five cases, each valid/invalid documented:
  1. valid token / Passport grants access
  2. invalid / garbage bearer rejected
  3. expired token rejected
  4. wrong scope denied
  5. cross-service (e.g. Passport issued by broker accepted on Ferrum DRS — the existing infra check)
- How to run: Ferrum `make up-pilot-local` + ga4gh-infra, or the HMAC path where Passports are not up. Record which path was used. Reproducible = same command, same pin, same JSON names.

**Exit criterion:** Those five documented cases run reproducibly (command + fixture + expected HTTP class). Not a pentest, not a GA4GH Passport certification, not Solum consent.

**Status:** Started 2026-09-04 in Helix (`helix security <url>`). Security Behavior Profile: five documented HTTP invariants (`HLX-AUTH-010`–`014`). Crypt4GH protocol layout after those cases (`HLX-AUTH-050` / `053` / `054`; [CRYPT4GH.md](CRYPT4GH.md)). CI uses an in-process mock and dummy HMAC in `test-fixtures/` (NICHT FÜR PRODUKTION), including **negative** mocks that violate one HTTP invariant each, and negative Crypt4GH envelopes. Live Ferrum `make up-pilot-local` / HMAC-on path is documented, not the CI default. Stage 3 is **not exited**. Not a pentest, not a security audit, not certification. Crypt4GH pass is not “secure”.

**Not in this stage:**

- Signed audit trails or RO-Crate of the auth dance (HELIOS)
- Benchmark/perf (Stage 4)
- External design partners (Stage 5)
- Claiming Ferrum is production-hardened because these five pass

---

## Stage 4 — Benchmark module

**Goal:** Dock to [Ferrum-GA4GH-Demo](https://github.com/SynapticFour/Ferrum-GA4GH-Demo) for **performance regression between Ferrum versions**. Demo is a GIAB-slice smoke, not HelixTest conformance and not a publication benchmark (Demo’s own docs).

**Concrete result:**

- A Helix (or documented HelixTest-adjacent) command that records wall time and at least one resource figure (CPU and/or RSS, or compose stats) for a fixed Demo scenario.
- Comparison of **two Ferrum git tags/SHAs** on the same runner class, output as a small table (not a marketing chart).

**Exit criterion:** Two consecutive Ferrum versions can be compared objectively (runtime, resources) from stored artefacts. Same machine class, same Demo pins otherwise. Not clinical throughput, not “production proven.”

**Status:** Started 2026-09-04 in Helix (`helix bench --baseline <url> --candidate <url>`). Repeatable engine plus distribution analysis ([BENCHMARKS.md](BENCHMARKS.md)): warmup + measured runs; compare median / p95 where available / error-rate / optional RSS — not a single wall-clock sample. Measurement, warning, and regression are separate. A warning is human inspection, not “incorrect” and not a verification failure. Fixed workload **`http.drs.smoke.v1`**. Default **>10% worse = warning**. Different OS/arch/runtime is marked incomparable. The CLI and helix-action comment **do not fail the job** on that warning. Stage 4 is **not exited**. Sample percentiles are not a significance test.

**Not in this stage:**

- Ranking other vendors
- Genome-wide GIAB concordance
- Helix Cloud
- Using HELIOS PDF as the benchmark report

---

## Stage 5 — External visibility (careful)

**Goal:** Helix is usable by someone outside Synaptic Four who chose to try it. Audience order remains Ferrum → GA4GH implementers → later operators ([HELIX_VISION.md](HELIX_VISION.md) §5).

**Concrete result:**

- Install/docs a stranger can follow (`docs/INSTALL.md` actually installs `helix verify` or pinned HelixTest).
- One invited or inbound try — not a press blast, not a public bake-off.

**Exit criterion:** First **voluntary** feedback from a person or organisation outside the team. That is the exit. A public comparison report of other implementations is **not** the exit and is **not** in this stage (only after several successful individual feedbacks; see outreach/Teil C when that pack exists).

**Not in this stage:**

- Public ranking or “Helix vs X” without the other party’s consent
- Certification language
- DIZ/genomDE production claims
- SaaS onboarding

---

## Outside this ladder (and outside a 12-month horizon)

Mention only after Stages 0–5 have exited **and** there is real external demand:

- Helix Cloud / Synaptic Four-hosted running of other people’s stacks
- SLA
- Enterprise dashboard
- Public ranking of other implementations without their consent

These are not Stage 6. They are not scheduled. HELIOS remains separate even if they ever happen.
