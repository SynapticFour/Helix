# External target contract

Helix is HelixTest becoming a standalone VERIFY CLI (pin **v0.1.3**). HelixTest already runs; this document productizes the **HTTP surface** an external implementer must expose so:

```bash
helix verify <url>
```

can execute against a generic DRS and/or WES. It is not a new test platform. A green run is a technical signal, not GA4GH certification. HELIOS (`helios-audit`) is out of scope (no signed evidence, RO-Crate, or PDF).

This contract is **implementation-neutral**. The target must not need Ferrum, Synaptic Four, HelixTest crates or TOML, special headers, private APIs, or Synaptic Four configuration.

Specs referenced: [GA4GH DRS](https://ga4gh.github.io/data-repository-service-schemas/) OpenAPI `DrsObject` / GetObject; [GA4GH WES](https://ga4gh.github.io/workflow-execution-service-schemas/) OpenAPI ServiceInfo / RunWorkflow / GetRunStatus / GetRunLog. Helix does not add fields, status codes, or headers beyond those specs in the **standard** sections below.

---

## What the target must not need

| Must not require | Why |
|------------------|-----|
| Ferrum (or any named product) | Ferrum is a reference live target only. `--profile ferrum` is not this contract. |
| Synaptic Four accounts, licences, or config files | Public GA4GH HTTP only. |
| HelixTest internals | No `Mode`, `Features` TOML, `TestConfig`, crate pins, or HelixTest JSON on the wire. |
| Special HTTP headers | `helix verify` sends ordinary GET/POST. No `Authorization`, Passport, HMAC, or vendor `X-*` headers. |
| Private or vendor APIs | No `/admin`, compact-id resolvers, or product-specific paths. `access_url` may be any http(s) URL; the in-tree mock’s `/bytes/…` path is **not** required. |
| A particular `service-info.name` | Helix never chooses a profile from `name`. “Ferrum Gateway” does nothing. |

Unauthenticated HTTP is what this command issues. If the origin challenges with 401/403, discovery may still mark the service DETECTED; executed checks that need 2xx JSON will fail. Auth behaviour is `helix security`, not this contract.

---

## Command and URL

```text
helix verify <url>
```

`<url>` is an `http://` or `https://` origin (no userinfo, no trailing-slash requirement). Helix does not start a server.

Default profile is **`generic`**: no service is required to be present. A DRS-only origin can still exit 0. A WES-only origin can still exit 0. Skip-only (neither DRS nor WES) exits 1. DETECTED is not a pass. Skip is never pass.

TES, TRS, and htsget may be probed for presence. They are **not** executed.

---

## Discovery layouts

Helix looks for published GA4GH HTTP under the origin. First probe that returns **2xx, 401, or 403** wins. Redirects are not followed. Details: [DISCOVERY.md](DISCOVERY.md).

| Service | Layout | Probe |
|---------|--------|--------|
| DRS | Prefixed | `GET {url}/ga4gh/drs/v1/objects/{drs_object_id}` then `GET {url}/ga4gh/drs/v1/service-info` |
| DRS | Split (base = origin) | `GET {url}/objects/{drs_object_id}` |
| WES | Prefixed | `GET {url}/ga4gh/wes/v1/service-info` |
| WES | Split (base = origin) | `GET {url}/service-info` |

Split DRS does **not** treat `{url}/service-info` as DRS (that path is the WES split probe).

Either layout is enough. An implementer does not need both. Prefixed `/ga4gh/{api}/v1` is the usual GA4GH path convention, not a vendor prefix.

When DRS is DETECTED, checks use `{drs_base}/objects/{id}` with `drs_base` = `{url}/ga4gh/drs/v1` or `{url}`. When WES is DETECTED, checks use `{wes_base}/service-info` and `{wes_base}/runs` with `wes_base` = `{url}/ga4gh/wes/v1` or `{url}`.

---

## DRS

Executed when DRS is DETECTED. Identities: `HLX-DRS-001`–`005` ([TEST_IDENTITY.md](TEST_IDENTITY.md)).

### Standard requirements

What the DRS specification already requires. Helix does not add extra object fields here.

| # | HTTP | Spec obligation |
|---|------|-----------------|
| D1 | `GET {drs_base}/objects/{object_id}` | JSON `DrsObject` on 200. Required properties: `id`, `self_uri`, `size`, `created_time`, `checksums` (OpenAPI `DrsObject`). |
| D2 | `checksums[]` | Each entry has `checksum` and `type`. At least one checksum. Types are those the spec allows (`md5`, `sha-256`, …). |
| D3 | `access_methods` | If present, each method has `type`. Bytes are obtained via `access_url` **or** `access_id` (GetAccessURL). Helix’s **standard** contract accepts either; see optional vs current runner. |
| D4 | Unknown id | `GET {drs_base}/objects/{unknown}` → **404** (GetObject). |
| D5 | No auth header | GetObject succeeds without `Authorization` on this contract’s origin. |

`name` is **optional** in DRS. This contract does not require it.

`GET {drs_base}/service-info` is the DRS service-info operation. Useful for discovery; **not** required for the five DRS checks if the object probe already DETECTED the service.

Bulk endpoints, Passport, compact identifiers, and cloud-signed URLs are out of this contract.

### Fixture requirements

These identifiers are **not** in the DRS spec. They are test input Helix GETs. The default catalog uses `test-object-1`. An external origin that already has a different object must pass **`--drs-object-id`** (optional `--drs-object-sha256`). Helix does not enumerate objects. Catalog: [FIXTURES.md](FIXTURES.md) §1, [TARGETS.md](TARGETS.md) §11.

| Fixture | Request | Response |
|---------|---------|----------|
| Known object | `GET {drs_base}/objects/{drs_object_id}` | **200** `DrsObject` whose `id` equals the configured id. Default catalog: `test-object-1`. |
| Known blob | Whatever URL is in that object’s access method | Default catalog: **4096** bytes, each `0x41` (`A`). Operator-declared fixtures must supply independently known bytes/digest if checksum is to be tested. |
| Checksum | `checksums` on the object, or `--drs-object-sha256` | Default: advertised sha256 vs download. With `--drs-object-sha256`, expected digest is the operator value (not taken from the GetObject JSON under test). |
| Unknown object | `GET {drs_base}/objects/{derived helix.unknown.…}` | **404**. The unknown id is derived from `{drs_object_id}`; it is not a global hard-coded string. |

The access URL path is not specified. Any http(s) URL that returns those bytes is valid. Do not implement a Synaptic Four-only bytes route unless you want to; the in-tree mock happens to use `/bytes/test-object-1`.

Discovery’s DRS object probe uses the same configured id. If you only advertise DRS via `…/ga4gh/drs/v1/service-info`, you can still be DETECTED without that object; existing-object checks then **skip** `fixture_unavailable` rather than being recorded as DRS non-conformance.

### Optional capabilities

Not required for default `helix verify`. Skip is never pass; absence of an optional capability must not be reported as FAIL under this contract.

| Capability | Spec | Helix |
|------------|------|--------|
| HTTP Range on the access URL | Not required by DRS GetObject. RFC 7233 on the **bytes URL** if the server supports it. | Check `HLX-DRS-004` currently **executes** and expects **206** + `Content-Range` for `Range: bytes=0-1023`. That is **stricter than DRS**. Treat Range as optional until the runner skips it when the origin does not advertise Range. |
| `access_id` / GetAccessURL | Allowed by DRS instead of inline `access_url`. | Current runner reads only `access_methods[0].access_url.url`. Two-step access is optional and **not used** today. |
| DRS service-info | Service-info spec. | Discovery only. |
| Additional objects | Allowed. | Ignored. |
| Checksum types other than sha256 | Allowed. | Current runner’s checksum check looks for a `sha256` entry (not `sha-256`). Extra types are fine. |

---

## WES

Executed when WES is DETECTED. Identities: `HLX-WES-001`–`008`. Scatter (`HLX-WES-008`) is **optional** and is **skipped** on the default command.

### Standard requirements

What the WES specification already requires. Helix does not add ServiceInfo fields here.

| # | HTTP | Spec obligation |
|---|------|-----------------|
| W1 | `GET {wes_base}/service-info` | 200 JSON ServiceInfo. WES 1.1 ServiceInfo includes GA4GH Service fields (`id`, `name`, `type.{group,artifact,version}`, `organization.{name,url}`, `version`) and WES fields (`workflow_type_versions`, `supported_wes_versions`, `supported_filesystem_protocols`, `workflow_engine_versions`, `default_workflow_engine_parameters`, `system_state_counts`, `auth_instructions_url`, `tags`). |
| W2 | `POST {wes_base}/runs` | WES OpenAPI: **`multipart/form-data`** RunRequest (`workflow_url`, `workflow_type`, `workflow_type_version`, `workflow_params`, optional `tags`, attachments). Response includes `run_id`. |
| W3 | `GET {wes_base}/runs/{run_id}/status` | JSON with `run_id` and `state`. States from the spec: `UNKNOWN`, `QUEUED`, `INITIALIZING`, `RUNNING`, `PAUSED`, `COMPLETE`, `EXECUTOR_ERROR`, `SYSTEM_ERROR`, `CANCELED`, `CANCELING`. |
| W4 | `GET {wes_base}/runs/{run_id}` | RunLog; `outputs` when the run has completed. |
| W5 | No auth header | These operations succeed without `Authorization` on this contract’s origin. |

`workflow_attachment` is **not** used. Workflows are identified by `workflow_url` only (fixture TRS URIs below — not a live TRS).

List-runs, cancel, and TES are out of this contract.

### Fixture requirements

These `workflow_url` values are **not** in the WES spec. They are documented synthetic workflows Helix POSTs. The WES must accept them as `workflow_url` strings and drive the stated terminal state. They do not require a real TRS server.

Default `helix verify` posts the first five. It does **not** post scatter/gather.

| Helix id | POST fields | Expected terminal | Extra assertion |
|----------|-------------|-------------------|-----------------|
| `wes.run.lifecycle_success` | `workflow_url=trs://test-tool/echo/1.0`, `workflow_type=CWL`, `workflow_type_version=v1.2`, `workflow_params={"message":"hello-ga4gh"}` | `COMPLETE` | `outputs.echo_out` == `hello-ga4gh` |
| `wes.run.failure_state` | `trs://test-tool/fail/1.0`, CWL `v1.2`, `workflow_params={}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.missing_inputs` | `trs://test-tool/cwl-echo/1.0`, CWL `v1.2`, `workflow_params={}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.incompatible_type` | `trs://test-tool/cwl-echo/1.0`, **`WDL` `1.0`**, `workflow_params={"message":"hello-type-mismatch"}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.invalid_workflow` | `trs://nonexistent/invalid/0.0`, CWL `v1.2`, `workflow_params={}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |

Poll: `GET …/runs/{id}/status` until a terminal state. Time budget in the current runner is 300s at 2s intervals (not a spec limit). Terminal states for these fixtures: `COMPLETE`, `EXECUTOR_ERROR`, `SYSTEM_ERROR` (`CANCELED` is spec-valid but not the expected terminal here).

### Optional capabilities

| Capability | Spec | Helix |
|------------|------|--------|
| Scatter/gather workflow | Not a WES-required workflow. | Fixture `trs://test-tool/scatter-gather/1.0` with `workflow_params={"items":[1,2,3,4]}`, type CWL `v1.2`, terminal `COMPLETE`, `outputs.scatter_result` present. **Not posted** by `helix verify <url>`. Skip (`HLX-WES-008`) is not a fail. |
| Extra `workflow_type` values | Allowed if listed in ServiceInfo. | Ignored except the incompatible-type fixture (WDL against a CWL tool). |
| Pre-terminal states | Spec allows `QUEUED` / `INITIALIZING` / `RUNNING`. | Current runner **requires** seeing one of those before `COMPLETE` on the echo run. That is **stricter than WES**. A server that is already `COMPLETE` on the first status poll may fail today’s runner; it does not violate this contract. |
| `supported_wes_versions` member exactly `1.0` or `1.1` | Spec requires the array; version strings follow the WES document. | Current runner additionally requires a member equal to `1.0` or `1.1` (not e.g. `1.1.0`). That extra equality check is **not** this contract. |

---

## Checks vs this contract

| id | code | Contract class |
|----|------|----------------|
| `drs.object.reachable` | `HLX-DRS-001` | Standard GetObject + known fixture id |
| `drs.object.schema` | `HLX-DRS-002` | Standard `DrsObject` schema + fixture id |
| `drs.object.checksum` | `HLX-DRS-003` | Standard checksums + fixture sha256/blob |
| `drs.object.range` | `HLX-DRS-004` | **Optional** (Range). Executed today; see runner table |
| `drs.object.not_found` | `HLX-DRS-005` | Standard 404 + unknown fixture id |
| `wes.service_info.reachable` | `HLX-WES-001` | Standard GET service-info |
| `wes.service_info.schema` | `HLX-WES-002` | Standard ServiceInfo schema |
| `wes.run.lifecycle_success` | `HLX-WES-003` | Standard run API + echo fixture |
| `wes.run.failure_state` | `HLX-WES-004` | Standard run API + fail fixture |
| `wes.run.missing_inputs` | `HLX-WES-005` | Standard run API + missing-input fixture |
| `wes.run.incompatible_type` | `HLX-WES-006` | Standard run API + type-mismatch fixture |
| `wes.run.invalid_workflow` | `HLX-WES-007` | Standard run API + invalid-URL fixture |
| `wes.run.scatter_gather` | `HLX-WES-008` | **Optional**. Skipped on default `helix verify` |

---

## Current runner (pin v0.1.3) — not this contract

Helix still wraps HelixTest for execution. These behaviours are **stricter or different** from the GA4GH specs. They are **not** requirements on an external implementer under this document. A spec-compliant server may fail today’s binary until the runner is aligned. Do not copy them into a new implementation “to satisfy Helix.”

| Extra | Spec | Today’s runner |
|-------|------|----------------|
| DRS `name` required | Optional | Schema helper requires `name` |
| DRS `access_methods` non-empty + inline `access_url` on **first** method | `access_url` **or** `access_id`; array may be omitted in some deployments | Requires non-empty `access_methods` and `access_methods[0].access_url.url` |
| Checksum type `sha256` (no hyphen) | IANA example `sha-256`; other types allowed | Looks for type `sha256` (case-insensitive) |
| HTTP Range 206 on access URL | Not required by GetObject | `HLX-DRS-004` fails if not 206 |
| WES `POST /runs` JSON body | OpenAPI: `multipart/form-data` | JSON `{workflow_url, workflow_type, workflow_type_version, tags, workflow_params}` |
| Echo run must show `QUEUED`\|`INITIALIZING`\|`RUNNING` before `COMPLETE` | Not required | Fail if first observed state is already terminal |
| `supported_wes_versions` contains exactly `1.0` or `1.1` | Array of version strings | Extra equality check |
| `--profile ferrum` | Not a GA4GH concept | Opt-in: expects DRS **and** WES present; posts scatter fixture. **Not** the external command. |

Wire details of today’s DRS/WES checks: [DRS_PROFILE.md](DRS_PROFILE.md), [WES.md](WES.md). In-process mocks: [FIXTURES.md](FIXTURES.md).

---

## What a result means

- **PASS / FAIL / ERROR / SKIP** are check outcomes. Skip is never pass. DETECTED is not a pass.
- Exit 0: at least one pass, no fail/error (DRS-only or WES-only is allowed).
- Exit 1: fail, error, skip-only, or unreachable.
- Not certification. Not HELIOS evidence.

Report a failure: [GitHub Issues](https://github.com/SynapticFour/Helix/issues) on this repo. Include Helix commit, command, `<url>` layout (prefixed vs split), and stdout/stderr. Do not send production secrets.

---

## Out of this contract

TES, TRS, htsget, Beacon execution; `helix security`; `helix bench`; HELIOS; Ferrum as a dependency or profile; GA4GH certification; any requirement not in the DRS/WES OpenAPI or the fixture tables above.

Cross-implementation recording of the **same** generic `helix verify` command: [INTEROP.md](INTEROP.md). That matrix does not add implementation-specific verify logic. External independent evidence is pending.
