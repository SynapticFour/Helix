# Decisions (Helix)

Recorded 2026-09-03. HelixTest Stage 0 decoupling (generic vs Ferrum) is implemented in HelixTest: no WES-name auto-switch, in-tree mock DRS in CI (D2).

## D1 — Keep HelixTest as its own git root

**Decision:** Do not merge HelixTest into Helix until the revisit criteria in [HELIX_VISION.md](HELIX_VISION.md) §7 are met.

**Why:** Ferrum / Lab Kit / ga4gh-infra pin HelixTest **v0.1.3**. SF-TR-2026-001/002 cite HelixTest. Helix is VERIFY docs + later wrapper, not a second suite.

**Consequence:** Stage 0 (generic vs Ferrum coupling) is implemented **in HelixTest**. Stage 1 `helix verify` lives in Helix and path-depends on HelixTest crates. Ferrum `HELIXTEST_REF` stays until Stage 2 explicitly moves it. **helix-action** (Stage 2) checks out both repos as siblings; it is not a reason to merge git histories.

### D1 revisit — 2026-09-04 (helix-action / Stage 2)

**Still keep separate.** Ferrum, Lab Kit, and ga4gh-infra still pin HelixTest **v0.1.3**. SF-TR-2026-001/002 still cite HelixTest. `helixtest-action` still downloads HelixTest release binaries. Merging HelixTest into Helix now would be a lockfile and citation blast radius, and would not make `helix verify` any more correct — the Action already builds Helix against a sibling HelixTest checkout.

### D1 revisit — 2026-09-04 (Stage 3 security module)

**Still keep separate.** Helix now owns a named Security Behavior Profile (`helix security`, [SECURITY_PROFILE.md](SECURITY_PROFILE.md): five HTTP invariants) and Crypt4GH protocol layout after those cases ([CRYPT4GH.md](CRYPT4GH.md); dummy fixtures in `test-fixtures/`). HelixTest still owns the tagged HMAC suite (`framework/src/auth.rs`), `--mode ferrum+infra` Passport checks, and env-gated Crypt4GH HTTP (secret key). This is the HELIX_VISION §7 case (“security-behaviour may live in Helix as an extra module”) — it is **not** a reason to merge git histories or to drop Ferrum’s `HELIXTEST_REF`. It is not a security audit. Crypt4GH pass is not “secure”.

### D1 revisit — 2026-09-04 (Stage 4 bench)

**Still keep separate.** `helix bench` is Helix-owned (repeatable HTTP measurement of `http.drs.smoke.v1` + percent diff of medians). It does not run HelixTest checks and does not replace Demo’s hap.py smoke. HelixTest remains the tagged conformance suite Ferrum pins. Absorbing HelixTest would not make a 3-GET timing series more correct. See [BENCHMARKS.md](BENCHMARKS.md).

## D2 — Non-Ferrum Stage 0 target

**Decision:** Stage 0 exit is an in-tree HTTP fixture (wiremock or equivalent in HelixTest CI), **not** `helixtest/docker/docker-compose.yml` (`ghcr.io/example/mock-*`) until those images are proven to exist and run.

**Why:** [INVENTORY.md](../INVENTORY.md) marked those images UNKLAR. A CI job that needs Ferrum GHCR is not a non-Ferrum proof.

## D3 — Report contract

**Decision:** `helix verify` JSON is Helix `VerificationRun` (DRS and WES rows). `helix security` JSON is HelixTest `OverallReport`. Skips are not passes. Discovery is not a pass. No HELIOS fields (signatures, RO-Crate, PDF). See [CLI_CONTRACT.md](CLI_CONTRACT.md). `helix bench` stays timing-diff JSON (`BenchOutcome`).

### D3 revisit — 2026-09-04 (DRS profile)

**Verify JSON is now Helix-owned.** Operators need target, Helix/HelixTest versions, which DRS checks ran or were skipped, stable ids, and failure codes. HelixTest names are preserved on each row (`helixtest_name`) and still implemented in the sibling repo. `helix security` is unchanged (OverallReport). helix-action compares `VerificationRun` at stable Helix `id` via `helix compare` (fallback: `compare_reports.py` mirroring `src/compare.rs`). The PR comment leads with new regressions / fixed failures / existing failures, not an X/Y score. Not a required Ferrum check.

## D4 — HELIOS

**Decision:** Unchanged. [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) is the feature gate.
