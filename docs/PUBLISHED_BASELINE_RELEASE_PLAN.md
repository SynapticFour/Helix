# Published baseline — release plan

Helix is HelixTest becoming a standalone VERIFY CLI. This document is the **planning record** for publishing the existing DRS 1.4.0 product as an early-stage **source** baseline.

It is not a release. It does not commit, merge, push, or tag. It is not Phase D. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht. Vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: DEFERRED / CLOSED
drs.security.authorization = UNEVALUATED
```

Readiness: [PUBLISHED_BASELINE_READINESS.md](PUBLISHED_BASELINE_READINESS.md). Product: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md).

---

## 1. Purpose

Turn the **already implemented** DRS 1.4.0 Helix product into an honest published baseline on the default branch.

This plan answers how. It does not perform the work.

---

## 2. Proposed baseline

Verified against `src/main.rs`, [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), [HELIX_PRODUCT.md](HELIX_PRODUCT.md), `Makefile`.

```text
source checkout (immutable git object)
    ↓
sibling HelixTest at VERSIONS.lock HELIXTEST_SHA
    ↓
make fetch && make prove
    ↓
make verify-drs
    OR
helix verify TARGET --standard drs --version 1.4.0 --output FILE
    ↓
helix inspect FILE
    ↓
optional:
helix differential FILE_A FILE_B
```

### Core baseline (the published product claim)

- Versioned **GA4GH DRS 1.4.0** technical verification within declared **partial** coverage.
- Persisted `helix-verification-v1`.
- `helix inspect` (recompute standing; does not rewrite).

Clean-environment proof: `make verify-drs` (in-process fixture). That is **not** independent-implementation evidence.

### Optional evaluator workflow

- Operator-started Starter Kit / Bento ([INDEPENDENT_DRS.md](INDEPENDENT_DRS.md)).
- `make verify-independent` (optional live; **not** prove; no `docker pull`; does not restamp `local/b12/`).
- `helix differential` — comparison, not ranking.

### Explicitly not the baseline claim

Full DRS coverage; authorization (B13); versioned WES SUPPORTED; GA4GH certification; production readiness; ranking; cryptographic signing; HELIOS; crates.io binary; Ferrum as a Helix dependency.

Unversioned `helix verify URL` (DRS+WES when TESTABLE) remains in the binary. It is **not** the named-release product sentence.

---

## 3. Repository state

Inspected **2026-09-09**. Readiness claims checked against git, not reused blindly.

| Fact | Observed | Readiness claim |
|------|----------|-----------------|
| Branch | `wip/drs-140-productization` | Confirmed |
| HEAD | `8ab4b1f15df07e2ed1256a3a87975e7582029030` (`cli: first usable DRS 1.4.0 operator path`) | Confirmed |
| Dirty tree | Yes. Modified productization files + untracked Phase B/C/reset/readiness docs, C1–C3 tests, `scripts/verify-independent-drs.sh` | Confirmed |
| `origin/wip/...` | Local **ahead 10** | Confirmed |
| Tags | None | Confirmed |
| CHANGELOG | `## Unreleased` | Confirmed |
| `origin/main` | `a8d1c20` | Confirmed |
| Merge-base | `4ee590a` | Confirmed |
| `origin/main` not in HEAD | `26a3209` freeze; `1304d92` LICENSE/NOTICE/toolchain pin; `a8d1c20` restore public wording | Confirmed |
| HEAD not in `main` | **14** commits | Productization lives only here until merged |

`publish = false`. No GitHub Release artefacts. HelixTest path dep `../HelixTest`. Pin `1baddfd3d75f01dc7c149074a785616fa014c725` / tag **v0.1.3**.

CI (`.github/workflows/ci.yml`) runs on **`main`/`master` PRs**, not this WIP ref.

`.gitignore` already excludes `/local/` and `/verify.json`. Those must stay uncommitted.

---

## 4. Release boundary

### Include in the publishable snapshot

Everything that implements or documents the baseline:

- DRS 1.4.0 pack, registry, schemas, `helix verify` / `inspect` / `differential`.
- Operator docs: HELIX_PRODUCT, OPERATOR_VERIFY, INDEPENDENT_DRS, INSTALL, FOR-EVALUATORS, evaluator-pack.
- Phase B/C closes, roadmap reset, readiness, **this plan**.
- C1–C3 tests and `scripts/verify-independent-drs.sh`.
- Existing B-gate technical docs that state the contract (coverage, claims, B13 deferred).
- `Cargo.lock`, `VERSIONS.lock`, `rust-toolchain.toml`.

### Do not include

| Item | Why |
|------|-----|
| `/local/` (B12 JSON, B13 source copies) | Gitignored operator evidence. Historical. Do not restamp. |
| `/verify.json` | Operator output. Gitignored. |
| `target/` | Build artefacts. |
| Unrelated WIP, debug dumps, credentials | None observed as tracked files. Do not add. |

### Accidental / must-fix, not exclude

README line in “Other shipped commands”: `helix standards list` annotated **“None are SUPPORTED.”** That is **false** in this tree (DRS 1.4.0 is SUPPORTED). Correct before the baseline PR. Do not drop `helix standards`.

### Generated files

No generated release tarball. Schema JSON in `schemas/` is source. Do not commit `target/`.

### Local-only paths

`Cargo.toml` path-depends on `../HelixTest/helixtest/crates/{common,framework}`. That is the **supported** early-stage build. CI clones HelixTest as a sibling at `HELIXTEST_SHA`. An external user must do the same ([INSTALL.md](INSTALL.md), `scripts/require-helixtest.sh`).

**Acceptable** for an early-stage source release. **Not** a self-contained binary. Later standalone distribution would need a non-path HelixTest pin (git/crates.io) and a D1 revisit. Do not pretend that is this baseline.

---

## 5. Branch reconciliation plan

**Do not rebase** the 14 productization commits onto `main`. Phase B close records HEAD `8ab4b1f…`. Rebase would rewrite those SHAs.

**Do not force-push `main`.**

### What is on the productization line

Merge-base `4ee590a` → 14 commits through P2 (`8ab4b1f`) **plus** the dirty working tree (Phase C, closes, reset, readiness).

### What is on `origin/main` only

| Commit | Intent | Keep? |
|--------|--------|--------|
| `26a3209` | Freeze wording; HelixTest pin `1832c043` on **main** at freeze time | Historical. `a8d1c20` already dropped freeze pages. **Do not** restore freeze STATUS or the old HelixTest SHA. |
| `1304d92` | GitHub-detectable Apache LICENSE; `NOTICE`; pin `dtolnay/rust-toolchain` | **Keep LICENSE/NOTICE layout and the action pin** unless they break prove. |
| `a8d1c20` | Restore public wording; drop freeze STATUS | Already the current `main` tip. Productization README/roadmap are **newer** and must win on product claims. |

### Overlap files (both sides changed)

`README.md`, `scripts/prove.sh`, `docs/HELIX_ROADMAP.md`, `.github/workflows/ci.yml`, `Cargo.lock`, `SECURITY.md`.

Also take from `main` if missing on this branch: `NOTICE` (HEAD has no `NOTICE`).

### Recommended method

**Merge `origin/main` into the productization branch** (after the dirty tree is committed), then **PR that branch into `main`**. Merge commit preserves productization SHAs.

Conflict policy:

- Product claims, DRS 1.4.0 SUPPORTED, Phase B/C, prove greps → **productization**.
- LICENSE/NOTICE GitHub detection → **main’s `1304d92` shape**, then re-check `scripts/prove.sh` LICENSE greps if any.
- CI toolchain: keep **1.91.1**; prefer `main`’s pinned `dtolnay/rust-toolchain` SHA over `@master`.
- `Cargo.lock`: resolve by regenerating only if merge is inconsistent; do not casually bump crate versions.
- HelixTest pin on this branch (`1baddfd3…`) **must not** become `1832c043`.

### Must re-check after reconciliation

- `VERSIONS.lock` HelixTest SHA and checker source SHA unchanged.
- DRS `execution_id` / `coverage_id` / pack / catalog / binding / checker pins unchanged.
- B13 still UNEVALUATED / deferred tests still close the boundary.
- `./scripts/prove.sh` still exit 0.
- LICENSE still Apache-2.0 and GitHub-detectable.
- No freeze STATUS page claiming the product is frozen off.

---

## 6. Pre-merge validation

`make prove` = `require-helixtest.sh` + `./scripts/prove.sh` + `cargo test --locked --offline --all-targets`. It does **not** run fmt or clippy. GitHub CI **does** run fmt and clippy.

**All four of the following are required** before merge (CI will fail without fmt/clippy; prove without tests is incomplete):

```bash
cargo fmt --all -- --check
cargo clippy --offline --locked --all-targets --all-features -- -D warnings
cargo test --offline --locked --all-targets
./scripts/prove.sh
```

CI today uses clippy **without** `--all-features`. Running `--all-features` locally is stricter and acceptable (this crate has no optional feature flags of substance). Do not weaken CI.

Also required to match CI:

- `make fetch` first if `--offline` cannot see crates.
- `make independent-verify`
- `make verify-fixture` and/or `make verify-drs` (canonical baseline is **verify-drs**)

**Not required:** live Docker, Starter Kit, Bento, Ferrum, credentials.

Environment: Rust **1.91.1** (`rust-toolchain.toml`), sibling HelixTest at pin, `NO_COLOR=1` as in CI.

Focused tests that must remain in `--all-targets`: `tests/p2_first_usable_release.rs`, `tests/c1_two_target_workflow.rs`, `tests/c2_current_live_evidence.rs`, `tests/c3_differential_interpretation.rs`, `tests/b13_authorization_boundary.rs`, plus existing B8–B15 suites.

Do not add tests in the planning task. Do not put `verify-independent` into `make prove`.

---

## 7. Clean-checkout acceptance

Answers: can a new engineer obtain source and run the canonical DRS 1.4.0 workflow?

Use a **new directory**, not this dirty worktree.

### Source-release acceptance (required)

1. `git clone https://github.com/SynapticFour/Helix.git` and check out the **candidate SHA** (PR head or merged `main`).
2. Record `git rev-parse HEAD` and `helix --version` after build (Helix git SHA).
3. `git clone https://github.com/SynapticFour/HelixTest.git` as sibling. `git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"`.
4. `cd Helix && make fetch && make prove`.
5. `make verify-drs` → human report + `verify.json` + inspect.
6. Confirm inspect classifies standing, does not rewrite the file, and does not claim GA4GH certification.
7. Read [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md) Claims / Coverage: PASS ≠ VERIFIED; coverage partial; authorization unevaluated.

Failure: cannot clone pin, prove fails, verify-drs fails, inspect rewrites, or README still says no SUPPORTED pack.

### Live target availability (not required)

Operator-started DRS ([OPERATOR_VERIFY.md](OPERATOR_VERIFY.md)). Unreachable target is setup failure, not a release failure.

### Optional independent demonstration (not required)

[INDEPENDENT_DRS.md](INDEPENDENT_DRS.md). If stacks are down, skip. Do not manufacture JSON. Do not restamp `local/b12/`.

---

## 8. Release identity

The repo already uses Cargo **`0.1.0`** with `publish = false`. There is no separate public semver train.

**Smallest honest identity:** the **immutable git commit on `main`** after merge (`git rev-parse HEAD`), shown in `helix --version` as `Helix git:`.

**Recommended after merge + CI green:** annotated tag **`v0.1.0`** pointing at that commit, matching `Cargo.toml`, classified **early-stage source**. No crates.io, no binary assets required.

Do not tag until:

- tree on `main` is clean at that commit;
- CI is green;
- README SUPPORTED sentence is true;
- CHANGELOG names that baseline (see §9);
- HelixTest pin is the productization pin, not the freeze SHA.

Do not invent `0.2.0`. Do not create a GitHub Release that implies binaries.

---

## 9. Documentation changes

### Required before the baseline PR merges

| File | Change |
|------|--------|
| `README.md` | Replace “None are SUPPORTED” on `helix standards list`. DRS 1.4.0 is SUPPORTED; `list` without `--supported-only` includes AVAILABLE rows. |
| `CHANGELOG.md` | Keep honesty banner. Before tag: add a dated **Published baseline** / `0.1.0` subsection that is **not** “Unreleased”, stating early-stage source, no crates.io, B13 deferred. |
| `docs/HELIX_ROADMAP.md` | After merge only: “this position is on `main` at SHA …” — **not** until merge actually happens. Until then keep WIP wording. |

### Already adequate (no rewrite in the release PR except as needed for merge conflicts)

[HELIX_PRODUCT.md](HELIX_PRODUCT.md), [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md), [CLI_CONTRACT.md](CLI_CONTRACT.md), [FOR-EVALUATORS.md](FOR-EVALUATORS.md). Evaluator-pack already points at `make verify-drs`.

### Optional polish (not merge-blocking)

CoC, issue/PR templates; `rust-version` 1.88 vs toolchain 1.91.1; dated OPEN_SOURCE_RELEASE_CHECKLIST snapshot banner.

Do **not** rewrite those files in this planning task.

---

## 10. License / dependency observations

Not legal advice.

| Item | Observation |
|------|-------------|
| Helix | `LICENSE` Apache-2.0; `Cargo.toml` `license = "Apache-2.0"`. This branch still has a short copyright block in `LICENSE`; `origin/main` moved that to `NOTICE` for GitHub detection. |
| HelixTest | Sibling `LICENSE` is Apache-2.0 (header “Copyright 2025 Synaptic Four”). Helix docs state the same licence. This plan did not audit HelixTest third-party crates. |
| Cargo | Path dep on HelixTest; crates.io crates via `Cargo.lock`. `publish = false` is correct. |
| Vendored DRS OpenAPI | DRS 1.4.0 vendor `info.license` Apache 2.0 (GA4GH schema repo). Provenance in `standards/registry.yaml`. |

Enough information to publish **source** with an explicit sibling HelixTest clone. Uncertainty: GitHub license classifier on this branch’s LICENSE vs `main`’s NOTICE split — resolve at merge by taking `main`’s detectable layout if prove still passes. No crates.io until D1/path-dep changes.

---

## 11. Blocker classification

| Item | Classification | Why | Required before merge? | Required before tag? |
| ---- | -------------- | --- | ---------------------: | -------------------: |
| Dirty working tree | **BLOCKER** | Baseline must be a git object | yes | yes |
| Not on `origin/main` | **BLOCKER** | Stranger clone of `main` is not this product | yes | yes |
| Reconcile `origin/main`’s three commits | **REQUIRED** | Diverged; LICENSE/NOTICE/CI pin | yes | yes |
| README “None are SUPPORTED” | **REQUIRED** | False product sentence | yes | yes |
| CHANGELOG identity | **REQUIRED** | Unreleased cannot be the tagged story | important for PR; **yes** for tag | yes |
| Immutable commit on default branch | **REQUIRED** | Smallest identity | yes (the merge) | yes |
| CI green on the PR | **REQUIRED** | CI does not run on this WIP ref | yes | yes |
| Clean-checkout `make prove` + `make verify-drs` | **REQUIRED** | Proves the published SHA | yes (on candidate) | yes |
| Sibling HelixTest | **NOT REQUIRED** to change | Stated constraint; `require-helixtest.sh` | documented | same |
| Live Starter Kit / Bento | **NOT REQUIRED** | Optional evaluator path | no | no |
| Independent live differential evidence | **NOT REQUIRED** | Offline C3 tests exist | no | no |
| CoC / issue templates | **POLISH** | GitHub visitor expectation | no | no |
| crates.io / binary | **NOT REQUIRED** | `publish = false` | no | no |
| DRS coverage expansion | **POST-RELEASE** | Would change `coverage_id` | no | no |
| Authorization / B13 | **NOT REQUIRED** | **DEFERRED / CLOSED** | no | no |
| Versioned WES | **POST-RELEASE** | HTTPS `$ref`; not SUPPORTED | no | no |
| Ferrum CI / helix-action `main` | **POST-RELEASE** | Optional integration | no | no |
| HELIOS / signing | **NOT REQUIRED** | Out of Helix | no | no |
| Rebase productization onto main | **NOT REQUIRED** | Would rewrite recorded SHAs | **do not** | **do not** |

---

## 12. Release sequence

Derived from this repo. Do not execute in this task.

```text
1. Commit dirty productization on wip (identifiable snapshot)
2. Merge origin/main into that branch (no rebase, no force-push)
3. Resolve overlap files per §5
4. Pre-merge validation (§6)
5. README SUPPORTED correction; CHANGELOG baseline subsection
6. Clean-checkout acceptance on the candidate SHA (§7)
7. PR into main (so CI runs)
8. CI green
9. Merge to main (merge commit)
10. Verify merged commit: pin, prove, B13, identities
11. Optional annotated tag v0.1.0 on that commit
12. Post-merge clone from GitHub at that SHA; make prove && make verify-drs
```

| Step | Prerequisite | Action | Validation | Failure |
|------|--------------|--------|------------|---------|
| 1 | Dirty tree reviewed; no `/local/` | Commit productization | `git status` clean | Secrets or B12 JSON staged |
| 2 | Step 1 clean | `git merge origin/main` into wip | merge completes or conflicts listed | rebase used; freeze SHA restored |
| 3 | Overlap files | Productization wins claims; NOTICE/LICENSE from main if detectable | prove greps; GitHub license | HelixTest pin regresses to `1832c043` |
| 4 | Sibling HelixTest pin | fmt, clippy, test, prove.sh, independent-verify, verify-drs | all exit 0 | any non-zero; tests weakened |
| 5 | Known README defect | Fix SUPPORTED line; CHANGELOG baseline note | grep README no “None are SUPPORTED” | overclaim certification |
| 6 | Candidate SHA | Fresh clone pair | §7 | prove/verify-drs fail |
| 7 | Remote up to date | `gh pr create` into `main` | PR exists | targeting wrong branch |
| 8 | PR | Wait GitHub CI | prove + clippy + fmt | red CI |
| 9 | Green CI | Merge | `origin/main` contains product | force-push; identity loss |
| 10 | Merge SHA | Inspect VERSIONS.lock, B13, `helix --version` | pin `1baddfd3…`; B13 deferred | pin or coverage_id change |
| 11 | Steps 9–10 | Annotated `v0.1.0` if still desired | tag points at merge SHA | tag on dirty/WIP |
| 12 | Public SHA | Independent clone | prove + verify-drs | instructions wrong |

---

## 13. Acceptance criteria

The baseline is **published** only if all of the following are true:

- [ ] Productization (Phase B+C + this product) is on the **default branch**
- [ ] That commit’s tree is **clean**
- [ ] GitHub CI is **green** on that commit
- [ ] Identity is the merge SHA (and `v0.1.0` only if tagged per §8)
- [ ] README does **not** say no standards are SUPPORTED
- [ ] CHANGELOG identifies the published baseline (not only Unreleased)
- [ ] Clean-checkout: `make fetch && make prove && make verify-drs` works with sibling HelixTest pin
- [ ] Canonical DRS 1.4.0 workflow is [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md)
- [ ] Evidence can be persisted and inspected; inspect does not rewrite
- [ ] Limitations explicit: partial coverage, not certification, not HELIOS, not ranking
- [ ] B13 remains **DEFERRED / CLOSED**; `drs.security.authorization` **UNEVALUATED**
- [ ] No live evidence fabricated; `local/b12/` not restamped
- [ ] HelixTest pin and DRS verification identities unchanged by publication mechanics

---

## 14. Post-release candidates

Not committed as the next phase:

- DRS coverage expansion (new `coverage_id`)
- Additional independent implementations
- B13 only if a usable auth-enabled target exists
- Ferrum CI / helix-action on Ferrum `main`
- Versioned WES after local SpecSource
- HelixTest non-path packaging (D1)
- CoC / templates
- crates.io (blocked by path dep)

---

## 15. Explicit non-goals

- No Phase D
- No coverage expansion as part of publication
- No B13 reopen
- No WES SUPPORTED
- No Ferrum CI as the Helix identity
- No signing / HELIOS
- No ranking
- No live Docker in prove
- No commit / merge / push / tag **in this planning task**

---

## 16. Decision

**PLAN APPROVED — READY FOR IMPLEMENTATION**

The existing DRS 1.4.0 product can be published as an early-stage source baseline **after** the sequence in §12. This file is not that implementation.

**Vertraue mir nicht. Vertraue dem Code.**
