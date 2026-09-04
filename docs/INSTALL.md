# Install

Helix productizes **HelixTest** (separate git root, [DECISIONS.md](DECISIONS.md) D1). Clone them as siblings so the path dependency `../HelixTest` resolves:

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
cd Helix
make prove
cargo run --bin helix -- verify http://127.0.0.1:8080 --format json
```

`helix verify` discovers GA4GH HTTP APIs under the URL and runs **HelixTest DRS checks** when DRS answers. That is not GA4GH certification. WES checks are not wired yet (Stage 1 exit still needs DRS and WES against Ferrum local).

HelixTest remains usable on its own:

```bash
cd HelixTest
make prove
helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

See [HelixTest docs/INSTALL.md](https://github.com/SynapticFour/HelixTest/blob/main/docs/INSTALL.md). Ferrum as a reference target: start it with `make up`, then `helix verify http://127.0.0.1:8080`.
