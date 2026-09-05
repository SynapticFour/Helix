# Helix checkpoint audit

**Date:** 2026-09-04
**HEAD:** `3ca8b8c1bf635c45781e4f676b714e930e92f883` (`Productize helix verify as Helix-owned JSON with compare and fixtures.`)
**Scope:** `SynapticFour/Helix` plus the sibling HelixTest pin it path-depends on. HelixTest was **read, not modified**.
**Method:** File reads of Helix `src/`, `tests/`, `docs/`, `schemas/`, CI, `Cargo.toml`, `Cargo.lock`, `VERSIONS.lock`, and HelixTest `framework`, `ga4gh_schemas`, and vendored `helixtest/schemas/ga4gh/`. No new tests were run for this document. Prior `make prove` on this commit is recorded as passing in the commit hook, not re-executed here.

This is a forensic architecture and standards audit. It is **not** an implementation. It is **not** a public release. It is **not** GA4GH certification. Helix is HelixTest becoming a standalone VERIFY CLI. Reproducibility / signed evidence / RO-Crate / PDF stay in HELIOS (`helios-audit`). Ferrum has no real clinical pilot.

Claims are tagged:

| Tag | Meaning |
|-----|---------|
| **FACT** | Observed in this tree (or in the pinned HelixTest sources Helix compiles against). |
| **INFERRED** | Follows from facts; not independently re-executed in this review. |
| **UNKNOWN** | Not determined from the files read. Do not treat as true. |

Do not guess. Where a GA4GH normative clause cannot be pointed to **in these repositories**, the cell is **UNVERIFIED STANDARD PROVENANCE**.

---

## PART 1 — Current product state

### 1. What Helix can actually do today

**FACT:** The crate is `helix` **0.1.0**. Binary name `helix`. License Apache-2.0. Path-depends on sibling HelixTest crates `helixtest-common` and `helixtest-framework` (`Cargo.toml`). Pin: [VERSIONS.lock](../VERSIONS.lock) **v0.1.3** / SHA `1832c043e1679ec283cb2113510ee33684317cce`.

**FACT:** An operator can:

1. Point `helix verify <url>` at a running HTTP origin (or `make verify-fixture` at an in-process DRS mock).
2. Discover DRS, WES, TES, TRS, htsget under that origin (`src/discover.rs` `VERIFY_ORDER`).
3. Execute HelixTest DRS and WES checks when those services are DETECTED and TESTABLE (`src/verify.rs`, `src/adapter`).
4. Emit Helix `VerificationRun` JSON (`helix-verification-v1`) or a human text report.
5. Compare two verify JSON files at stable Helix `id` (`helix compare`).
6. Run a named HTTP security-behaviour profile (`helix security`) and a 3-GET smoke bench (`helix bench`).

**FACT:** Results are documented as a technical signal, not certification (README, `CLI_CONTRACT.md`, report footer).

### 2. Which commands work

| Command | Status | Evidence |
|---------|--------|----------|
| `helix --version` / `--help` | Works | clap in `src/main.rs`; `tests/cli_contract.rs` |
| `helix verify <url>` | Works | `src/verify.rs`; CLI + fixture tests |
| `helix verify --format json\|text` | Works | `--report` is a visible alias |
| `helix verify --profile generic\|ferrum` | Works | `src/profile.rs`; unknown values exit 2 |
| `helix compare <prev.json> <curr.json>` | Works | `src/compare.rs`; `tests/compare_cli.rs` |
| `helix security <url>` | Works (dummy HMAC/Crypt4GH) | `src/security/`; `tests/security_cli.rs` |
| `helix bench --baseline --candidate` | Works (warn-only) | `src/bench/`; `tests/bench_cli.rs` |
| TES/TRS/htsget/Beacon subcommands | **Not implemented** | `Commands` enum has four variants only |

**FACT:** `make prove` = docs greps + `cargo test --locked --all-targets`. `make verify-fixture` runs `helix verify` against the in-process DRS mock. `make test-live` is opt-in and not CI.

### 3. Which services are actually verified

**FACT:** `VERIFY_EXECUTABLE` is DRS and WES only (`src/discover.rs`). `helix verify` calls `HelixTestAdapter::run_drs` / `run_wes` when DETECTED+TESTABLE.

| Suite | Helix ids | Engine |
|-------|-----------|--------|
| DRS | `drs.object.reachable` … `not_found` (`HLX-DRS-001`–`005`) | HelixTest `framework::drs::run_drs_checks` |
| WES | `wes.service_info.reachable` … `scatter_gather` (`HLX-WES-001`–`008`) | HelixTest `framework::wes::run_wes_checks` |

**FACT:** Profile `generic` skips `wes.run.scatter_gather` (`supports_scatter_gather=false`). Profile `ferrum` enables that fixture. Execution mode is always HelixTest `Mode::Generic` (`src/adapter/mod.rs`; `prove.sh` forbids `Mode::Ferrum` in `src/`).

### 4. Which services are only discovered

**FACT:** TES, TRS, htsget are probed and recorded. If DETECTED they are **NOT_TESTABLE** with reason `Helix Stage 1 does not execute … checks; DETECTED is not a pass` (`testability_for`). Beacon is **not** in `VERIFY_ORDER` (not discovered by `helix verify`).

**FACT:** Catalog rows exist for TES/TRS/Beacon/htsget (`src/identity.rs`) but `helix verify` does not execute them.

### 5. Which features are scaffolds

| Item | Why scaffold |
|------|----------------|
| TES/TRS/htsget/Beacon catalog ids | Reserved mapping to HelixTest names; no adapter call |
| Discovery ids `HLX-DISCOVERY-001`–`005` | In catalog; discovery is not a check row in `VerificationRun` |
| HelixTest auth catalog `HLX-AUTH-001`–`006` | Mapped names; `helix security` uses Helix-native `HLX-AUTH-010`–`014` |
| Crypt4GH `HLX-AUTH-051`–`052` | Documented reserved (HelixTest secret-key path unwired) |
| `ConformanceAdapter` trait | One impl: `HelixTestAdapter` |
| `--profile ferrum` | Policy (expected DRS+WES + scatter fixture), not a Ferrum client |
| `helix security` JSON | Still HelixTest `OverallReport`, not `VerificationRun` |
| GitHub Release / crates.io | INSTALL: no formula, no release binary; path deps cannot publish |

### 6. Production-quality enough for **internal** CI

**INFERRED** from presence of locked tests, CI workflow, and the commit hook that ran prove on `3ca8b8c`:

- `helix verify` against in-process DRS/WES fixtures
- `helix compare` on `VerificationRun` at stable `id` (`NEW_FAIL` = PASS→FAIL)
- Schema validation of generated verify JSON (`tests/schema_verify.rs`)
- Redaction / no-redirect / body-cap on **Helix-owned** HTTP (`src/http_safety.rs`, `src/redact.rs`)

**FACT:** GitHub CI clones HelixTest at the pin SHA, runs `make prove`, `make verify-fixture`, clippy, fmt (`.github/workflows/ci.yml`).

Green CI remains a technical signal, not certification.

### 7. Not yet trustworthy (for external claims)

| Topic | Tag | Why |
|-------|-----|-----|
| “Verified against GA4GH DRS 1.5.0” | **FACT** this string is not supported | No DRS 1.5.0 artifact in Helix or the pin’s schema README. Vendored file is DRS **1.4.0**. Verify JSON has no `standard_version` field. |
| “Verified against GA4GH DRS 1.4.0” | **FACT** incomplete | Schema check uses HelixTest-vendored OpenAPI 1.4.0 `DrsObject`. Other DRS checks are HelixTest extras. Helix JSON does not record that 1.4.0 pin. YAML has **no commit / integrity hash** in HelixTest `schemas/ga4gh/README.md`. |
| TES/TRS/htsget/Beacon verify | **FACT** not executed | Discovery only (Beacon: not even discovery). |
| `helix security` as a GA4GH result | **FACT** Helix-owned invariants | Dummy HS256 / Crypt4GH layout. Docs: not a pentest, not ga4gh-infra. |
| `helix bench` as a publication benchmark | **FACT** smoke | `http.drs.smoke.v1`, warn-only, not GIAB / hap.py. |
| Live Ferrum / hospital target | **FACT** not in prove | `make test-live` opt-in. |
| HelixTest HTTP client | **FACT** residual | Discovery/security/bench use Helix `reqwest` (no redirects, 2 MiB). DRS/WES checks use HelixTest `HttpClient` (retries, different timeouts, gzip possible). [THREAT_MODEL.md](THREAT_MODEL.md). |
| Community OSS surface | **FACT** | No `CODE_OF_CONDUCT.md`, no issue templates ([OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md)). |
| `docs/DEPENDENCY.md` | **FACT** stale | Still says the repo has no Cargo lockfile. `Cargo.lock` is committed. |

---

## PART 2 — Architecture

### Intended vs actual separation

| Concern | Module | Separation |
|---------|--------|------------|
| CLI | `src/main.rs` | Clean. Four subcommands. |
| Discovery | `src/discover.rs` | Clean. Does not run HelixTest checks. |
| Verification orchestration | `src/verify.rs` | Clean. Discover → adapter → `VerificationRun`. |
| Test identity catalog | `src/identity.rs` | Clean as a **Helix** catalog. Not a GA4GH clause catalog. |
| Standard definitions | **Absent in Helix** | No `standards/` tree. Schema validation lives in HelixTest `common::ga4gh_schemas`. |
| Target adapters | `src/adapter` | One production adapter: HelixTest. Trait exists. |
| Security verification | `src/security/` | Separate command. Shares discovery + dummy JWT helper from HelixTest `common::auth`. JSON shape is `OverallReport`. |
| Benchmarking | `src/bench/` | Separate command. Helix-owned HTTP. Workload paths include `/health` and `/ga4gh/drs/v1/…`. |
| Reporting | `src/report.rs` | Projection of JSON. Verify text vs compare text vs security vs bench. |
| Regression | `src/compare.rs` | Separate. Keys on Helix `id`, not score. |
| Run identity | `src/run_identity.rs` | Compare metadata. Not HELIOS. |
| CI | `.github/workflows/ci.yml` | Docs + tests + fixture verify. Not live Ferrum. |
| Profiles | `src/profile.rs` | Policy (expected services, Features bits). Engine stays `Mode::Generic`. |

**FACT:** Helix does not import Ferrum as a crate.

### Coupling: Helix → Ferrum

These are **explicit** couplings (name, path, or fixture), not crate dependencies.

| Coupling | Kind | Evidence |
|----------|------|----------|
| `--profile ferrum` | Policy named after a product | `ProfileId::Ferrum`: expected DRS+WES; scatter-gather fixture on |
| Bench `/health` | Path convention | `SMOKE_REQUESTS` first GET is `/health` (`src/bench/workload.rs` comment: “Ferrum gateway and split mocks”) |
| htsget catalog names | Ferrum-specific HelixTest strings | `src/identity.rs`: “Ferrum does not slice” on reserved htsget POST-region names |
| VERSIONS.lock comments | Process | Pin bumps wait on Ferrum / Lab Kit / ga4gh-infra taking the same HelixTest tag |
| Docs / CLI help | Reference target | README honesty sentence; `helix bench` help says “Ferrum vX or Demo” |
| Dummy HMAC | Shared fixture style | `test-fixtures/hmac/` labeled NICHT FÜR PRODUKTION; not a Ferrum import |

**FACT:** `prove.sh` fails if `src/` contains `Mode::Ferrum`.

### Coupling: Helix → implementation-specific behaviour

| Coupling | Evidence |
|----------|----------|
| Fixture object `test-object-1` | Discovery probe and HelixTest DRS GETs that id (`drs.rs`, `FIXTURES.md`) |
| Unknown id string | HelixTest `nonexistent-object-id-for-conformance` (Helix mock documents the same) |
| WES workflow URLs | `trs://test-tool/echo/1.0`, `fail/1.0`, `cwl-echo/1.0`, `nonexistent/invalid/0.0`, optional scatter-gather |
| WES CWL `v1.2` + `echo_out` / `hello-ga4gh` | `docs/WES.md`, HelixTest `wes.rs` |
| Split-port DRS/WES probes | `{origin}/objects/test-object-1`, `{origin}/service-info` in addition to `/ga4gh/…/v1` |
| `supported_wes_versions` must contain `1.0` **or** `1.1` | HelixTest `wes.rs` extra after official ServiceInfo schema |
| Checksums on by default | Helix `GENERIC.capabilities.strict_drs_checksums = true` (HelixTest generic.toml defaults off; Helix docs say ga4gh-drs-style) |
| Dual HTTP stacks | Helix client vs HelixTest `HttpClient` for executed checks |

---

## PART 3 — HelixTest

### Pin and imports

**FACT:**

| Item | Value |
|------|--------|
| Tag | `v0.1.3` |
| SHA | `1832c043e1679ec283cb2113510ee33684317cce` |
| Cargo | `path = "../HelixTest/helixtest/crates/{common,framework}"` |
| CI checkout | same SHA hardcoded in `.github/workflows/ci.yml` |

**FACT:** Helix production imports from HelixTest:

- `common::config` (`TestConfig`, `ServiceConfig`, …)
- `common::http::HttpClient` (DRS/WES execution)
- `common::report` (`ServiceReport`, `TestCaseResult`, `OverallReport` for security)
- `common::auth::build_jwt` (security dummy tokens)
- `framework::drs::run_drs_checks`
- `framework::wes::run_wes_checks`
- `framework::{Features, Mode}` (`Mode::Generic` only)

**FACT:** Helix does **not** call `framework::{tes,trs,beacon,htsget,auth}` from `src/`.

### Execution boundary

**FACT:** `HelixTestAdapter` is the only conformance execution call site. It builds a `TestConfig` with a single service URL (discovered base), `Mode::Generic`, and `Features` from the Helix profile. Results are `ServiceReport` → `translate_service_report` → `VerificationResult`. HelixTest `passed` boolean is ignored (`src/adapter/translate.rs`). Skip is never pass.

**FACT:** HelixTest pin is copied onto `VerificationRun.helixtest_version` / `helixtest_sha`.

### Test identity mapping

**FACT:** `CheckSpec.helixtest_names` are exact HelixTest `TestCaseResult.name` strings (`src/identity.rs`). Changing an assigned Helix `id`/`code` is a compatibility change ([TEST_IDENTITY.md](TEST_IDENTITY.md)).

Unmapped HelixTest names become `helixtest.unmapped` / `UNMAPPED`.

### Ferrum dependency (HelixTest as used by Helix)

**FACT:** Helix’s adapter does not use HelixTest Ferrum mode or `profiles/ferrum.toml` loader. Scatter/gather is a `Features` bit, not `Mode::Ferrum`.

**INFERRED:** HelixTest crates still contain Ferrum-oriented tests and profiles; Helix does not invoke them.

### Helix-specific fixtures

**FACT:** Helix duplicates an in-process DRS mock (`tests/support/mock_ga4gh_drs.rs`) so prove does not need HelixTest’s testing mock at runtime. WES mock is Helix-owned (`mock_ga4gh_wes.rs`). Comments state duplication is intentional. Drift vs HelixTest B1 is a pin-bump risk.

### Classification

HelixTest **as consumed by Helix today** is **D) mixture**:

| Role | Why |
|------|-----|
| **A) verification engine** | `validate_drs_object` / `validate_wes_service_info` against vendored official OpenAPI (`common::ga4gh_schemas`). |
| **B) test library** | Helix calls `run_drs_checks` / `run_wes_checks` as functions, not the `helixtest` CLI. |
| **C) implementation-specific suite** | Hard-coded `test-object-1`, TRS workflow URLs, echo output key, scatter-gather, `supported_wes_versions` ∈ {1.0, 1.1}. |

It is **not** a pure GA4GH clause runner. Helix does not replace that mixture; it wraps it.

---

## PART 4 — GA4GH standards provenance

### What Helix itself contains

**FACT:** Helix has **no** vendored GA4GH OpenAPI, **no** `standards/registry.yaml`, **no** standard version CLI flag, **no** machine-readable map from Helix `id` → OpenAPI `operationId` / JSON Schema `$id`.

**FACT:** Helix docs cite GA4GH DRS/WES **sites** ([EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md): `ga4gh.github.io/data-repository-service-schemas/`, `…/workflow-execution-service-schemas/`). Those URLs are **not** pinned to a tag or commit in Helix.

**FACT:** Schema validation for executed DRS/WES **schema** checks happens inside **HelixTest** using files under `HelixTest/helixtest/schemas/ga4gh/` as described in that repo’s `schemas/ga4gh/README.md` and `common/src/ga4gh_schemas.rs` comments.

### HelixTest-vendored artifacts (pin v0.1.3 sources as present on disk)

| File | `info.version` in file / README claim | README source URL | Commit of that URL | Integrity hash |
|------|----------------------------------------|-------------------|--------------------|----------------|
| `drs-openapi.yaml` | **1.4.0** (YAML `info.version`) | `https://ga4gh.github.io/data-repository-service-schemas/preview/release/drs-1.4.0/openapi.yaml` | **UNKNOWN** (not recorded) | **UNKNOWN** |
| `wes-openapi.yaml` | **1.1.0** | `https://ga4gh.github.io/workflow-execution-service-schemas/openapi.yaml` | **UNKNOWN** | **UNKNOWN** |
| `tes-openapi.yaml` | README: TES 1.1.0 | GitHub `ga4gh/task-execution-schemas` (no tag in README) | **UNKNOWN** | **UNKNOWN** |
| `trs-openapi.yaml` | README: TRS 2.1.0 **develop** | `ga4gh/tool-registry-service-schemas` develop | **UNKNOWN** | **UNKNOWN** |
| `htsget-openapi.yaml` | README: 1.3.0 | `samtools/hts-specs` `pub/htsget-openapi.yaml` | **UNKNOWN** | **UNKNOWN** |
| `beacon-boolean-response.json` | README: Beacon v2, inlined draft-07 | `ga4gh-beacon/beacon-v2` `main` path | **UNKNOWN** | **UNKNOWN** |

**FACT:** TES/TRS/htsget/Beacon schema files exist in HelixTest but Helix **does not execute** those suites.

### Currently executed verification tests

Traceability:

- **Explicit in Helix source?** Only HelixTest **name** strings and human titles (“matches OpenAPI”).
- **Machine-readable in Helix?** No (no `standard`, `spec_path`, `operationId` fields on `VerificationResult`).
- **Independent reviewer path?** Clone HelixTest pin → `framework/src/drs.rs` or `wes.rs` → maybe `ga4gh_schemas` → vendored YAML. YAML **git commit of GA4GH** is not in-tree.

#### Table — executed `helix verify` checks

| Test ID | Service | Standard | Version | GA4GH repository | Ref/tag | Commit | Source file (engine) | Normative requirement | Traceability | Confidence |
|---------|---------|----------|---------|------------------|---------|--------|----------------------|----------------------|--------------|------------|
| `drs.object.reachable` | DRS | Data Repository Service (HTTP GET object) | **UNVERIFIED** as a clause | `ga4gh/data-repository-service-schemas` (via HelixTest README) | README: release/drs-**1.4.0** preview URL | **UNKNOWN** | HelixTest `framework/src/drs.rs` level0 | **UNVERIFIED STANDARD PROVENANCE** (reachable HTTP; no operationId in Helix) | Name wrap only | Low |
| `drs.object.schema` | DRS | DRS OpenAPI `DrsObject` | **1.4.0** in vendored YAML | same | `preview/release/drs-1.4.0/openapi.yaml` (README) | **UNKNOWN** | `ga4gh_schemas::validate_drs_object` + `validate_basic_drs_object` (expected id, non-empty `access_methods`) | Schema = compiled `components.schemas.DrsObject`. Extras (fixture id, access_methods non-empty) are **HelixTest**, not cited as a numbered MUST in Helix | Partial (schema call is explicit in HelixTest; extras and GA4GH git commit are not) | Medium on “uses 1.4.0 DrsObject schema”; Low on “this is DRS §X.Y” |
| `drs.object.checksum` | DRS | Checksums on `DrsObject` + byte download | 1.4.0 schema has `checksums`; test logic is HelixTest | same | same | **UNKNOWN** | `drs.rs` `level2_checksum_correctness` | **UNVERIFIED STANDARD PROVENANCE** (no clause id; requires `strict_drs_checksums`) | HelixTest extra on fixture `test-object-1` | Low |
| `drs.object.range` | DRS | HTTP Range on access URL | Vendored `drs-openapi.yaml` has **no** `Range` / `206` string (grep) | same | same | **UNKNOWN** | `drs.rs` Range `bytes=0-1023`, expect 206 + Content-Range | **UNVERIFIED STANDARD PROVENANCE** | Not in vendored OpenAPI text | Low |
| `drs.object.not_found` | DRS | GetObject **404** exists in OpenAPI (`operationId: GetObject`, `'404': 404NotFoundDrsObject`) | 1.4.0 YAML | same | same | **UNKNOWN** | `drs.rs` `level5_invalid_id_404` | HelixTest asserts status **404** for a **fixed unknown id**. OpenAPI documents 404 for GetObject. Link **operationId ↔ test** is **not** in Helix or in `drs.rs` comments | Partial | Medium that 404 is in the 1.4.0 file; Low that this fixture id is a normative MUST |
| `wes.service_info.reachable` | WES | WES HTTP GET `/service-info` | **UNVERIFIED** clause | `ga4gh/workflow-execution-service-schemas` (README) | WES **1.1.0** YAML | **UNKNOWN** | `wes.rs` level0 | **UNVERIFIED STANDARD PROVENANCE** | Name wrap | Low |
| `wes.service_info.schema` | WES | WES OpenAPI `ServiceInfo` **plus** HelixTest extra | **1.1.0** YAML; extra requires `supported_wes_versions` contain **`1.0` or `1.1`** | same | `openapi.yaml` (unpinned live URL in README) | **UNKNOWN** | `wes.rs` `validate_wes_service_info` + extra | Schema = official ServiceInfo. Extra is **HelixTest policy**, not found as a quoted MUST in Helix | Partial | Medium on schema file version; Low on extra |
| `wes.run.lifecycle_success` | WES | Run submit/poll/log | 1.1.0 API exists; **workflow is a HelixTest fixture** | same | same | **UNKNOWN** | `wes.rs` `trs://test-tool/echo/1.0`, CWL v1.2, `outputs.echo_out == hello-ga4gh` | **UNVERIFIED STANDARD PROVENANCE** (not a WES-required workflow) | Fixture contract ([WES.md](WES.md)) | Low as “GA4GH WES MUST”; High as “HelixTest echo fixture” |
| `wes.run.failure_state` | WES | Run terminal error states | same | same | same | **UNKNOWN** | `trs://test-tool/fail/1.0` → EXECUTOR_ERROR or SYSTEM_ERROR | **UNVERIFIED STANDARD PROVENANCE** | Fixture | Low |
| `wes.run.missing_inputs` | WES | same | same | same | same | **UNKNOWN** | `trs://test-tool/cwl-echo/1.0` empty params | **UNVERIFIED STANDARD PROVENANCE** | Fixture | Low |
| `wes.run.incompatible_type` | WES | `workflow_type` handling | same | same | same | **UNKNOWN** | HelixTest incompatible type case | **UNVERIFIED STANDARD PROVENANCE** | Fixture | Low |
| `wes.run.invalid_workflow` | WES | invalid `workflow_url` | same | same | same | **UNKNOWN** | `trs://nonexistent/invalid/0.0` | **UNVERIFIED STANDARD PROVENANCE** | Fixture | Low |
| `wes.run.scatter_gather` | WES | **Not** a WES-required workflow ([EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md) optional table) | n/a as WES MUST | n/a | n/a | n/a | `trs://test-tool/scatter-gather/1.0` | **UNVERIFIED STANDARD PROVENANCE** (explicitly optional / profile) | Fixture | n/a as standard; High as Helix profile optional |

**FACT:** A reviewer **cannot** independently trace Helix `id` → GA4GH git commit from the Helix repository alone.

**FACT:** Helix `helix security` / `helix bench` / discovery probes are **not** rows in this table (not GA4GH suite execution). Crypt4GH layout is not a GA4GH DRS OpenAPI test.

---

## PART 5 — Standards versioning

### What exists today

| Capability | Present? | Evidence |
|------------|----------|----------|
| Explicit version selection (`--standard` / `--version`) | **No** | `VerifyArgs`: endpoint, profile, format only |
| Automatic version detection used to **select tests** | **No** | `ServiceInfoSnapshot.version` / `type_version` are copied from 2xx JSON if present (`src/discover.rs`) and printed; **not** used in `verify.rs` |
| Version inference from URL (`/ga4gh/wes/v1` → 1.0) | **No** (explicitly rejected) | `does_not_invent_version_from_url` test |
| Multiple versions simultaneously | **No** | HelixTest compiles **one** `DrsObject` / **one** WES `ServiceInfo` schema (`OnceCell`) |
| Version-specific test suites | **No** | Single `run_drs_checks` / `run_wes_checks` |
| Compatibility testing (N vs N-1) | **No** | Not in CLI or CI |

**FACT:** WES schema check additionally requires `supported_wes_versions` to include `1.0` **or** `1.1` while the vendored OpenAPI file is labeled **1.1.0**. That is not a version selector; it is a fixed extra assertion.

### Proposed modes vs current architecture

**MODE A** — `helix verify TARGET --standard drs --version 1.5.0`

- **Cannot** be satisfied today. There is no 1.5.0 artifact, no registry, no CLI flag, no suite keyed by version.
- Required: standards registry + schema/test pack per version + CLI + JSON fields for selected vs tested version + fail-closed if pack missing.

**MODE B** — `helix verify TARGET` auto-detect declared/detectable version

- Discovery **already records** `version` / `type.version` when service-info JSON has them.
- **Does not** choose tests from that snapshot. Auto-detect would be a new policy (what if missing, conflicting, or 1.2 vs 1.4?).
- Risk: silent wrong suite if detection is trusted without a registry match.

**MODE C** — `--all-supported-versions`

- Requires Mode A packs for each supported official release, plus a result model that is a **list** of versioned runs (today: one `VerificationRun`).
- Not supportable without schema/registry work and a compare story per version.

**Architectural changes required (not implemented here):**

1. Separate **standard packs** from HelixTest’s single compiled schema.
2. Stop treating HelixTest extras (fixtures, 1.0-or-1.1) as “the standard.”
3. Record selected / detected / tested versions on `VerificationRun` (Part 8).
4. Decide whether HelixTest remains the engine (versioned feature flags) or Helix owns schema validation.

---

## PART 6 — Standard release classes

**FACT:** Helix does **not** distinguish official / ballot / snapshot / development.

**FACT:** HelixTest TRS README line says the TRS OpenAPI was taken from **develop** — that is a **development branch** vendor, unused by Helix verify today.

**FACT:** DRS path in HelixTest README includes `preview/release/drs-1.4.0` (release preview URL, not a git SHA).

**Recommended policy (evaluate only, do not implement):**

| Class | Default in `helix verify` | How requested |
|-------|---------------------------|---------------|
| **OFFICIAL** | Yes, if a pack exists in the registry | Default |
| **BALLOT** | No | Explicit `--release-class ballot` (name TBD) |
| **SNAPSHOT** | No | Explicit |
| **DEVELOPMENT** | Never automatic | Never on default; refuse or require a loud flag |

**INFERRED:** Adopting this without a registry would be theatre. TRS-from-develop in HelixTest is a cautionary example.

---

## PART 7 — Proposed standards registry

**Need:** **Yes**, if Helix will ever say “verified against GA4GH DRS x.y.z”. **FACT:** that statement is not honest with current JSON.

**Where:** Helix repository `standards/registry.yaml` (metadata). Engine code can stay HelixTest until D1 revisit.

| Question | Recommendation |
|----------|----------------|
| Belong in Helix? | **Yes** (Helix owns claims). HelixTest may keep copies for its CLI. |
| Commit generated artifacts? | **Metadata + integrity hash** required. Full OpenAPI **may** be vendored for offline prove. |
| Vendor specs? | Prefer **vendored files + hash** so prove stays offline. Do not fetch GA4GH at runtime. |
| Metadata only? | Insufficient for prove unless CI fetches and checks hash (network + non-determinism). |
| How to update | PR: new registry row, new file or hash, tests against fixture **and** documented delta. Pin HelixTest if engine still validates. |
| Helix versioning | New official standard pack is a **Helix minor** (new tests) or **patch** (hash refresh of same tests). Changing an assigned Helix `id` remains a compatibility break. Fixture catalog `helix-fixtures-v1` stays separate. |

**Do not** put HELIOS signing in the registry.

---

## PART 8 — Five version concepts (ambiguity today)

| # | Concept | Today |
|---|---------|--------|
| 1 | Claimed by target | Optionally copied from service-info `version` / `type.version`. Not a first-class field on `VerificationRun`. Unused for suite selection. |
| 2 | Detected by Helix | Same snapshot. Helix **does not infer** from URL. Empty if no 2xx service-info JSON. |
| 3 | Selected by user | **Does not exist.** Only `--profile generic\|ferrum`. |
| 4 | Actually tested | Implicit: whatever one HelixTest pin + one compiled schema does. Not named in JSON. |
| 5 | Authoritative pack Helix has | Implicit: HelixTest vendored DRS 1.4.0 / WES 1.1.0 **plus extras**. Not listed on the run. |

**FACT:** A green `helix verify` can occur when the target’s `type.version` is `1.2.0` (discovery test fixture uses that) while schema validation uses **1.4.0** DrsObject. Those numbers are not compared.

That is the core credibility gap.

---

## PART 9 — Result trustworthiness

**Could Helix honestly say “Verified against GA4GH DRS 1.5.0” today?**

**FACT: No.** There is no DRS 1.5.0 pack. Verify JSON has no standard version. Several DRS checks are fixture/HTTP extras, not traced clauses. Run identity records Helix/HelixTest/fixture/schema **Helix** versions, not GA4GH.

### Evidence required before that sentence is allowed

Minimum machine-readable fields on the run (names illustrative):

| Evidence | Today |
|----------|--------|
| Pinned standard source (repo + path) | Missing in Helix JSON |
| Standard version (e.g. 1.4.0, not 1.5.0 unless pack exists) | Missing |
| Exact release / git ref of the spec | Missing (HelixTest README URL only) |
| Integrity hash of the spec file | Missing |
| Test suite version (HelixTest pin **and** Helix catalog) | Partial: `helixtest_version` / `sha`, `helix_version` |
| Test IDs actually executed | Partial: `executed[].id` / `skipped[].id` |
| Target identifier | `target.url` |
| Timestamp | `timestamp` (wall clock, not a signature) |
| Fixture version | `fixture_version` (`helix-fixtures-v1`) |
| Helix version | `helix_version` |
| Schema of the **report** | `schema_version` `helix-verification-v1` |
| Selected vs detected vs tested standard version | Missing (must be three fields) |
| Which checks are “schema/official” vs “fixture extra” | Missing |

**FACT:** Signing, RO-Crate, and PDF are **not** required for this sentence; they are HELIOS. A timestamp is not a signature ([RUN_IDENTITY.md](RUN_IDENTITY.md), [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

Until the table is filled, honest language is: **“HelixTest v0.1.3 DRS/WES checks (including OpenAPI DrsObject 1.4.0 / WES ServiceInfo 1.1.0 as vendored in that pin) against this target, plus HelixTest fixture assertions.”** Even that needs the YAML hash to be strong.

---

## PART 10 — Red flags (before public release)

### P0 — correctness or credibility

1. **No standards provenance in Helix.** Public “GA4GH verify” will be read as certification-adjacent. Current JSON cannot name the spec version tested.
2. **Detected target version ≠ schema used.** Discovery can record 1.2.0 while validating 1.4.0 DrsObject with no warning.
3. **Fixture extras indistinguishable from spec.** Echo workflow, scatter-gather, Range (not in vendored OpenAPI text), checksum download, `supported_wes_versions` 1.0-or-1.1.
4. **DRS 1.5.0 / “latest” claims would be false.** Do not use them.
5. **Dual HTTP clients.** HelixTest `HttpClient` still follows its own redirect/timeout/gzip policy on the actual DRS/WES checks ([THREAT_MODEL.md](THREAT_MODEL.md)).
6. **Stale `DEPENDENCY.md`.** Contradicts `Cargo.lock` on a public repo.

### P1 — architecture

1. HelixTest is a **mixture** engine (Part 3). Helix cannot version standards without splitting “OpenAPI validate” from “fixture scenarios.”
2. `--profile ferrum` names a vendor in the CLI of an implementation-neutral tool.
3. `helix security` JSON ≠ `VerificationRun`.
4. CI HelixTest SHA duplicated vs `VERSIONS.lock`.
5. Clippy not `--locked`.
6. Local HelixTest HEAD can diverge (`require-helixtest.sh` warns, does not fail).
7. No CoC / issue templates ([OPEN_SOURCE_RELEASE_CHECKLIST.md](OPEN_SOURCE_RELEASE_CHECKLIST.md)).

### P2 — useful

1. Compare/bench JSON have no frozen JSON Schema.
2. `ENGINEERING_AUDIT.md` is a same-day snapshot and is already stale in places (treat as historical).
3. Panic on unknown catalog `spec()` / discovery `.expect`.
4. Range check should be labeled HTTP/fixture until a spec clause is cited **from a pinned file**.

### P3 — later

1. TES/TRS/htsget/Beacon execution.
2. Mode A/B/C CLI.
3. Release binaries / `publish = false`.
4. Coverage job.
5. Ballot/snapshot classes.

---

## PART 11 — Next engineering steps (≤ 15)

Priorities: correctness, provenance, versioning, independence, determinism, security, comparable results, evaluator usability. No novelty features.

1. **Fix `docs/DEPENDENCY.md`** so it matches `Cargo.lock` and path+SHA HelixTest pinning.
2. **Add report fields (or a sibling object) for:** detected `type.version` / `version`, HelixTest schema file ids (DRS 1.4.0, WES 1.1.0), and a boolean/enum **fixture_extra vs schema** per check. Do not claim 1.5.0.
3. **Record integrity hashes** of the HelixTest-vendored OpenAPI files used at the pin (even if the files stay in HelixTest).
4. **Label Range, checksum-download, WES workflows, WES 1.0-or-1.1 extra** as HelixTest/fixture in `TEST_IDENTITY.md` and JSON.
5. **Fail or warn** when discovered `type.version` is present and ≠ the schema pack version (policy to decide; default: warn in compare identity, not silent).
6. **Do not implement `--standard/--version` until a registry row exists** for that version.
7. **Draft `standards/registry.yaml` (metadata only) for DRS 1.4.0 and WES 1.1.0** pointing at the pin’s files + hashes. Official class only.
8. **Keep HelixTest as engine (D1)** but treat `ga4gh_schemas` as the only “standard” path; fixtures stay documented as fixtures.
9. **Unify or isolate HTTP:** either run DRS/WES through Helix `http_safety` or document HelixTest client as in-scope residual with tests.
10. **Rename or document `--profile ferrum`** as “target expects DRS+WES+scatter fixture,” not Ferrum SDK.
11. **OSS P0 from the release checklist:** CoC, issue templates, `publish = false` — still not a tag.
12. **CI:** clippy `--locked`; HelixTest `ref` from `VERSIONS.lock`; `permissions: contents: read`.
13. **Evaluator pack:** one paragraph: results are HelixTest pin + fixture catalog, not “DRS 1.5.0 certified.”
14. **Refuse HELIOS** features if they appear in PRs (already gated).
15. **Only then** revisit Mode A as a CLI on top of the registry — not before hashes and version fields exist.

### Independent reviewer: what is required before first public release?

If reviewing Helix as an independent technical reviewer, I would **not** allow a public “GA4GH verification CLI” release until:

1. `main` matches the product (this commit is a start) **and** CI is green on GitHub for that commit.
2. The product **cannot** be quoted as “verified against GA4GH DRS 1.5.0” (no such pack; JSON must not imply it).
3. A green verify run **names** the spec files and versions actually used (1.4.0 DrsObject / 1.1.0 ServiceInfo at this pin) **and** which checks are fixture extras.
4. Detected target version is **not silently ignored**.
5. Open-source basics exist (CoC, security contact, honest DEPENDENCY.md).
6. HelixTest pin SHA is the only engine, path-dep documented, no Ferrum crate.
7. HELIOS remains out of scope.
8. Language stays: technical signal, not certification, no Ferrum clinical pilot.

I would **allow** an **early-access / internal CI** description of: “Helix wraps HelixTest v0.1.3 DRS and WES checks, including vendored DRS 1.4.0 / WES 1.1.0 schema validation plus documented fixtures.” I would **not** allow a standards-conformance product announcement until Parts 4–9 are closed.

---

## What this audit did not do

- Did not implement registry, CLI flags, or code changes.
- Did not fetch GA4GH GitHub to verify YAML bytes against upstream.
- Did not re-run `make prove` in this review (commit `3ca8b8c` hook did).
- Did not treat HelixTest TRS-from-develop as a Helix verify fact (unused).
- Did not invent DRS 1.5.0 support.
