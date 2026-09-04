# Test fixture requirements

These identifiers are **not** in the GA4GH specs. Helix GETs/POSTs them. Mount them on a test origin (or use `make verify-fixture`, which starts the in-process DRS mock).

Catalog: [../FIXTURES.md](../FIXTURES.md). Contract: [../EXTERNAL_TARGET_CONTRACT.md](../EXTERNAL_TARGET_CONTRACT.md).

## DRS

| Request | Response |
|---------|----------|
| `GET {drs_base}/objects/test-object-1` | 200 `DrsObject`, `id` = `test-object-1` |
| Bytes URL in that object’s access method | 4096 bytes, each `0x41` (`A`) |
| `checksums` | Include type **`sha256`** of that blob (hex) |
| `GET {drs_base}/objects/nonexistent-object-id-for-conformance` | **404** |

The bytes path is not specified. Any http(s) URL that returns those bytes is valid. The in-tree mock uses `/bytes/test-object-1`; that path is not required.

`make verify-fixture` mounts this DRS fixture and does **not** mount WES.

## WES

No live TRS. Helix posts these `workflow_url` strings. Default `helix verify` posts the first five, not scatter.

| `workflow_url` | Other fields | Terminal | Extra |
|----------------|--------------|----------|--------|
| `trs://test-tool/echo/1.0` | CWL `v1.2`, `{"message":"hello-ga4gh"}` | `COMPLETE` | `outputs.echo_out` == `hello-ga4gh` |
| `trs://test-tool/fail/1.0` | CWL `v1.2`, `{}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `trs://test-tool/cwl-echo/1.0` | CWL `v1.2`, `{}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `trs://test-tool/cwl-echo/1.0` | **WDL `1.0`**, `{"message":"hello-type-mismatch"}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `trs://nonexistent/invalid/0.0` | CWL `v1.2`, `{}` | `EXECUTOR_ERROR` or `SYSTEM_ERROR` | — |
| `trs://test-tool/scatter-gather/1.0` (optional) | CWL `v1.2`, `{"items":[1,2,3,4]}` | `COMPLETE` | `outputs.scatter_result` present. **Not posted** by default. |

## Credentials

None. Dummy HMAC files under `test-fixtures/` are for `helix security` only (out of this pack). Do not put production secrets in a test origin.
