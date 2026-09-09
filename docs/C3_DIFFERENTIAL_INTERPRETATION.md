# C3 — Independent DRS differential interpretation

Helix is HelixTest becoming a standalone VERIFY CLI. This report records C3 of Phase C. It is not a coverage expansion. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
authorization = UNEVALUATED
```

Operator path: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md). C1: [C1_CANONICAL_TWO_TARGET_WORKFLOW.md](C1_CANONICAL_TWO_TARGET_WORKFLOW.md). C2: [C2_CURRENT_LIVE_EVIDENCE.md](C2_CURRENT_LIVE_EVIDENCE.md). Differential contract: [DIFFERENTIAL.md](DIFFERENTIAL.md).

---

## Objective

Make two existing `helix-verification-v1` artifacts understandable as a comparison:

- same Helix verification contract (`execution_id`)
- distinct target executions
- check-level PASS / FAIL / SKIP (or ABSENT)
- existing failure attribution
- `ga4gh_requirement` VERIFIED vs NOT_VERIFIED
- current vs historical standing

without ranking implementations, mutating inputs, or creating a third verification result.

Helix does not rank implementations.

## Existing differential capability

Already present before C3:

- `helix differential FILE FILE` (`src/differential.rs`)
- JSON `helix-differential-v1` (descriptive; `creates_verification: false`; `ranking_semantics: absent`)
- join by `check_id`
- difference classes (`same_behavior`, `target_behavior_difference`, `fixture_capability_difference`, …)
- per-target `ga4gh_requirement`, `execution_id`, `target_execution_id`
- missing checks already modeled as `insufficient_evidence` (not SKIP)

C3 did not replace that engine.

## Changes

- Human text names the **verification contract**, each **target identity**, **evidence standing**, coverage state, and a check table joined by `check_id`.
- Attribution uses the existing `FailureAttribution` model.
- Interpretation counts difference classes. PASS is not VERIFIED. No ranking language.
- `differential_files` fails closed on invalid evidence or mismatched standard / selected version / `execution_id`.
- Optional JSON fields: `coverage_id`, per-target `evidence_standing`, `coverage_state`, `implementation_name`. Schema remains `helix-differential-v1`.
- Inputs are never rewritten.

## Operator workflow

```bash
helix verify TARGET --standard drs --version 1.4.0 --output FILE
helix inspect FILE
helix differential starter-kit.json bento.json
```

Optional live helper (not prove): `make verify-independent` then differential of `local/independent/*-current.json`. Historical B12: `local/b12/starter-kit.json` and `local/b12/bento.json`.

## Example interpretation

**Automated fixture (this session).** Two in-process DRS 1.4.0 mocks share `execution_id` `ee00e1a4…` and differ in `target_execution_id`. Matching catalog rows show `PASS | PASS`. After flipping `drs.object.schema` on one side and restamping that run only, the row is `PASS | FAIL (target_failure)` with class `target_behavior_difference`. `ga4gh_requirement` becomes VERIFIED vs NOT_VERIFIED. That is fixture evidence, not live Starter Kit or Bento.

**Historical B12 (read-only, if present).** Starter Kit vs Bento JSON remains `historical_observation`, unrestamped. Same `execution_id`; distinct `target_execution_id`. Typical observed pattern under the existing contract: Starter Kit NOT_VERIFIED with FAIL/SKIP rows; Bento VERIFIED with `coverage` partial. That pattern is evidence, not a hard-coded differential rule.

## Identity handling

Same `execution_id` is printed as `same_execution_id: yes` when both files record the same value. Distinct `target_execution_id` values are printed per target and summarized. Target ids and implementation names come from the evidence. They are not collapsed.

## Check-level interpretation

| Observed pair | Class (existing) |
|---------------|------------------|
| PASS vs PASS | `same_behavior` |
| FAIL vs PASS | `target_behavior_difference` |
| SKIP `fixture_unavailable` vs PASS | `fixture_capability_difference` |
| Present vs missing | `insufficient_evidence` (shown as **ABSENT**, not SKIP) |

Attribution (`target_failure`, `transport_failure`, `helix_execution_failure`, …) is the existing model. SKIP is not rewritten as FAIL.

## VERIFIED interpretation

`PASS` is one check. `ga4gh_requirement` VERIFIED is a derived claim over the selected contract. More PASS rows are not a ranking. Historical standing is not NOT_VERIFIED; current standing is not VERIFIED.

## Immutability

`helix differential A B` does not write A or B. Tests require hash before equals hash after, including historical B12 when those files are present.

## Tests

`tests/c3_differential_interpretation.rs`: T1 same contract; T2 distinct targets; T3 PASS/PASS; T4 FAIL/PASS; T5 SKIP/PASS; T6 attribution; T7 VERIFIED/NOT_VERIFIED; T8 invalid rejected; T9 versioned vs unversioned rejected; T10 different standard/version rejected; T11 no mutation; T12 no ranking; T13 historical B12; T14 current standing unchanged. Negative: missing check is ABSENT; changing a result changes the differential; coverage/`drs.security.authorization` unevaluated.

Offline: `cargo fmt`, `cargo test --offline --locked --all-targets`, clippy `-D warnings`, `./scripts/prove.sh`.

## Live evidence

This session: **live evidence not exercised**.

| Target | Probe |
|--------|--------|
| Starter Kit `http://127.0.0.1:4500` | down |
| Bento `http://127.0.0.1:5000` | down |

No live JSON was manufactured. C3 tests used in-process fixtures and read-only historical B12 files.

## Boundaries

- no new DRS checks
- no new pack
- no coverage expansion
- B13 remains deferred
- no WES SUPPORTED
- no HELIOS signing
- no ranking
- no cache
- no telemetry
- no upload
- `make prove` still does not start live targets

## Findings

- Unsigned Helix JSON can still be restamped consistently (C2/HELIOS bound). Differential interprets what it is given after integrity checks; it does not sign.
- Live Starter Kit/Bento remain optional. This package does not require them for prove.
- `helix-differential-v1` is a derived comparison, not a third `VerificationRun`.

## Verdict

**PASS WITH FINDINGS**

The operator path can explain what was verified, where the two artifacts differ, how existing attribution classifies the difference, and what Helix does not conclude. Live targets were not required and were not fabricated. B13 remains deferred.
