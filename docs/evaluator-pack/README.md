# External evaluator pack

Self-contained instructions to **run Helix** from public git. No Synaptic Four conversation, account, cloud product, or telemetry.

Helix is HelixTest becoming a standalone VERIFY CLI (pin **v0.1.3**). It is not a new test platform. Results are not GA4GH certification. HELIOS (`helios-audit`) is a different product.

| Piece | File |
|-------|------|
| Installation | [install.md](install.md) |
| One-page explanation | [explanation.md](explanation.md) |
| Target requirements | [target.md](target.md) |
| Test fixture requirements | [fixtures.md](fixtures.md) |
| Example commands | [commands.md](commands.md) |
| Example JSON | [example-verify.json](example-verify.json) |
| Interpretation | [interpret.md](interpret.md) |
| Failure reporting template | [FAILURE_REPORT.md](FAILURE_REPORT.md) |

Full contract (spec vs fixtures vs optional vs current-runner extras): [../EXTERNAL_TARGET_CONTRACT.md](../EXTERNAL_TARGET_CONTRACT.md).

## What this pack does not do

- No sales call, demo booking, or mailing list.
- No Synaptic Four account. Cloning the public GitHub repositories does not require a GitHub account.
- No Helix Cloud or other hosted service. After the first crate download from crates.io (public, no account), `make prove` / `make verify-fixture` run on your machine.
- Helix does not send telemetry, usage pings, or crash reports.

GitHub Issues (optional, for filing a bug) uses a GitHub account, not a Synaptic Four account. You can fill [FAILURE_REPORT.md](FAILURE_REPORT.md) and keep it locally.
