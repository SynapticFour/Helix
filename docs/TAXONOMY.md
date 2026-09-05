# Helix check taxonomy

**Status:** Implemented. Catalog: `src/traceability.rs`. JSON: `traceability.category`, `traceability.check_kind` (same value), `traceability.claim_scope`, `traceability.authority`.

This is **not** GA4GH certification. Categories exist so a PASS cannot be silently read as a specification requirement.

Helix is HelixTest becoming a standalone VERIFY CLI. HELIOS (`helios-audit`) still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md). Provenance chain: [TRACEABILITY.md](TRACEABILITY.md).

Two different `category` fields exist on purpose:

| JSON path | Meaning | Example |
|-----------|---------|---------|
| `executed[].category` | Domain (what area of behaviour) | `schema`, `lifecycle`, `robustness` |
| `executed[].traceability.category` | Claim taxonomy (what a PASS is allowed to mean) | `fixture`, `interoperability`, `security` |

Do not collapse them. A DRS schema check can be domain `schema` and taxonomy `fixture` at the same time.

A third axis is the **evidence layer** (`executed[].layer`): `schema` | `behavior` | `security` | `interoperability`. SCHEMA PASS is not BEHAVIOR PASS. [BEHAVIOR.md](BEHAVIOR.md).

---

## 1. Why these categories exist

HelixTest already runs DRS and WES checks. Those checks mix:

- behaviour that looks like a GA4GH OpenAPI operation
- HelixTest-vendored schemas (not `standards/vendor` bytes)
- Helix-defined object ids, workflow URLs, and HTTP Range probes
- Helix-owned auth / Crypt4GH layout
- `helix bench` timings

Without a machine-readable split, a green `helix verify` is easy to quote as “the target conforms to GA4GH.” That sentence is not justified: DRS 1.4.0 SUPPORTED is not VERIFIED, unversioned verify does not load hashed vendor files, and several checks assert fixture extras.

The taxonomy is fail-closed:

- **Fewer PASSes that look official is acceptable.**
- **A fixture PASS labeled as a GA4GH MUST is not.**
- Uncertain classification is **not** upgraded toward `normative`.
- HelixTest policy is **not** `guidance`. Official GA4GH implementation guidance would need a pinned source. None is pinned.

---

## 2. Semantics

| `category` (`check_kind`) | `claim_scope` | Meaning | PASS may support “verified against GA4GH {product} {version}”? |
|---------------------------|---------------|---------|----------------------------------------------------------------|
| `normative` | `ga4gh_requirement` | Derived directly from a normative requirement in the **pinned** GA4GH file Helix **loaded**. `authority` is `ga4gh`. | Only if that check passed **and** selected pack = executed pack |
| `guidance` | `guidance_not_requirement` | Derived from **official** GA4GH implementation guidance, not itself a MUST. | **No** |
| `fixture` | `helix_fixture` | Defined by Helix or HelixTest for deterministic testing. Never a GA4GH requirement. | **No** |
| `interoperability` | `interoperability_observation` | Cross-system / practical interoperability. Must not automatically become a conformance claim. | **No** |
| `security` | `security_behavior` | Security behaviour Helix tests. | **No**. Security PASS does not imply GA4GH conformance or that the target is secure. |
| `benchmark` | `performance_measurement` | Performance measurement only. | **No**. Never a conformance claim. Never a verification failure. |

`authority` is `ga4gh` | `helix` | `helixtest`. Only `ga4gh` + `normative` + `claim_scope=ga4gh_requirement` + a complete locator may be quoted as a specification requirement.

HelixTest extras (`test-object-1`, `trs://test-tool/echo/1.0`, `supported_wes_versions` contains `1.0` **or** `1.1`, HTTP Range 206) are **fixture**, not `guidance`. They are not official GA4GH implementation guidance.

---

## 3. What is true today

**Exactly one shipped Helix check is `normative` (`drs.object.schema.openapi`).** **No Helix check is `guidance`.** Other DRS verify rows have `untraceable_reason`. Fixture `claim_scope` is never `ga4gh_requirement`.

Reasons (do not paper over): see [TRACEABILITY.md](TRACEABILITY.md) §3.

---

## 4. Audit of shipped checks

Conservative rule used: if a check mixes a schema/surface probe with HelixTest extras, it is `fixture`. If classification is uncertain, the row stays away from `normative` and `guidance`. No existing check was upgraded to `normative`.

### DRS (`helix verify` wrap)

| `check_id` | taxonomy | authority | why |
|------------|----------|-----------|-----|
| `drs.object.schema.openapi` | normative | ga4gh | Versioned SpecSource only: pinned DRS 1.4.0 DrsObject schema, no HelixTest extras. SCHEMA PASS is not behavior coverage. |
| `drs.object.reachable` | fixture | helixtest | Probe uses fixture object id `test-object-1` |
| `drs.object.schema` | fixture | helixtest | HelixTest-vendored schema **plus** extras (`id=test-object-1`, `self_uri`, `name`, non-empty `access_methods`). Mixed → fixture, not interoperability |
| `drs.object.checksum` | fixture | helixtest | Checksum of fixture object bytes |
| `drs.object.range` | fixture | helixtest | HTTP Range probe; not located in the pinned DRS entry YAML |
| `drs.object.not_found` | fixture | helixtest | Helix unknown-id fixture |

### WES (`helix verify` wrap)

| `check_id` | taxonomy | authority | why |
|------------|----------|-----------|-----|
| `wes.service_info.reachable` | interoperability | helixtest | GET service-info reachability; no Helix object-id fixture. Still not a MUST (HelixTest does not load vendor bytes) |
| `wes.service_info.schema` | fixture | helixtest | HelixTest-vendored schema **plus** HelixTest policy that `supported_wes_versions` contains `1.0` or `1.1`. That policy is not official guidance |
| `wes.run.lifecycle_success` | fixture | helixtest | Echo TRS URL fixture |
| `wes.run.failure_state` | fixture | helixtest | Fail-workflow fixture |
| `wes.run.missing_inputs` | fixture | helixtest | Missing-input fixture |
| `wes.run.incompatible_type` | fixture | helixtest | Incompatible `workflow_type` fixture |
| `wes.run.invalid_workflow` | fixture | helixtest | Invalid workflow fixture |
| `wes.run.scatter_gather` | fixture | helixtest | Scatter/gather fixture (skip is never pass) |

### TES / TRS / Beacon / htsget (catalogued, not executed by `helix verify`)

Reachable / schema wraps without a named Helix object fixture stay `interoperability` (cross-system observation, still not a MUST; no registry pack). Named scenario rows are `fixture`. `htsget.dataset.auth` is `security`.

| `check_id` | taxonomy |
|------------|----------|
| `tes.tasks.reachable` | interoperability |
| `tes.task.schema` | interoperability |
| `tes.task.lifecycle_checksum` | fixture |
| `trs.tools.reachable` | interoperability |
| `trs.tools.schema` | interoperability |
| `trs.descriptor.retrieve` | fixture |
| `beacon.query.reachable` | interoperability |
| `beacon.query.boolean_schema` | interoperability |
| `beacon.variant.known_exists` | fixture |
| `beacon.variant.negative_absent` | fixture |
| `htsget.reads.service_info` | interoperability |
| `htsget.variants.service_info` | interoperability |
| `htsget.reads.ticket.get` | fixture |
| `htsget.variants.ticket.get` | fixture |
| `htsget.variants.wrong_object` | fixture |
| `htsget.reads.ticket.post` | fixture |
| `htsget.reads.ticket.post_regions` | fixture |
| `htsget.variants.ticket.post` | fixture |
| `htsget.variants.ticket.post_regions` | fixture |
| `htsget.reads.post_query_invalid` | fixture |
| `htsget.reads.format_cram` | fixture |
| `htsget.reads.class_header` | fixture |
| `htsget.dataset.auth` | security |
| `htsget.suite.unresolved` | fixture |

### Auth and Crypt4GH

All `security`. HelixTest HMAC wraps (`auth.token.*`, `auth.service_info.reachable`) and Helix-native profile (`auth.helix.*`). A PASS is dummy-fixture behaviour. It does not prove the target is secure and does not imply GA4GH conformance.

### Discovery

`discovery.drs` / `wes` / `tes` / `trs` / `htsget`: `interoperability`, `authority: helix`. DETECTED is not a pass. Endpoint existence is not a requirement identifier.

### Benchmarks

`bench.get.*` and `bench.metric.*`: `benchmark`. Timing and RSS only. Thresholds do not fail CI.

---

## 5. How to read a result

JSON:

```text
executed[].traceability.category
executed[].traceability.claim_scope
executed[].traceability.authority
```

Text report (`helix verify`):

```text
        kind: fixture  claim_scope: helix_fixture  authority: helixtest
        not a GA4GH MUST  (PASS is not a conformance claim)
```

and a run-level **Evidence** count (classification, not a score). Default unversioned verify still has zero `normative` rows. Versioned DRS 1.4.0 includes `drs.object.schema.openapi`; that PASS still does not set `verified_version`.

Catalog without a target: `helix standards trace CHECK_ID`.

Schema: [helix-verification-v1.json](../schemas/helix-verification-v1.json). A fixture row with `check_kind: normative` or `claim_scope: ga4gh_requirement` is schema-invalid.

---

## 6. Tests that must stay red

| Failure | Where |
|---------|--------|
| Catalog row `normative` without GA4GH provenance | `src/traceability.rs` `catalog_covers_every_spec` |
| Fixture JSON contains `"normative"` | `fixture_json_cannot_serialize_as_normative` |
| Fixture mutated to `check_kind=normative` | `fixture_mislabeled_normative_fails_validate_result` |
| Fixture `claim_scope=ga4gh_requirement` | `fixture_claim_scope_ga4gh_requirement_fails` |
| Schema accepts fixture labeled normative | `tests/schema_verify.rs` |
| Mixed DRS/WES schema extras labeled interoperability | `mixed_schema_checks_are_fixture_not_interoperability` |

Do not weaken those tests to produce fewer failures.

---

## 7. When a check may become `normative` or `guidance`

`normative`: same bar as [TRACEABILITY.md](TRACEABILITY.md) §7 (SUPPORTED pack, vendor tree loaded, assertion equals locator, extras split out). B3 adds `drs.object.schema.openapi` only.

`guidance`: a pinned official GA4GH implementation-guidance document Helix loads, a locator in that file, `authority=ga4gh`, `claim_scope=guidance_not_requirement`, and still **no** conformance sentence.

Until then, fail closed. **Who reviews:** [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) §10.1. There is no GA4GH mapping board.

---

## 8. Claims

Human-readable VERIFIED / NOT_VERIFIED text is generated only from `claims[]` ([CLAIMS.md](CLAIMS.md)). Taxonomy `fixture` PASS cannot satisfy a `ga4gh_requirement` claim. Fixture FAIL cannot produce `normative_check_failed`.
