# Helix roadmap reset

Helix is HelixTest becoming a standalone VERIFY CLI. This document records the post-Phase-C **roadmap reset**. It is not a new verification contract. It is not Phase D. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht. Vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: DEFERRED / CLOSED
drs.security.authorization = UNEVALUATED
```

Product: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Roadmap file: [HELIX_ROADMAP.md](HELIX_ROADMAP.md). Phase B: [PHASE_B_CLOSURE.md](PHASE_B_CLOSURE.md). Phase C: [PHASE_C_CLOSURE.md](PHASE_C_CLOSURE.md).

---

## 1. Reset date

**2026-09-08**

This reset follows Phase C **PASS WITH FINDINGS — CLOSED**. It does not start a new product phase.

---

## 2. Repository state examined

Inspected on this date in `Helix/` (git root `wip/drs-140-productization`).

| Fact | Evidence |
|------|----------|
| Branch | `wip/drs-140-productization` |
| HEAD commit | `8ab4b1f15df07e2ed1256a3a87975e7582029030` (`cli: first usable DRS 1.4.0 operator path`) |
| Working tree | Dirty: Phase B close, Phase C C1–C3, Phase C close, this reset. Not `origin/main`. |
| Package | `helix` `0.1.0`, `publish = false` (`Cargo.toml`) |
| HelixTest pin | `HELIXTEST_SHA=1baddfd3d75f01dc7c149074a785616fa014c725` (`VERSIONS.lock`), tag **v0.1.3**, path dependency `../HelixTest` |
| Binary | `helix` (`src/main.rs`): `verify`, `inspect`, `security`, `bench`, `compare`, `differential`, `matrix`, `standards` |
| SUPPORTED pack | **DRS 1.4.0 only** (`standards/registry.yaml`; `src/standards/support.rs`) |
| AVAILABLE, not SUPPORTED | DRS 1.5.0; WES 1.1.0 |
| CI | `.github/workflows/ci.yml` runs `make prove` on `main`/`master` PRs. This WIP branch is not that workflow until a PR. |
| Live independent DRS | Optional (`make verify-independent`). Not prove. C1–C3 sessions: Starter Kit `:4500` and Bento `:5000` **down**. No live JSON manufactured. `local/b12/` unrestamped. |
| Original roadmap | First committed `docs/HELIX_ROADMAP.md` in `cb089f9` (2026-09-03). Stage 0–5 + outside-ladder. |

Docs and code consulted: `HELIX_ROADMAP.md`, `HELIX_PRODUCT.md`, Phase B/C closes, C1–C3 reports, `OPERATOR_VERIFY.md`, `INDEPENDENT_DRS.md`, `CLI_CONTRACT.md`, `FOR-EVALUATORS.md`, `EXTERNAL_EVIDENCE.md`, B13/B14/B15, P2, `src/`, `tests/`, `standards/`, `schemas/`, `scripts/`, `Makefile`, `Cargo.toml`.

Do not treat `origin/main` as containing this productization. Do not treat uncommitted working-tree files as a tagged release.

---

## 3. Phase B status

**PASS WITH FINDINGS — formally closed** ([PHASE_B_CLOSURE.md](PHASE_B_CLOSURE.md)).

B1–B15 plus P1/P2 established fail-closed DRS 1.4.0 technical verification: pinned packs, SpecSource execution, checker provenance, target/fixture identity, derived claims, partial coverage, unsigned inspectable evidence, operator UX. B13 authorization remains **DEFERRED**. Unsigned consistent restamp remains the HELIOS boundary.

---

## 4. Phase C status

**PASS WITH FINDINGS — CLOSED** ([PHASE_C_CLOSURE.md](PHASE_C_CLOSURE.md)).

C1–C3 implemented **verify → inspect → differential** as a repeatable product path for two independent DRS 1.4.0 implementations, without ranking them. Live target execution was optional and was not exercised in those sessions. Offline behavioural tests are the implementation evidence. B13 was not reopened. Coverage identities were not changed.

---

## 5. Original roadmap evaluation

Original wording recovered from `cb089f9:docs/HELIX_ROADMAP.md` (2026-09-03). Later commits refined Stage 2 from **scores** to **stable-id regressions**; Stage 3/4 CLI scaffolds appeared on 2026-09-04. Phase B/C **overtook** several Stage 1 exit stories without exiting those stages on their original Ferrum-centric wording.

| Original objective | Original intent | Current reality | Status | Evidence | Recommendation |
| ------------------ | --------------- | --------------- | ------ | -------- | -------------- |
| Stage 0: generic HelixTest without Ferrum leakage | `--mode generic` must not auto-switch to Ferrum; non-Ferrum HTTP proof | Exited 2026-09-03. In-process DRS mock in HelixTest CI. `--start-compose` alias. | **DONE** | [HELIX_ROADMAP.md](HELIX_ROADMAP.md) Stage 0 status; HelixTest `generic_drs_mock`; Helix `Mode::Generic` (`src/adapter`) | Keep as completed foundation. Do not re-open as Helix product work. |
| Stage 0: honest VERIFY docs | Vision, inventory, not certification | README / TRUST / IDENTITY still forbid certification and HELIOS merge. | **DONE** | `README.md`, [TRUST.md](TRUST.md), [IDENTITY.md](IDENTITY.md) | Maintain honesty; do not treat docs as a new phase. |
| Stage 1: `helix` CLI wrapping HelixTest | Small CLI in this repo; HelixTest stays a separate git root | `helix` binary exists; path-depends on HelixTest crates. | **DONE** | `Cargo.toml`, `src/main.rs`, [DECISIONS.md](DECISIONS.md) D1 | Treat the CLI as the product. Do not merge HelixTest (D1). |
| Stage 1: `helix verify <url>` for DRS and WES | Map a gateway URL onto DRS and WES; skips are not passes | Unversioned `verify` runs DRS+WES when TESTABLE. TES/TRS/htsget discovery-only. | **DONE** | `src/verify.rs`, [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md), `tests/verify_drs.rs`, `tests/verify_wes.rs` | Keep default verify unversioned. Do not infer a GA4GH pack. |
| Stage 1 exit: Ferrum-local public artefact | Documented `helix verify` against Ferrum `make up` as the Stage 1 public proof | CLI can hit Ferrum if the operator starts it. The product identity is now **generic + fixture**, not Ferrum. No published binary. Ferrum remains an optional reference target. | **SUPERSEDED** | [HELIX_PRODUCT.md](HELIX_PRODUCT.md); `make prove` / `make verify-drs` use in-process mocks; `Cargo.toml` has no Ferrum crate | Do not use Ferrum-local as Helix’s public artefact. Optional live Ferrum stays `make test-live`. |
| Versioned DRS 1.4.0 verification architecture | Not in the 2026-09-03 ladder; arrived as B1–B15 | Pinned vendor packs, support contract, SpecSource, claims, coverage, evidence. Only DRS 1.4.0 is SUPPORTED. | **DONE** (supersedes “thin HelixTest wrap” as the Stage 1 depth) | [PHASE_B_CLOSURE.md](PHASE_B_CLOSURE.md), `standards/vendor/ga4gh.drs.1.4.0/`, `src/claims.rs`, `src/coverage.rs` | This **is** the verification product. Freeze identities unless an explicit new `coverage_id` is chosen later. |
| P1/P2 first usable operator path | Human product page + source-build DRS 1.4.0 workflow | `HELIX_PRODUCT.md`, `OPERATOR_VERIFY.md`, `--output`, `helix inspect`, `make verify-drs`. | **DONE** | [P2_FIRST_USABLE_RELEASE.md](P2_FIRST_USABLE_RELEASE.md), `tests/p2_first_usable_release.rs` | Keep as the smallest coherent product unit. |
| Independent two-target DRS path | Not named on the 2026-09-03 ladder; B9/B12 then Phase C | Repeatable `verify → inspect → differential`. Live stacks optional. | **DONE** | [PHASE_C_CLOSURE.md](PHASE_C_CLOSURE.md), [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md), `tests/c1_*.rs`–`c3_*.rs` | Do not expand the catalog to “explain” Starter Kit vs Bento. Comparison is not ranking. |
| Stage 2 original: Ferrum PR **scores** | Comment previous vs current X/Y service scores | Scoring as a product comment is rejected. `helix compare` uses stable check id. helix-action comments lead with regressions, not X/Y. | **SUPERSEDED** | Original `cb089f9` Stage 2; current Stage 2 text; [REGRESSION.md](REGRESSION.md); `src/compare.rs` | Do not restore scores, leaderboards, or coverage percentages. |
| Stage 2 remaining: Ferrum CI visibility | Reusable action; no false alarms; not a required Ferrum `main` check | Helix’s own CI proves fixtures on `main` PRs. `helix-action` is documented as a Ferrum **pilot** (`ci/helix-verify-pilot`). Stage 2 **not exited**. | **STILL VALID** | [HELIX_ROADMAP.md](HELIX_ROADMAP.md) Stage 2; `.github/workflows/ci.yml`; [CLI_CONTRACT.md](CLI_CONTRACT.md) | Optional operational integration for Ferrum. Not the Helix product identity. Not the next verification phase. |
| Stage 3: five Passport/OIDC/Crypt4GH cases on ga4gh-infra | Reproducible API behaviour vs Ferrum+infra or HMAC | `helix security` implements five dummy-HMAC HTTP invariants plus Crypt4GH layout against mocks. Live Ferrum+infra is documented, not CI default. Original Passport-on-DRS exit is **not** met. | **IN PROGRESS** (scaffold) | `src/security/`, [SECURITY_PROFILE.md](SECURITY_PROFILE.md), `tests/security_cli.rs` | Do not treat dummy HMAC as DRS authorization. Do not expand this module as a substitute for B13. |
| DRS authorization verification (B13) | Later security-adjacent; investigated in Phase B | `drs.security.authorization` is **UNEVALUATED**. Bento auth is a separate OIDC service. `helix verify` sends no credentials. | **DEFERRED** | [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md), `src/authorization.rs`, `tests/b13_authorization_boundary.rs` | **Do not reopen** until a genuine locally runnable auth-enabled implementation exists. |
| Stage 4: compare two Ferrum versions via Demo | Wall time + resource figure; stored artefacts | `helix bench` is a generic two-URL HTTP smoke (`http.drs.smoke.v1`). Not Demo. Not two Ferrum git SHAs. Warn-only. Not verification. | **IN PROGRESS** (engine) / original Demo exit **STILL VALID** as Ferrum-adjacent work | [BENCHMARKS.md](BENCHMARKS.md), `src/bench/`, `tests/bench_cli.rs` | Keep measurement **out of** `ga4gh_requirement`. Do not make bench the next Helix phase. |
| Stage 5: stranger can install | `INSTALL.md` actually installs `helix` or pinned HelixTest | Source-build docs exist (`INSTALL.md`, evaluator pack). Sibling HelixTest required. No GitHub release binary, no crates.io, no container. | **STILL VALID** | [INSTALL.md](INSTALL.md), [FOR-EVALUATORS.md](FOR-EVALUATORS.md), [OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md) | This is the largest remaining **product** gap after Phase C. It is process/release, not new checks. |
| Stage 5 exit: first voluntary external feedback | One invited/inbound try; not a bake-off | No such feedback is recorded in this tree. `PUBLIC_READINESS_AUDIT.md` still says not ready for a public announcement. No `CODE_OF_CONDUCT.md`. | **STILL VALID** | [PUBLIC_READINESS_AUDIT.md](PUBLIC_READINESS_AUDIT.md); missing CoC / issue templates | Do not announce this WIP branch as if it were `main`. |
| TES / TRS / htsget execution | Stage 1 service order; not the Stage 1 exit | Discovery only. Not executed. | **STILL VALID** | [DISCOVERY.md](DISCOVERY.md), [CLI_CONTRACT.md](CLI_CONTRACT.md) | Later standards work. Not next. |
| Versioned WES as SUPPORTED pack | Implied by Stage 1 service order once versioning existed | Unversioned WES checks run. Vendor WES 1.1.0 OpenAPI still `$ref`s `https://raw.githubusercontent.com/ga4gh-discovery/ga4gh-service-info/...` (line 604). Not SpecSource-ready. | **STILL VALID** (blocked) | [WES.md](WES.md), `standards/vendor/ga4gh.wes.1.1.0/workflow_execution_service.openapi.yaml` | Do not mark WES SUPPORTED until a local complete vendor tree exists. Unversioned WES already works. |
| DRS 1.4.0 catalog expansion | Not original Stage 1; often assumed as “next checks” | Partial coverage is the honest baseline. GetAccessURL etc. remain UNEVALUATED. Expanding changes `coverage_id`. | **STILL VALID** as later expansion; **SUPERSEDED** as the default next move | [COVERAGE.md](COVERAGE.md), [PHASE_C_PLAN.md](PHASE_C_PLAN.md) | Only with an explicit new coverage contract. More checks ≠ more product. |
| Additional independent DRS lineage | Not original; B9 reviewed two | Starter Kit and Bento already exist. A third clone of the same family adds little until the two-target path is adoptable. | **STILL VALID** later | [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md), [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md) | Not next. Do not count extra tags of the same image as a new implementation. |
| Absorb HelixTest into Helix | Explicitly **not** Stage 0–1 | D1: keep separate. Ferrum/Lab Kit/ga4gh-infra pin HelixTest v0.1.3. | **DEFERRED** | [DECISIONS.md](DECISIONS.md) D1, [HELIX_VISION.md](HELIX_VISION.md) §7 | Revisit only on the recorded criteria. Path dependency remains an adoption friction. |
| Helix Cloud / SLA / enterprise dashboard | Outside 12-month ladder | Unscheduled. No code path. | **NO LONGER DESIRED** (this horizon) | Original “Outside this ladder”; [HELIX_VISION.md](HELIX_VISION.md) §6 | Do not schedule. |
| Public ranking / bake-off without consent | Outside ladder; Stage 5 explicitly excludes it | Differential `ranking_semantics: absent`. Product docs forbid scores. | **NO LONGER DESIRED** | [DIFFERENTIAL.md](DIFFERENTIAL.md), [C3_DIFFERENTIAL_INTERPRETATION.md](C3_DIFFERENTIAL_INTERPRETATION.md) | Comparison, not ranking. Do not add leaderboards. |
| HELIOS features inside Helix | Explicitly out of every stage | No signatures, RO-Crate, or PDF on `helix-verification-v1`. | **NO LONGER DESIRED** (as Helix work) | [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md), [RUN_IDENTITY.md](RUN_IDENTITY.md) | Unsigned restamp is accepted. Signing stays HELIOS. |
| Sequential Stage 0→5 as the execution order | Do not start *n+1* until *n* exits | Phase B/C built a verification product **without** exiting Stages 2–5 on original wording. Treating unexited Stage 2–5 as “next” would mis-order the product. | **SUPERSEDED** (as the sequencing rule) | This file; Phase B/C closes | Follow the **current** roadmap below. Keep Stage 0–5 as history. |

---

## 6. Current maturity assessment

Do not conflate implemented with adopted.

| Capability | Implemented | Behaviorally tested | Operator usable | External-ready | Notes |
| ---------- | ----------: | ------------------: | --------------: | -------------: | ----- |
| Unversioned DRS + WES `helix verify` | yes | yes | yes (source build) | no | Fixture and live HTTP. Exit 0 is not VERIFIED. |
| DRS 1.4.0 versioned verification | yes | yes | yes (source build) | no | Partial coverage. SUPPORTED ≠ VERIFIED. Only this pack is SUPPORTED. |
| `helix inspect` / evidence standing | yes | yes | yes | no | Unsigned. Standing is per-binary. Not HELIOS. |
| Two-target independent DRS workflow | yes | yes (offline) | yes if operator starts stacks | no | Live Starter Kit/Bento not exercised in C1–C3 sessions. Not CI. |
| `helix differential` | yes | yes | yes | no | Derived comparison. Not a third `VerificationRun`. Not ranking. |
| `helix compare` regression | yes | yes | yes | no | Stable id. Used by helix-action contract. |
| `helix standards` provenance | yes | yes | yes | no | Does not fetch. AVAILABLE ≠ SUPPORTED. |
| `helix security` dummy HMAC | yes | yes (mocks) | yes as demo | no | Not DRS authorization. Not a pentest. |
| `helix bench` smoke | yes | yes | yes as measurement | no | Not verification. Ferrum Demo two-version exit not met. |
| `helix matrix` | partial | yes (harness) | labeling only | no | Public slots pending. Mocks are not independent evidence. |
| Versioned WES SUPPORTED pack | no | n/a | unversioned only | no | HTTPS `$ref` blocks honest SpecSource. |
| DRS authorization (B13) | deferred | tests keep it **closed** | no claim | no | UNEVALUATED. Do not reopen. |
| Published binary / tag / crates.io | no | n/a | source build only | no | `publish = false`. No CoC. WIP branch. |
| Ferrum `main` helix-action | pilot only | not in this repo’s CI | Ferrum-side | no | Stage 2 not exited. |

**External-ready** here means: a stranger can adopt from the **published default branch** without this WIP working tree, without unpublished productization docs, and without treating live Docker/QEMU as universally replayable. That bar is **not** met.

---

## 7. Current product synthesis

### What is Helix today?

Helix is a **source-built VERIFY CLI**. You point it at an HTTP origin (or the in-process fixture). It can run unversioned HelixTest DRS and WES checks, and it can run **bounded technical verification of GA4GH DRS 1.4.0** under a pinned pack, then retain unsigned `helix-verification-v1` evidence.

It is no longer primarily “a Ferrum CI comment bot waiting to happen.” That remains a possible integration. It is not the product.

It is not a new test platform. Check bodies still come from HelixTest. Helix owns selection, provenance, claims, coverage, evidence standing, and operator interpretation.

### What is the strongest demonstrated capability?

**Fail-closed DRS 1.4.0 technical verification** with inspectable evidence, plus **check-level comparison of two independent implementations** under the same contract — without ranking them and without pretending coverage is complete.

The in-process fixture proves the harness. Historical B12 JSON shows two real lineages were exercised once. Phase C made the workflow repeatable. Live current evidence still depends on stacks the operator starts.

### What is the smallest coherent product unit?

```text
source build (sibling HelixTest)
    → helix verify URL --standard drs --version 1.4.0 --output FILE
    → helix inspect FILE
```

That is P2 (`make verify-drs` without a live target). The second journey is the two-target path ([INDEPENDENT_DRS.md](INDEPENDENT_DRS.md)). Neither is a certificate.

### What is deliberately outside the product boundary?

- GA4GH certification and ranking
- Cryptographic signing, RO-Crate, PDF (HELIOS)
- Authorization verification until a real auth-enabled target exists (B13)
- Result cache, telemetry, upload, dashboards, Helix Cloud
- Ferrum as a Helix dependency or product identity
- Silent version substitution; AVAILABLE as SUPPORTED

### What remains the biggest gap for an external engineering team?

Not “missing GetAccessURL.” An external team cannot **confidently adopt this tree as the public product** because:

1. The verification product lives on `wip/drs-140-productization` plus uncommitted close docs, not as the published `main` baseline.
2. Build requires a **sibling HelixTest** checkout at an exact SHA.
3. There is no tagged source release and no binary.
4. Live independent evidence is optional, environment-sensitive (Docker/QEMU), and was not re-run in Phase C sessions.
5. Evidence is unsigned (accepted HELIOS split) and currentness is per-binary.
6. Stranger-facing hygiene from [PUBLIC_READINESS_AUDIT.md](PUBLIC_READINESS_AUDIT.md) / [OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md) is still incomplete (no `CODE_OF_CONDUCT.md`, no issue templates, checklist still says not ready to announce).

Those are adoption and honesty gaps. They are not a reason to reopen B13 or to expand the DRS catalog.

---

## 8. Candidate next-direction comparison

No candidate is started here. Qualitative values are relative to **this** repository.

| Candidate | User value | Evidence/value already present | New engineering required | Dependencies | Risk | Strategic fit |
| --------- | ---------- | ------------------------------ | ------------------------ | ------------ | ---- | ------------- |
| 1. External adoption / release | **High** — a stranger can use what already exists | P2 path, evaluator pack, `make prove`, product docs, Phase B/C contracts | **Medium** process (merge/PR, tag policy, CoC/templates, HelixTest clone story). Little new verify logic | Default-branch decision; D1 remains | **Medium** if announced off this WIP branch or as certification | **High** for the product that now exists |
| 2. DRS coverage expansion | Medium (explains some Starter Kit SKIP vs Bento PASS) | Partial coverage is honest and tested | **High** — new catalog rows **and** new `coverage_id` / claim meaning | Must not touch B13 ops; new fixtures | **High** (quietly changing what VERIFIED means) | Medium **later**; **Low** as the immediate next move |
| 3. Additional independent DRS implementation | Medium | Two lineages already reviewed | **High** ops (source/image, qemu/docker finding) | Operator environment | Medium (false “third implementation”) | **Low** until the two-target path is adoptable |
| 4. Authorization / security behaviour | High **if** a real auth target exists | Dummy HMAC `helix security`; B13 investigation | **High** and currently **blocked** | Usable auth-enabled implementation + black-box matrix | **High** (mock-as-evidence; credential handling) | **Low now.** B13 stays **DEFERRED / CLOSED** |
| 5. WES enablement (versioned SUPPORTED) | High for pipeline stacks | Unversioned WES already executes | **High** — localize `ga4gh-service-info` `$ref`, then full support-contract path | Vendor rewrite; must not fetch at runtime | **High** if SUPPORTED without local bytes | Medium later; **Low** until provenance is clean |
| 6. CI / GitHub integration | Medium for Ferrum; Low for a DRS author | Helix `make prove` CI; helix-action pilot documented | Medium (Ferrum politics, false-alarm budget) | Ferrum `main` consent | Medium (skip-as-green, infra flakes) | Medium as **optional** integration; not Helix identity |
| 7. Benchmark / regression infrastructure | Low for verification users | `helix bench` + `helix compare` already exist | Low to polish; **High** to meet original Demo/Ferrum-version exit | Same runner class; Demo pins | Claim contamination if bench is treated as verify | **Low** as next Helix phase |
| 8. Other (from this tree): HelixTest path-dep packaging | High for clone friction | D1 + `VERSIONS.lock` + `scripts/require-helixtest.sh` | Medium (without merging git histories) | D1 revisit criteria | Medium (citation/lockfile blast radius if merged) | Tied to candidate 1; not a verification expansion |

---

## 9. Revised roadmap

Five conceptual stages. Two are **complete**. None of the remaining stages is started by this file. There is **no Phase D**.

### Foundation — complete

- **Purpose:** Helix is a non-Ferrum VERIFY CLI with a fail-closed DRS 1.4.0 verification architecture.
- **Why it exists:** The original Stage 0–1 “thin wrap” was overtaken by SpecSource, support contracts, claims, coverage, and evidence.
- **Entry:** Historical (Stage 0 exit + Phase B).
- **Outcome:** Stage 0 exited; Phase B **PASS WITH FINDINGS**; P1/P2 usable fixture path.
- **Non-goals:** Certification, HELIOS, Ferrum as a crate, B13.

### Independent DRS path — complete

- **Purpose:** Same DRS 1.4.0 contract on two independent implementations the operator starts: verify → inspect → differential.
- **Why it exists:** B12 was a one-off lab capture. A user needed a product path, not more catalog rows.
- **Entry:** Phase B closed.
- **Outcome:** Phase C **PASS WITH FINDINGS — CLOSED**. Live stacks remain optional.
- **Non-goals:** Ranking, coverage expansion, restamping `local/b12/`, putting qemu DRS into `make prove`.

### External adoption — not started

- **Purpose:** Make the **existing** product the published baseline an external engineer can build and interpret.
- **Why it exists:** After Phase C, the largest gap is adoptability (default branch, source-release hygiene, sibling HelixTest, honesty about live evidence), not missing DRS operations.
- **Likely entry:** Explicit product decision to treat this DRS 1.4.0 product as the public baseline. Not automatic.
- **Desired outcome:** A stranger can follow INSTALL/evaluator docs against a published ref; early-stage classification retained; no certification language.
- **Non-goals:** crates.io while HelixTest is a path dep (unless D1 changes); binary/SaaS; announcing WIP as `main`; ranking; live Docker in prove.

### Verification expansion — later

- **Purpose:** Widen what Helix may claim, only as a **new** coverage/support contract.
- **Why it exists:** Partial DRS 1.4.0 coverage is honest. Some unevaluated operations are real product questions. They must not silently redefine current VERIFIED.
- **Likely entry:** After the adoption decision is made (either way), and only with an explicit new `coverage_id`.
- **Desired outcome:** Operators understand a new boundary; old artefacts remain interpretable as historical under the old contract.
- **Non-goals:** Quiet catalog growth; B13 via “one more check”; DRS 1.5.0 SUPPORTED as a drive-by.

### Operational integration — optional

- **Purpose:** Helix results in someone else’s CI (Ferrum first historically).
- **Why it exists:** Original Stage 2. Still valid for Ferrum. Not required for Helix to be a DRS verification product.
- **Likely entry:** Ferrum (or another implementer) wants id-level comments without false alarms.
- **Desired outcome:** Pilot evidence of boring comments; still not a required `main` gate until that is true.
- **Non-goals:** Scores; required merge blocker as the first step; public leaderboard.

### Broader standards — later

- **Purpose:** Versioned verification beyond DRS 1.4.0 (WES first among GA4GH siblings), plus B13 only if a real auth environment appears.
- **Why it exists:** Original service order and Stage 3 intent. The provenance architecture can extend, but WES is not SpecSource-ready and B13 is blocked on a target.
- **Likely entry:** Local complete vendor tree (WES); or a genuine auth-enabled implementation (B13).
- **Desired outcome:** AVAILABLE → SUPPORTED only after the same fail-closed path DRS 1.4.0 already uses.
- **Non-goals:** Marking WES SUPPORTED with an HTTPS `$ref`; dummy HMAC as authorization; TES/TRS/htsget as a shortcut.

---

## 10. Next product decision

**Should the existing DRS 1.4.0 verification product become the published baseline, or should Helix remain a productization branch and next spend capacity on verification expansion (coverage, WES) or Ferrum CI?**

That is the fork this reset exposes.

Repository evidence does **not** say “write more DRS checks tomorrow.” It says the architecture is coherent enough that the next choice is **adoption vs remaining internal**, not the next B-gate.

This file does **not** choose. It does not start External adoption. It does not start Verification expansion. The subsequent readiness record is [PUBLISHED_BASELINE_READINESS.md](PUBLISHED_BASELINE_READINESS.md).

---

## 11. Explicit deferred / out-of-scope items

| Item | Standing |
|------|----------|
| B13 authorization | **DEFERRED / CLOSED**. `drs.security.authorization` **UNEVALUATED**. Reopen only with a usable auth-enabled target/environment. |
| DRS 1.4.0 full coverage | Not established. Partial is the baseline. |
| Implementation ranking | Will not be added. Differential is comparison. |
| HELIOS signing / RO-Crate / PDF | Out of Helix. Unsigned restamp remains possible. |
| Ferrum as Helix identity | Reference target only. No clinical pilot. |
| Versioned WES SUPPORTED | Blocked on local vendor completeness. |
| Helix Cloud / SLA / dashboard | **NO LONGER DESIRED** this horizon. |
| Phase D | **Not started. Not named as committed work.** |
| Live Docker in `make prove` | Out of scope. Live independent verification stays optional. |
| Absorbing HelixTest | **DEFERRED** (D1). |

---

## 12. Validation performed

```text
./scripts/prove.sh
```

No live Docker targets. No manufactured live JSON. `local/b12/` not rewritten. HelixTest pins and DRS `execution_id` / `coverage_id` identities not altered. No new verification checks. No schema change in this task.

---

## Verdict

**ROADMAP RESET — COMPLETE**

The original Stage 0–5 ladder is preserved as history. The current roadmap describes the Helix that exists after Phase B and Phase C: a DRS 1.4.0 verification CLI with an independent two-target path. The next action is a **product decision**, not automatic implementation.

**Vertraue mir nicht. Vertraue dem Code.**
