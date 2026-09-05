# Helix public-repository audit

**Date:** 2026-09-05
**Audience assumed:** a technically competent stranger who has not heard of Synaptic Four, Ferrum, HELIOS, or HelixTest.
**Method:** Start at README and the public tree. Read architecture, registry, versioning, CLI help, schemas, examples, fixtures, tests, CI, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG. Look for overclaims. This file is a report, not a tag, not a GitHub Release, not GA4GH certification.

**Do not quote this audit as multi-implementation validation or as a SUPPORTED-pack verification.**

Helix is HelixTest becoming a standalone VERIFY CLI (pin **v0.1.3**). HELIOS (`helios-audit`) still owns signed evidence / RO-Crate / PDF. Ferrum is a reference HTTP target, not a Helix crate, and has no clinical pilot.

---

## Verdict for a stranger

They can **run a representative fixture test** and **see what Helix refuses to claim**. They cannot honestly treat a green `helix verify` as “verified against GA4GH DRS 1.5.0” or as independent-implementation evidence.

**Recommended release classification:** early-stage public source. Not a tagged release. Not crates.io. Not an external announcement that Helix verifies named GA4GH releases.

---

## Reviewer questions

| Question | Answer from this tree | Where |
|----------|----------------------|--------|
| What is Helix? | A CLI. Point it at a GA4GH HTTP origin you already run. It discovers APIs, then runs documented DRS and WES checks when TESTABLE. Check bodies come from HelixTest. | [README.md](../README.md), [evaluator-pack/explanation.md](evaluator-pack/explanation.md) |
| What is it NOT? | Not a server, not HELIOS, not GA4GH certification, not a Ferrum production/clinical claim, not completed multi-implementation validation, not a paid SKU. | README “What this is not”, [FOR-EVALUATORS.md](FOR-EVALUATORS.md) |
| Can they run it? | Yes, with friction: sibling HelixTest at `VERSIONS.lock` SHA, rustup **1.91.1**, `make fetch` then `make prove`. Homebrew `cargo` on PATH will ignore the toolchain file. | [INSTALL.md](INSTALL.md), [EVALUATOR_JOURNEY.md](EVALUATOR_JOURNEY.md) |
| Representative test? | `make verify-fixture` runs `helix verify` against an in-process DRS mock. Expected: five DRS PASS, WES SKIP (not mounted), exit 0. Claims remain NOT_VERIFIED. | [FIXTURES.md](FIXTURES.md), [evaluator-pack/commands.md](evaluator-pack/commands.md), `examples/verify_fixture.rs` |
| Supported standards? | DRS 1.4.0 only (`helix standards list --supported-only`). YAML is not sufficient. | [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md), `src/standards/support.rs` |
| AVAILABLE vs SUPPORTED? | AVAILABLE = pinned official bytes (DRS 1.4.0, DRS 1.5.0, WES 1.1.0). SUPPORTED = mapping + engine loads those bytes + fixture prove. Only AVAILABLE exists. | Registry §3.3, [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md) §8 |
| Exact specification provenance? | Registry `commit` + `vendor_path` + SHA-256. `helix standards show` / `validate`. Default `helix verify` does **not** load those bytes (HelixTest-vendored OpenAPI). | `standards/vendor/`, [TRACEABILITY.md](TRACEABILITY.md) §3 |
| Normative vs fixture? | Exactly one shipped check is `normative` (`drs.object.schema.openapi`). JSON `traceability.check_kind` / `claim_scope`. Domain `executed[].category` is a different field. | [TAXONOMY.md](TAXONOMY.md), `src/traceability.rs` |
| Why a result passed? | Check `message`, `status=pass`, taxonomy `kind` (fixture today). PASS is not a GA4GH MUST. | [REPORT.md](REPORT.md), [CLAIMS.md](CLAIMS.md) |
| Why a result failed? | `failure.code`, `diagnostic.expected` / `observed` / `possible_causes` (not a root-cause claim). | [DIAGNOSTICS.md](DIAGNOSTICS.md) |
| Unsupported claims? | Six `claims[]` rows, all NOT_VERIFIED on default/fixture runs. Forbidden sentences listed in TRUST. | [CLAIMS.md](CLAIMS.md), [TRUST.md](TRUST.md) “insufficient” table |
| Ferrum not required? | `Cargo.toml` has no Ferrum crate. CI and `make prove` use in-process fixtures. Live Ferrum is `make test-live` (opt-in). | `Cargo.toml`, [.github/workflows/ci.yml](../.github/workflows/ci.yml), [PROVE.md](PROVE.md) |
| HELIOS separate? | No HELIOS dependency. Binary `helix`, never `helios`. No `signature` / `ro_crate` / `pdf` on verify JSON. | [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md), [SCHEMA.md](SCHEMA.md) |
| Limitations? | Unversioned default verify; DRS 1.4.0 SUPPORTED is not VERIFIED; DRS 1.5.0/WES not supported; TES/TRS/htsget discovery-only; sibling HelixTest path dep; single steward; interop pending. | TRUST, [INTEROP.md](INTEROP.md), this file |
| Contribution rules? | [CONTRIBUTING.md](../CONTRIBUTING.md). No `CODE_OF_CONDUCT.md`, no issue forms, no CODEOWNERS. | CONTRIBUTING, this file § remaining blockers |
| How a GA4GH release becomes SUPPORTED? | Seven ordered steps. After step 3 = AVAILABLE. After step 7 = SUPPORTED. | Registry §4 |
| How a check becomes normative? | SUPPORTED pack, complete vendor tree loaded, assertion equals locator, extras split, catalog + tests updated in one change. | [TRACEABILITY.md](TRACEABILITY.md) §7 |
| Who reviews normative mappings? | No GA4GH board. Single steward may merge a PR that meets the artefacts list. Stewardship is not approval. | Registry §10.1, [IDENTITY.md](IDENTITY.md) |

---

## Surface-by-surface

### README

The previous sentence “Helix tests behavior against the GA4GH spec” was an overclaim. Replaced with the documented DRS/WES suite sentence that `scripts/prove.sh` greps. DRS 1.4.0 support is technical verification within declared coverage, not certification.

### Architecture documentation

[ARCHITECTURE.md](ARCHITECTURE.md) now leads with **as-built 2026-09-05** vs intended. Dated snapshots ([ENGINEERING_AUDIT.md](ENGINEERING_AUDIT.md), [OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md)) are labelled snapshots. CLI list includes `matrix` and `standards`. Stage 1 Ferrum-local **exit** is not claimed as a public artefact.

### Standards registry and versioning

Registry vocabulary is strong. Mode 2 auto-detect is specified at length; it is now labelled **Not shipped**. Default `helix verify TARGET` stays unversioned. `--standard drs --version 1.5.0` is `AVAILABLE_BUT_NOT_SUPPORTED`.

### CLI help

`helix --help` about-text: DRS/WES VERIFY CLI wrapping HelixTest. Not HELIOS. Not GA4GH certification. Subcommands match [CLI_CONTRACT.md](CLI_CONTRACT.md). Dummy HMAC help still says “NICHT FÜR PRODUKTION” (fixture label); English “not for production” is used in SECURITY.md and README.

### Schemas

Frozen: `helix-verification-v1`, `helix-standard-version-v1`, `helix-interop-matrix-v1`. Compare / bench / security JSON are not schema-frozen. Security JSON remains HelixTest `OverallReport`. No HELIOS keys.

### Examples, fixtures, tests, CI

- Example: `examples/verify_fixture.rs` / `make verify-fixture`.
- Evaluator example JSON: `docs/evaluator-pack/example-verify.json`.
- Fixtures: [FIXTURES.md](FIXTURES.md). Mutants: [MUTATION.md](MUTATION.md) (seven documented misses).
- Tests: `cargo test --locked --offline --all-targets` in `make prove`. JWT tests can flake under parallel `--lib`; not ignored.
- CI: checkout HelixTest at pin SHA, `make prove`, `make independent-verify`, `make verify-fixture`, clippy `--locked --offline -D warnings`, rustfmt. No Ferrum job.

### Contribution, security, licence, changelog, release notes

- CONTRIBUTING: issue-first, Apache-2.0, no HELIOS, no Ferrum pilot claims, pointer to registry §10.1.
- SECURITY.md: private email; not a scanner; no SLA.
- LICENSE: Apache-2.0.
- CHANGELOG: Unreleased, with a **current facts** banner so older chronological bullets are not read as today’s product.
- Release notes: **none**. No git tags, no GitHub Releases.

---

## Strengths

1. Fail-closed claim engine: honest DRS PASS is still NOT_VERIFIED.
2. AVAILABLE vs SUPPORTED is implemented in CLI, not only in prose.
3. Traceability catalog forbids `normative` until the chain exists; tests must stay red.
4. Fixture path does not require Ferrum, Docker, or credentials.
5. HELIOS surfaces are excluded from schemas and from prove greps.
6. Evaluator pack has no sales CTA and no telemetry claim.
7. Independent-verification doc states what is **not** bit-for-bit.
8. Interop matrix exists and marks external validation **pending**.

---

## Weaknesses

1. Two-repo clone plus exact HelixTest SHA is a real first-run cost. Path dependency means two clones with different sibling HEADs compile different engines (`require-helixtest.sh` warns, does not fail).
2. Docs volume is large. A stranger can still get lost in vision/roadmap/checkpoint audits if they skip README grouping.
3. `helix security` JSON is a different shape (`OverallReport`) than `helix verify`.
4. Mode 2/3 prose remains long; only banners stop a reader from thinking auto-detect is live.
5. German fixture label `NICHT FÜR PRODUKTION` remains on dummy files (intentional shared label with other repos).
6. MSRV `rust-version` 1.88 is untested; CI is 1.91.1.
7. No Code of Conduct, issue templates, PR template, or CODEOWNERS.
8. [HELIX_VISION.md](HELIX_VISION.md) still frames Helix as the “VERIFY pillar” next to other org products. README now explains enough to skip that file.

---

## Remaining blockers

Before a **tagged** `v0.1.0` or an external announcement that this is a GA4GH verification product:

| Blocker | Why it matters to a stranger |
|---------|------------------------------|
| No git tag; CHANGELOG is Unreleased | Clone of `main` is the product. There is no frozen release. |
| No SUPPORTED pack | Cannot claim a named GA4GH version. |
| No independent implementation JSON | Cannot claim multi-implementation validation. |
| HelixTest still not loading `standards/vendor` | Provenance pins are inspectable, not executed. |
| Community files missing (CoC, issue forms) | Fine for source inspection; weak for “open to contributors” messaging. |
| Working tree vs published `main` | Announcing features that are only local is a credibility failure. Commit and wait for CI on `main` before any announcement. |

Not blockers for **reading the source**: LICENSE, SECURITY, CONTRIBUTING, schemas, fixtures, CI prove.

---

## Overclaiming risks

| Risk | Status after this audit |
|------|-------------------------|
| “Tests behavior against the GA4GH spec” | Removed from README, FOR-EVALUATORS, architecture invariant 14, discovery/security/bench footers. Prove greps the replacement. |
| AVAILABLE described as SUPPORTED | CLI still fail-closes. Docs repeat empty OfficialSupported. |
| Planned Mode 2 described as shipped | Standard-versioning §4 and registry Mode B labelled **Not shipped**. |
| CHANGELOG older bullets contradict current DRS+WES | Current-facts banner at top of Unreleased. |
| Vision table “API behaves against published contract = Yes” | Replaced with documented-suite / no named-release rows. |
| Roadmap “Stage 1 in progress (DRS only)” | Updated: DRS+WES execute; Ferrum-local exit not a public artefact. |
| `Cargo.toml` “GA4GH conformance” | Description no longer says conformance. `publish = false`. |
| Multi-implementation validation | Still pending; README and INTEROP say so. |
| GA4GH-certified / approved | Forbidden in TRUST; prove greps README for “GA4GH certification” as a **not**. |
| Ferrum required | Still not in Cargo.toml or CI. |

A remaining residual: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) row “API conformance = Yes” is **heritage scope**, not a SUPPORTED claim. Read it with TRUST.

---

## Reproducibility status

**Technical (this repo):** after `make fetch`, `make prove` and `make independent-verify` run offline. Two fixture verifies match after stripping `timestamp`. Vendor SHA-256 is checked by `helix standards validate`. Not bit-for-bit JSON. Not HELIOS.

**Scientific / signed:** not in Helix. HELIOS.

**Binary reproducibility:** no release binaries, no SBOM, HelixTest is a path pin not `git+SHA` Cargo dep.

---

## External validation status

**Pending.**

`helix matrix` with no `--run` emits pending slots `ferrum` and `independent`. `independent_evidence` is false. In-process mocks are not a second implementation. Do not announce multi-implementation validation until an operator-labeled `independent_implementation` JSON exists beside a `reference_target` run.

---

## Recommended release classification

| Class | Fits? |
|-------|--------|
| Private / docs-only | No. Engine, tests, and CI exist. |
| Early-stage public source | **Yes.** Apache-2.0, single-steward, sibling HelixTest required, honesty docs in place. |
| Tagged `v0.1.0` | Not until `main` matches what README describes and a human requests the tag. |
| crates.io crate | No. Path dependency; `publish = false`. |
| GA4GH verification product announcement | **No.** No SUPPORTED pack, no normative checks, interop pending. |
| Multi-implementation validated | **No.** |

Judge it from the pins, the tests, and a recorded `helix verify --format json`. Do not trust this audit instead of those artefacts.
