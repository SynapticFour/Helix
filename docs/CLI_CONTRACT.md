# CLI contract

Helix does not invent a second report language for **conformance**. `helix verify` and `helix security` stay HelixTest `OverallReport` ([DECISIONS.md](DECISIONS.md) D3). `helix bench` is a Helix-owned JSON shape (timing diff, not a test suite). HelixTest stays a **separate git root** at the pin in [VERSIONS.lock](../VERSIONS.lock). Stage 1 **exit** still requires DRS **and WES** against Ferrum local.

## Names

| Surface | Name | Notes |
|---------|------|--------|
| Brand | Helix | VERIFY pillar |
| Repo (CLI + docs) | `SynapticFour/Helix` | This repo |
| Repo (tagged suite) | `SynapticFour/HelixTest` | Stay separate ([DECISIONS.md](DECISIONS.md) D1) |
| HelixTest binary | `helixtest` | Requires `--all` or it no-ops (exit 0) |
| Helix binary | `helix verify` / `helix security` / `helix bench` | Never named `helios` |

## Invocation

```text
helix verify <url>
helix verify <url> --format json
helix security <url>
helix security <url> --format json --hmac-secret-file test-fixtures/hmac/shared-secret.txt
helix bench --baseline <url> --candidate <url>
helix bench --baseline <url> --candidate <url> --format json --threshold 10
```

Checks wired today: **DRS** on `verify`; **Stage 3 auth behaviour** on `helix security` (dummy HMAC fixtures, not production keys); **Stage 4 scaffold** on `helix bench` (three small GETs). WES/TES/TRS/htsget checks are not run yet on `verify`.

`helix bench` is client-side timing of `GET /health`, `GET /ga4gh/drs/v1/service-info`, and `GET /ga4gh/drs/v1/objects/test-object-1`. Same request count as Demo DRS micro `n=3`, **not** Demo hap.py / GIAB, **not** a publication benchmark, **not** HELIOS.

Ferrum local (`make up`) is the Stage 1 proof target, not a clinical site.

HelixTest equivalent for DRS only:

```text
helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

## Reports

- **Terminal (`verify` / `security`):** discovery listing plus colored PASS / FAIL / SKIP (color when stdout is a TTY and `NO_COLOR` is unset). Skip is never painted as PASS.
- **JSON (`verify` / `security`):** HelixTest `--report json` shape (`OverallReport`: `services`, per-test `status` pass/fail/skip, `passed`, levels). Helix must not treat Skip as Pass.
- **JSON (`bench`):** Helix `BenchOutcome` (`workload`, `threshold_pct`, `warning`, `baseline` / `candidate` samples, `diff` with percent change, `warnings`). Not `OverallReport`.
- **Stdout vs stderr:** JSON on stdout; logs on stderr (HelixTest `HttpClient` logging).

No RO-Crate, PDF, signatures, or ISO/AI-Act scores ([HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

## Exit codes

| Code | Command | Meaning |
|------|---------|---------|
| 0 | `verify` / `security` | No `status: fail` in executed checks |
| 0 | `bench` | Finished. `warning: true` is **not** a failure (humans review CI comments) |
| 1 | `verify` / `security` | At least one FAIL, DRS not discovered, or a usage/runtime error |
| 1 | `bench` | Usage or runtime error only (unreachable URL parser, HTTP client build). Never a threshold miss |

helix-action may append bench warnings to the PR comment. That path must not change the compare-script exit code (still PASS → FAIL only).

## Pins

Operators and Helix CI pin **git tag / SHA**, not the Cargo crate `0.1.0`. See `VERSIONS.lock`. Ferrum continues to pin HelixTest until Stage 2 says otherwise.
