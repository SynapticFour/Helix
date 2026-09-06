# External-target evidence (B7)

Helix is HelixTest becoming a standalone VERIFY CLI. This document says how to reproduce a **live independent local DRS** run. It does not store live JSON in git. It does not claim the GA4GH Starter Kit is DRS 1.4.0 VERIFIED. HELIOS still owns signed evidence. Trust: [TRUST.md](TRUST.md). Checker identity: [CHECKER_PROVENANCE.md](CHECKER_PROVENANCE.md).

CI does **not** run this. Do **not** `docker pull` from prove. Do **not** auto-start Docker.

---

## 1. What is authoritative

| Artefact | Authoritative? |
|----------|----------------|
| `VERSIONS.lock` `HELIXTEST_CHECKER_SOURCE_SHA256` + sibling HelixTest sources Cargo compiles | Yes — verifier identity |
| Pinned DRS 1.4.0 vendor pack hashes | Yes — standard identity |
| `cargo test --offline --locked` | Yes — harness behaviour |
| Live `helix verify` JSON | Only the file you just produced. `/tmp` is not a source of truth |
| This document’s historical Starter Kit notes | Operator-reproduced observations, not a certificate |

Do not fake timestamps, target state, or results.

---

## 2. Reproduce the Starter Kit observation

Image used in B5/B6/B7 local runs (linux/amd64; qemu on arm64 Docker):

```text
ga4gh/ga4gh-starter-kit-drs:0.3.2
digest sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1
```

The image reports `type.version` = `1.3.0experimental`. Its own documentation does not claim a published DRS specification. Helix still runs the **DRS 1.4.0** verification path when asked. That is attribution, not a claim the Starter Kit implements 1.4.0.

```bash
# Only if the image is already present. Do not docker pull from Helix CI.
docker run -d --name helix-b7-starter-kit-drs -p 127.0.0.1:4500:4500 \
  ga4gh/ga4gh-starter-kit-drs:0.3.2

# Wait until service-info answers, then:
curl -sS http://127.0.0.1:4500/ga4gh/drs/v1/objects/b8cd0667-2c33-4c9f-967b-161b905932c9 | head

cd Helix
RUST_LOG=error cargo run --locked --bin helix -- verify http://127.0.0.1:4500 \
  --standard drs --version 1.4.0 --release-class official \
  --drs-object-id b8cd0667-2c33-4c9f-967b-161b905932c9 \
  --target-id ga4gh-starter-kit-drs-0.3.2 \
  --target-kind real-independent-local-implementation \
  --implementation-name ga4gh-starter-kit-drs \
  --implementation-version 0.3.2 \
  --format json

docker rm -f helix-b7-starter-kit-drs
```

Expected honest outcome (do not weaken checks to green this):

- `target_kind` = `real_independent_local_implementation` (operator label)
- HLX-DRS-001 PASS (object GET 200)
- HLX-DRS-006 PASS (pinned DRS 1.4.0 SpecSource against the JSON)
- HLX-DRS-002 FAIL if `access_methods` extras are absent
- HLX-DRS-003 / HLX-DRS-004 SKIP `fixture_unavailable` when there is no `access_url`
- HLX-DRS-005 PASS (derived unknown id)
- `verified_version` = null
- overall **NOT VERIFIED**

**Starter Kit is NOT VERIFIED.**

Record from the JSON you produced: standard commit, pack/schema hashes, `checker_id`, fixture object id, results. That file is the evidence. Copy it somewhere durable if you need it; Helix does not commit live captures.

---

## 3. Reproduce the Bento DRS observation (B9 Target B)

Pinned source release (not `main` / `latest`):

```text
repository: https://github.com/bento-platform/bento_drs
tag: v0.21.5
commit: 1dc55ebea90185b1fec2c78c8c52909dd0ca889e
license: LGPL-3.0
lineage: C3G Bento platform (not GA4GH starter-kit, not Ferrum, not HelixTest)
```

`AUTHZ_ENABLED=false` is operator configuration of the target so Helix can exercise the public DRS HTTP API without credentials. Helix still does not send or record credentials. This is not a Helix catalog special-case.

```bash
git clone --branch v0.21.5 --depth 1 https://github.com/bento-platform/bento_drs.git
cd bento_drs
# confirm: git rev-parse HEAD == 1dc55ebea90185b1fec2c78c8c52909dd0ca889e
poetry env use python3.12
poetry install --without dev

mkdir -p /tmp/b9-bento-data
python3 -c "open('/tmp/b9-bento-data/helix-b9.bin','wb').write(b'A'*4096)"
export AUTHZ_ENABLED=false
export BENTO_DEBUG=true
export SERVICE_BASE_URL=http://127.0.0.1:5000
export DATABASE=/tmp/b9-bento-data
export DATA=/tmp/b9-bento-data
export FLASK_APP=wsgi:application

poetry run flask db upgrade
poetry run flask ingest /tmp/b9-bento-data/helix-b9.bin
# note the printed object UUID, then:
poetry run flask run --host 127.0.0.1 --port 5000
```

```bash
cd Helix
RUST_LOG=error cargo run --locked --bin helix -- verify http://127.0.0.1:5000 \
  --standard drs --version 1.4.0 --release-class official \
  --drs-object-id <ingested-uuid> \
  --target-id bento-drs-0.21.5 \
  --target-kind real-independent-local-implementation \
  --implementation-name bento_drs \
  --implementation-version 0.21.5 \
  --format json
```

Do not infer DRS version from the git tag or README. Record `detected_version` from service-info. `selected_version` is 1.4.0 because the operator selected the Helix support contract. `verified_version` is stamped only from executed predicates.

Then:

```text
helix differential starter-kit.json bento.json --format json
```

CI does **not** clone or start Bento. Live JSON is not committed.
