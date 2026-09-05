# Helix verification model

Helix-native domain types for a verification run. `helix verify --format json` emits this shape (DRS + WES when TESTABLE; [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md)). Profiles (`generic` / `ferrum`) are policy, not a second engine ([PROFILES.md](PROFILES.md)). `helix compare` diffs two of these runs at stable `id` ([REGRESSION.md](REGRESSION.md)); a regression is PASS→FAIL, not a score drop. `helix security` still emits HelixTest `OverallReport` ([CLI_CONTRACT.md](CLI_CONTRACT.md), [DECISIONS.md](DECISIONS.md) D3 revisit). HelixTest execution sits behind the adapter ([HELIXTEST_ADAPTER.md](HELIXTEST_ADAPTER.md) / [ARCHITECTURE.md](ARCHITECTURE.md) §5).

HelixTest already runs; this does not rewrite it. HelixTest stays a separate git root (D1). Ferrum is a reference target, not a field in this model. HELIOS is out of scope.

Source: `src/model.rs`. Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md) / `src/identity.rs`.

---

## Why this exists

HelixTest `TestCaseResult` uses `status` plus a `passed` boolean (`true` iff Pass). That is easy to misuse. Helix’s model has **no `passed` field**. Status is `pass` | `fail` | `skip` | `error`. Skip cannot be stored or serialized as pass.

Every check has a **stable machine-readable identity**:

```text
id:   drs.object.not_found
code: HLX-DRS-005
```

`id` is dotted and service-scoped. `code` is the Helix catalog token (`HLX-<SVC>-NNN`). Titles may match HelixTest human names; identities are Helix’s contract.

---

## Types

| Type | Role |
|------|------|
| `Target` | Origin URL Helix was pointed at |
| `DiscoveredService` | Whether a named service was present (`present` = DETECTED, `testable` = TESTABLE). Neither is a pass |
| `CheckIdentity` | `id` + `code` |
| `VerificationCheck` | Definition (identity, name, service, category, severity, optional profile) |
| `VerificationStatus` | `pass` / `fail` / `skip` / `error` |
| `Severity` | `info` / `warn` / `error` — not a score |
| `FailureCode` | Why a fail/error happened (`code`, optional `detail`) |
| `FailureDiagnostic` | Optional structured explanation on fail/error for catalogued DRS/WES ids ([DIAGNOSTICS.md](DIAGNOSTICS.md)). Possible causes, not a cause. Absent on pass/skip |
| `StandardSelection` | How Helix chose or refused a registry pack. `selected_version` is not a target declaration |
| `VerificationResult` | One executed or skipped check. Always records the seven standard-version fields (null when empty) |

Translated rows may include `helixtest_name`: the original HelixTest `TestCaseResult.name`. Helix `id` / `code` / `name` still come from the catalog. The adapter never converts SKIP into PASS and does not read HelixTest `passed`.
| `VerificationSummary` | Counts: passed, failed, skipped, errors, total |
| `VerificationRun` | Whole run |

`model::DiscoveredService` is not `discover::DiscoveredService`. Discovery probes use a Stage 1 enum (`DRS` … `htsget`). The run model uses an open string so a later `beacon` (or a profile-specific name) does not change JSON shape.

---

## `VerificationRun` fields

| Field | Content |
|-------|---------|
| `schema_version` | Frozen document id `helix-verification-v1` ([SCHEMA.md](SCHEMA.md)) |
| `target.url` | Gateway-style origin |
| `helix_version` | Helix crate version (`0.1.0` today) |
| `helixtest_version` | HelixTest **tag** (`v0.1.3` from [VERSIONS.lock](../VERSIONS.lock)) |
| `helixtest_sha` | HelixTest git SHA from the lockfile |
| `profile` | Helix profile id: `generic` (default) or `ferrum`. Not HelixTest Mode. Not inferred from the target. Each result has `service` |
| `fixture_version` | Fixture catalog id `helix-fixtures-v1` ([FIXTURES.md](FIXTURES.md), [RUN_IDENTITY.md](RUN_IDENTITY.md)). Compare identity, not HELIOS |
| `standard_selection` | Pack selection for this run ([STANDARD_VERSIONING.md](STANDARD_VERSIONING.md)) |
| `timestamp` | RFC3339 UTC (wall clock). Identical inputs differ only here |
| `discovery` | Present / missing services (discovery is not a pass) |
| `executed` | Results with status pass, fail, or error |
| `skipped` | Results with status skip |
| `summary` | Counts derived from both lists |

No signatures, RO-Crate, evidence chains, audit trails, PDF, `overall_score`, or `overall_level`.

---

## Status semantics

| Status | Meaning | Counts as pass? | Blocks CI (`has_failures`)? |
|--------|---------|-----------------|------------------------------|
| `pass` | Assertion held | yes | no |
| `fail` | Target behaved wrong | no | yes |
| `skip` | Not executed (unwired, no fixture, not discovered when skip is the rule) | **never** | no |
| `error` | Helix could not run the check (transport, adapter panic, timeout) | no | yes |

`push_skipped` forces `status = skip` even if a `pass` result is passed in. `overall_status()` of an all-skip or empty run is `skip`, never `pass`.

`fail` vs `error`: fail is a negative assertion about the target; error is a runner problem. Both are blocking. They must not be collapsed.

---

## Catalog (DRS + WES in `helix verify`)

Identities, codes, Helix names, and HelixTest name mapping: [TEST_IDENTITY.md](TEST_IDENTITY.md). Example: `drs.object.not_found` / `HLX-DRS-005` wraps HelixTest `"DRS invalid object id returns 404"`. WES: `wes.service_info.reachable` / `HLX-WES-001` wraps `"WES service-info reachable"`. Scatter/gather (`HLX-WES-008`) is skipped on profile `generic` (`supports_scatter_gather=false`) and executed on profile `ferrum` ([PROFILES.md](PROFILES.md), [WES.md](WES.md)). Changing an assigned id or code is a compatibility change.

TES / TRS / htsget catalog rows exist; `helix verify` does not execute them. DETECTED + NOT_TESTABLE for those services is not a pass.

`model::catalog` is a thin lookup into `src/identity.rs`. Do not duplicate codes here.

---

## JSON example (illustrative)

```json
{
  "schema_version": "helix-verification-v1",
  "helix_version": "0.1.0",
  "helixtest_version": "v0.1.3",
  "helixtest_sha": "<executed DRS checker source sha256; not HELIXTEST_SHA>",
  "profile": "generic",
  "fixture_version": "helix-fixtures-v1",
  "timestamp": "2026-09-04T09:00:00Z",
  "target": { "url": "http://127.0.0.1:8080" },
  "discovery": [
    { "service": "drs", "present": true, "testable": true, "base_url": "http://127.0.0.1:8080/ga4gh/drs/v1" },
    { "service": "wes", "present": true, "testable": true, "base_url": "http://127.0.0.1:8080/ga4gh/wes/v1" },
    { "service": "tes", "present": true, "testable": false, "not_testable_reason": "Helix Stage 1 does not execute TES checks; DETECTED is not a pass" }
  ],
  "executed": [
    {
      "id": "drs.object.not_found",
      "code": "HLX-DRS-005",
      "name": "Unknown DRS object returns 404",
      "helixtest_name": "DRS invalid object id returns 404",
      "service": "drs",
      "category": "robustness",
      "profile": "generic",
      "status": "fail",
      "severity": "error",
      "message": "expected 404, got 200",
      "failure": { "code": "HLX-DRS-005", "detail": "expected 404, got 200" },
      "diagnostic": {
        "code": "HLX-DRS-005",
        "id": "drs.object.not_found",
        "expected": "HTTP 404 for an unknown object ID",
        "observed": "HTTP 200",
        "likely_category": "error_handling",
        "hint": "The target appears to return a successful response for an unknown DRS object. Verify object lookup error handling.",
        "possible_causes": [
          "Unknown ids are treated as existing objects.",
          "A catch-all handler returns 200 or another success status.",
          "Auth or a gateway maps missing objects to 401/403/500 instead of 404."
        ]
      }
    }
  ],
  "skipped": [],
  "summary": { "passed": 0, "failed": 1, "skipped": 0, "errors": 0, "total": 1 }
}
```

This **is** what `helix verify --format json` prints. `present` / `testable` on discovery are not passes. Skip cannot be stored or serialized as pass. WES scatter/gather appears in `skipped` on profile `generic` and in `executed` on profile `ferrum`. Run-level `profile` is the Helix profile id; per-check `profile: "generic"` is HelixTest Mode::Generic. `diagnostic` is optional and omitted on pass/skip; it is not a new check and does not change status or exit codes ([DIAGNOSTICS.md](DIAGNOSTICS.md)). `traceability` is always emitted by producers: `category` / `check_kind` is not `normative` in the shipped catalog; `claim_scope` is never `ga4gh_requirement` ([TAXONOMY.md](TAXONOMY.md), [TRACEABILITY.md](TRACEABILITY.md)). `layer` / `layer_summary` classify SCHEMA vs BEHAVIOR vs SECURITY vs INTEROPERABILITY independently; SCHEMA PASS is not BEHAVIOR PASS; there is no compliance percentage ([BEHAVIOR.md](BEHAVIOR.md)). `claims[]` is always emitted by producers and is the only source of VERIFIED / NOT_VERIFIED sentences; an honest DRS PASS is still `not_verified` ([CLAIMS.md](CLAIMS.md)).

---

## Out of this model

- HELIOS: signatures, RO-Crate, evidence chains, audit trails, PDF
- Compliance scoring, ISO 15189 / AI Act, certification marks
- Ferrum types or a required Ferrum URL
- Issuing credentials
- Helix as a server

---

## CLI

`helix verify --format json` emits `VerificationRun` as above ([DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md), [CLI_CONTRACT.md](CLI_CONTRACT.md)). `--format text` is the same facts as a `HELIX VERIFICATION` document ([REPORT.md](REPORT.md)). `helix security` / `helix bench` argv, JSON, and exit codes stay as in that contract. Do not silently replace security `OverallReport`.
