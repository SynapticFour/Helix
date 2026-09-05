# Install

Helix productizes **HelixTest** (separate git root, [DECISIONS.md](DECISIONS.md) D1). Five-minute briefing: [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

## Requirements

- **Rust 1.91.1** via [rustup](https://rustup.rs/) (`rust-toolchain.toml`). CI uses that channel.
  - Put `$HOME/.cargo/bin` **before** Homebrew `/opt/homebrew/bin` on `PATH`. Otherwise `which rustc` may be Homebrew 1.97+ and ignore `rust-toolchain.toml`.
  - `rustup toolchain install 1.91.1` if needed. `rustc --version` should report 1.91.1 when you are in this directory and using rustup’s cargo.
- A **sibling** HelixTest git clone, checked out at the SHA in [VERSIONS.lock](../VERSIONS.lock) (tag **v0.1.3**). Cargo.toml path-depends on `../HelixTest/helixtest/crates/{common,framework}`.
- First build: **`make fetch`** (`cargo fetch --locked`). That is crates.io at lockfile checksums, not a GA4GH download. After that, `make prove` is **offline**.

There is no Homebrew formula or GitHub release binary yet. `make install` is `cargo install --path .`.

## Commands

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch              # network, Cargo.lock; not GA4GH
make prove              # docs + cargo test --locked --offline
make independent-verify # vendor sha256 + two-run fixture equality
make verify-fixture     # helix verify against the mock DRS (prints HELIX VERIFICATION)
make install            # optional: helix on PATH (~/.cargo/bin)
```

If HelixTest is missing, Make prints the clone/pin commands (exit 2) instead of Cargo’s path-not-found error.

`make prove` does not need a running target ([FIXTURES.md](FIXTURES.md)). `make verify-fixture` starts the mock for you. `helix verify http://127.0.0.1:8080` needs a stack **you** started.

Skeptical reviewer path (pins, hashes, two-run equality, what is **not** bit-for-bit): [INDEPENDENT_VERIFICATION.md](INDEPENDENT_VERIFICATION.md).

`helix verify` discovers GA4GH HTTP APIs under the URL and runs HelixTest DRS and WES checks when those APIs answer. TES/TRS/htsget are discovered but not executed. That is not GA4GH certification.

HelixTest remains usable on its own:

```bash
cd ../HelixTest
make prove
helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

See [HelixTest docs/INSTALL.md](https://github.com/SynapticFour/HelixTest/blob/main/docs/INSTALL.md). Ferrum as an optional reference target: `cd ../Ferrum && make up`, then `make test-live HELIX_LIVE_URL=http://127.0.0.1:8080` from Helix.
