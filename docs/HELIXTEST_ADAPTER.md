# HelixTest adapter boundary

Helix is HelixTest becoming a standalone VERIFY product. HelixTest already runs (one of five public GA4GH-stack repos, CI, SF-TR-2026-001 / 002). This adapter productizes that engine; it does not invent a second suite.

HelixTest stays a **separate repository and a separate git root** (D1). Helix path-depends on the pinned sibling checkout (`helixtest-common` / `helixtest-framework`). Do not merge git histories, vendor the suite, or rewrite HelixTest in this repo.

Source: `src/adapter/`. Pin: [VERSIONS.lock](../VERSIONS.lock) **v0.1.3** / SHA `1832c043e1679ec283cb2113510ee33684317cce`. Domain types: [VERIFICATION_MODEL.md](VERIFICATION_MODEL.md). Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md). Architecture seam: [ARCHITECTURE.md](ARCHITECTURE.md) §5.

Ferrum is not a Helix runtime dependency. This adapter **does not import Ferrum**. Ferrum may appear only as a test target, a fixture, or a documentation reference. HELIOS stays out of this boundary (`helios-audit`; no RO-Crate / PDF / signed evidence here).

---

## Why this exists

`helix verify` discovers APIs, then needs to execute the existing HelixTest DRS and WES checks and speak Helix’s result language (`VerificationResult`: pass / fail / skip / error, no `passed` bool).

Without an adapter, `src/verify.rs` called `framework::drs::run_drs_checks` directly. That couples orchestration to HelixTest internals and makes a later replacement (or a second engine) a rewrite.

The adapter is the **only** conformance `framework::*` call site. Discovery, reporting, security, bench, and the CLI stay Helix-owned. `helix verify` JSON is Helix `VerificationRun`. `helix security` JSON remains HelixTest `OverallReport`.

---

## Boundary

```text
helix verify
    │
    ▼
src/verify.rs          orchestration (discovery → adapter → CLI report)
    │
    │  HelixTestAdapter::run_drs / run_wes(base_url)
    ▼
src/adapter            THIS BOUNDARY
    │  1. invoke pinned HelixTest (Mode::Generic; Features from Helix profile)
    │  2. translate TestCaseResult → VerificationResult
    │  3. record HelixTest tag + SHA
    │
    ├──────────────────────────────┬─────────────────────────────┐
    ▼                              ▼                             ▼
ServiceReport                 Vec<VerificationResult>      HelixTestPin
(adapter internal;            (Helix id/code/name;         tag v0.1.3
 not verify JSON)             original name preserved)     SHA 1832c043…
```

Pinned HelixTest crates live in the sibling git root `../HelixTest`. They are not this repository.

---

## Responsibilities

| Does | Does not |
|------|----------|
| Invoke pinned HelixTest DRS and WES checks (`run_drs_checks`, `run_drs_checks_with_spec`, `run_wes_checks`) | Merge or rewrite HelixTest |
| Translate each `TestCaseResult` into a Helix `VerificationResult` | Import Ferrum implementation code |
| Preserve original HelixTest test identity (`helixtest_name` + catalog map) | Rename HelixTest test names |
| Preserve PASS / FAIL / SKIP; **never convert SKIP into PASS** | Read HelixTest `passed` (ignored; Skip cannot become Pass) |
| Preserve HelixTest `error` text on `message` / `failure.detail` | Invent certification, scores, or HELIOS evidence |
| Record the HelixTest version/pin used (`HelixTestPin`) | Call `run_all`, `--mode ferrum`, or compose / `--start-ferrum` |
| Use `Mode::Generic` and discovery-filled URLs | Treat a WES `name` of “Ferrum Gateway” as a Ferrum stack |

---

## Invocation (current pin)

Allowed behind this adapter only:

- `framework::drs::run_drs_checks` (unversioned default verify) and `framework::drs::run_drs_checks_with_spec` (versioned DRS pack; caller supplies SpecSource bytes). `framework::wes::run_wes_checks` (TES / TRS / htsget later, same seam). The versioned path must not call bundled `run_drs_checks`.
- `Features` from the Helix profile (`src/profile.rs`): `generic` has checksums on and scatter off; `ferrum` has both on. Never Ferrum mode.
- `Mode::Generic` always (not Ferrum mode; not inferred from WES `name`)
- `TestConfig` with `drs_url` / `wes_url` from Helix discovery (not Ferrum defaults)
- `HttpClient` from HelixTest common (timeout/retry vs Helix discovery client are known)
- HelixTest report types (`ServiceReport`, `TestCaseResult`) so CLI JSON stays D3

The published pin is the **git tag / SHA**, not crate `0.1.0`:

| Field | Value |
|-------|--------|
| Tag (`HELIXTEST_PIN` / `HELIXTEST_REF`) | `v0.1.3` |
| SHA (`HELIXTEST_SHA`) | `1832c043e1679ec283cb2113510ee33684317cce` |

`HelixTestAdapter::pinned()` stamps every `AdapterOutcome` with that pair. Do not bump off this pin unless HelixTest has a tag Ferrum / Lab Kit / ga4gh-infra can take.

---

## Translation

HelixTest `TestCaseResult.status` is the only status input. Mapping:

| HelixTest `status` | Helix `VerificationStatus` | Notes |
|--------------------|----------------------------|--------|
| `Pass` | `pass` | Catalog `id` / `code` / Helix `name` |
| `Fail` | `fail` | Error string → `message` and `failure.detail` |
| `Skip` | `skip` | **never convert SKIP into PASS**; skip reason preserved |
| (no Error variant) | — | Adapter `Result` Err is a runner failure, not a Pass |

HelixTest also has `passed: bool` (`true` iff Pass). The adapter **does not read it**. A Skip row with `passed: true` (malformed) still becomes Helix `skip`.

Identity, where a catalog wrap exists ([TEST_IDENTITY.md](TEST_IDENTITY.md)):

| Helix field | Source |
|-------------|--------|
| `id` / `code` | Helix catalog via exact HelixTest name (`spec_by_helixtest_name`) |
| `name` | Helix catalog title (not a rename of the HelixTest string) |
| `helixtest_name` | Original HelixTest `TestCaseResult.name` (preserved) |
| `service` / `category` / `severity` | Catalog |
| `profile` | `generic` on each translated row (HelixTest **Mode::Generic**). Distinct from run-level Helix profile `generic`/`ferrum` ([PROFILES.md](PROFILES.md)) |
| `diagnostic` | Attached after translation for catalogued DRS/WES **fail** rows from the HelixTest error string ([DIAGNOSTICS.md](DIAGNOSTICS.md)). Absent on pass/skip and on `helixtest.unmapped` |

Example: HelixTest `"DRS invalid object id returns 404"` → `drs.object.not_found` / `HLX-DRS-005` / Helix name “Unknown DRS object returns 404”, with `helixtest_name` still the original string.

If a HelixTest name is not in the catalog, the adapter still emits a result (`id` `helixtest.unmapped`, `code` `UNMAPPED`) and keeps the original name. That is a passthrough, not a new assigned `HLX-DRS-*` code.

---

## Ferrum

This adapter **does not import Ferrum**.

Ferrum may only appear as:

- a **test target** (operator points `helix verify` at a Ferrum URL they started)
- a **fixture** (documentation / mock `name` fields that must *not* switch Helix to Ferrum mode)
- a **documentation reference** (this file, architecture, prove notes)

The in-process generic DRS fixture (`tests/support/mock_ga4gh_drs.rs`) is DRS-only: it does **not** mount a WES-shaped `/service-info`. HelixTest B1 still does (Ferrum-name trap); Helix adapter uses `Mode::Generic`, so that trap stays HelixTest’s concern. WES proof is `tests/support/mock_ga4gh_wes.rs` (not Ferrum; name is not “Ferrum Gateway”).

Do not change the Ferrum repository for this boundary.

---

## What stays where

| Lives in HelixTest (sibling git root) | Lives in Helix |
|---------------------------------------|----------------|
| Check implementations (`run_drs_checks`, `run_wes_checks`, later TES, …) | Adapter call + translation |
| HelixTest test **names** (unchanged) | Helix `id` / `code` catalog wraps |
| `OverallReport` / `ServiceReport` types | CLI `helix security` still prints OverallReport; verify prints VerificationRun |
| Pin tags | `VERSIONS.lock` + `HelixTestPin` |
| | Discovery, security, bench, `helix` CLI |

Narrow exception (not this adapter): Stage 3 dummy JWT via `common::auth::build_jwt` is a test-fixture helper, not conformance execution ([ARCHITECTURE.md](ARCHITECTURE.md) §5.4).

---

## Integration proof

`tests/adapter_drs.rs` and `tests/adapter_wes.rs` run `HelixTestAdapter` against in-process generic fixtures. DRS: five identities, PASS. WES default capabilities: seven PASS plus scatter SKIP (`supports_scatter_gather=false`), never SKIP-as-PASS. Profile `ferrum` turns scatter on (`tests/verify_profile.rs`). Pin `v0.1.3` / SHA `1832c043e1679ec283cb2113510ee33684317cce`.

`tests/verify_drs.rs` / `tests/verify_wes.rs` cover `helix verify` CLI JSON (`VerificationRun`). Those paths go through the same adapter.

---

## Out of this boundary

- HELIOS: signatures, RO-Crate, evidence chains, audit trails, PDF
- Rewriting or vendoring HelixTest
- Ferrum as a crate, `make up` inside Helix, `--mode ferrum` auto-switch
- GA4GH certification claims; green results are a technical signal
- Changing assigned Helix `id` / `code` (compatibility change)
