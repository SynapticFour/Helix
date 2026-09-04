# Helix diagnostics

HelixTest already runs DRS and WES checks. Helix productizes a **deterministic diagnostic layer** on verification **fail** and **error** rows. It is not an AI diagnosis system. It does not produce HELIOS evidence. It does not certify the target.

When Helix cannot parse a more specific observation, it says so. It never invents an HTTP status or WES state.

Source: `src/diagnostics.rs`. Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md). Model: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md).

---

## What a diagnostic is

For catalogued DRS and WES failures, Helix may attach `diagnostic` on the `VerificationResult`:

| Field | Meaning |
|-------|---------|
| `code` | Catalog failure code (`HLX-DRS-005`) |
| `id` | Stable test id (`drs.object.not_found`) |
| `expected` | Documented assertion for that check |
| `observed` | What Helix could extract from the failure text, or **not determined** |
| `likely_category` | A coarse class (`error_handling`, `schema`, …). Not a root cause |
| `hint` | A short, check-specific note for a human |
| `possible_causes` | A list of **possibilities**. Never a field named `cause` |

Pass and skip have no diagnostic. TES / TRS / htsget / auth / bench ids are **not** catalogued here yet.

---

## Catalog (DRS + WES)

Expected strings and categories are fixed in `src/diagnostics.rs`. They describe the **check**, not a guessed defect.

| Code | Id | Category | Expected |
|------|----|----------|----------|
| `HLX-DRS-001` | `drs.object.reachable` | `reachability` | HTTP 2xx or 401 on GET `/objects/test-object-1` |
| `HLX-DRS-002` | `drs.object.schema` | `schema` | DrsObject JSON with id, self_uri, name, non-empty `access_methods` |
| `HLX-DRS-003` | `drs.object.checksum` | `checksum` | sha256 matches download of `access_methods[0]` |
| `HLX-DRS-004` | `drs.object.range` | `range` | HTTP 206 with a valid Content-Range for `bytes=0-1023` |
| `HLX-DRS-005` | `drs.object.not_found` | `error_handling` | HTTP 404 for an unknown object ID |
| `HLX-WES-001` | `wes.service_info.reachable` | `reachability` | HTTP 2xx or 401 on GET WES `/service-info` |
| `HLX-WES-002` | `wes.service_info.schema` | `schema` | ServiceInfo JSON with `supported_wes_versions` containing 1.0 or 1.1 |
| `HLX-WES-003` | `wes.run.lifecycle_success` | `lifecycle` | Echo workflow reaches COMPLETE with documented outputs |
| `HLX-WES-004` | `wes.run.failure_state` | `lifecycle` | Bad workflow ends in `EXECUTOR_ERROR` or `SYSTEM_ERROR` |
| `HLX-WES-005` | `wes.run.missing_inputs` | `lifecycle` | Empty params end in `EXECUTOR_ERROR` or `SYSTEM_ERROR` |
| `HLX-WES-006` | `wes.run.incompatible_type` | `lifecycle` | CWL posted as WDL ends in `EXECUTOR_ERROR` or `SYSTEM_ERROR` |
| `HLX-WES-007` | `wes.run.invalid_workflow` | `lifecycle` | Invalid TRS URL ends in `EXECUTOR_ERROR` or `SYSTEM_ERROR` |
| `HLX-WES-008` | `wes.run.scatter_gather` | `lifecycle` | Scatter/gather reaches COMPLETE with `outputs.scatter_result` |

HelixTest already runs these checks. Helix productizes the diagnostic layer on top; it does not invent a second suite.

---

## Example (`HLX-DRS-005`)

```text
HLX-DRS-005
drs.object.not_found

Expected:
HTTP 404 for an unknown object ID

Observed:
HTTP 200

likely category: error_handling

hint:
The target appears to return a successful response for an unknown
DRS object. Verify object lookup error handling.

Possible causes:
- Unknown ids are treated as existing objects.
- A catch-all handler returns 200 or another success status.
- Auth or a gateway maps missing objects to 401/403/500 instead of 404.
```

The 2xx-specific hint is used **only** when Helix parsed an HTTP 2xx from the failure text. If the text is opaque, observed is not determined and Helix does **not** claim the target returned 200.

---

## How observed is filled

Deterministic parsers on the HelixTest / Helix failure string (pin v0.1.3 phrasing):

| Pattern | Observed |
|---------|----------|
| `got 200` / `got 200 OK` / `Unexpected HTTP status: 503 …` | `HTTP 200` / `HTTP 503` |
| `got COMPLETE` (WES) | `WES state COMPLETE` |
| `target unreachable` | Target was not reachable; checks not executed |
| `not detected` / `not TESTABLE` | Discovery/profile outcome |
| Anything else | `Not determined. Helix cannot extract a more specific observation from the failure text.` plus the check output |

Helix does not inspect live response bodies to invent a better observed line than that text.

---

## Measurement vs diagnosis

| Helix does | Helix does not |
|------------|----------------|
| Repeat the catalog expected behaviour | Name the root cause |
| Parse a status/state when the text contains it | Guess when the text does not |
| Offer **possible causes** | Emit `Cause:` |
| Attach diagnostics on fail/error for DRS `HLX-DRS-001`–`005` and WES `HLX-WES-001`–`008` | Diagnose TES/TRS/htsget, security, or bench |
| Keep `analysis.verification_failure` / bench warnings separate | Turn a bench warning into a verify diagnostic |

A diagnostic is not a new check. It does not change pass/fail. It does not change exit codes.

---

## JSON

Additive field on a fail/error row:

```json
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
```

Omitted on pass/skip (`skip_serializing_if`).

---

## Out of scope

- LLM / “AI SRE” root-cause generation
- Stack traces from Ferrum internals
- Claiming the implementer’s intent
- HELIOS signed evidence of the failure
- Changing HelixTest error strings (those stay in HelixTest)

Green CI with diagnostics present is still only a technical signal, not certification.
