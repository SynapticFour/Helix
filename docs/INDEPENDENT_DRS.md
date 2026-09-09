# Two independent DRS implementations

Helix can apply the **same** GA4GH DRS 1.4.0 verification contract to two independently maintained DRS implementations you start. This page is that operator path.

It is not a new check suite. It is not GA4GH certification. It is not HELIOS. Helix does not rank the two implementations.

**Vertraue mir nicht, vertraue dem Code.**

Fixture-only first run (in-process mock, not independent evidence): [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). Product picture: [HELIX_PRODUCT.md](HELIX_PRODUCT.md).

```text
start target
    ↓
helix verify TARGET --standard drs --version 1.4.0 --output FILE
    ↓
helix-verification-v1 JSON
    ↓
helix inspect FILE
    ↓
helix differential FILE_A FILE_B
```

---

## What this is

Two reviewed local lineages:

| Target | What it is | Independent of Helix / Ferrum / the other |
|--------|------------|-------------------------------------------|
| **GA4GH Starter Kit DRS** `0.3.2` | Java DRS from the GA4GH starter-kit project | Yes |
| **Bento DRS** `v0.21.5` | Python/Flask DRS from the C3G Bento platform | Yes |

Helix already knows how to verify DRS 1.4.0. You supply a running HTTP origin, a DRS object id (test input), and optional identity labels. Object ids are **not** a standard-version claim. A target’s declared or detected version does **not** become `verified_version`.

In-process mocks (`make verify-drs`) prove the harness. They are **not** Starter Kit or Bento. Automated C1/C2 tests use those fixtures; they are not live-target evidence. Live JSON is only what you produce against origins you started.

---

## OPTIONAL LIVE VERIFICATION

This path needs origins **you** start. Helix does not start Docker, does not `docker pull`, and does not clone Bento.

`make prove` and `cargo test --offline` do **not** require these stacks.

If a daemon, image, QEMU/`linux/amd64` emulation, or process is missing, that is a **setup failure**. It is not a DRS verification FAIL. Do not treat “target unreachable” as “the implementation failed a GA4GH requirement.”

Helper (orchestrates existing `helix` commands only; fails clearly when a target is down):

```bash
HELIX_BENTO_OBJECT_ID=<uuid-from-flask-ingest> make verify-independent
```

Defaults: Starter Kit `http://127.0.0.1:4500`, Bento `http://127.0.0.1:5000`. Override with `HELIX_STARTER_KIT_URL` / `HELIX_BENTO_URL`. Output goes to gitignored `local/independent/` as `starter-kit-current.json` and `bento-current.json` (never `local/b12/`).

---

## 1. Start GA4GH Starter Kit DRS

Pinned image (use this digest; do **not** pull `latest`):

```text
ga4gh/ga4gh-starter-kit-drs:0.3.2
sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1
platform: linux/amd64
```

**Prerequisites:** Docker. On Apple Silicon, the image is `linux/amd64` (QEMU). The image must **already** be present. Helix CI and `make prove` will not pull it.

```bash
docker run -d --name helix-starter-kit-drs -p 127.0.0.1:4500:4500 --platform linux/amd64 \
  ga4gh/ga4gh-starter-kit-drs@sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1
```

Wait until the origin answers. Fixture object id **inside this image**:

```text
b8cd0667-2c33-4c9f-967b-161b905932c9
```

service-info on this image has advertised `type.version` = `1.3.0experimental`. Helix still **selects** DRS 1.4.0 because you asked for that pack. Detected ≠ selected. Detected ≠ verified.

---

## 2. Start Bento DRS

Pinned source (not `main` / `latest`):

```text
https://github.com/bento-platform/bento_drs
tag: v0.21.5
commit: 1dc55ebea90185b1fec2c78c8c52909dd0ca889e
```

**Prerequisites:** git, Python 3.12, Poetry. `AUTHZ_ENABLED=false` is operator configuration of **Bento** so the public DRS HTTP API can be exercised without credentials. Helix still does not send credentials. That is **not** authorization verification. `drs.security.authorization` stays **UNEVALUATED**. B13 remains deferred.

```bash
git clone --branch v0.21.5 --depth 1 https://github.com/bento-platform/bento_drs.git
cd bento_drs
git rev-parse HEAD   # must be 1dc55ebea90185b1fec2c78c8c52909dd0ca889e
poetry env use python3.12
poetry install --without dev

mkdir -p /tmp/helix-bento-data
python3 -c "open('/tmp/helix-bento-data/helix.bin','wb').write(b'A'*4096)"
export AUTHZ_ENABLED=false
export BENTO_DEBUG=true
export SERVICE_BASE_URL=http://127.0.0.1:5000
export DATABASE=/tmp/helix-bento-data
export DATA=/tmp/helix-bento-data
export FLASK_APP=wsgi:application

poetry run flask db upgrade
poetry run flask ingest /tmp/helix-bento-data/helix.bin
# copy the UUID that ingest prints — that is HELIX_BENTO_OBJECT_ID
poetry run flask run --host 127.0.0.1 --port 5000
```

The SHA-256 of 4096 ASCII `A` bytes is:

```text
6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1
```

Use the UUID **this** ingest printed. Do not reuse a UUID from an old database or from historical B12 JSON.

---

## 3. Verify each target

Canonical command (actual CLI flags):

```bash
helix verify TARGET \
  --standard drs \
  --version 1.4.0 \
  --output FILE
```

`--target-kind real-independent-local-implementation` and `--target-id` / `--implementation-name` / `--implementation-version` are **operator labels**. They do not prove independence. Helix does not trust them to upgrade a mock into independent evidence.

### Starter Kit

```bash
NO_COLOR=1 helix verify http://127.0.0.1:4500 \
  --standard drs --version 1.4.0 \
  --target-id ga4gh-starter-kit-drs-0.3.2 \
  --target-kind real-independent-local-implementation \
  --implementation-name ga4gh-starter-kit-drs \
  --implementation-version 0.3.2 \
  --drs-object-id b8cd0667-2c33-4c9f-967b-161b905932c9 \
  --output starter-kit.json
helix inspect starter-kit.json
```

Honest expected shape under the current contract: selected 1.4.0, detected `1.3.0experimental`, **NOT VERIFIED**. Some checks PASS; `access_methods` typically FAIL; checksum/Range SKIP `fixture_unavailable` when the object is not a usable blob. SKIP is not PASS. Exit 0 is not VERIFIED.

**Starter Kit is NOT VERIFIED.** That is the contract working, not a Helix defect.

### Bento

```bash
NO_COLOR=1 helix verify http://127.0.0.1:5000 \
  --standard drs --version 1.4.0 \
  --target-id bento-drs-0.21.5 \
  --target-kind real-independent-local-implementation \
  --implementation-name bento_drs \
  --implementation-version 0.21.5 \
  --drs-object-id "$HELIX_BENTO_OBJECT_ID" \
  --drs-object-sha256 6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1 \
  --output bento.json
helix inspect bento.json
```

When the configured object is present and the catalog rows PASS: **VERIFIED** with `coverage.state = partial`. Partial is required: other DRS operations remain unevaluated. Bento’s own `DRS_SPEC_VERSION` / README is not `verified_version`.

---

## 4. Inspect

```bash
helix inspect starter-kit.json
helix inspect bento.json
```

`verify` writes `helix-verification-v1`. `inspect` reads the file, **recomputes** claims, coverage, and standing, and **does not rewrite** the JSON.

| Standing | Meaning |
|----------|---------|
| `current_verifier_evidence` | Produced by **this** Helix binary (git SHA and dirty flag match). Internally consistent. |
| `historical_observation` | Inspectable; produced by an earlier verifier state. |

Standing is **computed**. It is not a JSON field you can set. Pasting this binary’s SHA onto an old file does not make it current.

`current_verifier_evidence` is **not** `ga4gh_requirement` VERIFIED. `historical_observation` is **not** NOT_VERIFIED. Those are different questions.

Write new files with `--output`. Do **not** write into `local/b12/`. That directory is historical B12 evidence. `make verify-independent` writes `local/independent/starter-kit-current.json` and `bento-current.json`. Signing/audit is HELIOS, not Helix.

---

## 5. Compare the two results

```bash
helix differential starter-kit.json bento.json
```

Both runs used the same DRS 1.4.0 pack, checker, catalog, and coverage contract. They should share `execution_id` and differ in `target_execution_id`. Check-level differences (PASS vs FAIL, SKIP `fixture_unavailable` vs PASS) are observed **behavioural** differences. They are not a ranking.

`helix differential` prints the verification contract, each target’s identity and evidence standing, `ga4gh_requirement` (VERIFIED vs NOT_VERIFIED), coverage state, and a check-level table joined by check id. A missing check is **ABSENT**, not SKIP. Existing failure attribution is shown. PASS is not VERIFIED. The command does not rewrite either JSON file.

Helix does **not**:

- rank implementations
- declare a winner
- publish a quality score or leaderboard
- transfer VERIFIED from Bento onto Starter Kit

A PASS on one check is not a VERIFIED claim. Starter Kit NOT VERIFIED and Bento VERIFIED/partial can both be honest under the same contract.

Difference classes: [DIFFERENTIAL.md](DIFFERENTIAL.md).

---

## What Helix does not claim

- Official GA4GH certification
- Authorization / Passport / OAuth (B13 **DEFERRED**; `drs.security.authorization` = **UNEVALUATED**)
- Full DRS 1.4.0 (coverage is partial)
- That `make verify-drs` is independent evidence
- HELIOS signing, RO-Crate, PDF
- That historical B12 JSON is current

Engineering notes for the same stacks: [EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md). Reviewed identity records: [`targets/independence.yaml`](../targets/independence.yaml).
