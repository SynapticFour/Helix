# Target requirements

What an external origin must expose for `helix verify <url>`. No Ferrum, Synaptic Four config, HelixTest crates, special headers, or private APIs.

Full text: [../EXTERNAL_TARGET_CONTRACT.md](../EXTERNAL_TARGET_CONTRACT.md). This page is the pack summary.

## Layout

`<url>` is `http://` or `https://` (no userinfo). Helix does not start the server. Unauthenticated GET/POST only.

Either GA4GH-prefixed or split-at-origin is enough:

| Service | Prefixed | Split |
|---------|----------|--------|
| DRS | `{url}/ga4gh/drs/v1` | `{url}` (`GET /objects/{id}`) |
| WES | `{url}/ga4gh/wes/v1` | `{url}` (`GET /service-info`) |

`{url}/service-info` is the **WES** split probe, not DRS.

TES / TRS / htsget may be probed. They are not executed.

## DRS (standard)

GA4GH GetObject only. Helix does not add object fields here.

- `GET {drs_base}/objects/{id}` → 200 JSON `DrsObject`: `id`, `self_uri`, `size`, `created_time`, `checksums`.
- Unknown id → **404**.
- Bytes via `access_url` or `access_id` (spec). `name` is optional in the spec.

## WES (standard)

GA4GH WES HTTP:

- `GET {wes_base}/service-info`
- `POST {wes_base}/runs` (OpenAPI: **multipart/form-data**)
- `GET {wes_base}/runs/{id}/status`
- `GET {wes_base}/runs/{id}`

A DRS-only or WES-only origin is allowed. Neither present → skip-only, exit 1.

## Optional (not required for default `helix verify`)

HTTP Range on the bytes URL; DRS two-step `access_id`; WES scatter/gather. Default command **does not** post scatter.

## Current runner extras (not this contract)

Pin **v0.1.3** still requires some behaviours that are stricter than the specs (DRS `name`, inline `access_url` on the first method, Range 206, JSON POST `/runs`, echo must show a pre-terminal state). Listed in [../EXTERNAL_TARGET_CONTRACT.md](../EXTERNAL_TARGET_CONTRACT.md). A spec-correct server may fail today’s binary until the runner is aligned. Do not copy those extras into a new implementation “for Helix.”
