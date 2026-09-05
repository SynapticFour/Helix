# Helix multi-implementation contract

**Status:** Architecture and harness shipped. **External validation: pending.** Target identity: [TARGETS.md](TARGETS.md).

Helix is HelixTest becoming a standalone VERIFY CLI. The same generic `helix verify <url>` suite is the interoperability contract. Ferrum is a **reference** live target, not a dependency, and has no clinical pilot.

This is **not** GA4GH certification. HELIOS still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md). HTTP surface: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md). Claims: [CLAIMS.md](CLAIMS.md) — a green matrix is not a VERIFIED GA4GH-requirement claim.

---

## 1. What is true today

| Fact | Consequence |
|------|-------------|
| `helix verify` has **no** implementation-name branches | Ferrum, mocks, and any other origin take the same code path (`Mode::Generic`) |
| In-process DRS/WES mocks exist | They prove the harness. They are **not** a second implementation and **not independent evidence** |
| Ferrum may be started with `make test-live` | Opt-in. Not run by `make prove`. Not recorded here as a matrix cell |
| No independent implementation JSON is in this repo | Slots `ferrum` and `independent` stay **pending** until an operator supplies runs |

Do **not** quote this repository as having completed multi-implementation validation.

---

## 2. Target-neutral contract

Every implementation is pointed at with:

```text
helix verify <url>
```

Default profile **`generic`**. Do not use `--profile ferrum` for this matrix: that profile is operator policy (expect DRS+WES, enable scatter), not a second engine and not this contract.

The target must not need Ferrum, Synaptic Four config, HelixTest internals, or special headers. Details: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md).

Labels on `helix matrix --run` are **operator names**. Helix does not infer “this is Ferrum” from `service-info.name` or the URL.

---

## 3. Checks that must agree vs may differ

Catalog: `src/interop.rs` `CATALOG`. Same ids as [TEST_IDENTITY.md](TEST_IDENTITY.md).

### Must agree (once the contract fixture is mounted)

These are the same observable across implementations. A disagreement is **not** automatically an implementation bug — it is an unresolved discrepancy (Helix bug, implementation bug, or ambiguous spec).

| Check | Why |
|-------|-----|
| `drs.object.reachable` | GetObject 200 for the documented known id |
| `drs.object.checksum` | Advertised checksum matches downloaded bytes of that fixture |
| `drs.object.not_found` | Unknown id → **404** |
| `wes.service_info.reachable` | GET service-info when WES is present |
| `wes.run.failure_state` | Documented fail workflow → `EXECUTOR_ERROR` or `SYSTEM_ERROR` |
| `wes.run.missing_inputs` | Documented missing-input fixture |
| `wes.run.incompatible_type` | Documented type-mismatch fixture |
| `wes.run.invalid_workflow` | Documented invalid workflow URL |

### May differ (spec permits; not a spec failure)

| Check | Why |
|-------|-----|
| `drs.object.range` | HTTP Range on `access_url` is **not** required by DRS GetObject |
| `drs.object.schema` | Required DrsObject fields are standard; HelixTest also requires `name` and non-empty `access_methods` (**runner extra**) |
| `wes.service_info.schema` | Extra HelixTest equality on `supported_wes_versions` `1.0`\|`1.1` |
| `wes.run.lifecycle_success` | Echo URL is a Helix fixture; requiring a pre-terminal state before `COMPLETE` is runner extra |
| `wes.run.scatter_gather` | Not a WES-required workflow. `generic` skips it |

Other permitted differences (not separate check ids): `access_url` vs `access_id`, extra checksum types, extra ServiceInfo fields, prefixed vs split URL layout, optional DRS `name`.

Do **not** classify a may-differ disagreement as `contract_violation`.

---

## 4. Matrix

```text
helix matrix --format json
helix matrix --run mock=./fixture.json --kind mock=helix_fixture --format text
helix matrix \
  --run ferrum=./ferrum.json --kind ferrum=reference_target \
  --run other=./other.json --kind other=independent_implementation \
  --format json
```

Columns on every row:

| Column | Meaning |
|--------|---------|
| `standard` | `drs` / `wes` |
| `version` | `verified_version` or `selected_version` when recorded; else omitted (unversioned) |
| `check` | Stable Helix id |
| `implementation` | Operator `--run` id |
| `expected` | Catalog (`pass` or `pass_or_skip`) |
| `observed` | `pass` / `fail` / `skip` / `error` when a run was supplied |
| `result` | `pending` / `meets_contract` / `contract_violation` / `fixture_unsatisfied` / `runner_stricter` / `not_executed` |

`contract_violation` is **only** for `must_agree` + fail/error. Range FAIL is `meets_contract` (optional). Schema FAIL is `runner_stricter`.

Comparisons (`discrepancies[]`):

| Classification | When |
|----------------|------|
| `agree` | Two executed outcomes match |
| `implementation_specific` | `may_differ` and they disagree |
| `unresolved_discrepancy` | `must_agree` and they disagree. Hypotheses listed, not chosen: `helix_bug`, `implementation_bug`, `ambiguous_spec` |
| `pending` | Fewer than two recorded implementations |
| `not_comparable` | Skip vs executed |

`independent_evidence` is true only when the operator recorded **`independent_implementation` plus another `reference_target` or `independent_implementation`**. Two `helix_fixture` runs never set it. Helix does not invent a second implementation.

Schema: [helix-interop-matrix-v1.json](../schemas/helix-interop-matrix-v1.json).

Exit **1** only when `unresolved_discrepancy` count > 0. Pending-only exits **0**.

---

## 5. How to record real evidence (when you have it)

1. Run `helix verify URL --format json` against Ferrum (reference). Save JSON.
2. Run the **same** command against a distinct implementation. Save JSON.
3. `helix matrix --run ferrum=… --kind ferrum=reference_target --run other=… --kind other=independent_implementation`.
4. File discrepancies. Do not silently retune Helix to match one origin.

Until step 2 exists as a file you control, leave validation **pending**.

---

## 6. Tests that must stay red

| Failure | Where |
|---------|--------|
| Empty matrix claims independent evidence | `pending_matrix_has_no_independent_evidence` |
| Two mocks count as independent | `two_fixture_runs_are_not_independent_evidence` |
| Range FAIL vs PASS is a spec failure | `range_disagreement_is_implementation_specific` |
| 404 disagreement has no hypotheses | `not_found_disagreement_is_unresolved_with_hypotheses` |
| `verify.rs` branches on Ferrum Gateway / elixir / cromwell | `verify_source_has_no_implementation_name_branches` |

---

## Out of this document

- Manufacturing a second server in-tree and calling it independent
- `--profile ferrum` as the interop contract
- HELIOS envelopes
- Public “validated against N implementations” copy while `external_validation` is `pending`
