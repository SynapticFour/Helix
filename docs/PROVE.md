# Prove Helix without a running platform

`make prove` is the zero-risk path for **this** repo: honesty docs plus `cargo test --locked --all-targets` against **in-process fixtures**. No Docker, no Ferrum, no hospital, no real credentials, no HELIOS.

To **see** a `helix verify` report without Ferrum: `make verify-fixture` (same DRS mock as [FIXTURES.md](FIXTURES.md) §1). That is not part of `make prove`’s test loop; run it after prove if you want the human report.

Catalog: [FIXTURES.md](FIXTURES.md). HelixTest already runs; these fixtures productize DRS/WES/security/bench checks locally. Green prove is a technical signal, not GA4GH certification. Briefing: [FOR-EVALUATORS.md](FOR-EVALUATORS.md).

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make prove
make verify-fixture
```

Needs a sibling HelixTest checkout (D1 path dep, pin in [VERSIONS.lock](../VERSIONS.lock)). Does not start servers. Missing sibling → Make exit 2 with clone instructions (`scripts/require-helixtest.sh`).

`make prove` does **not** skip, ignore, or exclude tests. Live-stack HelixTest crates stay in HelixTest; Helix has no `#[ignore]` live cargo tests today. Do not weaken a live path by folding it into prove.

The tagged live GA4GH suite is still HelixTest (`make prove` there builds `helixtest` and excludes live-stack crates). See [HelixTest docs/PROVE.md](https://github.com/SynapticFour/HelixTest/blob/main/docs/PROVE.md) and [INVENTORY.md](../INVENTORY.md).

## Live proof (needs a target you control)

Helix never starts Ferrum for the customer path. Demo-open auth is not a hospital proof. Results are not GA4GH certification. First path without a stack: `make verify-fixture`.

```bash
cd ../Ferrum && make up
# Helix (opt-in; not part of make prove):
cd ../Helix && make test-live HELIX_LIVE_URL=http://127.0.0.1:8080

# HelixTest against the same origin:
helixtest --all --mode ferrum --only drs   # same DRS checks as the mock

# non-Ferrum DRS (HelixTest CI uses an in-process mock):
DRS_URL=http://127.0.0.1:$PORT helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

Passports / ADS (co-deploy):

```bash
cd ../Ferrum-GA4GH-Demo && make up-with-infra
helixtest --all --mode ferrum+infra --profile ferrum-infra
```
