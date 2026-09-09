# Operator DRS 1.4.0 verification workflow

This is the **canonical first-usable Helix workflow**. Helix is HelixTest becoming a standalone VERIFY CLI. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Product picture: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Claims: [CLAIMS.md](CLAIMS.md). Coverage: [COVERAGE.md](COVERAGE.md). Install: [INSTALL.md](INSTALL.md).

Helix is built **from source**. There is no crates.io crate, Homebrew formula, container image, or GitHub release binary.

---

## Canonical workflow

```text
1. Identify supported DRS version
        ↓
2. Identify the target
        ↓
3. Configure the DRS fixture (object id)
        ↓
4. Run versioned verification
        ↓
5. Save evidence (--output)
        ↓
6. Inspect evidence (helix inspect)
        ↓
7. Interpret PASS vs VERIFIED vs unevaluated
```

### Step 1 — Supported version

```bash
helix --version
helix standards list --supported-only
```

`--version` prints the Helix package version, compile-time Helix git SHA, HelixTest **tag lineage** (`v0.1.3`), exact HelixTest **source SHA** (`HELIXTEST_SHA`), and executed checker id. Lineage is not the compile commit. Those lines are **not** the GA4GH DRS version.

Today the only SUPPORTED pack is **ga4gh.drs.1.4.0**. SUPPORTED is not VERIFIED. Default `helix verify URL` does **not** select this pack.

### Step 2 — Target

Point Helix at an HTTP origin **you** started. Operator labels are untrusted:

```text
--target-id
--target-kind
--implementation-name
--implementation-version
```

`--implementation-version` never becomes `verified_version`.

No live DRS? Use the in-process fixture (not independent evidence):

```bash
make verify-drs
```

That runs DRS 1.4.0, writes `verify.json`, and prints `helix inspect`. Override the path with `HELIX_VERIFY_JSON`.

### Step 3 — Fixture

DRS test input (not a GA4GH MUST):

```text
--drs-object-id
--drs-object-sha256
```

Default object id is `test-object-1` (catalog). Independent implementations use their own object id. A 404 on the configured id is `fixture_unavailable` (SKIP), not DRS non-conformance.

### Step 4–6 — Verify, save, inspect

Against a DRS **you** started:

```bash
NO_COLOR=1 helix verify URL \
  --standard drs --version 1.4.0 \
  --target-id YOUR-ID \
  --target-kind real-independent-local-implementation \
  --drs-object-id YOUR-OBJECT-ID \
  --output verify.json
helix inspect verify.json
```

`--output` writes `helix-verification-v1` and does not replace `--format` (default is human text). `helix inspect` does **not** rewrite the file.

### Step 7 — Interpret

| Word | Means | Does not mean |
|------|--------|----------------|
| PASS / FAIL / SKIP / ERROR | One check outcome | The target is GA4GH-certified |
| VERIFIED / NOT_VERIFIED | Derived claim | Every DRS operation was tested |
| selected | Pack Helix chose | The target declared that version |
| detected | What the service advertised | Selected or verified |
| verified_version | Claim output when predicates hold | Full-standard compliance |
| coverage.state = partial | Required catalog rows passed; operations remain unevaluated | Complete DRS |
| Exit 0 | ≥1 PASS and no FAIL/ERROR | `ga4gh_requirement` VERIFIED |
| current_verifier_evidence | Artifact cites **this** binary’s git SHA | Historical files are worthless |
| historical_observation | Inspectable; not this build | Forged VERIFIED |

Authorization is **unevaluated**. Helix does not send credentials.

---

## Evidence retention

Recommended practice:

```text
helix verify … --output verify.json
    ↓
keep the human stdout if you need it in a ticket
    ↓
helix inspect verify.json   (now or later)
```

The JSON is meaningful because it records the target, selected standard/version, checker, checks, claims, coverage, and Helix/HelixTest provenance. `helix inspect` recomputes claims, coverage, and standing. It does not rewrite the file. It does not sign the file. Signing is HELIOS.

**Current verifier evidence** is a file this Helix binary just produced and that still matches this binary’s git identity. **Historical observation** is older evidence that remains inspectable. Do not restamp `helix_git_sha` to make history look current. Do not copy `local/b12/` into a “current” directory.

---

## Troubleshooting

| What you see | What it means |
|--------------|----------------|
| `AVAILABLE_BUT_NOT_SUPPORTED` / DRS 1.5.0 | That version is pinned but not a Helix verification pack. Helix did **not** substitute 1.4.0. |
| `target unreachable` / ERROR rows | The origin was not reachable. That is not VERIFIED. |
| SKIP `fixture_unavailable` | The configured object id was not a usable blob. SKIP is not PASS. |
| DRS `NOT_DETECTED` after a missing `--drs-object-id` | Discovery may probe that object. Advertise DRS `service-info` (or keep a reachable object) so DRS stays DETECTED; then the 404 is `fixture_unavailable`, not “unsupported test”. |
| FAIL `attribution: target_failure` | The check failed on the implementation. |
| Exit 0 and `NOT_VERIFIED` | Checks passed; the DRS 1.4.0 claim did not. Default unversioned verify does this. |
| `helix inspect` standing `invalid` | The file is not internally consistent. Do not treat it as evidence. |
| Missing HelixTest | Clone the sibling at the SHA in [VERSIONS.lock](../VERSIONS.lock). [INSTALL.md](INSTALL.md). |
| `make prove` needs crates | `make fetch` once (crates.io at lockfile checksums, not GA4GH). |

---

## What Helix does not do

- Official GA4GH certification
- Authorization / authentication evidence (B13 deferred)
- Signing, RO-Crate, PDF (HELIOS)
- Result cache, telemetry, remote upload
- Ranking implementations
- A published binary or container (source build only)

---

## Two independent DRS implementations

The in-process fixture is not independent evidence. To apply the same DRS 1.4.0 contract to GA4GH Starter Kit DRS and Bento DRS:

[INDEPENDENT_DRS.md](INDEPENDENT_DRS.md)

```text
start each target
    ↓
helix verify URL --standard drs --version 1.4.0 --output FILE
    ↓
helix inspect FILE
    ↓
helix differential starter-kit.json bento.json
```

Helix does not rank them. A check difference is an observed behavioural difference, not a quality ranking. VERIFIED and NOT_VERIFIED describe claim state, not implementation quality. Historical `local/b12/` JSON is not current evidence. `make prove` does not start those stacks.
