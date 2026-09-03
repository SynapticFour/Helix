# Helix inventory — HelixTest as it actually exists

**Date:** 2026-09-03  
**HelixTest checkout:** `1832c043e1679ec283cb2113510ee33684317cce` (git tag **v0.1.3**, `main` matches origin).  
**Also opened:** Ferrum (`VERSIONS.lock`, `.github/workflows/*`, `docs/HELIXTEST-INTEGRATION.md`), Ferrum-Lab-Kit, ga4gh-infra (`VERSIONS.lock`), HELIOS (scope boundary only), technical-reports (SF-TR-2026-001 / SF-TR-2026-002), Ferrum-GA4GH-Demo.  
**Not present as a submodule of HelixTest.** No `.gitmodules` in HelixTest.

This file records what the code does. It is not a product claim. HelixTest results are not GA4GH certification. HELIOS (reproducibility / signed evidence / RO-Crate / PDF) is out of Helix scope and was not duplicated here.

---

## 1. Conformance coverage (what the CLI actually runs)

Source of truth: `helixtest/crates/framework/src/lib.rs` `run_all`, plus `africa.rs` / `infra.rs` for opt-in modes. Test **names** below are the strings passed to `TestCaseResult::{pass,fail,skip,from_outcome}`.

Default `--all` (modes `generic` / `ferrum`) executes these `ServiceKind`s, in this order: WES, TES, DRS, TRS, Beacon, htsget, Auth, Age, Crypt4gh, E2e. `africa` and `infra` in a default `--all` are recorded as skipped (“use `--mode ferrum-africa` or `--mode ferrum+infra`”).

`--mode ferrum-africa` and `--mode ferrum+infra` **replace** `run_all`; they do not run the default ladder.

### DRS (`framework/src/drs.rs`)

Hard-coded object id `test-object-1`.

| Level | Name |
|-------|------|
| 0 | DRS object endpoint reachable |
| 1 | DRS DrsObject OpenAPI + access_methods |
| 2 | DRS checksum correctness (skipped unless profile `[features] strict_drs_checksums = true`) |
| 2 | DRS HTTP Range support |
| 5 | DRS invalid object id returns 404 |

### WES (`framework/src/wes.rs`)

Uses synthetic `trs://` URLs (`test-tool/echo`, `fail`, scatter-gather). `_mode` is unused.

| Level | Name |
|-------|------|
| 0 | WES service-info reachable |
| 1 | WES service-info schema (GA4GH official) |
| 2 | WES lifecycle success echo (API may show QUEUED/INITIALIZING/RUNNING before COMPLETE) |
| 2 | WES failure state for bad workflow |
| 2 | WES missing inputs leads to error state |
| 2 | WES incompatible workflow_type leads to error state |
| 3 | WES invalid workflow leads to error state |
| 2 | WES scatter/gather workflow (skipped unless `supports_scatter_gather`) |

### TES (`framework/src/tes.rs`)

Submits Alpine `echo hello-tes` tasks. `_mode` unused. Golden checksum file: `test-data/expected/workflows/tes_echo_out.txt.sha256`.

| Level | Name |
|-------|------|
| 0 | TES /tasks reachable |
| 1 | TES task schema (create + status) |
| 2 | TES task lifecycle + checksum (non-terminal states allowed until terminal) |

### TRS (`framework/src/trs.rs`)

| Level | Name |
|-------|------|
| 0 | TRS /tools reachable |
| 1 | TRS tools and versions schema |
| 2 | TRS descriptor retrieval |

### Beacon (`framework/src/beacon.rs`)

POST `/query`. Positive/negative variant checks skipped unless `supports_beacon_v2`.

| Level | Name |
|-------|------|
| 0 | Beacon /query reachable |
| 1 | Beacon boolean response (official schema) |
| 2 | Beacon known variant exists |
| 2 | Beacon negative variant not exists |

### htsget (`framework/src/htsget.rs`)

Base URL from `HTSGET_URL` / `GATEWAY_BASE` / `[services] htsget` / gateway-style WES/DRS URLs. Bare `http://host:port` is treated as a unified gateway only in Ferrum-like modes. If unresolved: one skip, “htsget suite (service-info, tickets, POST, errors)”.

Reads object default `test-object-1`; variants default `demo-sample-vcf` (env `HTSGET_READS_OBJECT_ID` / `HTSGET_VARIANTS_OBJECT_ID`).

| Level | Name |
|-------|------|
| 0 | htsget reads /reads/service-info (htsget 1.3.0) |
| 0 | htsget variants /variants/service-info (htsget 1.3.0) |
| 1 | htsget GET reads ticket (BAM + DRS stream URL) |
| 1 | htsget GET variants ticket (VCF/BCF + DRS stream URL) |
| 2 | htsget GET variants with reads-only object → NotFound |
| 2 | htsget POST reads ticket (JSON body, no query) |
| 2 | htsget POST reads ticket with regions → InvalidInput (Ferrum does not slice) **or** htsget POST reads ticket (JSON body with regions) |
| 2 | htsget POST variants ticket (JSON body, no query) |
| 2 | htsget POST variants ticket with regions → InvalidInput (Ferrum does not slice) **or** htsget POST variants ticket (JSON body with regions) |
| 2 | htsget POST reads with query params → InvalidInput |
| 2 | htsget GET reads ?format=CRAM on BAM object → UnsupportedFormat (skipped if object already reports CRAM) |
| 2 | htsget GET reads ?class=header → InvalidInput |
| 4 | htsget dataset auth (403 without token, 200 with Passport/JWT) — skipped unless `HELIXTEST_HTSGET_DATASET_OBJECT_ID` is set |

`ferrum_like(mode)` is `Ferrum | FerrumAfrica | FerrumInfra`. Those modes require HTTP 400 `InvalidInput` for POST `regions` (comment in source: Ferrum does not implement genomic slicing).

### Auth (`framework/src/auth.rs`)

Default path is **HMAC-SHA256 JWT fixture** against DRS, **not** GA4GH Passports (Passports are `infra.rs`). Skipped if `HELIXTEST_SHARED_SECRET` unset. Object id default `test-object-1`. `HELIXTEST_AUTH_SURFACE=service-info` targets DRS `/service-info` instead of an object.

If config `[auth_checks] mode = "token-protected-endpoints"`: Level 0 auth URL plus per-endpoint invalid/valid token checks (names from TOML `name` fields).

Default HMAC names:

| Level | Name |
|-------|------|
| 0 | Auth /service-info reachable (auth_url) — skip if `auth_url` empty |
| 4 | Auth (HMAC JWT fixture): valid token grants DRS access |
| 4 | Auth (HMAC JWT fixture): expired token rejected |
| 4 | Auth (HMAC JWT fixture): garbage bearer rejected |
| 4 | Auth (HMAC JWT fixture): wrong scope denied |
| 4 | Auth (HMAC JWT fixture): missing token returns 401 |

In `--mode ferrum`, `HELIXTEST_SKIP_AUTH=true` replaces the suite with one skip: “Auth suite skipped (HELIXTEST_SKIP_AUTH=true)”.

### Age (local, not a GA4GH service)

`crypt4gh.rs` `run_age_checks`: in-process `age` library. No HTTP. Always part of default `--all`.

- Local age library available (L0)
- Local age: roundtrip checksum / partial read / corrupted header fails / wrong passphrase fails / corrupted ciphertext fails / truncated ciphertext stream fails (L5)

### Crypt4GH HTTP (Ferrum-gated)

`run_crypt4gh_checks` + `crypt4gh_ferrum_http.rs`. Level 0 skip unless `HELIXTEST_FEATURE_CRYPT4GH_REWRAP=1`. Then:

- Crypt4GH DRS rewrap download (X-Crypt4GH-Public-Key)
- Crypt4GH plain download matches rewrap plaintext (decrypt_plain) — extra gate `HELIXTEST_FEATURE_CRYPT4GH_PLAIN=1`

### E2E (`framework/src/e2e.rs`)

Module comment: drives WES to terminal `COMPLETE`; **does not poll TES**. Full TRS→TES coupling is described as living in `e2e-tests` only when a mock stack defines that contract. The `e2e-tests` crate itself only calls `run_all(..., E2e)` — same framework function.

| Level | Name |
|-------|------|
| 0 | E2E TRS /tools reachable |
| 3 | E2E TRS→DRS→WES→DRS output→Beacon (WES polled to terminal; no TES poll in this module) |

### Africa (`--mode ferrum-africa` only)

`africa.rs`. Always: `africa: gateway /health`. Then by `--africa-profile`:

- **offline:** `offline: DRS service-info`, `offline: Beacon service-info`, `offline: reference registry seeded` (`GET {gateway}/api/v1/references`, expects ≥6 entries)
- **ont:** fixture `fixtures/africa/synthetic_ont_file.pod5.stub`; `POST {gateway}/api/v1/ingest/ont`; `ont: DRS object created`; `ont: ont_metrics on DRS object`; Beacon organism query
- **outbreak:** `POST /api/v1/outbreak/activate` / `deactivate`; `GET /api/v1/audit/residency/verify` (`chain_valid`)
- **federation:** skip unless `FERRUM_AFRICA_PEER_URL`; then local `federate=true` Beacon query and peer Beacon query

### Infra (`--mode ferrum+infra` only)

`infra.rs`. Unreachable broker/registry/login **fails** (not skip). Checks:

- infra: broker service-info (L0) — `GA4GH_BROKER_URL` default `http://127.0.0.1:8180`
- infra: service registry lists entries
- infra: Ferrum DRS registered in service registry
- infra: broker login issues Passport (mock-idp cookie flow; `MOCK_IDP_SUBJECT` default `researcher@uni-heidelberg.de`)
- infra: Passport accepted on Ferrum DRS (`HELIXTEST_AUTH_OBJECT_ID` or `test-object-1`)

### Profiles on disk (`helixtest/profiles/`)

`ferrum.toml`, `ferrum-infra.toml`, `ferrum-infra-pilot.toml`, `ferrum-africa.toml`, `generic.toml`, `strict.toml`, `bioresearch-assistant.toml`.

`ferrum.toml` points all services at `http://localhost:8080/ga4gh/...` and sets `supports_scatter_gather`, `supports_beacon_v2`, `strict_drs_checksums`.

### Cargo test crates vs CLI

Live-stack crates (`api-tests`, `auth-tests`, `e2e-tests`, `workflow-tests`) call `framework::run_all` and need a running target. They are **excluded** from `make test` / CI `make prove`. `crypt4gh-tests` runs in CI (in-process age).

---

## 2. How HelixTest is invoked

### CLI (this repo)

Binary name `helixtest` (`helixtest-cli` crate). Without `--all` it prints “Nothing to do” and exits 0.

```
helixtest --all
  [--mode generic|ferrum|ferrum-africa|ferrum+infra]
  [--profile NAME]
  [--start-ferrum] [--compose-file PATH]
  [--report table|json|scores|coverage]
  [--fail-level 0-5]
  [--only wes|tes|drs|trs|beacon|htsget|auth|age|crypt4gh|e2e|africa|infra]…
  [--africa-profile offline|ont|outbreak|federation|all]
  [--verbose]
```

`--start-ferrum` runs `docker compose up -d` on `--compose-file` or `helixtest/docker/docker-compose.yml`, then waits 60s for WES `/service-info`. That compose file defines **mock-*** services (`ghcr.io/example/mock-wes:latest` etc.), not a Ferrum image. **UNKLAR — bitte prüfen:** whether those `ghcr.io/example/mock-*` images exist or are placeholders.

Config load (`common/src/config.rs`): `--profile` / `HELIXTEST_PROFILE` → `profiles/<name>.toml`; else `HELIXTEST_CONFIG`; else `./helixtest-config.toml`; else env `WES_URL`… with split-port defaults (`8080`–`8085`). Env overrides file URLs.

Local: `make prove` = offline tests + SPDX check + release CLI. `docs/PROVE.md`: live run needs a target you start (Ferrum / Demo). HelixTest does not deploy servers except optional `--start-ferrum`.

### HelixTest GitHub Actions

| Workflow file | Name | Triggers | What it runs |
|---------------|------|----------|----------------|
| `conformance.yml` | CI | push/PR `main`/`master`, `workflow_dispatch` | `make prove` (offline). MSRV 1.88 `cargo check`. ARM `make prove` on push to main / dispatch only. **Does not** hit a live Ferrum stack. |
| `live-ferrum-ghcr.yml` | Live Ferrum GHCR | cron Mon 04:17 UTC, `workflow_dispatch` | Pull `ghcr.io/synapticfour/ferrum:edge`, demo-mode auth-off SQLite, `helixtest --all --mode ferrum --fail-level 2`. Schedule defaults `--only beacon`. Not PR. |
| `live-ferrum-ghcr-auth.yml` | Live Ferrum GHCR auth-on | cron Mon 04:27 UTC, `workflow_dispatch` | Same image, `require_auth=true`, HS256; `helixtest --only auth --fail-level 4`. Not Passports. |
| `release-binaries.yml` | Release binaries | tag `v*`, `workflow_dispatch` | linux-gnu x86_64/aarch64, darwin aarch64 |
| `spdx.yml` | SPDX | push/PR, dispatch | `scripts/spdx-rs.py` Apache-2.0 |
| `secret-scan.yml` | Secret Scan | PR, push main | gitleaks |
| `codeql.yml` | CodeQL | weekly cron, dispatch | Rust |
| `dependency-review.yml` | Dependency Review | PR | `continue-on-error: true` |

Third-party action mentioned in HelixTest `docs/IDENTITY.md`: `synapticfour/helixtest-action` (stated pin v0.1.1). **This inventory did not open that repo.**

### Callers in sibling repos (HelixTest cloned or on PATH)

**Ferrum** `VERSIONS.lock`: `HELIXTEST_REF=v0.1.3` / `HELIXTEST_SHA=1832c043…`.

| Workflow | Triggers | HelixTest usage (from file comments / `docs/HELIXTEST-INTEGRATION.md`) |
|----------|----------|--------|
| `conformance.yml` | push/PR main | Clone pin; demo stack; `HELIXTEST_SKIP_AUTH=true`; TES noop + stubs; `--all --mode ferrum --fail-level 1` plus per-service `--only`. Every PR. Labeled NON-PILOT. |
| `africa-conformance.yml` | push/PR main | `--mode ferrum-africa` profiles |
| `helixtest-ferrum-infra.yml` | **`workflow_dispatch` only** (schedule cron is commented “temporarily disabled”) | `make up-pilot-local`; `--mode ferrum+infra` |
| `helixtest-pilot-auth.yml` | **`workflow_dispatch` only** (schedule similarly commented out) | `require_auth=true`; `--only auth --fail-level 4`; stubs off |

**Ferrum-Lab-Kit:** CI checks out `SynapticFour/HelixTest` at SHA in `config/ci/helixtest-revision.txt` (`1832c043…`). Live suite optional. CLI `lab-kit conformance run` always passes `--all --mode ferrum --report json` (`conformance_run.rs`); timeout kills the process.

**ga4gh-infra:** `VERSIONS.lock` pins the same HelixTest tag/SHA. Docs tell operators to run `helixtest --mode ferrum+infra`. No HelixTest workflow file was found in ga4gh-infra `.github/workflows/` during this pass (CI is `ci.yml` for infra itself).

**Ferrum-GA4GH-Demo:** documents HelixTest as the conformance runner; its own CI does **not** run HelixTest (stated in `TESTING.md` / `docs/ECOSYSTEM.md`).

**HELIOS:** no HelixTest invocation found. Separate ambassador (`helios-audit`).

### technical-reports

SF-TR-2026-001 and SF-TR-2026-002 cite HelixTest (`@helixtest2026` → `https://github.com/SynapticFour/HelixTest`), including `--mode ferrum` / `ferrum-africa` / `ferrum+infra` command blocks. They state HelixTest is a technical signal, not certification.

---

## 3. Coupling to Ferrum (what Prompt B1 would have to break)

**No Cargo/git crate dependency.** HelixTest `Cargo.toml` / crate `Cargo.toml`s have no `ferrum` package. No `.gitmodules`. Coupling is HTTP + naming + fixtures + reverse CI clone.

| Kind | Where | What |
|------|--------|------|
| Mode names | `cli/src/main.rs`, `framework/src/lib.rs` | Opt-in: `ferrum`, `ferrum-africa`, `ferrum+infra`. **Generic does not auto-switch** (WES `service-info` `name` containing `"Ferrum"` is ignored). |
| Profiles | `helixtest/profiles/ferrum*.toml`, `ga4gh-drs.toml` | Ferrum: single-gateway `localhost:8080/ga4gh/...`. `ga4gh-drs`: DRS-only, `strict_drs_checksums=true`, any DRS URL via `DRS_URL`. |
| CLI flag | `--start-compose` (alias `--start-ferrum`) | Starts generic docker compose (often `helixtest/docker/docker-compose.yml`). Not used by generic CI. |
| Ferrum-only HTTP | `crypt4gh_ferrum_http.rs` | Headers/paths for Ferrum Crypt4GH rewrap / `decrypt_plain`. |
| Ferrum-only HTTP | `africa.rs` | `/api/v1/ingest/ont`, `/api/v1/references`, `/api/v1/outbreak/*`, `/api/v1/audit/residency/verify`; env `FERRUM_AFRICA_PEER_URL`. |
| Ferrum-only HTTP | `infra.rs` | Broker/registry/mock-idp login; DRS with Passport; default gateway `http://localhost:18080` if DRS URL has no `/ga4gh/drs`. |
| Behaviour fork | `htsget.rs` `ferrum_like` | Region POST must 400 on Ferrum modes. |
| Shared fixture ids | DRS/Auth/htsget | `test-object-1`, `demo-sample-vcf`. Ferrum demo seed must match (Ferrum `docs/HELIXTEST-INTEGRATION.md`). |
| Reverse pin | Ferrum / Lab Kit / ga4gh-infra | CI clones `SynapticFour/HelixTest` at a SHA. Ferrum demo compose sets `FERRUM_TES_HELIXTEST_STUB` / `FERRUM_WES_HELIXTEST_STUBS` / `HELIXTEST_SKIP_AUTH` (Ferrum side, not HelixTest imports). |
| Docs | `helixtest/docs/ferrum.md` | Ferrum operator guide inside HelixTest. |

HelixTest talks to **any** HTTP target that implements the published GA4GH APIs. Ferrum is a reference target and opt-in profile, not a library. Proof: in-process mock DRS in HelixTest CI (`helixtest/crates/framework/tests/generic_drs_mock.rs`, `helixtest/crates/cli/tests/generic_drs_independence.rs`).

---

## 4. License

HelixTest:

- Root `LICENSE`: **Apache License 2.0**, Copyright 2025 Synaptic Four.
- `Cargo.toml` workspace: `license = "Apache-2.0"`, `license-file = "LICENSE"`.
- First-party `.rs` files: `// SPDX-License-Identifier: Apache-2.0` (CI `spdx.yml`).
- README / IDENTITY: Apache-2.0 ambassador, not a product SKU.

No BUSL file in HelixTest. Ferrum remains BUSL-1.1 (sibling product). HELIOS is Apache-2.0 (separate repo).

---

## 5. Gaps (code and sibling docs — no `TODO`/`FIXME` in HelixTest)

`rg TODO|FIXME|XXX|HACK|unimplemented!` over HelixTest `*.rs` / `*.toml` / workflows: **no matches**.

Documented or encoded gaps:

| Source | Gap |
|--------|-----|
| `helixtest/docs/known-limitations.md` | WES/TES checks are **serial**. Live-stack cargo tests excluded from CI. `africa`/`infra` not on the default ladder. jsonschema 0.17 uses `Box::leak` once per schema (`OnceCell`); left on 0.17 to avoid reqwest 0.12. `once_cell` used because `OnceLock::get_or_try_init` is unstable. |
| `e2e.rs` module docs | Framework E2E does **not** poll TES. |
| `auth.rs` | HMAC fixture ≠ Passports; Passports only in `ferrum+infra`. Missing Bearer on `service-info` is skipped (public metadata). |
| `htsget.rs` | Dataset-gated L4 skipped without env. CRAM-on-BAM skipped if object already CRAM. Split-port generic mocks skip whole htsget suite. |
| `crypt4gh.rs` | HTTP Crypt4GH skipped unless feature env set. |
| `africa.rs` | Federation skipped without `FERRUM_AFRICA_PEER_URL`. Outbreak activate can skip if endpoint down/auth required. |
| `lib.rs` | `HELIXTEST_SKIP_AUTH` in Ferrum mode (used by Ferrum PR CI). |
| Ferrum `docs/HELIXTEST-INTEGRATION.md` | Claims **`/api/v1/ingest/*` is not in HelixTest today**. **Contradiction:** `africa.rs` **does** `POST /api/v1/ingest/ont` in `--mode ferrum-africa`. Default `--mode ferrum` does not. |
| HelixTest `docs/IDENTITY.md` | Still says product pin **v0.1.1** / `helixtest-action` v0.1.1. Ferrum/Lab Kit/ga4gh-infra lock **v0.1.3** (`1832c043`). Crate `version` in `helixtest-cli/Cargo.toml` is **0.1.0**. Operators are told to pin the **git tag**. |
| `helixtest/docker/docker-compose.yml` | Images `ghcr.io/example/mock-*` — **UNKLAR — bitte prüfen** if runnable. |
| Ferrum workflows | `helixtest-ferrum-infra.yml` and `helixtest-pilot-auth.yml` scheduled crons are **commented out**; only `workflow_dispatch`. |
| HELIOS | No overlap in HelixTest for RO-Crate, signed audit PDF, Nextflow/Snakemake wrap. Do not add those to Helix. |

---

## Helix (this repo) vs HelixTest

HelixTest already runs (CI, Ferrum pin, SF-TR citations). This repository is the **independence vehicle** for that suite: conformance, security behaviour, benchmark/regression. Implementation of the CLI still lives in `HelixTest` until it is moved or vendored here. Reproducibility/evidence stays in HELIOS.
