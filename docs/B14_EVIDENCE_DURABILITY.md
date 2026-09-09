# B14 — Verification Evidence Durability

Helix is HelixTest becoming a standalone VERIFY CLI. This document records B14: a persisted `helix verify` result must remain an honest, provenance-bound evidence object. It is not a DRS coverage expansion, not a new DRS implementation, not HELIOS, and not GA4GH certification.

**Vertraue mir nicht, vertraue dem Code.**

B14 does not reopen B13.

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
```

```text
authorization = UNEVALUATED
```

Schema revision: **helix-verification-v1 unchanged**. Standing is computed, not stored.

---

## 1. Decision

```text
PASS WITH FINDINGS
```

The durability architecture is correct: claims, coverage, and `verified_version` are derived; forged or stale persisted JSON cannot become a stronger *current* claim. Bounded limitations remain (unsigned restamp of consistent observations; historical standing does not authenticate a foreign `helix_git_sha`; no compile-time git ⇒ no Current standing). Those are documented below. They are not silent claim inflation.

---

## 2. Question

Can a Helix verification result remain an honest, provenance-bound evidence object across serialization, persistence, reload, verifier changes, target changes, fixture changes, and adversarial mutation—without trusting serialized claims, and without restamping historical B12/B12.1 artefacts?

---

## 3. Existing architecture

Pre-B14 the path was already evidence-shaped, not a pretty-print-only report:

```text
helix verify
  → bind_run (traceability)
  → guardrails::check_run (Emit)
  → claims::evaluate          (derived; not a VerificationRun field)
  → claim_integrity::finalize_run  (stamps verified_version, coverage, claim_join)
  → report::verify_json       (recomputes claims[] / claim_join / coverage into JSON)
  → compare::parse_verification_run → check_run_with(Load)
```

Facts the forensic inspection recorded:

- JSON is `VerificationRun` (`schema_version: helix-verification-v1`). It is inspectable evidence. It is **not** HELIOS (no signatures, RO-Crate, PDF).
- `claims[]` is injected at serialize time. `VerificationRun` has **no** `claims` field; serde drops `claims[]` on load. `verify_json` overwrites `value["claims"]` from `evaluate(run)`.
- `claim_join` and `coverage` **are** stored. `validate_claim_integrity` recomputes both.
- `CheckMode::Emit` called `validate_claim_integrity` (join, coverage, claims, identities, and Helix git SHA/dirty vs **this** binary).
- `CheckMode::Load` did **not** call claim integrity. Pre-B8 `docs/evaluator-pack/example-verify.json` has no `claim_join` and must still Load.
- Identity (`execution_id`, `coverage_id`, `target_execution_id`) is hashed from semantic inputs (`reproduction_tuple` / spec-join / target+fixture). JSON key order, whitespace, `timestamp`, hostnames, usernames, and filesystem paths are not identity inputs.
- There is no result cache. B14 adds none.
- Historical B12 live JSON has no `helix_git_sha`. Absence is allowed by integrity; `cites_this_verifier_build` is false.

B14 does not replace the claim evaluator. It classifies standing and applies artifact consistency on Load when join/coverage is present.

---

## 4. Changes

| Area | Change |
|------|--------|
| `src/claim_integrity.rs` | Split `validate_artifact_consistency` (join/coverage/claims/identities/SHA *shape*) from `validate_claim_integrity` (consistency plus SHA/dirty vs **this** build). |
| `src/evidence.rs` | `EvidenceStanding::{Invalid, HistoricalObservation, CurrentVerifierEvidence}`; `classify_evidence`; `is_current_verification`; `revalidate_evidence`. Standing is **not** serialized. |
| `src/guardrails.rs` | Load: if `claim_join` or `coverage` is present, require `validate_artifact_consistency`. Do **not** require this-build SHA match (historical inspectability). Pre-B8 example JSON without join still Loads. |
| `src/lib.rs` | `pub mod evidence`. |
| `tests/b14_evidence_durability.rs` | B14-T1–T24 plus combined forgery, evaluator-example Load, pin isolation. Live B12 files remain optional. |
| `schemas/helix-verification-v1.json` | **Unchanged.** |
| HelixTest | **Unchanged** (`1baddfd3d75f01dc7c149074a785616fa014c725`). |
| HELIOS | **Unchanged.** |
| B12/B12.1 JSON | **Not rewritten.** |
| Cache / telemetry / upload | **None.** |

DRS 1.4.0 identities are unchanged: pack `c3836145…`, schema document `3d8de69f…`, schema component `b27ef764…`, checker `helixtest-drs:18bf4a44…`, binding `72da037c…`, catalog `03ce38f6…`, coverage `082f63c9…`, execution `ee00e1a4…`, DRS release commit `36145d389e0a454428d1dac5c4a30870995fdd7c`.

---

## 5. Evidence model

| Object | Authority |
|--------|-----------|
| Observations (`executed[]` / `skipped[]` status, attribution, diagnostics) | **Authoritative input** (what was recorded). |
| Standard / verifier / target / fixture provenance fields | **Authoritative input** for identity and standing. |
| `execution_id` / `target_execution_id` / `coverage_id` / `binding_id` / `catalog_id` | **Derived** from semantic inputs; stored values are checked against recomputation. |
| `claims[]` / `verified_version` / `ga4gh_requirement` / `coverage.state` / `required_complete` | **Derived output.** Never trusted as input. Recomputed on emit, inspect, and Load (when join/coverage present). |
| `EvidenceStanding` | **Computed** on inspect. Not a JSON field. Not stored. |

A forged `claims[].status = verified` in JSON is dropped on deserialize. A forged `claim_join` / `coverage` / `verified_version` that disagrees with observations is `Invalid`.

---

## 6. Identity rules

| Identity | Inputs | Neutral (does not change it) | Must change / invalidate |
|----------|--------|------------------------------|---------------------------|
| `execution_id` | pack id, pack/schema hashes, checker id, schema entry/component | JSON formatting, timestamp, host, path, `helix_git_sha` | pack/schema/checker |
| `target_execution_id` | target id/kind/endpoint + spec-join fields + fixture object id / expected sha256 | formatting, timestamp, host user, `helix_git_sha` | target relabel, fixture mutation, spec identity |
| `coverage_id` | compiled coverage contract + checker + binding + pack hash | formatting, timestamp, target, `helix_git_sha` | coverage/checker/binding/pack contract |
| Helix git provenance | compile-time `HELIX_GIT_SHA` / dirty | not an identity input | mismatch ⇒ not Current (historical or integrity-fail for current use) |
| Fixture | `drs_fixture.object_id`, `expected_sha256` | formatting | object id / expected digest change without matching observations |

Representation (key order, whitespace, pretty vs compact) is not hashed.

---

## 7. Mutation results

Mutations are **without** `finalize_run` restamp unless noted. “Historical” means internally consistent and not attributable to this binary. “Invalid” means join/coverage/claims/identities cannot be re-derived.

| Mutation | Expected result | Observed |
|----------|-----------------|----------|
| formatting only | unchanged identity | unchanged (`execution_id` / `coverage_id` / `target_execution_id`) |
| Helix provenance (`helix_git_sha`) without restamp | invalid | Invalid (`claim_join` includes SHA from C2; paste-without-restamp cannot become Current) |
| Helix provenance (`helix_git_sha`) restamped to another commit | stale / not current | `HistoricalObservation`; `validate_claim_integrity` err; not Current |
| checker | stale/invalid | Invalid |
| binding | stale/invalid | Invalid |
| catalog | stale/invalid | Invalid |
| coverage id/state | stale/invalid | Invalid |
| pack hash | stale/invalid | Invalid |
| schema hash | stale/invalid | Invalid |
| target relabel | invalid | Invalid |
| fixture object id / expected checksum | invalid | Invalid |
| forged `verified_version` / claim status / coverage state | invalid | Invalid (`claims[]` alone is dropped; join/coverage forgery is Invalid) |
| removed required check | not verified / invalid | Invalid without restamp; after restamp: not `ga4gh_requirement` VERIFIED |
| added unknown check | invalid (cannot satisfy contract) | Invalid without restamp; with restamp after dropping a required check: not VERIFIED |
| FAIL→PASS / SKIP→PASS | not trusted | Invalid |
| attribution `target_failure` → `spec_failure` | not trusted | Invalid |
| historical B12 | preserved as historical | `helix_git_sha` absent; standing Historical; files not rewritten |

---

## 8. Historical evidence

B12/B12.1 live JSON under gitignored `local/b12/` (optional for prove):

- Not rewritten. Not restamped. Not given this Helix commit.
- `helix_git_sha` is absent ⇒ `cites_this_verifier_build` is false ⇒ `HistoricalObservation` when internally consistent.
- `validate_artifact_consistency` still applies (join/coverage present). `validate_claim_integrity` still passes on honest B12 files because **absence** of SHA is allowed; a *forged* SHA fails current integrity.
- Starter Kit and Bento share `execution_id` / `coverage_id` and differ in `target_execution_id`. Relabeling one as the other is Invalid.
- Historical JSON remains inspectable (`evaluate`, coverage pins). It is not Current verifier evidence.

Pre-B8 `example-verify.json` (no join) still Loads. Missing join is not treated as VERIFIED.

---

## 9. Security

- `verify_json` runs `redact::redact_text` on the serialized document (Authorization / Bearer / Basic / JWT-shaped tokens / URL userinfo / `HELIX_HMAC_SECRET`).
- `helix verify` still has no credential field. B13 remains deferred: `authorization = UNEVALUATED`.
- Deterministic identities do not include secrets, timestamps, hostnames, usernames, or filesystem paths.
- No telemetry, upload, remote evidence store, or cloud callback.
- Residual: a diagnostic string that does not match the redaction patterns could persist. Existing redaction tests remain. B14-T23 asserts a Bearer/Authorization value does not survive durable JSON.

---

## 10. Regression validation

Focused B14 (29 passed, including optional live B12 T20–T22 when `local/b12/` is present):

```text
cargo test --offline --locked --test b14_evidence_durability
# test result: ok. 29 passed; 0 failed; 0 ignored
```

Full required suite (this worktree, `CARGO_TARGET_DIR` unchanged):

```text
cargo fmt --all -- --check
# exit 0

cargo test --offline --locked --all-targets
# all packages: 0 failed; 0 ignored (lib 288 passed; b14 29 passed)

cargo clippy --offline --locked --all-targets --all-features -- -D warnings
# exit 0

./scripts/prove.sh
# prove: docs OK
```

No flaky B14 tests. HelixTest pin remains `1baddfd3d75f01dc7c149074a785616fa014c725`. No result cache. Schema remains `helix-verification-v1`. Compile-time `helix_git_sha` is the Helix commit after this change lands on a clean tree (`build.rs` / `b12_1_t5`).

---

## 11. Findings

### HIGH

None that allow forged/stale JSON to become a stronger **current** claim.

### MEDIUM

1. **Consistent restamp is unsigned.** If an adversary changes observations *and* calls `finalize_run` (or regenerates join/coverage/`verified_version` together), Helix cannot distinguish that from a genuine re-run. That is the HELIOS boundary: Helix does not sign artefacts. B14 tests mutate **without** restamping. Do not add signatures to Helix to close this.

2. **Pasted `helix_git_sha` without restamping `claim_join` is Invalid (C2).** A restamped file whose SHA is some other commit remains `HistoricalObservation` (historical inspectability). The recorded SHA is not authenticated as “that other commit” without HELIOS. Pasting this binary’s SHA **and** restamping the join can still look Current — same unsigned-restamp bound as finding 1.

3. **Missing compile-time git metadata.** If `HELIX_GIT_SHA` is empty, `cites_this_verifier_build` is always false, so no run is Current. Fail closed for current attribution; historical inspectability remains.

### LOW

1. Standing is computed, not persisted. Two inspectors with different binaries will classify the same file differently (Current vs Historical). That is intended.
2. Extra unknown check IDs are allowed as additional observations after an honest restamp; they cannot replace a missing required catalog check or flip `UNEVALUATED` coverage rows to PASS (existing B11 contract).
3. `is_current_verification` means “this binary may treat the artefact as current verifier evidence,” not “`ga4gh_requirement` is VERIFIED.” An incomplete catalog can be Current and still NOT VERIFIED.

---

## 12. Remaining boundary

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
```

```text
authorization = UNEVALUATED
```

`drs.security.authorization` stays unevaluated. `AUTHZ_ENABLED` is not evidence. Helix does not send credentials. B14 does not fill authorization.

HELIOS remains responsible for signed audit trails, reproducibility evidence, RO-Crate, and PDF. B14 is Helix verification evidence integrity and semantic durability only.
