# Example commands

No account. No cloud target required. Helix does not send telemetry.

## 1. Prove the repo (in-process fixtures)

```bash
cd Helix
make prove
```

Exit 0 prints `Helix prove OK (in-process fixtures; not Ferrum, not certification).` That is not a `HELIX VERIFICATION` report.

## 2. See a verification report (no live stack)

```bash
cd Helix
make verify-fixture
```

Starts the DRS fixture ([fixtures.md](fixtures.md)), runs `helix verify` against it, prints `HELIX VERIFICATION` on stdout. Expected: five DRS PASS, eight WES SKIP (WES not mounted), exit 0. DETECTED is not a pass. Skip is never pass.

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

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Overall pass (≥1 pass, no fail/error). Not certification. |
| 1 | Fail, error, skip-only, unreachable, or runtime error. |
| 2 | Usage (bad argv). |

## Not in this pack

`helix security`, `helix bench`, `--profile ferrum`, Ferrum `make up`. Those are optional and not required to evaluate Helix.
