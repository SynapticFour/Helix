# Helix discovery

Discovery answers: which Stage 1 GA4GH HTTP APIs **answer** under an origin, and which of those Helix **will execute checks for**. It is not conformance. It is not certification.

`helix verify` runs DRS and WES checks after discovery when those services are TESTABLE ([DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md)). Those results are PASS/FAIL/SKIP/ERROR. Discovery states are never PASS.

Source: `src/discover.rs`. Identity catalog (not executed here): [TEST_IDENTITY.md](TEST_IDENTITY.md) `HLX-DISCOVERY-*`.

---

## Three words

| State | Meaning |
|-------|---------|
| **NOT_DETECTED** | No probe returned 2xx / 401 / 403. Helix will not treat the service as present. |
| **DETECTED** | A documented probe got 2xx, 401, or 403. The API appears to exist (or to challenge auth). **Not a pass.** |
| **TESTABLE** | DETECTED **and** `helix verify` currently **executes** checks for that service. Not “the checks passed”. |

When DETECTED but Helix does not run that suite yet, the second column is **NOT_TESTABLE** with a reason.

```text
DRS       DETECTED     TESTABLE
WES       DETECTED     TESTABLE
TES       DETECTED     NOT_TESTABLE  Helix Stage 1 does not execute TES checks; DETECTED is not a pass
TRS       DETECTED     NOT_TESTABLE  …
htsget    DETECTED     NOT_TESTABLE  …
```

Today **DRS and WES** are TESTABLE (`VERIFY_EXECUTABLE`). TES/TRS/htsget can be DETECTED and still NOT_TESTABLE. Detection rules stay the same.

Never print `found` as if it meant verified.

---

## What is recorded (per service)

| Field | Content |
|-------|---------|
| service type | `DRS` / `WES` / `TES` / `TRS` / `htsget` |
| detection | `NOT_DETECTED` \| `DETECTED` |
| testability | `TESTABLE` \| `NOT_TESTABLE` |
| reason | Why NOT_TESTABLE (always set when not testable) |
| base URL | HelixTest-style base when DETECTED (no trailing slash) |
| discovery method | Which probe won (see below) |
| HTTP status | Status of the **winning** probe |
| service-info | Extra lightweight GET (or reuse of a service-info probe): `available` only on **2xx**. Optional JSON fields: `id`, `name`, `version`, `type.artifact`, `type.version` |

Capabilities and versions are copied **only** from a 2xx JSON service-info body. Helix does **not** invent `wes` / `1.0` from `/ga4gh/wes/v1` in the URL.

---

## Probes (order unchanged)

Presence: **2xx, 401, or 403**. Network error, 404, **3xx (redirects are not followed)**, and bodies over 2 MiB: try the next probe; if none match → NOT_DETECTED. First hit wins. Endpoint URLs must not include userinfo. See [THREAT_MODEL.md](THREAT_MODEL.md).

| Service | Method id | Probe |
|---------|-----------|--------|
| DRS | `ga4gh_drs_object` | `{origin}/ga4gh/drs/v1/objects/test-object-1` |
| DRS | `ga4gh_drs_service_info` | `{origin}/ga4gh/drs/v1/service-info` |
| DRS | `split_drs_object` | `{origin}/objects/test-object-1` |
| WES | `ga4gh_wes_service_info` | `{origin}/ga4gh/wes/v1/service-info` |
| WES | `split_wes_service_info` | `{origin}/service-info` |
| TES | `ga4gh_tes_service_info` | `{origin}/ga4gh/tes/v1/service-info` |
| TES | `ga4gh_tes_tasks` | `{origin}/ga4gh/tes/v1/tasks` |
| TRS | `ga4gh_trs_service_info` | `{origin}/ga4gh/trs/v2/service-info` |
| TRS | `ga4gh_trs_tools` | `{origin}/ga4gh/trs/v2/tools` |
| htsget | `ga4gh_htsget_reads_service_info` | `{origin}/ga4gh/htsget/v1/reads/service-info` |

Split DRS does **not** treat `{origin}/service-info` as DRS service-info (that path is the WES split probe).

---

## What discovery does not do

- Run HelixTest / Helix conformance checks
- Send credentials (URL userinfo is rejected; Helix-owned probes do not follow redirects)
- Follow `Location` on Helix-owned GETs (a 302 is not DETECTED)
- Start Ferrum or any server
- Score, certify, or map ISO/AI Act
- Infer GA4GH versions from path prefixes
- Treat Ferrum `name` in service-info as a mode switch (Stage 0)

`helix security` and `helix bench` still use the same probes only to find a DRS base URL / hit fixed paths. This task does not change their cases or workloads.

---

## CLI vs JSON

**Text (`helix verify`):** one [REPORT.md](REPORT.md) document (`HELIX VERIFICATION`). Services use `NOT_DETECTED` / `DETECTED` / `TESTABLE` / `NOT_TESTABLE`. Results are PASS/FAIL/SKIP/ERROR with Helix ids. Discovery lines are never green PASS. JSON and text share those facts.

**JSON (`--format json`):** Helix `VerificationRun`. `discovery[]` has `present` / `testable`. DETECTED is not a pass. Skip is not pass.

---

## Ferrum

Ferrum is a reference **target**. Discovery uses published GA4GH prefixes plus the split-port DRS object path. It does not import Ferrum.

External implementers: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md).
