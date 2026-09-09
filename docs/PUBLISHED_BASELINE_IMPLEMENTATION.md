# Published baseline — implementation record

Helix is HelixTest becoming a standalone VERIFY CLI. This document records the **implementation** of [PUBLISHED_BASELINE_RELEASE_PLAN.md](PUBLISHED_BASELINE_RELEASE_PLAN.md). It is not a default-branch publication. It is not a tag. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht. Vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: DEFERRED / CLOSED
drs.security.authorization = UNEVALUATED
```

---

## 1. Verdict

**PUBLISHED BASELINE IMPLEMENTATION — READY FOR PR**

This branch is a validated PR candidate for `main`. It is **not** published until `origin/main` contains this product and GitHub CI is green on that merge.

No tag was created. No crates.io publication. No binary distribution.

---

## 2. Git operations actually performed

| Step | Result |
|------|--------|
| Starting branch | `wip/drs-140-productization` |
| Starting SHA (dirty tree) | `8ab4b1f15df07e2ed1256a3a87975e7582029030` |
| Productization snapshot commit | `a2813800edd31b2bfbc4b3adcd4a99ff5ae822a9` — *Prepare DRS 1.4.0 productization snapshot for a source baseline.* |
| `git fetch origin` + `git merge origin/main` | **Yes.** Merge commit `f96786e9b3ac2e1e6cd09341eed0bd2c6deb6970` |
| Rebase | **Not used.** |
| Force-push | **Not used.** |
| Tag `v0.1.0` | **Not created** (plan: tag only after merge to `main` + CI green). |
| Push / PR | Recorded at the end of this file after those operations are attempted. |

`origin/main` unique commits taken in the merge: `26a3209` (freeze), `1304d92` (LICENSE/NOTICE + toolchain pin), `a8d1c20` (restore public wording).

---

## 3. Merge conflicts and resolution

Conflicts were limited to three files. No ambiguous identity conflict.

| File | Resolution |
|------|----------|
| `README.md` | **Productization** DRS 1.4.0 SUPPORTED wording. Discarded `main`’s outdated “DRS-only / WES not executed” freeze-era paragraph. |
| `SECURITY.md` | **Productization** VERIFY-CLI / threat-model / no-SLA paragraph. |
| `scripts/prove.sh` | **Productization** README greps (DRS 1.4.0 coverage sentence). Did **not** restore `main`’s older grep. |

Taken from `main` without conflict (hygiene):

- `LICENSE` GitHub-detectable Apache-2.0; copyright in `NOTICE` (`Copyright 2026 Synaptic Four`)
- `.github/workflows/ci.yml` pinned `dtolnay/rust-toolchain@d1031067263f94b142dd6c0ce24c5eb9d02d52a0`
- `INVENTORY.md` placeholder-image wording (HelixTest inventory SHA `1832c043…` remains a **historical HelixTest inventory date**, not Helix `VERSIONS.lock`)

**HelixTest pin was not restored to `1832c043…`.** Exact pin remains `1baddfd3d75f01dc7c149074a785616fa014c725`.

Release-facing follow-up on the merged tree: `47a24825b52adc463da5fd6ff50065885f21385c` — README SUPPORTED sentence; CHANGELOG `0.1.0` baseline identity. This implementation record is a later commit on the same branch.

---

## 4. Verification identities (unchanged)

Compared `a281380` (productization snapshot) to the merged tree. No diff in `src/live_evidence.rs`, `src/standards/support.rs`, `VERSIONS.lock`, or `standards/registry.yaml`.

| Identity | Value |
|----------|--------|
| HelixTest git pin | `1baddfd3d75f01dc7c149074a785616fa014c725` (tag v0.1.3) |
| Checker source SHA-256 | `18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a` |
| Checker id | `helixtest-drs:18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a` |
| DRS pack | `ga4gh.drs.1.4.0` |
| Pack integrity SHA-256 | `c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89` |
| Schema document SHA-256 | `3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608` |
| Schema component SHA-256 | `b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003` |
| Binding | `72da037c4ce2383f116bf195507fe1b45c60d6917acf5a87c6e5bba7043c69e2` |
| Catalog | `03ce38f690ed679ff967e636bb037e0ed4ebe42f2cbde62766921ad7eae96ac1` |
| Coverage id | `082f63c9eec7472a66f8121a66e28ffb7f68680791f7bb5ea5ef47441d18f08c` |
| Execution id | `ee00e1a49e6b3f7d47314bde77738faf4b6cec4d3325dfe56d139809ea97037e` |
| Release commit (SpecSource) | `36145d389e0a454428d1dac5c4a30870995fdd7c` |

Observed on `make verify-drs` at `47a2482`: `support_status: SUPPORTED`; `coverage.state: partial`; WES rows SKIP (`unsupported_test`; standard not selected). WES is **not** a published Supported standard.

---

## 5. Validation (working tree at `47a2482`)

Toolchain: rustc **1.91.1** (rustup; `rust-toolchain.toml`). Sibling HelixTest at the pin.

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --locked --all-targets --all-features -- -D warnings` | OK |
| `cargo test --offline --locked --all-targets` | OK (0 failed) |
| `./scripts/prove.sh` | OK (`prove: docs OK`) |
| `make independent-verify` | OK (offline; `official_supported: 1`; 4 repro tests) |
| `make verify-drs` | OK (fixture; not independent evidence) |
| `helix inspect verify.json` | standing `current_verifier_evidence`; **did not rewrite** (`sha256` unchanged) |
| `helix --version` | `helix 0.1.0` / `Helix git: 47a24825b52adc463da5fd6ff50065885f21385c` / HelixTest pin v0.1.3 |
| README `"None are SUPPORTED"` | **0** matches |

`verify.json` remains gitignored. `local/b12/` was not restamped.

---

## 6. Clean-checkout acceptance

Separate worktree (not this working copy):

```text
/Users/SynapticFour/devel/helix-pb-accept/Helix      @ 47a2482 (detached, clean)
/Users/SynapticFour/devel/helix-pb-accept/HelixTest   @ 1baddfd3d75f01dc7c149074a785616fa014c725
```

| Step | Result |
|------|--------|
| `make fetch` | OK |
| `make prove` | OK |
| `make verify-drs` | OK |
| `helix inspect` | standing `current_verifier_evidence`; file not rewritten |
| `helix --version` | git SHA `47a2482…` |
| Live Starter Kit / Bento | **Not run** (optional; not required; no manufactured JSON) |

Source-build acceptance: **PASS**. Live independent target verification: **not exercised**.

---

## 7. CI configuration (inspected, not executed here)

`.github/workflows/ci.yml` runs on **push/PR to `main`/`master`** only (this WIP ref does not run GitHub CI until a PR targeting `main`).

Existing jobs: `make prove`, `make independent-verify`, `make verify-fixture`, clippy `-D warnings`, `cargo fmt --check`. HelixTest checkout uses `VERSIONS.lock` `HELIXTEST_SHA`. Toolchain pin from `main` retained.

`make verify-drs` is the canonical DRS 1.4.0 fixture path (Makefile + `examples/verify-drs-140`). It is **not** a separate GitHub job. That was not added (no new CI system). In-process fixture; live Docker is not mandatory.

---

## 8. Documentation

| File | Change |
|------|--------|
| `README.md` | Removed false “None are SUPPORTED”. DRS 1.4.0 is SUPPORTED within declared coverage; `standards list` still shows AVAILABLE rows. Not certification. |
| `CHANGELOG.md` | `0.1.0` early-stage source baseline. Unreleased is empty of product scope. |
| `docs/HELIX_PRODUCT.md` | Unchanged (still accurate). |
| `docs/OPERATOR_VERIFY.md` | Unchanged. |
| `docs/INDEPENDENT_DRS.md` | Unchanged. |
| `docs/HELIX_ROADMAP.md` | Still states this position is on the WIP branch, not `origin/main`. Do not rewrite to “on main” until merge actually happens. |

---

## 9. Deferred (not implemented)

- DRS coverage expansion
- WES as a published Supported standard
- B13 authorization (remains DEFERRED / CLOSED; UNEVALUATED)
- Ferrum CI / helix-action on Ferrum `main`
- Third independent DRS implementation
- Signing / HELIOS
- Benchmarks as verification, rankings, dashboards, telemetry
- Cloud service
- crates.io / binary distribution
- Tag `v0.1.0`

---

## 10. Remaining external steps

These have **not** been claimed complete unless this section is updated after they happen:

1. Push `wip/drs-140-productization` to `origin`.
2. Open PR into `main`.
3. Wait for GitHub CI green.
4. Merge (no rebase of productization history; no force-push of `main`).
5. Verify merged SHA on `origin/main`: pin, identities, B13, README, CHANGELOG.
6. Optional annotated tag `v0.1.0` on the **merged** commit only.
7. Independent clone from GitHub at that SHA: `make fetch && make prove && make verify-drs`.

---

## 11. Post-merge operator checklist

Do **not** tick these until they have been observed on the merged default branch.

- [ ] Default branch contains this productization state (Phase B/C docs, DRS 1.4.0 SUPPORTED).
- [ ] Phase B remains **CLOSED** (`docs/PHASE_B_CLOSURE.md`).
- [ ] Phase C remains **CLOSED**.
- [ ] B13 remains **DEFERRED / CLOSED**; `drs.security.authorization` **UNEVALUATED**.
- [ ] DRS 1.4.0 remains **SUPPORTED** (partial coverage).
- [ ] HelixTest pin remains `1baddfd3d75f01dc7c149074a785616fa014c725`.
- [ ] Pack / schema / checker / binding / catalog / coverage / execution ids unchanged (§4).
- [ ] README does not claim no Supported standards.
- [ ] CHANGELOG identifies the `0.1.0` source baseline.
- [ ] GitHub CI is green on the merged commit.
- [ ] Clean checkout of the merged SHA: `make fetch && make prove && make verify-drs`.
- [ ] `helix --version` reports the merged commit SHA.
- [ ] Tag `v0.1.0` exists only if explicitly created after the above.

---

## 12. Push / PR / tag (fill after attempting)

| Operation | Observed |
|----------|----------|
| Pushed | *pending this file’s commit* |
| PR URL | *not created at the time this paragraph was written* |
| Merged to `main` | **no** |
| Tag | **none** |

**Vertraue mir nicht. Vertraue dem Code.**
