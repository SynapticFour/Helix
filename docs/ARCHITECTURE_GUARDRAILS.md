# Architecture guardrails

Helix is HelixTest becoming a standalone VERIFY CLI, not a new test platform. This file is the **enforceable trust map**. Prefer a schema or a failing test over a contributor instruction.

A future contributor should have to **consciously violate** the trust model (delete a test, weaken a schema, bypass `check_run`) rather than accidentally doing so.

Not HELIOS. Not GA4GH certification. Do not ask anyone to trust Helix or its authors: [TRUST.md](TRUST.md).

---

## 1. How a rule is encoded

| Layer | When it runs | What it is for |
|-------|----------------|----------------|
| JSON Schema | `helix standards validate`; `tests/schema_verify.rs` on emitted JSON | Shape that must not exist (SUPPORTED without provenance, `substituted: true`, HELIOS keys, fixture labeled normative) |
| Runtime | `src/standards/validate.rs`, `src/guardrails.rs`, `src/claims.rs` `check_set` | Values schema cannot compare (selected == verified, VERIFIED predicates, citation in the pin) |
| Source scan | `tests/guardrails.rs`, `scripts/prove.sh` | Crates, `Mode::Ferrum`, spec fetches, HELIOS imports, wiring of `check_run` |
| Documented only | This file | Implicit trust that cannot be a boolean without lying |

If a rule can be a schema or a test, it is. If it cannot, it is listed in §4 so it is not mistaken for an unenforced hope.

---

## 2. Rule table

| # | Invariant | Schema | Runtime | Test |
|---|-----------|--------|---------|------|
| 1 | A registry row cannot become **SUPPORTED** without required provenance (`test_bindings`, `fixture_catalog`, `vendor_path` on every normative source) | `helix-standard-version-v1.json` `if support_status=supported` | `validate.rs` `check_version` | `tests/standards_registry.rs` `supported_requires_*` |
| 2 | A **development** release cannot become SUPPORTED | `if development then support_status=available`; supported ⇒ `release_class` not `development` | `DEVELOPMENT cannot be supported` | `development_cannot_be_supported` |
| 3 | A mutable `release_ref` (`HEAD` / `main` / `master` / `develop`) cannot enter an official / ballot / snapshot pack | `if official\|ballot\|snapshot then release_ref not those names` | `is_forbidden_release_ref` | `official_release_ref_cannot_be_head`, `official_release_ref_cannot_be_main` |
| 4 | A **normative** check cannot lack provenance (commit, source file, version, registry entry, GA4GH authority) | catalog + traceability `if check_kind=normative` (citation required on registry bindings) | `validate_result`; registry citation must be in `normative_sources` | `normative_without_provenance_is_rejected`; `normative_binding_requires_citation`; `normative_binding_source_file_must_be_in_the_pin` |
| 5 | A **fixture** check cannot be serialized as normative / `ga4gh_requirement` | verification schema `allOf` on `traceability` | `validate_result` | `fixture_serialized_as_normative_is_rejected`; `tests/schema_verify.rs` fixture-labeled-normative |
| 6 | `verified_version` cannot be set without `selected_version`, cannot disagree with it, cannot be set without join hashes, and cannot be set unless `ga4gh_requirement` is VERIFIED. Detected/declared/implementation metadata never stamp it. `selected_version` without `verified_version` is allowed. | JSON Schema cannot compare two field values | `guardrails::check_run`; `claim_integrity::validate_claim_integrity` | `selected_ne_verified_is_rejected`; `selected_without_verified_is_allowed`; `verified_without_join_hashes_is_rejected`; `tests/b8_claim_integrity.rs` B8-T14–T18 |
| 7 | A **VERIFIED** claim cannot exist without the claim predicates (and interoperability / benchmark never VERIFIED) | claims object shape only | `claims::check_set`; emit path calls it | `check_set_rejects_verified_without_predicates`; `tests/claims.rs` honest DRS PASS is `not_verified` |
| 8 | A run cannot silently substitute versions (`substituted` is not a free boolean) | `standard_selection.substituted` `const: false` | `check_selection` rejects `true` | `substituted_true_is_rejected`; `substituted_true_is_schema_invalid`; selector tests in `tests/verify_versions.rs` |
| 9 | A standard source cannot silently be fetched from HEAD / `main` / `master` / `develop` | pin `commit` is 40 hex chars; `release_ref` lock above | `is_mutable_source_url`; `source_url` must contain `commit`; **Helix does not fetch spec URLs** | `source_url_cannot_be_branch_head`; `src_must_not_fetch_standard_sources_from_the_network` |
| 10 | Ferrum-specific dependencies must not enter the generic verifier | — | — | `Cargo.toml` has no `ferrum` crate; `src` has no `Mode::Ferrum`; adapter stays `Mode::Generic`; `framework::` imports only under `src/adapter` |
| 11 | HELIOS functionality must not enter Helix verification semantics | verification + registry `additionalProperties: false`; HELIOS keys are not properties | `forbid_helios_keys` on **raw** JSON before serde (unknown keys would otherwise be dropped) | `helios_key_on_run_json_is_rejected`; `load_rejects_helios_ro_crate`; `helios_field_on_registry_record_is_rejected`; `src_must_not_import_helios` |
| 12 | Operator `--target-kind` cannot turn a mock or reference implementation into independent evidence | reviewed `targets/independence.yaml` | `run_counts_as_independent` | `tests/b9_independent_differential.rs` B9-T1–T3 |
| 13 | A differential artifact cannot create verification or ranking semantics | `helix-differential-v1.json` `creates_verification: false`, `ranking_semantics: absent` | `differential_json` / `contains_ranking_semantics` | B9-T17, B9-T21 |
| 14 | A target mutation must not change verifier identity or preserve a stale VERIFIED claim | — | `claim_integrity::validate_claim_integrity`; `MutationEvidence` does not stamp `verified_version` | `tests/b10_negative_control.rs` B10-T8–T17, B10-T20 |
| 15 | A verification claim cannot exceed the compiled coverage contract. `UNEVALUATED` ≠ `OUT_OF_SCOPE`. Target metadata cannot expand or shrink required rows. `required_complete` is derived. | `coverage` optional on v1; `ranking_semantics` const `absent`; `creates_verification` const false | `coverage::CoverageReport::from_run`; `validate_claim_integrity` recomputes coverage | `tests/b11_coverage_boundary.rs` B11-T1–T32 |
| 16 | Live independent observation cannot be a mock or default-catalog run with forged implementation metadata. Live JSON shares `coverage_id` / `execution_id` and differs in `target_execution_id`. | — | `live_independent_observation`; `validate_claim_integrity` on live artifacts | `tests/b12_live_reconciliation.rs`; [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md) |
| 17 | Emitted `helix_git_sha` must match the compile-time git HEAD. Dirty checkout cannot claim clean commit evidence. Missing `.git` does not fabricate a SHA. Provenance is not `execution_id`. | optional `helix_git_sha` / `helix_git_dirty` | `provenance::cites_this_verifier_build`; `validate_claim_integrity` (SHA/dirty vs this build) | `tests/b12_1_verifier_provenance.rs` |
| 18 | `AUTHZ_ENABLED`, implementation name, or an Authorization header is not authorization evidence. The coverage row `drs.security.authorization` stays unevaluated until a live protected-resource matrix exists. Forged authorization PASS fails integrity. | — | `authorization::configuration_is_not_evidence`; `validate_claim_integrity` | `tests/b13_authorization_boundary.rs`; [AUTHORIZATION.md](AUTHORIZATION.md) |
| 19 | Persisted JSON cannot strengthen verification. Claims/coverage are recomputed. Missing or mismatched `helix_git_sha` is historical, not current. Forged join/status/identity is invalid. Formatting is not identity. | `helix-verification-v1` unchanged | `evidence::classify_evidence`; `validate_artifact_consistency` | `tests/b14_evidence_durability.rs`; [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md) |
| 20 | Operator text must not equate check PASS or process exit 0 with `ga4gh_requirement` VERIFIED. Standing is computed (`helix inspect`), not stored. Attribution is visible. | v1 unchanged | `format_verify_text`; `format_inspect_text` | `tests/b15_operator_ux.rs`; [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md) |

Emit path (`helix verify` JSON and text, `bind_run`): `CheckMode::Emit` — every executed/skipped row must have valid traceability.

Load path (`helix compare`, `helix matrix`, `helix differential`): `CheckMode::Load` — HELIOS keys, substitution, and version mismatch still fail; **missing** traceability on old example JSON is allowed so compare does not rewrite history. If `claim_join` or `coverage` is present, `validate_artifact_consistency` runs. `helix_git_sha` need not match this binary on Load (historical inspectability). Current-vs-historical standing is computed (`evidence::classify_evidence`), not stored.

---

## 3. Wiring (do not bypass)

```text
helix verify
    → traceability::bind_run → guardrails::check_run (Emit)
    → report::verify_json / print_text → check_run again
    → claims injected → check_serialized_claims

helix compare / helix matrix
    → compare::parse_verification_run
    → forbid_helios_keys(raw JSON)
    → deserialize
    → check_run_with(Load)

helix standards validate
    → standards::validate_yaml (schema + extra_checks)
```

`tests/guardrails.rs` greps this wiring. Removing `check_run` from `report.rs` or `bind_run` is a test failure, not a silent hole.

---

## 4. Implicit trust assumptions (audit)

These are real. Do not paper over them with a fallback or a SUPPORTED tag.

### Encoded as fail-closed (already in the table)

- Default `helix verify TARGET` is **unversioned**. A pack is not inferred from `/v1` or from an AVAILABLE row.
- AVAILABLE is not executable. Mode 1 for a pinned-but-unsupported version is `AVAILABLE_BUT_NOT_SUPPORTED`.
- Discovery DETECTED is not a pass.
- Skip is not pass.
- Catalog rows are not `normative` / `guidance` until vendor bytes are loaded (`prove.sh` greps the catalog).

### Recorded gaps (must stay visible)

| Assumption | What is actually trusted | What must not be claimed |
|------------|--------------------------|---------------------------|
| Executed OpenAPI bytes | HelixTest pin (`VERSIONS.lock`), **not** `standards/vendor` hashes | “This run tested the pinned registry bytes” |
| Generic engine | HelixTest `Mode::Generic` behind `src/adapter` | Ferrum as a Helix crate or auto-selected mode |
| Independent implementations | Reviewed records in `targets/independence.yaml` (Starter Kit, Bento). Not a cryptographic URL bind. CI does not run live targets. | Multi-implementation certification, ranking, or “validated against every DRS” |
| Load-mode old JSON | Files without `traceability` can still be compared | That those files were emit-checked |
| `debug_assert!(check_set)` | Debug builds only | Release emit — **also** calls `check_set` via `check_run` |
| serde unknown fields | Dropped on deserialize | HELIOS keys surviving compare — raw JSON is scanned first |
| Human report sentences besides `Claims:` | Honesty greps in `prove.sh` | A VERIFIED sentence that did not come from `claims[]` |
| Future SUPPORTED locators | Schema + hash + citation path | A GA4GH board; review is single-steward ([STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) §10.1) |
| HelixTest internals | Sibling git root (D1); Helix cannot schema-check HelixTest’s vendored spec without merging repos | That Helix owns HelixTest’s OpenAPI pin |

### What Helix does not fetch

`helix standards` records `source_url` as provenance. Runtime reads **local** `vendor_path` and compares SHA-256. There is no spec HTTP client in `src/`. Adding `raw.githubusercontent.com` or `ga4gh.github.io` under `src/` fails `tests/guardrails.rs`.

Target HTTP (discovery, verify, security, bench) is a different class: that is the untrusted implementation under test, not a standard source.

---

## 5. How to add a rule

1. Prefer JSON Schema if the forbidden document has a local shape (`const`, `required`, `additionalProperties`, `if`/`then`).
2. If two fields must be equal, or a predicate set must hold, add runtime in `guardrails.rs` / `validate.rs` / `claims::check_set` and a test that **fails** when the invariant is broken.
3. If the risk is a crate or import, add a source scan in `tests/guardrails.rs` and a `prove.sh` grep for the test name.
4. If it still cannot be a boolean, add a row to §4. Do not write “contributors must remember.”

Do not weaken a test to match a convenient implementation. Do not mark a pack SUPPORTED to make Mode 1 green. Do not import HELIOS types to “complete” a report.

---

## 6. Related documents

| Document | Role |
|----------|------|
| [TRUST.md](TRUST.md) | Principle and reviewer map |
| [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) | Pack lifecycle; who reviews normative mappings |
| [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md) | Selection modes; fail closed |
| [CLAIMS.md](CLAIMS.md) | VERIFIED predicates |
| [COVERAGE.md](COVERAGE.md) | Verification boundary |
| [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md) | Live independent targets vs the same coverage contract |
| [TRACEABILITY.md](TRACEABILITY.md) / [TAXONOMY.md](TAXONOMY.md) | check_kind vs fixture |
| [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) | Feature gate |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Layer map (this file is the lock on that map) |
