# Helix fixtures

Deterministic, in-process targets so every contributor can run meaningful Helix verification **without** Ferrum, Docker, hospital infrastructure, credentials, or external services.

HelixTest already runs DRS and WES checks. These fixtures productize that engine against known HTTP. They are not a second suite, not HELIOS evidence, and not GA4GH certification. Ferrum remains a **reference live target** (opt-in). It is never required for `make prove`.

Source: `tests/support/`, `test-fixtures/`. Tests: **`make prove`**. Human `helix verify` report without Ferrum: **`make verify-fixture`**.

**No real secrets.** Dummy HMAC and Crypt4GH bytes are labeled NICHT FÜR PRODUKTION / NOT FOR PRODUCTION. CLI stdout/stderr must not print those values or `Authorization` headers ([THREAT_MODEL.md](THREAT_MODEL.md)).

---

## Strategy

| Rule | Meaning |
|------|---------|
| In-process HTTP | wiremock on `127.0.0.1`. No compose, no GHCR mock images (D2). |
| Deterministic bytes | Same object id, blob, checksum, workflow URLs, and JWT claims every run. |
| Valid and invalid | Passing fixtures prove the engine can pass. Invalid fixtures prove it can fail. Skip-only and unreachable are separate. |
| CI uses the catalog | GitHub CI runs `cargo fetch --locked`, `make prove` (docs + `cargo test --locked --offline --all-targets`), `make independent-verify`, then `make verify-fixture`. Fixture tests are **not** `#[ignore]`. |
| Live stays live | `make prove` does not skip, exclude, or ignore tests. Pointing `helix` at Ferrum is **`make test-live`** (opt-in). Do not fold live HTTP into prove. |
| Public interfaces | Fixtures speak published GA4GH HTTP. WES `name` never switches Helix to Ferrum mode ([PROFILES.md](PROFILES.md)). External origin: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md). |

`cargo test -- --ignored` is empty today. Do not mark fixture tests ignored to “save CI time.”

Catalog id recorded on every `helix verify` JSON: **`helix-fixtures-v1`** (`fixture_version`). Bump that string when `test-object-1` bytes, the unknown-id string, or WES TRS fixture URLs change. It is compare identity only ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not a HELIOS crate version. Not a signature.

---

## How to run (no Ferrum)

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch
make prove
make verify-fixture
```

Needs a sibling HelixTest checkout (path dependency, D1). `make fetch` is crates.io at `Cargo.lock` checksums. Prove then runs **offline**. Does **not** need Docker, Ferrum `make up`, hospital IdP, or GA4GH downloads. Localhost wiremock only during tests.

Live (you start the stack): [PROVE.md](PROVE.md) / `make test-live HELIX_LIVE_URL=http://127.0.0.1:8080`.

---

## Deterministic test object

Default catalog input for in-process mocks and Ferrum’s demo object. **Not a GA4GH requirement.** External targets declare their own id with `--drs-object-id` ([TARGETS.md](TARGETS.md) §11, [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md)).

HelixTest DRS checks GET `objects/{object_id}` (default `test-object-1`) and a **derived** unknown id. Helix’s valid mock implements the default catalog table.

| Field | Value |
|-------|--------|
| Object id | `test-object-1` (override: `--drs-object-id`) |
| Blob | 4096 bytes, each `0x41` (`'A'`) |
| Checksum | SHA-256 of that blob (`common::util::sha256_bytes`). Operator `--drs-object-sha256` supplies an independent expected digest; otherwise checksum compares advertised GetObject sha256 vs download. |
| Access | `GET /bytes/test-object-1` (HTTP Range → 206) |
| Unknown id | `helix.unknown.` + first 32 hex of `sha256("helix.unknown\n{object_id}")` → **404**. Not the global string `nonexistent-object-id-for-conformance`. |
| `self_uri` | `drs://example.invalid/test-object-1` (not a live host) |
| `created_time` | `2020-01-01T00:00:00Z` |

Constants: `tests/support/mock_ga4gh_drs.rs` (`TEST_OBJECT_ID`, `BLOB_LEN`). The mock still 404s the legacy unknown path; the checker no longer assumes that string is globally unused. Security cases hit the same default object id behind an Authorization header.

A missing configured object is **fixture-unavailable** (skip), not target non-conformance. Helix does not auto-discover objects. A pass on unknown-id `HLX-DRS-005` alone is not an overall DRS verification pass.

---

## Fixture catalog

Each row: **purpose**, **expected behavior**, **valid / invalid**, **CI**, **credentials**.

### 1. Valid mock DRS

| | |
|--|--|
| Source | `tests/support/mock_ga4gh_drs.rs` `start_mock_ga4gh_drs`. Evaluator path: **`make verify-fixture`**. |
| Purpose | Passing DRS target for `helix verify` / adapter tests |
| Expected | DRS DETECTED+TESTABLE; five `HLX-DRS-*` **pass**. WES not mounted → skip under profile `generic`. Exit 0. |
| Target | **Valid** DRS. Not a WES, TES, TRS, or htsget service. |
| CI | **Yes** (`tests/verify_drs.rs`, `adapter_drs.rs`, `cli_contract.rs`, `verify_profile.rs`) |
| Credentials | **None** |

No `/service-info` on this mock (that path is the WES probe). HelixTest B1 still mounts a Ferrum-name WES trap; Helix does not copy it.

### 2. Known-invalid DRS object

| | |
|--|--|
| Source | `start_mock_invalid_drs_object` — `GET /objects/test-object-1` → `{ "id": "test-object-1" }` only |
| Purpose | Prove DETECTED is not a pass; schema/checksum/range/bytes **fail** with `HLX-DRS-*` |
| Expected | `present: true`, `testable: true`, executed `fail`, exit 1. No `discovery.drs` check row. |
| Target | **Intentionally invalid** DRS |
| CI | **Yes** (`tests/verify_drs.rs`, `cli_discover.rs`) |
| Credentials | **None** |

### 2a. Schema-ok, checksum-wrong DRS

| | |
|--|--|
| Source | `start_mock_schema_ok_checksum_wrong` |
| Purpose | Prove SCHEMA PASS is not BEHAVIOR PASS: JSON extras pass; downloaded bytes do not match advertised sha256 |
| Expected | `drs.object.schema` **pass**, `drs.object.checksum` **fail** (message contains checksum/mismatch). `layer_summary.schema.passed` ≥ 1, `layer_summary.behavior.failed` ≥ 1. Exit 1 |
| Target | **Intentionally broken behaviour**, valid-looking schema |
| CI | **Yes** (`tests/behavior.rs`) |
| Credentials | **None** |

### 2b. Schema-ok, unknown-id HTTP 200 DRS

| | |
|--|--|
| Source | `start_mock_schema_ok_unknown_id_200` |
| Purpose | Prove schema-valid object JSON does not imply error semantics: unknown id returns 200 |
| Expected | `drs.object.schema` **pass**, `drs.object.not_found` **fail** with observed 200. Exit 1 |
| Target | **Intentionally broken behaviour**, valid-looking schema |
| CI | **Yes** (`tests/behavior.rs`) |
| Credentials | **None** |

### 3. Valid mock WES

| | |
|--|--|
| Source | `tests/support/mock_ga4gh_wes.rs` `start_mock_ga4gh_wes` |
| Purpose | Passing WES HTTP for HelixTest `run_wes_checks` (generic Mode) |
| Expected | First status poll `RUNNING`; later poll terminal from the workflow table. Profile `generic`: seven WES **pass**, scatter **skip**. Exit 0 if something else passed. |
| Target | **Valid** WES for HelixTest’s synthetic TRS URLs. Default `name` is not “Ferrum Gateway”. |
| CI | **Yes** (`tests/verify_wes.rs`, `adapter_wes.rs`) |
| Credentials | **None** |

Deterministic WES responses (HelixTest fixture URLs, not invented):

| `workflow_url` | Terminal | Outputs |
|----------------|----------|---------|
| `trs://test-tool/echo/1.0` + CWL | `COMPLETE` | `echo_out` = `workflow_params.message` |
| `trs://test-tool/fail/1.0` | `EXECUTOR_ERROR` | — |
| `trs://test-tool/cwl-echo/1.0` (missing inputs or WDL) | `EXECUTOR_ERROR` | — |
| `trs://nonexistent/invalid/0.0` | `EXECUTOR_ERROR` | — |
| `trs://test-tool/scatter-gather/1.0` | `COMPLETE` | `scatter_result` present |

Scatter is **not posted** on profile `generic`. Profile `ferrum` posts it ([WES.md](WES.md), [PROFILES.md](PROFILES.md)).

### 4. Known-invalid WES service-info

| | |
|--|--|
| Source | `start_mock_wes_incomplete_service_info` |
| Purpose | Incomplete ServiceInfo JSON still DETECTED+TESTABLE; schema check fails |
| Expected | WES `fail` + `failure.code` `HLX-WES-*`, exit 1 |
| Target | **Intentionally invalid** WES |
| CI | **Yes** (`tests/verify_wes.rs`) |
| Credentials | **None** |

### 5. Combined DRS + WES

| | |
|--|--|
| Source | `start_mock_ga4gh_drs_and_wes` |
| Purpose | Both suites on one origin (profile `ferrum` scatter path) |
| Expected | Profile `generic`: DRS pass + WES pass + scatter skip. Profile `ferrum`: scatter **pass**. |
| Target | **Valid** DRS and WES |
| CI | **Yes** (`tests/verify_wes.rs`, `verify_profile.rs`) |
| Credentials | **None** |

### 6. WES named “Ferrum Gateway”

| | |
|--|--|
| Source | `start_mock_ga4gh_wes_named("Ferrum Gateway")` |
| Purpose | Prove Helix does **not** auto-switch profile or enable scatter from `service-info.name` |
| Expected | Profile stays `generic`; `HLX-WES-008` **skip** (`supports_scatter_gather=false`) |
| Target | **Valid** WES HTTP; name is a trap, not Ferrum |
| CI | **Yes** (`tests/verify_profile.rs`) |
| Credentials | **None** |

### 7. Empty HTTP origin

| | |
|--|--|
| Source | wiremock with **no** DRS/WES mounts (`MockServer::start`) |
| Purpose | Skip-only run: services NOT_DETECTED |
| Expected | All DRS+WES rows `skip`, `summary.passed = 0`, exit **1**. Skip is never pass. |
| Target | **Valid** HTTP server, **no** GA4GH APIs (not an invalid DRS body) |
| CI | **Yes** |
| Credentials | **None** |

### 8. Unreachable TCP

| | |
|--|--|
| Source | bind `127.0.0.1:0`, drop listener, use that URL (`closed_origin` in tests) |
| Purpose | Distinguish transport **error** from skip/fail |
| Expected | DRS and WES rows `error` (“unreachable”), exit 1. Not skip, not pass. |
| Target | **No server** (not an HTTP 404) |
| CI | **Yes** |
| Credentials | **None** |

### 9. Auth-gated DRS (security fixture)

| | |
|--|--|
| Source | In-process `AuthGate` (`VerifierPolicy`) in `src/security/mod.rs` tests and `tests/security_cli.rs`. Dummy HMAC from `test-fixtures/hmac/shared-secret.txt` |
| Purpose | Security Behavior Profile: valid / expired / wrong-scope / garbage / wrong-audience. **Negative** policies (`reject_all`, `ignore_expiry`, `ignore_scope`, `ignore_signature`, `ignore_audience`) prove Helix fails the matching case. |
| Expected | Fail-closed: valid Bearer → 2xx; expired/garbage → 401; wrong scope/aud → 401 or 403. `helix security` exit 0. Broken mocks: matching `HLX-AUTH-01x` **fail**. Crypt4GH header case uses the well-formed file (or embedded bytes) **after** the five HTTP cases. |
| Target | **Valid** for selected behaviour checks; **not** ga4gh-infra, not Passports, not a hospital IdP, not a security audit |
| CI | **Yes** |
| Credentials | **Dummy only** (see HMAC file). No real secrets. |

Tokens are minted in-process (`helix-stage3-fixture-user`, issuer `https://helix.test.invalid`). Never log the secret. Passing does not prove the implementation is secure. Profile: [SECURITY_PROFILE.md](SECURITY_PROFILE.md).

### 10. HMAC file

| | |
|--|--|
| Source | `test-fixtures/hmac/shared-secret.txt` |
| Purpose | CI can mint/verify HS256 JWTs without an IdP |
| Expected | `load_hmac_secret` reads the non-comment line. CLI `--hmac-secret-file` / `HELIX_HMAC_SECRET` |
| Target | N/A (file, not HTTP) |
| CI | **Yes** |
| Credentials | **Dummy:** `helix-dummy-hmac-not-for-production-do-not-use`. NICHT FÜR PRODUKTION. Not `FERRUM_AUTH__JWT_SECRET`. |

### 11–13. Crypt4GH headers

| File | Purpose | Expected | Valid / invalid | CI | Credentials |
|------|---------|----------|-----------------|----|-------------|
| `test-fixtures/crypt4gh/well-formed.c4gh` | Structure pass | magic `crypt4gh`, version 1, one dummy packet | **Valid** header layout | **Yes** | **None** (no private key material) |
| `test-fixtures/crypt4gh/wrong-magic.c4gh` | Structure fail | `BadMagic`; error text must not dump bytes | **Intentionally invalid** | **Yes** | **None** |
| `test-fixtures/crypt4gh/truncated.c4gh` | Structure fail | `TooShort` | **Intentionally invalid** | **Yes** | **None** |

Not a genome file. Helix does not decrypt. Default `helix security` embeds `well-formed.c4gh` for `HLX-AUTH-050`. `HLX-AUTH-053` must reject wrong-magic, truncated, version 2, and zero-packet envelopes. `HLX-AUTH-054` inspects a live 2xx DRS body only if magic is present; otherwise skip. See [CRYPT4GH.md](CRYPT4GH.md).

### 14. Crypt4GH key placeholder

| | |
|--|--|
| Source | `test-fixtures/crypt4gh/dummy-x25519.placeholder` |
| Purpose | Explicit fake keypair label so nobody “fills in” a real X25519 secret |
| Expected | Header tests **do not read** this file |
| Target | N/A |
| CI | File present; not executed as a check |
| Credentials | **Placeholder only** (all-zero / DO-NOT-USE). Not a Crypt4GH secret. |

### 15. Bench tiny workload

| | |
|--|--|
| Source | `tests/bench_cli.rs` `mount_tiny_workload` |
| Purpose | `helix bench` measurement engine without Ferrum (`http.drs.smoke.v1`) |
| Expected | Two origins, `--repetitions 2`; 500 on candidate object → `warning: true`, `analysis.regression` false (error-rate warning), process still exit 0. JSON includes `workload_id`, metadata, `analysis`. |
| Target | **Minimal** HTTP, not a full DRS mock |
| CI | **Yes** |
| Credentials | **None** |

Not GIAB. Prove does not fail on a bench threshold. Contract: [BENCHMARKS.md](BENCHMARKS.md).

### 16. Adversarial / malformed local servers

Hostile HTTP **only on localhost wiremock or a local TCP closer**. These are not exploits, not a pentest suite, and not a security product. They prove the verifier **fails closed**: process terminates, timeouts hold, no panic, no credential leak, deterministic fail/skip/error, **malformed is not PASS**.

Source: `tests/support/mock_adversarial.rs`. Tests: `tests/adversarial.rs`. Helix-owned client rules: [THREAT_MODEL.md](THREAT_MODEL.md).

| Helper | Behavior | Expected |
|--------|----------|----------|
| `start_malformed_json` | Truncated JSON + decoy `Authorization: Bearer` JWT on `/objects/test-object-1` | DETECTED; `drs.object.schema` **fail**; overall not pass; JWT absent from JSON |
| `start_huge_json` | Body > 2 MiB on DRS probes | NOT_DETECTED (size cap); skip-only; `passed = 0`; terminates |
| `start_invalid_headers` | Odd `Content-Type`, `WWW-Authenticate: Bearer <jwt>`, incomplete DrsObject | Schema not pass; JWT not printed |
| `start_redirect` | 302 on every DRS probe (one `Location` has URL userinfo; bait 200 is **not** a probe path) | Not followed; DRS not present; `passed = 0`; password not echoed |
| `start_slow_response` | First DRS probe delayed 6s (> 5s Helix-owned timeout) | Completes in well under HelixTest’s 30s; not pass |
| `start_connection_reset` | Accept TCP, close without HTTP | Terminates; `passed = 0` |
| `start_invalid_content_type` | `text/html` with a decoy Authorization line | Schema not pass; JWT not printed |
| `start_unexpected_status` | HTTP 418 on DRS probes | NOT_DETECTED (not 2xx/401/403); `passed = 0` |
| `start_malformed_service_info` | 200 JSON whose `id`/`name`/`type` are the wrong JSON types | DETECTED is not a pass; checks fail or skip, never a green run |
| `start_extremely_long_strings` | DrsObject with a 32 KiB `name`, required fields missing | No panic; schema not pass |
| `start_ansi_and_log_injection` | CSI green `PASS` plus forged `HELIX VERIFICATION` newlines in `name` | Schema not pass; no ESC in JSON/text; exactly one `HELIX VERIFICATION` header |

`reachable` may still **pass** when the server returns HTTP 200 garbage (that check only asks whether the object URL answered). **Overall** verify is not pass. Schema/content checks must not pass.

Decoy JWT / userinfo exist **only** so tests can assert Helix does not print them. They are not real credentials. No fixture here contacts an address other than the in-process mock (except a 302 `Location` Helix must **not** fetch).

CI: **Yes**. Credentials: **None** (decoys only).

### 17. Mutation corpus (one defect each)

| | |
|--|--|
| Source | `tests/support/mock_mutation.rs` `start_mutant`; catalog `src/mutation.rs` |
| Purpose | Prove known-bad targets fail for the **recorded** check and diagnostic; record misses instead of hiding them |
| Expected | Detected mutants: overall not PASS; named check `fail` with the catalog diagnostic class. Missed mutants: hypothesized check does **not** fail; reason in [MUTATION.md](MUTATION.md). Honest DRS+WES control still PASS |
| Target | **Intentionally broken** (one defect) or honest control |
| CI | **Yes** (`tests/mutation.rs`) |
| Credentials | Dummy HMAC for HLX-MUT-016/017 only |

Not a pentest. Not certification. Do not weaken Helix so a mutant “passes.”

---

## What is not a Helix fixture

| Thing | Why |
|-------|-----|
| Ferrum / Demo Docker | Live reference target. Opt-in (`make test-live`). Not prove. |
| `ghcr.io/example/mock-*` | Unverified (D2). Not a proof target. |
| Hospital / DIZ credentials | Never. Dummy HMAC only. |
| HelixTest live-stack crates | Stay in HelixTest (`api-tests`, …). Helix prove does not run them and does not replace them. |
| TES / TRS / htsget execution mocks | Discovery only today. Do not invent TES fixtures. |
| HELIOS RO-Crate / signed evidence | Out of scope. |

---

## `make prove`

Repo-root target. Verifies Helix **core** on this catalog:

1. `scripts/require-helixtest.sh` — sibling HelixTest must exist (exit 2 with clone commands if not)
2. `scripts/prove.sh` — required docs (including this file) and honesty strings
3. `cargo test --locked --offline --all-targets` — **all** Helix tests (fixture-backed). No `--ignored` filter, no crate excludes. If the crate cache is empty, run `make fetch` first.

Does not start Ferrum. Does not download GA4GH files. Localhost wiremock only. Green prove is a technical signal, not certification. Clone-and-run procedure: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md).

`make verify-fixture` is the human `helix verify` report against §1. It is not inside the prove test loop; CI runs it after prove. `make test-live` is **not** part of prove. It runs `helix verify` against `HELIX_LIVE_URL` when you already started a stack. Do not fold live HTTP into prove.

---

## Adding a fixture

1. Prefer a named helper in `tests/support/` (valid or known-invalid).
2. Document it in this file with the five fields above.
3. Keep it deterministic (fixed ids, bytes, URLs).
4. No real secrets. Label dummies NICHT FÜR PRODUKTION.
5. Do not `#[ignore]` it. Do not require Ferrum for `make prove`.
6. Do not import Ferrum. Do not auto-switch on WES `name`.
