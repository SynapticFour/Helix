# Operator DRS 1.4.0 verification workflow

Helix is HelixTest becoming a standalone VERIFY CLI. This page is the operator path for **technical verification** of supported GA4GH DRS 1.4.0. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Five-minute briefing: [FOR-EVALUATORS.md](FOR-EVALUATORS.md). Claims: [CLAIMS.md](CLAIMS.md). Coverage: [COVERAGE.md](COVERAGE.md). Evidence durability: [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md).

---

## 1. Prerequisites

- Rust via rustup (`rust-toolchain.toml`).
- Sibling HelixTest at the SHA in [VERSIONS.lock](../VERSIONS.lock).
- `make fetch` once, then offline `make prove`.
- A DRS HTTP origin **you** started, **or** `make verify-fixture` (in-process mock; not independent evidence).

Helix does not send credentials. Authorization stays unevaluated ([AUTHORIZATION.md](AUTHORIZATION.md)).

---

## 2. Supported standard / version

```text
helix standards list --supported-only
```

Today that lists **ga4gh.drs.1.4.0**. SUPPORTED is not VERIFIED. Default `helix verify URL` is **unversioned** and does not select this pack.

---

## 3. Target and fixture

Operator-declared, untrusted metadata:

```text
--target-id
--target-kind
--implementation-name
--implementation-version
```

`--implementation-version` never becomes `verified_version`.

DRS test input (not a GA4GH MUST):

```text
--drs-object-id
--drs-object-sha256
```

Default object id is `test-object-1` (catalog). Independent implementations use their own object id. A 404 on the configured id is `fixture_unavailable`, not DRS non-conformance.

---

## 4. Exact verify command

Technical verification against DRS 1.4.0:

```bash
NO_COLOR=1 helix verify URL \
  --standard drs --version 1.4.0 \
  --target-id YOUR-ID \
  --target-kind real-independent-local-implementation \
  --drs-object-id YOUR-OBJECT-ID \
  --format json > verify.json
```

Human-readable (same facts):

```bash
NO_COLOR=1 helix verify URL --standard drs --version 1.4.0 --format text
```

Retain `verify.json`. Classify later:

```bash
helix inspect verify.json
```

---

## 5. How to read the result

| Word | Means | Does not mean |
|------|--------|----------------|
| PASS / FAIL / SKIP / ERROR | One check outcome | The target is GA4GH-certified |
| VERIFIED / NOT_VERIFIED | Derived claim (`claims[]`) | Every DRS operation was tested |
| selected | Pack Helix chose | The target declared that version |
| detected | service-info `type.version` | Selected or verified |
| verified_version | Claim output when predicates hold | Full-standard compliance |
| coverage.state = partial | Required catalog rows passed; OpenAPI ops remain unevaluated | Complete DRS |
| Exit 0 | ≥1 PASS and no FAIL/ERROR | `ga4gh_requirement` VERIFIED |
| current_verifier_evidence | Artifact cites **this** binary’s git SHA | Historical files are worthless |
| historical_observation | Inspectable; not this build | Forged VERIFIED |

Attribution on FAIL/SKIP (`target_failure`, `spec_failure`, `target_configuration_failure`, …) is in JSON `executed[].attribution` and in the text report.

Unevaluated rows (service-info, bulk, access, authorization, …) are listed under `Coverage:`. They are not silently VERIFIED.

---

## 6. Evidence artifact

`--format json` is `helix-verification-v1` (`VerificationRun`). Reload:

```bash
helix inspect verify.json
helix compare previous.json current.json
```

`helix inspect` does **not** rewrite the file. Standing is computed against this binary and is **not** a JSON field (no schema bump). Forged `claims[]` / `verified_version` / coverage / check status cannot manufacture current verification ([B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md)).

---

## 7. What Helix does not do

- Official GA4GH certification
- Authorization / authentication evidence (B13 deferred)
- Signing, RO-Crate, PDF (HELIOS)
- Result cache, telemetry, remote upload
- Ranking implementations
