# Helix adversarial mutation corpus

**Status:** Implemented. Catalog: `src/mutation.rs`. Fixtures: `tests/support/mock_mutation.rs`. Tests: `tests/mutation.rs`.

Helix is HelixTest becoming a standalone VERIFY CLI. This corpus productizes **known-bad targets** so Helix is shown to detect incorrect implementations, not merely accept correct ones. It is **not** GA4GH certification, not a pentest suite, and not HELIOS.

Trust: [TRUST.md](TRUST.md). Layers: [BEHAVIOR.md](BEHAVIOR.md). Diagnostics: [DIAGNOSTICS.md](DIAGNOSTICS.md). Fixtures: [FIXTURES.md](FIXTURES.md).

A **SCHEMA PASS is not a BEHAVIOR PASS.** A mutation PASS (honest control) is not certification. There is no “mutations detected %” compliance score.

---

## 1. What this proves

| Claim | How |
|-------|-----|
| correct target → PASS | Honest DRS+WES mock: DRS five pass, WES seven pass, scatter skip |
| known-bad target → FAIL | Each **Detected** mutant is overall not PASS |
| known-bad target → **correct failure reason** | Named `expected_check_id` is `fail`, with the recorded diagnostic class and message substring |
| misses are not hidden | Each **Missed** mutant has a reason; CI fails if that check starts failing (catalog stale) |

Do **not** make Helix less strict to make a mutant fixture pass.

No new `kind=normative` check was added. Misses that would require inventing a MUST stay misses.

---

## 2. Mutation-testing summary

Counts come from `helix::mutation::summary()` (catalog length, not a score):

| | Count |
|--|------:|
| Mutations attempted | **24** |
| Mutations detected | **17** |
| Mutations missed | **7** |

If these numbers change, update this table to match `CATALOG`.

---

## 3. Catalog (one defect each)

`HLX-MUT-*` is a Helix corpus id, not a GA4GH requirement id. Suite is `verify` unless noted `security`.

### Detected

| ID | Defect | Expected Helix result | Check | Diagnostic |
|----|--------|------------------------|-------|------------|
| HLX-MUT-001 | missing required `self_uri` | FAIL | `drs.object.schema` | schema |
| HLX-MUT-002 | `size` is a JSON string | FAIL | `drs.object.schema` | schema |
| HLX-MUT-003 | existing object HTTP 403 | FAIL | `drs.object.reachable` | reachability |
| HLX-MUT-005 | `text/html` body that is not JSON | FAIL | `drs.object.schema` | schema |
| HLX-MUT-007 | truncated JSON | FAIL | `drs.object.schema` | schema |
| HLX-MUT-008 | `id` is not `test-object-1` | FAIL | `drs.object.schema` | schema |
| HLX-MUT-010 | unknown id HTTP 200 | FAIL | `drs.object.not_found` | error_handling |
| HLX-MUT-011 | unknown id HTTP 500 | FAIL | `drs.object.not_found` | error_handling |
| HLX-MUT-012 | echo first state COMPLETE | FAIL | `wes.run.lifecycle_success` | lifecycle |
| HLX-MUT-013 | fail workflow ends COMPLETE | FAIL | `wes.run.failure_state` | lifecycle |
| HLX-MUT-014 | `supported_wes_versions` is only `2.0` | FAIL | `wes.service_info.schema` | schema |
| HLX-MUT-016 | expired Bearer accepted (`helix security`) | FAIL | `auth.helix.token.expired` | security |
| HLX-MUT-017 | valid Bearer denied (`helix security`) | FAIL | `auth.helix.token.valid` | security |
| HLX-MUT-018 | Range ignored (HTTP 200 full body) | FAIL | `drs.object.range` | range |
| HLX-MUT-019 | 206 without `Content-Range` | FAIL | `drs.object.range` | range |
| HLX-MUT-021 | object probes slower than Helix timeout | not PASS, `passed=0` | (none executed) | fail_closed |
| HLX-MUT-022 | WES service-info wrong JSON types | FAIL | `wes.service_info.schema` | schema |

Security rows have no `diagnostic` object today. The corpus still records class `security`. Dummy HMAC only.

HLX-MUT-021 is fail-closed, not a dedicated timeout check id. That is still detection of a hostile target (overall not PASS).

### Missed (reason for every miss)

| ID | Defect | Hypothesized check | Why Helix misses it |
|----|--------|--------------------|---------------------|
| HLX-MUT-004 | existing object HTTP 500 | `drs.object.reachable` | Discovery treats non-2xx/401/403 as NOT_DETECTED. The reachable check never runs, so Helix cannot classify “wrong HTTP status”. Overall is not PASS (skip-only). Fail-closed ≠ classified HTTP-semantics fail. |
| HLX-MUT-006 | valid JSON + `Content-Type: text/plain` | `drs.object.schema` | HelixTest `get_json` ignores Content-Type. No content-negotiation check. Not added as a Helix-owned MUST. |
| HLX-MUT-009 | DRS bulk `GET /objects` pagination broken | — | `helix verify` never lists objects. Pagination uncovered ([BEHAVIOR.md](BEHAVIOR.md)). |
| HLX-MUT-015 | `type.version` `9.9.9` vs list still containing `1.1` | `wes.service_info.schema` | HelixTest only requires the list to contain `1.0` or `1.1`. Unversioned verify does not fail on `type.version` disagreement. |
| HLX-MUT-020 | extra DrsObject field | `drs.object.schema` | Vendored schema does not set `additionalProperties: false`. |
| HLX-MUT-023 | `type.version` `1.0.0` vs `supported_wes_versions` `["1.1"]` | `wes.service_info.schema` | Both can satisfy the list check independently. No contradiction MUST. |
| HLX-MUT-024 | WES `GET /runs` list paging broken | — | `helix verify` never calls ListRuns. |

A miss becoming an executed **FAIL** of the hypothesized check is a catalog bug: update `src/mutation.rs` from Missed to Detected. Do not delete the row.

---

## 4. What was not added (on purpose)

- No new check ids.
- No `kind=normative` bindings.
- No DRS/WES pagination or Content-Type MUST.
- No fail on extra JSON fields.
- No unversioned-verify fail on declared-version contradiction.
- No “500 object is DETECTED DRS” change (that would silently treat every 5xx origin as a DRS).
- No aggregated detection percentage.

If a behaviour cannot be justified from pinned spec text Helix loads, a normative API definition Helix executes, official guidance (none pinned), or a clearly labeled Helix fixture/interoperability test, it is not implemented as normative.

---

## 5. How to read a miss

A missed mutation is evidence that Helix **does not** cover that defect class as an executed, classified fail. It is not a promise to add the check later, and it is not a claim the target is correct.

```text
attempted = detected + missed
```

That identity is tested. There is no `percent` field.
