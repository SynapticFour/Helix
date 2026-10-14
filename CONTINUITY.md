# CONTINUITY — Helix

Survivability notes. **True on `origin/main` today.** Not legal advice. Not GA4GH certification. Not HELIOS.

Checked on 2026-10-14. Release tags: `git tag` (do not copy a tag or commit SHA into this file). The HelixTest revision this build compiles is `HELIXTEST_SHA` in `VERSIONS.lock`, and that tree must be a git checkout.

## What this project is

Standalone **`helix`** VERIFY CLI around the existing [HelixTest](https://github.com/SynapticFour/HelixTest) engine. Points at a GA4GH HTTP origin you already run; records what was tested. License: **Apache-2.0**. Check bodies come from HelixTest; Helix is not a new test platform. Evidence JSON is **not** HELIOS (no signatures, RO-Crate, PDF).

Exact HelixTest pin: [`VERSIONS.lock`](VERSIONS.lock) (`HELIXTEST_SHA`, `HELIXTEST_REF`). Read the file. Do not treat a SHA pasted into this note as current. `HELIXTEST_REF` is release lineage and is not necessarily the commit `HELIXTEST_SHA` names.

## Build from a clean machine (from a given tag)

### Prerequisites

- Git, `make`
- rustup toolchain **1.91.1** (`rust-toolchain.toml`) — also documented in `docs/INSTALL.md`
- Network for `make fetch` (crates.io at `Cargo.lock` checksums)
- **Sibling HelixTest as a real Git clone** at path `../HelixTest`, checked out to `HELIXTEST_SHA` (must be a normal repository with working `git rev-parse HEAD` — source tarballs or incomplete worktree copies fail `checker_provenance` / `git_pin_and_source_digest_match_lock_and_sibling`). Example: `git clone https://github.com/SynapticFour/HelixTest.git && git -C HelixTest fetch origin <SHA> && git -C HelixTest checkout <SHA>`
- On Debian/Ubuntu containers: `pkg-config`, `libssl-dev` (OpenSSL for native TLS deps)
- Enough RAM for linking debug tests (parallel `ld` may be OOM-killed on small hosts; set `CARGO_BUILD_JOBS=1` if linkers are SIGKILL’d)

### Source (both repos)

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
cd Helix
git tag -l            # if a tag exists, check it out; otherwise use the default branch
# Pin HelixTest to the SHA Helix compiles (from this Helix tree):
git -C ../HelixTest checkout "$(grep '^HELIXTEST_SHA=' VERSIONS.lock | cut -d= -f2)"
make fetch             # network: cargo fetch --locked
make prove             # docs + cargo test --locked --offline
```

Optional binary install (still needs sibling HelixTest to compile):

```bash
make install           # cargo install --path . --locked
```

DRS fixture path: `make verify-drs` (writes `verify.json`). Live origin: `docs/OPERATOR_VERIFY.md` / `make test-live HELIX_LIVE_URL=…` (you start the target).

## How to run tests

| Command | Meaning |
|---------|---------|
| `make prove` | Docs + offline tests (in-process fixtures; no Ferrum/Docker/credentials) |
| `make independent-verify` | Registry hashes + reproducibility tests (offline) |
| `make verify-drs` | DRS 1.4.0 technical verification vs fixture |
| `make verify-fixture` | Unversioned fixture verify |
| `make test-live` | Opt-in against `HELIX_LIVE_URL` |

Trust / inspectability: `docs/TRUST.md`.

## How to cut a release

No root `RELEASING.md` on audited main. Practical steps consistent with sibling repos and CI:

1. Ensure HelixTest pin in `VERSIONS.lock` is intentional and CI-green.
2. Update `CHANGELOG.md` if present / version metadata as the repo documents.
3. Annotated tag `vX.Y.Z` → push (triggers CI as configured).
4. Update any action/docs that pin Helix releases.

Confirm workflows under `.github/workflows/` before relying on automated assets.

## Continuity

Helix is maintained by a single steward (Synaptic Four). To reduce dependence on any one person, the build is documented from a clean machine in this file, the release steps are written down, and the source can be inspected and built without access to Synaptic Four systems (clean-room check dated 2026-10; Helix needs a git checkout and a pinned HelixTest SHA). Operational details such as accounts, publication rights and escrow arrangements are being documented in a non-public continuity plan.

## Minimal operator runbook

| Action | Command |
|--------|---------|
| Install | Clone Helix + HelixTest → pin SHA → `make fetch` → `make prove` |
| Start | N/A (CLI); start target separately for live verify |
| Stop | N/A |
| Back up | Retain `verify.json` / inspect outputs you care about; pin SHAs in your run records |
| Restore | Re-clone pins; re-run verify |
| Upgrade | Bump Helix tag; update `VERSIONS.lock` HelixTest SHA when checkers change; `make prove` |
