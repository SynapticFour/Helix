# Helix targets (B4)

Helix is HelixTest becoming a standalone VERIFY CLI. This document describes **target identity** for GA4GH DRS 1.4.0 technical verification. It is not GA4GH certification. HELIOS still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md).

**B4 architecture: present. B4 multi-implementation evidence: pending a real second implementation.**

---

## 1. What a target is

A target is the HTTP origin Helix was pointed at. It is not:

- a GA4GH standard
- a standard version
- a release commit
- a specification pack
- a checker
- a binding
- a catalog

The DRS 1.4.0 Supported Pack (B3) is the specification authority. The target is the implementation under test.

---

## 2. Target identity

Recorded on `helix verify` JSON as `target.identity` (`src/target.rs`).

| Field | Meaning | Trusted as proof? |
|-------|---------|-------------------|
| `target_id` | Operator `--target-id`, else `endpoint:<url>` | Identifies the run, not the product |
| `target_kind` | Operator `--target-kind` (default `unspecified`) | No — operator label |
| `implementation_name` | `--implementation-name` | No — untrusted metadata |
| `implementation_version` | `--implementation-version` | **No.** Never inferred from headers, Docker names, package names, or URLs |
| `endpoint` | Normalized operator URL | Where Helix was pointed |
| `declared` | Operator labels, including a declared standard version | Untrusted |
| `detected` | Copied from 2xx service-info `type.version` when present | Untrusted. Not `verified_version` |
| `verified` | `target_id` + `endpoint` + `kind` actually used | Identity of the run, **not** a GA4GH version claim |

Helix never infers implementation version from HTTP server headers, package names, Docker image names, or endpoint URLs.

---

## 3. Target execution vs standard identity

Two identities must stay distinct:

| Identity | What it binds | Same across two targets using DRS 1.4.0? |
|----------|---------------|------------------------------------------|
| `standard_selection.execution_id` | Pack + schema hashes + checker (B2/B3 spec-join) | **Yes** |
| `standard_selection.target_execution_id` | Spec-join **plus** `target_id` / kind / endpoint | **No** — must differ |

`selected_version` is the pack Helix selected. `verified_version` remains a claims-engine field. Declaring `DRS 1.4.0` on the target does not set `verified_version`.

Helix does **not** cache verification results. If a cache is added, the key must be `verification_cache_key` (same ingredients as `target_execution_id`).

---

## 4. Why Ferrum is not a dependency

`src/adapter/mod.rs` and `src/target.rs` take a public HTTP base URL (`HttpDrsTarget`: `identity()` + `base_url()`). There is no `use ferrum`. Ferrum may be one live origin (`make test-live`, kind `reference_implementation`). It is not the architecture. CI / `make prove` use in-process mocks.

---

## 5. What qualifies as an independent implementation

| `target_kind` | Independent implementation evidence? |
|---------------|--------------------------------------|
| `real_external_implementation` | Yes (operator-declared) |
| `real_independent_local_implementation` | Yes (operator-declared) |
| `reference_implementation` | No (e.g. Ferrum). Real, not a second independent impl by itself |
| `mock` / `fixture` / `synthetic_target` | **No** |
| `unspecified` | **No** (fail closed) |

A mock is not an independent implementation. A fixture is not an independent implementation. A hand-written test server (`synthetic_target`) is not an implementation. Helix does not invent a second server and call it independent.

---

## 6. Live tests vs deterministic tests

| Suite | Network | Target |
|-------|---------|--------|
| `cargo test --locked --offline` / `make prove` | Localhost wiremock only | `mock` / `synthetic_target` |
| `make test-live` | Opt-in live stack | Operator labels the kind; typically `reference_implementation` |

A missing external target must skip / not-available, never a fabricated PASS. Live implementations are not a mandatory CI dependency.

---

## 7. Failure attribution

Per-check `attribution` maps existing PASS/FAIL/SKIP/ERROR + diagnostics. Not a score.

| Value | Meaning |
|-------|---------|
| `spec_failure` | Normative check FAIL (target response vs pinned schema) |
| `target_failure` | Fixture/behavior FAIL |
| `transport_failure` | Reachability FAIL |
| `target_configuration_failure` | Expected service not TESTABLE / not detected, **or** DRS fixture unavailable (404 on the configured object / no `access_url` for checksum-range) |
| `helix_execution_failure` | ERROR: adapter, wall clock, SpecSource identity mismatch |
| `unsupported_test` | SKIP |
| `unknown` | FAIL without a catalogued diagnostic |

A broken checker must not be recorded as target non-conformance. Skip is never pass.

---

## 8. Why mocks are not implementation evidence

`compare_target_runs` sets `independent_implementation_evidence` only when both kinds are usable (not mock/fixture/synthetic/unspecified) **and** at least one is `real_*`. Two mocks with identical packs still produce separate `target_execution_id` values. That proves the harness, not multi-implementation validation.

The interop matrix (`helix matrix`) remains the operator-labeled comparison harness. [INTEROP.md](INTEROP.md).

---

## 9. Same pack, checker, catalog

For every B4 target, DRS 1.4.0 must keep:

```text
pack_id = ga4gh.drs.1.4.0
release_commit = 36145d389e0a454428d1dac5c4a30870995fdd7c
pack_integrity_sha256 = c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89
schema_document_sha256 = 3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608
```

plus the same `checker_id` (executed source digest, [CHECKER_PROVENANCE.md](CHECKER_PROVENANCE.md)), `binding_id`, and `catalog_id` from the B3 support contract. Helix does not recompile a per-target checker.

---

## 10. What B4 does not claim

- GA4GH certification or official status
- `verified_version` from target metadata
- Ranking, scoreboard, compliance percentage
- That Ferrum (or any mock) is a second independent implementation
- That two PASS results mean two independent implementations were validated
- HELIOS evidence packs

Provenance stays standard-centric: GA4GH → DRS → 1.4.0 → release commit → pack → schema → HelixTest checker → binding → catalog → **target execution**.

The specification still comes only from the pinned local vendor pack. A live target endpoint is allowed. A live GA4GH specification download is not.

---

## 11. DRS test fixture (B6)

B5 showed that a real independent DRS can be DETECTED and still 404 on Helix’s default catalog id `test-object-1`. That id is **not** a DRS 1.4.0 requirement. It was harness test input mixed into standard-level checks.

| Identity | What it is | Must not become |
|----------|------------|-----------------|
| `execution_id` | Pack + schema hashes + checker (spec-join) | Pack + target fixture |
| `target_execution_id` | Spec-join + target identity **+ DRS fixture** (`object_id`, optional expected sha256) | A GA4GH version claim |
| Fixture `object_id` | Which target-owned object Helix GETs | A MUST in DrsObject |

Operator configuration (matches existing CLI style):

```text
helix verify <url> --standard drs --version 1.4.0 --drs-object-id <id>
helix verify <url> --standard drs --version 1.4.0 --drs-object-id <id> --drs-object-sha256 <64 hex>
```

JSON records `drs_fixture` (`object_id`, `unknown_object_id`, `source` = `default_catalog` | `operator_declared`, optional `expected_sha256`, `checksum_mode`). `checksum_mode` is `operator_digest` when `--drs-object-sha256` is set (downloaded bytes vs operator digest; advertised GetObject sha256 cannot manufacture a PASS) or `advertised_consistency` otherwise (advertised vs download; not an independent blob-integrity oracle). Source `operator_declared` is explicit test configuration, not automatic object discovery. Helix does not scan IDs, enumerate the store, or branch on implementation name.

A 404 on the **configured** existing-object id is `fixture_unavailable` (SKIP, attribution `target_configuration_failure`), not `spec_failure`. That is distinct from a 404 on the derived unknown id (`helix.unknown.<sha256 prefix>`), which remains the HLX-DRS-005 behavior check.

A fixture does **not** establish DRS 1.4.0 compliance. Mocks remain mocks. Ferrum remains an optional `reference_implementation` using the default catalog id unless the operator overrides it.

B4 multi-implementation evidence remains pending a real second implementation **with a valid fixture contract**. B5’s starter-kit run against `test-object-1` stays historically a missing-fixture result, not a rewritten PASS.
