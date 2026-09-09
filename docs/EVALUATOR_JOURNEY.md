# Evaluator journey (2026-09-04)

Record of walking into Helix as a technically competent outsider: GitHub → clone → README → install → `make prove` → `helix verify` on the deterministic fixture. This is not a product brief. Fixes that followed are listed at the end. The five-minute briefing is [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

Helix is HelixTest becoming a standalone VERIFY CLI. HelixTest already runs. HELIOS (`helios-audit`) stays separate. Ferrum is a reference target, not a Helix dependency, and has **no** real clinical pilot.

Method: clone `https://github.com/SynapticFour/Helix` into a clean `/tmp` tree; follow README/INSTALL only; then retry with a sibling HelixTest clone (default branch, not the pin).

---

## What would confuse an evaluator

### 1. GitHub org does not lead with Helix

`github.com/SynapticFour` lists unrelated public repos first (gatk-rs, cognitive-landscape, …). Helix is not in the org “top repositories.” An evaluator looking for a GA4GH verifier has to already know the repo name.

**Residual.** Org profile is outside this repository.

### 2. GitHub one-line description is insider language

The GitHub subtitle is “Independence of HelixTest…”. That does not say what to clone, what binary to run, or what a result means.

**Residual** (GitHub UI field). README first paragraph must carry the meaning.

### 3. `docs/FOR-EVALUATORS.md` said the repo was not runnable

On clone, that file opened with: “This repository is not yet a runnable suite.” README and `make prove` contradict it. An evaluator who opens the evaluator doc first will stop.

**Fixed:** [FOR-EVALUATORS.md](FOR-EVALUATORS.md) is now the five-minute briefing. `scripts/prove.sh` fails if the stale sentence returns.

### 4. Cloning Helix alone does not build

```text
failed to load source for dependency `helixtest-common`
unable to update …/HelixTest/helixtest/crates/common
No such file or directory
```

README does mention a sibling clone, but after a `git clone Helix && cd Helix && cargo test` the Cargo error never says “clone HelixTest next to this directory” or points at INSTALL.

**Fixed:** `scripts/require-helixtest.sh` runs before `make prove` / `make test` / `make verify-fixture` and prints the clone + pin commands.

### 5. Default HelixTest branch is not the pin

`VERSIONS.lock` wants `HELIXTEST_SHA=1832c043…` (tag **v0.1.3**). `git clone HelixTest` (default branch) resolved to a different SHA (`166c676` on this walk). `cargo check` still succeeded. The evaluator’s tree is then not CI.

**Fixed:** INSTALL and FOR-EVALUATORS check out the lock SHA. `require-helixtest.sh` warns on mismatch (does not fail; the tree may still compile).

### 6. “Install Helix” is not an install

There is no release binary, formula, or `cargo install` in the README quick start. INSTALL jumps to `make prove` and then `helix verify http://127.0.0.1:8080`. `helix` is never on `PATH` unless the evaluator invents `cargo install --path .`.

**Fixed:** INSTALL documents `cargo install --path . --locked` (still needs the sibling). Makefile `make install`. Quick start does not pretend a package exists.

### 7. Two Rust compilers

`rust-toolchain.toml` asks for **1.91.1**. `which rustc` on this machine was Homebrew **1.97.1**; rustup had 1.91.1 as default but not first on `PATH`. CI is 1.91.1. Evaluators will not know which compiler `make prove` used.

**Fixed:** INSTALL states: use rustup 1.91.1 (`$HOME/.cargo/bin` before Homebrew). README points there.

### 8. `make prove` is not `helix verify`

`make prove` is docs greps + `cargo test --locked`. Green prove prints “Helix prove OK”. It never prints `HELIX VERIFICATION`. An evaluator who was asked to “run Helix” has not seen a verification report.

**Fixed:** `make verify-fixture` runs the `helix verify` path against the in-process DRS fixture and prints the human report.

### 9. Documented `helix verify` target is Ferrum, not the fixture

README / INSTALL / `make help` all showed `http://127.0.0.1:8080` first. That port is empty unless the evaluator already has Ferrum (`make up`) or something else. Connection refused / unreachable ERROR looks like Helix is broken. The deterministic fixture in [FIXTURES.md](FIXTURES.md) §1 is only used inside `cargo test`, which the README never says to use as `helix verify`.

**Fixed:** first live command is `make verify-fixture`. `127.0.0.1:8080` is labeled optional Ferrum/live.

### 10. Too many docs, too many names

README “Documentation” is a long dump (vision, roadmap, schema, threat model, …). Names collide: Helix, HelixTest, `helix`, `helixtest`, HELIOS, `helios-audit`, Ferrum, profile `ferrum`. Stage numbers, D1, SKU, ambassador appear before “open an issue.”

**Fixed:** FOR-EVALUATORS answers the seven questions in five minutes. README points there first. EVALUATOR_JOURNEY (this file) is the confusion log, not the briefing.

### 11. What a result means is split across files

DETECTED vs PASS, skip-only exit 1, “not certification” live in CLI_CONTRACT, DISCOVERY, REPORT. The sample README report is a Ferrum-shaped gateway with TES/TRS DETECTED — not what `make verify-fixture` prints (DRS only; WES skipped).

**Fixed:** FOR-EVALUATORS states: DETECTED is not a pass; skip is never pass; green is a technical signal; fixture run is DRS pass + WES skip.

### 12. How to report a failure is hedged

CONTRIBUTING / README: Issues, plus Discussions “once that tab is enabled.” An evaluator does not know whether Discussions exists.

**Fixed:** report via GitHub Issues on this repo. Security issues: SECURITY.md (email). Discussions optional.

### 14. HelixTest DEBUG traces hid the report

`make prove` and a raw `helix verify` printed HelixTest `HttpClient` DEBUG/INFO lines (`common=debug` default) before `HELIX VERIFICATION`. An evaluator could think the product output is HTTP traces.

**Fixed:** Helix sets `RUST_LOG=error` when unset (`helix::default_client_log_filter`). Makefile exports the same. `RUST_LOG=debug` still restores traces. The report is stdout; traces stay stderr.

---

## Commands that should work after the fixes

Sibling clones, HelixTest at `VERSIONS.lock` SHA, rustup 1.91.1:

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make prove
make verify-fixture
```

`make prove` = docs honesty + unit/integration tests on in-process fixtures. `make verify-fixture` = `helix verify` against [FIXTURES.md](FIXTURES.md) §1 (not Ferrum). Neither is GA4GH certification.

---

## What this file will not fix

- GitHub org landing page and repo subtitle
- Homebrew `rustc` preceding rustup on a given machine (documented only)
- HelixTest’s own CLI (`helixtest`) as a second entry point (still valid; not required for this journey)
