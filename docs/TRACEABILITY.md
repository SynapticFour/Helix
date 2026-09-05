# Helix check traceability

**Status:** Implemented. Catalog: `src/traceability.rs`. JSON: `traceability` on every `helix verify` check row. CLI: `helix standards trace CHECK_ID`.

This is not GA4GH certification, endorsement, or a MUST list. It exists so a skeptical reviewer can follow one Helix check back to whatever source Helix is willing to name — or see, in machine-readable form, that Helix cannot name one.

Helix is HelixTest becoming a standalone VERIFY CLI. HELIOS (`helios-audit`) still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md). Registry: [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md).

---

## 1. The chain (when it is complete)

For a **normative** check the recorded chain is:

GA4GH standard → release → exact repository → exact commit → exact file → structural locator (OpenAPI `operationId`, schema name, HTTP path, JSON Pointer, or quoted status) → Helix `check_id` → implementation → expected behaviour → observed behaviour → result

Helix **does not invent** requirement identifiers. GA4GH OpenAPI does not ship RFC-style `MUST-001` numbers. The most stable identifier in a pinned OpenAPI file is used (`operationId`, `components.schemas.*` name, or a `paths` key). Line numbers are not used: they move.

If that chain is incomplete, the check **must not** have `check_kind: normative`.

---

## 2. Kinds

Claim taxonomy (same as `traceability.category`). Full semantics: [TAXONOMY.md](TAXONOMY.md).

| `check_kind` / `category` | `claim_scope` | May support “verified against GA4GH {product} {version}”? |
|---------------------------|---------------|----------------------------------------------------------|
| `normative` | `ga4gh_requirement` | Only if that check passed **and** selected pack = executed pack |
| `guidance` | `guidance_not_requirement` | **No** (official GA4GH implementation guidance only; HelixTest policy is **not** this) |
| `fixture` | `helix_fixture` | **No** |
| `interoperability` | `interoperability_observation` | **No** |
| `security` | `security_behavior` | **No**. Security PASS is not GA4GH conformance |
| `benchmark` | `performance_measurement` | **No** |

`authority` is `ga4gh` | `helix` | `helixtest`. Only `ga4gh` + `normative` + a complete locator may be quoted as a specification requirement.

`executed[].category` (schema, lifecycle, …) is a **domain** label. It is not this taxonomy.

`executed[].layer` / `traceability.layer` is a third axis (schema vs behavior vs security vs interoperability). SCHEMA PASS is not BEHAVIOR PASS. [BEHAVIOR.md](BEHAVIOR.md).

---

## 3. What is true today (fail closed)

**Exactly one shipped Helix check is `normative` (`drs.object.schema.openapi`).** Other DRS verify rows remain fixture. A fixture PASS is not a normative PASS.

Reasons (do not paper over):

1. Default `helix verify` is still unversioned and uses HelixTest-vendored OpenAPI.
2. DRS 1.4.0 is **SUPPORTED** for technical verification within the declared coverage boundary (partial schema: GetObject 200 DrsObject via SpecSource; behavior/security none). SUPPORTED is not VERIFIED. `verified_version` stays empty until every claims predicate holds.
3. DRS 1.5.0 and WES 1.1.0 remain AVAILABLE, not SUPPORTED. WES 1.1.0 still has an HTTPS `$ref` to `ga4gh-service-info` and is **not** executable as a SpecSource.
4. HelixTest fixture extras (`test-object-1`, non-empty `access_methods`, Range 206) are not locators that Helix treats as GA4GH MUSTs.

WES 1.1.0 is the one pin whose vendor file **does** contain `operationId: GetServiceInfo`, `ServiceInfo`, and `RunWorkflow`. Helix still does **not** load that file at verify time, so `wes.service_info.reachable` stays `interoperability` with a `related_source` plus a **limitation**. Schema and run checks that include HelixTest extras stay `fixture`.

TES, TRS, Beacon, and htsget have **no** Helix registry row. They are catalogued as HelixTest wraps, not executed by `helix verify`.

---

## 4. How to follow one check by hand

Example: `drs.object.schema` (a typical JSON PASS on the fixture mock).

### Step A — result

```bash
helix verify http://127.0.0.1:PORT --format json
```

Open `executed[]` where `id` is `drs.object.schema`. Read:

| Field | What it answers |
|-------|-----------------|
| `id` / `code` | Helix identity ([TEST_IDENTITY.md](TEST_IDENTITY.md)) |
| `status` / `message` / `diagnostic.observed` | Result and observation |
| `traceability.category` / `check_kind` | `fixture` today — **not** a MUST |
| `traceability.claim_scope` | `helix_fixture` |
| `traceability.authority` | `helixtest` |
| `traceability.expected_behavior` | What HelixTest asserted |
| `traceability.implementation` | HelixTest `framework/src/drs.rs` function |
| `traceability.untraceable_reason` | Why this is not `normative` |
| `traceability.version` | Empty unless `kind=normative` **and** a pack ran |
| `traceability.related_source` | AVAILABLE pin a reviewer can open. **Not** verified-against |

`selected_version` / `verified_version` on the same row stay empty on default unversioned verify. Do not copy `related_source.version` into a certification sentence.

### Step B — catalog without a target

```bash
helix standards trace drs.object.schema
helix standards trace drs.object.schema --format json
```

Same facts as JSON `traceability`, plus the manual follow-back commands. Exit 1 on an unknown id. Does not run verify. Does not download GA4GH.

### Step C — the related pin (if present)

```bash
helix standards show drs 1.4.0
```

Confirm `commit`, `vendor_path`, and `integrity.hex`. Open:

`standards/vendor/ga4gh.drs.1.4.0/openapi/components/schemas/DrsObject.yaml`

and the rest of that pack’s `openapi/` tree. Search for `related_source.locator` (for this check: `/objects/{object_id}`). Confirm the **limitation**: default unversioned verify still does not load these bytes; the versioned path does only when a pack is selected for execution (no shipped SUPPORTED row).

Hash the file (`sha256sum`) and compare to the registry hex. `helix standards validate` already does that. Join hashes on a versioned run are `pack_integrity_sha256` / `schema_document_sha256` / `schema_component_sha256` — not `verified_version`.

### Step D — the implementation

Open the `implementation` path (HelixTest `helixtest/crates/framework/src/drs.rs` `level1_basic_schema_and_fields` on pin [VERSIONS.lock](../VERSIONS.lock)). Confirm it calls `validate_drs_object` on HelixTest’s flatten **and** extra field checks (`test-object-1`, `access_methods`). Those extras are why `category` is `fixture` and `claim_scope` is `helix_fixture`, not `normative`.

### Step E — reject the run if needed

A reviewer may reject the result by showing any of:

- `check_kind` is `normative` while `untraceable_reason` is set (Helix tests forbid this in the catalog)
- a `fixture` row is serialized as `normative` or `claim_scope: ga4gh_requirement`
- `traceability.version` ≠ `selected_version` of the executed pack
- `source_file` is not in that pack’s `normative_sources`
- the vendor hash does not match
- the English report says “GA4GH MUST” / “certification” for a fixture row

---

## 5. JSON field list (normative minimum)

On `traceability`:

`check_id`, `category`, `check_kind` (must equal `category`), `claim_scope`, `authority`, `expected_behavior`, `implementation`, plus when `kind=normative`: `standard`, `version`, `release_class`, `registry_entry`, `source_repository`, `source_commit`, `source_sha256`, `source_file`, `source_location`.

Today those pack fields are **omitted** (not guessed). `related_source` may still name an AVAILABLE pin.

Schema: [helix-verification-v1.json](../schemas/helix-verification-v1.json) `$defs/traceability`. A `kind=normative` object without commit/file is schema-invalid.

---

## 6. Tests that must stay red if someone weakens this

| Failure | Where |
|---------|--------|
| Catalog id without a provenance row | `src/traceability.rs` `catalog_covers_every_spec` |
| Catalog row labeled `normative` without GA4GH provenance | same (B3: only `drs.object.schema.openapi`) |
| Related locator missing from vendor bytes | `related_locators_are_in_vendor_files` |
| Fixture JSON contains `"normative"` | `fixture_json_cannot_serialize_as_normative` |
| Fixture row mutated to `normative` | `fixture_mislabeled_normative_fails_validate_result` |
| Normative row without commit | `normative_without_commit_fails` |
| `traceability.version` ≠ executed pack | `claimed_version_must_match_executed_pack` |
| `source_file` not in the pin | `source_file_not_in_normative_sources_fails` |
| `release_class: latest` | `invalid_release_class_fails` |
| Registry SUPPORTED binding citing a file not in `normative_sources` | `tests/standards_registry.rs` |
| Generated verify JSON `check_kind=normative` | `tests/schema_verify.rs` |

Do not weaken those tests to match a convenient catalog edit.

---

## 7. When a check may become `normative`

All of the following, in one change:

1. Registry row **SUPPORTED** (steps 1–7 in [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md)).
2. Complete vendor tree (DRS `$ref` siblings included) hashed.
3. HelixTest / Helix validator **loads those bytes** (hash equal).
4. The check asserts only what the locator says — no silent fixture extras, or extras split into a second `kind=fixture` id.
5. Catalog row: `kind=normative`, `authority=ga4gh`, no `untraceable_reason`, commit + file + locator filled from the executed pack.
6. Tests above updated to expect that **one** id, not to allow a blanket “everything is normative.”

Until then, fail closed. A related AVAILABLE locator is an audit hint, not a MUST.

**Who reviews:** [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) §10.1. There is no GA4GH mapping board. The steward merge is not approval.
