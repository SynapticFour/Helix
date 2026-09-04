# Helix profiles

A **profile** is a small, explicit policy for `helix verify`. It is not a plugin system, not a second test engine, and not HelixTest `--mode ferrum`.

HelixTest already runs DRS and WES checks. A profile only says **which of those public checks to run**, **what the target is expected to expose**, and **which HelixTest `Features` bits to set**. Execution stays `Mode::Generic` against discovered HTTP URLs.

Ferrum is a **reference target**. The `ferrum` profile encodes Ferrum-shaped *expectations* (from HelixTest `profiles/ferrum.toml` features Helix actually runs). It does **not** import Ferrum, switch mode on WES `name`, or call Ferrum-only APIs.

Source: `src/profile.rs`. CLI: `helix verify <url> [--profile generic|ferrum]`. Default **`generic`**.

Not certification. Not HELIOS.

---

## What a profile declares

| Field | Meaning |
|-------|---------|
| Expected services | Must be DETECTED. Missing → **fail** (not skip). Empty = none required. |
| Enabled services | Suites Helix executes when DETECTED + TESTABLE. Today: DRS and WES only. |
| Optional checks | Helix ids that may **skip** when a capability is off. Skip is never pass. |
| Capabilities | HelixTest `Features`: `strict_drs_checksums`, `supports_scatter_gather`. |
| Required fixtures | HelixTest fixture ids the target must satisfy when those checks run (documented; HelixTest still hardcodes the HTTP). |

**Severity**

| Event | Status | Blocks exit 0? |
|-------|--------|----------------|
| Enabled check assertion failed | `fail` | yes |
| Runner could not execute (unreachable, adapter error) | `error` | yes |
| Expected service missing | `fail` | yes |
| Optional check skipped (capability off) | `skip` | no |
| Enabled service not present and **not** expected | `skip` | no (skip-only run still exits 1) |

Catalog `severity` on a fail is unchanged ([TEST_IDENTITY.md](TEST_IDENTITY.md)). Profiles do not invent new codes.

TES / TRS / htsget are **not** enabled. Discovery still records them. They are not expected by either profile today.

---

## `generic` (default)

Public GA4GH HTTP. No service is required to answer. A DRS-only target can still pass (WES skipped). External origin: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md).

| | |
|--|--|
| Expected | *(none)* |
| Enabled | DRS, WES |
| Optional | `wes.run.scatter_gather` (`HLX-WES-008`) |
| Capabilities | `strict_drs_checksums=true`, `supports_scatter_gather=false` |
| Fixtures | DRS `test-object-1`; WES `trs://test-tool/{echo,fail,cwl-echo}/1.0` and `trs://nonexistent/invalid/0.0` ([WES.md](WES.md), [DRS_PROFILE.md](DRS_PROFILE.md)) |

Checksums are on (same as HelixTest `ga4gh-drs`, not HelixTest `generic.toml` where checksums default off). Scatter/gather stays skipped until a profile turns the capability on. Helix does not invent scatter fixtures for generic.

A WES `service-info` `name` of “Ferrum Gateway” does **not** switch this profile to `ferrum`.

---

## `ferrum`

Opt-in. Same public HTTP as `generic`. Operator must pass `--profile ferrum`.

HelixTest `profiles/ferrum.toml` sets `supports_scatter_gather = true` and `strict_drs_checksums = true` because Ferrum’s demo WES stubs those synthetic TRS tools. Helix copies **those feature bits** and expects DRS + WES to be present (Stage 1 proof target). It still calls `run_drs_checks` / `run_wes_checks` with **`Mode::Generic`**.

| | |
|--|--|
| Expected | DRS, WES |
| Enabled | DRS, WES |
| Optional | *(none)* — scatter/gather is enabled, not skipped |
| Capabilities | `strict_drs_checksums=true`, `supports_scatter_gather=true` |
| Fixtures | generic WES/DRS fixtures **plus** `trs://test-tool/scatter-gather/1.0` (`outputs.scatter_result`) |

TES, TRS, htsget, Beacon, and auth are listed in HelixTest `ferrum.toml` URLs. Helix does **not** execute those suites yet and does **not** fail the profile if they are absent.

Missing DRS or WES under `--profile ferrum` is **fail** (`… expected by profile ferrum but not detected`), not skip.

`supports_beacon_v2` from HelixTest ferrum.toml is **not** set: Helix does not run Beacon.

---

## Engine rules (non-negotiable)

1. Always `Mode::Generic`. Never `Mode::Ferrum` / `FerrumAfrica` / `FerrumInfra`.
2. Never choose a profile from service-info `name`, URL path, or “looks like Ferrum”.
3. Discovery probes are unchanged ([DISCOVERY.md](DISCOVERY.md)). TESTABLE is engine capability, not a Ferrum flag.
4. Adapter URLs come from discovery, not from Ferrum localhost defaults.
5. Adding a profile is a new `const Profile` in `src/profile.rs`, not a dynamic loader.

---

## CLI and JSON

```text
helix verify <url>                     # profile generic
helix verify <url> --profile generic
helix verify <url> --profile ferrum
```

JSON `profile` is `generic` or `ferrum` ([CLI_CONTRACT.md](CLI_CONTRACT.md)). Per-check `profile: "generic"` on translated rows still means HelixTest **Mode::Generic**, not the Helix profile id.

Unknown `--profile` is a clap usage error (exit 2).

---

## Out of this mechanism

- Plugin / WASM / user TOML loaders
- HelixTest `--mode ferrum*`
- Importing Ferrum
- Auto-switch on Ferrum Gateway
- HELIOS evidence
- TES / TRS / htsget / Beacon execution
