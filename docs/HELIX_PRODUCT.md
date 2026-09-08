# Helix

Helix is verification infrastructure for genomic and clinical computing.

You point it at a running implementation. It tests that implementation against an explicitly selected standard and version, then records what was tested, what can be claimed, and what was left unevaluated.

Helix is the standalone `helix` command-line product built from [HelixTest](https://github.com/SynapticFour/HelixTest). It is not a new test platform. It is not official [GA4GH](https://www.ga4gh.org/) certification. It is not [HELIOS](https://github.com/SynapticFour/HELIOS).

**Vertraue mir nicht, vertraue dem Code.** Do not take this page on trust. The commands, reports, and linked technical documents are the evidence.

This page answers: **is Helix useful to me right now?** How to run it: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). Five-minute clone-and-run: [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

---

## What Helix is

Helix tests a real implementation against a defined technical contract and produces evidence showing what was tested and what can actually be claimed.

Today that contract is a **bounded technical verification of GA4GH DRS 1.4.0**. Helix also runs documented DRS and WES checks without selecting a standard version. Those unversioned runs are useful as tests. They are not a named-release verification claim.

Helix is a client, not a server. It does not replace the implementation under test. It does not start Ferrum or any other stack. You bring an HTTP origin (or use the in-process fixture).

---

## Why Helix exists

Standards compliance is often treated as a yes/no property. Real implementations differ: they advertise different versions, implement different parts of a specification, skip optional operations, and fail in ways that a single “compliant” boolean hides.

Helix exists to make those differences **visible and testable**:

- concrete behaviour on a live HTTP origin;
- an explicit selected standard and version (when you ask for one);
- a recorded boundary between tested, verified, unevaluated, and out of scope;
- an inspectable evidence file rather than an assertion.

It does not invent a market problem. It productizes a check engine that already exists (HelixTest) so a result can be interpreted without reading source.

---

## Who Helix is for

Helix is for people who need **technical evidence** that a genomic HTTP service behaves as expected — not a certificate.

Typical situations Helix can address today:

- “We built a DRS service and want to test interoperability against GA4GH DRS 1.4.0.”
- “We are evaluating a genomic data service and need more than a green/red badge.”
- “We want a retained JSON file attached to a release, and we want to know later whether that file is still current verifier evidence.”
- “We need to know which standard version Helix selected, as distinct from what the service advertised.”
- “A check failed and we want attribution (target vs configuration vs missing fixture), not a single compliance bit.”

It is also for evaluators, architects, and interoperability specialists who will read the linked technical documents after this page.

Helix is **not** for deploying DRS/WES (that is an implementation such as Ferrum), issuing Passports, signing pipeline runs (HELIOS), or clinical consent/policy (Solum).

---

## What Helix can do today

| Status | Capability |
|--------|------------|
| **Available now** | Versioned technical verification of **GA4GH DRS 1.4.0** within a declared coverage boundary. Unversioned DRS and WES checks. Human report and `helix-verification-v1` JSON. `helix inspect` of a retained file. Compare two runs for check-level regression. List pinned specification releases. In-process fixture so you can try Helix without a live stack. |
| **Partially available / deliberately bounded** | DRS 1.4.0 coverage is **partial**. Independent-implementation evidence exists for two local DRS targets (not CI, not a ranking). `helix security` is selected dummy-HMAC behaviour, not a security audit. `helix bench` is a small HTTP smoke measurement, not verification. TES/TRS/htsget may be discovered and are not executed. |
| **Not yet available** | Versioned verification of DRS 1.5.0, WES, TES, TRS, htsget, or Beacon. Exhaustive DRS 1.4.0 coverage. Signed evidence, RO-Crate, or PDF. A published binary on crates.io. Official GA4GH certification. |
| **Deferred** | **Authorization verification.** Investigated and closed until a usable live authorization environment exists. |

A registry row that is **AVAILABLE** is a pinned specification Helix knows about. It is not **SUPPORTED**. **SUPPORTED** is not **VERIFIED**.

---

## What a Helix result means

### Tested vs verified

A **PASS** is one check outcome. Several checks can pass without establishing the higher-level verification claim.

**VERIFIED** (`ga4gh_requirement`) means the current Helix verification contract’s predicates were satisfied for the selected standard and version, the recorded target and fixture, and the current coverage boundary.

It does **not** mean:

- full DRS 1.4.0 compliance;
- production readiness;
- that the service is secure;
- that authorization was tested;
- official GA4GH certification or endorsement.

Process **exit 0** means at least one executed check passed and none failed or errored. That is **not** VERIFIED. Default `helix verify URL` can exit 0 with every claim **NOT_VERIFIED**, because it did not select a standard pack.

### Unevaluated

**Unevaluated** means Helix deliberately made **no claim** about that area.

It is not PASS. It is not FAIL. It is not “broken,” “secure,” or “insecure.”

Authorization is unevaluated. Bulk DRS operations, access URLs, object OPTIONS, Passport POST, other checksum types, bundle contents, and TLS-as-a-DRS-requirement are unevaluated. Those are coverage boundaries, not bugs.

### Detected, selected, verified

These four version fields must not be collapsed:

| Word | Meaning |
|------|---------|
| Declared | What the operator said about the target (untrusted) |
| Detected | What the service advertised (for example in service-info) |
| Selected | The standard version Helix actually tested against |
| Verified | Set only when the verification claim is justified; otherwise empty |

A service that advertises `1.3.0experimental` can still be tested against selected DRS 1.4.0. That does not make it DRS 1.4.0 verified.

### Current vs historical evidence

A retained JSON file can be reloaded with `helix inspect`. Helix classifies it as **current verifier evidence**, a **historical observation**, or **invalid**. Standing is computed against the binary you are running. It is not stored in the file. Historical files remain inspectable. They are not silently treated as current.

---

## What you can try today

Prerequisites, in ordinary language: a Rust toolchain, a checkout of Helix next to HelixTest at the pin Helix records, and either a DRS HTTP origin you started or the in-process fixture.

```text
This page
    ↓
[INSTALL.md](INSTALL.md) (source build; no published binary)
    ↓
[OPERATOR_VERIFY.md](OPERATOR_VERIFY.md)
    ↓
helix verify URL --standard drs --version 1.4.0 --output verify.json
    ↓
helix inspect verify.json
    ↓
Read Claims, Coverage, Results, and standing
```

Minimum versioned command (actual CLI):

```bash
NO_COLOR=1 helix verify URL --standard drs --version 1.4.0 --output verify.json
helix inspect verify.json
```

No live DRS:

```bash
make fetch
make verify-drs
```

`make verify-drs` runs DRS 1.4.0 against the in-process mock, writes `verify.json`, and inspects it. That is not independent-implementation evidence. `make verify-fixture` remains the unversioned human report.

Default `helix verify URL` does **not** select DRS 1.4.0. That is intentional.

Helix does not send credentials.

---

## Current DRS 1.4.0 coverage

DRS 1.4.0 verification is currently **partial**.

Helix currently evaluates a small catalog around a known DRS object: reachability, schema (including the pinned object schema), checksum, HTTP Range, and unknown-id 404. Those catalog rows must pass for the DRS 1.4.0 verification claim. Passing them still leaves other DRS operations **unevaluated**.

Major unevaluated areas (not silently verified):

- service-info as a DRS operation;
- bulk object operations;
- access and bulk access;
- object OPTIONS;
- object POST / Passport;
- **authorization**;
- checksum types other than the exercised sha256 path;
- bundle contents;
- TLS as a DRS requirement;
- HTTP timeout behaviour as a DRS requirement.

Honest catalog success is therefore `coverage.state = partial`. That is the intended boundary. Detail: [COVERAGE.md](COVERAGE.md).

---

## Independent implementations

The same DRS 1.4.0 contract has been applied to two reviewed independent local implementations. Helix does not rank them. Different outcomes are observed facts, not a winner.

| Target | Observed standing (operator evidence, not CI) |
|--------|--------------------------------------------------|
| GA4GH Starter Kit DRS 0.3.2 | Detected `1.3.0experimental`, selected 1.4.0, **not verified**. Some checks pass; schema/`access_methods` fails; checksum/Range skip when the configured object is not a usable blob fixture. |
| Bento DRS v0.21.5 | Detected, selected, and **verified** 1.4.0 under the current contract, with **partial** coverage. Authorization was not enabled on that run and remains unevaluated. |

In-process mocks prove the harness. They are not a second implementation. `helix matrix` is a labeling harness; completed public multi-implementation validation in CI is **not** claimed. Detail: [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md), [INTEROP.md](INTEROP.md).

---

## Evidence

Every `helix verify --format json` run produces a `helix-verification-v1` artifact. From that file, without reading Helix source, you can determine:

- which target was tested;
- which standard and version were selected;
- which checks ran and which passed, failed, or skipped;
- whether the higher-level claim is VERIFIED;
- what remains unevaluated;
- whether the file is current verifier evidence or a historical observation (via `helix inspect`).

The artifact is inspectable and tamper-checked. It is **not** a signed audit pack. Signing, RO-Crate, and PDF stay with HELIOS.

Schema and report layout: [SCHEMA.md](SCHEMA.md), [REPORT.md](REPORT.md). Durability: [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md). Operator presentation: [B15_OPERATOR_VERIFICATION_UX.md](B15_OPERATOR_VERIFICATION_UX.md).

---

## What Helix does not do yet

| Area | Status |
|-------|--------|
| Broader DRS 1.4.0 operations (bulk, access, OPTIONS, Passport, …) | Not yet available (unevaluated by design) |
| Authorization / authentication verification | **Deferred** (see below) |
| DRS 1.5.0 as a supported verification pack | Not yet available (AVAILABLE in the registry) |
| Versioned WES / TES / TRS / htsget / Beacon | Not yet available |
| Additional independent implementations in CI | Not yet available |
| Signed evidence, RO-Crate, PDF | Not Helix (HELIOS) |
| Result cache, telemetry, remote upload | Not available; not planned as Helix features |
| Ranking or “Helix-certified” marks | Will not be added |

No dates. A possibility is not a commitment.

---

## Authorization verification is currently deferred

**Authorization verification is currently deferred.**

Helix investigated a real DRS authorization architecture (Bento DRS delegates policy to a separate authorization service). The demonstrated targets did not provide a suitable live, independently verifiable authorization environment. Helix `verify` does not send credentials. The coverage row stays unevaluated.

This is **not** an unfinished Helix feature in progress. It is deliberately closed until an appropriate usable project exists. It will be reopened then.

[AUTHORIZATION.md](AUTHORIZATION.md) · [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md)

---

## Related projects

| Project | Role |
|---------|------|
| **Helix** | Technical verification of a running implementation. This product. |
| **HelixTest** | Existing check engine Helix wraps. Separate repository. |
| **HELIOS** | Durable, signed reproducibility and audit evidence (RO-Crate, PDF). Not this binary. |
| **Ferrum** | Reference GA4GH runtime / implementation (BUSL-1.1). Optional live target. Not a Helix dependency. No clinical pilot. |
| **Solum** | Clinical-data compliance layer (policy, consent, interchange). Separate product. Helix does not implement it. |

Helix answers *whether* a running system behaves on a documented contract. HELIOS answers *what* ran and *how* to reproduce a pipeline. Split: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md). Portfolio map: [ECOSYSTEM.md](ECOSYSTEM.md).

---

## Trust and provenance

Helix does not simply say “DRS 1.4.0 passed.” It records which standard release was selected, which specification bytes were used, which checker ran, which target and fixture were involved, and which checks justified the claim.

A technically interested reader can drill down:

```text
This product claim
    ↓
Evidence artifact (`helix verify --format json` / helix inspect)
    ↓
Verification contract ([CLAIMS.md](CLAIMS.md), [COVERAGE.md](COVERAGE.md))
    ↓
Technical gates (linked below)
```

Do not ask anyone to trust Helix or its authors. Inspect the repo and the artifact. [TRUST.md](TRUST.md).

Helix is early-stage, Apache-2.0, single-steward. Green CI is a technical signal, not certification.

---

## Status vocabulary

User-facing words on this page, and how they relate to Helix output:

| Word | Meaning here | Related Helix terms |
|------|----------------|---------------------|
| **Available** | You can use it today | Shipped CLI / supported pack |
| **Partial** | Function exists with an explicit boundary | `coverage.state = partial` |
| **Unevaluated** | Helix makes no claim | coverage class `unevaluated` |
| **Not yet available** | Not implemented or not supported as a verification pack | Registry **AVAILABLE** but not **SUPPORTED** |
| **Deferred** | Postponed until a prerequisite exists | B13 authorization |
| **SUPPORTED** | Helix may select this pack for technical verification | Not VERIFIED |
| **PASS / FAIL / SKIP** | One check outcome | Not a certification result |
| **VERIFIED / NOT_VERIFIED** | Derived claim | `ga4gh_requirement` |
| **Current verifier evidence** | Artifact cites this Helix build | Computed standing; not a JSON field |
| **Historical observation** | Inspectable; not this build | Same |

---

## Current implementation status

| Capability | Status | What you can do | Important limitation | Technical documentation |
|-----------|--------|------------------|----------------------|-------------------------|
| DRS 1.4.0 technical verification | Available, partial | `helix verify URL --standard drs --version 1.4.0` | Partial coverage; not certification | [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md), [COVERAGE.md](COVERAGE.md), [CLAIMS.md](CLAIMS.md) |
| Unversioned DRS/WES checks | Available | `helix verify URL` | Does not select a GA4GH pack; exit 0 is not VERIFIED | [CLI_CONTRACT.md](CLI_CONTRACT.md), [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md) |
| Standard selection / provenance | Available | `helix standards list --supported-only` | DRS 1.5.0 and WES 1.1.0 are AVAILABLE only | [STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md), [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md) |
| Evidence artifact | Available | `--output FILE` or `--format json`; `helix inspect FILE` | Unsigned; not HELIOS | [SCHEMA.md](SCHEMA.md), [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md), [B15_OPERATOR_VERIFICATION_UX.md](B15_OPERATOR_VERIFICATION_UX.md) |
| Independent DRS targets | Partial | Same verify path against Starter Kit / Bento | Operator evidence, not CI; not a ranking | [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md), [TARGETS.md](TARGETS.md) |
| Check-level compare | Available | `helix compare`, `helix differential` | Compare ≠ verify; differential ≠ ranking | [REGRESSION.md](REGRESSION.md), [DIFFERENTIAL.md](DIFFERENTIAL.md) |
| Interop matrix | Partial | `helix matrix` with operator-supplied runs | Public slots pending without independent JSON in-repo | [INTEROP.md](INTEROP.md) |
| Security behaviour | Partial | `helix security` with dummy HMAC | Not DRS authorization; not a pentest | [SECURITY_PROFILE.md](SECURITY_PROFILE.md) |
| HTTP smoke measurement | Available | `helix bench` | Not a verification claim | [BENCHMARKS.md](BENCHMARKS.md) |
| TES/TRS/htsget | Discovery only | May appear as DETECTED | Not executed | [DISCOVERY.md](DISCOVERY.md) |
| Authorization | Deferred | No claim | Unevaluated; verify sends no credentials | [AUTHORIZATION.md](AUTHORIZATION.md), [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md) |
| Signed audit / RO-Crate / PDF | Not Helix | Use HELIOS | Out of Helix scope | [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) |

---

## For technical readers

Do not start here if you only need the product picture above.

| Topic | Document |
|-------|----------|
| Operator workflow | [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md) |
| Trust / independent inspection | [TRUST.md](TRUST.md) |
| Claims (VERIFIED predicates) | [CLAIMS.md](CLAIMS.md) |
| Coverage boundary | [COVERAGE.md](COVERAGE.md) |
| Evidence durability | [B14_EVIDENCE_DURABILITY.md](B14_EVIDENCE_DURABILITY.md) |
| Operator UX / inspect | [B15_OPERATOR_VERIFICATION_UX.md](B15_OPERATOR_VERIFICATION_UX.md) |
| Independent implementations | [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md), [EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md) |
| Authorization feasibility | [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md) |
| Negative controls | [NEGATIVE_CONTROL.md](NEGATIVE_CONTROL.md) |
| CLI freeze | [CLI_CONTRACT.md](CLI_CONTRACT.md) |
| Helix vs HELIOS | [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md) |
| Positioning / stages | [HELIX_VISION.md](HELIX_VISION.md), [HELIX_ROADMAP.md](HELIX_ROADMAP.md) (stages, not dates) |
