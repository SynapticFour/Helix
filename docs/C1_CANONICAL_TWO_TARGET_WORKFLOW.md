# C1 — Canonical two-target operator workflow

Helix is HelixTest becoming a standalone VERIFY CLI. This report records C1 of Phase C. It is not a new verification gate. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
authorization = UNEVALUATED
```

Operator path: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md). Phase C plan: [PHASE_C_PLAN.md](PHASE_C_PLAN.md).

---

## Objective

Make the existing independent DRS 1.4.0 verification capability a discoverable operator workflow:

```text
start target → helix verify → helix-verification-v1 → helix inspect → helix differential
```

against the two already-reviewed lineages (GA4GH Starter Kit DRS, Bento DRS), without adding checks, packs, or coverage.

## Existing capability reused

Unchanged Phase B machinery:

- `helix verify --standard drs --version 1.4.0 --output FILE`
- `--drs-object-id` / `--drs-object-sha256` / `--target-kind` / `--target-id`
- `helix inspect` (recompute standing; does not rewrite)
- `helix differential` (`helix-differential-v1`, `ranking_semantics: absent`)
- DRS 1.4.0 pack, checker, catalog, `coverage_id`, `execution_id` pins
- Independence records in `targets/independence.yaml`

No new catalog rows. No new SUPPORTED pack. No B13.

## Operator workflow

Canonical verify:

```bash
helix verify TARGET \
  --standard drs \
  --version 1.4.0 \
  --output FILE
```

Plus, where the target requires it: `--drs-object-id`, `--target-kind real-independent-local-implementation`, Bento `--drs-object-sha256`. Exact commands: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md).

```bash
helix inspect FILE
helix differential starter-kit.json bento.json
```

Optional helper (not `make prove`):

```bash
HELIX_BENTO_OBJECT_ID=<uuid-from-flask-ingest> make verify-independent
```

## Target setup

**Starter Kit:** Docker image `ga4gh/ga4gh-starter-kit-drs:0.3.2` at digest `sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1`, `linux/amd64` (QEMU on arm64). Image must already be present. Object id in the image: `b8cd0667-2c33-4c9f-967b-161b905932c9`. Helix does not pull.

**Bento:** source `bento_drs@v0.21.5` commit `1dc55ebea90185b1fec2c78c8c52909dd0ca889e`. Operator `AUTHZ_ENABLED=false` is not authorization evidence. Use the UUID `flask ingest` prints. Digest of 4096 ASCII `A` is `6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1`.

## Evidence workflow

`verify` writes `helix-verification-v1`. `inspect` recomputes claims/coverage/standing and does not rewrite the file. Fresh files from this binary can be `current_verifier_evidence`. `local/b12/` remains `historical_observation` and must not be restamped. Signing is HELIOS.

## Tests

`tests/c1_two_target_workflow.rs`:

- Help and docs expose the two-target path and actual CLI flags
- Helper exits 2 with `OPTIONAL LIVE VERIFICATION` when Bento object id is missing
- Helper refuses `HELIX_INDEPENDENT_OUT=local/b12`
- Two in-process fixtures: `--output` is valid `helix-verification-v1`; inspect does not rewrite; differential shares `execution_id`, distinct `target_execution_id`, no ranking
- Historical B12 files, if present, are unchanged and classified historical
- Relabeling the mock as Starter Kit is not `live_independent_observation`

Offline: `cargo fmt`, `cargo test --offline --locked --all-targets`, clippy `-D warnings`, `./scripts/prove.sh`.

## Boundary verification

- no new checks
- no new pack
- no coverage / `execution_id` / `coverage_id` change
- B13 remains deferred; `drs.security.authorization` remains UNEVALUATED
- no restamp of `local/b12/`
- no ranking / winner / score
- `make prove` does not invoke live independent verification or `docker pull`

## Remaining limitations

- Live Docker/Bento is optional. This C1 session did not require those stacks.
- QEMU/`linux/amd64` Starter Kit on arm64 remains an operator environment issue.
- `--target-kind` remains untrusted. The helper does not start targets.
- Current-vs-historical capture protocol for newly produced live JSON is C2, not C1.
- Deeper differential interpretation (C3) is not this package.

## Verdict

**PASS WITH FINDINGS**

C1 made the existing two-target path discoverable (docs, CLI help, optional helper, offline tests). Live Starter Kit and Bento were not exercised in this implementation session. That is an accepted C1 limitation, not a coverage expansion. B13 remains deferred.
