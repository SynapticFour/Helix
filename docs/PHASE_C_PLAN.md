# Phase C plan

Helix is HelixTest becoming a standalone VERIFY CLI. This document chooses what Phase C should accomplish after Phase B closed **PASS WITH FINDINGS**. It is not an implementation prompt. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
authorization = UNEVALUATED
```

Product: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Phase B close: [PHASE_B_CLOSURE.md](PHASE_B_CLOSURE.md). Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). Independent live notes: [EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md).

---

## Executive summary

**Phase C should make independent DRS 1.4.0 verification as a repeatable product path.**

An operator who starts the two already-reviewed independent DRS implementations (GA4GH Starter Kit and Bento) should be able to run the **same** Phase B contract, retain **current** `helix-verification-v1` files, inspect standing, and interpret check-level differences with `helix differential` — without Helix authors, without ranking, and without expanding the DRS catalog.

That is the next thing a real user cannot honestly do today. P2 made the in-process fixture usable. B12 proved two real implementations once, as gitignored historical JSON. Phase C turns that into a first-class workflow.

Do **not** start Phase C with more DRS operations, WES SUPPORTED, dummy-HMAC security expansion, or a third implementation of the same architectural family.

---

## Current baseline

Inspected 2026-09-08. Helix `wip/drs-140-productization` at `8ab4b1f15df07e2ed1256a3a87975e7582029030` plus uncommitted Phase B closure docs. HelixTest pin `1baddfd3d75f01dc7c149074a785616fa014c725`.

### What a user can actually do today

- Build Helix from source (sibling HelixTest required). No crate, binary, or container.
- `make verify-drs` / `helix verify URL --standard drs --version 1.4.0 --output FILE` then `helix inspect`.
- Unversioned `helix verify URL` for DRS+WES HelixTest checks (not a named-release claim).
- `helix compare`, `helix differential`, `helix standards list|show|validate`, `helix security` (dummy HMAC), `helix bench` (smoke measurement), `helix matrix` (pending slots).

### What Helix can actually verify today

Versioned technical verification of **GA4GH DRS 1.4.0** inside a declared partial coverage boundary. Only that pack is SUPPORTED. DRS 1.5.0 and WES 1.1.0 are AVAILABLE, not SUPPORTED. Default verify does not select a pack.

### What evidence Helix produces

Unsigned `helix-verification-v1`. Claims, coverage, and `verified_version` are derived. `helix inspect` classifies `current_verifier_evidence` vs `historical_observation`. Not HELIOS.

### What VERIFIED means / does not mean

**Means:** Helix DRS 1.4.0 support-contract predicates held for that target and fixture (selected supported pack, integrity, checker, SpecSource, catalog completeness, claim join, coverage rules).

**Does not mean:** full DRS, authorization, security, certification, reproducibility, or that every OpenAPI operation was tested.

### Partial / deferred / scaffolded / experimental

| Area | State |
|-------|--------|
| DRS 1.4.0 coverage | **Partial.** Unevaluated: service-info op, bulk, GetAccessURL, OPTIONS, Passport POST, authorization, other checksum types, bundles, TLS-as-DRS, client timeouts. |
| Independent DRS | Two local lineages in gitignored B12 JSON (**historical**). Not CI. `helix matrix` pending. |
| Authorization | **DEFERRED** (B13). |
| WES | Unversioned checks **executable**. Versioned WES **not** SpecSource-ready (`$ref` to `https://raw.githubusercontent.com/ga4gh-discovery/ga4gh-service-info/...`). |
| `helix security` | Executable dummy-HMAC HTTP profile. Not DRS authorization. Different JSON shape (`OverallReport`). |
| `helix bench` | Executable smoke measurement. Out of verification (`helix.bench.as_verification` is OUT_OF_SCOPE). |
| CI | `make prove` on `main` PRs. This productization branch is not that workflow until a PR. `helix-action` is a Ferrum **pilot**, Stage 2 not exited. |
| Ferrum | Reference target only. Not a Helix crate. Stage 1 Ferrum-local artefact still not a public artefact. |

---

## Roadmap reconciliation

The old [HELIX_ROADMAP.md](HELIX_ROADMAP.md) is a Ferrum-centric Stage 0–5 ladder. Phase B was a DRS verification productization sequence that **overtook** several stage exit stories without exiting those stages on their original wording.

| Old item | Class | Note |
|---------|--------|------|
| Stage 0 decoupling / non-Ferrum mock | `DONE` | Exited 2026-09-03. |
| Stage 1 `helix verify` DRS+WES CLI | `NEEDS_REDEFINITION` | CLI exists and is the product. Original exit was Ferrum-local public artefact — **not met**. Treat Helix CLI as done; Ferrum artefact as later/optional. |
| Stage 2 helix-action on Ferrum PRs | `STILL_VALID` | Pilot only. Not Phase C. |
| Stage 3 security module (Passport/OIDC/Crypt4GH) | `NEEDS_REDEFINITION` | Dummy HMAC `helix security` started. Real DRS authorization is B13. Do not fold into Phase C. |
| Stage 4 `helix bench` vs Demo | `NEEDS_REDEFINITION` | Engine exists; Ferrum-version compare exit not met. Not verification. Later. |
| Stage 5 external stranger try | `STILL_VALID` | P2 reduced friction. Blocked on published default branch + tag more than on new checks. Process, not Phase C mission. |
| Helix Cloud / ranking / SLA | `NO_LONGER_DESIRED` (this horizon) | Unchanged: outside the ladder. |
| “Next: more DRS checks” as default | `SUPERSEDED` as the *next* move | Coverage expansion changes `coverage_id`. Independent evidence is still historical. |
| B13 authorization | `DEFERRED` | Explicit. |
| P1 product page / P2 first usable path | `DONE` | On this branch; closure docs may still be uncommitted. |
| Phase B B1–B15 | `DONE` | PASS WITH FINDINGS. |

---

## Candidate capabilities

Default hypothesis (DRS depth → more implementations → security → WES) **does not survive** contact with the repo.

| Candidate | User | Verification | Moat | Evidence | Independence | Reuse | Effort | Risk | Deps | Ready | Phase C? |
|-----------|------|--------------|------|----------|--------------|-------|--------|------|------|-------|----------|
| A DRS catalog expansion | Med | High *if* GetAccessURL/GetServiceInfo | Med | High (fixtures) | Low | Med | Med–High | **High** (`coverage_id`, VERIFIED meaning) | Must not touch B13 ops | Ready but wrong *next* | **No** |
| B Third DRS lineage | Med | Med | Med | Hard (ops) | High *if* new architecture | Low | High | Med (qemu/docker finding) | Docker/source | Not needed yet | **No** (two lineages exist) |
| C Security behaviour | Low without B13 | Low for DRS authors | Low | Dummy HMAC only | None | Low | Low | Confuses with authorization | B13 for the useful part | HMAC ready; useful part not | **No** |
| D Versioned WES | High for pipeline stacks | High *after* provenance | High | Blocked | Med | High | High | High if SUPPORTED without local `$ref` | Vendor rewrite | **Open** | **Defer** |
| E CI / helix-action | Med for Ferrum | Low (same checks) | Low | Helix CI already proves mocks | None | Med | Med | False alarms | Ferrum politics | Pilot exists | **No** |
| F Bench | Low | None (must stay out) | None | Exists | None | Low | Low | Claim contamination | None | Exists | **No** |
| G Stranger install/tag | High | None new | Low | P2 | None | Low | Low | Announcing off `main` | Merge/CI | Process | **Parallel, not C** |
| **Independent two-target path** | **High** | High (real targets) | **High** | B12 + EXTERNAL_EVIDENCE | **Uses existing two** | High (differential) | Med | Live stacks / qemu | Docker optional; not prove | **Yes** | **Phase C** |

### Challenge notes

- **More DRS checks** would teach GetAccessURL, which *does* explain Starter Kit SKIP vs Bento PASS. They also **change the coverage contract**. Do that in a later phase as an explicit new `coverage_id`, not as the first move while independent evidence is still a one-off lab capture.
- **A third implementation** of another Java/Python DRS clone adds less than making the two existing lineages **current and operator-repeatable**.
- **WES** is strategically large and **not ready** (HTTPS `$ref` at `standards/vendor/ga4gh.wes.1.1.0/workflow_execution_service.openapi.yaml` line 604). Unversioned WES already runs.
- **CI** of the in-process fixture is already `make prove`. Putting qemu Starter Kit in GitHub Actions fights the B15 finding and does not help an external DRS author.
- **Adoption** (merge, tag, HelixTest sibling) should happen, but it is not a verification capability. Do it as release hygiene, not as Phase C.

---

## Recommended Phase C

**Mission:** Independent DRS 1.4.0 verification as a product path.

Same pinned pack, checker, catalog, coverage, and claim join as Phase B. Two real implementations the operator starts. Current evidence. Inspect. Differential. No catalog expansion. No ranking. No B13.

**Why it wins:** It is the only candidate that lets a user learn something **new and trustworthy** (why two real DRS systems differ under the *existing* contract) without weakening identities or pretending WES/authorization are ready.

---

## Phase-C work packages

### C1 — Canonical two-target operator workflow

**Status:** Implemented. Operator path: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md). Report: [C1_CANONICAL_TWO_TARGET_WORKFLOW.md](C1_CANONICAL_TWO_TARGET_WORKFLOW.md). Helper: `make verify-independent` (optional live; not prove; no `docker pull`; does not restamp `local/b12/`).

**Objective:** One documented path, using the actual CLI, as obvious as `make verify-drs`.

**Why:** The independent path lived in engineering notes ([EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md)). P2 never made it the second operator journey.

**Surface:** [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md), Makefile/help, `helix verify --output`, `helix inspect`, `helix differential`. No new checks.

**Evidence:** Commands in `--help` / Makefile; tests in `tests/c1_two_target_workflow.rs`. Live runs remain optional.

**Exit:** An operator can find: start target → flags (object id, target-kind, standard/version) → output JSON → inspect → differential. CI still does not require live HTTP.

### C2 — Current live evidence protocol

**Objective:** Newly produced live JSON cites this Helix git SHA (`current_verifier_evidence`). Historical B12 files stay historical and **unrestamped**.

**Why:** B12 JSON cannot be “the” product evidence for this binary. Users must know how to capture current files and where to put them (`local/` gitignored / `HELIX_B12_LIVE_DIR`).

**Dependencies:** C1 flags. Existing B14/B15 standing code.

**Surface:** Docs + regression tests that inspect does not rewrite files; optional live tests skip when ports are down.

**Exit:** Protocol written; tests prove restamp-of-B12 is forbidden; a fresh live run (when stacks exist) classifies as current.

**Status:** Implemented. Report: [C2_CURRENT_LIVE_EVIDENCE.md](C2_CURRENT_LIVE_EVIDENCE.md). Helper writes `local/independent/*-current.json`. Historical `local/b12/` is refused and unrestamped.

### C3 — Differential interpretation of the two outcomes

**Objective:** The Starter Kit vs Bento split is operator-understandable: NOT_VERIFIED vs VERIFIED/partial, SKIP `fixture_unavailable` vs PASS, same `execution_id`, distinct `target_execution_id`, no ranking.

**Why:** Without this, C1 is two JSON files and a shrug.

**Dependencies:** C1. Existing `helix differential` / `src/differential.rs`.

**Surface:** Operator text, maybe `after_help` on `helix differential`. Tests that fixture-capability vs target-behavior language remains; ranking needles stay absent.

**Status:** Implemented. Report: [C3_DIFFERENTIAL_INTERPRETATION.md](C3_DIFFERENTIAL_INTERPRETATION.md). Operator text names the shared contract, distinct target executions, PASS/FAIL/SKIP, attribution, VERIFIED vs NOT_VERIFIED, and evidence standing. No ranking. Inputs are not rewritten.

**Exit:** Same conceptual distinction as B12, now on the operator path, without new verification claims.

---

## Boundaries

Phase C will **not**:

- add DRS catalog rows or change `coverage_id` / `execution_id` pins;
- add standards or DRS 1.5.0 / WES SUPPORTED;
- reopen B13 or send credentials on `helix verify`;
- treat `helix security` dummy HMAC as authorization evidence;
- add HELIOS signing, RO-Crate, PDF, telemetry, upload, cache, ranking, certification;
- fetch standards at runtime;
- put qemu/Docker live DRS into `make prove`;
- count extra tags of the same Starter Kit image as a new independent implementation;
- announce Helix as verifying named GA4GH releases from `main` until this branch is the published baseline.

---

## Exit criteria

Phase C closes only if **behavioural** evidence shows:

1. The two-target workflow is discoverable from shipped docs/CLI (tests, not a private script).
2. When both stacks are up, two `helix-verification-v1` files can be produced with `--standard drs --version 1.4.0`, inspectable, **not** rewritten by inspect.
3. Fresh files from this binary are `current_verifier_evidence`; `local/b12/*.json` remains historical and byte-identical to the pre-Phase-C files.
4. Starter Kit remains NOT VERIFIED; Bento remains VERIFIED with `coverage.state = partial` under the **same** Phase B pins (`execution_id`, `coverage_id`, checker, pack hashes).
5. `helix differential` of those two files contains no ranking; identities stay distinct.
6. `cargo test --offline --locked --all-targets` and `./scripts/prove.sh` still pass **without** live HTTP.
7. B13 still deferred; `drs.security.authorization` still UNEVALUATED.
8. No new SUPPORTED pack.

Do not invent the final test IDs here.

---

## Deferred items

**Authorization verification remains outside Phase C until a reproducible, locally runnable, real authorization-enabled implementation is available.**

Also deferred: WES SpecSource/local `ga4gh-service-info`; DRS catalog expansion (new coverage contract); helix-action Ferrum `main`; `helix bench` as anything but measurement; tagged crates.io/binary.

---

## Risks

- Live stacks down → C1/C2 cannot produce current JSON in a session. **Mitigation:** fail closed; optional tests; do not fake.
- qemu/amd64 Starter Kit on arm64 remains environmental. **Mitigation:** document; do not make prove depend on it.
- Operator labels (`--target-kind`) remain untrusted. **Mitigation:** existing independence classifier; do not weaken it to green a run.
- Scope creep into GetAccessURL “while we are here.” **Mitigation:** C1–C3 only; coverage pins frozen.
- Publishing this branch as if `main` already contained Phase B. **Mitigation:** process item, called out, not a C package.

---

## Next implementation step

Phase C is closed ([PHASE_C_CLOSURE.md](PHASE_C_CLOSURE.md)). Do not start Phase D here. Do not expand coverage. Do not reopen B13.

The roadmap reset is complete ([ROADMAP_RESET.md](ROADMAP_RESET.md)). The next action is a product decision, not automatic Phase D implementation.
