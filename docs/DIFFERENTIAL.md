# Helix target differential

Helix is HelixTest becoming a standalone VERIFY CLI. This document describes **check-level comparison** of two already-executed `helix verify` runs. It is not GA4GH certification. It does not rank implementations. HELIOS still owns signed evidence / RO-Crate / PDF. Trust: [TRUST.md](TRUST.md).

---

## What it is

`helix differential first.json second.json` loads two `VerificationRun` documents and emits `helix-differential-v1`:

- shared specification identity (standard, selected version, release commit, pack/schema hashes, checker, binding, catalog)
- per-target identity, reviewed independence, `execution_id`, `target_execution_id`, declared / detected / selected / verified versions, and `ga4gh_requirement`
- per-check status, attribution, diagnostic, and a difference class

Each target’s claim remains derived from its own `VerificationRun`. The differential artifact is descriptive. `creates_verification` is always false.

---

## What it is not

- a ranking, leaderboard, score, or compliance percentage
- a statement that one implementation is better
- a global “DRS 1.4.0 VERIFIED” claim transferred from one target to another
- HELIOS (no signatures, RO-Crate, PDF)
- `helix compare` (that command is PASS→FAIL regression on one target over time)
- `helix matrix` (that command is operator-labeled interop; it is not reviewed independence)

---

## Difference classes

| Class | Typical meaning |
|-------|-----------------|
| `same_behavior` | Same status and attribution |
| `target_behavior_difference` | Both executed; responses differ (e.g. FAIL vs PASS on `access_methods`) |
| `fixture_capability_difference` | One side SKIP `fixture_unavailable` |
| `target_configuration_difference` | Configuration/TESTABLE skip without fixture-unavailable |
| `environment_difference` | Transport failure on one side |
| `verification_execution_difference` | Helix execution failure on one side |
| `insufficient_evidence` | Check present on only one run |

Do not read every difference as a spec-compliance difference.

---

## Independence

`independent_implementation_evidence` is true only when **both** runs satisfy `run_counts_as_independent` ([TARGETS.md](TARGETS.md) §5). Operator `--target-kind` alone is not enough.

---

## Reproduction

```text
helix differential starter-kit.json bento.json --format json
```

Same DRS 1.4.0 pack, checker, and catalog must already be recorded on both input files. Helix does not re-run verify here. Files that are invalid, or that do not share a selected standard/version/`execution_id`, are rejected. Historical B12 JSON remains comparable as `historical_observation`. `helix differential` does not rewrite either file.

Operator interpretation (C3): [C3_DIFFERENTIAL_INTERPRETATION.md](C3_DIFFERENTIAL_INTERPRETATION.md). Two-target path: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md).
