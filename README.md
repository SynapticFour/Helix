# Helix

Helix checks a genomic HTTP service you already run, and it records what was tested, what was left out, and what must not be claimed.

For someone who already runs that service and wants the record, not a new server.

**Status: No public status recorded yet.** The claims registry has no Helix stage.

On the default branch `9327b96aa71f9b757bcb52dd4a554aa2faa0ddb3`, this repository contains versioned GA4GH DRS 1.4.0 technical verification with declared partial coverage, a persisted `helix-verification-v1` report, `helix inspect`, and optional `helix differential`. DRS 1.4.0 is SUPPORTED for that technical verification. SUPPORTED is not a VERIFIED claim. WES 1.1.0 is AVAILABLE and is not a published Supported standard. Authorization verification is deferred. Default `helix verify` without `--standard` and `--version` stays unversioned. The source of that support line is `src/standards/support.rs` and [docs/STANDARDS_REGISTRY.md](docs/STANDARDS_REGISTRY.md).

[![CI](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml/badge.svg)](https://github.com/SynapticFour/Helix/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

The check bodies come from [HelixTest](https://github.com/SynapticFour/HelixTest), a separate git repository. Helix is that engine as a standalone `helix` binary. The exact HelixTest source is the SHA in [VERSIONS.lock](VERSIONS.lock). Tag v0.1.3 is the HelixTest release lineage, not the commit Helix compiles. [HELIOS](https://github.com/SynapticFour/HELIOS) stays signed evidence, RO-Crate, and PDF. Those do not move into Helix.

Do not take this page on trust. Inspect pins, tests, and recorded JSON: [docs/TRUST.md](docs/TRUST.md). What the program can do today: [docs/HELIX_PRODUCT.md](docs/HELIX_PRODUCT.md).

## What it does

- You point it at a GA4GH HTTP origin you already run. GA4GH publishes HTTP API specifications. Helix is not GA4GH and is not certified by GA4GH.
- Helix runs the same documented DRS and WES checks against any HTTP origin that implements those GA4GH paths. Ferrum is used as a reference target, not a dependency.
- Helix supports technical verification checks for GA4GH DRS 1.4.0 within the declared coverage boundary. That is not a VERIFIED claim and not GA4GH certification.
- `make prove` runs in-process fixtures. It does not need Ferrum, Docker, or credentials.
- `helix inspect` classifies a saved report. `helix differential` compares two artifacts under the same contract.

## Limits

- Not a server. Point `helix` at a target you already started, or use `make verify-fixture` or `make verify-drs`.
- Not HELIOS. No signed audit trails, no RO-Crate, no PDF, and no scientific-reproducibility envelope. The binary name is `helix`.
- Not GA4GH certification. Green CI and a green `helix verify` are a technical signal.
- Not complete DRS coverage. Declared coverage is partial.
- WES is not a published Supported standard. Authorization verification is deferred.
- Not a Ferrum production claim. Ferrum is on-premises, Rust, and tested. A demo and CI are not a pilot and not production. There is no clinical deployment with German hospital data-integration centres (DIZ) or the genomDE programme.
- Not a ranking of implementations. In-process mocks are not a second implementation.
- The dummy HMAC files under `test-fixtures/hmac/` are not for a deployment. `helix security` is not a security audit.

## Quick start

Prerequisites: Rust 1.91.1, and a sibling [HelixTest](https://github.com/SynapticFour/HelixTest) git checkout at `HELIXTEST_SHA` in [VERSIONS.lock](VERSIONS.lock).

```bash
make fetch
make prove
make verify-drs
```

You should see the line `Helix prove OK (in-process fixtures; not Ferrum, not certification).`

`make fetch` needs a network. `make prove` and `make verify-drs` then run offline against in-process fixtures. The clone and pin commands are in [docs/GETTING-STARTED.md](docs/GETTING-STARTED.md).

Against an origin you started, for example [Ferrum](https://github.com/SynapticFour/Ferrum) after `make up` there:

```bash
NO_COLOR=1 cargo run --bin helix -- verify http://127.0.0.1:8080 --standard drs --version 1.4.0 --output verify.json
```

This repository does not start that origin.

## Documentation

Every file under `docs/` is listed in [docs/README.md](docs/README.md). The groups are a tutorial, how-to guides, reference, and explanation.

### Tutorial

- [Getting started](docs/GETTING-STARTED.md)
- [Install](docs/INSTALL.md)
- [Prove](docs/PROVE.md)
- [Operator verify](docs/OPERATOR_VERIFY.md)

### How-to guides

- [Fixtures](docs/FIXTURES.md)
- [External target](docs/EXTERNAL_TARGET_CONTRACT.md)
- [Independent DRS](docs/INDEPENDENT_DRS.md)
- Further task pages are in the [index](docs/README.md#how-to-guides)

### Reference

- [For evaluators](docs/FOR-EVALUATORS.md)
- [Trust](docs/TRUST.md)
- [Standards registry](docs/STANDARDS_REGISTRY.md)
- [Command contract](docs/CLI_CONTRACT.md)
- [Changelog](CHANGELOG.md)
- Further reference pages are in the [index](docs/README.md#reference)

### Explanation

- [Architecture](docs/ARCHITECTURE.md)
- [Product](docs/HELIX_PRODUCT.md)
- [Helix and HELIOS](docs/HELIX_VS_HELIOS.md)
- [Decisions](docs/DECISIONS.md)

## Contributing

Open an issue for a bug, missing coverage, or a `helix verify` or `make verify-fixture` run. Small changes after an issue: [CONTRIBUTING.md](CONTRIBUTING.md). Who reviews a normative mapping: [docs/STANDARDS_REGISTRY.md](docs/STANDARDS_REGISTRY.md). Do not add HELIOS-style evidence here.

## Licence

Apache License 2.0. The file is [LICENSE](LICENSE). The same licence is on HelixTest. There is no separate Helix licence claim in the registry.

## Cite

Use [CITATION.cff](CITATION.cff). Prefer that file over a hand-copied line.

## Website

- Open tools: [https://synapticfour.com/en/offene-werkzeuge/](https://synapticfour.com/en/offene-werkzeuge/)
- Technical profile: [https://synapticfour.com/en/technical/code/](https://synapticfour.com/en/technical/code/)

The same pages exist in German (`/de/offene-werkzeuge/`, `/de/technik/code/`) and in French (`/fr/offene-werkzeuge/`, `/fr/technique/code/`).

The map of repositories: [profile/README.md](https://github.com/SynapticFour/.github/blob/main/profile/README.md).

Contact: [contact@synapticfour.com](mailto:contact@synapticfour.com)
