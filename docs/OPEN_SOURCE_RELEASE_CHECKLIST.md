# Open-source release checklist (Helix)

**Date:** 2026-09-04 (snapshot). Follow-up stranger-facing audit: [PUBLIC_READINESS_AUDIT.md](PUBLIC_READINESS_AUDIT.md) (2026-09-05). Do not treat this checklist as the current README claim set.

**Scope:** `SynapticFour/Helix` working tree as reviewed (VERIFY CLI around HelixTest).
**This document is a report.** It is not a release, a tag, a GitHub Release, a crates.io publish, or a public announcement.

**Do not tag. Do not publish. Do not announce from this file.**

Helix is HelixTest becoming a standalone VERIFY CLI, not a new test platform. HelixTest already runs (public repo, CI, SF-TR-2026-001/002, pin **v0.1.3** / `1832c043…`). Reproducibility, signed evidence, RO-Crate, and PDF stay in HELIOS (`helios-audit`). Ferrum has no real clinical pilot. Green CI is a technical signal, not GA4GH certification.

---

## Verdict

**Not ready for an external public announcement or a tagged release.**

The GitHub remote `https://github.com/SynapticFour/Helix.git` already exists (`origin/main`). There are **no git tags**. The current VERIFY CLI (VerificationRun, compare, fixtures, evaluator pack, threat model, run identity) is largely **uncommitted** on `main`. Announcing that product would describe work that is not on the published default branch.

Community files that first-time GitHub visitors expect are missing (`CODE_OF_CONDUCT.md`, issue templates, PR template). There is no release workflow. `docs/DEPENDENCY.md` still says the repo has no Cargo lockfile; `Cargo.lock` is present.

When the gaps below are closed, the first public tag should still be **early-stage** (`0.1.0`), Apache-2.0, single-steward, sibling HelixTest required, not crates.io (path dependency).

---

## Status at a glance

| Check | Status | Notes |
|-------|--------|--------|
| LICENSE | Pass (suite-shaped) | Apache-2.0; same hybrid header as HelixTest. No `NOTICE`. |
| SECURITY.md | Pass with gaps | Private email path exists. No GitHub private vulnerability reporting. |
| CONTRIBUTING.md | Pass with gaps | Clear, honest. No CoC / DCO / issue-template pointer. |
| CODE_OF_CONDUCT | **Blocker** | File missing. HelixTest has a short in-house CoC. |
| Issue templates | **Blocker** | None. Evaluator pack has a pasteable failure report only. |
| CI | Pass with gaps | Prove + fixture + clippy + fmt. Clippy not `--locked`. HelixTest SHA duplicated. |
| Release process | **Blocker** | No tags, no release workflow, no `publish = false`. |
| Dependency pinning | Pass with gaps | `Cargo.lock` committed; HelixTest is a path pin via `VERSIONS.lock`. Docs stale. |
| Rust toolchain | Pass with gaps | CI / `rust-toolchain.toml` = **1.91.1**. `rust-version` = **1.88**. MSRV untested. |
| Reproducible builds | Gap | `--locked` on test/install. No git+SHA Cargo dep, no SBOM, no release binaries. |
| Documentation | Pass with gaps | Strong honesty docs. `DEPENDENCY.md` and `ENGINEERING_AUDIT.md` are stale. |
| Examples | Pass | `examples/verify_fixture.rs` / `make verify-fixture`. |
| Error handling | Pass | clap 2 / anyhow 1 / redacted stderr. |
| Panic paths | Gap | Catalog `spec()` panics on unknown ids; a few invariant `.expect`s. |
| Test coverage | Pass with gaps | Broad fixture/CLI tests. No coverage % / llvm-cov job. |
| Schema compatibility | Pass (`verify` only) | Frozen `helix-verification-v1`. Compare/bench/security JSON not schema-frozen. |
| CLI compatibility | Pass | [CLI_CONTRACT.md](CLI_CONTRACT.md). `helix security` still HelixTest `OverallReport`. |

---

## 1. LICENSE

**Pass (suite-shaped).**

- File: `LICENSE`. Apache License 2.0 terms 1–9 plus appendix.
- `Cargo.toml` `license = "Apache-2.0"`. First-party `.rs` files carry `SPDX-License-Identifier: Apache-2.0`.
- Same extra short notice + full terms as HelixTest (`Copyright 2026 Synaptic Four` here; HelixTest still says 2025). GitHub will typically detect Apache-2.0.

**Gaps**

- No `NOTICE` file (Apache §4(d) is optional until third-party notices need a home).
- Path-depends on HelixTest (Apache-2.0) and crates.io crates. No `cargo deny` / license bill of materials.
- `Cargo.toml` has no `readme`, `homepage`, or `publish = false`. Path deps cannot be published to crates.io anyway; an accidental `cargo publish` would fail late.

**Proposed fixes**

1. Add `NOTICE` with copyright line and “HelixTest remains a separate git root; this binary links those crates at the `VERSIONS.lock` pin.”
2. Set `publish = false` in `Cargo.toml` until a git/crates.io HelixTest pin exists (D1: do not vendor HelixTest).
3. Optional later: `cargo deny` advisories + license allowlist. Not a HELIOS SBOM.

---

## 2. SECURITY.md

**Pass with gaps.**

- Private reports: **contact@synapticfour.com**. Do not open public issues.
- Honest: single-steward, not a security product, threat model linked, HELIOS out of scope.

**Gaps**

- GitHub **private vulnerability reporting** is not enabled in-repo (no `.github/SECURITY.md` extra, no advisory process).
- No response-time SLA (acceptable for single-steward if stated).
- Dummy HMAC/Crypt4GH fixtures live in `test-fixtures/` (labeled NICHT FÜR PRODUKTION). Gitleaks allowlists that path.

**Proposed fixes**

1. Enable GitHub Security Advisories / private vulnerability reporting on `SynapticFour/Helix`.
2. One sentence in `SECURITY.md`: best-effort acknowledgement, no SLA; dummy fixtures are not production secrets.
3. Keep public issues for non-security failures ([evaluator-pack/FAILURE_REPORT.md](evaluator-pack/FAILURE_REPORT.md)).

---

## 3. CONTRIBUTING.md

**Pass with gaps.**

- Issue-first, small PRs, tests, `pre-commit` = CI, sibling HelixTest, no HELIOS, no Ferrum pilot claims, Apache-2.0 on contributions.

**Gaps**

- No Code of Conduct link (file missing).
- No Developer Certificate of Origin / CLA beyond “contributions under Apache-2.0”.
- No pointer to GitHub issue forms (forms missing).
- Does not say “match `VERSIONS.lock` SHA locally or CI and your tree diverge.”

**Proposed fixes**

1. Link `CODE_OF_CONDUCT.md` once it exists.
2. One paragraph: checkout HelixTest at `HELIXTEST_SHA`; `scripts/require-helixtest.sh` already warns on mismatch.
3. Optional: DCO `Signed-off-by` only if the rest of the GA4GH-stack repos use it. Do not invent a CLA product.

---

## 4. CODE_OF_CONDUCT

**Blocker for a community announcement.**

- Helix: **no file.**
- HelixTest / Ferrum / HELIOS: `CODE_OF_CONDUCT.md` (short in-house text, contact@synapticfour.com). Not Contributor Covenant.

**Proposed fix**

Copy HelixTest’s CoC verbatim (same org, same contact). Do not switch to Contributor Covenant unless the whole suite does. Link it from README and CONTRIBUTING.

---

## 5. Issue templates

**Blocker for a community announcement.**

Missing:

- `.github/ISSUE_TEMPLATE/` (bug / docs / question)
- `.github/ISSUE_TEMPLATE/config.yml` (`blank_issues_enabled`, security contact link)
- `.github/pull_request_template.md`
- `.github/CODEOWNERS`

HelixTest has a PR template and CODEOWNERS (`* @SynapticFour`), still no issue forms. Helix’s evaluator pack already has the fields a bug form needs.

**Proposed fixes**

1. GitHub bug form fields matching [evaluator-pack/FAILURE_REPORT.md](evaluator-pack/FAILURE_REPORT.md): command, expected, got, Helix commit, HelixTest SHA, target, rustc. No PHI, no production secrets.
2. `config.yml`: contact links → `SECURITY.md`; blank issues on or off explicitly.
3. PR template: HelixTest SHA, `make prove`, no HELIOS, no Ferrum pilot language.
4. CODEOWNERS: `* @SynapticFour` (single-steward).

Do not add a “sales / demo” issue type.

---

## 6. CI

**Pass with gaps.**

`.github/workflows/ci.yml`: checkout Helix + HelixTest at `1832c043e1679ec283cb2113510ee33684317cce`, rustc **1.91.1**, `make prove`, `make verify-fixture`, clippy `-D warnings`, `cargo fmt --check`. Secret-scan (Gitleaks) and Dependency Review exist.

**Gaps**

| Item | Evidence |
|------|----------|
| Clippy not `--locked` | `cargo clippy --all-targets` in CI and `scripts/hooks/ci-check.sh`. Tests use `--locked`. |
| HelixTest SHA duplicated | Hardcoded in `ci.yml`; also `VERSIONS.lock` and `src/model.rs`. Drift is possible. |
| Workflow `permissions` unset | Default GITHUB_TOKEN may be write. Prefer `contents: read` on prove. |
| Dependency Review `continue-on-error: true` | Non-fatal by design ([DEPENDENCY.md](DEPENDENCY.md)); still true after lockfile exists. |
| No CodeQL / SPDX job | HelixTest has both. Optional for Helix (Rust + SPDX already in sources). |
| Pre-commit needs sibling HelixTest | Honest; first-time contributors will hit this. |

**Proposed fixes**

1. `cargo clippy --locked --all-targets -- -D warnings` in CI and `ci-check.sh`.
2. CI checkout `ref:` from `VERSIONS.lock` (or fail if `ci.yml` SHA ≠ lock SHA).
3. `permissions: { contents: read }` on the prove workflow.
4. Revisit Dependency Review `continue-on-error` now that `Cargo.lock` exists.
5. Do not add Ferrum live jobs to this workflow (`make test-live` stays opt-in).

---

## 7. Release process

**Blocker.**

- **No tags** on `origin`.
- **No** `.github/workflows/release-*.yml`. HelixTest has `release-binaries.yml` for `helixtest`.
- INSTALL: no Homebrew formula, no GitHub Release binary.
- Crate version `0.1.0` is the Cargo package version, not a GitHub Release.
- CHANGELOG “Unreleased” is the honest state.

**Proposed process (do not run it from this review)**

1. Commit the working tree so `main` matches what evaluators are told to clone.
2. CI green on that commit (prove + verify-fixture + clippy `--locked` + fmt).
3. Close CoC + issue templates + `publish = false` + DEPENDENCY.md refresh.
4. Tag **only** when a human asks: `v0.1.0` matching `Cargo.toml`. Annotated tag, CHANGELOG section, GitHub Release notes that repeat: not certification, not HELIOS, sibling HelixTest pin required.
5. Optional later: cross-compile `helix` like HelixTest’s release-binaries (linux-gnu, darwin). Still needs HelixTest sources at build time unless D1 is revisited.
6. **Do not** `cargo publish`. Path deps cannot; even with git deps this is not a crates.io product until D1 says otherwise.

---

## 8. Dependency pinning

**Pass with gaps.**

- `Cargo.lock` is committed (lockfile version 4). `make test` / `make install` use `--locked`.
- HelixTest pin: [VERSIONS.lock](../VERSIONS.lock) tag **v0.1.3**, SHA `1832c043e1679ec283cb2113510ee33684317cce`. Cargo uses **path** `../HelixTest/helixtest/crates/{common,framework}` (D1).
- Dependabot/Renovate off by choice (suite ambassadors).

**Gaps**

- [DEPENDENCY.md](DEPENDENCY.md) records the lockfile, `--locked --offline` after `make fetch`, and Dependabot off. (Stale “no lockfile / docs-only prove” sentences are gone.)
- HelixTest is not a `git = … rev = SHA` Cargo dependency. Two clones with different sibling HEADs compile different engines. `require-helixtest.sh` warns; it does not fail on SHA mismatch.
- Transitive tree includes HelixTest’s `age` and friends even though Helix does not call that path. Supply-chain surface is HelixTest’s, not a Helix feature.

**Proposed fixes**

1. Rewrite DEPENDENCY.md: lockfile exists; HelixTest pin is SHA + path; Dependabot still off; Dependency Review can become fatal later.
2. Optional: `require-helixtest.sh` exit 2 on SHA mismatch (today it only warns). CI already checks out the pin.
3. Do not vendor HelixTest. Do not add HELIOS as a dependency.

---

## 9. Rust toolchain

**Pass with gaps.**

| Surface | Value |
|---------|--------|
| `rust-toolchain.toml` | channel **1.91.1**, rustfmt + clippy |
| CI `dtolnay/rust-toolchain` | **1.91.1** |
| `Cargo.toml` `rust-version` | **1.88** (HelixTest / Ferrum-group MSRV convention) |
| Documented footgun | Homebrew `rustc` 1.97+ on PATH ignores the toolchain file ([INSTALL.md](INSTALL.md)) |

CI does **not** prove the crate builds on 1.88. Operators on rustup in this directory get 1.91.1.

**Proposed fixes**

1. Keep 1.91.1 as the CI pin (match HelixTest/Ferrum-group toolchains).
2. Either add a weekly MSRV job on 1.88, or change `rust-version` to 1.91.1 if 1.88 is untested. Do not claim MSRV 1.88 without a job.
3. Keep the Homebrew PATH warning in INSTALL / evaluator journey.

---

## 10. Reproducible builds (where practical)

**Gap — practical, not scientific reproducibility.** Scientific / signed reproducibility is HELIOS. This row is “same commit → same binary/tests,” not an evidence pack.

What exists:

- `--locked` for test and `cargo install --path .`
- HelixTest SHA in VERSIONS.lock and CI
- Fixture catalog id `helix-fixtures-v1` ([RUN_IDENTITY.md](RUN_IDENTITY.md)) so two verify JSON files can be compared

What does not:

- Cargo git+SHA for HelixTest (path only)
- Release binaries / `SOURCE_DATE_EPOCH` / `cargo-auditable`
- SBOM, signed artifacts, RO-Crate (must **not** be added here)

**Proposed fixes**

1. Document in INSTALL: “reproducible enough” = rustup 1.91.1 + this commit + HelixTest SHA + `Cargo.lock`. Not HELIOS.
2. After a real tag: `cargo build --locked --release` in a release workflow; publish sha256 of the `helix` binary. No signatures in Helix.

---

## 11. Documentation

**Pass with gaps.**

Strengths: README honesty sentence, FOR-EVALUATORS, evaluator-pack (no sales CTA, no telemetry), HELIX_VS_HELIOS, CLI_CONTRACT, SCHEMA, THREAT_MODEL, FIXTURES, INSTALL sibling-clone.

**Stale / misleading**

| File | Problem |
|------|---------|
| [DEPENDENCY.md](DEPENDENCY.md) | Claims no lockfile / docs-only prove |
| [ENGINEERING_AUDIT.md](ENGINEERING_AUDIT.md) | Snapshot from earlier the same day: verify JSON still described as HelixTest `OverallReport`; `helix compare` missing from command surface. Do not treat as current architecture. |

**Proposed fixes**

1. Refresh DEPENDENCY.md (required before announcement).
2. Banner on ENGINEERING_AUDIT: date-stamped snapshot; current sources are SCHEMA / CLI_CONTRACT / ARCHITECTURE. Or regenerate. Do not silently delete facts.
3. README docs list: add this checklist; keep HELIOS split and “not certification.”

---

## 12. Examples

**Pass.**

- `examples/verify_fixture.rs` — in-process DRS mock, `make verify-fixture`, prints `HELIX VERIFICATION`.
- Evaluator pack example JSON: `docs/evaluator-pack/example-verify.json` (schema-validated).

**Gap:** no `examples/compare_two_runs.rs`. Not required for announcement; `helix compare` is documented.

**Proposed fix:** none before tag. Optional later: two checked-in JSON files and a `helix compare` one-liner in commands.md.

---

## 13. Error handling

**Pass.**

- Invalid argv: clap, exit **2**, no VerificationRun on stdout.
- Runtime: `anyhow` on stderr, redacted (`src/redact.rs`), exit **1**.
- Verify/security fail: report printed, then `process::exit(1)`.
- Compare: exit 1 only on `NEW_FAIL` or unreadable JSON.
- Bench warnings do not fail the process.
- URL userinfo rejected; secrets not echoed.

Matches [CLI_CONTRACT.md](CLI_CONTRACT.md).

**Proposed fix:** none for announcement. Optional: map adapter panics (if HelixTest ever panics) to ERROR rows instead of process abort — already the `Err` path for adapter `Result`.

---

## 14. Panic paths

**Gap (acceptable if documented; tighten before a “stable” claim).**

| Location | Kind | Risk |
|----------|------|------|
| `identity::spec` | `panic!` on unknown catalog id | Programming error; only compile-time ids should be passed. |
| `verify.rs` | `.expect("discovery always records VERIFY_ORDER")` | Invariant; would be a Helix bug. |
| `verify.rs` / security HTTP | `.expect("DETECTED TESTABLE … has a base URL")` | Invariant; discovery bug if hit. |
| `crypt4gh_header.rs` | `try_into().unwrap()` on 4-byte slices after `len >= 16` / packet bounds | Invariant-safe. |
| `redact.rs` | `chars().next().unwrap()` on a byte index | Relies on walking UTF-8 char boundaries. |
| `bench/stats.rs` | `assert!(!values_ms.is_empty())` | Empty measurement series. Engine should not call this with zero runs. |

Library `unwrap` in `#[cfg(test)]` modules is fine.

**Proposed fixes**

1. Keep `spec()` panic as a catalog bug, not a target bug. Document in CONTRIBUTING: do not call `spec` with runtime strings.
2. Replace discovery `.expect` with `anyhow::bail!` / ERROR rows so a logic bug becomes exit 1 with a report, not an abort.
3. Do not add `unwrap()` on HTTP responses in non-test code (already `?`).

---

## 15. Test coverage

**Pass with gaps.**

Present (not ignored; `make prove` runs them):

- CLI contract, discover, DRS/WES verify, profiles, adapters
- Compare CLI, schema validation of generated JSON + evaluator example
- Threat model / adversarial HTTP, security CLI, bench CLI
- Unit tests in model, identity, compare, run_identity, diagnostics, report

Absent:

- No `cargo llvm-cov` / tarpaulin job, no coverage badge (do not fake a percentage).
- TES/TRS/htsget **execution** untested because it is not implemented (discovery only). Honest.
- Live Ferrum is `make test-live`, not prove.

**Proposed fixes**

1. Do not add a coverage number to README without a job.
2. Optional later: llvm-cov on CI with a floor that does not block single-steward velocity. Not certification.

---

## 16. Schema compatibility

**Pass for `helix verify`. Gap for other JSON.**

- Frozen file: `schemas/helix-verification-v1.json`, `additionalProperties: false`.
- CI: `tests/schema_verify.rs`.
- Policy: [SCHEMA.md](SCHEMA.md). Exception: optional `fixture_version` on the same v1 id (compare identity, not HELIOS).
- Old files without `schema_version` / `fixture_version` deserialize to current defaults.

**Not frozen as JSON Schema**

- `helix compare` (`CompareReport` + run identity)
- `helix bench` (`BenchOutcome`)
- `helix security` (HelixTest `OverallReport`)

**Proposed fixes**

1. Before announcement: keep verify v1 as the only frozen schema. Say so in README/SCHEMA (already listed as not covering security/bench/compare).
2. Before a compatibility-sensitive tag: optional `schemas/helix-compare-v1.json` without HELIOS keys. Do not bump verify `schema_version` for that.
3. Never add `signature` / `ro_crate` / `pdf` to any Helix schema.

---

## 17. CLI compatibility

**Pass.**

Frozen verify argv/exits: [CLI_CONTRACT.md](CLI_CONTRACT.md). Binary `helix`, never `helios`. `--report` alias of `--format`. Profiles additive. Compare/security/bench documented as shipped, not the original verify freeze.

**Gaps**

- `helix security --format json` is still HelixTest `OverallReport` (`services` / `passed`). Call that out in any announcement so consumers do not treat it as `VerificationRun`.
- Reserved TES/TRS/htsget command namespaces: not implemented.

**Proposed fixes**

1. Announcement / Release notes: four commands, two JSON shapes (verify `VerificationRun` vs security `OverallReport`).
2. Do not rename `helix verify` or change lowercase JSON status strings.

---

## Proposed fix order (human, not this review)

**P0 — before any external announcement**

1. Commit the working tree that evaluators are told to clone; wait for CI green on `main`.
2. Add `CODE_OF_CONDUCT.md` (HelixTest text).
3. Add issue forms + PR template + CODEOWNERS.
4. Rewrite [DEPENDENCY.md](DEPENDENCY.md) so it matches `Cargo.lock`.
5. `publish = false` on the crate.
6. Still **do not tag** until a human requests `v0.1.0`.

**P1 — before first git tag**

1. Clippy `--locked`; CI HelixTest SHA from VERSIONS.lock; workflow `permissions`.
2. SECURITY.md + GitHub private vulnerability reporting.
3. CONTRIBUTING → CoC + pin SHA.
4. Stale ENGINEERING_AUDIT banner.
5. Optional: discovery `.expect` → ERROR/bail.

**P2 — after tag, still not HELIOS**

1. Release-binaries workflow for `helix` (sha256, not signed evidence).
2. Compare JSON Schema if helix-action consumers need it.
3. `require-helixtest.sh` fail on SHA mismatch.
4. MSRV job or honest `rust-version`.

---

## Explicit non-goals (do not “fix” these in Helix)

- Signed audit trails, RO-Crate, PDF, evidence packs → [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)
- Ferrum production / clinical-pilot claims
- GA4GH certification language
- Merging HelixTest into this git root (D1)
- crates.io publish while HelixTest is a path dependency

---

## What this review did not do

- Did not create a git tag, GitHub Release, or `cargo publish`.
- Did not push or change remotes.
- Did not enable GitHub repository settings (private vuln reporting, discussions).
- Did not re-measure line coverage.
- Did not treat green tests as certification.
