# Install

There is **no Helix CLI binary in this repository yet**. Install and run **HelixTest**:

See [HelixTest docs/INSTALL.md](https://github.com/SynapticFour/HelixTest/blob/main/docs/INSTALL.md).

```bash
git clone https://github.com/SynapticFour/HelixTest.git && cd HelixTest
make prove
helixtest --all --mode ferrum   # needs a running target
```

This repo: `make prove` only checks documentation. `helix verify` is Stage 1 ([HELIX_ROADMAP.md](HELIX_ROADMAP.md)).
