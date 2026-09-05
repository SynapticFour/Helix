# Independent verification

**Status:** Procedure implemented. Script: `scripts/independent-verify.sh`. Tests: `tests/repro.rs`. Helpers: `src/repro.rs`.

Helix is HelixTest becoming a standalone VERIFY CLI. This document is the skeptical-reviewer path: clone, pin, run, inspect. It does **not** ask anyone to trust authors or development process. Authorship (human or otherwise) is irrelevant.

It is **not** HELIOS (no signatures, RO-Crate, PDF). It is **not** GA4GH certification. It is **not** a claim of bit-for-bit identity of `helix verify` JSON files.

Trust constraint: [TRUST.md](TRUST.md). Pins: [VERSIONS.lock](../VERSIONS.lock), [Cargo.lock](../Cargo.lock), `standards/registry.yaml`. Fixtures: [FIXTURES.md](FIXTURES.md).

---

## 1. What is reproducible

Demonstrated in CI (`make independent-verify` after `make fetch`):

| Artefact | Reproduced how |
|----------|----------------|
| Check outcomes on the honest DRS fixture | Two `helix verify` calls on one mock: same `id` / `status` / `code`. JSON equal after replacing `timestamp` |
| Known-bad DRS fixture | Two runs: same fail ids, same `diagnostic.expected` / `observed` |
| `helix compare` of those two honest runs | `has_regression` is false; `same_measurement` is true |
| Registry + vendor files | `helix standards validate` checks SHA-256 of `standards/vendor/**` against `standards/registry.yaml`. No network |
| Unit/integration suite | `cargo test --locked --offline --all-targets` after `cargo fetch --locked` |

`--locked --offline` means test execution does not talk to crates.io or GitHub. Crate **versions** are the checksums in `Cargo.lock`, not “latest”. `make fetch` downloads **every** lockfile crate, including unused target triples (Windows/WASM). A same-platform `cargo test` cache alone is not enough for `cargo metadata --offline`; that is why the procedure starts with `make fetch`.

---

## 2. What is not reproducible (do not claim it)

| Source | What varies | Why it is recorded anyway |
|--------|-------------|---------------------------|
| `timestamp` | UTC seconds from the clock | Wall clock, not a signature ([RUN_IDENTITY.md](RUN_IDENTITY.md)) |
| `target.url` | `127.0.0.1:<ephemeral port>` when the mock binds `:0` | Different OS processes. Same fixture bytes |
| `helix bench` timings | p95, wall ms, RSS | Measurement, not a conformance layer |
| JWT `exp` relative to `Utc::now()` | Security fixture tokens | Dummy HMAC only; not `helix verify` |
| Pretty-printed JSON bytes of two files | Timestamp (and port if the mock restarted) | Compare `id`/`status` or canonicalize timestamp |
| First crate download | Network to crates.io | Explicit `make fetch`. Not a GA4GH fetch |
| Live `helix verify <url>` | The target you started | Operator action. Not `make prove` |
| HelixTest-vendored OpenAPI vs `standards/vendor` | Default verify still uses HelixTest’s copy; versioned DRS join uses vendor bytes when a pack is selected | Honest gap for unversioned runs ([TRUST.md](TRUST.md)). Registry hashes still inspectable |

Raw `helix verify --format json` files are **not bit-for-bit** identical across runs. **Bit-for-bit identity is not demonstrated and is not claimed.** The demonstrated property is: after replacing `timestamp`, the JSON value matches; check fingerprints match; compare is not `NEW_FAIL`.

---

## 3. Required environment

| Item | Pin |
|------|-----|
| Rust | **1.91.1** (`rust-toolchain.toml`). rustup’s `cargo` before Homebrew |
| HelixTest sibling | Git SHA in [VERSIONS.lock](../VERSIONS.lock) (`HELIXTEST_SHA`). Executed checker is `HELIXTEST_CHECKER_SOURCE_SHA256` ([CHECKER_PROVENANCE.md](CHECKER_PROVENANCE.md)) |
| Crates | [Cargo.lock](../Cargo.lock) (`cargo fetch --locked` then `--offline`) |
| OS | CI is `ubuntu-latest`. Local macOS/Linux with rustup is the documented path. Windows is unproven here |
| Locale / clock for this script | `TZ=UTC` `LC_ALL=C` `LANG=C` `NO_COLOR=1` `RUST_LOG=error` |
| Network during **prove / independent-verify** | **None** (localhost wiremock only). Network is `make fetch` and optional `make test-live` |

No Ferrum, Docker, hospital IdP, or HELIOS.

---

## 4. Exact commands (offline after fetch)

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix

# Explicit network: crates.io at Cargo.lock checksums. Not GA4GH GitHub “latest”.
make fetch

make prove                 # docs + cargo test --locked --offline --all-targets
make independent-verify    # standards validate + tests/repro.rs
make verify-fixture        # human HELIX VERIFICATION against fixture §1
```

If `cargo test --offline` fails because the crate cache is empty: run `make fetch`, then retry. Do not `cargo update`.

Inspectable CLI (still offline):

```bash
cargo run --locked --offline --bin helix -- standards validate
cargo run --locked --offline --bin helix -- standards list
cargo run --locked --offline --bin helix -- standards show ga4gh.drs.1.4.0
cargo run --locked --offline --bin helix -- standards trace drs.object.schema
```

`helix standards *` does not download specification files.

---

## 5. Pinned artefacts the reviewer opens

| Artefact | Role |
|----------|------|
| `VERSIONS.lock` | HelixTest git SHA/tag |
| `Cargo.lock` | crate checksums |
| `rust-toolchain.toml` | rustc 1.91.1 |
| `standards/registry.yaml` | pack id, GA4GH commit, vendor path, sha256 |
| `standards/vendor/ga4gh.drs.1.4.0/` (and 1.5.0, WES 1.1.0) | Bytes `helix standards validate` hashes |
| `schemas/helix-verification-v1.json` | Report shape |
| `src/identity.rs` / `src/traceability.rs` | Check ids; expected behaviour; **no shipped `normative` row** |
| `tests/support/mock_ga4gh_drs.rs` | Honest and known-invalid DRS |

---

## 6. Expected results (technical signal)

| Command | Expected |
|---------|----------|
| `make prove` | Exit 0; `Helix prove OK (in-process fixtures; not Ferrum, not certification).` |
| `make independent-verify` | Exit 0; `independent-verify: OK` |
| `make verify-fixture` | DRS five **PASS**; WES **SKIP** (not mounted); exit 0 |
| Known-invalid DRS (`start_mock_invalid_drs_object`) | `drs.object.schema` **FAIL**; diagnostic `expected` + `observed`; exit 1 |
| `helix standards list --supported-only` | `ga4gh.drs.1.4.0`. YAML is not sufficient. SUPPORTED is not VERIFIED |

Green prove is not a GA4GH MUST and not certification.

---

## 7. The eleven questions (evidence, not narrative)

| # | Question | What to run / open |
|---|----------|--------------------|
| 1 | Clone | Two git clones + HelixTest checkout at `VERSIONS.lock` |
| 2 | Install dependencies | `make fetch` (Cargo.lock). Path dep: sibling HelixTest |
| 3 | Run the suite | `make prove` |
| 4 | Same results | `tests/repro.rs`: two verifies, timestamp stripped |
| 5 | Inspect standards material | `standards/vendor/**`, `helix standards show PACK` |
| 6 | Integrity | `helix standards validate` (sha256). `Cargo.lock` checksums |
| 7 | Source of every **normative** check | There are **none**. `helix standards trace CHECK_ID` shows `check_kind` / `untraceable_reason` |
| 8 | Expected behaviour | `traceability.expected_behavior`; diagnostic `expected` on fail |
| 9 | Observed behaviour | `diagnostic.observed` / `message` on fail |
| 10 | Reproduce a failure | Invalid DRS fixture or [MUTATION.md](MUTATION.md) |
| 11 | Why Helix produced the result | `claims[]` ([CLAIMS.md](CLAIMS.md)), `status`, skip message, `standard_selection`, `layer_summary`, diagnostic `possible_causes` (not `cause`) |

---

## 8. Nondeterminism inventory

| Class | In `helix verify` (fixture)? | Mitigation |
|-------|------------------------------|------------|
| Dependency versions | No if `--locked` | `Cargo.lock`; `make fetch` then `--offline` |
| Network | Localhost mock only | No crates.io/GA4GH during prove |
| Timestamps | **Yes** (`timestamp`) | Strip for compare; do not treat as identity |
| Random values | Mock bind port if a **new** mock | Same process: same URL. JWT random n/a for verify |
| Environment variables | `RUST_LOG`, `NO_COLOR`, HMAC for **security** only | Script sets `RUST_LOG=error` `NO_COLOR=1` |
| Locale | Not used for JSON numbers/status | Script sets `LC_ALL=C` |
| Filesystem ordering | Registry YAML array order | Fixed file |
| Concurrency | tokio multi-thread; checks sequential per suite | Outcome fingerprint stable in tests |
| External services | None in prove | Live verify is explicit |
| Generated artefacts | Report JSON is an output, not a golden file | `docs/evaluator-pack/example-verify.json` is an example |
| Floating versions in `Cargo.toml` | `anyhow = "1"` etc. | **Lockfile** is the pin. Always `--locked` |
| Mutable downloads | Default verify does not fetch GA4GH | Vendor copies + hash. Do not `cargo update` for this procedure |

HelixTest HTTP client retries exist; the in-process mock answers immediately.

---

## 9. Network that is an explicit user action

| Action | Network | Mutable? |
|--------|---------|----------|
| `make fetch` / `cargo fetch --locked` | crates.io | No: lockfile checksums |
| `git clone` / HelixTest pin | Git hosting | Pin the SHA |
| `make test-live HELIX_LIVE_URL=…` | The origin **you** name | The target, not crates.io |
| `helix verify https://…` | That origin | Operator-chosen |
| `make prove` / `make independent-verify` | **Must not** | Fail if crate cache missing (exit 2 → `make fetch`) |

Normal verification (`make prove`, fixture `helix verify`) must not download GA4GH specification files. `helix standards validate` hashes local vendor bytes.

---

## 10. Known limitations

- Default `helix verify` still executes HelixTest-vendored schemas, not `standards/vendor` bytes. The versioned DRS join compiles those vendor bytes when DRS 1.4.0 is selected (`--standard drs --version 1.4.0`). Join success and check PASS are not VERIFIED.
- Exactly one shipped check is `kind=normative` (`drs.object.schema.openapi`). Fixture PASS is not a normative PASS.
- CI runner image (`ubuntu-latest`) is not pinned to a digest in this repo. rustc **is** pinned (1.91.1).
- `helix bench` is not part of independent verification of conformance outcomes.
- `helix security` uses dummy HMAC and `Utc::now()` for expiry; it is outside this fixture-verify procedure.
- This procedure does not reproduce a live hospital or Ferrum stack.

---

## 11. CI

`.github/workflows/ci.yml`: `cargo fetch --locked`, then `make prove` (`cargo test --locked --offline`), then `make independent-verify`.

That is a technical signal that the suite ran offline from the lockfile and that vendor hashes matched. It is not certification.
