# Published baseline readiness

Helix is HelixTest becoming a standalone VERIFY CLI. This document is a **release-readiness assessment** of the existing DRS 1.4.0 product. It is not a release. It is not a tag. It is not Phase D. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht. Vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: DEFERRED / CLOSED
drs.security.authorization = UNEVALUATED
```

Roadmap reset: [ROADMAP_RESET.md](ROADMAP_RESET.md). Product: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md).

This file does **not** implement a release. It does not choose a git tag. It does not merge to `main`.

---

## 1. Assessment date

**2026-09-08**

Examined after Phase B close, Phase C **PASS WITH FINDINGS — CLOSED**, and [ROADMAP_RESET.md](ROADMAP_RESET.md).

---

## 2. Repository state

| Fact | Observed |
|------|----------|
| Branch | `wip/drs-140-productization` |
| HEAD | `8ab4b1f15df07e2ed1256a3a87975e7582029030` |
| Working tree | **Dirty.** Phase B/C/reset docs, C1–C3 tests, and related productization changes are uncommitted or modified. |
| `origin/wip/drs-140-productization` | Local branch is **ahead 10** commits; working tree still dirtier than that. |
| `origin/main` | `a8d1c20`. Merge-base with HEAD: `4ee590a`. **Diverged:** three commits on `main` are not in this HEAD (`26a3209` freeze, `1304d92` LICENSE/toolchain, `a8d1c20` restore public wording). |
| Git tags | **None.** |
| Published binary | **None.** `publish = false`. No GitHub Release artefacts. |
| Package | `helix` `0.1.0` (`Cargo.toml`) |
| HelixTest | Path dep `../HelixTest`. Pin `HELIXTEST_SHA=1baddfd3d75f01dc7c149074a785616fa014c725` (`VERSIONS.lock`), tag **v0.1.3**. |
| CI | `.github/workflows/ci.yml` runs `make prove` on **`main`/`master` PRs**. This WIP branch is not that workflow until a PR. |
| Live independent DRS | Optional. C1–C3 sessions: Starter Kit `:4500` and Bento `:5000` down. `local/b12/` historical, unrestamped. |

A stranger who clones `https://github.com/SynapticFour/Helix.git` and checks out **`main` does not obtain this productization.** Announcing the current working tree as the public Helix would describe work that is not the published default branch and is not even a committed snapshot.

---

## 3. Proposed published baseline

Inferred from what the repository actually implements and documents as operator-facing. Not a larger product.

```text
source build (sibling HelixTest at VERSIONS.lock)
    → helix verify URL --standard drs --version 1.4.0 --output FILE
    → helix inspect FILE
    → optional: second target + helix differential FILE_A FILE_B
```

Clean-environment proof without a live DRS: `make verify-drs` (in-process fixture). That is **not** independent-implementation evidence.

### Baseline capability

What would be published (source, early-stage):

- The `helix` CLI as built from this repo.
- **Versioned technical verification of GA4GH DRS 1.4.0** within the declared partial coverage boundary (only SUPPORTED pack).
- Persist `helix-verification-v1`; classify standing with `helix inspect` (does not rewrite the file).
- Optional two-target operator path against implementations the user starts ([INDEPENDENT_DRS.md](INDEPENDENT_DRS.md)): comparison, not ranking.
- Unversioned `helix verify URL` remains in the binary (DRS and WES checks when TESTABLE). It is **not** a named-release verification claim and is **not** the baseline’s product sentence.

Shipped extras (`security`, `bench`, `compare`, `matrix`, `standards`) stay in the tree with their existing limits. They are **not** the published product claim.

### Baseline contract

A user gets:

- A documented source build ([INSTALL.md](INSTALL.md), [FOR-EVALUATORS.md](FOR-EVALUATORS.md)).
- One canonical DRS 1.4.0 command path ([OPERATOR_VERIFY.md](OPERATOR_VERIFY.md)).
- An inspectable unsigned JSON artefact (`schemas/helix-verification-v1.json`).
- Explicit PASS / FAIL / SKIP / ERROR at check level, and VERIFIED / NOT_VERIFIED as derived claims.
- Honest partial coverage; unevaluated rows named, not silently passed.
- Offline `make prove` against in-process fixtures (after `make fetch`).

### Baseline limitations

The user does **not** get:

- GA4GH certification or full DRS 1.4.0 coverage.
- Authorization verification (B13 **DEFERRED / CLOSED**).
- Versioned WES / DRS 1.5.0 SUPPORTED packs.
- A published binary, crates.io crate, or container.
- Cryptographic signing, RO-Crate, or PDF (HELIOS).
- Ranking of implementations.
- Live Starter Kit/Bento as CI or as universally replayable evidence.
- Helix as a Ferrum dependency or Ferrum `main` CI gate.

### Intended audience

A technically competent engineer who can install Rust **1.91.1**, clone two public git repositories, and either use the in-process fixture or point Helix at a DRS HTTP origin they already run.

Not: a clinical operator, a GA4GH certification applicant, or someone who needs a one-click binary.

### Evidence model

`helix-verification-v1` records target, selected standard/version, checker provenance, executed/skipped checks, derived claims, and coverage. `helix inspect` recomputes claims, coverage, and **standing** (current verifier evidence vs historical observation). Standing is not a JSON field.

The artefact does **not** establish: cryptographic non-repudiation (unsigned consistent restamp remains possible), authorization, full DRS, or that another Helix binary is the same verifier.

---

## 4. Current capability

The documented product exists in this working tree:

| Surface | Exists | Tests |
|---------|--------|-------|
| `helix verify --standard drs --version 1.4.0` | `src/verify.rs`, `src/main.rs` | `tests/p2_first_usable_release.rs`, `tests/verify_drs.rs`, B8–B11 |
| `--output` / `helix inspect` | `src/evidence.rs`, `src/report.rs` | `tests/b14_*.rs`, `tests/b15_*.rs`, `tests/c2_*.rs` |
| `helix differential` | `src/differential.rs` | `tests/c3_differential_interpretation.rs`, `tests/b9_*.rs` |
| Two-target operator docs + helper | [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md), `make verify-independent` | `tests/c1_two_target_workflow.rs` (offline; live optional) |
| DRS 1.4.0 pack | `standards/vendor/ga4gh.drs.1.4.0/`, registry `supported` | `src/standards/support.rs`, `scripts/prove.sh` |
| B13 kept closed | `src/authorization.rs`, `src/coverage.rs` | `tests/b13_authorization_boundary.rs` |

Phase B and Phase C closes already recorded offline `cargo test --offline --locked --all-targets`, clippy `-D warnings`, and `./scripts/prove.sh`. This assessment does not re-run the full cargo suite; it does not treat that omission as missing product code.

WES unversioned checks exist (`tests/verify_wes.rs`). They are **not** proposed as the published baseline claim. Vendor WES 1.1.0 still has an HTTPS `$ref`; it is not SpecSource-ready.

---

## 5. Release-readiness matrix

| Area | Current state | Ready? | Evidence | Remaining work |
| ---- | ------------- | -----: | -------- | -------------- |
| Source acquisition | GitHub remote exists. **Default branch is not this product.** | **No** | `origin/main` ≠ this HEAD; dirty WIP | Land a committed snapshot on the published default branch (or an explicitly labeled public ref). |
| Build | Source build documented; sibling HelixTest required; `make fetch` then offline prove | **Yes, with friction** | [INSTALL.md](INSTALL.md), `scripts/require-helixtest.sh` (missing sibling → exit 2; SHA mismatch → exit 1) | Keep the two-clone story. Do not pretend a single clone suffices. |
| Reproducibility | Pins in `VERSIONS.lock`, `Cargo.lock`, vendor hashes, `helix --version` git SHA | **Yes for source** | [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md); not bit-for-bit JSON; not HELIOS | State early-stage; no binary reproducibility claim. |
| Dependencies | `Cargo.lock` committed; crates.io only via `make fetch`; no runtime GA4GH fetch | **Yes** | [DEPENDENCY.md](DEPENDENCY.md), `Cargo.toml` path dep | HelixTest remains a separate git root (D1). |
| CLI | Canonical versioned command is in help, OPERATOR_VERIFY, P2 | **Yes** | `src/main.rs` `after_help`; [CLI_CONTRACT.md](CLI_CONTRACT.md) | Default `helix verify URL` must stay unversioned in the published story. |
| Target workflow | Fixture vs operator-started HTTP origin is documented | **Yes** | [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), [FIXTURES.md](FIXTURES.md) | Helix does not start stacks. |
| DRS 1.4.0 | Only SUPPORTED pack; partial coverage named | **Yes** | registry; [COVERAGE.md](COVERAGE.md); [HELIX_PRODUCT.md](HELIX_PRODUCT.md) | Do not expand coverage as a release prerequisite. |
| Evidence | `--output` + inspect; schema frozen | **Yes** | [SCHEMA.md](SCHEMA.md), [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md) | Unsigned. HELIOS stays out. |
| Failure interpretation | Report + inspect distinguish PASS vs VERIFIED, attribution, unevaluated | **Yes** | [REPORT.md](REPORT.md), [CLAIMS.md](CLAIMS.md), [C3_DIFFERENTIAL_INTERPRETATION.md](C3_DIFFERENTIAL_INTERPRETATION.md) | — |
| Independent implementations | Operator path exists; live optional; not certification | **Yes as optional** | [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md); C1–C3; B12 gitignored | Do not require live Docker for a source release. |
| Documentation | HELIX_PRODUCT + OPERATOR_VERIFY + evaluator pack exist; B-gate reports are not the front door | **Mostly** | README groups “Start” vs technical docs | Fix stale README line that `standards list` shows none SUPPORTED. CoC/templates are GitHub hygiene, not the operator path. |
| Release identity | `0.1.0`, no tag, CHANGELOG Unreleased, dirty tree | **No** | `git tag` empty; [P2_FIRST_USABLE_RELEASE.md](P2_FIRST_USABLE_RELEASE.md) | Need a committed, named ref before calling anything published. |
| Testing | Broad offline suite; prove greps; C1–C3 tests | **Yes for source** | `Makefile` `prove`; `tests/` | Live HTTP not in prove (intentional). |
| CI | Sufficient **once on `main`**. This branch is not in that workflow. | **Not for this ref** | `.github/workflows/ci.yml` | PR/merge so CI proves the published snapshot. |
| Live verification | Fixture: yes. Independent live: optional, not exercised in C sessions. | **N/A as source gate** | [PHASE_C_CLOSURE.md](PHASE_C_CLOSURE.md) | Do not manufacture live JSON. |
| Security boundaries | B13 closed; verify sends no credentials; dummy HMAC is not DRS auth | **Yes** | [AUTHORIZATION.md](AUTHORIZATION.md), [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md) | Keep excluded from the baseline claim. |
| Licensing | Helix `LICENSE` Apache-2.0; `Cargo.toml` Apache-2.0; README says same licence as HelixTest | **Sufficient to state, not a legal opinion** | `LICENSE`, [CONTRIBUTING.md](CONTRIBUTING.md) | Path-dep HelixTest is a **separate** Apache-2.0 repo; this assessment did not re-audit HelixTest `LICENSE` bytes. crates.io remains inappropriate while the path dep exists. |

---

## 6. Blockers vs polish

| Item | Class |
|------|--------|
| Productization not on the published default branch | **RELEASE BLOCKER** |
| Working tree is not a committed identifiable snapshot | **RELEASE BLOCKER** |
| No tag / no documented published commit as the baseline identity | **RELEASE BLOCKER** for calling it published; **RELEASE REQUIREMENT** to pick one (tag **or** a named commit on default branch) |
| Reconcile the three `origin/main` commits not in this HEAD | **RELEASE REQUIREMENT** |
| CI green on the ref that will be published | **RELEASE REQUIREMENT** |
| README still says `standards list` has “None are SUPPORTED” | **RELEASE REQUIREMENT** (honesty). DRS 1.4.0 is SUPPORTED in this tree. |
| Sibling HelixTest clone + pin | **NOT REQUIRED** to change. It is a stated baseline constraint. |
| `CODE_OF_CONDUCT.md`, issue forms, PR template | **IMPORTANT POLISH** (GitHub visitor expectation). Not required to build or run `make verify-drs`. |
| `rust-version` 1.88 vs CI/toolchain 1.91.1 | **IMPORTANT POLISH** |
| Dated OPEN_SOURCE_RELEASE_CHECKLIST / PUBLIC_READINESS snapshots | **DOCUMENTATION IMPROVEMENT** (they already warn they are snapshots) |
| Evaluator pack still centres unversioned `verify-fixture` | **DOCUMENTATION IMPROVEMENT** (OPERATOR_VERIFY is already the canonical DRS 1.4.0 path) |
| MSRV untested, no SBOM, no release binaries | **POST-RELEASE** / **NOT REQUIRED** for a source baseline |
| crates.io / Homebrew / container | **NOT REQUIRED** (`publish = false` is correct) |
| Live Starter Kit / Bento / restamping B12 | **NOT REQUIRED** |
| DRS catalog expansion, versioned WES, B13, Ferrum `main` helix-action | **POST-RELEASE** or **NOT REQUIRED** for this baseline |
| CoC as a press-announcement gate | Treat as polish unless the org decides to do a public announcement; this assessment recommends **early-stage source**, not a launch |

Do not treat missing live Docker as a source-release blocker. Do not treat missing CoC as “the product cannot be used.”

---

## 7. Strategic option comparison

| Option | User value now | Evidence already available | Engineering distance | Risk | Strategic fit | Recommendation |
| ------ | -------------- | -------------------------- | -------------------- | ---- | ------------- | -------------- |
| **A — Publish the DRS 1.4.0 baseline** | **High** — a stranger can run the product that exists | Phase B+C, P2, OPERATOR_VERIFY, prove, fixture path | **Medium process** (commit, reconcile `main`, PR/CI, identity). Little new verify logic | Announcing off WIP/`main`; overclaiming certification | **High** — matches the product the repo actually is | **Plan this.** Do not implement in this task. |
| **B — Expand verification first** | Medium later; **Low now** for adoption | Partial coverage is already honest | **High** (new `coverage_id`, new meaning of VERIFIED) | Changing the contract while the product is still unpublished | **Low as next move** — does not fix default-branch or identity | **Do not prioritize** before a baseline exists. |
| **C — Ferrum CI first** | Medium for Ferrum; **Low** for a DRS implementer | Helix CI on `main`; helix-action pilot documented; Stage 2 not exited | Medium (Ferrum politics, false alarms) | Skip-as-green; Helix mistaken for a Ferrum tool | **Low** as the product direction after Phase C | **Keep optional.** Not the published Helix identity. |

Grounding: [ROADMAP_RESET.md](ROADMAP_RESET.md) already found that the architecture is coherent enough that the fork is adoption vs remaining internal. This assessment agrees. Expansion and Ferrum CI do not make `main` contain the product.

---

## 8. Minimum release delta

### Must have before release

1. **Commit** the productization working tree (Phase B/C/reset and the code they describe) so the baseline is a git object, not a dirty directory.
2. **Publish that snapshot on the default branch** (reconcile `origin/main`; open a PR so CI runs). Do not announce a WIP working tree.
3. **Identify the baseline** with a documented commit on that branch, and optionally an early-stage git tag. CHANGELOG must not imply a GitHub Release that does not exist.
4. **CI green** on that ref (`make prove` path already defined).
5. **Honesty pass** on stranger-facing files for that snapshot (at least README’s false “None are SUPPORTED”).

### Should have before release

- `CODE_OF_CONDUCT.md` and a minimal issue/PR template if the org will accept public GitHub issues.
- Align documented toolchain: INSTALL/CI **1.91.1** is the real requirement; `rust-version` 1.88 is untested.
- One-line CHANGELOG “current facts” date aligned with the published snapshot.

### Can happen after release

- crates.io, binaries, containers.
- DRS coverage expansion (new contract).
- Versioned WES SUPPORTED.
- helix-action on Ferrum `main`.
- Live independent JSON in CI (still not recommended as prove).
- Absorbing HelixTest (D1).

Do **not** implement these items in this assessment.

---

## 9. What the release can honestly claim

If the must-haves are met, an early-stage source publication can say:

- Helix is a CLI. You point it at a DRS you run (or the in-process fixture).
- Helix supports **technical verification checks for GA4GH DRS 1.4.0 within a declared partial coverage boundary**.
- PASS is a check outcome. VERIFIED is a derived claim. Exit 0 is not VERIFIED.
- Evidence is inspectable unsigned JSON. Standing is computed.
- Two independent implementations can be compared under the same contract. Helix does not rank them.
- Green `make prove` is a technical signal, not certification.
- Single-steward, Apache-2.0, sibling HelixTest required.

Existing pages that already carry this: [HELIX_PRODUCT.md](HELIX_PRODUCT.md), [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), [TRUST.md](TRUST.md), README “What this is not”, [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

---

## 10. What it must explicitly not claim

The published baseline must **not** imply:

- GA4GH certification, endorsement, or official status
- full DRS conformance or complete DRS coverage
- production readiness
- that authorization was verified
- ranking or a “better” implementation
- cryptographic signing or HELIOS functionality
- that `origin/main` already contained this work before it actually does
- that live Starter Kit/Bento results in C1–C3 were current live evidence

Current documentation **can** make these boundaries clear. One stranger-facing defect in this tree: README “Other shipped commands” still says `helix standards list` has **“None are SUPPORTED.”** That is false after Phase B and must not ship in a baseline snapshot.

[OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md) (2026-09-04) and [PUBLIC_READINESS_AUDIT.md](PUBLIC_READINESS_AUDIT.md) (2026-09-05) are **dated snapshots**. They must not be quoted as if they forbade DRS 1.4.0 SUPPORTED. PUBLIC_READINESS already banners the Phase B supersession.

---

## 11. Recommendation

**Plan Option A** — a published **early-stage source baseline** of the existing DRS 1.4.0 product — after the must-have delta.

Do not expand DRS coverage first. Do not make Ferrum CI the Helix product. Do not start Phase D. Do not tag or merge from this file.

The product is coherent. The repository is **not** yet in a state that can honestly be called published.

---

## 12. Conditions for proceeding

Proceed to a release **plan** only if all of these remain true:

1. B13 stays **DEFERRED / CLOSED**; `drs.security.authorization` stays **UNEVALUATED**.
2. Coverage identities are not changed as part of publication.
3. No live JSON is manufactured; `local/b12/` is not restamped.
4. HelixTest pins are not casually moved.
5. Classification remains early-stage source, not crates.io, not a certification announcement.
6. `origin/main` is reconciled, not overwritten without reviewing the three commits that landed there.
7. HELIOS, ranking, Ferrum-as-dependency, and Helix Cloud stay out.

This assessment is **not permission** to tag, push, or announce.

---

## 13. Post-release candidates

After a published baseline exists (not before):

- Verification expansion under a new `coverage_id`
- Versioned WES only after a local complete vendor tree
- Optional Ferrum operational integration
- Community hygiene beyond the should-have list
- D1 revisit only on recorded criteria

---

## 14. Deferred items

| Item | Standing |
|------|----------|
| B13 | **DEFERRED / CLOSED**. UNEVALUATED. |
| DRS 1.4.0 full coverage | Partial remains the baseline. |
| Differential ranking | Will not be added. |
| HELIOS | Separate. |
| Ferrum | Reference target, not a Helix crate. |
| Versioned WES | Not in this baseline. |
| Phase D | Not started. |

---

## Verdict

**PUBLISHED BASELINE — READY TO PLAN**

The existing DRS 1.4.0 product is a coherent public baseline **to plan**. It is **not** ready to tag, merge, or announce today: the work is not on `main`, the working tree is dirty, and there is no published identity.

**Vertraue mir nicht. Vertraue dem Code.**
