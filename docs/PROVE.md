# Prove Helix without a running platform

`make prove` is the zero-risk path for **this** repo: required docs exist and honesty strings are present. No Docker, no Ferrum, no HELIOS.

```bash
git clone https://github.com/SynapticFour/Helix.git && cd Helix
make prove
```

The live GA4GH suite is still HelixTest (`make prove` there builds `helixtest`). See [HelixTest docs/PROVE.md](https://github.com/SynapticFour/HelixTest/blob/main/docs/PROVE.md) and [INVENTORY.md](../INVENTORY.md).

## Live proof (needs a target you control)

Helix/HelixTest never start Ferrum for the customer path. Demo-open auth is not a hospital proof. Results are not GA4GH certification.

```bash
cd ../Ferrum && make up
helixtest --all --mode ferrum --only drs   # same DRS checks as the mock

# non-Ferrum DRS (HelixTest CI uses an in-process mock):
DRS_URL=http://127.0.0.1:$PORT helixtest --all --mode generic --only drs --profile ga4gh-drs --report json
```

Passports / ADS (co-deploy):

```bash
cd ../Ferrum-GA4GH-Demo && make up-with-infra
helixtest --all --mode ferrum+infra --profile ferrum-infra
```
