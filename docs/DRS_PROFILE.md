# DRS verification (HelixTest wrap)

`helix verify <endpoint>` discovers Stage 1 APIs, then runs **DRS** checks when DRS is TESTABLE. WES is a separate suite ([WES.md](WES.md)). TES / TRS / htsget are discovered but not executed.

What a generic DRS must expose (spec vs fixtures vs optional; no Ferrum): [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md).

HelixTest already runs the five DRS checks; this document productizes them behind the HelixTest adapter ([HELIXTEST_ADAPTER.md](HELIXTEST_ADAPTER.md)) and the Helix verification model ([VERIFICATION_MODEL.md](VERIFICATION_MODEL.md)).

HelixTest stays a separate git root (D1). Ferrum is a reference target, not a dependency. HELIOS is out of scope.

Source: `src/verify.rs`. Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md) `drs.object.*` / `HLX-DRS-001`–`005`.

---

## Pipeline

```text
helix verify <endpoint>
        │
        ▼
1. Reachability     TCP to host:port
        │           unreachable → ERROR on DRS and WES rows (not skip, not pass)
        ▼
2. Discovery        Stage 1 probes (DRS, WES, TES, TRS, htsget)
        │           DETECTED is not a pass
        ▼
3. Testability      DRS and WES are TESTABLE when DETECTED
        │           TES/TRS/htsget stay NOT_TESTABLE
        │           NOT_DETECTED → skip unless the profile expects that service (then fail)
        ▼
4. Execute          HelixTestAdapter::run_drs / run_wes (Mode::Generic, pin v0.1.3)
        ▼
5. Translate        TestCaseResult → VerificationResult
        │           stable Helix id/code; original HelixTest name preserved
        ▼
6. Report           human text + VerificationRun JSON + exit code
```

Discovery of TES/TRS/htsget is recorded as `present` / `testable: false`. Those rows are **not** verification passes and **not** TES/TRS/htsget checks.

---

## JSON (`--format json`)

Helix `VerificationRun` (not HelixTest `OverallReport`). Field order is struct order; `executed` / `skipped` are sorted by `code` then `id`. Identical runs differ only in `timestamp`. Run-level `profile` is `generic` or `ferrum` ([PROFILES.md](PROFILES.md)). `fixture_version` is `helix-fixtures-v1` ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Each result has `service`. Per-check `profile: "generic"` is HelixTest Mode::Generic.

| Field | Meaning |
|-------|---------|
| `target.url` | Origin that was tested |
| `helix_version` | Helix crate version |
| `helixtest_version` | HelixTest tag (`v0.1.3`) |
| `helixtest_sha` | HelixTest git SHA from [VERSIONS.lock](../VERSIONS.lock) |
| `fixture_version` | Catalog id `helix-fixtures-v1` ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not HELIOS |
| `discovery[]` | Per-service `present` (DETECTED) and `testable` (TESTABLE). **Not a pass.** |
| `executed[]` | Checks that ran: `pass` / `fail` / `error`, with `id`, `code`, `failure` |
| `skipped[]` | Checks not run (`skip`). Skip is never pass |
| `summary` | Counts only (not a score, not certification) |

Each executed/skipped row has stable `id` (e.g. `drs.object.not_found`) and `code` (`HLX-DRS-005`). Fail/error rows include `failure.code` (same catalog code) and `message` / `failure.detail` for why. Catalogued DRS fail/error rows also attach a deterministic `diagnostic` (expected / observed / **possible causes** — not a root cause). Pass and skip do not. Details: [DIAGNOSTICS.md](DIAGNOSTICS.md).

No `passed` boolean. No signatures, RO-Crate, PDF, or `overall_score`.

---

## Human output

Structured report ([REPORT.md](REPORT.md)). Same facts as JSON `VerificationRun`. Never `found` as if verified.

```text
HELIX VERIFICATION
…
Services:
  DRS      DETECTED     TESTABLE  …
Results:

DRS
  PASS   drs.object.reachable  HLX-DRS-001  DRS object endpoint is reachable
  FAIL   drs.object.schema     HLX-DRS-002  … — <reason>
           expected: …
           observed: …
           category: …
           hint: …
           possible causes:
             - …
  SKIP   … — <reason>
  ERROR  … — target unreachable

Summary:
  N PASS
  N FAIL
  N ERROR
  N SKIP

Changes:
  Not compared. …
```

Skip is never green. Diagnostic lines appear only on fail/error for catalogued ids. DETECTED is not a pass.

---

## Exit codes

| Code | When |
|------|------|
| 0 | Overall status is **pass** (at least one check passed, no fail/error) |
| 1 | Fail, error, skip-only (missing DRS and WES), unreachable target, or usage/runtime error |

Skip-only is not a pass: pointing `helix verify` at a live HTTP server with no DRS and no WES exits 1.

A DRS-only target can still exit 0: five DRS `pass` plus eight WES `skip`.

---

## Outcomes (tests)

| Target | Discovery DRS | Checks | Exit |
|--------|---------------|--------|------|
| Valid DRS (in-process generic fixture) | present, testable | five DRS `pass`; WES skipped if absent | 0 |
| Invalid DRS body | present, testable | at least one `fail` + failure code | 1 |
| Missing DRS (HTTP up, no DRS) | present=false | five DRS `skip` | 1 if WES also absent/fails |
| Unavailable (TCP fail) | all not present | DRS and WES `error` | 1 |

Catalog: [FIXTURES.md](FIXTURES.md). DETECTED + TESTABLE on an invalid object is **not** a verification pass.

---

## Out of this suite

- TES / TRS / htsget check execution ([WES.md](WES.md) covers WES)
- HELIOS evidence
- Ferrum as a crate
- GA4GH certification claims
