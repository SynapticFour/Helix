# Helix verification report

Human text for `helix verify` and `helix compare` is a **projection of the same JSON** (`VerificationRun` / `CompareReport`). The terminal does not add facts that JSON lacks, and JSON does not encode a second verdict.

HelixTest already runs the DRS and WES checks. This document productizes how Helix **presents** a run. It is not a new suite, not HELIOS, and not GA4GH certification.

Source: `src/report.rs` (`format_verify_text`, `format_compare_text`). Model: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md). Diagnostics on fail/error: [DIAGNOSTICS.md](DIAGNOSTICS.md). Compare kinds: [REGRESSION.md](REGRESSION.md). CLI: [CLI_CONTRACT.md](CLI_CONTRACT.md).

---

## What the report answers

| Question | Text section | JSON |
|----------|--------------|------|
| WHAT was tested? | `What:` | implied by executed/skipped `service` + run `profile` |
| AGAINST WHAT? | `Target:` | `target.url` |
| WITH WHICH HELIX VERSION? | `Helix:` | `helix_version` + `schema_version` + `fixture_version` |
| WHICH TEST SUITE VERSION? | `Test suite:` | `helixtest_version` + `helixtest_sha` |
| WHICH SERVICES were detected? | `Services:` | `discovery[]` `present` / `testable` |
| WHICH TESTS ran? | `Results:` executed rows | `executed[]` |
| WHICH passed / failed / skipped? | `PASS` / `FAIL` / `SKIP` / `ERROR` | `status` |
| WHY? | `— {message}` and diagnostic block | `message`, `failure`, `diagnostic` |
| WHAT changed since the previous run? | `Changes:` | verify JSON has no previous; `helix compare` is `CompareReport` |

A single `helix verify` cannot know a previous run. Its `Changes:` section says **Not compared** and points at `helix compare`. That is the honest answer, not a hidden delta.

---

## `helix verify` (text)

```text
HELIX VERIFICATION

This is a technical verification signal.
It is not GA4GH certification.

What:
  DRS and WES checks (HelixTest wrap). TES/TRS/htsget discovered only, not executed.

Target:
  http://127.0.0.1:8080

Helix:
  0.1.0
  schema helix-verification-v1
  profile generic
  fixtures helix-fixtures-v1
  2026-09-04T09:00:00Z

Test suite:
  HelixTest v0.1.3 (1832c043e1679ec283cb2113510ee33684317cce)

Services:
  DRS      DETECTED     TESTABLE  http://127.0.0.1:8080/ga4gh/drs/v1
  WES      DETECTED     TESTABLE  http://127.0.0.1:8080/ga4gh/wes/v1
  TES      DETECTED     NOT_TESTABLE  …

Results:

DRS
  PASS  drs.object.reachable  HLX-DRS-001  DRS object endpoint is reachable
  FAIL  drs.object.not_found  HLX-DRS-005  Unknown DRS object returns 404 — expected 404, got 200
        expected: HTTP 404 for an unknown object ID
        observed: HTTP 200
        category: error_handling
        hint: …
        possible causes:
          - …

WES
  SKIP  wes.run.scatter_gather  HLX-WES-008  … — supports_scatter_gather=false in features

Summary:
  18 PASS
  1 FAIL
  0 ERROR
  3 SKIP

Changes:
  Not compared. This report is a single run.
  What changed: helix compare <previous.json> <current.json>

Discovery is not conformance. DETECTED is not a pass. Skip is never pass.
```

Word mapping (frozen):

| JSON | Text |
|------|------|
| `discovery.present: true` | `DETECTED` |
| `discovery.present: false` | `NOT_DETECTED` |
| `discovery.testable: true` | `TESTABLE` |
| `discovery.testable: false` (and present) | `NOT_TESTABLE` |
| `status: pass` | `PASS` |
| `status: fail` | `FAIL` |
| `status: skip` | `SKIP` |
| `status: error` | `ERROR` |
| `summary.passed` | `N PASS` |
| `summary.failed` | `N FAIL` |
| `summary.errors` | `N ERROR` |
| `summary.skipped` | `N SKIP` |

`TESTABLE` is not a pass. Skip is never painted as PASS. Fail/error may include [diagnostics](DIAGNOSTICS.md) (**possible causes**, never `Cause:`).

`--format json` prints `VerificationRun` only. It does not duplicate this heading layout, ANSI, or `PASS` marks.

---

## `helix compare` (text)

Answers **WHAT changed** against two `VerificationRun` files.

```text
HELIX VERIFICATION COMPARE
…
Previous:
  …
Current:
  …

Identity:
  same measurement: yes|no
  suite changed: yes|no
  mismatches: none | field : previous -> current
  Not a signed audit trail. Not HELIOS.

Changes:
  NEW_FAIL  drs.object.reachable  HLX-DRS-001  pass → fail

Unchanged:
  UNCHANGED_FAIL  drs.object.not_found  HLX-DRS-005  fail → fail

Summary:
  1 NEW_FAIL
  …

Result: REGRESSION | NO_NEW_REGRESSION
```

Kinds stay [REGRESSION.md](REGRESSION.md). Identity is [RUN_IDENTITY.md](RUN_IDENTITY.md): recorded so two files can be paired; mismatch is not `NEW_FAIL`. A diagnostic hint change is not a row here.

---

## Out of this report

- PDF, RO-Crate, signatures, audit trails (HELIOS)
- Scores, certification marks, “Helix-certified”
- Inventing a previous run
- Treating DETECTED or SKIP as PASS
