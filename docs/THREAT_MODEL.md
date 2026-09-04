# Helix threat model (the tool)

Helix is HelixTest becoming a standalone VERIFY CLI. HelixTest already runs (public GA4GH-stack repo, CI, SF-TR-2026-001 / 002). This document is about **Helix as an HTTP client** pointed at a target the operator chose. It is not a pentest methodology, not a WAF, not an SSRF filter product, and not HELIOS (`helios-audit` keeps reproducibility / signed evidence / RO-Crate / PDF).

Passing `helix verify` / `helix security` / `helix bench` is a technical signal. It is **not** GA4GH certification, not a security audit, and not proof that the target is safe to expose.

Source of mitigations: `src/redact.rs`, `src/http_safety.rs`, `src/discover.rs` (`normalize_endpoint`, `http_client`).

---

## What this document is not

- Helix is **not a security product**. This file exists so operators know what the CLI will and will not fetch or print.
- Helix does **not** own audit trails, signed evidence, or RO-Crate (HELIOS).
- Helix does **not** claim to block every SSRF, DNS rebind, or malicious target.
- Dummy HMAC / Crypt4GH fixtures are **not** production credentials (`test-fixtures/`, NICHT FÜR PRODUKTION).

---

## Actor and trust

| Actor | Trust |
|-------|--------|
| Operator | Chooses the origin (`helix verify <url>`), local files (`helix compare`, `--hmac-secret-file`, `--crypt4gh-file`), and CI `endpoint`. Helix runs with that operator’s privileges. |
| Target HTTP service | **Untrusted.** It may be Ferrum, a mock, a third-party GA4GH stack, or a malicious host. Response bodies, headers, redirects, and advertised DRS `access_url`s are attacker-controlled once the operator points Helix there. |
| GitHub Actions `endpoint` input | Trusted only if the **workflow author** hard-codes or otherwise controls it. A PR-controlled URL is “SSRF from the runner.” That is a workflow bug, not something Helix can fully prevent. |
| HelixTest (separate git root, pin v0.1.3) | Dependency. Helix does not vendor or silently patch it (D1). Residual client behaviour lives there. |

Helix is **not** an open proxy. An untrusted third party cannot ask Helix to fetch a URL unless the operator (or a workflow they wrote) passes that URL in.

**Do not** block localhost / RFC1918 as a generic allowlist. Ferrum on `127.0.0.1` and in-process wiremock are first-class prove targets. Blocking them would break the product.

---

## Assets to protect

1. **Secrets the operator gave Helix** — dummy HMAC (`HELIX_HMAC_SECRET` / `--hmac-secret-file`), minted JWTs, `Authorization` request headers.
2. **Secrets in the environment** — GitHub `github-token` (helix-action must not pass it into Helix). Shell history is out of scope.
3. **Operator machine / CI runner** — memory, disk, and outbound HTTP as the Helix process user.
4. **Logs and artifacts** — stdout JSON, human text, helix-action logs, `VerificationRun` files.

**Hard rule:** Helix must never accidentally print secrets or `Authorization` header values (stdout, stderr, JSON, text report, skip/fail messages, diagnostics `observed` / `Check output:`).

---

## Helix-owned HTTP vs HelixTest HTTP

| Client | Used by | Timeouts | Redirects | Body cap | Compression |
|--------|---------|----------|-----------|----------|-------------|
| `http_safety::http_client` | Discovery, `helix security` HTTP, Crypt4GH HTTP envelope probe, `helix bench` | 5s request / 3s connect | **None** (`Policy::none()`) | **2 MiB** | gzip/brotli **off** (`reqwest` `default-features = false`) |
| HelixTest `HttpClient` | `helix verify` DRS/WES checks via adapter | 30s / 5s connect (HelixTest) | **Follows** (reqwest default) | Unbounded | Workspace reqwest **default features** (gzip likely on) |

Helix-owned probes do not follow `Location`. A 302 is **not** 2xx/401/403, so discovery will not treat a redirect as DETECTED. That is a deliberate SSRF mitigator for Helix-owned GETs. It can miss a target that only answers after a redirect; operators should point at the real origin.

---

## Findings and treatment

### SSRF (Helix-owned)

Operator-supplied `http`/`https` URLs are fetched. That is the product. Mitigations that fit this scope:

- Reject non-`http`/`https` (no `file:`, `ftp:`, `gopher:`).
- Reject URL **userinfo** (`user:pass@host`) so credentials never become `target.url` or log lines.
- Do **not** follow redirects on the Helix-owned client (stops bounce to link-local / metadata from a 3xx).

Not in scope: IP allowlists, blocking `169.254.169.254`, DNS pinning.

### SSRF (HelixTest / DRS `access_url`) — accepted residual

HelixTest DRS checksum / bytes checks may **GET `access_url` advertised by the target**. A malicious `DrsObject` can point that client at cloud metadata or an internal HTTP service. Redirects may be followed. Helix documents this; it does not rewrite HelixTest in this repo. Operators must not point `helix verify` at an untrusted DRS if that fetch is unacceptable.

### Redirects

Helix-owned: no follow. 301/302/307/308 are not DETECTED.

HelixTest: may follow. Residual.

### URL parsing

`normalize_endpoint` requires a host, `http` or `https`, strips a trailing slash, and **rejects userinfo**. The error string must **not** echo the password or the raw URL.

### DNS rebinding — accepted residual

Helix does not pin DNS to the connect-time address. A host that answers public A then rebinds to a private IP on the next request can reach different machines. Typical for an operator-run CLI; fixing it would be a resolver/pinning product. Documented, not implemented.

### Credential leakage / bearer tokens / logs / JSON / errors

| Path | Mitigation |
|------|------------|
| Minted JWT | Sent as `Authorization: Bearer` only. Never written to reports. |
| HMAC secret | Loaded for signing only. Fail/skip text uses the **fixture path**, not the secret. |
| Target reflects `Authorization` or a JWT in a body / error | `redact_text` on verification messages, diagnostics, security errors, JSON/text printers, and `main` stderr |
| URL userinfo | Rejected at parse; also stripped if it appears in a string we print |
| `RUST_LOG=debug` | Helix tests set `error`. HelixTest may log a **body prefix** at debug. Do not run verbose logging against an untrusted target if you care about body leakage. Helix does not enable debug itself. |

Redaction is pattern-based (`Authorization` values, `Bearer` / `Basic` tokens, JWT-shaped `eyJ….…`, URL userinfo, `HELIX_HMAC_SECRET` when set). It is a leak-prevention layer, not a guarantee against every encoding.

### Test fixtures

`test-fixtures/hmac/shared-secret.txt` is a **dummy** labeled NOT FOR PRODUCTION. Tests assert CLI stdout/stderr do not contain that value or `Authorization: Bearer <jwt>`. Fixtures are not real hospital secrets. Do not replace them with production keys.

Local **adversarial** HTTP mocks (`tests/support/mock_adversarial.rs`, [FIXTURES.md](FIXTURES.md) §16) replay malformed / huge / slow / redirecting responses on localhost only. They are not real-world attacks. They lock fail-closed behavior (no overall PASS, no decoy-token leak, timeouts).

### Local file arguments

| Input | Cap | On error |
|-------|-----|----------|
| `helix compare` JSON | 8 MiB | Path + limit; contents not printed |
| `--hmac-secret-file` | 64 KiB | Path + limit; contents not printed |
| `--crypt4gh-file` | 1 MiB | Path + limit; bytes not dumped (Crypt4GH errors already say “not dumping bytes”) |

These are operator paths. Helix does not fetch `file:` URLs.

### GitHub Action (sibling `helix-action`)

Residual / caller duty, documented here so Helix operators see it:

- Do **not** pass a PR-controlled `endpoint` (SSRF from the runner).
- Pin Helix / HelixTest SHAs; `persist-credentials: false` on those checkouts.
- `github-token` is for `gh` comment/baseline, not for Helix argv.
- Action logs currently echo `Running: $HELIX verify ${ENDPOINT}`. After Helix rejects userinfo, that line should not contain embedded passwords; still do not put secrets in the URL.
- `RUST_LOG=error`; stderr is captured to a file next to the JSON artifact. Helix JSON/text must still be redacted so artifacts stay clean.

Helix does not grow a pentest mode for Actions.

### Untrusted target responses / size / timeouts / decompression

| Control | Helix-owned client |
|---------|---------------------|
| Timeouts | 5s request, 3s connect |
| Response size | Stop at 2 MiB; oversized probe is **not** DETECTED (and is a bench error) |
| Decompression | gzip/brotli features off — no gzip-bomb on this client |
| JSON snapshots | service-info fields are copied as strings; printers redact auth/JWT patterns |

HelixTest responses remain unbounded (residual). Compare JSON is capped so a huge artifact cannot OOM the compare process as easily.

---

## Residual risks (honest)

- HelixTest `HttpClient`: redirects, gzip, larger timeout, DRS `access_url` fetch, debug body prefix.
- DNS rebinding.
- Operator or workflow points Helix at an internal URL on purpose (that is the job).
- Redaction may miss exotic encodings (`Authorization` split across fields, Unicode lookalikes).
- A 2 MiB cap can still allocate 2 MiB per probe.

---

## What we will not do in this scope

- Turn Helix into a vulnerability scanner, fuzzer, or “security platform.”
- Duplicate HELIOS evidence.
- Patch HelixTest in this repository to change its HTTP client.
- Block loopback so local Ferrum / prove mocks stop working.
- Claim Ferrum production/clinical deployments or GA4GH certification.
