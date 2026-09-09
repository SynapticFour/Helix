# C2 — Current live-evidence protocol

Helix is HelixTest becoming a standalone VERIFY CLI. This report records C2 of Phase C. It is not a coverage expansion. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
authorization = UNEVALUATED
```

Operator path: [INDEPENDENT_DRS.md](INDEPENDENT_DRS.md). C1: [C1_CANONICAL_TWO_TARGET_WORKFLOW.md](C1_CANONICAL_TWO_TARGET_WORKFLOW.md).

---

## Objective

A newly produced `helix-verification-v1` file must be recognizable as **current verifier evidence** when this Helix binary produced it. Historical B12 evidence must remain inspectable, byte-identical, and unrestamped.

Current evidence is **produced by execution**, not declared by metadata.

## Current evidence contract

Standing is computed by `classify_evidence` (`src/evidence.rs`). It is **not** a JSON field.

`current_verifier_evidence` requires:

1. Internal consistency (`claim_join`, coverage, claims, identities).
2. The file’s `helix_git_sha` and `helix_git_dirty` match **this** binary (`cites_this_verifier_build`).

`helix_git_sha` / `helix_git_dirty` are also bound into `claim_join`. Pasting this binary’s SHA onto an old file **without** restamping the join is **Invalid**, not Current.

Operator labels (`--target-id`, `--implementation-version`, endpoint) are not verifier provenance. `current_verifier_evidence` is **not** `ga4gh_requirement` VERIFIED.

Schema remains `helix-verification-v1`. The join fields are optional so historical files still load.

## Historical evidence

`local/b12/starter-kit.json` and `local/b12/bento.json` have no `helix_git_sha`. They classify as `historical_observation` when consistent. `helix inspect` does not rewrite them. `make verify-independent` refuses that directory.

Pinned byte identity (must remain):

| File | SHA-256 |
|------|---------|
| `local/b12/starter-kit.json` | `5e8cc568f942b93af4e97f3f8da48b5ee58a833975f2e1495310d2106547c08b` |
| `local/b12/bento.json` | `61d99c5e0da5a307e0e577fbdd1e951fa52b4aea6d5ce9cf8bf35f3ae4a5f6e6` |

## Operator workflow

```bash
helix verify TARGET --standard drs --version 1.4.0 --output FILE
helix inspect FILE
```

Choose `--output` yourself. Helper default (optional live, not prove):

```text
local/independent/starter-kit-current.json
local/independent/bento-current.json
```

Do not copy B12 into that directory. Automated tests use in-process fixtures; they are not live Starter Kit or Bento evidence.

## Inspect behaviour

`helix inspect FILE` reads, recomputes claims/coverage/standing, and never writes the file. Hash before equals hash after.

## Tamper tests

| Case | Result |
|------|--------|
| Genuine current (this binary) | `current_verifier_evidence` |
| Consistent historical (no SHA / other SHA with matching join) | `historical_observation` |
| JSON `"standing": "current_verifier_evidence"` | Ignored; standing recomputed |
| Paste current SHA without restamping join | `invalid` |
| Check PASS→FAIL without restamp | `invalid` |
| Paste current SHA **and** recompute join | Can look current — **unsigned restamp**. HELIOS is the signing bound. |

## Live execution

This session: **live evidence not exercised**.

| Target | Probe |
|--------|--------|
| Starter Kit `http://127.0.0.1:4500` | down |
| Bento `http://127.0.0.1:5000` | down |

No live JSON was manufactured. Automated C2 tests used the in-process DRS mock. Historical B12 files were read, not rewritten.

## Identity invariants

Unchanged pins: `execution_id` `ee00e1a4…`, `coverage_id` `082f63c9…`, checker `helixtest-drs:18bf4a44…`, pack `c3836145…`, binding `72da037c…`, catalog `03ce38f6…`. `helix_git_sha` does not enter those hashes.

## Boundaries

- no new DRS checks
- no new pack
- no new coverage contract
- B13 remains deferred
- no WES SUPPORTED
- no HELIOS signing
- no ranking
- no cache
- no telemetry
- no automatic upload
- `make prove` still does not start live targets

## Findings

- **Unsigned consistent restamp** can still classify as current if someone recomputes `claim_join` after writing this binary’s SHA. That is not cryptographic non-repudiation. HELIOS remains the signing/audit product.
- If the binary was built without `.git`, new files have no SHA and cannot be `current_verifier_evidence`.
- Live targets remain optional. This session did not fabricate live evidence.

## Verdict

**PASS WITH FINDINGS**

The protocol is executable and tested. Live Starter Kit/Bento were not exercised here. The HELIOS restamp limitation is explicit, not hidden.
