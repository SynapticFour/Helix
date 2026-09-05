# Helix verification schema (v1)

Frozen machine-readable contract for **`helix verify --format json`**. File: [`schemas/helix-verification-v1.json`](../schemas/helix-verification-v1.json). Rust type: `VerificationRun` (`src/model.rs`).

HelixTest already runs DRS and WES checks. This schema productizes that JSON. It is **not** a HELIOS evidence schema (no signature, RO-Crate, audit trail, PDF). Results are not GA4GH certification.

Human text is a projection of the same document ([REPORT.md](REPORT.md)). JSON status is lowercase; text prints PASS/FAIL/SKIP/ERROR. VERIFIED / NOT_VERIFIED in the Claims section comes only from `claims[]` ([CLAIMS.md](CLAIMS.md)).

---

## Document id

| Field | Value |
|-------|--------|
| `schema_version` | `helix-verification-v1` (const) |
| JSON Schema `$id` | this repo’s `schemas/helix-verification-v1.json` |

Helix always **emits** `schema_version`. Files produced before this freeze omit it; Helix **deserializes** a missing field as `helix-verification-v1` so `helix compare` still reads them.

---

## Concept → JSON name

The operator questions use “services” and “checks”. v1 **does not** use those keys. HelixTest `OverallReport` already owns `services` and a `passed` boolean; `helix verify` must not emit that shape.

| Concept | v1 JSON | Notes |
|---------|---------|--------|
| Schema version | `schema_version` | const `helix-verification-v1` |
| Helix version | `helix_version` | crate version |
| HelixTest pin | `helixtest_version` / `helixtest_sha` | optional |
| Profile | `profile` | `generic` or `ferrum` |
| Fixture catalog | `fixture_version` | `helix-fixtures-v1`; compare identity only ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not HELIOS |
| Target | `target.url` plus optional `target.identity` | origin; identity is B4 ([TARGETS.md](TARGETS.md)). Optional on old files |
| Timestamp | `timestamp` | RFC3339 UTC seconds `…Z`. Wall clock, not a signature |
| Services | `discovery[]` | `present` / `testable`; not a pass |
| Checks that ran | `executed[]` | `status` is `pass`, `fail`, or `error` |
| Checks skipped | `skipped[]` | `status` is `skip` only |
| Status | `status` | `pass` \| `fail` \| `skip` \| `error` |
| Failure code | `failure.code` | on fail/error; repeats catalog `code` |
| Diagnostic | `diagnostic` | optional on fail/error; **possible_causes**, never `cause` |
| Standard version | per-check `standard` / `requested_version` / `detected_version` / `selected_version` / `verified_version` / `standards_registry_entry` / `standards_source_commit`; run `standard_selection` | Four version facts must not collapse. Null when empty. [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md) |
| Traceability | per-check `traceability` | `category` / `check_kind` / `claim_scope` / `authority` / `layer` / `request`. Taxonomy: [TAXONOMY.md](TAXONOMY.md). Layers: [BEHAVIOR.md](BEHAVIOR.md). Not a MUST |
| Layers | `layer` / `layer_summary` | SCHEMA vs BEHAVIOR vs SECURITY vs INTEROPERABILITY. No percentage |
| Claims | `claims[]` | Six kinds. `verified` only when every predicate holds. [CLAIMS.md](CLAIMS.md) |
| Summary | `summary` | counts, not a score |

There is no `checks` array and no root `services` array in v1.

---

## Status (PASS / FAIL / SKIP / ERROR)

JSON enum is **lowercase**. These four are distinct. Do not add `passed: true`.

| JSON | Text | Meaning |
|------|------|---------|
| `pass` | PASS | Assertion held |
| `fail` | FAIL | Target behaved wrong |
| `skip` | SKIP | Not executed. **Never** a pass |
| `error` | ERROR | Helix could not run the check |

`fail` and `error` must not be collapsed. Skip cannot be stored in `executed`. Pass/fail/error cannot be stored in `skipped`.

---

## Backwards compatibility

**Breaking** (requires `helix-verification-v2` and a new schema file):

- Rename or remove a required field (`schema_version`, `helix_version`, `timestamp`, `target`, `discovery`, `executed`, `skipped`, `summary`)
- Change `status` strings
- Add a `passed` boolean, or emit HelixTest `OverallReport` (`services`, per-test `passed`)
- Add HELIOS fields (`signature`, `ro_crate`, `audit_trail`, evidence, PDF)
- Move skip rows into `executed`, or allow `status: skip` there
- Change assigned Helix `id` / `code` pairs ([TEST_IDENTITY.md](TEST_IDENTITY.md))

**Additive / non-breaking for consumers** (still a schema-file change if this v1 document uses `additionalProperties: false`):

- New optional property that consumers ignore
- New check rows, new `discovery` services (e.g. later TES execution)
- New optional diagnostic on an existing fail row

This v1 schema sets `additionalProperties: false` so CI rejects accidental HELIOS keys and unknown renames. Adding an optional field means publishing a **new schema file** (for example `helix-verification-v1.1.json`) and a new `schema_version` const. Consumers of v1 should **ignore unknown properties** if they parse without this file.

**Exception (compare identity only):** `fixture_version` is an optional property on this same `helix-verification-v1` file. `schema_version` stays `helix-verification-v1`. It is Helix-owned compare metadata (`helix-fixtures-v1`), not a HELIOS envelope, not a required field for old files. Producers always emit it. Missing on deserialize → `helix-fixtures-v1`. Do not use this exception for signatures, RO-Crate, PDF, or other HELIOS fields.

**Exception (standard-version fields):** `standard_selection` and per-check `standard`, `requested_version`, `detected_version`, `selected_version`, `verified_version`, `standards_registry_entry`, `standards_source_commit` are optional on this same v1 file. `schema_version` stays `helix-verification-v1`. Producers always emit them (null when empty). Missing on old files deserializes as empty. `selected_version` is Helix’s choice, not a target declaration. `substituted` is always false. Not HELIOS.

**Exception (check traceability):** per-check `traceability` (`category`, `check_kind`, `claim_scope`, `authority`, `expected_behavior`, `implementation`, `untraceable_reason`, optional `related_source`, and pack identity fields used only when `category` is `normative`) is optional on this same v1 file. `schema_version` stays `helix-verification-v1`. Producers always emit it. Missing on old files deserializes as empty. `category` is the claim taxonomy; it must equal `check_kind`; `claim_scope` must match. `related_source` is an AVAILABLE pin hint, not a verified-against claim. `kind=normative` is empty in the shipped catalog. A fixture row cannot be serialized as `normative`. Not HELIOS. Not certification. [TAXONOMY.md](TAXONOMY.md), [TRACEABILITY.md](TRACEABILITY.md).

**Exception (check layers):** `layer`, `observed_response`, and run-level `layer_summary` are optional on this same v1 file. `schema_version` stays `helix-verification-v1`. Producers always emit them. `layer_summary` has no `percent` / `score` / `compliant` field. SCHEMA PASS is not BEHAVIOR PASS. [BEHAVIOR.md](BEHAVIOR.md).

**Exception (claims):** run-level `claims` is optional on this same v1 file. `schema_version` stays `helix-verification-v1`. Producers always emit six items computed from the rest of the document. Human VERIFIED text is generated only from this array. Missing on old files is not a silent pass. Not a score. Not HELIOS. [CLAIMS.md](CLAIMS.md).

**Helix producers:**

- Must emit `schema_version: helix-verification-v1` while this file is current
- Must emit `fixture_version` (`helix-fixtures-v1` today)
- Must emit the seven standard-version fields on every check row (null when empty)
- Must emit `traceability` on every check row (`category` / `check_kind` is not `normative` in the shipped catalog; `claim_scope` is never `ga4gh_requirement`)
- Must emit `layer` and `layer_summary` (no percentage)
- Must emit `claims` (six kinds; `not_verified` unless every predicate holds)
- Must not emit `services`, `checks`, `passed`, `signature`, `ro_crate`

Identical inputs (same binary, target, HelixTest pin, fixture catalog) produce identical JSON after replacing `timestamp`. Two comparable runs may also differ in Helix/HelixTest version; that is recorded identity, not a schema break ([RUN_IDENTITY.md](RUN_IDENTITY.md)).

---

## What this schema is not

- HELIOS / `helios-audit` evidence
- Certification, scoring, ISO 15189 / AI Act
- `helix security` JSON (still HelixTest `OverallReport`)
- `helix bench` / `helix compare` JSON (separate documents)

CI: `tests/schema_verify.rs` validates generated `helix verify` JSON against this file. Integrity constraints that schema cannot express (`verified_version` requires `selected_version` and join hashes; VERIFIED predicates) are enforced by `src/guardrails.rs` ([ARCHITECTURE_GUARDRAILS.md](ARCHITECTURE_GUARDRAILS.md)).
