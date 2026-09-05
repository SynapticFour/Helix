# Helix Standards Registry

**Status:** Implemented as provenance metadata. File: [`standards/registry.yaml`](../standards/registry.yaml). Schema: [`schemas/helix-standard-version-v1.json`](../schemas/helix-standard-version-v1.json). CLI: `helix standards list|show|validate`.

`helix verify` does load this registry for `--standard` / `--version` / `--all-supported-versions` and must not fetch GA4GH files at runtime. Default `helix verify TARGET` does **not** select a pack. **DRS 1.4.0 is SUPPORTED** for technical verification within the declared coverage boundary (`src/standards/support.rs` is the source of truth, not YAML alone). DRS 1.5.0 and WES 1.1.0 are **AVAILABLE**. A GitHub tag alone does not make a version supported. SUPPORTED is not VERIFIED.

Helix is HelixTest becoming a standalone VERIFY CLI. The registry exists so a Helix result can name an **authoritative GA4GH specification release**, not a moving GitHub branch and not a Ferrum implementation. It is **not** HELIOS (no signatures, RO-Crate, PDF, audit trail). It is **not** GA4GH certification. SUPPORTED without provenance, development SUPPORTED, and HEAD/`main` pins are rejected by schema + `src/standards/validate.rs` ([ARCHITECTURE_GUARDRAILS.md](ARCHITECTURE_GUARDRAILS.md)).

Baseline: [HELIX_CHECKPOINT_AUDIT.md](HELIX_CHECKPOINT_AUDIT.md) Parts 4–9. Execution modes: [STANDARD_VERSIONING.md](STANDARD_VERSIONING.md). Trust: [TRUST.md](TRUST.md) (fail closed; no silent substitution). HelixTest remains the execution engine ([DECISIONS.md](DECISIONS.md) D1). Helix owns the **claim**, which must be justified from recorded fields.

---

## 1. Why this exists

Today a green `helix verify` cannot honestly say “Verified against GA4GH DRS 1.4.0” or “1.5.0”. Default verify is unversioned and uses HelixTest’s bundled OpenAPI. DRS 1.4.0 is SUPPORTED (executable pack + SpecSource + HelixTest binding + non-empty catalog + declared coverage). Mode 1 for 1.4.0 can SELECT and run checks; `verified_version` stays empty. Mode 1 for 1.5.0 is still `AVAILABLE_BUT_NOT_SUPPORTED`. Fixture extras remain distinguishable from the one normative schema check.

The registry is the missing layer between:

- GA4GH’s published specification artifacts, and
- Helix check ids (`drs.object.schema`, …).

Without a registry row that is **SUPPORTED**, Helix must not emit a GA4GH version in claim language.

---

## 2. What the registry is (and is not)

| Is | Is not |
|----|--------|
| Machine-readable pin of GA4GH spec **bytes** Helix is allowed to test against | A live fetch of `main` / `develop` / GitHub Pages |
| Helix-owned metadata in this repository | A Ferrum, Lab Kit, or ga4gh-infra catalog |
| Input to `--standard` / `--version` / `--all-supported-versions` | A licence to test every GitHub tag |
| The source of “which official release is this pack?” | HELIOS evidence that a pipeline ran |
| Separate from Helix test identity (`id` / `code`) | A replacement for [TEST_IDENTITY.md](TEST_IDENTITY.md) |

Authoritative source is **GA4GH** (the GitHub organization and the specification repositories GA4GH publishes). HelixTest, Ferrum, and this repo are **not** the standard.

---

## 3. Vocabulary

JSON uses lowercase enums. Prose uses the same words in capitals when naming classes and states.

### 3.1 Identity fields (every version record)

| Field | Meaning | Must not be |
|-------|---------|-------------|
| **standard** | Machine id, same family as Helix discovery (`drs`, `wes`, `tes`, `trs`, `htsget`, `beacon`) | A target product name, a profile (`ferrum`), a Git branch |
| **product** | Official GA4GH **product title** (e.g. `Data Repository Service`) | Ferrum, Helix, HelixTest, a hospital stack |
| **version** | Version string **as published by GA4GH** on that release (e.g. `1.4.0`) | Inferred from `/ga4gh/drs/v1`, “latest”, or Helix crate version |
| **release_class** | How GA4GH published it: `official` \| `ballot` \| `snapshot` \| `development` | Helix support_status |
| **support_status** | What Helix has done with the pin: `available` \| `supported` | A target result (`tested` / `verified`) |
| **repository** | Canonical HTTPS URL of the **specification** git repo (no `.git` suffix) | `SynapticFour/Helix`, `SynapticFour/HelixTest`, Ferrum |
| **release_ref** | Immutable-enough published ref: **release tag** or named GitHub Release | `HEAD`, `main`, `master`, `develop` except on class `development` |
| **commit** | Exact 40-character SHA of that repository at the pin | A moving branch tip recorded as “whatever CI fetched” |
| **normative source** | File(s) inside that commit that Helix treats as the spec (OpenAPI, JSON Schema, …) | HelixTest `framework/src/drs.rs`, fixture CWL, Ferrum utoipa |
| **source_url** | Exact URL from which those bytes were retrieved | A directory index, an unpinned Pages root |
| **retrieved_at** | UTC calendar date of that retrieval (`YYYY-MM-DD`) | Build time of Helix |
| **integrity** | SHA-256 of the **file bytes** Helix will use | GitHub “latest”, a signature (HELIOS), a hash of the Helix binary |

**product vs standard:** `standard` is the short id Helix already uses on checks (`service: drs`). `product` is the human GA4GH name so a reviewer can search GA4GH’s site. Neither field is the software under test.

**pack_id:** `ga4gh.{standard}.{version}` for a single official line (example: `ga4gh.drs.1.4.0`). If the same numeric version exists in more than one **release_class**, disambiguate the id (`ga4gh.drs.1.5.0-ballot`). Pack ids are stable once published in the registry; do not recycle them.

### 3.2 Release classes (GA4GH publication)

| Class | JSON | What it is | Default `helix verify` | `--all-supported-versions` |
|-------|------|------------|------------------------|----------------------------|
| **OFFICIAL** | `official` | A GA4GH **released** specification (GitHub Release / release tag that GA4GH presents as the version) | May run **only if** `support_status` is `supported` | **Included** if `supported` |
| **BALLOT** | `ballot` | A ballot / review draft GA4GH published as such | **Never** unless the operator names it | **Never** |
| **SNAPSHOT** | `snapshot` | Preview, “release candidate pages”, dated snapshot, `preview/release/…` artifacts that are not the git release tag | **Never** unless the operator names it | **Never** |
| **DEVELOPMENT** | `development` | `main` / `develop` / unreleased commits | **Never** automatically. Must not be `supported` | **Never** |

Default automated verification (CI, `make prove` fixture verify, future helix-action without extra flags) uses **OFFICIAL ∩ SUPPORTED** only.

Ballots and snapshots may become `supported` so an operator can request them **explicitly**. They still **must not** enter the “all supported versions” set.

Development pins may be recorded as `available` for forensics. They **must not** be marked `supported`. Helix must not treat a branch name as a release.

HelixTest’s TRS OpenAPI taken from **develop** ([checkpoint audit](HELIX_CHECKPOINT_AUDIT.md) Part 6) is the cautionary example: that is class `development`. Helix `verify` does not execute TRS today; it still must not be imported as an official pack later without a real release pin.

### 3.3 AVAILABLE / SUPPORTED / TESTED / VERIFIED

These four words are **not** interchangeable. The first two live on a **registry row**. The last two live on a **run or a sentence about a target**.

| Term | Where | Meaning |
|------|--------|---------|
| **AVAILABLE** | Registry `support_status` | Helix has completed steps 1–3: authoritative GA4GH source identified, exact release identified, **source pinned** (commit + per-file SHA-256 + retrieval date). Operators **cannot** select this pack for verification yet. |
| **SUPPORTED** | Registry `support_status` | Helix has completed steps 1–7. There is a documented test mapping, an implemented suite, and a green **deterministic fixture** prove for that pack. The pack may be selected. Default automation may use it **only** if `release_class` is `official`. |
| **TESTED** | Run | This verification run **executed** a named `supported` pack against a named target (or Helix’s own fixtures). Outcome may be pass, fail, skip, or error. TESTED does not mean the target passed. |
| **VERIFIED** | Claim language only | Allowed **sentence** about a target: Helix executed that pack, every check bound as `normative` for the pack passed, evidence in §11 is present, and selected pack = tested pack. **Still not** GA4GH certification. |

A GitHub tag with **no** Helix registry row is **unknown to Helix**. It is not AVAILABLE. It is not SUPPORTED.

A version is **not** SUPPORTED merely because:

- the tag exists on the GA4GH repository,
- HelixTest vendored a YAML whose `info.version` matches,
- discovery read `type.version` from a target,
- CI is green on today’s unversioned suite.

**TESTED** and **VERIFIED** must not appear as `support_status` on a registry row. Putting them there would mix “Helix has a pack” with “this hospital passed.”

---

## 4. How a standard version becomes SUPPORTED

Ordered. No skipping. Each step is a reviewed change in Helix (and HelixTest if the engine still compiles the schema).

| Step | Name | Done when |
|------|------|-----------|
| 1 | Authoritative source identified | `repository` is the GA4GH (or GA4GH-published) spec repo, not Ferrum/HelixTest. `standard` + `product` agree with that repo. |
| 2 | Exact release identified | `version` + `release_class` + `release_ref` point at one published artifact. Not “current docs”. Not URL path `/v1`. |
| 3 | Source pinned | `commit` is a 40-char SHA. Every normative file has `source_url`, `retrieved_at`, and SHA-256 of the **exact bytes** Helix will load. Fail closed on mismatch. |
| 4 | Test mapping documented | Every Helix `id` in the pack has a binding: `normative` (locator inside the pinned file) or `fixture` / `policy` (explicitly **not** a GA4GH MUST in that file). No silent extras. |
| 5 | Test suite implemented | Engine path exists (HelixTest `ga4gh_schemas` and/or Helix) and loads **those** bytes, not a second copy. |
| 6 | Deterministic fixtures pass | `make prove` / fixture verify is green **for this pack** with pinned fixtures ([FIXTURES.md](FIXTURES.md)). Network fetch of GA4GH is not required. |
| 7 | Marked SUPPORTED | `support_status` flips to `supported` in the same change that lands mapping + fixture proof. Changelog records the pack_id. |

After step 3 the row is **AVAILABLE**. After step 7 it is **SUPPORTED**.

Steps 4–6 may iterate on AVAILABLE rows. They must not advertise SUPPORTED in docs or JSON until step 7.

---

## 5. Pinning rules (never silent HEAD)

Helix must **never** replace a pinned standard definition with the current HEAD of a GA4GH repository.

1. **Runtime:** Helix does not download OpenAPI/JSON Schema from the network in order to verify. Missing vendor bytes or hash mismatch is an **error**, not a fetch.
2. **CI / prove:** Same. No `git clone` of GA4GH without a pinned SHA. No “update schemas” job that commits HEAD on a schedule.
3. **Pages URLs:** `ga4gh.github.io/.../openapi.yaml` without a release path is not a pin. A `preview/release/drs-1.4.0/` URL is at best class **SNAPSHOT** until the same bytes are tied to a **git commit** of the spec repository.
4. **Refresh:** Changing `commit`, a hash, or a source_url is a **reviewed PR**. New `retrieved_at`. Tests re-run. If the bytes change and the tests do not, that is still a Helix versioning event (§10).
5. **Branch refs:** For `official`, `ballot`, and `snapshot`, `release_ref` must not be `HEAD`, `main`, `master`, or `develop`.
6. **Two copies:** If Helix vendors files and HelixTest also vendors files, Helix’s registry hash is authoritative for Helix claims. Drift is a bug; do not “prefer whichever loaded first.”

---

## 6. Check bindings: claim taxonomy

A SUPPORTED pack lists `test_bindings` (schema: `testBinding`). Semantics: [TAXONOMY.md](TAXONOMY.md).

| `kind` | Meaning | May support the sentence “Verified against GA4GH {product} {version}”? |
|--------|---------|---------------------------------------------------------------------|
| `normative` | The assertion is located in the **pinned** normative file Helix **loaded** (OpenAPI `operationId`, `components.schemas.*`, JSON Pointer, quoted status code). | **Yes**, if that check passed |
| `guidance` | Official GA4GH implementation guidance, not a MUST. HelixTest extras (`supported_wes_versions` contains `1.0` **or** `1.1`) are **not** this — they are `fixture`. | **No** |
| `fixture` | Helix / HelixTest scenario (`test-object-1`, `trs://test-tool/echo/1.0`, Range probe, …). Useful. **Not** a numbered GA4GH MUST. | **No** (must be disclosed as fixture) |
| `interoperability` | Related to an API surface but not bound to vendor bytes Helix loaded. Must not automatically become a conformance claim. | **No** |
| `security` | Helix / HelixTest auth or Crypt4GH layout. Out of this registry for pack selection. Security PASS is not GA4GH conformance. | **No** |
| `benchmark` | `helix bench`. Out of this registry. Never a conformance claim. | **No** |

If a reviewer cannot find the locator in the pinned file, the binding **must not** be `normative`. That is **UNVERIFIED STANDARD PROVENANCE** until remapped. Result rows record `traceability`. Exactly one shipped check is `normative` (`drs.object.schema.openapi`).

Helix ids stay stable ([TEST_IDENTITY.md](TEST_IDENTITY.md)). Bindings point at ids; they do not rename them.

`helix security` and `helix bench` are **out of this registry**. They are Helix-owned behaviour and smoke measurement, not GA4GH packs.

---

## 7. Five version concepts (must stay separate)

These are different facts. The registry only stores **(5)**. Future verify JSON must not collapse them into one `version` field.

| # | Concept | Owner | Today ([audit](HELIX_CHECKPOINT_AUDIT.md) Part 8) | Future |
|---|---------|--------|---------------------------------------------------|--------|
| 1 | **Claimed** by the target | Target `service-info` / `type.version` | Sometimes copied in discovery; unused | Record as claimed; never select a pack from a claimed value that is not in the registry |
| 2 | **Detected** by Helix | Helix discovery | Same snapshot; **not** inferred from URL | Record as detected; empty if no 2xx JSON |
| 3 | **Selected** by the operator | CLI / CI | Does not exist (`--profile` only) | `--standard` / `--version` / `--release-class` resolving to one `pack_id` |
| 4 | **Tested** | The run | Implicit HelixTest pin + one compiled schema | The `pack_id` actually loaded. Must equal selected, or the run is an error |
| 5 | **Authoritative pack Helix has** | This registry | Implicit, unhashed HelixTest YAML | Rows with `supported` |

**Fail closed:** if selected ≠ tested, do not emit a versioned claim. If detected is present and ≠ the tested pack version, do not stay silent (warn at minimum; a VERIFIED sentence requires the mismatch to be explicit in the report).

Do **not** infer a standard version from `/ga4gh/wes/v1`.

---

## 8. Modes A / B / C

Implemented as fail-closed selection on `helix verify`. Default `helix verify TARGET` is **not** Mode B labelled as a pack.

**Mode A** — `helix verify TARGET --standard drs --version 1.4.0`

Resolve `standard` + `version` (+ optional `--release-class`, default `official`) to **exactly one** `supported` row. If zero or many, **error**. Do not fall back to another version. AVAILABLE → `AVAILABLE_BUT_NOT_SUPPORTED`.

**Mode B** — `helix verify TARGET` (auto-detect)

**Not shipped as a labelled pack.** Default verify does not select a registry row. If Mode B is implemented later: use claimed/detected version **only** to look up a registry row. If missing, ambiguous, or not `supported`+`official` for default automation → **error** or “no pack”, not a silent other suite. Empty detection is not a licence to pick “latest.”

**Mode C** — `--all-supported-versions`

Run **every** row matching:

```text
support_status == supported
AND release_class == official
AND (optional) standard filter
```

**Exclude** ballot, snapshot, and development even if someone marked a ballot `supported`.

One `VerificationRun` today cannot honestly hold Mode C. Architecture needed: a list of versioned runs or a parent document that references several `pack_id`s. Do not overload `helix-verification-v1` with HELIOS fields to fake that.

**Default verify after the first OFFICIAL SUPPORTED pack exists:** name the pack in the report. If more than one OFFICIAL SUPPORTED pack exists for a detected service and Mode B cannot match exactly one, **require Mode A** rather than sorting versions and taking the newest.

Until any row is SUPPORTED, default verify may keep today’s unversioned HelixTest wrap but **must not** print a GA4GH version as tested.

---

## 9. On-disk layout

```text
standards/
  registry.yaml
  vendor/
    ga4gh.drs.1.4.0/openapi/   # complete git openapi/ tree at release_commit
    ga4gh.drs.1.5.0/openapi/
    ga4gh.wes.1.1.0/workflow_execution_service.openapi.yaml
schemas/
  helix-standard-version-v1.json
src/standards/
```

`helix standards validate` checks each `vendor_path` SHA-256 against `integrity.hex`, and for DRS packs with `schema_entry` also the `sha256-manifest-v1` pack digest and local SpecSource resolve. Mismatch is an error. Helix does not download a replacement.

DRS 1.4.0 and 1.5.0 vendor the complete `openapi/` tree from the pinned `release_commit`. The GitHub Pages bundled OpenAPI is not this pack. The `DrsObject` schema-entry closure is local (`openapi/components/schemas/{DrsObject,Checksum,AccessMethod,AccessURL,Authorizations,ContentsObject}.yaml`). WES 1.1.0 still has an HTTPS `$ref` to `ga4gh-service-info` and is **not** executable as a SpecSource.

### 9.1 What is committed

| Artifact | In Helix |
|----------|----------|
| Registry metadata (`registry.yaml`) | Yes |
| Integrity hashes | Yes, per source file plus DRS `pack_integrity` |
| Vendored DRS `openapi/` trees | Yes, hashed |
| WES entry OpenAPI | Yes, hashed; not a complete executable SpecSource |
| HELIOS signatures / RO-Crate | No |

### 9.2 CLI

```text
helix standards list
helix standards list --supported-only
helix standards show drs 1.5.0
helix standards validate
```

`list` prints every row’s provenance and the default discovery set (**OFFICIAL ∩ SUPPORTED**, currently `ga4gh.drs.1.4.0`). `--supported-only` prints only that set. Ballot/snapshot are never in it. YAML `supported` without the executable gate is rejected.

`show` is an **exact** `(standard, version)` match. Unknown versions exit 1, set `substituted: false`, and may list other rows as **not selected**. Showing DRS 1.5.0 does not return 1.4.0.

`validate` uses the frozen JSON Schema plus Helix rules (GA4GH repo allowlist, commit in `source_url`, vendor hash). No network.

Optional `--registry PATH` (and a positional path on `validate`) is for tests. Default is this crate’s `standards/registry.yaml`.

Helix owns the registry (Helix owns the claim). Do not put it in Ferrum. HelixTest README URLs are not a pin.

Duplicate `pack_id` values are invalid. Duplicate `(standard, version, release_class)` tuples are invalid.

---

## 10. Reviewing an update; Helix versioning

### 10.1 Who reviews normative mappings

There is **no** GA4GH-appointed mapping board and no external certification panel in this repository.

Until a pack is SUPPORTED, **no shipped check may be `normative`**. A change that would mark a check `normative` or a row `supported` must:

1. Land as a pull request against this repo (and HelixTest if the engine still compiles the schema).
2. Include the artefacts in the list below (registry diff, vendor bytes, bindings, fixture prove, changelog).
3. Keep the tests in [TRACEABILITY.md](TRACEABILITY.md) §6 red if the chain is incomplete.

**Who may merge that PR today:** the single repository steward ([IDENTITY.md](IDENTITY.md)). That is an organisational fact, not evidence that the mapping is correct. A skeptical reviewer rejects the mapping by showing a missing locator, a hash mismatch, HelixTest still loading different bytes, or a fixture extra left inside a `normative` row.

Do not treat a steward merge as GA4GH approval.

A PR that adds or changes a pack must include:

1. Registry diff (source, ref, commit, hashes, `retrieved_at`).
2. Vendor file bytes if the hash changed.
3. Test-binding diff (new normative locators or explicit `fixture`/`policy`).
4. Fixture prove for that pack.
5. Changelog: pack_id and whether claim language changes.

| Change | Helix crate version |
|--------|---------------------|
| New OFFICIAL **SUPPORTED** pack (new tests or new standard version) | **Minor** |
| Hash / commit refresh, same bindings and same Helix `id`s | **Patch** (still a reviewed pin change) |
| Assigned Helix `id` / `code` change | **Compatibility break** ([TEST_IDENTITY.md](TEST_IDENTITY.md)) — not a silent pack bump |
| Fixture catalog bump (`helix-fixtures-v1` → v2) | Independent of pack_id; record both on the run |
| Ballot/snapshot pack added | Minor if `supported`; must not change default verify |

HelixTest pin bumps ([VERSIONS.lock](../VERSIONS.lock)) are a **different** pin from the GA4GH `release_commit`. The versioned DRS path compiles registry vendor bytes via `*_with_spec`; default unversioned verify still uses HelixTest’s bundled OpenAPI. A HelixTest bump that changes bundled YAML does not silently become a registry-pack execution.

---

## 11. Evidence required to say “Verified against GA4GH DRS 1.4.0”

That sentence is **forbidden** until all of the following exist on the **run** (names illustrative; not added to `helix-verification-v1` in this change):

| Evidence | Role |
|----------|------|
| `pack_id` e.g. `ga4gh.drs.1.4.0` | Which authoritative pack |
| Registry `version` + `release_class: official` | Not a ballot/snapshot/develop |
| `repository` + `release_ref` + `commit` | Exact GA4GH git pin |
| SHA-256 of each normative file used | Bytes, not a URL |
| Helix version | Tool |
| HelixTest tag + SHA (if engine) | Engine pin |
| Helix `id`s executed, with `kind=normative` all `pass` | What “verified” covers |
| Fixture `kind` results disclosed separately | Must not be implied by the sentence |
| `target.url` | What was tested |
| `timestamp` | Wall clock, **not** a signature |
| `fixture_version` | Helix fixture catalog |
| `schema_version` of the report | Helix JSON contract |
| selected pack_id = tested pack_id | No silent substitution |
| detected/claimed versions recorded | Honesty if they differ |

**DRS 1.5.0:** the version is AVAILABLE in the registry (tag pinned). It is **not** SUPPORTED. The sentence “Verified against GA4GH DRS 1.5.0” remains false.

Signing / RO-Crate / PDF are **not** required for this sentence; they are HELIOS ([HELIX_VS_HELIOS.md](HELIX_VS_HELIOS.md)).

---

## 12. Current tree vs this model

| Pack | Status |
|------|--------|
| DRS 1.4.0 | **AVAILABLE**, official, tag `drs-1.4.0`, commit `36145d389e0a454428d1dac5c4a30870995fdd7c`. Not SUPPORTED. |
| DRS 1.5.0 | **AVAILABLE**, official, tag `drs-1.5.0`. The tag exists; Helix still does not treat it as SUPPORTED. |
| WES 1.1.0 | **AVAILABLE**, official, tag `1.1.0`. Not SUPPORTED. |
| Default discovery (OFFICIAL ∩ SUPPORTED) | **Empty** |
| `helix verify` | Default: unversioned HelixTest wrap. `--standard drs --version 1.4.0` can SELECT the supported pack. Mode 1 for 1.5.0 is `AVAILABLE_BUT_NOT_SUPPORTED`. SUPPORTED is not VERIFIED. |

Honest language remains: HelixTest pin + documented fixtures. Do not say “Verified against GA4GH DRS 1.5.0.”

HelixTest’s own vendored YAML remains unpinned for `helix verify`. TES/TRS/htsget/Beacon YAML in HelixTest is unused by verify. TRS-from-develop must not be imported as OFFICIAL.

Next (not this change): test mapping, pack prove, then mark SUPPORTED.

---

## 13. What this change does not do

- Does not mark any pack SUPPORTED (steps 4–7 incomplete).
- Does not duplicate HELIOS.
- Does not fetch GA4GH at runtime.
- Does not couple packs to Ferrum. `--profile ferrum` remains a **target policy** name, not a registry product.
- Does not automatically test every GA4GH release.
