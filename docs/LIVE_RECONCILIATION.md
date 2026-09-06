# Live evidence reconciliation (B12)

Helix is HelixTest becoming a standalone VERIFY CLI. This document is the **live evidence reconciliation** gate. It does not expand the DRS catalog, add endpoints, or rank implementations.

HelixTest already runs the DRS checks. Helix productizes that engine against **real independent local implementations** using the same pinned DRS 1.4.0 contract proven in-process (B8–B11).

**Vertraue mir nicht, vertraue dem Code.**

Trust: [TRUST.md](TRUST.md). Coverage: [COVERAGE.md](COVERAGE.md). Claims: [CLAIMS.md](CLAIMS.md). External reproduction: [EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md). Independence records: [`targets/independence.yaml`](../targets/independence.yaml). Classifier: `src/live_evidence.rs`. Tests: `tests/b12_live_reconciliation.rs`.

---

## 1. What B12 proves

The B8–B11 verification semantics survive contact with two real independent DRS implementations:

```text
same pinned verifier
+ same DRS 1.4.0 pack / checker / binding / catalog / coverage
+ real target observation
= live JSON whose claim and coverage semantics match the in-process model
```

Helix compares evidence. It does not decide which implementation is “better.”

`VERIFIED` means: verified under the declared Helix verification contract. It does not mean every DRS behavior was tested, that the implementation is certified, that authorization was verified, or that HELIOS evidence was produced.

---

## 2. Live JSON is operator evidence, not a git golden

`make prove` does **not** start Docker, clone Bento, or require live HTTP.

Live `helix verify --format json` output belongs in gitignored `local/b12/` (or `HELIX_B12_LIVE_DIR`). Do not commit volatile runtime JSON. Do not treat a checked-in example as current live evidence.

JSON captured during B12, before this Helix commit existed, has no `helix_git_sha`. Those files are **pre-commit observations**. Do not relabel them as produced by a later commit. Post-commit emits include compile-time `helix_git_sha` / `helix_git_dirty`.

There is no cosmetic `live_observation: true` flag. Stronger existing fields:

| Field | Role |
|-------|------|
| `target.identity.target_kind` | operator label |
| `targets/independence.yaml` + `run_counts_as_independent` | reviewed classification |
| `drs_fixture.source = operator_declared` | not the in-process catalog |
| `live_independent_observation` | rejects mock kind, default catalog, and `test-object-1` |

A fake `implementation_name = bento_drs` on a mock run is not live Bento evidence.

Helix still does **not** cryptographically bind a URL to a Docker digest or git commit. Artifact identity is provenance/context, not verification evidence.

---

## 3. Contract identity (must not silently change)

If any of these computed values differ from the pin, stop. Do not regenerate identities to make a live target pass.

| Identity | Pin |
|----------|-----|
| HelixTest SHA | `1baddfd3d75f01dc7c149074a785616fa014c725` |
| checker | `helixtest-drs:18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a` |
| release commit | `36145d389e0a454428d1dac5c4a30870995fdd7c` |
| pack integrity | `c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89` |
| schema document | `3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608` |
| DrsObject component | `b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003` |
| binding_id | `72da037c4ce2383f116bf195507fe1b45c60d6917acf5a87c6e5bba7043c69e2` |
| catalog_id | `03ce38f690ed679ff967e636bb037e0ed4ebe42f2cbde62766921ad7eae96ac1` |
| coverage_id | `082f63c9eec7472a66f8121a66e28ffb7f68680791f7bb5ea5ef47441d18f08c` |
| execution_id | `ee00e1a49e6b3f7d47314bde77738faf4b6cec4d3325dfe56d139809ea97037e` |

Both live targets must share the same coverage_id and the same execution_id. They must have distinct target_execution_id values.

When the spec-join completes (schema hashes present), `coverage_id` and `execution_id` do not include the target URL, target id, or fixture object id.

A SELECTED run whose checker never returned a SpecSource compile result keeps schema hashes / `execution_id` unset (`join_pack_loaded` in `src/verify.rs`). That incomplete join must not look like a successful spec-join. It is not a target choosing a different coverage contract.

---

## 4. Portable fixtures

Use `--drs-object-id` / `--drs-object-sha256` (`DrsVerifyFixture`). Do not add `--starter-kit-object-id` or `--bento-object-id`.

A different object id changes `target_execution_id` and observed evidence. It does not change pack, checker, binding, catalog, coverage, or spec-join `execution_id`.

---

## 5. Reproduce (not CI)

Reviewed target ids match `targets/independence.yaml` (`bento-drs-0.21.5`, not `bento-drs-v0.21.5`).

### Starter Kit (`ga4gh/ga4gh-starter-kit-drs:0.3.2`)

Do **not** `docker pull latest`. Use the pinned digest when the image is already present:

```text
ga4gh/ga4gh-starter-kit-drs:0.3.2
sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1
linux/amd64
```

```bash
docker run -d --name helix-b12-starter-kit-drs -p 127.0.0.1:4500:4500 --platform linux/amd64 \
  ga4gh/ga4gh-starter-kit-drs@sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1

cd Helix
mkdir -p local/b12
RUST_LOG=error cargo run --locked --bin helix -- verify http://127.0.0.1:4500 \
  --standard drs --version 1.4.0 --release-class official \
  --target-id ga4gh-starter-kit-drs-0.3.2 \
  --target-kind real-independent-local-implementation \
  --implementation-name ga4gh-starter-kit-drs \
  --implementation-version 0.3.2 \
  --drs-object-id b8cd0667-2c33-4c9f-967b-161b905932c9 \
  --format json > local/b12/starter-kit.json
```

Expected honest shape (reconcile; do not force): `detected_version = 1.3.0experimental`, `selected_version = 1.4.0`, `verified_version` null, HLX-DRS-002 FAIL, 003/004 SKIP `fixture_unavailable`, `coverage.state = blocked`, `ga4gh_requirement = not_verified`.

**Starter Kit is NOT VERIFIED.** A correct Docker digest is not verification.

### Bento DRS (`github.com/bento-platform/bento_drs@v0.21.5`)

Commit `1dc55ebea90185b1fec2c78c8c52909dd0ca889e`. Operator configuration `AUTHZ_ENABLED=false` is **not** authorization verification. `drs.security.authorization` remains `UNEVALUATED`.

```bash
RUST_LOG=error cargo run --locked --bin helix -- verify http://127.0.0.1:5000 \
  --standard drs --version 1.4.0 --release-class official \
  --target-id bento-drs-0.21.5 \
  --target-kind real-independent-local-implementation \
  --implementation-name bento_drs \
  --implementation-version 0.21.5 \
  --drs-object-id f23b1635-4a65-40fa-8b29-75b4734b602a \
  --drs-object-sha256 6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1 \
  --format json > local/b12/bento.json
```

Expected honest shape when catalog evidence PASSes: `selected_version = verified_version = 1.4.0`, `ga4gh_requirement = verified`, `coverage.state = partial`. Partial is correct: unevaluated DRS operations remain. Do not convert partial to complete because six catalog checks passed.

---

## 6. Isolation

There is no global latest result and no target result cache. `OnceLock` is used only for the independence YAML and JSON Schema. Run targets separately. Re-reading Starter Kit JSON after a Bento run must show the Starter Kit file unchanged.

---

## 7. Tamper (B12-T1–T12)

When live JSON is present, `tests/b12_live_reconciliation.rs` mutates the deserialized artifact **without** restamping. Forged `verified_version`, `coverage.state = complete`, `coverage_id`, `execution_id`, target identity, endpoint, deleted PASS evidence, SKIP→PASS, UNEVALUATED→PASS, OUT_OF_SCOPE→PASS, and authorization PASS all fail `validate_claim_integrity`.

Renaming Starter Kit as Bento does not create Bento evidence: check results and `verified_version` stay derived from the recorded execution.

---

## 8. Security (not expanded in B12)

Recorded, not newly claimed:

- Helix-owned HTTP: no redirect follow, 2 MiB body cap, rustls, no gzip ([THREAT_MODEL.md](THREAT_MODEL.md)).
- Fixture object ids remain validated (`src/fixture.rs`).
- Credentials are not sent; Authorization values are redacted.
- HelixTest `access_url` fetch remains a documented residual.
- `drs.security.authorization` is UNEVALUATED even when the target was started with `AUTHZ_ENABLED=false`.

---

## 9. HELIOS

B12 does not add RO-Crate, PDF, signatures, or archival provenance. Live JSON is inspectable Helix output, not a HELIOS evidence pack.
