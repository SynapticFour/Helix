# Helix verification report

Human text for `helix verify` and `helix compare` is a **projection of the same JSON** (`VerificationRun` / `CompareReport`). The terminal does not add facts that JSON lacks, and JSON does not encode a second verdict.

HelixTest already runs the DRS and WES checks. This document productizes how Helix **presents** a run. It is not a new suite, not HELIOS, and not GA4GH certification.

Source: `src/report.rs` (`format_verify_text`, `format_compare_text`). Claims: `src/claims.rs` ([CLAIMS.md](CLAIMS.md)). Model: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md). Diagnostics on fail/error: [DIAGNOSTICS.md](DIAGNOSTICS.md). Compare kinds: [REGRESSION.md](REGRESSION.md). CLI: [CLI_CONTRACT.md](CLI_CONTRACT.md).

---

## What the report answers

| Question | Text section | JSON |
|----------|--------------|------|
| WHAT was tested? | `What:` | implied by executed/skipped `service` + run `profile` |
| AGAINST WHAT? | `Target:` | `target.url` |
| WITH WHICH HELIX VERSION? | `Helix:` | `helix_version` + `schema_version` + `fixture_version` |
| WHICH TEST SUITE VERSION? | `Test suite:` | `helixtest_version` + `helixtest_sha` |
| WHICH STANDARD VERSION? | `Standards:` | `standard_selection` + per-check version fields |
| WHICH SERVICES were detected? | `Services:` | `discovery[]` `present` / `testable` |
| WHICH TESTS ran? | `Results:` executed rows | `executed[]` |
| WHICH passed / failed / skipped? | `PASS` / `FAIL` / `SKIP` / `ERROR` | `status` |
| WHY is this VERIFIED / NOT VERIFIED? | `Claims:` | `claims[]` ([CLAIMS.md](CLAIMS.md)). Not a PASS/FAIL grep |
| WHY this check? | `— {message}` and diagnostic block; `kind:` / `claim_scope:` / `authority:` | `message`, `failure`, `diagnostic`, `traceability` |
| WHAT kinds of evidence? | `Evidence (classification, not a score):` | `traceability.category` / `claim_scope` on each row ([TAXONOMY.md](TAXONOMY.md)) |
| SCHEMA vs BEHAVIOR vs SECURITY vs INTEROPERABILITY | `Layers:` SCHEMA PASS / BEHAVIOR FAIL / … | `layer` / `layer_summary` ([BEHAVIOR.md](BEHAVIOR.md)). SCHEMA PASS is not BEHAVIOR PASS |
| WHAT changed since the previous run? | `Changes:` | verify JSON has no previous; `helix compare` is `CompareReport` |

A single `helix verify` cannot know a previous run. Its `Changes:` section says **Not compared** and points at `helix compare`. That is the honest answer, not a hidden delta.

Target-controlled text in `message` / `diagnostic.observed` is sanitized before print (`src/sanitize.rs`): no ANSI, no extra newlines, length-capped. Helix colour is only Helix’s own PASS/FAIL/SKIP marks. Not a security scanner.

---

## `helix verify` (text)

```text
HELIX VERIFICATION

This is a technical verification signal.
It is not GA4GH certification.

Claims (predicates; not GA4GH certification):
  No VERIFIED claim is justified by this run.

  ga4gh_requirement  NOT_VERIFIED
    Why not verified:
      - unversioned_run
          field: standard_selection.mode
          observed: unversioned
          expected: explicit or automatic with SELECTED
      - no_normative_checks
          field: traceability.category
          observed: no BindingKind::Normative rows
          expected: at least one executed normative check

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
  HelixTest tag v0.1.3
  git checkout pin: (VERSIONS.lock HELIXTEST_SHA)
  executed checker: helixtest-drs:(HELIXTEST_CHECKER_SOURCE_SHA256)

Standards:
  mode: unversioned
  selection_status: UNVERSIONED
  requested_version: (none)
  selected_version: (none)
  verified_version: (none)
  helix verify did not select a GA4GH registry pack. Detected service-info versions are recorded, not selected.

Services:
  DRS      DETECTED     TESTABLE  http://127.0.0.1:8080/ga4gh/drs/v1
  WES      DETECTED     TESTABLE  http://127.0.0.1:8080/ga4gh/wes/v1
  TES      DETECTED     NOT_TESTABLE  …

Results:

DRS
  PASS  drs.object.reachable  HLX-DRS-001  DRS object endpoint is reachable
        kind: fixture  claim_scope: helix_fixture  authority: helixtest
        not a GA4GH MUST  (PASS is not a conformance claim)
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

Evidence (classification, not a score):
  0 normative
  0 guidance
  5 fixture
  …
  No check in this run is a GA4GH MUST.

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

`TESTABLE` is not a pass. Skip is never painted as PASS. Fail/error may include [diagnostics](DIAGNOSTICS.md) (**possible causes**, never `Cause:`). Diagnostic `category:` is a likely-failure class (`error_handling`, …), not the claim taxonomy. Taxonomy is `kind:` / `claim_scope:` on every row.

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
