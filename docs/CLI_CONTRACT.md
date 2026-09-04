# CLI contract

Helix does not invent a second report language. This is the contract for `helix verify`. HelixTest stays a **separate git root** at the pin in [VERSIONS.lock](../VERSIONS.lock). Stage 1 **exit** still requires DRS **and WES** against Ferrum local.

## Names

| Surface | Name | Notes |
|---------|------|--------|
| Brand | Helix | VERIFY pillar |
| Repo (CLI + docs) | `SynapticFour/Helix` | This repo |
| Repo (tagged suite) | `SynapticFour/HelixTest` | Stay separate ([DECISIONS.md](DECISIONS.md) D1) |
| HelixTest binary | `helixtest` | Requires `--all` or it no-ops (exit 0) |
| Helix binary | `helix verify <url>` | Never named `helios` |

## Invocation

```text
helix verify <url>
helix verify <url> --format json
helix verify <url> --report json
```

`<url>` is a gateway-style base (e.g. `http://127.0.0.1:8080`). Discovery order: DRS → WES → TES → TRS → htsget. **Checks wired today: DRS only** (existing HelixTest `run_drs_checks`, generic mode, `strict_drs_checksums`). WES/TES/TRS/htsget checks are not run yet; if those APIs are discovered they are recorded as **skipped**, not passed.

Ferrum local (`make up`) is the Stage 1 proof target, not a clinical site.

HelixTest equivalent for DRS only:

```text
helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

## Reports

- **Terminal:** discovery listing plus colored PASS / FAIL / SKIP (color when stdout is a TTY and `NO_COLOR` is unset). Skip is never painted as PASS.
- **JSON:** HelixTest `--report json` shape (`OverallReport`: `services`, per-test `status` pass/fail/skip, `passed`, levels). Helix must not treat Skip as Pass.
- **Stdout vs stderr:** JSON on stdout; logs on stderr (HelixTest `HttpClient` logging).

No RO-Crate, PDF, signatures, or ISO/AI-Act scores ([HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | No `status: fail` in executed HelixTest checks |
| 1 | At least one FAIL, DRS not discovered, or a usage/runtime error |

## Pins

Operators and Helix CI pin **git tag / SHA**, not the Cargo crate `0.1.0`. See `VERSIONS.lock`. Ferrum continues to pin HelixTest until Stage 2 says otherwise.
