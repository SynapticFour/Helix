# Helix verification regression

A **regression** is a previously **passing** verification check that now **fails**, compared at a **stable Helix `id`**.

It is **not** “overall score decreased”, **not** `summary.passed` going down, **not** certification, and **not** HELIOS evidence. A regression is not a score.

HelixTest already runs the checks. `helix compare` productizes two `helix verify --format json` files (`VerificationRun`). Check identities: [TEST_IDENTITY.md](TEST_IDENTITY.md) (example `drs.object.not_found` / `HLX-DRS-005`). Run identity (versions, target, fixtures): [RUN_IDENTITY.md](RUN_IDENTITY.md). Identity mismatch is not a regression.

A **bench** warning or bench regression ([BENCHMARKS.md](BENCHMARKS.md)) is a different concept. It is not `NEW_FAIL` and must not be treated as a verification failure.

A **diagnostic** on a fail/error row ([DIAGNOSTICS.md](DIAGNOSTICS.md)) is explanatory. Changing `hint` or `possible_causes` is not a regression. `helix compare` keys on `id` + status, not on diagnostic text.

Source: `src/compare.rs`. CLI: `helix compare <previous.json> <current.json>`.

---

## Definition

```text
Previous:  drs.object.not_found = PASS
Current:   drs.object.not_found = FAIL
=> REGRESSION (NEW_FAIL)
```

```text
Previous:  drs.object.not_found = FAIL
Current:   drs.object.not_found = FAIL
=> EXISTING_FAILURE (UNCHANGED_FAIL), not a new regression
```

Key is Helix `id` (dotted), not row order, not `summary.passed`, not discovery `present`.

`fail` and `error` are both **blocking** (not pass). `PASS → ERROR` is `NEW_FAIL`. `FAIL → ERROR` is still `UNCHANGED_FAIL`.

---

## Kinds (per id)

Tracked kinds:

| Kind | Previous | Current | Regression? |
|------|----------|---------|-------------|
| **NEW_FAIL** | pass | fail or error | **yes** |
| **FIXED** | fail or error | pass | no |
| **UNCHANGED_FAIL** | fail or error | fail or error | no (existing failure) |
| **UNCHANGED_PASS** | pass | pass | no |
| **NEW_SKIP** | pass, fail, or error | skip or absent | no |
| **FIXED_SKIP** | skip | pass, fail, or error | no |

Supporting kinds so every id is classified (not silent):

| Kind | Meaning |
|------|---------|
| **UNCHANGED_SKIP** | skip → skip (or skip → absent) |
| **ADDED** | id only in current. A new fail is **not** `NEW_FAIL` (it never passed) |

### SKIP must not silently become PASS

```text
Previous: skip
Current:  pass
=> FIXED_SKIP (skip_became_pass: true)
```

Never `UNCHANGED_PASS`. Text prints `(SKIP must not silently become PASS)`. JSON `skip_became_pass` is true on that row and counted on `summary.skip_became_pass`. Skip is never pass ([VERIFICATION_MODEL.md](VERIFICATION_MODEL.md)).

Skip → fail is also `FIXED_SKIP`, not `NEW_FAIL` (it was not previously passing).

---

## What is not a regression

- `summary.passed` decreased because a pass became **skip** (`NEW_SKIP`)
- A check that was already failing stays failing (`UNCHANGED_FAIL`)
- A check that never existed before fails on first run (`ADDED`)
- Discovery present/testable changing
- Profile string changing (`generic` / `ferrum`)
- Timestamp / helix_version / profile / target / `fixture_version` (those are **run identity**; mismatch is recorded, not `NEW_FAIL`) ([RUN_IDENTITY.md](RUN_IDENTITY.md))

---

## CLI

```text
helix compare <previous.json> <current.json>
helix compare <previous.json> <current.json> --format text
helix compare <previous.json> <current.json> --format json
```

`--report` is an alias of `--format`. Files must be Helix `VerificationRun` (`helix verify --format json`), not HelixTest `OverallReport` (`helix security`). Text layout: [REPORT.md](REPORT.md) (`HELIX VERIFICATION COMPARE`). JSON is the same kinds and counts.

### Exit codes

Print the report first, then exit.

| Code | When |
|------|------|
| **0** | No `NEW_FAIL`. Existing failures, fixes, skips, and SKIP→PASS do **not** set 1. |
| **1** | At least one `NEW_FAIL`, or the files could not be read/parsed as `VerificationRun`. |
| **2** | Clap usage (missing paths, unknown `--format`). |

`--help` exits 0.

### JSON

One `CompareReport` on stdout. Pretty-printed. No ANSI. No `overall_score`. No HELIOS fields.

`kind` strings: `NEW_FAIL`, `FIXED`, `UNCHANGED_FAIL`, `UNCHANGED_PASS`, `NEW_SKIP`, `FIXED_SKIP`, `UNCHANGED_SKIP`, `ADDED`.

`has_regression` is true iff `summary.new_fail > 0`.

`previous_identity` / `current_identity` / `identity_mismatches` / `same_measurement` / `suite_changed` describe whether the two files are the same kind of measurement ([RUN_IDENTITY.md](RUN_IDENTITY.md)). They do not set `has_regression`. No `signature`, `ro_crate`, or PDF.

Rows sorted by `id`.

---

## Out of this model

- Scoring, ISO 15189 / AI Act, certification
- HELIOS RO-Crate / signatures / PDF
- Comparing `helix security` OverallReport
- Ferrum as a required target
