# Helix engineering audit

**Date:** 2026-09-04
**Scope:** `SynapticFour/Helix` as checked out next to `SynapticFour/HelixTest`.
**Method:** Read README, INVENTORY, `docs/*`, `Cargo.toml`, `VERSIONS.lock`, `src/`, `tests/`, CI workflows, Makefile, scripts, and HelixTest crate APIs that Helix actually calls. No source was modified. This audit did not re-run `cargo test`, clippy, or a live Ferrum stack.

## How to read this file

Every claim is tagged:

| Tag | Meaning |
|-----|---------|
| **FACT** | Observed in this repository, or in the HelixTest crates Helix path-depends on, as they exist on disk for this audit. |
| **INFERRED** | Follows from facts but was not independently executed or measured here. |
| **UNKNOWN** | Not determined. Do not treat as true. |

Do not treat this document as certification, a clinical claim, or a HelixTest absorption plan.

## Verified product baseline (must stay true)

**FACT:** Helix is HelixTest becoming a standalone VERIFY CLI, not a new test platform. HelixTest already exists as one of five public GA4GH-stack repos (Ferrum, ga4gh-infra, Lab Kit, Demo, HelixTest), is already in CI, and is cited in SF-TR-2026-001 and SF-TR-2026-002 (`docs/HELIX_VISION.md`, `docs/IDENTITY.md`, README).

**FACT:** Helix scope in the docs is conformance, security behaviour, and benchmark/regression. Reproducibility, signed audit trails, RO-Crate, and PDF export stay in HELIOS (`docs/HELIX_VS_HELIOS.md`, D4 in `docs/DECISIONS.md`).

**FACT:** HELIOS is a separate brand (`helios-audit` on PyPI; Apache-2.0; Early Access in `docs/HELIX_VISION.md`). Helix must not implement HELIOS surfaces.

**FACT:** Ferrum is BUSL-1.1, on-premise, Rust, tested, and has **no** real clinical pilot (DIZ / genomDE) (`docs/IDENTITY.md`, README). Demos and CI are not a pilot and not production.

**FACT:** HelixTest / Helix results are not GA4GH certification (README, `docs/FOR-EVALUATORS.md`, `docs/CLI_CONTRACT.md`).

**FACT:** Capacity is described as single-steward. Roadmap stages are scope exits, not calendar dates (`docs/HELIX_ROADMAP.md`).

**FACT:** Decision D1: HelixTest stays its own git root. Helix path-depends on sibling crates; it does not vendor HelixTest (`docs/DECISIONS.md`, `Cargo.toml`).

**FACT:** README contains the honesty sentence that `scripts/prove.sh` greps: *Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency.*

---

## 1. Current architecture

**FACT:** This repo is a single Cargo package `helix` version `0.1.0`, edition 2021, `rust-version = "1.88"`, license Apache-2.0 (`Cargo.toml`).

**FACT:** Binary `helix` is `src/main.rs`. Library crate `helix` is `src/lib.rs` with modules `bench`, `discover`, `report`, `security`, `verify`.

**FACT:** Helix is a client you point at a URL. It does not start Ferrum, Docker, or any GA4GH server (`docs/INSTALL.md`, `docs/ECOSYSTEM.md`, Makefile comments).

**FACT:** Runtime HTTP for discovery, security probes, and bench uses Helix’s own `reqwest` client (`src/discover.rs` `http_client`: 5s request timeout, 3s connect timeout). DRS conformance checks use HelixTest `common::http::HttpClient::new()` (`src/verify.rs`).

**FACT:** HelixTest `HttpClient::new()` uses a 30s request timeout, 5s connect timeout, and GET retries (`HelixTest/helixtest/crates/common/src/http.rs`). Discovery and DRS checks therefore do not share the same timeout/retry policy.

**FACT:** Helix always calls `framework::drs::run_drs_checks(Mode::Generic, …)`. In HelixTest `drs.rs` the `Mode` argument is named `_mode` and is unused by that function.

**FACT:** There is no Beacon / africa / infra / E2E / Age path in Helix source (`VERIFY_ORDER` in `src/discover.rs`; no `framework::{wes,tes,trs,beacon,htsget,auth}` imports).

**FACT:** JSON for `verify` and `security` is HelixTest `common::report::OverallReport` (D3). Bench JSON is Helix-owned `BenchOutcome` (`src/report.rs`, `docs/CLI_CONTRACT.md`).

**FACT:** CI clones HelixTest at a sibling path `HelixTest/` next to `Helix/` (`.github/workflows/ci.yml`). Local `cargo` resolves `../HelixTest/helixtest/crates/{common,framework}` (`Cargo.toml`).

**INFERRED:** A developer whose sibling HelixTest is not the pin SHA compiles a different HelixTest than GitHub CI. CHANGELOG Unreleased records one such local prove (HelixTest HEAD `29472d2c…` vs pin `1832c043…`).

**UNKNOWN:** Whether every clone of this workspace currently matches the pin. This audit did not `git rev-parse` HelixTest as a required step of the written baseline.

---

## 2. Current command surface

**FACT:** clap subcommands are `verify`, `security`, `bench` (`src/main.rs`). The binary name is `helix`, never `helios` (`docs/CLI_CONTRACT.md`).

**FACT:** `helix verify <endpoint> [--format|--report text|json]`. Default format is text. `--report` is a visible alias of `--format`.

**FACT:** `helix security <endpoint> [--format json] [--hmac-secret-file PATH] [--crypt4gh-file PATH]`. HMAC file default if env unset: `test-fixtures/hmac/shared-secret.txt`. Env `HELIX_HMAC_SECRET` wins over the file (`resolve_hmac_secret` in `src/main.rs`).

**FACT:** `helix bench --baseline URL --candidate URL [--baseline-label] [--candidate-label] [--threshold 10] [--warmup 1] [--repetitions 5] [--no-rss] [--format json]`. Default labels `baseline` / `candidate`. Default threshold `10.0` (`DEFAULT_THRESHOLD_PCT`). Warmup default 1, measured repetitions default 5. Threshold never fails the process.

**FACT:** There is no `helix --all`, no `--mode`, no `--only`, no compose start, no `--start-ferrum`. Those remain HelixTest CLI surfaces (`docs/CLI_CONTRACT.md`, INVENTORY HelixTest section).

**FACT:** HelixTest binary `helixtest` is not built by this package. Ferrum CI still clones tagged HelixTest (`docs/IDENTITY.md`, `docs/ECOSYSTEM.md`).

---

## 3. Current HelixTest dependency boundary

**FACT:** Direct path dependencies (`Cargo.toml`):

```text
common     = helixtest-common     ../HelixTest/helixtest/crates/common
framework  = helixtest-framework  ../HelixTest/helixtest/crates/framework
```

**FACT:** Pin file `VERSIONS.lock`: `HELIXTEST_REF=v0.1.3`, `HELIXTEST_TAG=v0.1.3`, `HELIXTEST_SHA=1832c043e1679ec283cb2113510ee33684317cce`.

**FACT:** GitHub CI checks out `SynapticFour/HelixTest` at that same SHA, hardcoded in `.github/workflows/ci.yml` (not read from `VERSIONS.lock` at runtime). `scripts/prove.sh` greps `HELIXTEST_SHA=1832c043…` in `VERSIONS.lock`.

**FACT:** Helix **calls** from HelixTest:

- `framework::drs::run_drs_checks`
- `framework::{Features, Mode}`
- `common::config::{TestConfig, ServiceConfig, SubsetConfig, AuthChecksConfig}`
- `common::http::HttpClient`
- `common::report::{OverallReport, ServiceReport, TestCaseResult, TestStatus, ServiceKind, SkippedService, ComplianceLevel, TestCategory}`
- `common::auth::build_jwt` (Stage 3 JWTs)
- `common::util::sha256_bytes` (B1 mock in tests)

**FACT:** Helix does **not** call HelixTest `run_all`, WES/TES/TRS/Beacon/htsget/auth/africa/infra modules, or the `helixtest` binary.

**FACT:** HelixTest DRS object id is hard-coded `test-object-1` (`HelixTest/.../drs.rs`; Helix discovery probes the same id).

**FACT:** Integration mock `tests/support/mock_ga4gh_drs.rs` is an in-tree **copy** of HelixTest B1 so CI can compile against the published pin without HelixTest’s `helixtest/testing/mock_ga4gh_drs.rs`. Comment in that file states the duplication is intentional.

**FACT:** D1 revisit notes (2026-09-04) in `docs/DECISIONS.md` keep HelixTest separate after helix-action, Stage 3, and Stage 4.

**FACT:** `docs/DEPENDENCY.md` still says “Until this repo contains a Rust lockfile” and “Docs-only `make prove`”. `Cargo.lock` exists in this repo. `make prove` runs `scripts/prove.sh` then `cargo test` (`Makefile`). Those DEPENDENCY.md sentences are stale relative to the tree.

---

## 4. Current GA4GH service discovery behavior

**FACT:** Order is DRS → WES → TES → TRS → htsget (`VERIFY_ORDER`). Beacon, africa, and infra are not probed.

**FACT:** Endpoint must parse as `http` or `https` with a host; trailing slash is stripped (`normalize_endpoint`).

**FACT:** A probe counts as “API present” on HTTP **2xx or 401 or 403**. Network error and 404 do not (`status_means_api`). First matching candidate wins (`first_present`).

**FACT:** DRS probe order:

1. `{origin}/ga4gh/drs/v1/objects/test-object-1` → base `{origin}/ga4gh/drs/v1`
2. `{origin}/ga4gh/drs/v1/service-info` → same gateway base
3. `{origin}/objects/test-object-1` → base `{origin}` (split-port)

Gateway object wins over split if both exist (unit test `discovers_gateway_drs_before_split`).

**FACT:** WES: `{origin}/ga4gh/wes/v1/service-info` then `{origin}/service-info` (split). TES: `/ga4gh/tes/v1/service-info` then `/ga4gh/tes/v1/tasks` (no split-port TES). TRS: `/ga4gh/trs/v2/service-info` then `/ga4gh/trs/v2/tools`. htsget: only `/ga4gh/htsget/v1/reads/service-info`.

**FACT:** Discovery results are printed in text mode. JSON `OverallReport` has **no** `discovery` key (`src/report.rs` test `json_shape_is_helixtest_overall_report`). Discovered but unwired services go to `skipped_services` with reason Stage 1 DRS-first.

**FACT:** B1 mock mounts a WES-shaped `{origin}/service-info` named `"Ferrum Gateway"` so generic DRS mode must not auto-switch to Ferrum (`tests/support/mock_ga4gh_drs.rs`). That split WES probe can mark WES “found” on the mock even though no WES checks run.

**INFERRED:** A host that returns 401/403 on a GA4GH path is treated as present even if the body is an HTML login page. No body/schema check happens at discovery time.

**UNKNOWN:** Behaviour against a live Ferrum `make up` stack was not executed in this audit. README shows an example of five services `found`; that example is documentation, not a captured log from this audit.

---

## 5. Current DRS verification behavior

**FACT:** If DRS is discovered, Helix runs `run_drs_checks` with `Features { strict_drs_checksums: true, ..Default }` and `TestConfig` with only `drs_url` set (`src/verify.rs`).

**FACT:** The five executed names (`DRS_CHECK_NAMES`) match HelixTest DRS and INVENTORY.md:

1. DRS object endpoint reachable
2. DRS DrsObject OpenAPI + access_methods
3. DRS checksum correctness
4. DRS HTTP Range support
5. DRS invalid object id returns 404

**FACT:** With `strict_drs_checksums = true`, HelixTest does not skip the checksum test (`framework/src/drs.rs`).

**FACT:** If DRS is not discovered, Helix injects one synthetic FAIL on name (1) with message that DRS was not discovered. `has_failures()` is then true → process exit 1 (`src/verify.rs`, `src/main.rs`).

**FACT:** WES/TES/TRS/htsget HelixTest checks are **not** called. Discovery listing is not a pass (`src/report.rs` `overall_report`).

**FACT:** Stage 1 **exit** in `docs/HELIX_ROADMAP.md` / `docs/CLI_CONTRACT.md` still requires DRS **and WES** against Ferrum local. That exit is **not** met by current code.

**FACT:** HelixTest `run_drs_checks` GETs `{drs_url}/objects/test-object-1` (HelixTest `drs.rs`). Helix passes the discovered `base_url`, which is either `{origin}/ga4gh/drs/v1` or the origin itself for split-port.

**UNKNOWN:** Exact OpenAPI assertions inside HelixTest level-1 DRS (field set, access_methods) beyond what INVENTORY.md lists. This audit did not re-read every helper in `drs.rs`.

---

## 6. Current security behavior

**FACT:** `helix security` is Stage 3 **started, not exited** (`docs/HELIX_ROADMAP.md`). Fixtures are labeled NICHT FÜR PRODUKTION (`test-fixtures/README.md`). Gitleaks allowlists `test-fixtures/` (`.gitleaks.toml`).

**FACT:** Five HTTP case names (`AUTH_CASE_NAMES`) hit the discovered DRS `{base}/objects/test-object-1` with dummy HS256 JWTs from HelixTest `common::auth::build_jwt`:

| Case | Expectation in Helix |
|------|----------------------|
| valid token grants access | HTTP 2xx |
| expired token rejected | HTTP 401 |
| wrong scope denied | HTTP 403 or 401 |
| invalid/manipulated token | flipped signature **and** garbage Bearer both 401 |
| token for another service | WES audience on DRS → 403 or 401 |

**FACT:** Issuer/subject used when minting: `https://helix.test.invalid` / `helix-stage3-fixture-user` (`src/security/jwt.rs`).

**FACT:** `classify_bearer` is used by **Helix unit-test mocks**, not as the live target’s verifier (`src/security/mod.rs` test `AuthGate`).

**FACT:** No HMAC secret → all five auth cases **Skip**, not Pass. Crypt4GH header case still runs. `has_failures()` is false if only skips + crypt4gh pass → exit 0.

**FACT:** Secret present but DRS not discovered → **one** FAIL on the first auth case name only (not five skips and not five fails).

**FACT:** Crypt4GH in Helix is protocol framing only (`HLX-AUTH-050` well-formed fixture, `HLX-AUTH-053` invalid envelopes rejected, `HLX-AUTH-054` HTTP body if magic present else skip). Default 050 bytes: `test-fixtures/crypt4gh/well-formed.c4gh` (or `--crypt4gh-file`). No `crypt4gh` crate in Helix `Cargo.toml`, no decrypt, no private keys (`src/security/crypt4gh_header.rs`). A pass is not “secure”. HelixTest secret-key HTTP (`HLX-AUTH-051`–`052`) is not wired.

**FACT:** This is not Passport/OIDC, not ga4gh-infra JWKS, not Ferrum `ferrum+infra`, not HelixTest `framework/src/auth.rs` ladder, not HELIOS evidence.

**FACT:** Roadmap Stage 3 exit still wants five documented cases reproducible against Ferrum `make up-pilot-local` / HMAC-on path. CI default is in-process mock + dummy HMAC. Stage 3 is not exited.

**UNKNOWN:** Whether a live Ferrum HMAC-on DRS accepts these dummy JWTs (issuer/audience/scope layout). Not executed here.

---

## 7. Current benchmark behavior

**FACT:** `helix bench` is Stage 4 **started, not exited** (`docs/HELIX_ROADMAP.md`, [BENCHMARKS.md](BENCHMARKS.md)). Workload id **`http.drs.smoke.v1`** (`src/bench/workload.rs`):

1. `/health`
2. `/ga4gh/drs/v1/service-info`
3. `/ga4gh/drs/v1/objects/test-object-1`

**FACT:** Engine: discarded warmup runs, then measured repetitions. Analysis compares **distributions** (median, p95 when both n≥20, error rate, median per-run RSS when both recorded it). Single-run series are measurement only. `analysis.warning` is inspect-threshold on a comparable distribution compare. `analysis.regression` is median-worse only. `analysis.verification_failure` is always false. `warning: true` on the outcome does **not** change process exit. Different OS/arch/Helix version/workload/timeouts → `environment.comparable: false`; threshold `worse` does not fire. Sample percentiles are not a significance test.

**FACT:** On non-Linux, `rss_kb` is always `None` and omitted from JSON (`skip_serializing_if`). RSS is not Ferrum RSS. `--no-rss` disables collection.

**FACT:** Bench does **not** call `discover()`. Split-port DRS that only serves `/objects/test-object-1` will error those gateway paths. Comment in `workload.rs` says paths work on Ferrum gateway and “split mocks” — the listed paths are gateway-shaped, not split `/objects/…` only.

**FACT:** Not Demo hap.py, not GIAB, not HELIOS, not two Ferrum git tags on the same runner class (Stage 4 exit unmet).

---

## 8. Current JSON contracts

**FACT (verify):** stdout is pretty-printed HelixTest `OverallReport`:

- `services` — only DRS `ServiceReport`
- `enabled_services` — `[Drs]`
- `skipped_services` — discovered WES/TES/TRS/htsget with Stage 1 reason
- `executed_test_modules` — `[Drs]`
- `diagnostics` — always `None` from Helix (field omitted when None)

**FACT (Helix `diagnostic` vs HelixTest `diagnostics`):** HelixTest `OverallReport.diagnostics` is unused by Helix. Separate from that, `helix verify` `VerificationResult` may include optional `diagnostic` on DRS/WES fail/error (`src/diagnostics.rs`, [DIAGNOSTICS.md](DIAGNOSTICS.md)). Catalog-driven. Not AI. Not a root-cause claim. Not HELIOS. A bench warning is not this field.

**FACT:** Per-test objects include at least `name`, `status` (`pass`/`fail`/`skip`), `passed` (true iff Pass), plus HelixTest fields `level`, `error`, `category`, `weight` (`common::report::TestCaseResult`).

**FACT:** There is no `discovery` object in verify JSON.

**FACT (security):** `OverallReport` with `services` Auth + Crypt4gh, `enabled_services` / `executed_test_modules` those two kinds, empty `skipped_services`.

**FACT (bench):** Helix `BenchOutcome`: `workload_id` (`http.drs.smoke.v1`), `analysis` (`measurement` / `warning` / `regression` / `verification_failure`), `environment`, `baseline` / `candidate` `Sample`, `diff`, `warnings`, `note`. Not `OverallReport`. D3 exception. Bench warning is not a verification failure.

**FACT:** Logs from HelixTest `HttpClient` may appear on stderr; JSON is stdout (`docs/CLI_CONTRACT.md`). This audit did not capture a live stderr sample.

**FACT:** No RO-Crate, PDF, signatures, or ISO/AI-Act scores in these JSON shapes (D3, D4).

---

## 9. Current exit-code behavior

**FACT:** `verify` / `security`: after printing, `std::process::exit(1)` if `has_failures()` (any `TestStatus::Fail`). Skip is not Fail.

**FACT:** Missing DRS on verify → synthetic Fail → exit 1. Missing DRS on security with a secret → one Fail → exit 1. Missing secret on security → skips → exit 0 if crypt4gh passes.

**FACT:** `bench`: successful compare always returns `Ok(())` even when `warning` is true. Unreachable/invalid URL that makes `run_bench` return `Err` bubbles from `main() -> Result<()>` .

**FACT:** `docs/CLI_CONTRACT.md` says verify/security usage/runtime errors are exit **1**, and bench usage/runtime errors are exit **1**.

**FACT:** `Cli::parse()` is clap derive. Clap usage errors terminate inside clap and never reach `has_failures()`.

**UNKNOWN:** Numeric clap usage-error code for this binary (clap 4 commonly uses 2). Not executed (`helix` with missing args) in this audit. If it is 2, CLI_CONTRACT’s “usage → 1” is inaccurate.

**INFERRED:** `anyhow` `Err` from `main` typically becomes a non-zero process status (commonly 1). Not re-measured here.

---

## 10. Current CI integration

**FACT:** Workflows: `.github/workflows/ci.yml`, `secret-scan.yml` (gitleaks-action v3, fetch-depth 0), `dependency-review.yml` (action v5, **`continue-on-error: true`**).

**FACT:** `ci.yml` on push/PR to `main`/`master` and `workflow_dispatch`: checkout Helix + HelixTest at pin SHA, rustc **1.91.1** via `dtolnay/rust-toolchain` with rustfmt+clippy, then `make prove`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`. Working directory is `Helix`.

**FACT:** `make prove` = docs greps + `cargo test`. It does **not** run clippy or rustfmt. Pre-commit `scripts/hooks/ci-check.sh` does fmt + clippy + prove (needs sibling HelixTest). CHANGELOG records a clippy failure that `make prove` alone missed.

**FACT:** CI `cargo test` / clippy are **not** invoked with `--locked` in `ci.yml` or `Makefile`.

**FACT:** `rust-toolchain.toml` pins channel `1.91.1`. `Cargo.toml` MSRV is `1.88`. CHANGELOG records a local prove on Homebrew rustc **1.97.1**.

**FACT:** Dependabot/Renovate are off by documented choice (`docs/DEPENDENCY.md`).

**FACT:** Ferrum **`main`** still runs HelixTest, not helix-action. helix-action is a **sibling repo**; Ferrum pilot branch `ci/helix-verify-pilot` only (`docs/ECOSYSTEM.md`, `docs/HELIX_ROADMAP.md`). helix-action is not part of this git tree.

**FACT:** `scripts/prove.sh` requires listed docs, honesty greps, `fn discover` / `run_drs_checks` / `overall_report` / `run_security` / `run_bench` in source, VERSIONS.lock SHA, and `NICHT FÜR PRODUKTION` in `test-fixtures/README.md`. It does not require this audit file.

---

## 11. Current test coverage

**FACT:** Count of `#[test]` / `#[tokio::test]` in Helix (excluding ignored):

| Location | Count |
|----------|-------|
| `src/discover.rs` | 9 |
| `src/report.rs` | 2 |
| `src/security/jwt.rs` | 3 |
| `src/security/crypt4gh_header.rs` | 3 |
| `src/security/mod.rs` | 2 |
| `src/bench/mod.rs` | 4 |
| `src/bench/engine.rs` | 1 |
| `tests/verify_drs.rs` | 4 |
| `tests/cli_discover.rs` | 1 |
| `tests/security_cli.rs` | 1 |
| `tests/bench_cli.rs` | 2 |
| **Total** | **30** |

**FACT:** CHANGELOG Unreleased claims a local `make prove` with 30 tests passed (2026-09-04), on rustc 1.97.1 against HelixTest HEAD not the pin.

**FACT:** Tests cover: URL normalize; discovery order and 401-as-present; split vs gateway DRS; skip-not-green; OverallReport shape without `discovery`; B1 mock DRS pass via library + CLI JSON; verify exit 1 when DRS missing; `--report` alias; security six cases + skip-without-secret; bench warn-on-worse still exit 0.

**FACT:** There is no test in this repo that starts Ferrum, Demo, or ga4gh-infra.

**FACT:** There is no test that executes HelixTest WES/TES/TRS/htsget check functions through Helix.

**UNKNOWN:** Coverage percentage (`cargo tarpaulin` / llvm-cov). Not run.

**UNKNOWN:** Whether GitHub `ci.yml` on `main` is green at the instant this file is written. Not fetched.

---

## 12. Current external dependencies

**FACT (Helix `Cargo.toml` direct):** anyhow, base64 0.21, chrono 0.4 (clock+std), clap 4 derive, hmac 0.12, reqwest 0.11 (json, rustls-tls, no default features), serde, serde_json, sha2 0.10, tokio (macros, rt-multi-thread, time), path deps `helixtest-common` and `helixtest-framework`.

**FACT (dev):** wiremock 0.6, assert_cmd 2, predicates 3.

**FACT:** `Cargo.lock` exists. Exact transitive versions live there; this audit does not enumerate the lockfile.

**FACT:** HelixTest crates pull their own graph (including `tokio-retry` on `HttpClient`). Helix inherits that when linking `framework` / `common`.

**FACT:** No Docker images, no PyPI, no HELIOS crate, no Ferrum crate.

**UNKNOWN:** License/audit of every transitive crate. Dependency-review is non-fatal.

---

## 13. Known technical debt

**FACT:** B1 DRS mock is duplicated (`tests/support/mock_ga4gh_drs.rs` vs HelixTest testing mock). Drift risk if HelixTest fixture changes and Helix pin does not, or the reverse after a pin bump.

**FACT:** Two HTTP stacks (Helix 5s/3s reqwest vs HelixTest 30s + retries). Discovery can declare DRS present; DRS checks can still time out or retry differently.

**FACT:** CI HelixTest SHA is duplicated (`.github/workflows/ci.yml` vs `VERSIONS.lock`). They match today; nothing in CI reads the lock file.

**FACT:** Local path dep is “whatever is in `../HelixTest`”; CI is the pin SHA. `scripts/hooks/ci-check.sh` only checks the sibling directory exists, not the SHA.

**FACT:** Docs drift (still in-tree):

- `INVENTORY.md` last paragraph: “Implementation of the CLI still lives in HelixTest until it is moved or vendored here.”
- `docs/FOR-EVALUATORS.md`: “This repository is not yet a runnable suite” (dated 2026-09-03).
- `docs/PROVE.md`: `make prove` described as docs-only; live examples are `helixtest`, not `helix verify`.
- `docs/DEPENDENCY.md`: “until a Rust lockfile” / docs-only prove.
- `docs/HELIX_VISION.md` §7: “Helix currently has inventory and vision, not a second implementation.”
- `docs/HELIX_VS_HELIOS.md`: table still says CLI today is HelixTest / `helix verify` is Stage 1.

README, INSTALL, CLI_CONTRACT, IDENTITY, ECOSYSTEM, ROADMAP are closer to the actual `helix` binary.

**FACT:** `make prove` ≠ GitHub CI (no clippy/fmt in prove).

**FACT:** Stage 1 JSON omits discovery; operators must use text mode to see found/missing URLs.

**FACT:** Security missing-DRS reports one FAIL instead of five skips or five fails — asymmetric with missing-secret (five skips).

**FACT:** Bench RSS measures Helix, not the target. Stage 4 docs originally wanted resource figures for a Demo/Ferrum scenario.

---

## 14. Architecture risks

**FACT:** Path dependency on a sibling git root is required to build. A Helix-only clone does not compile (`docs/INSTALL.md`, `ci-check.sh`).

**INFERRED:** Pin bump of HelixTest can break Helix if `run_drs_checks` / `OverallReport` / `build_jwt` / `TestConfig` fields change. Blast radius is Ferrum/Lab Kit/ga4gh-infra pins plus this repo (D1 text).

**FACT:** Discovery 401/403 = present can mark auth-walled junk as a GA4GH API. Subsequent DRS checks then run against that base URL.

**FACT:** WES split probe `{origin}/service-info` can collide with non-WES service-info (B1 mock does this on purpose).

**INFERRED:** Calling HelixTest WES later with only a discovered URL may still need HelixTest profile/features/env that Helix does not set today (`TestConfig` other URLs are empty strings).

**FACT:** Absorbing HelixTest into Helix is an explicit non-goal until HELIX_VISION §7 criteria (D1). Doing it “to simplify the path dep” would hit Ferrum `VERSIONS.lock` and SF-TR citations.

---

## 15. Product risks

**FACT:** Early-stage README: DRS executed; WES/TES/TRS/htsget discovered but not executed; security and bench are scaffolds.

**FACT:** Evaluators reading stale `FOR-EVALUATORS.md` / INVENTORY closing paragraph may think there is no `helix` binary, or that HelixTest is the only CLI.

**FACT:** A green `helix security` with dummy HMAC is not production hardening and not Passport certification (fixture README, ROADMAP Stage 3 “Not in this stage”).

**FACT:** A green `helix bench` warning-or-not is not a publication benchmark and not two Ferrum versions compared (ROADMAP Stage 4).

**FACT:** Ferrum has no clinical pilot. Helix must not be used as evidence of DIZ/genomDE production.

**FACT:** Team is single-steward; skipping Stage 1 exit to chase Stage 2/5 visibility is forbidden by ROADMAP (“Do not start n+1 until n has exited”). Stage 2/3/4 have **started** before Stage 1 exit — that is already a process tension recorded in ROADMAP (“Stage 1 in progress”; “Stage 2 has started as a pilot only”; “Stage 3/4 started”).

**INFERRED:** Starting later stages before Stage 1 exit increases the chance of unfinished surfaces being read as product-complete.

---

## 16. What is already production-quality

Here “production-quality” means **engineering completeness for the narrow behaviour**, not a product SKU, not certification, not clinical use.

**FACT:** Discovery probes have unit tests for normalize, order, split vs gateway, 401-as-present, empty origin.

**FACT:** DRS-on-discovered-URL via HelixTest `run_drs_checks` (generic + strict checksums) has an in-process B1 mock path that the integration tests require to pass all five names.

**FACT:** Skip-is-not-pass is tested for terminal color and JSON `passed`/`status`.

**FACT:** Honesty strings and D1/D3/D4/HELIOS split are enforced by `scripts/prove.sh` greps and by README.

**FACT:** CI exists (prove + clippy `-D warnings` + rustfmt + gitleaks). Pre-commit mirrors fmt/clippy/prove.

**FACT:** Dummy fixtures are labeled not-for-production; gitleaks allowlist is explicit.

**INFERRED:** The DRS verify path against the **in-tree mock** is the most mature Helix behaviour. It is still not “run against Ferrum local” Stage 1 exit evidence in this audit.

---

## 17. What is only a scaffold

**FACT (docs + code):** `helix security` — dummy HMAC HTTP + local Crypt4GH header. Stage 3 not exited.

**FACT:** `helix bench` — `http.drs.smoke.v1` measurement engine (warmup + measured, median, warn-only, Helix RSS on Linux). Stage 4 not exited. Thresholds do not fail CI.

**FACT:** WES/TES/TRS/htsget: discovery only. No HelixTest check functions wired.

**FACT:** Stage 2 helix-action on Ferrum `main` is not done; pilot branch only.

**FACT:** Stage 1 exit (DRS **and** WES vs Ferrum local, documented command) is not done.

**FACT:** Beacon, africa, infra, E2E, Passport-on-DRS, Crypt4GH HTTP against Ferrum: HelixTest inventory, not Helix CLI.

**FACT:** No installable release artifact / crates.io publish process is described in INSTALL beyond `cargo run` with a sibling HelixTest.

---

## 18. What must NOT be changed yet

Do not treat the list below as implementation work. It is a freeze set for the next development phase.

**FACT (D1):** Do not merge HelixTest into Helix. Do not vendor the suite to avoid the sibling checkout.

**FACT (D4 / HELIX_VS_HELIOS):** Do not add RO-Crate, PDF, signed evidence, Nextflow/Snakemake envelopes, or ISO 15189 / AI Act checklist scores to Helix.

**FACT (D2):** Do not treat `ghcr.io/example/mock-*` compose as a Stage 0 or Helix proof until those images are proven.

**FACT (ROADMAP Stage 2):** Do not put helix-action on Ferrum `main` as a required check until the pilot false-alarm criterion is met.

**FACT (VERSIONS.lock comments):** Do not bump the published HelixTest pin off v0.1.3 / `1832c043…` without a HelixTest tag that Ferrum/Lab Kit/ga4gh-infra also take.

**FACT (IDENTITY / README):** Do not claim Ferrum production deployments, clinical live use, GA4GH certification, or HelixTest/Helix results as certificates.

**FACT (CLI_CONTRACT D3):** Do not invent a second conformance JSON language for `verify` / `security`. Do not treat Skip as Pass.

**FACT:** Do not rename the binary to `helios` or merge HELIOS pricing/docs into Helix.

**FACT:** Do not drop the README honesty sentence; `prove.sh` requires it.

**INFERRED:** Do not silently change DRS check names or HelixTest `OverallReport` field meaning; Ferrum and helix-action comments consume that shape (CLI_CONTRACT / ROADMAP Stage 2).

---

## Proposed implementation sequence

Not implemented in this audit. Order is scope, not dates. Single-steward: finish a stage exit before expanding the next.

1. **Doc honesty (cheap, unblocks evaluators)**
   Align INVENTORY closing paragraph, `FOR-EVALUATORS.md`, `PROVE.md`, `DEPENDENCY.md`, and HELIX_VISION §7 / HELIX_VS_HELIOS CLI table with the fact that `helix` already runs DRS verify. Keep HelixTest as the tagged Ferrum pin. Do not weaken certification/Ferrum-pilot language.

2. **Pin/CI hygiene (still Stage 1 supporting)**
   Make CI checkout ref a single source (`VERSIONS.lock` or generated). Optionally `cargo test --locked` / clippy `--locked`. Document that local `../HelixTest` should match the SHA (or fail `ci-check` on mismatch). Do not bump the pin.

3. **Stage 1 exit — WES on `helix verify`**
   Wire HelixTest WES checks when WES is discovered; keep DRS-first. Prove against **Ferrum local** (`make up`) as the roadmap exit, plus keep the non-Ferrum B1 mock for DRS. JSON stays `OverallReport`. Skips remain skips. Do not add Beacon/africa/infra.

4. **TES / TRS / htsget (same stage if cheap, not the exit)**
   Only after WES is real. Same discovery bases; do not invent new report schema.

5. **Stage 2 exit (helix-action)**
   Keep Ferrum `main` on HelixTest until the pilot has no known-bad comments. Fail jobs only on PASS→FAIL vs last successful run. Do not make Helix a required Ferrum check on first landing.

6. **Stage 3 exit**
   Reproduce the five HTTP cases against a documented Ferrum HMAC or `make up-pilot-local` path. Keep dummy fixtures out of production. Do not add HELIOS wrapping. Do not claim Passport certification.

7. **Stage 4 exit**
   Compare two Ferrum versions on the same runner class with stored artefacts. Keep `http.drs.smoke.v1` unless a later decision adds a larger Helix-owned workload (not GIAB). Do not fail CI on threshold miss unless a later decision changes CLI_CONTRACT. Do not dock HELIOS PDF as the bench report.

8. **Stage 5**
   Only after 0–4 exits: stranger-installable `helix` (or documented pin) and one voluntary external try. No public bake-off.

Out of sequence on purpose: Helix Cloud, SLA, dashboards, vendor ranking, HelixTest git merge, any HELIOS feature in this repo.
