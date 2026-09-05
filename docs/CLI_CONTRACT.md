# Helix CLI contract

This is an **API compatibility contract** for the `helix` binary. Changing a frozen rule here is a compatibility break for CI, helix-action, and any consumer that parses stdout or exit codes.

Helix is HelixTest becoming a standalone VERIFY CLI (separate git root, pin **v0.1.3**). This document freezes **how operators invoke Helix**, not HelixTest’s own `helixtest` CLI. Results are not GA4GH certification. HELIOS (`helios-audit`) is out of scope. The binary is **`helix`**, never `helios`.

JSON shape details: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md). Human text: [REPORT.md](REPORT.md). Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md). Discovery words: [DISCOVERY.md](DISCOVERY.md). DRS/WES execution: [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md). Profiles: [PROFILES.md](PROFILES.md). Diagnostics: [DIAGNOSTICS.md](DIAGNOSTICS.md). Regression: [REGRESSION.md](REGRESSION.md). Threat model (Helix as a client): [THREAT_MODEL.md](THREAT_MODEL.md).

Source of truth for argv: `src/main.rs`. Source of truth for verify JSON: `src/model.rs` `VerificationRun`.

---

## Frozen verify invocations

These three are equivalent except for output encoding. `<url>` is a required positional gateway-style origin (`http` or `https`, host required; trailing slash stripped; **URL userinfo `user:pass@` is rejected** and the password is not echoed).

```text
helix verify <url>
helix verify <url> --format text
helix verify <url> --format json
```

`--profile generic|ferrum` is an **additive** optional flag. Default is `generic`. Omitting it is the same as `--profile generic`. A generic target never switches to `ferrum` from service-info `name` or URL. Unknown values are usage errors (exit 2). See [PROFILES.md](PROFILES.md).

Versioned selection (additive; default `helix verify <url>` is unchanged):

```text
helix verify <url> --standard drs --version 1.5.0
helix verify <url> --standard drs --all-supported-versions
```

| Form | Meaning |
|------|---------|
| `helix verify <url>` | Unversioned HelixTest wrap. Does **not** select a registry pack. |
| `--standard drs --version 1.4.0` | Mode 1. Exact pack. DRS 1.4.0 is SUPPORTED. Not VERIFIED by default. |
| `--standard drs --version 1.5.0` | Mode 1. Exact pack. No substitution. DRS 1.5.0 today is `AVAILABLE_BUT_NOT_SUPPORTED`. |
| `--standard drs` (no `--version`) | Mode 2. Fail closed (`INSUFFICIENT`). Helix does not guess 1.4.0. |
| `--standard drs --all-supported-versions` | Mode 3. OfficialSupported only. Today: DRS 1.4.0. Does not iterate AVAILABLE rows (1.5.0). |
| `--release-class` | Mode 1 only (requires `--version`). Default `official`. `development` is never selectable. |

`--version` without `--standard`, `--all-supported-versions` without `--standard`, and `--version` with `--all-supported-versions` are usage (exit 2). Details: [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md).

Rules:

| Form | Output | Notes |
|------|--------|--------|
| `helix verify <url>` | **text** (default) | Same as `--format text` |
| `--format text` | Human report on **stdout** | Color only when stdout is a TTY and `NO_COLOR` is unset |
| `--format json` | Helix `VerificationRun` JSON on **stdout** | Pretty-printed. One JSON value. No ANSI. No discovery table |

`--report` is a **visible alias** of `--format` (`--report json` ≡ `--format json`). Do not remove it.

`--format` values are `text` and `json` only. Other values are usage errors.

TES / TRS / htsget checks are not executed by `verify` today. Discovery of those APIs is recorded; it is not a pass.

---

## Names

| Surface | Name | Frozen? |
|---------|------|---------|
| Binary | `helix` | yes — never `helios` |
| Verify | `helix verify` | yes |
| Compare | `helix compare` | shipped; not the verify freeze; [REGRESSION.md](REGRESSION.md) |
| Matrix | `helix matrix` | shipped; pending without independent runs; [INTEROP.md](INTEROP.md) |
| Format flag | `--format` / `--report` | yes |
| Format values | `text`, `json` | yes |
| Profile flag | `--profile` (`generic` or `ferrum`) | additive; default `generic`; not inferred from the target |
| Standard / version flags | `--standard`, `--version`, `--all-supported-versions`, `--release-class` | additive; default verify stays unversioned; [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md) |
| Target identity flags | `--target-id`, `--target-kind`, `--implementation-name`, `--implementation-version` | additive; declared untrusted metadata; [TARGETS.md](TARGETS.md) |
| HelixTest binary | `helixtest` | separate product; not this contract |
| Standards registry | `helix standards` | shipped; provenance only; [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) |

---

## Exit codes (`helix verify`)

Print the report **first** (text or JSON), then exit.

| Code | When |
|------|------|
| **0** | Overall status is **pass**: at least one executed check has `status: pass`, and none have `fail` or `error`. |
| **1** | Verification did not pass: any `fail` or `error`, skip-only (no pass), unreachable target (ERROR rows), or a runtime error after argv parsed (invalid URL, adapter `Err` that is not turned into ERROR rows). |
| **2** | Clap usage error (missing `<url>`, unknown flag, unknown `--format` value, unknown `--profile` value, `--version` without `--standard`, `--all-supported-versions` without `--standard`, `--version` with `--all-supported-versions`, unknown subcommand). |

Skip-only is **not** a pass. A live HTTP server with no DRS and no WES exits **1** under `generic`. A DRS-only success (DRS passes, WES skipped) exits **0** under `generic`. `--profile ferrum` expects DRS and WES; a DRS-only target fails missing WES and exits **1**.

`--help` / `--version` exit **0**.

`helix security` and `helix bench` keep their own exit tables (below). `helix compare` is a separate table ([REGRESSION.md](REGRESSION.md)): exit 1 only on `NEW_FAIL` (PASS→FAIL at stable id), not on a score drop. They are not this verify freeze.

---

## Stdout / stderr

### `--format json`

- **Stdout** is exactly one UTF-8 JSON value: Helix `VerificationRun`, pretty-printed, optional trailing newline.
- Stdout is **machine-readable**. It must not contain:
  - ANSI / color codes
  - the human discovery table
  - `PASS` / `FAIL` / `SKIP` / `ERROR` marks (JSON uses lowercase `pass` / `fail` / `skip` / `error`)
  - log lines, progress, or tracing
- **Stderr** is for logs and usage/runtime errors. HelixTest `HttpClient` logging is initialized to **stderr** (`common::logging`). Helix sets `RUST_LOG=error` when unset so the default CLI/`make verify-fixture` path does not dump DEBUG GET traces before `HELIX VERIFICATION`. `RUST_LOG=debug` restores HelixTest traces; it must not write those lines to stdout.
- **Secrets.** Stdout and stderr must not contain HMAC secret values, minted JWTs, or `Authorization` header values. Target bodies that echo those strings are redacted ([THREAT_MODEL.md](THREAT_MODEL.md)). Dummy fixtures only (NICHT FÜR PRODUKTION).

A consumer may `json.loads(stdout)` (or equivalent) without stripping logs.

### `--format text` (and default)

- **Stdout** is the human report ([REPORT.md](REPORT.md)): `HELIX VERIFICATION`, then Target / Helix / Test suite / Standards / Services / Results grouped by service / Summary / Changes. Marks are `PASS`/`FAIL`/`SKIP`/`ERROR` with Helix `id` and `code`. Fail/error rows for catalogued DRS/WES ids may print indented `expected` / `observed` / `category` / `hint` / `possible causes` ([DIAGNOSTICS.md](DIAGNOSTICS.md)). That is not a new status and must not be headed `Cause:`.
- Services use the same words as JSON `present`/`testable`: `NOT_DETECTED` / `DETECTED` / `TESTABLE` / `NOT_TESTABLE`.
- `Changes:` on a single verify run is **Not compared** (no previous `VerificationRun`). Deltas are `helix compare`.
- Color is allowed on a TTY when `NO_COLOR` is unset. Skip is never green. Discovery lines are never a green PASS. Target-controlled `message` / `observed` bytes are sanitized (no CSI, no extra newlines) so they cannot paint a fake PASS or forge a second `HELIX VERIFICATION` header ([THREAT_MODEL.md](THREAT_MODEL.md)).
- Text must not print `found` as if the service were verified.
- Closing lines: this is a technical verification signal; it is not GA4GH certification; discovery is not conformance.

### Usage / runtime errors (no verification run)

If argv is invalid or the URL cannot be parsed before a run exists: **no** `VerificationRun` JSON on stdout. Clap usage text goes to stderr (exit 2). `anyhow` runtime errors (e.g. `ftp://…`, URL userinfo) go to stderr (exit 1) and must not echo embedded passwords.

---

## JSON stability (`--format json`)

Schema: `VerificationRun` ([VERIFICATION_MODEL.md](VERIFICATION_MODEL.md), frozen JSON Schema [SCHEMA.md](SCHEMA.md) / `schemas/helix-verification-v1.json`).

**Required objects / arrays** (always present):

| Field | Meaning |
|-------|---------|
| `schema_version` | Frozen document id: `helix-verification-v1` ([SCHEMA.md](SCHEMA.md), `schemas/helix-verification-v1.json`) |
| `helix_version` | Helix crate version (`Cargo.toml`, same as `helix --version` package version) |
| `timestamp` | RFC3339 UTC seconds (`…Z`). Wall clock, not a signature. Not a reproducibility hash |
| `target.url` | Normalized origin (no trailing slash) |
| `discovery` | Array of `{service, present, testable, …}` — **not a pass** |
| `executed` | Results with `pass` / `fail` / `error` |
| `skipped` | Results with `skip` only |
| `summary` | `{passed, failed, skipped, errors, total}` counts, not a score |

**Optional** (omitted when unset; consumers must not require them): `helixtest_version`, `helixtest_sha`, `profile`, `fixture_version` (Helix always emits `helix-fixtures-v1` today; missing on old files deserializes as that value), `standard_selection` (Helix always emits it today; missing on old files is unversioned), `layer_summary` (Helix always emits it today; no `percent` / `score` / `compliant`; SCHEMA PASS is not BEHAVIOR PASS; [BEHAVIOR.md](BEHAVIOR.md)), `claims` (Helix always emits six items today; missing on old files is not VERIFIED; human text is generated only from this array; [CLAIMS.md](CLAIMS.md)), per-result `helixtest_name` / `profile` / `message` / `failure` / `diagnostic` / `standard` / `requested_version` / `detected_version` / `selected_version` / `verified_version` / `standards_registry_entry` / `standards_source_commit` / `layer` / `observed_response` / `traceability` (Helix always emits `traceability` today; missing on old files is empty; `category`/`check_kind=normative` is not used in the shipped catalog; `claim_scope` is never `ga4gh_requirement`), discovery `base_url` / `not_testable_reason`.

`requested_version` is the operator instruction. `detected_version` is copied from 2xx service-info `type.version` only (never from URL `/v1`). `selected_version` / `verified_version` are Helix choices and stay empty when the row is AVAILABLE-only or unknown. Those fields must not imply the target declared a version Helix merely selected. `substituted` on `standard_selection` is always `false`.

Run identity for `helix compare` is these fields plus check ids and timestamp ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not a signed audit trail. Not HELIOS.

`diagnostic` on a fail/error row is additive and catalogued for DRS/WES only ([DIAGNOSTICS.md](DIAGNOSTICS.md)). It does not change `status` or exit codes. Pass/skip omit it. It is **possible causes**, never a field named `cause`. A bench warning is unrelated and is not this field.

`helix verify` always sets run-level `profile` to `generic` or `ferrum` ([PROFILES.md](PROFILES.md)). That is the Helix profile id, not HelixTest Mode. Per-check `profile: "generic"` on translated rows still means Mode::Generic. Each result has `service`.

**Stability rules (breaking if violated):**

- Do not rename or remove a required field.
- Do not change `status` strings (`pass` \| `fail` \| `skip` \| `error`).
- Do not add a `passed` boolean. Skip cannot be stored or serialized as pass.
- Do not emit HelixTest `OverallReport` (`services`, per-test `passed`) from `verify`. There is no `checks` array; checks are `executed` + `skipped`.
- Do not add HELIOS fields (`signature`, `ro_crate`, PDF, audit trail).
- `executed` / `skipped` are sorted by `code`, then `id`.
- Assigned Helix `id` / `code` pairs are a compatibility change ([TEST_IDENTITY.md](TEST_IDENTITY.md)).
- Adding an **optional** field is non-breaking for consumers that ignore unknown keys. This v1 schema file uses `additionalProperties: false`; a new field needs a new schema file and `schema_version` ([SCHEMA.md](SCHEMA.md)), except `fixture_version` (compare identity), the standard-version fields, per-check `traceability`, the layer fields (`layer`, `observed_response`, `layer_summary`), and `claims` on this same v1 file.

Same mock URL, same binary, same HelixTest pin, same fixture catalog: JSON values match after replacing `timestamp` (`tests/repro.rs`). That is **not** bit-for-bit identity of two raw files. A new mock process may bind a different `target.url` port. Comparable runs may also differ in Helix/HelixTest version (`suite_changed`); that is identity, not `NEW_FAIL` ([RUN_IDENTITY.md](RUN_IDENTITY.md), [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md)).

---

## Version reporting

| Surface | What |
|---------|------|
| `helix --version` | clap banner including Cargo package version |
| JSON `helix_version` | that same package version string (e.g. `0.1.0`) |
| JSON `schema_version` | frozen document id `helix-verification-v1` |
| JSON `fixture_version` | fixture catalog id `helix-fixtures-v1` ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not HELIOS |
| JSON `helixtest_version` | HelixTest **git tag** from [VERSIONS.lock](../VERSIONS.lock) (`v0.1.3`) |
| JSON `helixtest_sha` | HelixTest git SHA from the lockfile |

Operators pin HelixTest by **tag/SHA**, not crate `0.1.0`. Do not invent a later HelixTest tag here.

---

## Error semantics

| Class | stdout | stderr | exit |
|-------|--------|--------|------|
| Usage (clap) | empty / clap help | clap message | 2 |
| Invalid URL / empty host / non-http(s) | no VerificationRun | anyhow message | 1 |
| Target TCP unreachable | VerificationRun (or text): DRS and WES rows `error` | logs only | 1 |
| HelixTest adapter `Err` | those service rows `error` (`HelixTest adapter error: …`) | logs | 1 |
| Target assertion failed | those checks `fail` + `failure.code` | logs | 1 |
| Skip-only | `skipped` rows, `summary.passed = 0` | logs | 1 |

`fail` is a negative assertion about the target. `error` is the runner could not execute the check. Both block exit 0. They are not collapsed.

---

## SKIP semantics

- `status` is `skip`. Never `pass`. JSON has no `passed` field to misuse.
- Skip does not increment `summary.passed`. Skip does not set exit 0 by itself.
- Skip is not green in text.
- Causes today: service `NOT_DETECTED` and **not** expected by the profile; service DETECTED but not TESTABLE (when not expected); HelixTest skip (WES scatter/gather on `generic`, `supports_scatter_gather=false`); version selection failed (`AVAILABLE_BUT_NOT_SUPPORTED`, `UNKNOWN_TO_HELIX`, `NO_OFFICIAL_SUPPORTED`, `standard not selected`). Expected-but-missing services are **fail**, not skip (`ferrum` expects DRS and WES).
- `push_skipped` forces skip even if a pass result is passed in (library invariant).

---

## Discovery semantics

Discovery is **not** conformance. JSON `discovery[].present` is DETECTED. `discovery[].testable` is TESTABLE (Helix will execute that suite). **Neither is a pass.** `summary.passed` is not `present`.

Text uses `NOT_DETECTED` / `DETECTED` / `TESTABLE` / `NOT_TESTABLE`. Never `found` as verified.

Order in the table and in `discovery[]`: DRS, WES, TES, TRS, htsget.

Today TESTABLE when DETECTED: **DRS and WES**. TES / TRS / htsget stay NOT_TESTABLE with a reason. Consumers must **read** `testable`; do not hard-code that WES is never testable.

Discovery rows are not emitted as `executed[]` checks (`discovery.drs` is catalog-only, not a verify result).

---

## Future command namespaces (not implemented)

Reserved. Do not ship these names as silent aliases of `verify`. Implementing one is a **new** contract revision, not a drive-by.

| Namespace | Intent when implemented |
|-----------|-------------------------|
| `helix tes` | TES verification (same format/exit/skip/discovery rules unless that revision says otherwise) |
| `helix trs` | TRS verification |
| `helix htsget` | htsget verification |
| `helix beacon` | Beacon verification |
| `helix certify` / `helix certificate` | **Never** — not certification |

Until implemented, these are unknown clap subcommands (exit 2). TES/TRS/htsget remain discoverable under `helix verify` only.

**Shipped (not reserved):** `helix matrix` — interoperability matrix ([INTEROP.md](INTEROP.md)). Default (no `--run`) emits **pending** Ferrum and independent slots. In-process fixtures are not independent evidence. Not certification.

**Never reserved for Helix:** `helios` (HELIOS CLI / `helios-audit`).

---

## Other shipped commands (not the verify freeze)

These exist today. Their JSON is **not** `VerificationRun`. Changing them is a separate compatibility discussion.

### `helix security <url>`

Security Behavior Profile ([SECURITY_PROFILE.md](SECURITY_PROFILE.md)), then Crypt4GH protocol layout ([CRYPT4GH.md](CRYPT4GH.md)). Dummy HMAC (`test-fixtures/`, NICHT FÜR PRODUKTION). `--format json` is HelixTest `OverallReport`. Exit 0 if no executed `status: fail`; skip-only auth (no secret) is not a fail. Crypt4GH HTTP envelope (`HLX-AUTH-054`) skips when the body is not Crypt4GH. Text prints: Helix verifies selected security behavior; it is not a penetration test, security audit, or certification. Passing does not prove the implementation is secure. A Crypt4GH pass is not “secure”. Not HELIOS.

### `helix bench --baseline <url> --candidate <url>`

Stage 4 measurement engine ([BENCHMARKS.md](BENCHMARKS.md)). Fixed workload **`http.drs.smoke.v1`**. Warm-up then measured repetitions. JSON is Helix `BenchOutcome` with distribution analysis (`analysis.measurement` / `warning` / `regression`; `verification_failure` is always false). Compares median, p95 where available, error-rate, optional RSS — not a single wall-clock sample. `warning: true` does **not** change exit (always 0 after a successful run) and is not a verification failure. A warning means performance changed enough to merit human inspection; it does not mean the implementation is incorrect. Thresholds do not fail CI. Sample percentiles, not a significance test. Not Demo hap.py, not GIAB, not HELIOS.

```text
helix security <url>
helix security <url> --format json --hmac-secret-file test-fixtures/hmac/shared-secret.txt
helix bench --baseline <url> --candidate <url>
helix bench --baseline <url> --candidate <url> --format json --threshold 10 --warmup 1 --repetitions 5
helix compare <previous.json> <current.json>
helix compare <previous.json> <current.json> --format json
helix matrix
helix matrix --format json
helix matrix --run ferrum=ferrum.json --kind ferrum=reference_target --run other=other.json --kind other=independent_implementation
```

### `helix compare <previous.json> <current.json>`

Compares two `helix verify --format json` files at stable Helix `id`. A **regression** is PASS→FAIL (`NEW_FAIL`). Fail→fail is `UNCHANGED_FAIL` (existing failure), not a new regression. SKIP→PASS is `FIXED_SKIP`, never a silent pass. JSON includes `previous_identity` / `current_identity` / `same_measurement` ([RUN_IDENTITY.md](RUN_IDENTITY.md)); identity mismatch is not `NEW_FAIL`. Text: [REPORT.md](REPORT.md) (`HELIX VERIFICATION COMPARE`). Exit 0 if no `NEW_FAIL`; exit 1 on regression or unreadable JSON; exit 2 on usage. Not a score. Not HELIOS. Details: [REGRESSION.md](REGRESSION.md).

### `helix matrix`

Same generic `helix verify` suite, compared across operator-labeled run JSON files ([INTEROP.md](INTEROP.md)). JSON is `helix-interop-matrix-v1`, not `VerificationRun`. No `--run` → pending Ferrum and independent slots. Two fixture runs are not independent evidence. Exit 1 only on `must_agree` executed disagreement. Not certification.

### `helix standards list|show|validate|trace`

Pinned GA4GH specification provenance ([STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md)), claim taxonomy ([TAXONOMY.md](TAXONOMY.md)), and per-check kind/authority ([TRACEABILITY.md](TRACEABILITY.md)). Does **not** run `helix verify`. Does **not** download specs. Default discovery is **OFFICIAL ∩ SUPPORTED** (currently empty). `show` is an exact version match (`substituted: false`). `trace CHECK_ID` prints catalog provenance for one Helix id (not a MUST; none are `normative` today). Exit 0 on `list` / successful `validate` / `show` of a registry row (including AVAILABLE-not-SUPPORTED) / `trace` of a catalogued id. Exit 1 on validate failure, unknown version, or unknown check id. Exit 2 on usage. Not certification. Not HELIOS.

```text
helix standards list
helix standards list --supported-only --format json
helix standards show drs 1.5.0
helix standards validate
helix standards trace drs.object.schema
helix standards trace drs.object.schema --format json
```

---

## Out of this contract

- HELIOS: signatures, RO-Crate, evidence, PDF
- Scores, ISO 15189 / AI Act, certification marks
- Ferrum as a crate or a required URL
- `helixtest --all --mode …` (HelixTest CLI)
- Starting servers (`make up`)
