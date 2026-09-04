# Verification-run identity

A **run identity** is the small set of fields Helix records so two `helix verify` JSON files can be compared (`helix compare`).

It is **not** a signed audit trail. It is **not** scientific reproducibility. It does **not** belong to HELIOS. No signature, RO-Crate, PDF, evidence pack, or hash chain.

HelixTest already runs the checks. This document names which JSON fields identify a measurement. Distinction vs HELIOS: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md).

Source: `src/run_identity.rs` (`RunIdentity::from_verify`). Compare: [REGRESSION.md](REGRESSION.md).

---

## What is recorded

| Fact | JSON | Notes |
|------|------|--------|
| Helix version | `helix_version` | Crate version (`0.1.0`) |
| HelixTest version | `helixtest_version`, `helixtest_sha` | Pin **v0.1.3** / SHA from [VERSIONS.lock](../VERSIONS.lock) |
| Profile | `profile` | `generic` or `ferrum`. Not HelixTest Mode |
| Test IDs | `executed[].id` + `skipped[].id` | Stable Helix ids ([TEST_IDENTITY.md](TEST_IDENTITY.md)) |
| Target identifier | `target.url` | Normalized origin. Not a Ferrum id |
| Fixture version | `fixture_version` | Catalog `helix-fixtures-v1` ([FIXTURES.md](FIXTURES.md)) |
| Schema version | `schema_version` | `helix-verification-v1` |
| Timestamp | `timestamp` | RFC3339 UTC seconds. Wall clock, not a signature |
| Benchmark workload version | `helix bench` JSON `workload_id` / `workload_version` | `http.drs.smoke.v1` / `1`. **Not** on `helix verify`. [BENCHMARKS.md](BENCHMARKS.md) |

`helix compare` copies the verify fields into `previous_identity` / `current_identity` on `CompareReport`. It does not add new facts.

---

## What compare does with identity

| Flag | True when | Effect on exit |
|------|-----------|----------------|
| `same_measurement` | schema, profile, `fixture_version`, target (and bench workload if present) match | None. Informational |
| `suite_changed` | `helix_version` or HelixTest pin differs | None. Informational |
| `identity_mismatches` | listed field pairs that differ | None. Informational |

Check-id set differences are a **catalog** change (`check_ids` in mismatches). They already appear as `ADDED` / `NEW_SKIP` rows. They do not set `NEW_FAIL` by themselves.

Timestamp is **recorded**. It is **not** a mismatch field (two runs always differ in time).

Identity mismatch is **not** a verification regression. Exit 1 remains **only** `NEW_FAIL` (or unreadable JSON).

---

## What this is not

- Not a signed audit trail
- Not scientific reproducibility
- Not HELIOS (`helios-audit`): no signed evidence chain, no RO-Crate 1.1, no PDF, no pipeline envelope
- Not GA4GH certification
- Not a content hash of the target
- Not `helix security` OverallReport

Do not add `signature`, `ro_crate`, `audit_trail`, `pdf`, or `evidence` to `VerificationRun` or `CompareReport`.
