# Helix DRS 1.4.0 verification coverage

Helix is HelixTest becoming a standalone VERIFY CLI. This document describes the **machine-enforced verification boundary** for versioned DRS 1.4.0 (`src/coverage.rs`). It is not a test-count goal, not a coverage percentage, not a score, not a ranking, not GA4GH certification, and not HELIOS.

HelixTest already runs the DRS checks. Helix productizes what those checks are allowed to **mean**.

**Vertraue mir nicht, vertraue dem Code.**

Trust: [TRUST.md](TRUST.md). Claims: [CLAIMS.md](CLAIMS.md). Negative controls: [NEGATIVE_CONTROL.md](NEGATIVE_CONTROL.md). Schema: [SCHEMA.md](SCHEMA.md).

---

## 1. What B11 proves

Helix can enumerate, for a selected DRS 1.4.0 pack:

- which behaviors are inside the current verification contract and were evaluated
- which evaluated behaviors are **not** GA4GH normative requirements
- which relevant DRS operations remain **unevaluated**
- which behaviors the contract **explicitly excludes**

A `ga4gh_requirement` VERIFIED sentence is technical verification **under this contract**. It is not “full DRS 1.4.0 compliance.”

```text
UNEVALUATED ≠ OUT_OF_SCOPE ≠ PASS ≠ VERIFIED
```

`verified_version` remains derived (`src/claim_integrity.rs`). `coverage.required_complete` is derived. A JSON field `required_complete: true` cannot stamp verification.

---

## 2. Coverage classes

Every compiled row is exactly one of:

| Class | Meaning |
|-------|---------|
| `normative` | Inside the support contract; SpecSource-backed; can contribute to `ga4gh_requirement` / `schema` when PASS |
| `evaluated_non_normative` | Exercised HelixTest fixture/capability probe. Required for catalog completeness. Cannot independently create a GA4GH VERIFIED claim |
| `unevaluated` | Relevant to the pinned DRS 1.4.0 boundary; Helix has no executable catalog binding on this run |
| `out_of_scope` | The Helix verification contract **explicitly excludes** it |

A missing test is `unevaluated`, not `out_of_scope`. YAML `scope: out_of_scope` is not a Helix input. Target metadata cannot reclassify a required row.

---

## 3. Coverage identity

`coverage_id` is SHA-256 of a canonical string (`helix-coverage-v1`) bound to:

- pack id, selected version, release commit
- `catalog_id`, `checker_id`, `binding_id`, pack integrity
- compiled row identities and classifications

It does **not** include timestamp, hostname, target URL, filesystem path, PID, UUID, dirty tree, or check **results**.

The same DRS 1.4.0 contract therefore has the same `coverage_id` for Starter Kit-shaped and Bento-shaped targets. `execution_id` is also shared. `target_execution_id` differs.

Changing the coverage contract, checker, release, binding, or catalog changes `coverage_id`. Forged `coverage_id` fails `validate_claim_integrity`.

`coverage_id` is **not** hashed into `catalog_id`. Catalog identity is unchanged.

---

## 4. Coverage state

| State | Predicate |
|-------|-----------|
| `inapplicable` | No selected supported pack (unversioned / selection failed) |
| `blocked` | A required Helix-contract row is missing, SKIP, FAIL, or ERROR |
| `partial` | Required Helix-contract rows all PASS, and unevaluated DRS operations remain |
| `complete` | Required rows PASS and the compiled contract lists no unevaluated remainder |

Honest versioned DRS 1.4.0 with a passing catalog is **`partial`**: the catalog can complete while pinned OpenAPI operations remain unevaluated. That is intended. Do not widen the catalog to empty `unevaluated` merely to print `complete`.

`required_complete` means every **required** catalog row PASSed. It does not mean the pinned standard was exhaustively tested.

---

## 5. DRS 1.4.0 matrix (compiled contract)

Catalog rows come from `DRS_140_CONTRACT.checks` (`src/standards/support.rs`). Boundary rows are compiled in `src/coverage.rs`.

| Id | Code | Class | Required for `ga4gh_requirement` | Binding |
|----|------|-------|----------------------------------|---------|
| `drs.object.schema.openapi` | HLX-DRS-006 | normative | yes | HelixTest SpecSource DrsObject |
| `drs.object.schema` | HLX-DRS-002 | evaluated_non_normative | yes (catalog) | HelixTest extras schema |
| `drs.object.reachable` | HLX-DRS-001 | evaluated_non_normative | yes (catalog) | Level-0 reachable |
| `drs.object.checksum` | HLX-DRS-003 | evaluated_non_normative | yes (catalog) | advertised `sha256` / operator digest |
| `drs.object.range` | HLX-DRS-004 | evaluated_non_normative | yes (catalog) | HTTP Range on `access_url` |
| `drs.object.not_found` | HLX-DRS-005 | evaluated_non_normative | yes (catalog) | unknown object 404 |
| `drs.op.service_info` | — | unevaluated | no | none |
| `drs.op.objects_bulk` | — | unevaluated | no | none |
| `drs.op.access` | — | unevaluated | no | none |
| `drs.op.access_bulk` | — | unevaluated | no | none |
| `drs.op.object_options` | — | unevaluated | no | none |
| `drs.op.object_post_passport` | — | unevaluated | no | none |
| `drs.security.authorization` | — | unevaluated | no | none |
| `drs.checksum.types.other` | — | unevaluated | no | none |
| `drs.object.bundle_contents` | — | unevaluated | no | none |
| `http.tls.as_drs_requirement` | — | unevaluated | no | none |
| `http.client.timeouts` | — | unevaluated | no | none |
| `helix.helios` | — | out_of_scope | no | excluded |
| `helix.bench.as_verification` | — | out_of_scope | no | excluded |
| `target.non_drs_http` | — | out_of_scope | no | excluded (e.g. `/internal/not-in-drs-catalog`) |
| `http.discovery.redirect_follow` | — | out_of_scope | no | Helix discovery `Policy::none` |

There is no `/search` path in pinned DRS 1.4.0 OpenAPI. Do not invent it.

HLX-DRS-006 PASS does not imply `access_methods` present (that field is not a normative MUST on the pinned schema path). HLX-DRS-003 PASS does not verify every Checksum.yaml type. HLX-DRS-004 PASS does not verify every HTTP transport behavior.

---

## 6. Claim derivation

```text
execute catalog checks
→ bind evidence / attribution
→ evaluate() predicates (including catalog_evidence_complete)
→ CoverageReport::from_run
→ stamp verified_version only if ga4gh_requirement is VERIFIED
→ serialize; JSON coverage/claims/claim_join are overwritten from evaluate/from_run
```

`ga4gh_requirement` VERIFIED requires the existing B8 predicates **and** catalog completeness (001–006 present and PASS; fixture SKIP `fixture_unavailable` blocks). Schema VERIFIED still uses only the normative schema-layer row (006). Coverage evaluation does not replace that engine.

A target with `AUTHZ_ENABLED=false` cannot emit an authorization-verified claim. Authorization is `unevaluated`. `security` stays NOT_VERIFIED. B13: [AUTHORIZATION.md](AUTHORIZATION.md).

---

## 7. Fixture-dependent evidence

Missing `--drs-object-id` / unreachable configured object remains `fixture_unavailable` / `target_configuration_failure`, not `spec_failure`. That SKIP is preserved on the catalog row and listed in `required_missing` when the row is required.

Starter Kit without `access_url` may SKIP checksum/range. Bento with `access_url` may execute them. Same coverage contract; different observations.

---

## 8. Standard-derived vs Helix-derived

The pinned DRS 1.4.0 OpenAPI defines operations Helix does not execute. The Helix verification contract is **narrower**. JSON `coverage.unevaluated` is that honesty. Do not treat `ga4gh_requirement` VERIFIED as identity with the full standard.

---

## 9. Ranking

Coverage JSON may list counts of evaluated / unevaluated / out-of-scope rows. It must not emit a percentage, score, rank, winner, or “better implementation.” `ranking_semantics` is `absent`. `creates_verification` is `false`.

---

## 10. HELIOS

B11 does not add RO-Crate, PDF, signatures, or archival provenance. Helix remains verification infrastructure. HELIOS remains reproducibility / signed evidence.

---

## 11. Tests

`tests/b11_coverage_boundary.rs` (B11-T1–T32). Forged `coverage_id`, forged `required_complete`, forged OUT_OF_SCOPE on a required row, target-metadata inflation, and stale 1.5.0 / checker / binding identities fail closed.

Authorization remains unevaluated until a genuine live matrix exists: [AUTHORIZATION.md](AUTHORIZATION.md), [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md) (**DEFER**), `tests/b13_authorization_boundary.rs`.

Live reconciliation of the same `coverage_id` against independent implementations: [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md), `tests/b12_live_reconciliation.rs`. Prove does not require those live JSON files.
