# Getting started

Helix checks an HTTP origin you already run. It does not start that origin. The command is `helix`. The check bodies come from [HelixTest](https://github.com/SynapticFour/HelixTest), which has to be a sibling git checkout.

## Names

| Name | What it is here |
|------|-----------------|
| GA4GH | Global Alliance for Genomics and Health. It publishes HTTP API specifications (DRS, WES, and others). Helix is not GA4GH and is not certified by GA4GH. |
| HelixTest | The existing test engine. Helix wraps it. Required as a sibling git checkout at `HELIXTEST_SHA` in [VERSIONS.lock](../VERSIONS.lock). A tarball without git metadata is not enough. |
| Ferrum | Optional live reference implementation. Not a Helix dependency. Not required for `make prove`. |
| HELIOS | A different program (`helios-audit`): signed evidence, RO-Crate, and PDF. Not this binary. |

## Prerequisites

Rust 1.91.1 via rustup. See [INSTALL.md](INSTALL.md). A sibling [HelixTest](https://github.com/SynapticFour/HelixTest) git checkout at the SHA in [VERSIONS.lock](../VERSIONS.lock) (`HELIXTEST_SHA`, path `../HelixTest`). Tag lineage in `HELIXTEST_REF` is not a substitute for that SHA.

## In this repository, with the sibling pin

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch
make prove
make verify-drs
```

`make fetch` is crates.io at [Cargo.lock](../Cargo.lock) checksums. It is an explicit network fetch, not a GA4GH download. `make prove` is the document checks plus `cargo test --locked --offline --all-targets` against in-process fixtures ([FIXTURES.md](FIXTURES.md)). It does not need Ferrum, Docker, or credentials. You should see the line `Helix prove OK (in-process fixtures; not Ferrum, not certification).`

`make verify-drs` is the DRS 1.4.0 path against the in-process fixture: a human report, `verify.json`, and `helix inspect`. It is not independent-implementation evidence. Unversioned `make verify-fixture` still exists.

## Install the binary

```bash
make install
helix verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
helix inspect verify.json
```

`make install` is `cargo install --path . --locked`. Compiling still needs the sibling HelixTest checkout.

## Against an origin you started

A gateway on `http://127.0.0.1:8080`, for example Ferrum with `make up` in [that repository](https://github.com/SynapticFour/Ferrum), is a reference live target. It is not required for prove. This repository does not start it.

```bash
NO_COLOR=1 cargo run --bin helix -- verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
```

`make test-live HELIX_LIVE_URL=http://127.0.0.1:8080` is the same idea. `--format json` (alias `--report json`) prints a Helix `VerificationRun` on stdout. The default profile is `generic`. `--profile ferrum` is opt-in and is never inferred from the target ([PROFILES.md](PROFILES.md)). Exit `0` if the overall status is pass. Exit `1` on FAIL, ERROR, a skip-only report, an unreachable target, or a runtime error.

The operator path is [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). Two independent DRS implementations, Starter Kit and Bento, are in [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md).

## Commands

These need a running origin except `inspect`, `differential`, `compare`, `matrix`, and `standards` on saved files. The dummy HMAC files are not for a deployment.

```bash
cargo run --bin helix -- verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
cargo run --bin helix -- inspect verify.json
cargo run --bin helix -- differential first.json second.json
cargo run --bin helix -- compare previous.json current.json --format json
cargo run --bin helix -- standards list
cargo run --bin helix -- standards list --supported-only
cargo run --bin helix -- standards show drs 1.5.0
cargo run --bin helix -- standards validate
cargo run --bin helix -- standards trace drs.object.schema.openapi
cargo run --bin helix -- security http://127.0.0.1:8080 \
  --hmac-secret-file test-fixtures/hmac/shared-secret.txt
cargo run --bin helix -- bench --baseline http://127.0.0.1:8080 --candidate http://127.0.0.1:8080
cargo run --bin helix -- matrix --format json
```

`inspect` classifies current and historical evidence and does not rewrite the file. `differential` compares two artifacts under the same contract and does not rank implementations. `compare` treats PASS to FAIL at a stable id as a regression. It is not a score. `standards list` does not run verify and does not use the network. DRS 1.4.0 is SUPPORTED for technical verification within the declared coverage. The default `list` includes AVAILABLE rows. `--supported-only` lists DRS 1.4.0. That is not GA4GH certification and not complete DRS coverage. `security` is selected dummy-HMAC behaviour checks, not a security audit. `bench` is a 3-GET smoke (`http.drs.smoke.v1`). More than 10% worse warns. It does not fail the process. `matrix` stays pending without independent run JSON. Mocks are not a second implementation.

TES, TRS, Beacon, htsget, HMAC auth, and the rest of the HelixTest ladder still live in HelixTest. See [INVENTORY.md](../INVENTORY.md). WES fixture assumptions: [WES.md](WES.md). Authorization: [AUTHORIZATION.md](AUTHORIZATION.md) (UNEVALUATED; deferred).

The command contract is [CLI_CONTRACT.md](CLI_CONTRACT.md). What the program can do today is [HELIX_PRODUCT.md](HELIX_PRODUCT.md).
