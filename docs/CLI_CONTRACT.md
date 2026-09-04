# CLI contract

Helix does not invent a second report language. This is the contract for Stage 1 `helix verify`. Until that binary exists, the live CLI is **`helixtest`** from [HelixTest](https://github.com/SynapticFour/HelixTest) at the pin in [VERSIONS.lock](../VERSIONS.lock).

## Names

| Surface | Name | Notes |
|---------|------|--------|
| Brand | Helix | VERIFY pillar |
| Repo (docs / later wrapper) | `SynapticFour/Helix` | This repo |
| Repo (tagged suite) | `SynapticFour/HelixTest` | Stay separate ([DECISIONS.md](DECISIONS.md) D1) |
| Live binary today | `helixtest` | Requires `--all` or it no-ops (exit 0) |
| Stage 1 binary | `helix verify <url>` | Wrapper or thin CLI; never named `helios` |

## Invocation (Stage 1 exit)

```text
helix verify <url>
```

`<url>` is a gateway-style base (e.g. `http://127.0.0.1:8080`). The implementation maps it onto at least **DRS** and **WES** (priority order for later surfaces: TES → TRS → htsget). Ferrum local (`make up`) is the Stage 1 proof target, not a clinical site.

Until `helix` exists, the equivalent is:

```text
helixtest --all --mode generic --only drs --report json
```

Stage 1 in this repo starts with **discovery**: `helix verify <url>` probes public GA4GH HTTP paths (DRS first). HelixTest checks are added next, DRS then WES.

## Reports

- **Terminal:** human table (HelixTest `--report table`).
- **JSON:** HelixTest `--report json` shape (`services`, per-test `status` Pass/Fail/Skip, levels, scores). Helix must not treat Skip as Pass.
- **Stdout vs stderr:** JSON on stdout; logs on stderr (HelixTest v0.1.3).

No RO-Crate, PDF, signatures, or ISO/AI-Act scores ([HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

## Exit codes

Same as HelixTest:

| Code | Meaning |
|------|---------|
| 0 | Suite met the fail-level (and no hard failures, per HelixTest rules) |
| 1 | Failures, usage error, or overall level below `--fail-level` |

## Pins

Operators and Helix CI pin **git tag / SHA**, not the Cargo crate `0.1.0`. See `VERSIONS.lock`. Ferrum continues to pin HelixTest until Stage 2 says otherwise.
