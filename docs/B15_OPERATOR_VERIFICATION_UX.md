# B15 — Operator Verification UX & Evidence Contract

Helix is HelixTest becoming a standalone VERIFY CLI. This document records B15: an independent technical operator must be able to run Helix against a supported DRS target, understand what was selected, tested, verified, and left unevaluated, and retain a durable evidence artifact — without reading Helix source.

It is a productization and evidence-contract gate. It is not a new verification-capability gate. It does not expand DRS coverage, add a standard or version, add authorization, add HELIOS, or alter B11/B12/B14 identities.

**Vertraue mir nicht, vertraue dem Code.**

B15 does not reopen B13.

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
```

```text
authorization = UNEVALUATED
```

Schema revision: **helix-verification-v1 unchanged**. Standing remains computed, not stored.

Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). Durability: [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md). Coverage: [COVERAGE.md](COVERAGE.md). Claims: [CLAIMS.md](CLAIMS.md).

---

## 1. Executive verdict

```text
PASS WITH FINDINGS
```

An independent operator can discover the versioned DRS 1.4.0 workflow from `--help` and [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), retain `helix-verification-v1` JSON, reload it with `helix inspect`, and distinguish PASS / SKIP / FAIL from `ga4gh_requirement` VERIFIED / NOT_VERIFIED, and current verifier evidence from historical observation.

Presentation defects that would have collapsed those distinctions are fixed (static NOT_VERIFIED claim line; FAIL/SKIP attribution hidden in JSON; standing not operator-visible; pack/execution identity missing from human Standards; `--help` silent on the versioned path). B14 tamper and historical semantics are unchanged.

Findings that remain are documented, not hidden: live Docker targets were not running in this session (historical B12 JSON used); `helix verify` exit 0 remains check-PASS, not VERIFIED (CLI freeze, now stated); standing is not a JSON field (B14 design); unsigned consistent restamp remains a HELIOS gap.

---

## 2. Question under test

> Can an independent technical operator run Helix against a supported DRS target, understand exactly what was selected, tested, verified, and left unevaluated, and retain a durable evidence artifact — without needing to read Helix source code?

---

## 3. Repository state

Read from the repository at B15 proof time. Not copied from the prompt.

| Identity | Value | Source |
|----------|--------|--------|
| Helix HEAD at B15 start (B14 clean) | `dfd7bede076ab332f094cc5ef9704e42116f97c1` | `git rev-parse HEAD` before B15 edits |
| Helix working tree at B15 start | clean | `git status` |
| HelixTest SHA | `1baddfd3d75f01dc7c149074a785616fa014c725` | `VERSIONS.lock` / sibling checkout |
| Checker | `helixtest-drs:18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a` | `src/live_evidence.rs` `PINNED_CHECKER_ID` |
| DRS 1.4.0 release commit | `36145d389e0a454428d1dac5c4a30870995fdd7c` | `PINNED_RELEASE_COMMIT` |
| Pack | `c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89` | `PINNED_PACK_INTEGRITY_SHA256` |
| Schema document | `3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608` | `PINNED_SCHEMA_DOCUMENT_SHA256` |
| Schema component `DrsObject` | `b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003` | `PINNED_SCHEMA_COMPONENT_SHA256` |
| Binding | `72da037c4ce2383f116bf195507fe1b45c60d6917acf5a87c6e5bba7043c69e2` | `PINNED_BINDING_ID` |
| Catalog | `03ce38f690ed679ff967e636bb037e0ed4ebe42f2cbde62766921ad7eae96ac1` | `PINNED_CATALOG_ID` |
| Coverage | `082f63c9eec7472a66f8121a66e28ffb7f68680791f7bb5ea5ef47441d18f08c` | `PINNED_COVERAGE_ID` |
| Execution identity | `ee00e1a49e6b3f7d47314bde77738faf4b6cec4d3325dfe56d139809ea97037e` | `PINNED_EXECUTION_ID` |

These pins match the live B12 JSON (`local/b12/starter-kit.json`, `local/b12/bento.json`). HelixTest was not modified.

B15 source edits land on a later Helix commit. After that commit, `helix_git_sha` on a *new* verify run cites that SHA. Historical B12 JSON has `helix_git_sha: null` and classifies as `historical_observation`. That file is **not** restamped.

---

## 4. Operator journey

Exact path (no invented commands):

```text
helix verify --help
  → documents: default is unversioned; DRS 1.4.0 needs --standard drs --version 1.4.0;
    PASS ≠ VERIFIED; exit 0 ≠ VERIFIED; retain JSON; helix inspect FILE

helix standards list --supported-only
  → ga4gh.drs.1.4.0 (SUPPORTED is not VERIFIED)

NO_COLOR=1 helix verify URL \
  --standard drs --version 1.4.0 \
  --target-id YOUR-ID \
  --target-kind real-independent-local-implementation \
  --drs-object-id YOUR-OBJECT-ID \
  --format json > verify.json

NO_COLOR=1 helix verify URL --standard drs --version 1.4.0 --format text
  → same VerificationRun, human presentation

helix inspect verify.json
  → standing, recomputed ga4gh_requirement, identities; does not rewrite the file
```

Implementation (forensic, not operator-required):

```text
helix verify
  → src/main.rs verify_cmd
  → VerifyOptions / VerifySelection
  → verify_with_options (src/verify.rs)
  → support gate, pack load, SpecSource, discovery, fixture, HelixTest
  → bind_run → check_run → evaluate → finalize_run
  → print_text / print_json (src/report.rs)

verify.json
  → parse_verification_run / serde VerificationRun
  → check_run(Load) / validate_artifact_consistency
  → classify_evidence (src/evidence.rs)
  → format_inspect_text / inspect JSON summary
```

Live Docker/HTTP (`127.0.0.1:4500` Starter Kit, `127.0.0.1:5000` Bento) was **down** during this session (`connect` failed). Two-target proof uses the existing gitignored B12 artifacts (`local/b12/*.json`, timestamps `2026-09-06T20:17:49Z` / `2026-09-06T20:17:51Z`). Those files were not rewritten. In-process mock DRS proves the same operator text/JSON contract (`tests/b15_operator_ux.rs`).

---

## 5. CLI evidence

`helix verify --help` (B15-T1) states:

- default `helix verify URL` does not select a GA4GH pack;
- versioned command: `--standard drs --version 1.4.0 --format json`;
- PASS is a check outcome; VERIFIED is a derived claim; exit 0 is not VERIFIED;
- `--drs-object-id` is test input, not a GA4GH requirement;
- retain JSON and `helix inspect FILE`.

Human `HELIX VERIFICATION` (`src/report.rs` `format_verify_text`) now shows, without reading source:

| Operator question | Text field |
|-------------------|------------|
| Target | `Target:` url, `target_id`, `target_kind`, implementation name/version, `target_execution_id` |
| Selected standard/version | `Standards:` `standard`, `selected: …`, `selected_version` |
| Detected vs selected vs verified | `Versions (these four must not be collapsed):` declared / detected / selected / verified |
| Release / immutable source | `standards_source_commit`, `release_class` when present |
| Support | `support_status`, `selection_status` |
| Checker / pack / schema / binding / catalog | `checker`, `pack_integrity_sha256`, `schema_document_sha256`, `schema_component_sha256`, `binding`, `catalog` |
| Execution identity | `execution_id` |
| Fixture | `DRS fixture:` `object_id`, `expected_sha256`, `checksum_mode` |
| Checks | `Results:` id, code, name, PASS/FAIL/SKIP/ERROR, message, `attribution` |
| Claims | `Claims:` recomputed; `ga4gh_requirement` in Standards as VERIFIED or NOT_VERIFIED |
| Coverage / unevaluated | `Coverage:` `state`, `unevaluated:` list with `standard_role` |
| Standing | `Evidence standing:` computed for this binary |
| Not certification | header + Standards footer |

`declared` is the operator-declared **GA4GH** version (`target.identity.declared.standard_version`), not the image/implementation tag. Implementation version stays under `Target:`.

`check_outcome` (PASS/FAIL) is labeled separately from `ga4gh_requirement`. The line `check_outcome PASS is not ga4gh_requirement VERIFIED` is printed.

`helix inspect` prints `HELIX EVIDENCE INSPECT` with standing, recomputed claim, coverage state, and identities. Inspect JSON is a summary (`not_helix_verification_v1: true`), not a second `helix-verification-v1` document.

---

## 6. JSON evidence contract

`--format json` remains `schema_version: helix-verification-v1`. No schema bump. Standing is **not** serialized (`evidence_standing` absent; B15-T27).

| Meaning | JSON path |
|---------|-----------|
| Schema | `schema_version` |
| Standard / versions | `standard_selection.standard`, `.detected_version`, `.selected_version`, `.verified_version`, `.requested_version`, `.selection_status` |
| Release / source | `standard_selection.standards_source_commit` |
| Pack / schema | `.pack_integrity_sha256`, `.schema_document_sha256`, `.schema_component_sha256`, `.schema_entry` |
| Checker / binding / catalog | `.checker_id`, `.binding_id`, `.catalog_id` |
| Coverage / execution | `.coverage_id` / `coverage.*`, `.execution_id`, `.target_execution_id` |
| Target | `target.url`, `target.identity.{target_id,target_kind,implementation_name,implementation_version,endpoint,declared,detected,verified}` |
| Fixture | `drs_fixture.{object_id,expected_sha256,source,checksum_mode}` |
| Checks | `executed[]` / `skipped[]`: `id`, `code`, `name`, `status`, `message`, `attribution`, `failure`, `diagnostic`, `traceability` |
| Claims | `claims[]` (recomputed on load; not authority) |
| Join | `claim_join` (identities + check statuses + derived claims) |
| Coverage report | `coverage.{state,coverage_id,unevaluated,out_of_scope,required_complete,rows}` |
| Helix provenance | `helix_version`, `helix_git_sha`, `helix_git_dirty`, `timestamp` |
| HelixTest | `helixtest_version`, `helixtest_sha` (checker digest), `helixtest_git_sha` |

Interpretation does not require mutable external state. `helix inspect` re-derives claims, coverage, and standing from the file plus **this** binary’s compile-time git SHA.

---

## 7. Coverage presentation

B11 contract unchanged. Human `Coverage:` prints `coverage_id`, `state`, `required_complete` (derived), then:

```text
UNEVALUATED is not OUT_OF_SCOPE. PASS is not VERIFIED.
Partial coverage means unevaluated operations are not part of the VERIFIED claim.
```

Unevaluated and out-of-scope ids include `standard_role` in parentheses.

Authoritative unevaluated list (from compiled contract / live JSON `coverage.unevaluated`):

- `drs.op.service_info`
- `drs.op.objects_bulk`
- `drs.op.access`
- `drs.op.access_bulk`
- `drs.op.object_options`
- `drs.op.object_post_passport`
- `drs.security.authorization`
- `drs.checksum.types.other`
- `drs.object.bundle_contents`
- `http.tls.as_drs_requirement`
- `http.client.timeouts`

Out of scope (not silently verified): `helix.helios`, `helix.bench.as_verification`, `target.non_drs_http`, `http.discovery.redirect_follow`.

Honest catalog PASS is `coverage.state = partial`. Starter Kit required-row FAIL/SKIP is `blocked`. Authorization stays `UNEVALUATED`.

---

## 8. Claim presentation

| Layer | What it is | Where |
|-------|------------|--------|
| Observation | Detected version, HTTP facts, fixture bytes | `detected_version`, diagnostics, `drs_fixture` |
| Tested | Individual check status | `executed[]` / `Results:` PASS/FAIL/SKIP/ERROR |
| Verified | Derived claim after `evaluate` + join | `claims[]`, `verified_version`, text `ga4gh_requirement` |
| Not verified | Predicates failed or incomplete | `NOT_VERIFIED`, empty `verified_version` |
| Unevaluated | Compiled coverage remainder | `coverage.unevaluated` |
| Standing | Current vs historical vs invalid | `helix inspect` / `Evidence standing:` — not JSON |

A PASSING check cannot manufacture `ga4gh_requirement = verified` (B15-T9, B15-T25). Unversioned `helix verify URL` can exit 0 with six NOT_VERIFIED claims. Versioned mock DRS 1.4.0 can be technically VERIFIED with `coverage.state = partial`. Starter Kit remains NOT VERIFIED despite HLX-DRS-001/005/006 PASS.

---

## 9. Durability / tamper results

B14 machinery is reused. B15 adds operator inspect + presentation tests.

| Test | Result |
|------|--------|
| B15-T15 persisted JSON reloads | `parse_verification_run` + `helix inspect` succeed on a current mock artifact |
| B15-T16 formatting mutation | pretty-print / field reorder: `validate_artifact_consistency` holds; semantic identity unchanged |
| T3 current provenance | mock verify from this binary → `current_verifier_evidence` when `HELIX_GIT_SHA` is compiled in |
| B15-T21 / T4 historical | B12 JSON (`helix_git_sha` absent) → `historical_observation`; inspectable; `current_verifier_evidence: no` |
| B15-T17 / T5 forged claim | mutating `ga4gh_requirement` / `verified_version` without observations → integrity fail / inspect invalid |
| B15-T18 / T6 forged check | FAIL/SKIP rewritten to PASS → rejected |
| B15-T19 / T7 forged coverage | unevaluated removed / state changed → rejected |
| B15-T20 / T8 target relabel | target id changed, observations kept → rejected |
| T9 fixture relabel | covered by B14 fixture identity in join; B15 inspect of checker-id tamper exits 1 |
| T10 verifier identity | mutated `checker_id` → `helix inspect` exit 1, standing `invalid` |

Unsigned restamp of a *consistent* observation+join with a matching `helix_git_sha` remains undetectable without HELIOS (B14 finding; not weakened).

---

## 10. Starter Kit result

Artifact: `local/b12/starter-kit.json` (gitignored operator output). Not restamped. Live HTTP `http://127.0.0.1:4500` was not reachable this session.

| Field | Observed |
|-------|----------|
| `target_id` | `ga4gh-starter-kit-drs-0.3.2` |
| `target_kind` | `real_independent_local_implementation` |
| `implementation_name` | `ga4gh-starter-kit-drs` |
| Image / digest (independence record) | `ga4gh/ga4gh-starter-kit-drs:0.3.2` / `sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1` |
| detected | `1.3.0experimental` |
| selected | `1.4.0` |
| verified | `null` |
| `ga4gh_requirement` | `not_verified` |
| `coverage.state` | `blocked` |
| fixture | `b8cd0667-2c33-4c9f-967b-161b905932c9` (no independent sha256) |
| `execution_id` | `ee00e1a49e6b3f7d47314bde77738faf4b6cec4d3325dfe56d139809ea97037e` |
| `target_execution_id` | `816eb8c7b62252ae0f149da60feb8faa1ee8262219779fad882de2af329f2021` |
| `helix_git_sha` | absent → standing `historical_observation` |

Checks:

| Code | Status | Attribution / reason |
|------|--------|----------------------|
| HLX-DRS-001 | PASS | — |
| HLX-DRS-002 | FAIL | `target_failure` (`access_methods` missing) |
| HLX-DRS-003 | SKIP | `target_configuration_failure` / `fixture_unavailable` |
| HLX-DRS-004 | SKIP | `target_configuration_failure` / `fixture_unavailable` |
| HLX-DRS-005 | PASS | — |
| HLX-DRS-006 | PASS | — |

Human report (B15-T23): `NOT_VERIFIED`, `detected: 1.3.0experimental`, `selected: 1.4.0`, `verified: (none)`, `declared: (none)` (image `0.3.2` is not printed as a DRS version). This is not “DRS 1.4.0 verified.”

Observed `helix inspect local/b12/starter-kit.json` (this binary; file not restamped):

```text
Standing:
  historical_observation
  current_verifier_evidence: no
Claim:
  ga4gh_requirement: NOT_VERIFIED
  verified_version: (none)
  coverage.state: blocked
  detected_version: 1.3.0experimental
  selected_version: 1.4.0
  target_id: ga4gh-starter-kit-drs-0.3.2
  helix_git_sha: (none)
```

---

## 11. Bento result

Artifact: `local/b12/bento.json`. Not restamped. Live HTTP `http://127.0.0.1:5000` was not reachable this session.

| Field | Observed |
|-------|----------|
| `target_id` | `bento-drs-0.21.5` |
| `target_kind` | `real_independent_local_implementation` |
| `implementation_name` | `bento_drs` |
| source commit (independence record) | `1dc55ebea90185b1fec2c78c8c52909dd0ca889e` |
| detected / selected / verified | `1.4.0` / `1.4.0` / `1.4.0` |
| `ga4gh_requirement` | `verified` |
| `coverage.state` | `partial` |
| fixture | `f23b1635-4a65-40fa-8b29-75b4734b602a` / sha256 `6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1` |
| `execution_id` | same as Starter Kit |
| `target_execution_id` | `9653706c89a258511421448452e2405de5b2cf96549b6308120916547f264641` (distinct) |
| standing | `historical_observation` |

HLX-DRS-001–006 all PASS. WES rows SKIP (`unsupported_test`; standard not selected). Unevaluated list identical to Starter Kit. VERIFIED is the existing claim-integrity result under partial coverage — not “full DRS” and not certification.

The same operator workflow produced two different standings because observations differ (schema/`access_methods`, fixture availability), not because of target-specific verification logic.

Observed `helix inspect local/b12/bento.json`:

```text
Standing:
  historical_observation
  current_verifier_evidence: no
Claim:
  ga4gh_requirement: VERIFIED
  verified_version: 1.4.0
  coverage.state: partial
  detected_version: 1.4.0
  selected_version: 1.4.0
  target_id: bento-drs-0.21.5
  fixture.expected_sha256: 6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1
  helix_git_sha: (none)
```

---

## 12. Documentation audit

Inspected: `README.md`, `docs/OPERATOR_VERIFY.md` (new), `docs/REPORT.md`, `docs/CLI_CONTRACT.md`, `docs/FOR-EVALUATORS.md`, `docs/evaluator-pack/{README,commands,interpret}.md`, `docs/TRUST.md`, `docs/ARCHITECTURE_GUARDRAILS.md` (rule 20), `docs/CLAIMS.md`, `docs/COVERAGE.md`, `docs/B14_EVIDENCE_DURABILITY.md`, `CHANGELOG.md`.

Operator can find: prerequisites, supported DRS 1.4.0, target/fixture flags, exact verify command, PASS vs VERIFIED, partial coverage, artifact location, `helix inspect`, historical vs current, no GA4GH certification.

`helix inspect` is additive. `helix verify` exit 0/1/2 freeze is unchanged and now stated in help, report header, CLI_CONTRACT, and OPERATOR_VERIFY.

---

## 13. Test results

Commands (Helix root; `CARGO_TARGET_DIR=/Users/SynapticFour/devel/b71b-clean/target`):

```bash
cargo fmt --all -- --check
# exit 0

cargo test --offline --locked --test b15_operator_ux -- --test-threads=1
# test result: ok. 31 passed; 0 failed

cargo test --offline --locked --all-targets
# all crates/bins/tests: ok (0 failed). Includes b15_operator_ux 31 passed.

cargo clippy --offline --locked --all-targets --all-features -- -D warnings
# Finished `dev` profile; Helix clippy exit 0 (-D warnings). HelixTest has an unrelated tokio_retry deprecation warning.

./scripts/prove.sh
# prove: docs OK
```

B15 matrix: T1–T28 plus `b15_inspect_json_is_not_verification_schema` and `b15_tamper_via_inspect_exits_1`. Live T23/T24 executed on this machine (`local/b12/*.json` present). CI without those files skips T23/T24.

Product-language search (`certified`, `certification`, `fully compliant`, `production ready`, `guaranteed`, …): remaining “certification” hits are negations (“not GA4GH certification”, reserved `helix certify` never shipped). No output path claims official GA4GH certification (B15-T28). TRUST.md “no second implementation is certified” is a prohibition, not a claim.

---

## 14. Findings

### HIGH — static Standards claim line (fixed)

- **Location:** `src/report.rs` `format_standards_section` (pre-B15).
- **Evidence:** text printed `verification_claim: NOT_VERIFIED unless claims[] says otherwise` even when `evaluate` returned VERIFIED; `target_result: PASS` read as verification.
- **Impact:** operator could treat a PASSING check suite as the absence of a VERIFIED claim, or treat PASS as VERIFIED.
- **Fixed:** `check_outcome` vs actual `ga4gh_requirement`; explicit “PASS is not VERIFIED” lines. Tests T9, T10, T25, T28.

### MEDIUM — FAIL/SKIP attribution not in human report (fixed)

- **Location:** `format_result_block`; JSON already had `attribution`.
- **Evidence:** forensic audit: operator reading text only could not see `target_failure` vs `target_configuration_failure`.
- **Fixed:** `attribution: …` on the result block. T5, T6.

### MEDIUM — standing and pack/execution identity not in operator text (fixed)

- **Location:** B14 standing computed but unpublished; Standards omitted `pack_integrity_sha256` / `execution_id`.
- **Fixed:** `format_standing_section`; Standards + inspect identity list. T13, T14, T21, T27.

### MEDIUM — `--help` / docs did not expose the versioned workflow (fixed)

- **Location:** `src/main.rs` verify `about`; missing `OPERATOR_VERIFY.md`.
- **Fixed:** help `after_help`, new operator doc, README / evaluator-pack / CLI_CONTRACT / REPORT / FOR-EVALUATORS. T1.

### MEDIUM — `declared` printed implementation tag as a GA4GH version (fixed)

- **Location:** `format_standards_section` Versions block.
- **Evidence:** would have shown Starter Kit `declared: 0.3.2` beside `detected: 1.3.0experimental` / `selected: 1.4.0`.
- **Impact:** operator could treat an image tag as a declared DRS version.
- **Fixed:** `declared` uses `identity.declared.standard_version`. T3, T23.

### OBSERVATION — live targets down this session

- HTTP 4500/5000 not listening. Two-target proof uses historical B12 JSON. Identities match pins. Files not restamped.

### OBSERVATION — exit 0 is check-PASS, not VERIFIED

- Frozen `helix verify` CLI contract. Documented in help, report, OPERATOR_VERIFY, CLI_CONTRACT. Inspect exit 1 only for invalid artifacts. T10, T26.

### OBSERVATION — standing is not a JSON field

- B14: no schema bump. Inspect computes it. Historical B12 remains inspectable.

### MEDIUM (unchanged from B14) — unsigned consistent restamp

- A rewritten file that keeps observations+join and cites this SHA is still Current without HELIOS signatures. Out of Helix scope.

---

## 15. Explicit non-goals

B15 did **not** add:

- authorization verification or credentials;
- another GA4GH standard or DRS version;
- new DRS coverage (B11 list unchanged);
- signing, RO-Crate, PDF, or HELIOS integration;
- result cache, telemetry, or remote upload;
- target ranking or production claims;
- official GA4GH certification language;
- a `verified=true` cosmetic flag;
- HelixTest or checker identity changes.

```text
authorization = UNEVALUATED
```

---

## 16. Final conclusion

**Yes, with the documented bounds.** An independent operator can run `helix verify --standard drs --version 1.4.0`, retain `helix-verification-v1` JSON, and from CLI + JSON + `helix inspect` see what was selected, which checks passed/failed/skipped and why, what is VERIFIED versus merely tested, what remains unevaluated, and whether the file is current verifier evidence or a historical observation.

The operator does not need Helix source for that interpretation. The operator does need the documented versioned flags (default verify stays unversioned). Exit 0 is still not VERIFIED. Partial coverage is still partial. Starter Kit is still not DRS 1.4.0 verified. Bento’s VERIFIED claim is still the existing predicate result, not certification. Authorization is still unevaluated. HELIOS still owns signed evidence.
