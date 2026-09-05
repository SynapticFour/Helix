# Helix trust principle

**Do not ask the user to trust Helix.**
**Make the code, specifications, provenance, tests, and recorded results sufficient for independent verification.**

The implementation must stand on its own.

Helix is HelixTest becoming a standalone VERIFY CLI. This document is an architectural constraint, not a capability claim and not GA4GH certification. HELIOS (`helios-audit`) still owns signed audit trails, RO-Crate, and PDF. Helix “evidence” here means **inspectable, pinned, recorded run artefacts** (JSON fields, vendor hashes, fixtures, tests). It is not a HELIOS evidence pack.

---

## What a reviewer must be able to determine

A reviewer who inspects this repository (and a `helix verify` JSON file) must be able to answer:

1. **what Helix claims**
2. **what Helix actually tests**
3. **why each test exists**
4. **which specification release it derives from**
5. **which exact source files and commits were used**
6. **whether a check is normative or Helix-defined**
7. **what the target actually reported**
8. **what Helix selected**
9. **what Helix executed**
10. **what was observed**
11. **why the final result was produced**

Never replace evidence with trust.

Never replace provenance with convention.

Never replace a normative source with an assumption.

Never silently substitute one standard version for another.

Never infer a GA4GH standard version from a URL path such as `/v1`.

Never infer compliance from the existence of an endpoint.

Never call a target “verified” merely because a schema parser succeeded.

Never call a Helix-defined fixture a GA4GH normative requirement.

Never claim GA4GH certification, endorsement, approval, or official status unless explicitly authorized by GA4GH.

If evidence is insufficient, Helix must say so.

If a standard cannot be identified precisely, fail closed.

If a standard release is available but unsupported, fail closed.

If provenance is incomplete, fail closed.

If a verification claim cannot be justified from recorded evidence, do not produce the claim.

**Enforcement** (schema, runtime, tests, and what cannot be a boolean): [ARCHITECTURE_GUARDRAILS.md](ARCHITECTURE_GUARDRAILS.md).

---

## Anti-AI principle

The origin of implementation code is irrelevant to the validity of the result.

Do not defend Helix by saying that code was written by humans rather than AI.

Instead, make every important behavior auditable through:

- source code
- tests
- schemas
- fixtures
- specification provenance
- deterministic execution
- explicit expected behavior
- explicit observed behavior
- recorded run artefacts (what matches after stripping wall-clock `timestamp`: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md); not bit-for-bit JSON files; not HELIOS)

A reviewer should be able to reject a Helix result by demonstrating a concrete technical error, not merely by distrusting its authors or its development process.

Single-steward capacity (see [IDENTITY.md](IDENTITY.md)) is an organisational fact. It is **not** an argument that a result is correct.

---

## Engineering rule

Prefer:

explicit > implicit
pinned > floating
deterministic > convenient
fail-closed > guess
traceable > magical
explainable > opaque
reproducible > environment-dependent
evidence > assertion

Before changing code, inspect the existing architecture and tests.

Do not create compatibility behavior merely to make tests pass.

Do not weaken tests to accommodate an implementation.

Do not introduce fallback behavior unless its semantics are explicitly specified and tested.

When a requirement is ambiguous, document the ambiguity and fail safely rather than inventing semantics.

This principle applies to CLI behavior, the standards registry and provenance, version selection, conformance / behavioral / security checks, benchmarks, fixtures, reports, JSON schemas, error handling, documentation, CI, release automation, and public claims.

---

## Done-check (every Helix change)

> Could an independent skeptical engineer inspect this repository and reproduce the conclusion without trusting the Helix authors?

If the answer is no, identify what evidence is missing and address it. Do not ship the claim.

---

## Where to look (map, not a promise)

These files are the inspectable answers. Empty or explicit “none” is still an answer.

| # | Question | Inspect |
|---|---------|---------|
| 1 | Claims | This file, [README.md](../README.md), [CLAIMS.md](CLAIMS.md), JSON `claims[]`, report `Claims:`: technical signal, not certification. VERIFIED only if predicates hold. Stranger-facing audit: [PUBLIC_READINESS_AUDIT.md](PUBLIC_READINESS_AUDIT.md). Enforcement: [ARCHITECTURE_GUARDRAILS.md](ARCHITECTURE_GUARDRAILS.md) |
| 2 | What is tested | [TEST_IDENTITY.md](TEST_IDENTITY.md), `src/identity.rs`, `src/verify.rs`, [INVENTORY.md](../INVENTORY.md), layers [BEHAVIOR.md](BEHAVIOR.md) |
| 3 | Why a test exists | Catalog names/codes; [DIAGNOSTICS.md](DIAGNOSTICS.md); [FIXTURES.md](FIXTURES.md) for fixture-kind rows; [TRACEABILITY.md](TRACEABILITY.md) `expected_behavior` / `untraceable_reason`; known-bad mutants [MUTATION.md](MUTATION.md) |
| 4 | Which spec release | [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md), `standards/registry.yaml`, JSON `requested_version` / `selected_version` / `verified_version` ([STANDARD_VERSIONING.md](STANDARD_VERSIONING.md)) |
| 5 | Source files and commits | Registry `commit` + `vendor_path` + sha256; HelixTest pin in [VERSIONS.lock](../VERSIONS.lock) |
| 6 | Normative vs Helix-defined | JSON `traceability.category` / `check_kind` / `claim_scope` / `authority` ([TAXONOMY.md](TAXONOMY.md), [TRACEABILITY.md](TRACEABILITY.md), `src/traceability.rs`). Domain `executed[].category` is schema/lifecycle/…, not this taxonomy. Until a pack loads vendor bytes, Helix must **not** label a check as a GA4GH MUST |
| 7 | What the target reported | `detected_version` from 2xx service-info `type.version` only ([DISCOVERY.md](DISCOVERY.md)). Never `/v1` |
| 8 | What Helix selected | `standard_selection.selected_version` (empty when selection failed) |
| 9 | What Helix executed | Versioned join: `pack_integrity_sha256` / `schema_document_sha256` / `schema_component_sha256` / `execution_id` plus `executed[]` / `skipped[]`. `verified_version` is a claim field (empty in B2), not proof that a pack ran |
| 10 | What was observed | `message`, `diagnostic.observed` on fail/error |
| 11 | Why the result | `claims[]` ([CLAIMS.md](CLAIMS.md)), `selection_status`, skip reasons, `summary`, exit code ([CLI_CONTRACT.md](CLI_CONTRACT.md)). Reproduce the run: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md) |

`helix standards list|show|validate` is provenance inspection. It does not run verify and does not download specs.

---

## What is still insufficient (fail closed, do not paper over)

A skeptical engineer can already reject several public sentences. That is intended.

| Gap | Recorded fact | Forbidden claim |
|-----|---------------|-----------------|
| DRS 1.4.0 is **SUPPORTED** for technical verification within declared coverage; DRS 1.5.0 is not | `helix standards list --supported-only` lists `ga4gh.drs.1.4.0` only. Mode 1 for DRS 1.5.0 is `AVAILABLE_BUT_NOT_SUPPORTED`. YAML `support_status` is not sufficient (`src/standards/support.rs`). | “Verified against GA4GH DRS 1.5.0” or “GA4GH certified” |
| No second independent implementation is recorded | `helix matrix` slots `ferrum` and `independent` are **pending**. In-process mocks are not independent evidence ([INTEROP.md](INTEROP.md), [TARGETS.md](TARGETS.md)). `target_kind=mock` never sets independent evidence. | “Helix is validated against multiple implementations” |
| Default `helix verify TARGET` is unversioned | `standard_selection.mode` is `unversioned`; `selected_version` / `verified_version` are empty | Labelling that run as GA4GH DRS 1.4.0 (or any pack) |
| Unversioned OpenAPI is still HelixTest-vendored | Default adapter uses HelixTest `include_str` OpenAPI, not `standards/vendor/…` | “This unversioned run tested the pinned registry bytes” |
| Versioned DRS 1.4.0 join is not a versioned VERIFIED claim | Join hashes may be recorded; `verified_version` stays empty until every claims predicate holds | “Verified against GA4GH DRS 1.4.0” from join or check PASS |
| Checks have `traceability`; one is `normative` | `drs.object.schema.openapi` is Normative / Ga4gh / Schema. Other DRS verify rows stay fixture. SCHEMA PASS is not behavior coverage. | Calling a fixture extra a GA4GH MUST, or quoting schema PASS as DRS compliant |
| WES declared version lists | Discovery copies `type.version` when 2xx JSON exists; it does not yet treat `supported_wes_versions` as selection evidence | Guessing WES 1.1.0 from `/ga4gh/wes/v1` |

HelixTest already runs those DRS/WES checks. Helix productizes the report. Green CI and a green unversioned `helix verify` remain a **technical signal**, not a specification-release claim.

Do not fill remaining gaps with fallback, silent substitution, or “the suite is obviously 1.4.0.” SUPPORTED is not VERIFIED. Record the four version facts separately.
