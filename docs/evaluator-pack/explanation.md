# What Helix is (one page)

Helix is a **CLI**. You point it at a GA4GH HTTP origin you already run. It discovers which APIs answer, then runs HelixTest **DRS** and **WES** checks when those services are TESTABLE.

HelixTest already exists (public repo, CI, pin **v0.1.3**). Helix productizes that engine as `helix`. It is not a new test platform.

```text
your DRS/WES  →  helix verify <url>  →  text report + JSON + exit code
```

`make prove` / `make verify-fixture` use in-process mocks so you do not need a live stack.

## What it is not

- Not a server. It does not start Ferrum or any stack.
- Not HELIOS. No signed trails, RO-Crate, PDF, or reproducibility envelope (`helios-audit` is a different repo).
- Not GA4GH certification. Green prove / green verify is a technical signal.
- Not a Ferrum production or clinical-pilot claim. Ferrum is a **reference** live target only (BUSL-1.1, on-prem). There is no DIZ / genomDE pilot. `--profile ferrum` is not required.
- Not a pentest product. `helix security` is out of this pack.
- Not a Synaptic Four-hosted service. No account, no telemetry.

## How to run (this pack)

[install.md](install.md) then [commands.md](commands.md). Default command: `helix verify <url>` or `make verify-fixture`.

## What a result means

[interpret.md](interpret.md). DETECTED is not a pass. Skip is never pass. Exit 0 is not certification.

## Why Ferrum appears in other docs

Ferrum is this organisation’s on-prem GA4GH implementation, used as an optional live target. Helix must work against Ferrum **and** against any origin that implements the [target](target.md) + [fixture](fixtures.md) contract. Helix does not import Ferrum.

## Why HELIOS is separate

Helix answers **whether** a running system behaves. HELIOS answers **what** ran and **how** to reproduce it. Do not expect RO-Crate or signed evidence from `helix verify`.

## Failures

Fill [FAILURE_REPORT.md](FAILURE_REPORT.md). You do not need to speak to Synaptic Four to run Helix.
