# Example commands

No account. No cloud target required. Helix does not send telemetry.

## 1. Prove the repo (in-process fixtures)

```bash
cd Helix
make fetch    # once: Cargo.lock checksums; not a GA4GH download
make prove
```

Exit 0 prints `Helix prove OK (in-process fixtures; not Ferrum, not certification).` That is not a `HELIX VERIFICATION` report.

## 2. See a verification report (no live stack)

```bash
cd Helix
make verify-drs
```

Canonical DRS 1.4.0 path against the in-process fixture: human report, `verify.json`, `helix inspect`. Not independent evidence. Unversioned `make verify-fixture` still prints `HELIX VERIFICATION` without selecting the DRS 1.4.0 pack.

## 3. JSON (same fixture, after `make install` or via cargo)

`make verify-fixture` prints text. For JSON against a URL you control:

```bash
make install    # optional; still needs sibling HelixTest at build time
NO_COLOR=1 helix verify http://127.0.0.1:<port> --format json
```

Or without installing:

```bash
NO_COLOR=1 cargo run --quiet --locked --bin helix -- verify http://127.0.0.1:<port> --format json
```

Replace `<port>` with an origin **you** started that implements [target.md](target.md) + [fixtures.md](fixtures.md). Do not assume port 8080 is listening.

Stdout is `VerificationRun` ([example-verify.json](example-verify.json), schema `schemas/helix-verification-v1.json`). Stderr is logs (default `RUST_LOG=error`).

## 3b. Supported DRS 1.4.0 (technical verification)

Default `helix verify URL` is unversioned. To select the supported pack:

```bash
NO_COLOR=1 helix verify http://127.0.0.1:<port> \
  --standard drs --version 1.4.0 --output verify.json
helix inspect verify.json
```

Operator path: [../OPERATOR_VERIFY.md](../OPERATOR_VERIFY.md). PASS is not VERIFIED. Exit 0 is not VERIFIED. `inspect` does not rewrite the file.

## 3c. Two independent DRS implementations (optional live)

In-process fixtures are not independent evidence. If you start GA4GH Starter Kit DRS and Bento DRS yourself:

[INDEPENDENT_DRS.md](../INDEPENDENT_DRS.md)

```bash
helix verify URL --standard drs --version 1.4.0 --output FILE
helix inspect FILE
helix differential starter-kit.json bento.json
```

`make verify-independent` is OPTIONAL LIVE VERIFICATION. It is not `make prove`. Helix does not rank the two implementations.

## 3d. Standards provenance (no live target)

Does not run `helix verify`. Does not download specs.

```bash
helix standards list --supported-only
helix standards validate
helix standards trace drs.object.schema.openapi
```

DRS 1.4.0 is SUPPORTED for technical verification within declared coverage. That is not GA4GH certification and not complete DRS coverage. WES is not a published Supported standard.

## 4. Interop matrix (external validation pending)

```bash
NO_COLOR=1 helix matrix --format json
```

Same generic `helix verify` suite, compared later when you have two recorded JSON files. With no `--run`, slots `ferrum` and `independent` are **pending**. In-process fixtures are **not independent evidence**. Do not quote a pending matrix as multi-implementation validation. Details: [INTEROP.md](../INTEROP.md).

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Overall pass (≥1 pass, no fail/error). Not certification. |
| 1 | Fail, error, skip-only, unreachable, or runtime error. |
| 2 | Usage (bad argv). |

## Not in this pack

`helix security`, `helix bench`, `--profile ferrum`, Ferrum `make up`. Those are optional and not required to evaluate Helix.
