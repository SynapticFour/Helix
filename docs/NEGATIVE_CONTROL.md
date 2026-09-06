# Helix negative-control / mutation verification (B10)

Helix is HelixTest becoming a standalone VERIFY CLI. This document is the **versioned DRS 1.4.0** mutation gate. It is not a general mutation-testing framework, not a detection-percentage score, not GA4GH certification, and not HELIOS.

The unversioned known-bad corpus (`HLX-MUT-*`) remains [MUTATION.md](MUTATION.md). Do not mix those catalogs. B10 asks a different question:

> When a known in-scope DRS behavior is deliberately broken, does Helix detect it and revoke `verified_version` without changing the verifier identity?

And separately:

> Which deliberately changed behaviors remain outside the current verification closure?

Trust: [TRUST.md](TRUST.md). Claims: [CLAIMS.md](CLAIMS.md). Evidence types: `src/negative_control.rs`. Target fixtures: `tests/support/mock_b10.rs`. Tests: `tests/b10_negative_control.rs`.

**Vertraue mir nicht, vertraue dem Code.**

---

## 1. What this proves

The golden in-process DRS mock (4096 × `'A'`, object `test-object-1`) verifies under Mode 1 DRS 1.4.0 (`verified_version = 1.4.0`). Each in-contract mutation is applied **to the HTTP target**, then `helix verify` is run again through the same path as production:

```text
helix verify
→ VerifyOptions (DRS 1.4.0)
→ pack load / SpecSource
→ HelixTest checker
→ catalog checks
→ bind_run
→ claim_integrity::stamp_verified_version_if_justified
→ report evaluate()
```

HelixTest checker source, DRS pack, SpecSource, catalog, binding, and claim engine are **not** mutated to produce a failure.

A mutation that does not change the result is not automatically a Helix failure. First decide whether the mutated behavior is inside the tested closure.

---

## 2. Two schema checks (do not conflate)

Pinned `DrsObject.yaml` `required`: `id`, `self_uri`, `size`, `created_time`, `checksums`.

`access_methods` is **not** in that `required` list (prose requires it for blobs; the schema allows bundles without it).

| Check | Helix id | Kind | What it tests |
|-------|----------|------|----------------|
| HLX-DRS-006 | `drs.object.schema.openapi` | **normative** | Pinned DRS 1.4.0 SpecSource only |
| HLX-DRS-002 | `drs.object.schema` | **fixture** | SpecSource **plus** HelixTest extras (`id` match, `name`, `self_uri`, non-empty `access_methods`) |

M1 (omit `access_methods`) therefore FAILs 002 (`target_failure`) while 006 can remain PASS. That is recorded, not forced into a schema failure.

---

## 3. Checksum oracle (M3 vs M4)

HelixTest `checksum_against_bytes`:

- `--drs-object-sha256` present → compare **download vs operator digest** (`operator_digest`). Advertised checksum is ignored.
- otherwise → compare **advertised `type=sha256` vs download** (`advertised_consistency`). That is **not** independent evidence.

M3 uses advertised_consistency (lie in metadata, keep bytes).
M4 keeps advertised metadata and mutates bytes, with the independently supplied digest of the original 4096 × `'A'`.

Checksum `type` lookup is `sha256` (ASCII case-insensitive), not IANA `sha-256`. Bento's `sha-256` advertisement is therefore **not** the M3 oracle unless `--drs-object-sha256` is supplied.

---

## 4. Mutation matrix (from tests, not assumed)

| Mutation | Behavior | Expected check | Baseline | Mutated | Attribution | Claim |
|----------|----------|----------------|----------|---------|-------------|-------|
| M1 | omit `access_methods` | HLX-DRS-002 | PASS | FAIL | `target_failure` | NOT VERIFIED |
| M2 | `size` JSON string | HLX-DRS-006 | PASS | FAIL | `spec_failure` | NOT VERIFIED |
| M3 | advertised sha256 lie | HLX-DRS-003 | PASS | FAIL | `target_failure` | NOT VERIFIED |
| M4 | bytes lie; operator digest of original | HLX-DRS-003 | PASS | FAIL | `target_failure` | NOT VERIFIED |
| M5 | Range → HTTP 200 | HLX-DRS-004 | PASS | FAIL | `target_failure` | NOT VERIFIED |
| M6 | derived unknown id → 200 | HLX-DRS-005 | PASS | FAIL | `target_failure` | NOT VERIFIED |

M1 also SKIPs HLX-DRS-003/004 (`fixture_unavailable`, no `access_url`). Multiple check changes are causally expected. Do not force a single FAIL.

P1 key reorder, P2 whitespace, P3 schema-allowed `description`, P4 different `target_id` with identical behavior: no false `target_failure`.

---

## 5. Forensic coverage table

| Behavior | Covered by Helix? | Check ID | Normative/fixture | Mutation tested? | Expected | Observed | Attribution | Claim impact |
|----------|-------------------|----------|-------------------|------------------|----------|----------|-------------|--------------|
| Object GET 200 | yes | HLX-DRS-001 | fixture | not M1–M6 primary | PASS | PASS | — | none alone |
| Non-empty `access_methods` | yes | HLX-DRS-002 | fixture | M1 | FAIL | FAIL | `target_failure` | revokes VERIFIED |
| OpenAPI `DrsObject` types | yes | HLX-DRS-006 | normative | M2 (`size` string) | FAIL | FAIL | `spec_failure` | revokes VERIFIED |
| Advertised sha256 vs bytes | yes | HLX-DRS-003 | fixture | M3 | FAIL | FAIL | `target_failure` | revokes VERIFIED |
| Operator digest vs bytes | yes | HLX-DRS-003 | fixture | M4 | FAIL | FAIL | `target_failure` | revokes VERIFIED |
| `Range: bytes=0-1023` → 206 | yes | HLX-DRS-004 | fixture | M5 | FAIL | FAIL | `target_failure` | revokes VERIFIED |
| Derived unknown id → 404 | yes | HLX-DRS-005 | fixture | M6 | FAIL | FAIL | `target_failure` | revokes VERIFIED |
| JSON key order | yes (must ignore) | (all) | — | P1 | PASS | PASS | — | still VERIFIED |
| JSON whitespace | yes (must ignore) | (all) | — | P2 | PASS | PASS | — | still VERIFIED |
| Optional `description` | yes (allowed) | HLX-DRS-006 | normative permits | P3 | PASS | PASS | — | still VERIFIED |
| `target_id` label only | identity only | — | — | P4 | VERIFIED, different `target_execution_id` | same | — | still VERIFIED |
| `GET /internal/not-in-drs-catalog` | **no** | — | — | O1 | unchanged | unchanged | — | still VERIFIED; `mutation_outside_closure` |
| service-info `name` (gateway `/ga4gh/drs/v1`) | **no** (not a catalog check) | — | — | not used as O1 | — | — | — | mounting it changes discovery prefix; not a clean unused-path control |
| Optional `mime_type` vs Range check | **no** (unused by 004) | HLX-DRS-004 | fixture | H1 | no detection | no detection | — | still VERIFIED; harness not tautological |
| Missing configured object | skip, not fail | several | fixture | T23 | SKIP `fixture_unavailable` | SKIP | `target_configuration_failure` | not PASS, not FAIL |
| IANA checksum type `sha-256` vs `sha256` | advertised_consistency looks up `sha256` only | HLX-DRS-003 | fixture | not mutated here | — | — | — | coverage boundary; Bento used operator digest |
| AuthZ when disabled, `/search`, DB internals, performance | **no** | — | — | not in B10 CI | — | — | — | outside closure |
| Unversioned `HLX-MUT-*` misses | see [MUTATION.md](MUTATION.md) | various | unversioned | not B10 | — | — | — | different catalog |

This table is the claim about **what Helix tests**. It is not a completeness score.

---

## 6. Identity invariants

| Field | Under target mutation |
|-------|------------------------|
| `execution_id` | **unchanged** (pack + schema + checker) |
| pack / schema hashes | **unchanged** |
| `checker_id` / `binding_id` / `catalog_id` | **unchanged** |
| `target_execution_id` | **may change** (endpoint, `target_id`, fixture digest) |
| `implementation_name` / `implementation_version` / `target_kind` | **not** used to label the mutation |

There is no sticky verification cache. `verified_version` is derived again on each run. Mutation evidence copies `evaluate()`; it does not stamp claims. `helix verify` JSON still recomputes `claims[]` / `claim_join` through `evaluate`.

---

## 7. Security / reproducibility

Mutations are localhost wiremock. No reverse proxy, no URL forwarding, no Docker pull, no credential logging, no externally reachable mutation service. Reproducible from: Helix commit, HelixTest pin, DRS 1.4.0 pack hash, mutation id, fixture object id, operator digest when used.

Live Bento mutations are **not** required for CI. If a live mutated Bento run is not captured, mark live evidence unavailable rather than fabricating JSON.

---

## 8. What this does not prove

- Complete implementation verification
- Ranking, scores, or “N% of mutations detected”
- GA4GH certification
- HELIOS signed evidence / RO-Crate / PDF
- Security certification or fuzzing
- That an outside-closure PASS means the implementation is correct
