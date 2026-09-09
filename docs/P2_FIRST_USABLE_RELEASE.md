# P2 — First usable Helix release

Helix is HelixTest becoming a standalone VERIFY CLI. This document records the first usable operator release of the existing DRS 1.4.0 technical verification capability. It is not a new verification architecture. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Product contract: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md).

```text
B13 remains closed/deferred.
authorization = UNEVALUATED
```

---

## Verdict

```text
PASS WITH FINDINGS
```

An independent operator can obtain Helix from source, learn that DRS 1.4.0 is the supported pack, run that verification, retain `helix-verification-v1`, and inspect it without developer assistance. The in-process fixture is the clean-environment proof. Live Starter Kit `:4500` and Bento `:5000` were not listening; `local/b12/` JSON was loaded for P2-T14/T15 and was not restamped.

---

## Release identity

Proof binary (`helix --version` at P2 time):

```text
helix 0.1.0
Helix git: 8d4110856222ae8d3c3184912fc4e193e70291c4
HelixTest pin: v0.1.3
Not GA4GH certification. Not HELIOS.
```

That P2 snapshot presented the HelixTest tag as `HelixTest pin:`. Current `helix --version` prints **lineage** (`v0.1.3`) separately from the exact source SHA (`HELIXTEST_SHA`) and checker id. The SHA was already `1baddfd3d75f01dc7c149074a785616fa014c725` at P2.

| Identity | Value | How an operator sees it |
|----------|--------|-------------------------|
| Helix package version | `0.1.0` (`Cargo.toml`; `publish = false`) | `helix --version` first line |
| Helix git | compile-time SHA (`HELIX_GIT_SHA`); this proof compiled against B15 HEAD `8d4110856222ae8d3c3184912fc4e193e70291c4` with a dirty tree (P1+P2 uncommitted at compile) | `helix --version` `Helix git:` / report `Helix:` |
| HelixTest pin | tag lineage `v0.1.3` / exact SHA `1baddfd3d75f01dc7c149074a785616fa014c725` | Current `helix --version` `HelixTest lineage:` / `HelixTest source:` / report `Test suite:` |
| Selected GA4GH version | `1.4.0` when `--standard drs --version 1.4.0` | report `selected:` / JSON `selected_version` |

There is **no** git tag, crates.io crate, Homebrew formula, or container image. Installation is a source build ([INSTALL.md](INSTALL.md)). Do not confuse these four identities. A later rebuild after this commit is committed will stamp a different `HELIX_GIT_SHA`.

---

## Supported workflow

Canonical path ([OPERATOR_VERIFY.md](OPERATOR_VERIFY.md)):

```text
helix --version
helix standards list --supported-only
helix verify URL --standard drs --version 1.4.0 --output verify.json
helix inspect verify.json
```

Without a live DRS:

```bash
make verify-drs
```

Default `helix verify URL` stays unversioned. `make verify-fixture` stays the unversioned human report.

---

## Environment

| Item | This session |
|-------|----------------|
| Date | 2026-09-08 |
| Helix tree | P1 product page + P2 operator path (uncommitted at proof compile) |
| HelixTest | `1baddfd3d75f01dc7c149074a785616fa014c725` (not modified) |
| Live Starter Kit `:4500` | not listening |
| Live Bento `:5000` | not listening |
| Fixture | in-process mock (`make verify-drs` / P2 tests) |
| B12 JSON | gitignored `local/b12/` present; historical; not restamped |
| `CARGO_TARGET_DIR` | `/Users/SynapticFour/devel/b71b-clean/target` (local disk; not a Helix runtime dependency) |

No undocumented env vars. No Ferrum. No HELIOS. No credentials. `verify.json` is gitignored.

---

## Installation/build

Validated procedure:

```bash
git clone https://github.com/SynapticFour/Helix.git
git clone https://github.com/SynapticFour/HelixTest.git
git -C HelixTest checkout "$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)"
cd Helix
make fetch
make prove
make verify-drs
```

`make install` is optional (`cargo install --path . --locked`). Verify the binary with `helix --version` and `helix standards list --supported-only`.

---

## Verification

`HELIX_VERIFY_JSON=<temp> make verify-drs` on 2026-09-08:

- selected `drs` / `1.4.0`; `support_status: SUPPORTED`; `substituted: no`;
- 6 PASS / 0 FAIL / 0 ERROR / 8 SKIP (WES unselected);
- `ga4gh_requirement` VERIFIED; `coverage.state: partial`; authorization unevaluated;
- `schema_version` `helix-verification-v1`;
- identities match B11/B12 pins (`execution_id`, pack, schema, binding, catalog, coverage);
- inspect printed `current_verifier_evidence` and did not rewrite the file.

That fixture is **not** independent-implementation evidence. `helix_git_dirty` was true at this compile.

---

## Two-target validation

Same operator flags as [LIVE_RECONCILIATION.md](LIVE_RECONCILIATION.md). Live HTTP was down. P2-T14 / P2-T15 load `local/b12/*.json` when present:

| Target | Claim | Coverage |
|--------|--------|----------|
| Starter Kit DRS 0.3.2 | NOT_VERIFIED (detected `1.3.0experimental`) | blocked |
| Bento DRS v0.21.5 | VERIFIED 1.4.0 under the current contract | partial |

Not a ranking. Not certification. Authorization remains unevaluated on both.

---

## Evidence

`--output FILE` or `--format json` emits `helix-verification-v1`. `helix inspect FILE` recomputes claims, coverage, and standing. Schema is not bumped. Inspect does not rewrite. Historical `helix_git_sha` mismatch is `historical_observation`. Unsigned. Not HELIOS.

---

## Limitations

- Source build only; sibling HelixTest required at compile time.
- DRS 1.4.0 coverage is partial; authorization deferred.
- DRS 1.5.0 and WES packs are AVAILABLE, not SUPPORTED (no silent substitution).
- `make verify-drs` is a fixture, not Starter Kit or Bento.
- Exit 0 is check-PASS, not VERIFIED.
- No published distribution artifact.

---

## Findings

### HIGH — first documented path never produced inspectable DRS 1.4.0 evidence (fixed)

README / `make verify-fixture` ran **unversioned** verify and printed text only. An operator could not complete verify → JSON → inspect without developer knowledge.

**Fixed:** `make verify-drs`, `--output FILE`, OPERATOR_VERIFY seven-step workflow, `helix --version` long identity.

### MEDIUM — Helix identity not visible as `helix --version` (fixed)

Package `0.1.0` was the only CLI version line. Long version now includes git SHA, HelixTest lineage, exact source SHA, and checker id.

### OBSERVATION — missing `--drs-object-id` without DRS service-info looks like unsupported_test

If discovery’s only DRS probe is a 404 on the configured object, DRS can be `NOT_DETECTED` and skips attribute as `unsupported_test`. Documented in [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). P2-T13 mounts DRS `service-info` so the 404 is `fixture_unavailable` / `target_configuration_failure`. Not a coverage expansion.

### OBSERVATION — live independent targets down

Fixture + historical B12 JSON. Do not restamp B12.

### OBSERVATION — no GitHub release binary

Documented. Not manufactured for P2.

---

## Test results

```text
cargo fmt --all -- --check                         # exit 0
cargo clippy --offline --locked --all-targets --all-features -- -D warnings  # exit 0
cargo test --offline --locked --test p2_first_usable_release -- --test-threads=1
  20 passed; 0 failed  (T1–T18 + detected-vs-selected + Makefile/docs)
cargo test --offline --locked --test b15_operator_ux -- --test-threads=1
  31 passed
./scripts/prove.sh                                 # prove: docs OK
HELIX_VERIFY_JSON=<temp> make verify-drs           # fixture VERIFIED/partial; inspect OK
cargo test --locked --offline --all-targets       # all crates/examples/tests ok (no FAILED)
```

T14/T15 executed against `local/b12/` in this session. T12 used `http://127.0.0.1:1` (connect timeout). T11: DRS 1.5.0 → `AVAILABLE_BUT_NOT_SUPPORTED`, `substituted: false`. `/local/` is gitignored and is not required to prove.

---

## Scope confirmation

P2 did **not** add:

- authorization;
- new standards or DRS versions;
- coverage expansion;
- HELIOS integration;
- signing;
- result cache;
- telemetry;
- remote upload;
- ranking;
- official GA4GH certification claims.

B13 remains closed/deferred. Schema remains `helix-verification-v1`.

---

## After this baseline

Do not start the next engineering gate because this file exists. Update the product page if the operator path drifted, review [HELIX_ROADMAP.md](HELIX_ROADMAP.md), then decide the next capability from actual product need.
