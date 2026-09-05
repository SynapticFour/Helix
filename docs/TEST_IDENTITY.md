# Helix test identity

Stable identities for Helix verification checks. HelixTest **names** (the `TestCaseResult.name` strings) are not renamed. Where Helix wraps a HelixTest check, the mapping is **Helix ID → HelixTest test name** (exact string).

Source of truth in code: `src/identity.rs`. Domain types: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md). HelixTest name inventory: [INVENTORY.md](../INVENTORY.md) (pin v0.1.3).

Changing an **existing** Helix `id` or `code` is a **compatibility change**. helix-action, CI comments, and any stored JSON keyed by these values break. Add new IDs; do not recycle codes; do not rename in place.

This is not certification, not HELIOS, not a compliance score.

---

## 1. Fields every check has

| Field | Rule | Example |
|-------|------|---------|
| `id` | Dotted semantic id, lowercase, service first | `drs.object.not_found` |
| `code` | Failure / catalog code, `HLX-<FAMILY>-NNN` | `HLX-DRS-005` |
| `name` | Helix human-readable title (may differ from HelixTest) | `Unknown DRS object returns 404` |
| `service` | Open string (`drs`, `wes`, …) | `drs` |
| `category` | Helix **domain** (not a HelixTest level; not the claim taxonomy) | `robustness` |
| `severity` | Default severity if the check **fails** (`info` / `warn` / `error`) | `error` |

HelixTest wrap (optional): one or more exact `TestCaseResult.name` strings. Empty for Helix-native checks (discovery, `helix security`, `helix bench`).

Every assigned id also has a **traceability** row (`src/traceability.rs`, JSON `traceability`, `helix standards trace`): `category` (claim taxonomy), `check_kind` (same value), `claim_scope`, `authority`, `expected_behavior`, `implementation`, `layer` (schema / behavior / security / interoperability / benchmark), optional `request`, and either a complete GA4GH locator or `untraceable_reason`. None of the shipped rows are `kind=normative` or `guidance`. Domain `category` (this table) is not `traceability.category`. `layer` is a third axis: SCHEMA PASS is not BEHAVIOR PASS ([BEHAVIOR.md](BEHAVIOR.md)). Follow-back: [TAXONOMY.md](TAXONOMY.md), [TRACEABILITY.md](TRACEABILITY.md). Not certification.

`code` is the stable failure code. A result’s `failure.code` repeats it on FAIL/ERROR. Do not invent a second numbering system. Fail/error rows for assigned DRS and WES ids may also carry a deterministic `diagnostic` that repeats `id` / `code` plus expected/observed text ([DIAGNOSTICS.md](DIAGNOSTICS.md)). That is not a new family of codes.

---

## 2. Code families and reserved ranges

Format: `HLX-<FAMILY>-` + three digits. Families:

| Family | Prefix | Use |
|--------|--------|-----|
| DRS | `HLX-DRS-` | DRS conformance (HelixTest wrap) |
| WES | `HLX-WES-` | WES conformance |
| TES | `HLX-TES-` | TES conformance |
| TRS | `HLX-TRS-` | TRS conformance |
| htsget | `HLX-HTSGET-` | htsget conformance |
| Beacon | `HLX-BEACON-` | Beacon conformance |
| Auth | `HLX-AUTH-` | Auth behaviour (HelixTest HMAC wrap + Helix `security`) |
| Bench | `HLX-BENCH-` | Performance measurement (Helix-native) |
| Discovery | `HLX-DISCOVERY-` | Service presence probes (Helix-native) |

**Per-family number ranges (do not pick digits at random):**

| Range | Meaning |
|-------|---------|
| **001–049** | Assigned or next sequential assignments for that family’s current suite |
| **050–099** | Reserved same-family expansion (e.g. AUTH 050+ = Crypt4GH) |
| **100–899** | Reserved future expansion |
| **900–999** | Reserved runner / meta (do not assign yet) |

Next new DRS check is `HLX-DRS-006`, not `HLX-DRS-100` and not a gap. Do not reuse a retired code.

Not assigned (HelixTest still has names; Helix has no ID yet): Age, E2E, africa, infra, Auth token-only (dynamic `format!` names), Crypt4GH HTTP beyond the reserved AUTH 051–052 placeholders. Assigning those later is an **addition**, not a rename.

There is no `HLX-BCN-` prefix. Beacon is `HLX-BEACON-`.

---

## 3. Compatibility

**Breaking (must not do silently):**

- Change `id` or `code` of an assigned check
- Point an existing `id` at a different HelixTest name
- Reuse a code for a different semantic check

**Non-breaking:**

- Add a new id/code in the next sequential slot
- Fill a reserved range when the stage actually wraps that surface
- Change Helix `name` (human title) only with a changelog note — machines must key on `id`/`code`
- HelixTest renaming its string is a HelixTest change; Helix must update the mapping table and treat it as compatibility for anyone matching HelixTest names

Unit tests freeze `(id, code)` pairs and HelixTest name → id maps. If you need to change a frozen pair, you are making a compatibility change: update [CHANGELOG.md](../CHANGELOG.md) and this file in the same commit.

---

## 4. Categories and severity

Helix categories (JSON snake_case): `discovery`, `schema`, `lifecycle`, `checksum`, `robustness`, `security`, `performance`, `other`.

They are **not** HelixTest `ComplianceLevel` and not a score. HelixTest `TestCategory` is not imported here.

Default fail severity: `error` for conformance and auth; `warn` for bench metrics (bench does not fail the process today — [CLI_CONTRACT.md](CLI_CONTRACT.md)). Runner `error` status always uses severity `error`.

---

## 5. Catalog and HelixTest mapping

Helix `name` may differ from the HelixTest string. The **mapping column is the HelixTest name** and must match `framework` source on pin v0.1.3.

### 5.1 DRS — `HLX-DRS-001`–`005` (wired in `helix verify`)

| id | code | Helix name | HelixTest test name |
|----|------|------------|---------------------|
| `drs.object.reachable` | `HLX-DRS-001` | DRS object endpoint is reachable | `DRS object endpoint reachable` |
| `drs.object.schema` | `HLX-DRS-002` | DRS object matches OpenAPI and has access methods | `DRS DrsObject OpenAPI + access_methods` |
| `drs.object.checksum` | `HLX-DRS-003` | DRS object checksum is correct | `DRS checksum correctness` |
| `drs.object.range` | `HLX-DRS-004` | DRS object supports HTTP Range | `DRS HTTP Range support` |
| `drs.object.not_found` | `HLX-DRS-005` | Unknown DRS object returns 404 | `DRS invalid object id returns 404` |

006–049 reserved sequential. Do not rename HelixTest names in HelixTest to match Helix `name`.

### 5.2 WES — `HLX-WES-001`–`008` (wired in `helix verify`)

HelixTest already runs these eight names. Helix executes them through the generic adapter when WES is DETECTED and TESTABLE. Fixture URLs, poll contract, and scatter skip: [WES.md](WES.md). Scatter/gather (`HLX-WES-008`) is **skipped** on profile `generic` (`supports_scatter_gather=false`) and **executed** on profile `ferrum`; skip is never pass. A generic target never auto-switches to ferrum. See [PROFILES.md](PROFILES.md).

| id | code | Helix name | HelixTest test name |
|----|------|------------|---------------------|
| `wes.service_info.reachable` | `HLX-WES-001` | WES service-info is reachable | `WES service-info reachable` |
| `wes.service_info.schema` | `HLX-WES-002` | WES service-info matches HelixTest-vendored ServiceInfo schema | `WES service-info schema (GA4GH official)` |
| `wes.run.lifecycle_success` | `HLX-WES-003` | WES echo workflow reaches success | `WES lifecycle success echo (API may show QUEUED/INITIALIZING/RUNNING before COMPLETE)` |
| `wes.run.failure_state` | `HLX-WES-004` | WES reports failure for a bad workflow | `WES failure state for bad workflow` |
| `wes.run.missing_inputs` | `HLX-WES-005` | WES errors when inputs are missing | `WES missing inputs leads to error state` |
| `wes.run.incompatible_type` | `HLX-WES-006` | WES errors on incompatible workflow_type | `WES incompatible workflow_type leads to error state` |
| `wes.run.invalid_workflow` | `HLX-WES-007` | WES errors on an invalid workflow | `WES invalid workflow leads to error state` |
| `wes.run.scatter_gather` | `HLX-WES-008` | WES scatter/gather workflow | `WES scatter/gather workflow` |

### 5.3 TES — `HLX-TES-001`–`003` (not wired in `helix verify`)

| id | code | Helix name | HelixTest test name |
|----|------|------------|---------------------|
| `tes.tasks.reachable` | `HLX-TES-001` | TES /tasks is reachable | `TES /tasks reachable` |
| `tes.task.schema` | `HLX-TES-002` | TES task create and status match schema | `TES task schema (create + status)` |
| `tes.task.lifecycle_checksum` | `HLX-TES-003` | TES task lifecycle and output checksum | `TES task lifecycle + checksum (non-terminal states allowed until terminal)` |

### 5.4 TRS — `HLX-TRS-001`–`003` (not wired in `helix verify`)

| id | code | Helix name | HelixTest test name |
|----|------|------------|---------------------|
| `trs.tools.reachable` | `HLX-TRS-001` | TRS /tools is reachable | `TRS /tools reachable` |
| `trs.tools.schema` | `HLX-TRS-002` | TRS tools and versions match schema | `TRS tools and versions schema` |
| `trs.descriptor.retrieve` | `HLX-TRS-003` | TRS descriptor can be retrieved | `TRS descriptor retrieval` |

### 5.5 Beacon — `HLX-BEACON-001`–`004`

| id | code | Helix name | HelixTest test name |
|----|------|------------|---------------------|
| `beacon.query.reachable` | `HLX-BEACON-001` | Beacon /query is reachable | `Beacon /query reachable` |
| `beacon.query.boolean_schema` | `HLX-BEACON-002` | Beacon boolean response matches schema | `Beacon boolean response (official schema)` |
| `beacon.variant.known_exists` | `HLX-BEACON-003` | Beacon reports a known variant exists | `Beacon known variant exists` |
| `beacon.variant.negative_absent` | `HLX-BEACON-004` | Beacon reports a negative variant as absent | `Beacon negative variant not exists` |

### 5.6 htsget — `HLX-HTSGET-001`–`014` (not wired in `helix verify`)

One Helix ID per semantic check. Where HelixTest uses two names (generic vs Ferrum `InvalidInput` for POST regions), **both** strings map to the same Helix ID.

| id | code | HelixTest test name(s) |
|----|------|------------------------|
| `htsget.reads.service_info` | `HLX-HTSGET-001` | `htsget reads /reads/service-info (htsget 1.3.0)` |
| `htsget.variants.service_info` | `HLX-HTSGET-002` | `htsget variants /variants/service-info (htsget 1.3.0)` |
| `htsget.reads.ticket.get` | `HLX-HTSGET-003` | `htsget GET reads ticket (BAM + DRS stream URL)` |
| `htsget.variants.ticket.get` | `HLX-HTSGET-004` | `htsget GET variants ticket (VCF/BCF + DRS stream URL)` |
| `htsget.variants.wrong_object` | `HLX-HTSGET-005` | `htsget GET variants with reads-only object → NotFound` |
| `htsget.reads.ticket.post` | `HLX-HTSGET-006` | `htsget POST reads ticket (JSON body, no query)` |
| `htsget.reads.ticket.post_regions` | `HLX-HTSGET-007` | `htsget POST reads ticket (JSON body with regions)` **and** `htsget POST reads ticket with regions → InvalidInput (Ferrum does not slice)` |
| `htsget.variants.ticket.post` | `HLX-HTSGET-008` | `htsget POST variants ticket (JSON body, no query)` |
| `htsget.variants.ticket.post_regions` | `HLX-HTSGET-009` | `htsget POST variants ticket (JSON body with regions)` **and** `htsget POST variants ticket with regions → InvalidInput (Ferrum does not slice)` |
| `htsget.reads.post_query_invalid` | `HLX-HTSGET-010` | `htsget POST reads with query params → InvalidInput` |
| `htsget.reads.format_cram` | `HLX-HTSGET-011` | `htsget GET reads ?format=CRAM on BAM object → UnsupportedFormat` |
| `htsget.reads.class_header` | `HLX-HTSGET-012` | `htsget GET reads ?class=header → InvalidInput` |
| `htsget.dataset.auth` | `HLX-HTSGET-013` | `htsget dataset auth (403 without token, 200 with Passport/JWT)` |
| `htsget.suite.unresolved` | `HLX-HTSGET-014` | `htsget suite (service-info, tickets, POST, errors)` |

### 5.7 Auth — HelixTest HMAC wrap `HLX-AUTH-001`–`006`

| id | code | HelixTest test name |
|----|------|---------------------|
| `auth.service_info.reachable` | `HLX-AUTH-001` | `Auth /service-info reachable (auth_url)` |
| `auth.token.valid` | `HLX-AUTH-002` | `Auth (HMAC JWT fixture): valid token grants DRS access` |
| `auth.token.expired` | `HLX-AUTH-003` | `Auth (HMAC JWT fixture): expired token rejected` |
| `auth.token.garbage` | `HLX-AUTH-004` | `Auth (HMAC JWT fixture): garbage bearer rejected` |
| `auth.token.wrong_scope` | `HLX-AUTH-005` | `Auth (HMAC JWT fixture): wrong scope denied` |
| `auth.token.missing` | `HLX-AUTH-006` | `Auth (HMAC JWT fixture): missing token returns 401` |

`Auth service URL reachable` (token-protected-endpoints path) is **not** assigned (different code path, not the default HMAC ladder).

### 5.8 Auth — Helix-native Security Behavior Profile `HLX-AUTH-010`–`014`, Crypt4GH `050`

These are **not** wraps of `framework/src/auth.rs`. Similar intent; different names and code. `helixtest_names` is empty. Profile contract (invariant, request, status class, fixture): [SECURITY_PROFILE.md](SECURITY_PROFILE.md). Not a security audit.

| id | code | Helix name (`AUTH_CASE_NAMES` / Crypt4GH) |
|----|------|------------------------------------------|
| `auth.helix.token.valid` | `HLX-AUTH-010` | Security: valid token grants access |
| `auth.helix.token.expired` | `HLX-AUTH-011` | Security: expired token rejected with 401 |
| `auth.helix.token.wrong_scope` | `HLX-AUTH-012` | Security: wrong scope denied |
| `auth.helix.token.manipulated` | `HLX-AUTH-013` | Security: invalid or manipulated token rejected |
| `auth.helix.token.wrong_audience` | `HLX-AUTH-014` | Security: token for another service rejected |
| `auth.helix.crypt4gh.header` | `HLX-AUTH-050` | Security: Crypt4GH header structure is well-formed (no key material in output) |
| `auth.helix.crypt4gh.invalid_rejected` | `HLX-AUTH-053` | Crypt4GH: invalid envelope is rejected (layout only) |
| `auth.helix.crypt4gh.http_envelope` | `HLX-AUTH-054` | Crypt4GH: HTTP body is a Crypt4GH envelope when magic is present |

Reserved AUTH **051–052** for a future wrap of HelixTest Crypt4GH HTTP names (not wired; those checks need a client **secret** key, which Helix will not hold):

- `Crypt4GH DRS rewrap download (X-Crypt4GH-Public-Key)`
- `Crypt4GH plain download matches rewrap plaintext (decrypt_plain)`

Layout contract: [CRYPT4GH.md](CRYPT4GH.md). A pass is not “secure”.

### 5.9 Discovery — Helix-native `HLX-DISCOVERY-001`–`005`

No HelixTest wrap. Presence is not a pass ([ARCHITECTURE.md](ARCHITECTURE.md)).

| id | code | service |
|----|------|---------|
| `discovery.drs` | `HLX-DISCOVERY-001` | drs |
| `discovery.wes` | `HLX-DISCOVERY-002` | wes |
| `discovery.tes` | `HLX-DISCOVERY-003` | tes |
| `discovery.trs` | `HLX-DISCOVERY-004` | trs |
| `discovery.htsget` | `HLX-DISCOVERY-005` | htsget |

### 5.10 Bench — Helix-native `HLX-BENCH-001`–`003`, metrics `010`–`016`

Workload **`http.drs.smoke.v1`**: paths match `src/bench/workload.rs`. Repeatable engine: [BENCHMARKS.md](BENCHMARKS.md). Not Demo hap.py. Not HelixTest. Not a significance test.

| id | code | Helix name |
|----|------|------------|
| `bench.get.health` | `HLX-BENCH-001` | GET /health |
| `bench.get.drs_service_info` | `HLX-BENCH-002` | GET /ga4gh/drs/v1/service-info |
| `bench.get.drs_object` | `HLX-BENCH-003` | GET /ga4gh/drs/v1/objects/test-object-1 |
| `bench.metric.wall_ms` | `HLX-BENCH-010` | Client wall time (median of measured runs) |
| `bench.metric.rss_kb` | `HLX-BENCH-011` | Helix process RSS (Linux) |
| `bench.metric.error_rate` | `HLX-BENCH-012` | Request error rate |
| `bench.metric.p95_ms` | `HLX-BENCH-013` | Sample p95 wall time (reported at ≥ 20 measured runs) |
| `bench.metric.min_ms` | `HLX-BENCH-014` | Minimum measured wall time |
| `bench.metric.max_ms` | `HLX-BENCH-015` | Maximum measured wall time |
| `bench.metric.bytes` | `HLX-BENCH-016` | Response body bytes (measured runs) |

---

## 6. CLI

`helix verify --format json` emits Helix `VerificationRun` with Helix `id` / `code` and `helixtest_name` (original HelixTest string, not renamed). `helix security` still emits HelixTest `OverallReport` with HelixTest names.
