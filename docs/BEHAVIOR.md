# Helix behavioural coverage

**Status:** Pattern implemented. Layers: `src/layer.rs`. Negative fixtures: `tests/support/mock_ga4gh_drs.rs`. Tests: `tests/behavior.rs`.

This is **not** GA4GH certification. Helix is HelixTest becoming a standalone VERIFY CLI. HELIOS (`helios-audit`) still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md). Taxonomy: [TAXONOMY.md](TAXONOMY.md). Provenance: [TRACEABILITY.md](TRACEABILITY.md).

A **SCHEMA PASS is not a BEHAVIOR PASS.** Helix does not emit a “100% compliant” figure. `layer_summary` is four independent count objects plus an honesty `note`. No `percent`, `score`, or `compliant` field. SCHEMA PASS is not a schema VERIFIED claim ([CLAIMS.md](CLAIMS.md)).

---

## 1. Why this exists

OpenAPI/JSON Schema validation answers whether a body matches a shape HelixTest (today) or a pinned vendor file (not yet loaded) knows. It does not answer whether:

- a GET returns the documented status for a missing resource
- advertised checksums match downloaded bytes
- a WES run actually reaches a documented `State`
- HTTP Range, pagination, or content negotiation work

Those are **behaviour**. Mixing them into one PASS count is how schema-only targets get quoted as conformant.

---

## 2. Evidence layers (machine-readable)

JSON: `executed[].layer` and `traceability.layer` (same catalog). Run: `layer_summary`.

| Layer | Report token | What a PASS means | Automatic conformance claim? |
|-------|--------------|-------------------|------------------------------|
| `schema` | SCHEMA PASS / FAIL / ERROR / NONE | Body matched the schema probe | **No** |
| `behavior` | BEHAVIOR PASS / FAIL / ERROR / NONE | Request/response/HTTP/error/state/range/async probe | **No** (today: fixture / interoperability taxonomy) |
| `security` | SECURITY PASS / FAIL / ERROR / NONE | Helix or HelixTest auth / Crypt4GH layout | **No** |
| `interoperability` | INTEROPERABILITY PASS / FAIL / ERROR / NONE | Reachability / discovery | **No**. DETECTED is not a pass |
| `benchmark` | (not a conformance layer) | Timing | **No** |

**NONE** means that layer did not execute. That is not PASS.

Claim taxonomy (`traceability.category`) is a different axis: even a BEHAVIOR PASS is `fixture` today, not `normative`. See [TAXONOMY.md](TAXONOMY.md).

Per behavioural row Helix records: `check_id`, domain `category`, `traceability` (provenance), `traceability.request`, `traceability.expected_behavior`, `observed_response` (fail/error), `diagnostic`.

---

## 3. Inventory (in-scope standards)

Helix `helix verify` executes DRS and WES HelixTest wraps only. TES / TRS / Beacon / htsget are catalogued, not executed. Registry packs are AVAILABLE, not SUPPORTED. HelixTest does not load `standards/vendor` bytes. **No layer is `normative`.**

### 3.1 DRS (executed)

Pinned entry YAML lists path keys and `$ref`s sibling files Helix has **not** vendored. Tags such as `DrsApiPrinciples.md` are not in the pin. Do not invent MUSTs from unvendored `$ref`s.

| Dimension | In the pin? | Helix check | Layer | Classification |
|-----------|-------------|-------------|-------|----------------|
| Schema-level | DrsObject schema is **not** in the entry YAML (`$ref`) | `drs.object.schema` | schema | fixture (HelixTest extras: `test-object-1`, `self_uri`, `name`, non-empty `access_methods`) |
| Request-level | Path `/objects/{object_id}` is listed | reachable + schema GET | interoperability / schema | fixture / interoperability |
| Response-level | Response codes live in unvendored `$ref` | mixed into HelixTest schema/404 | — | not separately bound |
| HTTP semantics | GET implied by path item `$ref` | reachable | interoperability | not a MUST |
| Error semantics | 404 not in the local file | `drs.object.not_found` | behavior | **fixture** (Helix unknown-id). Clearly labeled interoperability/fixture, not normative |
| State transitions | DRS has no run state machine | — | — | **uncovered** (N/A) |
| Pagination | `/objects` bulk `$ref` not vendored | — | — | **uncovered** |
| Content negotiation | not in entry YAML | — | — | **uncovered** |
| Range | not in entry YAML | `drs.object.range` | behavior | **fixture** (HelixTest Range: bytes=0-1023). Not RFC-7233-as-GA4GH-MUST |
| Auth | `security: [{}]` plus Basic/Bearer schemes in the pin: auth is **optional** | Helix `helix security` | security | Helix-owned. Optional DRS auth is **not** implemented as “MUST reject missing token” |
| Checksum vs bytes | checksums field is in unvendored schema | `drs.object.checksum` | behavior | fixture (`test-object-1` download). Not a MUST mapping |

**Covered as a pattern (not a complete DRS suite):** schema vs checksum vs 404 vs Range, with negative fixtures that pass schema and fail behaviour.

### 3.2 WES (executed)

WES 1.1.0 vendor file **does** contain `GetServiceInfo`, `ServiceInfo`, `RunWorkflow`, `GetRunStatus`, `State` enum, ListRuns paging **text**, and 404 on GetRunLog/GetRunStatus. HelixTest **does not load those bytes**. Related locators are audit hints.

| Dimension | In the pin? | Helix check | Layer | Classification |
|-----------|-------------|-------------|-------|----------------|
| Schema-level | `ServiceInfo` in vendor file | `wes.service_info.schema` | schema | fixture (HelixTest `1.0` **or** `1.1` extra) |
| Request-level | POST `/runs` `RunWorkflow` | `wes.run.*` | behavior | fixture (echo TRS URLs) |
| Response-level | `RunId`, `RunStatus` | mixed into lifecycle polls | behavior | fixture |
| HTTP semantics | GET service-info | `wes.service_info.reachable` | interoperability | not a MUST |
| Error semantics | 404 on GetRunStatus in vendor file | not a dedicated Helix id | — | **uncovered** as a separate 404 check |
| State transitions | `State` enum + descriptions in vendor file | `wes.run.lifecycle_success` / `failure_state` / … | behavior | fixture (HelixTest echo/fail workflows). Not “every legal transition” |
| Pagination | ListRuns description in vendor file | — | — | **uncovered** |
| Content negotiation | not asserted | — | — | **uncovered** |
| Range | N/A | — | — | N/A |
| Auth | not a WES MUST in this wrap | — | — | **uncovered** for WES |
| Async workflow | poll `GetRunStatus` until COMPLETE / error states | `wes.run.lifecycle_*` | behavior | fixture |

Scatter/gather (`wes.run.scatter_gather`) is a HelixTest extra; skip on profile `generic` is never pass.

### 3.3 TES / TRS / Beacon / htsget

Not executed by `helix verify`. Catalog rows exist so ids stay stable. Treat every dimension as **uncovered at runtime**. Do not infer coverage from catalog presence.

---

## 4. Negative fixtures (known-bad must fail for the right reason)

| Fixture | SCHEMA | BEHAVIOR | Why it exists |
|---------|--------|----------|----------------|
| Valid mock DRS | PASS | PASS | Good implementation can pass |
| `{ "id": "test-object-1" }` only | FAIL | FAIL (checksum/range/bytes) | Schema-invalid is not the only failure mode |
| Schema-ok, checksum-wrong (`start_mock_schema_ok_checksum_wrong`) | PASS | FAIL (`drs.object.checksum`) | Schema pass must not hide byte mismatch |
| Schema-ok, unknown id returns 200 (`start_mock_schema_ok_unknown_id_200`) | PASS | FAIL (`drs.object.not_found`) | Schema pass must not hide error-semantics failure |

CI: `tests/behavior.rs`. Catalog: [FIXTURES.md](FIXTURES.md). One-defect mutants (including schema-ok / behaviour-fail and documented misses): [MUTATION.md](MUTATION.md).

---

## 5. What was not implemented (on purpose)

- No new `normative` check. Vendor `$ref` tree still incomplete; HelixTest still uses its own OpenAPI.
- No DRS pagination, content negotiation, or “auth required” checks.
- No WES ListRuns paging, GetRunStatus 404, or full `State` transition table.
- No TES/TRS/Beacon/htsget runtime behaviour.
- No aggregated compliance percentage.
- No RFC 7233 Range as a GA4GH MUST.

If a behaviour cannot be justified from the pin, official guidance (none pinned), or a clearly labeled Helix interoperability/fixture test, it is not added as normative.

---

## 6. How to read a run

```bash
helix verify http://127.0.0.1:PORT --format json
```

Look at `layer_summary.schema` vs `layer_summary.behavior`. Text report:

```text
Layers (not a score; SCHEMA PASS is not BEHAVIOR PASS):
  SCHEMA PASS
  BEHAVIOR FAIL
    - drs.object.checksum (fail)
  SECURITY NONE
  INTEROPERABILITY PASS
```

SECURITY NONE is not SECURITY PASS.
