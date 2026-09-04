# Helix architecture

Intended architecture for Stages 1–4. Not a capability claim. Not certification.

Helix is HelixTest becoming a standalone VERIFY product. HelixTest already runs (public repo, CI, Ferrum pin, SF-TR-2026-001 / 002). This document does not invent a second suite. It names the layers Helix owns, the adapter HelixTest occupies, and the rules later work must keep.

**As-built snapshot:** [ENGINEERING_AUDIT.md](ENGINEERING_AUDIT.md). **Decisions:** [DECISIONS.md](DECISIONS.md) (D1–D4). **CLI/JSON/exit codes:** [CLI_CONTRACT.md](CLI_CONTRACT.md). **Stage exits:** [HELIX_ROADMAP.md](HELIX_ROADMAP.md). **HELIOS gate:** [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md).

This file is the target shape. Adapter isolation is as-built (`src/adapter`); remaining gaps are called out. Do not treat “intended” as already implemented.

---

## 1. Product shape

Helix is a **client**. An operator starts a GA4GH HTTP stack they control. Helix is pointed at that origin. Helix discovers which Stage 1 APIs answer, runs verification, and emits a human report, a machine report, and an exit code that CI can consume. The origin is treated as an untrusted HTTP target ([THREAT_MODEL.md](THREAT_MODEL.md)); Helix is not a security product.

```text
target implementation
      ↓
Helix discovery
      ↓
Helix verification engine
      ↓
verification result
      ↓
human report / machine report / CI
```

Ferrum is the **reference target** used to prove Stage 1 exit (`make up` / demo stack). It is never a Helix runtime crate, never started by Helix, and never required for `make prove` in this repo.

Helix must be able to test **non-Ferrum** implementations. Stage 0 already proved generic DRS against an in-tree HTTP mock (D2). Helix CI uses the same idea (B1 mock in `tests/support/`; catalog [FIXTURES.md](FIXTURES.md)).

**HELIOS is not part of this architecture.** Reproducibility, signed trails, RO-Crate, PDF, and ISO/AI-Act checklists stay in HELIOS (`helios-audit`). Helix answers *whether* a running system behaves. HELIOS answers *what* ran and *how* to reproduce it.

Helix does not become a server. Helix does not issue authentication credentials (no IdP, no Passport broker, no production secrets). Dummy HS256 fixtures in `test-fixtures/` are test-only (NICHT FÜR PRODUKTION). Helix does not claim GA4GH certification. Green CI and a green `helix verify` are a technical signal.

---

## 2. Layer map

Nine layers. Each has one job. Later stages add implementations behind the same seams; they do not collapse layers.

| # | Layer | Owns | Does not own |
|---|--------|------|----------------|
| 1 | Discovery | Which Stage 1 APIs answer under a URL | Check execution, scoring, Ferrum |
| 2 | Verification | Orchestration of conformance cases; profiles (`src/profile.rs`) declare expected/enabled services and capabilities | HTTP path probing, HELIOS evidence, Ferrum-special engine paths |
| 3 | Security behaviour | Fail-closed auth / Crypt4GH *behaviour* against a running target | Issuing real credentials, ga4gh-infra, HELIOS |
| 4 | Performance measurement | Repeatable client-side timing / regression compare | hap.py, GIAB publication benches, HELIOS PDF |
| 5 | Reporting | Text + JSON + skip-is-not-pass + exit mapping; `helix compare` at stable id | New conformance languages, signatures, score deltas as “regression” |
| 6 | Target adapters | How a URL becomes service bases Helix can hit | Shipping DRS/WES (Ferrum) |
| 7 | Test definitions | Stable names, order, skip/fail meaning | Vendor-specific forks in generic mode |
| 8 | CLI | `helix verify` / `security` / `bench` | `helios`, `helixtest --all` ladder |
| 9 | CI integration | Prove this repo; optional helix-action comments | Required Ferrum `main` gate before Stage 2 exit |

As-built modules (`src/discover.rs`, `verify.rs`, `profile.rs`, `compare.rs`, `diagnostics.rs`, `adapter/`, `security/`, `bench/`, `report.rs`, `main.rs`) already *name* most of these layers. HelixTest DRS/WES execution is isolated behind `src/adapter` ([HELIXTEST_ADAPTER.md](HELIXTEST_ADAPTER.md)). Profiles are declarative policy ([PROFILES.md](PROFILES.md)); the engine always uses public HTTP and `Mode::Generic`. Do not add new `framework::*` call sites outside that module. Do not merge HelixTest (D1).

---

## 3. Pipeline (intended)

```text
                    ┌─────────────────────────────────────────┐
                    │  CLI  (helix verify | security | bench | compare) │
                    └──────────────────┬──────────────────────┘
                                       │ origin URL(s), format
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │  Target adapter                         │
                    │  gateway-style http(s) origin           │
                    │  → normalized endpoint                  │
                    └──────────────────┬──────────────────────┘
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │  Discovery                              │
                    │  DRS → WES → TES → TRS → htsget         │
                    │  NOT_DETECTED / DETECTED                │
                    │  TESTABLE / NOT_TESTABLE                │
                    └──────────────────┬──────────────────────┘
                                       ▼
         ┌─────────────────────────────┼─────────────────────────────┐
         ▼                             ▼                             ▼
┌─────────────────┐         ┌─────────────────┐           ┌─────────────────┐
│ Verification    │         │ Security        │           │ Performance     │
│ engine          │         │ behaviour       │           │ measurement     │
│                 │         │                 │           │                 │
│ uses            │         │ Helix-owned     │           │ Helix-owned     │
│ Conformance     │         │ HTTP cases +    │           │ (not HelixTest) │
│ Adapter         │         │ header checks   │           │                 │
└────────┬────────┘         └────────┬────────┘           └────────┬────────┘
         │                           │                             │
         └───────────────────────────┼─────────────────────────────┘
                                     ▼
                    ┌─────────────────────────────────────────┐
                    │  Verification result                    │
                    │  (pass / fail / skip per named case)    │
                    └──────────────────┬──────────────────────┘
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │  Reporting                              │
                    │  text (TTY)  │  JSON (stdout)  │  exit  │
                    └──────────────────┬──────────────────────┘
                                       ▼
                    ┌─────────────────────────────────────────┐
                    │  Humans  /  helix-action  /  this CI    │
                    └─────────────────────────────────────────┘
```

Discovery DETECTED is not a pass. Only executed cases with `status: pass` are passes. Skip is never pass ([CLI_CONTRACT.md](CLI_CONTRACT.md), D3).

---

## 4. Layers in detail

### 4.1 Discovery

**Job:** Answer “which Stage 1 GA4GH HTTP APIs are present under this origin?”

**Helix-owned.** Order is DRS → WES → TES → TRS → htsget. Not Beacon, africa, infra, E2E.

**Input:** normalized `http`/`https` origin. **Output:** one record per Stage 1 service with `NOT_DETECTED` | `DETECTED`, and when detected `TESTABLE` | `NOT_TESTABLE`. Details: [DISCOVERY.md](DISCOVERY.md). DETECTED is not a pass.

**Presence rule (as-built, keep unless a decision changes it):** HTTP 2xx, 401, or 403 on a probe URL counts as present. Network error and 404 do not. First matching probe wins.

**Must remain implementation-agnostic:** probe published GA4GH prefixes (`/ga4gh/drs/v1`, …) plus documented split-port fallbacks. Do not probe Ferrum-only paths as the generic definition. Do not import Ferrum.

Discovery does not run checks. It does not score. It does not start servers.

### 4.2 Verification

**Job:** For each discovered service that this stage executes, run the named conformance cases and collect pass/fail/skip.

**Helix-owned orchestration.** Check *bodies* currently live in HelixTest (`framework::drs::run_drs_checks`, `framework::wes::run_wes_checks`). Helix must call them only through the **conformance adapter** (§5).

Stage 1 exit still requires DRS **and** WES against Ferrum local. TES/TRS/htsget may follow in the same stage if cheap; they are not the exit. Beacon / africa / infra wait.

If a service is missing, verification records a Fail or Skip according to the test definitions — it does not pretend the service passed. Today, missing DRS is a synthetic Fail on the first DRS name.

**Mode:** generic. Ferrum-named WES must not auto-switch Helix into Ferrum-only checks (Stage 0 already exited in HelixTest). Opt-in Ferrum modes stay in HelixTest (`--mode ferrum*`), not in `helix verify`.

### 4.3 Security behaviour verification

**Job:** Security Behavior Profile — five fail-closed HTTP invariants (valid / expired / wrong-scope / garbage / wrong-audience) as black-box tests against a target Helix did not start. Not a security audit, pentest, or certification. Passing does not prove the implementation is secure. Crypt4GH **protocol layout** runs **after** those five cases ([CRYPT4GH.md](CRYPT4GH.md)): well-formed envelope, invalid envelope rejected, HTTP envelope-or-skip. Helix does not decrypt and does not load private keys. A Crypt4GH pass is not “secure”.

**Helix-owned surface** (`helix security`). HelixTest already has HMAC fixtures and `ferrum+infra` Passport checks; those remain in HelixTest until Stage 3 exit says otherwise. Helix must not become an IdP.

Tokens used in CI are dummy HS256 from `test-fixtures/` or `HELIX_HMAC_SECRET`. They are not production keys, not Passport visas, not ga4gh-infra output. Helix may *mint test JWTs* against a caller-supplied dummy secret so the target can be exercised. Helix must not *issue* credentials for operators, hospitals, or Ferrum deployments.

Crypt4GH in Helix is protocol framing only (`HLX-AUTH-050` / `053` / `054`). HTTP Crypt4GH decrypt/rewrap against Ferrum stays HelixTest inventory (secret key required) and is not a Helix default.

Not HELIOS: no signed dance, no RO-Crate of tokens.

### 4.4 Performance measurement

**Job:** Compare two running origins on a fixed small workload. Warn on regression. Do not fail the process on threshold miss unless [CLI_CONTRACT.md](CLI_CONTRACT.md) is explicitly changed.

**Helix-owned.** Does not call HelixTest checks. Does not replace Ferrum-GA4GH-Demo hap.py / GIAB. Does not use HELIOS PDF as the report.

As-built: workload `http.drs.smoke.v1`, warmup + measured runs, distribution analysis (median / p95 where available / error-rate / optional RSS). Measurement, warning, and regression are separate. A warning is human inspection, not “incorrect” and not a verification failure. Default >10% worse = warning; does not fail CI. Cross-environment compares are marked. Contract: [BENCHMARKS.md](BENCHMARKS.md). Stage 4 exit still needs two Ferrum versions on the same runner class with stored artefacts.

### 4.5 Reporting

**Job:** Turn a verification result into:

- **Human report** — `HELIX VERIFICATION` text ([REPORT.md](REPORT.md)). Same facts as JSON. Colored PASS/FAIL/SKIP/ERROR when stdout is a TTY and `NO_COLOR` is unset. Skip is never green.
- **Machine report** — JSON on stdout. Frozen as `helix-verification-v1` ([SCHEMA.md](SCHEMA.md)). Logs on stderr.
- **CI** — exit 0 if overall status is pass (`verify`) or no `status: fail` (`security`); bench warnings do not change exit.

**`helix verify` JSON is Helix `VerificationRun`** (DRS + WES, D3 revisit). Fail/error rows for catalogued DRS/WES ids may include a deterministic `diagnostic` ([DIAGNOSTICS.md](DIAGNOSTICS.md)): expected/observed/**possible causes**, not an AI root cause. **`helix security` JSON stays HelixTest `OverallReport`**. Skip is never pass. Discovery `present`/`testable` is not a pass. Bench JSON is Helix `BenchOutcome`. A bench warning is not a verification diagnostic.

No signatures, RO-Crate, PDF, ISO 15189 scores, or “Helix-certified” marks.

### 4.6 Target adapters

**Job:** Bind Helix to a running implementation without coupling Helix to that vendor’s crates.

The only target adapter for Stages 1–4 is **HTTP origin**:

- Input: operator-supplied `http(s)` URL (gateway-style).
- Output: normalized origin for discovery.
- Optional later: explicit per-service URLs if a target is split-port only. That is still HTTP, still not a Ferrum crate.

**Ferrum adapter (conceptual, not a library):** “run Ferrum `make up`, then pass `http://127.0.0.1:8080`.” Ferrum is a reference *deployment*, not a Helix dependency.

**Non-Ferrum adapter:** any stack that answers the published GA4GH HTTP paths (or the documented split-port DRS fallback). The in-process B1 mock is the CI stand-in. Unverified `ghcr.io/example/mock-*` images are not a proof target (D2).

Helix does not start Docker. Helix does not clone Ferrum. Helix does not embed Ferrum types.

### 4.7 Test definitions

**Job:** Stable case names, service order, and skip/fail semantics so reports stay comparable across HelixTest pin bumps and across targets.

Helix **owns the contract** (names in `DRS_CHECK_NAMES`, `WES_CHECK_NAMES`, `AUTH_CASE_NAMES`, CLI_CONTRACT). HelixTest **currently implements** the DRS and WES bodies. Replacing or supplementing HelixTest means mapping the same names — not renaming cases to match a new engine.

Rules:

- Skip is not pass (`passed: true` iff `status == pass`).
- Discovered-but-unwired services are skipped with an explicit reason, not omitted as if they passed.
- Generic definitions must not require `ferrum_like` forks.
- Object id for DRS remains HelixTest’s `test-object-1` while that engine is the adapter (do not silently change the fixture id).

### 4.8 CLI

**Job:** Thin argv → layer call → print → exit.

Surfaces: `helix verify`, `helix security`, `helix bench`. Binary name is `helix`. Never `helios`. HelixTest’s `helixtest` remains the tagged Ferrum-pin CLI until Stage 2 explicitly moves Ferrum’s pin.

CLI does not start targets, does not open a server, does not bundle HELIOS.

### 4.9 CI integration

Three consumers, different jobs:

| Consumer | Role |
|----------|------|
| **This repo** (`.github/workflows/ci.yml`) | Checkout Helix + HelixTest at `VERSIONS.lock` SHA. `make prove`, `make verify-fixture`, clippy `-D warnings`, rustfmt. Secret-scan. Dependency-review (non-fatal). Proves Helix against in-process fixtures ([FIXTURES.md](FIXTURES.md)). No Ferrum required. Live `make test-live` is not CI. |
| **helix-action** (sibling repo) | Optional PR comments at stable Helix `id` (`NEW_FAIL` / `FIXED` / `UNCHANGED_FAIL`). Fail only on new verification regressions vs last successful run of that workflow, or runtime errors. Bench warnings may be comments; they must not change that compare exit. Pilot on Ferrum `ci/helix-verify-pilot` only until Stage 2 exit. Not a required Ferrum `main` check. |
| **Ferrum `main`** | Still clones tagged **HelixTest**, not Helix, until Stage 2 says otherwise. Not a Helix runtime dependency. |

Helix CI clones HelixTest as a **build-time** sibling because of D1 path deps. That is a compile pin, not “Ferrum inside Helix.”

---

## 5. HelixTest adapter boundary

### 5.1 Why an adapter

HelixTest is the existing conformance engine. For the current stage it **must remain a dependency** (path crates `helixtest-common` / `helixtest-framework`, pin [VERSIONS.lock](../VERSIONS.lock) **v0.1.3**). D1: do not merge git histories, do not vendor the suite to avoid the sibling checkout.

A clean boundary lets HelixTest later be **replaced, extended, or supplemented** without redesigning CLI, discovery, reporting, or CI comments.

### 5.2 Intended seam

All HelixTest *execution* goes through one conformance adapter. Nothing in `main.rs`, `discover.rs`, `report.rs`, `bench/`, or `security/` (except optional JWT helper, §5.4) should call `framework::*` directly.

```text
Helix verification engine (src/verify.rs)
        │
        │  ConformanceAdapter::run_drs(base_url)
        │    → ServiceReport (CLI D3) + Helix VerificationResult
        ▼
┌───────────────────────────────────────┐
│ HelixTestAdapter  (src/adapter)       │  ← current and only impl
│   framework::drs::run_drs_checks      │
│   framework::wes::… (Stage 1 exit)    │
│   Mode::Generic, Features as Helix sets│
│   translate status; never Skip → Pass │
│   pin from VERSIONS.lock              │
└───────────────────────────────────────┘
        │
        ▼
pinned HelixTest crates (separate git root)
```

Later, a second impl (in-tree cases, another engine) can sit behind the same trait. `helix verify` prints `VerificationRun`. Operators still run `helix verify <url>`.

**As-built:** `src/adapter` invokes `run_drs_checks` / `run_wes_checks` and translates into Helix `VerificationResult`. `src/verify.rs` orchestrates discovery → testable → adapter. `helix verify --format json` prints `VerificationRun`. Details: [HELIXTEST_ADAPTER.md](HELIXTEST_ADAPTER.md), [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md). This is not a HelixTest merge, and not a new product.

### 5.3 What the adapter may use from HelixTest

Allowed behind the adapter (current pin):

- `run_drs_checks` and `run_wes_checks` (TES/TRS/htsget check functions as stages wire them)
- `Features`, `Mode::Generic`
- `TestConfig` / service URLs **filled from discovery**, not from Ferrum defaults
- `HttpClient` for those checks (timeout/retry differences vs Helix discovery client are known; do not “fix” by importing Ferrum)
- `OverallReport` / `ServiceReport` / `TestCaseResult` types (D3)

Not allowed anywhere in Helix:

- HelixTest `run_all` as the Helix CLI
- `--mode ferrum` auto-switch on WES `name`
- HelixTest compose / `--start-ferrum` as Helix’s customer path
- Copying HelixTest into this git root

### 5.4 JWT helper (narrow exception)

Stage 3 mints dummy HS256 via `common::auth::build_jwt`. That is a **test-fixture helper**, not conformance execution. If HelixTest is replaced, this helper can be inlined or swapped without touching discovery or `OverallReport`. It must never mint production tokens or talk to ga4gh-infra as a library.

### 5.5 Pin

Operators and Helix CI pin **git tag / SHA**, not crate `0.1.0`. Do not bump off v0.1.3 / `1832c043e1679ec283cb2113510ee33684317cce` without a HelixTest tag that Ferrum / Lab Kit / ga4gh-infra also take.

---

## 6. What is out of this architecture

| Out | Where it lives |
|-----|----------------|
| Reproducibility, signed evidence, RO-Crate, PDF | HELIOS |
| Shipping Beacon/DRS/WES/TES | Ferrum |
| Issuing Passports / OIDC | ga4gh-infra |
| Clinical consent | Solum |
| GIAB concordance publication bench | Ferrum-GA4GH-Demo (smoke, not Helix) |
| Helix Cloud, SaaS dashboard, vendor ranking | Not on the ladder ([HELIX_ROADMAP.md](HELIX_ROADMAP.md)) |
| GA4GH certification mark | Nobody here |

---

## 7. Mapping as-built → intended

| Layer | As-built (audit) | Intended (this file) |
|-------|------------------|----------------------|
| Discovery | `src/discover.rs` | Unchanged ownership; stay Helix-native |
| Verification | `src/verify.rs` orchestrates; `src/adapter` runs HelixTest DRS | Same flow via `ConformanceAdapter` |
| Security | `src/security/` + dummy fixtures + [SECURITY_PROFILE.md](SECURITY_PROFILE.md) | Stay Helix-owned; no IdP; not an audit |
| Performance | `src/bench/` `http.drs.smoke.v1` engine | Stay Helix-owned; Stage 4 exit still required |
| Reporting | `src/report.rs` → `VerificationRun` (verify) / `CompareReport` (compare) / `OverallReport` (security) / `BenchOutcome` | Skip ≠ pass; discovery ≠ pass; regression ≠ score drop |
| Target adapter | Implicit URL string | Keep HTTP-only; Ferrum = reference URL |
| Test definitions | Constants + HelixTest names | Helix-owned contract; adapter fills bodies |
| CLI | `src/main.rs` clap | Thin; never `helios` |
| CI | prove + clippy; helix-action pilot | Same consumers; Ferrum `main` stays HelixTest until Stage 2 exit |

---

## 8. Architecture invariants

Rules for future Cursor agents and humans. Violating one is a defect, not a shortcut. These do not replace [DECISIONS.md](DECISIONS.md); they restate what the architecture forbids.

1. **HelixTest stays a separate git root (D1).** Do not merge histories, vendor the suite, or “simplify” by copying `framework/` into Helix.

2. **HelixTest is the current conformance engine, accessed through an adapter.** Do not grow new `framework::` call sites outside that boundary. Do not wrap the `helixtest` binary as a hidden second CLI.

3. **Do not bump the published HelixTest pin** off `VERSIONS.lock` unless HelixTest has a tag Ferrum / Lab Kit / ga4gh-infra can take.

4. **Ferrum is a reference target, never a Helix runtime dependency.** No Ferrum crate, no Ferrum types, no `make up` inside Helix. `make prove` must not require Ferrum.

5. **Helix must test non-Ferrum implementations.** Generic discovery and generic DRS checks must keep working against the in-tree HTTP mock. Do not gate `helix verify` on Ferrum `name` in service-info.

6. **HELIOS is not in this architecture (D4).** Refuse RO-Crate, PDF, signed evidence, Nextflow/Snakemake envelopes, and ISO 15189 / AI Act scores in Helix. Point those requests at HELIOS.

7. **Helix is not a server.** No listen port, no hosted runner, no cloud SKU in this design.

8. **Helix does not issue authentication credentials.** Dummy fixtures only, labeled not-for-production. No Passport broker, no production HMAC, no ga4gh-infra as a library.

9. **Helix does not claim certification.** Do not write “GA4GH certified”, “production deployment”, or Ferrum clinical pilot (DIZ / genomDE). Green results are a technical signal.

10. **`helix verify` JSON is Helix `VerificationRun` (D3 revisit).** `helix security` stays HelixTest `OverallReport`. Skip is never pass. Discovery is not scored as pass. Do not add HELIOS fields. Bench JSON stays `BenchOutcome`.

11. **CLI name is `helix`, never `helios`.** Do not share a binary or a report that pretends to be both VERIFY and evidence.

12. **Discovery does not imply pass.** Found services that are not executed must be skipped with a reason, not omitted.

13. **Do not start Stage *n+1* product work in place of Stage *n* exit.** Stage 1 still needs DRS and WES against Ferrum local. Do not make helix-action a required Ferrum `main` check before Stage 2 exit. Do not use unverified `ghcr.io/example/mock-*` as proof (D2).

14. **Keep the README honesty sentence** that prove greps: Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency.

15. **Do not change DRS (and later WES) case names** without treating it as a contract break for helix-action and HelixTest JSON consumers.

If a request conflicts with these invariants, stop and say so. Do not “just add it.”
