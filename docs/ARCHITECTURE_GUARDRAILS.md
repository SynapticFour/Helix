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
| 6 | `verified_version` cannot be set without `selected_version`, cannot disagree with it, and cannot be set without join hashes. `selected_version` without `verified_version` is allowed (B2). | JSON Schema cannot compare two field values | `guardrails::check_run` | `selected_ne_verified_is_rejected`; `selected_without_verified_is_allowed`; `verified_without_join_hashes_is_rejected` |
| 7 | A **VERIFIED** claim cannot exist without the claim predicates (and interoperability / benchmark never VERIFIED) | claims object shape only | `claims::check_set`; emit path calls it | `check_set_rejects_verified_without_predicates`; `tests/claims.rs` honest DRS PASS is `not_verified` |
| 8 | A run cannot silently substitute versions (`substituted` is not a free boolean) | `standard_selection.substituted` `const: false` | `check_selection` rejects `true` | `substituted_true_is_rejected`; `substituted_true_is_schema_invalid`; selector tests in `tests/verify_versions.rs` |
| 9 | A standard source cannot silently be fetched from HEAD / `main` / `master` / `develop` | pin `commit` is 40 hex chars; `release_ref` lock above | `is_mutable_source_url`; `source_url` must contain `commit`; **Helix does not fetch spec URLs** | `source_url_cannot_be_branch_head`; `src_must_not_fetch_standard_sources_from_the_network` |
| 10 | Ferrum-specific dependencies must not enter the generic verifier | — | — | `Cargo.toml` has no `ferrum` crate; `src` has no `Mode::Ferrum`; adapter stays `Mode::Generic`; `framework::` imports only under `src/adapter` |
| 11 | HELIOS functionality must not enter Helix verification semantics | verification + registry `additionalProperties: false`; HELIOS keys are not properties | `forbid_helios_keys` on **raw** JSON before serde (unknown keys would otherwise be dropped) | `helios_key_on_run_json_is_rejected`; `load_rejects_helios_ro_crate`; `helios_field_on_registry_record_is_rejected`; `src_must_not_import_helios` |

Emit path (`helix verify` JSON and text, `bind_run`): `CheckMode::Emit` — every executed/skipped row must have valid traceability.

Load path (`helix compare`, `helix matrix`): `CheckMode::Load` — HELIOS keys, substitution, and version mismatch still fail; **missing** traceability on old example JSON is allowed so compare does not rewrite history.

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
| Independent implementations | None recorded; `helix matrix` slots pending | Multi-implementation validation |
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
| [TRACEABILITY.md](TRACEABILITY.md) / [TAXONOMY.md](TAXONOMY.md) | check_kind vs fixture |
| [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) | Feature gate |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Layer map (this file is the lock on that map) |
