# Interpretation

Same facts as JSON `VerificationRun`. Human text starts at `HELIX VERIFICATION` on **stdout**. Not HELIOS. Not GA4GH certification.

## Discovery vs checks

| Word | Means | Does not mean |
|------|--------|----------------|
| NOT_DETECTED | No probe returned 2xx/401/403 | The service is “failed” |
| DETECTED | A probe got 2xx/401/403 | The checks passed |
| TESTABLE | Helix will execute DRS/WES checks | Those checks passed |
| NOT_TESTABLE | Helix does not execute that suite yet (TES/TRS/htsget) | A fail |

JSON: `discovery[].present` = DETECTED, `discovery[].testable` = TESTABLE. There is no `services` array and no `passed` boolean (`helix verify` is not HelixTest `OverallReport`).

## Check status

| Text | JSON | Meaning |
|------|------|---------|
| PASS | `pass` | Assertion held |
| FAIL | `fail` | Target behaved wrongly |
| SKIP | `skip` | Not executed. **Never** a pass |
| ERROR | `error` | Helix could not run the check (e.g. unreachable) |

Skip-only (no DRS, no WES) → exit **1**. DRS-only with five PASS and WES SKIP → exit **0** is allowed.

## Fixture run (`make verify-fixture`)

Example: [example-verify.json](example-verify.json).

- DRS DETECTED + TESTABLE; five `HLX-DRS-001`–`005` **pass**.
- WES NOT_DETECTED; eight WES rows **skip** (`WES not detected; … (not a pass)`).
- TES/TRS/htsget NOT_DETECTED, not executed.
- `summary` counts are not a score.
- `Changes` / compare is **Not compared** on a single run (`helix compare` is a different command).
- `fixture_version` is `helix-fixtures-v1` (catalog identity so two JSON files can be paired). Not a signature. Not HELIOS.

## Fail/error rows

May include `failure.code` (catalog code, e.g. `HLX-DRS-005`) and `diagnostic` (`expected`, `observed`, `possible_causes`). That is not a root-cause claim and not an AI diagnosis.

## What success is not

Not GA4GH certification. Not a Ferrum or clinical-pilot claim. Not HELIOS evidence (no signature, RO-Crate, PDF).
