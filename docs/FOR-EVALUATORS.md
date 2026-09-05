# For evaluators

Five minutes. Helix is a CLI that wraps [HelixTest](https://github.com/SynapticFour/HelixTest) (existing engine, pin **v0.1.3**) as a standalone `helix` binary. It is not a new test platform. Results are not GA4GH certification. Standalone pack (install, contract summary, commands, example JSON, report template): [evaluator-pack/README.md](evaluator-pack/README.md). Confusion log of a first clone: [EVALUATOR_JOURNEY.md](EVALUATOR_JOURNEY.md). Install detail: [INSTALL.md](INSTALL.md).

## What Helix is

A CLI (`helix`) you point at a GA4GH HTTP origin you already run. It discovers which APIs answer, then runs HelixTest **DRS** and **WES** checks when those services are TESTABLE. `make prove` / `make verify-fixture` use in-process mocks so you do not need Ferrum. An external origin: [EXTERNAL_TARGET_CONTRACT.md](EXTERNAL_TARGET_CONTRACT.md) (`helix verify <url>`; documented GA4GH HTTP + fixtures only).

Helix runs the same documented DRS and WES checks against any HTTP origin that implements those GA4GH paths. Ferrum is a reference target, not a dependency. Helix supports technical verification checks for GA4GH DRS 1.4.0 within the declared coverage boundary. A PASS is not a GA4GH-release VERIFIED claim ([TRUST.md](TRUST.md), [CLAIMS.md](CLAIMS.md)).

## What it is not

- Not a server. It does not start Ferrum or any stack.
- Not HELIOS. No signed trails, RO-Crate, PDF, or reproducibility envelope (`helios-audit` is a different repo). Independent **technical** reproduction of Helix fixture results: [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md) (not bit-for-bit JSON files).
- Not GA4GH certification. Green prove / green verify is a technical signal. Inspect pins and JSON rather than trusting the authors ([TRUST.md](TRUST.md)).
- Not a Ferrum production or clinical-pilot claim. Ferrum is a **reference target** (BUSL-1.1, on-prem). There is no clinical deployment with German hospital data-integration centres (DIZ) or the genomDE programme. Demos and CI ≠ production.
- Not a pentest product. `helix security` is selected dummy-HMAC behaviour checks.

## How to run it

Needs [Rust](https://rustup.rs/) **1.91.1** (see `rust-toolchain.toml`; put rustup’s `cargo` on `PATH` before Homebrew) and a **sibling** HelixTest checkout at the SHA in [VERSIONS.lock](../VERSIONS.lock):

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch
make prove
make verify-fixture
```

| Command | What happens |
|---------|----------------|
| `make fetch` | `cargo fetch --locked`: crates.io at lockfile checksums. Explicit network. Not GA4GH. |
| `make prove` | Docs checks + `cargo test --locked --offline --all-targets`. In-process fixtures. No Ferrum, Docker, or credentials. |
| `make independent-verify` | Vendor SHA-256 + two-run fixture equality ([INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md)). Offline. |
| `make verify-fixture` | Starts the deterministic mock DRS ([FIXTURES.md](FIXTURES.md) §1) and runs **`helix verify`** against it. Prints `HELIX VERIFICATION`. |
| `make install` | `cargo install --path . --locked` (still needs the sibling at build time). |

If `require-helixtest.sh` warns that HelixTest HEAD ≠ `VERSIONS.lock`, checkout the pin. Cargo may still compile; CI will not match.

Optional: a stack you started (e.g. Ferrum `make up`) then `make test-live HELIX_LIVE_URL=http://127.0.0.1:8080`. That is not prove.

## What the result means

- **PASS / FAIL / ERROR / SKIP** are check outcomes. Skip is never pass.
- **DETECTED** means an HTTP probe got 2xx/401/403. It is **not** a pass.
- **TESTABLE** means Helix will execute checks for that service. DRS and WES are TESTABLE today. TES/TRS/htsget may be DETECTED and still not executed.
- Fixture run: DRS checks should **pass**; WES is **not mounted** → skip. Overall can still be pass (passes exist, no fail/error).
- Exit 0 = overall pass. Exit 1 = fail, error, skip-only, or unreachable. Not certification.
- Human report: [REPORT.md](REPORT.md). JSON: `--format json` ([SCHEMA.md](SCHEMA.md)).
- The report is **stdout**, starting at `HELIX VERIFICATION`. HelixTest HTTP traces are **stderr** and are off unless `RUST_LOG=debug`.

## Why Ferrum is mentioned

Ferrum is the on-prem GA4GH **implementation** this org uses as a reference live target. Helix must work against Ferrum **and** against non-Ferrum HTTP (the in-process fixture proves that). Helix does not import Ferrum. Profile `--profile ferrum` is opt-in and is never chosen from service-info `name`.

## Why HELIOS is separate

Helix answers **whether** a running system behaves. HELIOS (`helios-audit`) answers **what** ran and **how** to reproduce it (signed evidence, RO-Crate, PDF). Do not file HELIOS features against this repo. Gate: [HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md).

## How to report a failure

1. [Open a GitHub Issue](https://github.com/SynapticFour/Helix/issues) on **this** repo. Template: [evaluator-pack/FAILURE_REPORT.md](evaluator-pack/FAILURE_REPORT.md). Include Helix commit, HelixTest SHA (`VERSIONS.lock` or `git -C ../HelixTest rev-parse HEAD`), command, stdout/stderr, and whether you used `verify-fixture` or a live URL.
2. Security vulnerabilities: [SECURITY.md](../SECURITY.md) (email; not a public issue).

Do not claim the failure is a GA4GH certification result. Do not send production secrets.

## AVAILABLE vs SUPPORTED

`helix standards list` shows pinned GA4GH releases. `--supported-only` lists **ga4gh.drs.1.4.0**. YAML `support_status` is not sufficient; the executable gate in `src/standards/support.rs` must pass. SUPPORTED is not VERIFIED ([STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md)).

## Who reviews a normative mapping

There is no GA4GH-appointed board. A check may become `normative` only through a reviewed change that meets [TRACEABILITY.md](TRACEABILITY.md) §7. Until then the catalog must stay non-normative. The current steward is the single maintainer named in [IDENTITY.md](IDENTITY.md). A reviewer rejects the mapping by pointing at missing locators, hashes, or tests — not by trusting the steward.
