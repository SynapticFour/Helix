# Helix claim engine

**Status:** Implemented. Source: `src/claims.rs` (`evaluate`, `check_set`, `format_claims_section`). JSON: `claims[]` on `helix verify`. Text: `Claims:` in [REPORT.md](REPORT.md). Unjustified VERIFIED rows fail `check_set` on emit ([ARCHITECTURE_GUARDRAILS.md](ARCHITECTURE_GUARDRAILS.md)).

Helix is HelixTest becoming a standalone VERIFY CLI. This engine productizes what a run is allowed to **say**. It does not invent a new suite.

This is **not** GA4GH certification. HELIOS (`helios-audit`) still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md). Taxonomy: [TAXONOMY.md](TAXONOMY.md).

---

## 1. Why this exists

A green DRS fixture (`executed[].status = pass`) is easy to quote as “verified against GA4GH DRS.” That sentence is not justified for default `helix verify`: it is unversioned; DRS 1.4.0 is SUPPORTED for technical verification within declared coverage; `verified_version` is stamped only when `ga4gh_requirement` predicates hold. Exactly one shipped check is `normative`. A successful versioned execution join still does not emit VERIFIED unless the support catalog also PASSes.

Human-readable verification statements are generated **only** from `claims[]`. The report does not search check messages for PASS/FAIL to decide VERIFIED.

---

## 2. Six claims, never one sentence

`evaluate` always emits six rows, in this order:

| `kind` | What VERIFIED would mean | Today |
|--------|--------------------------|--------|
| `ga4gh_requirement` | Verified against the selected GA4GH product + release (support catalog complete) | Default unversioned: **NOT_VERIFIED**. Versioned DRS 1.4.0 with a complete catalog PASS may be **VERIFIED** (technical verification, not certification). |
| `schema` | Schema-layer normative verification | Default unversioned: **NOT_VERIFIED** |
| `behavior` | Behavioural-layer normative verification | **NOT_VERIFIED** |
| `interoperability` | Interoperability testing as a requirement | **NOT_VERIFIED** (`interoperability_is_not_a_ga4gh_requirement`) |
| `security` | Security-layer normative verification | **NOT_VERIFIED** (security PASS is not this claim) |
| `benchmark` | Benchmark as verification | **NOT_VERIFIED** (`benchmark_is_measurement_only`) |

Do not collapse them. SCHEMA PASS is not a schema VERIFIED claim. `layer_summary` is counts, not this model.

`status` is `verified` or `not_verified` only. There is no `passed` boolean.

---

## 3. Predicates for VERIFIED

For `schema`, `behavior`, and `security`, **all** of the following must hold. If any is missing, the claim is **not** generated as VERIFIED.

`ga4gh_requirement` additionally requires `catalog_evidence_complete`: every check in the selected pack’s support catalog was executed, none of those rows are FAIL/ERROR, and no catalog row is SKIP because required evidence was unavailable (`fixture_unavailable`). Fixture FAIL is `catalog_check_failed`, not `normative_check_failed`. Fixture SKIP `fixture_unavailable` is `required_evidence_unavailable`. Those blocks prevent a DRS 1.4.0 `ga4gh_requirement` VERIFIED even when HLX-DRS-006 PASSes.

| Predicate (`satisfied[]`) | Recorded evidence |
|---------------------------|-------------------|
| `exact_standard_identified` | `standard_selection.standard` set |
| `supported_release_selected` | `selection_status = SELECTED` (Helix only selects OfficialSupported) |
| `pinned_specification_source` | `standards_registry_entry` and `standards_source_commit` set |
| `integrity_validation_successful` | `integrity_validated` and `integrity_ok = true` on this run |
| `selected_equals_tested` | `ga4gh_requirement`: `selected_version` equals `verified_version` (both set). Schema/behavior/security: selected version is identified (`verified_version` is a gate output, not an input). Selected ≠ verified still blocks. |
| `required_normative_checks_executed` | ≥1 executed check with taxonomy `normative` / `ga4gh_requirement` / `authority=ga4gh` (layer-filtered for schema/behavior/security) |
| `required_normative_checks_passed` | those executed rows are `status=pass` |
| `no_blocking_normative_failures` | none of those rows are fail or error |
| `coverage_requirements_satisfied` | no skipped normative row in that set |
| `evidence_recorded` | at least one check id on the run |
| `no_substitution` | `substituted = false` |
| `catalog_evidence_complete` | **`ga4gh_requirement` only.** Support-catalog rows all present and PASS |

`verified_version` is stamped only when `ga4gh_requirement` is VERIFIED (`src/claim_integrity.rs` `stamp_verified_version_if_justified`). Detected, declared, and `--implementation-version` never stamp it. The join is `claim_join` (IDs + check statuses), not a HELIOS pack.

An **empty** normative set is **not** vacuously verified. It blocks with `no_normative_checks`.

Layer claims use the same provenance predicates plus **that layer’s** normative rows only. A schema normative PASS does not verify `behavior`.

`interoperability` and `benchmark` cannot be VERIFIED. They still appear so the report can say why not.

---

## 4. Why NOT VERIFIED

Rejected claims list `blocks[].code` plus `evidence` (`field`, optional `check_id` / `observed` / `expected`).

Examples:

| Situation | Block code |
|-----------|------------|
| Default `helix verify TARGET` | `unversioned_run`, `no_version_selected` |
| Registry row exists, no SUPPORTED pack | `available_but_not_supported` |
| `selected_version` ≠ `verified_version` | `selected_ne_verified` |
| Pack id or commit empty | `provenance_missing` |
| Verify did not hash vendor bytes | `integrity_validation_not_recorded` |
| Hash recorded and failed | `integrity_mismatch` |
| No `BindingKind::Normative` rows | `no_normative_checks` |
| A normative row skipped | `incomplete_normative_coverage` |
| A normative row `fail` | `normative_check_failed` (with `check_id`) |
| Fixture FAIL | **not** `normative_check_failed`; `catalog_check_failed` on `ga4gh_requirement` |
| Catalog SKIP `fixture_unavailable` | `required_evidence_unavailable` (prevents VERIFIED) |
| Stale `checker_id` / `binding_id` / `catalog_id` | `checker_identity_mismatch` / `binding_identity_mismatch` / `catalog_identity_mismatch` |

Classification uses structured fields (`VerificationStatus`, `BindingKind`, `ClaimScope`, `CheckLayer`, `selection_status`). It does not grep messages for PASS/FAIL.

---

## 5. What is true today

Default and honest-fixture **unversioned** `helix verify` emit **six `not_verified` claims**. Exactly one shipped check is `normative`. DRS 1.4.0 is SUPPORTED; that is not automatically VERIFIED. Default unversioned verify does not hash vendor bytes: `integrity_validated` is false. `helix standards validate` is a separate command.

`verified_version` is stamped only when `ga4gh_requirement` is derived as VERIFIED (`src/claim_integrity.rs`). A constructed SELECTED + integrity-ok + one schema-normative PASS row can emit **schema** VERIFIED; `ga4gh_requirement` still requires the full support catalog. An in-process mock that PASSes every DRS 1.4.0 catalog check may stamp `verified_version = 1.4.0`. That is technical verification of the support contract, not GA4GH certification. The Starter Kit remaining NOT VERIFIED (HLX-DRS-002 FAIL, HLX-DRS-003/004 SKIP `fixture_unavailable`) is standing negative evidence.

---

## 6. Human report

`format_claims_section` iterates `ClaimSet` only. It prints `VERIFIED` / `NOT_VERIFIED` from `ClaimStatus`, then “Why verified:” (`satisfied[]`) or “Why not verified:” (`blocks[]`). Target-controlled evidence strings are sanitized ([THREAT_MODEL.md](THREAT_MODEL.md)).

The frozen line “It is not GA4GH certification” stays. It is a product constraint, not a result claim.

The interop matrix ([INTEROP.md](INTEROP.md)) is not a VERIFIED claim and not multi-implementation validation until independent run JSON exists.

---

## 7. Tests that must stay red

| Failure | Where |
|---------|--------|
| No version → VERIFIED | `src/claims.rs` `no_version_yields_no_verification_claim` |
| AVAILABLE but not SUPPORTED → VERIFIED | `available_but_unsupported_yields_no_verification_claim` |
| selected ≠ tested → VERIFIED | `selected_ne_tested_is_not_verified` |
| Fixture FAIL → `normative_check_failed` | `fixture_failure_is_not_a_normative_failure_claim`, `tests/claims.rs` |
| Incomplete coverage → full VERIFIED | `incomplete_normative_coverage_blocks_full_verification` |
| Provenance missing → VERIFIED | `provenance_missing_blocks_verification` |
| Integrity mismatch / not recorded → VERIFIED | `integrity_*_blocks_verification` |
| Honest unversioned DRS PASS → `ga4gh_requirement` VERIFIED | `tests/claims.rs` `honest_drs_pass_is_not_verified` |
| Catalog SKIP/FAIL → DRS 1.4.0 VERIFIED | `tests/b8_claim_integrity.rs` B8-T2 / T4 |
| Forged `verified_version` | `tests/b8_claim_integrity.rs` B8-T18 |

Do not weaken those tests. Do not reclassify fixture checks as `normative` to raise verification rates.

---

## Out of this engine

- HELIOS signatures, RO-Crate, PDF, audit trails
- An aggregated “100% verified” score
- Treating DETECTED, SKIP, or SCHEMA PASS as VERIFIED
- Asking anyone to trust Helix authors
