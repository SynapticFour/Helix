# Install

No Synaptic Four account. No cloud product. Helix does not phone home.

## Local tools

- [rustup](https://rustup.rs/) **Rust 1.91.1** (`Helix/rust-toolchain.toml`). Put `$HOME/.cargo/bin` before Homebrew `/opt/homebrew/bin` on `PATH`.
- `git`, `make`.
- First `cargo` / `make` may download crates from **crates.io** (public registry, no account). That is not Helix Cloud.

There is no release binary or Homebrew formula. `make install` is `cargo install --path .`.

## Clone (sibling trees)

Helix path-depends on HelixTest at `../HelixTest`. Check HelixTest out at the SHA in `Helix/VERSIONS.lock` (tag **v0.1.3**).

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make prove
make verify-fixture
```

Missing sibling → Make exit 2 with the same clone commands (`scripts/require-helixtest.sh`). If it warns that HelixTest HEAD ≠ the pin, checkout the pin; CI uses the pin.

`make prove` = docs checks + `cargo test --locked --all-targets` (in-process fixtures). No Ferrum, Docker, or credentials.

`make verify-fixture` = `helix verify` against the in-process DRS mock. Prints `HELIX VERIFICATION`. No stack to start.

Optional: `make install` then `helix verify <url>` against an origin **you** run (see [commands.md](commands.md)).

Detail: [../INSTALL.md](../INSTALL.md). First-clone pitfalls: [../EVALUATOR_JOURNEY.md](../EVALUATOR_JOURNEY.md).
