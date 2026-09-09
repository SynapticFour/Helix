# WES verification (HelixTest wrap)

HelixTest already runs WES checks (`framework::wes::run_wes_checks`). Helix productizes that through the generic target boundary: discover → TESTABLE → adapter → Helix `VerificationRun`.

What a generic WES must expose (spec vs fixtures vs optional; no Ferrum): [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md).

Helix does **not** invent WES behavior. The HTTP contract, fixture URLs, poll timing, and skip rule below are copied from HelixTest pin **v0.1.3**. TES / TRS / htsget are not wired. Ferrum is a reference target, not a dependency. HELIOS is out of scope.

Source: `src/verify.rs`, `src/adapter` (`run_wes`). Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md) `wes.*` / `HLX-WES-001`–`008`. In-process fixture: `tests/support/mock_ga4gh_wes.rs`. Fail/error diagnostics: [DIAGNOSTICS.md](DIAGNOSTICS.md).

---

## Generic boundary

`run_wes_checks` takes a `Mode` argument and **does not use it**. All eight checks are HTTP against `TestConfig.services.wes_url`. No Ferrum crate, no Ferrum mode switch.

Helix therefore executes WES the same way as DRS: `Mode::Generic`, discovery-filled `wes_url`, `HttpClient` from HelixTest common.

---

## Fixture assumptions (HelixTest, not invented)

These are the only workflows HelixTest posts. A target that does not implement this table will fail the matching checks. Helix’s mock implements the same table.

| Helix id | HelixTest POST | Expected terminal | Extra assertion |
|----------|----------------|-------------------|-----------------|
| `wes.run.lifecycle_success` | `trs://test-tool/echo/1.0`, type `CWL`, version `v1.2`, `workflow_params.message = "hello-ga4gh"` | `COMPLETE` | History includes `QUEUED` \| `INITIALIZING` \| `RUNNING`. First state is **not** terminal. `outputs.echo_out == "hello-ga4gh"` |
| `wes.run.failure_state` | `trs://test-tool/fail/1.0`, `CWL` `v1.2`, empty params | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.missing_inputs` | `trs://test-tool/cwl-echo/1.0`, `CWL` `v1.2`, empty params | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.incompatible_type` | `trs://test-tool/cwl-echo/1.0`, type **`WDL`** version `1.0`, `{message: "hello-type-mismatch"}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.invalid_workflow` | `trs://nonexistent/invalid/0.0`, `CWL` `v1.2` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `wes.run.scatter_gather` | `trs://test-tool/scatter-gather/1.0` with `{items: [1,2,3,4]}` | `COMPLETE` + `outputs.scatter_result` | Skipped on profile `generic`. Executed on profile `ferrum`. |

HTTP (HelixTest `common::workflow`):

- `GET {wes}/service-info`
- `POST {wes}/runs` JSON `{workflow_url, workflow_type, workflow_type_version, tags, workflow_params}` → `{run_id}`
- `GET {wes}/runs/{id}/status` → `{run_id, state}`; poll interval **2s**, timeout **300s** (scatter would be 5s / 600s)
- `GET {wes}/runs/{id}` → `{outputs}`
- Terminal states: `COMPLETE` \| `EXECUTOR_ERROR` \| `SYSTEM_ERROR` \| `CANCELED`
- Sequence: first observed state must not be terminal; phases are monotonic `QUEUED` → `INITIALIZING` → `RUNNING` → terminal (unknown states treated as Running-like)

Reachable (`HLX-WES-001`): HTTP **2xx or 401** on `GET …/service-info` (`level0_reachable_ok`). Schema (`HLX-WES-002`) still needs a 2xx JSON body that validates.

Schema (`HLX-WES-002`): official WES 1.1.0 ServiceInfo **plus** `supported_wes_versions` must contain **`1.0` or `1.1`**. Required ServiceInfo fields include Service (`id`, `name`, `type.{group,artifact,version}`, `organization.{name,url}`, `version`) and WES fields (`workflow_type_versions`, `supported_wes_versions`, `supported_filesystem_protocols`, `workflow_engine_versions`, `default_workflow_engine_parameters`, `system_state_counts`, `auth_instructions_url`, `tags`).

### Scatter skip (profile-controlled)

HelixTest skips `WES scatter/gather workflow` unless `Features.supports_scatter_gather` is true. Helix maps that bit from the **profile**, not from WES `name` and not from Ferrum mode ([PROFILES.md](PROFILES.md)):

| Profile | `supports_scatter_gather` | `HLX-WES-008` |
|---------|---------------------------|---------------|
| `generic` (default) | false | skip (`supports_scatter_gather=false in features`) |
| `ferrum` | true | execute the HelixTest scatter POST |

Helix does not invent scatter/gather fixtures. The mock implements HelixTest’s `trs://test-tool/scatter-gather/1.0` URL when a profile turns the capability on. Skip is never pass. A WES `name` of “Ferrum Gateway” does **not** enable scatter.

---

## Outcomes Helix distinguishes

| Situation | Discovery | Checks | Exit |
|-----------|-----------|--------|------|
| Service unavailable (HTTP up, no WES) | WES `present: false` | eight WES rows `skip` | 1 if nothing else passed |
| Target unreachable (TCP fail) | all not present | eight WES rows `error` (and DRS `error`) | 1 |
| Discovered but not testable | TES/TRS/htsget `present: true`, `testable: false` | **no** TES/TRS/htsget check rows | not a pass |
| Test skipped | WES TESTABLE, profile `generic` | `wes.run.scatter_gather` `skip` | 0 if other checks passed |
| Test failed | WES TESTABLE | `fail` + `failure.code` | 1 |
| Test passed | WES TESTABLE | `pass` | 0 if no fail/error and ≥1 pass |

DETECTED / TESTABLE on discovery is **not** a verification pass.

---

## In-process mock

`tests/support/mock_ga4gh_wes.rs` implements the HTTP contract above. First status poll returns `RUNNING`; later polls return the terminal state from the fixture table. Default name is **not** “Ferrum Gateway” (a named fixture exists only to prove Helix does not auto-switch). Combined DRS+WES: `start_mock_ga4gh_drs_and_wes`. Catalog: [FIXTURES.md](FIXTURES.md).

The Helix DRS fixture (`tests/support/mock_ga4gh_drs.rs`) does **not** mount a WES-shaped `/service-info`. HelixTest B1 still does (Ferrum-name trap); Helix adapter uses `Mode::Generic`, so that trap is HelixTest’s concern.

---

## Out of this suite

- TES / TRS / htsget execution
- HelixTest `--mode ferrum` / importing Ferrum
- HELIOS evidence
- Ferrum as a crate
- GA4GH certification claims
- AI / LLM root-cause diagnosis (WES fail/error rows get deterministic **possible causes** only)
