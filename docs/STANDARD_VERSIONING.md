# Helix standard-version execution model

**Status:** Implemented on `helix verify` for Mode 1 (`--standard` + `--version`) and Mode 3 (`--all-supported-versions`). Default `helix verify TARGET` remains the **unversioned** HelixTest wrap: it records detected service-info versions and does **not** select or verify a registry pack. `--standard drs` without `--version` is Mode 2 and fail-closes as `INSUFFICIENT` (OfficialSupported is not empty, but Helix will not guess 1.4.0).

Helix is HelixTest becoming a standalone VERIFY CLI. Results are a technical signal, not GA4GH certification. HELIOS (`helios-audit`) stays out of this model (no signatures, RO-Crate, PDF). Ferrum is a reference target, not a version source. Trust constraint: [TRUST.md](TRUST.md) (no silent substitution, no `/v1` inference, fail closed when a pack is not SUPPORTED).

Mode 1 for DRS 1.5.0 returns **`AVAILABLE_BUT_NOT_SUPPORTED`**: the version is AVAILABLE (pinned tag) but not SUPPORTED. Helix does not silently downgrade or substitute 1.4.0.

Cross-walk: Mode 1 = registry Mode A; Mode 2 = Mode B; Mode 3 = Mode C.

---

## 1. Four version facts (every report)

The report **must** distinguish these four. They must not collapse into one `version` field. A fifth fact — which packs Helix **has** — lives in the registry, not on the target.

| Report name | Meaning | Empty when |
|-------------|---------|------------|
| **declared** | What the target **asserts** in documented service-info fields about the **standard** | Those fields are absent, unreadable, or not 2xx JSON |
| **detected** | The version identifier Helix is willing to treat as observed, **only** from sufficient evidence | Evidence is insufficient (this is required honesty) |
| **selected** | The GA4GH release Helix **chose** for the attempted versioned execution | Selection failed (Mode 2 insufficient, unknown version, AVAILABLE-only, …) |
| **verified** | A version claim permitted by the complete claim predicates (join **and** SUPPORTED / normative / VERIFIED gates) | Predicates do not hold (B2: always empty) |

`selected_version` without `verified_version` is valid: Helix may select a pack, execute the join, and still emit no version claim. `verified_version` without `selected_version` is invalid. If both are set they must be equal **and** the join hashes must be present.

Join success (`pack_integrity_sha256`, `schema_document_sha256`, `schema_component_sha256`, `execution_id`) is **not** `verified_version`. It is **not** SUPPORTED, normative verification, or a VERIFIED claim.

**Verified** on the report is the version sentence allowed by [CLAIMS.md](CLAIMS.md). It does **not** by itself allow the English sentence “Verified against GA4GH DRS x.y.z”. That sentence still requires all `kind=normative` bindings for that pack to pass, plus the evidence table in the registry doc §11.

Do not use “detected” as a synonym for “we guessed from the URL.”

### 1.1 Where declared values come from

Copy **only** from a **2xx** JSON service-info body Helix already fetches in discovery ([DISCOVERY.md](DISCOVERY.md)). Do not invent fields.

| Source field | Role |
|--------------|------|
| `type.version` | Primary **declared standard version** (string). |
| `supported_wes_versions` (WES) | Declared **list** of WES protocol versions the target claims to speak. |
| `type.artifact` | Must match the Helix `standard` when present (`drs`, `wes`, …). Mismatch is a conflict, not a version. |
| `version` (service-info root) | **Implementation** version (product build). Record separately as `implementation_version`. **Never** use it to select a GA4GH pack. |

A target may declare **one** string (`type.version`) and/or a **list** (WES). The report must show the raw values. Normalization to a registry `version` string is a later step (§4) and may fail.

### 1.2 What counts as detection (sufficient evidence)

**Detected** is set only when **all** of the following hold:

1. The service is DETECTED.
2. Helix obtained **2xx** service-info JSON (not 401/403-only presence).
3. At least one standard-version field in §1.1 is present and is a non-empty string or a non-empty array of strings.
4. Helix did **not** infer the value from the path (`/ga4gh/drs/v1` is not `1.0` or `1.4.0`). That inference is already forbidden in code (`does_not_invent_version_from_url`).

If those fail, `detected` is omitted and `detection_status` is `insufficient`. Helix **must not** claim version detection in text or JSON.

**Detected is not behavioral fingerprinting.** Helix must not select DRS 1.5.0 because a response body contains a 1.5-only property. That would choose the suite after peeking at the answer. Feature overlap is Mode 3’s problem, not Mode 2’s.

When the only usable field is `type.version` and it is a single string, **declared** and **detected** are the same string. They still both appear. They diverge when:

- the body has a list (`supported_wes_versions`) and a different `type.version`;
- `type.artifact` disagrees with the discovered service;
- declared fields conflict with each other.

Conflicting evidence → `detection_status: conflict`. Do not pick a pack in Mode 2.

### 1.3 JSON (`helix-verification-v1`)

Every check row records these fields (null when empty). `selected_version` is Helix’s choice; `detected_version` is copied from 2xx service-info `type.version` only. They must not collapse. Run-level `standard_selection` repeats the same facts.

```json
{
  "standard": "drs",
  "requested_version": "1.5.0",
  "detected_version": "1.2.0",
  "selected_version": null,
  "verified_version": null,
  "standards_registry_entry": "ga4gh.drs.1.5.0",
  "standards_source_commit": "fe25c3953ae3398a31054d3f9f040d5e27aad517"
}
```

A looked-up AVAILABLE row may fill `standards_registry_entry` without filling `selected_version`. That is not a claim the target declared 1.5.0.

---

## 2. Registry sets used by execution

Let **S(standard)** be Helix registry rows with that `standard`.

| Set | Definition | Used by |
|-----|------------|---------|
| **OfficialSupported** | `support_status=supported` ∧ `release_class=official` | Mode 2 default, Mode 3 |
| **ExplicitSupported** | `supported` ∧ operator `--release-class` (default `official`) | Mode 1 |
| **AvailableOnly** | `support_status=available` | Never executed |

Ballot and snapshot: Mode 1 only, and only with `--release-class`. They are **never** in Mode 2 or Mode 3.

Development: **never** selected. Mode 1 must reject `development`.

A GitHub tag with no row is **unknown**. Unknown is not AvailableOnly.

---

## 3. Mode 1 — Explicit

```text
helix verify TARGET --standard drs --version 1.5.0
```

**Meaning:** Test this target against the Helix pack for **GA4GH DRS 1.5.0**, if that pack is **SUPPORTED**.

This is an operator instruction. It is **not** “the target declared 1.5.0.”

### 3.1 Resolution

1. `--standard` and `--version` are required together. `--version` without `--standard` is usage (exit 2).
2. Optional `--release-class` (default `official`). `development` → error.
3. Resolve to **exactly one** ExplicitSupported row with that `standard`, `version`, and class.
4. Load **those** pinned bytes. `selected.pack_id` = that row. Run that suite only for that standard.
5. After a successful execution join, record join hashes. `verified_version` stays empty until claim predicates permit a version sentence (B2: always empty).
6. Other services: discover as today; **do not** execute their packs unless also named. Unselected TESTABLE services are skipped with reason `standard_not_selected`.

### 3.2 Fail closed (no substitution)

| Condition | Behavior |
|-----------|----------|
| No registry row | Error: unknown to Helix. Do not run “closest” pack. Do not fetch GA4GH HEAD. |
| Row is AVAILABLE, not SUPPORTED | Error: pinned but no Helix pack. |
| Row is SUPPORTED ballot/snapshot but class omitted | Error: default class is official; operator must pass `--release-class`. |
| Hash mismatch / missing vendor file | Error: runner, not a target fail. |
| Target undeclared or declares another version | **Still run** the requested pack. Record declared/detected. Set `mismatch` (e.g. `declared_ne_selected`). Honest result language (§7). |

`--version 1.5.0` when Helix has only 1.4.0 SUPPORTED: **error** (`AVAILABLE_BUT_NOT_SUPPORTED` or `UNKNOWN_TO_HELIX`). Do not fall back to another version.

### 3.3 Claim language

Allowed if normative checks for **that pack** passed:

> Helix verification checks for GA4GH DRS 1.5.0 passed against this target.

Forbidden unless declared/detected also equal 1.5.0:

> This target claims GA4GH DRS 1.5.0.

---

## 4. Mode 2 — Automatic

**Not shipped.** Default `helix verify TARGET` does **not** run this algorithm. It stays the unversioned HelixTest wrap (`selected_version` / `verified_version` empty). `--standard drs` without `--version` fail-closes as `INSUFFICIENT` (OfficialSupported is DRS 1.4.0; Helix will not guess it). The rest of this section is the required algorithm **if** Mode 2 auto-detect is implemented later. Do not read it as current CLI behaviour.

```text
helix verify TARGET
```

**Meaning:** For each TESTABLE service, detect what the target **declares or exposes with sufficient evidence**, then verify the **most appropriate OfficialSupported** pack.

`--standard drs` without `--version` restricts Mode 2 to DRS (other services discovered only).

**Most appropriate** means: the unique OfficialSupported pack that **matches the target’s evidence**. It does **not** mean Helix’s newest pack, “latest GA4GH”, or a silent upgrade (today’s 1.2.0 discovery vs 1.4.0 schema is the credibility gap this mode exists to close).

### 4.1 Selection algorithm (per TESTABLE service)

Inputs: declared fields, `detection_status`, OfficialSupported for that `standard`.

**Normalize** a declared string to a registry `version` only by **exact match** against OfficialSupported `version` values (and AvailableOnly, for a better error). Do not coerce `1.2` → `1.2.0` unless the registry lists both as the same pack (it must not: one version string per pack). Do not treat `1.0` in `supported_wes_versions` as WES OpenAPI `1.1.0` without an explicit registry alias. **v1 has no aliases.** If GA4GH’s field uses `1.1` and the pack version is `1.1.0`, the registry row must record that correspondence in `notes` **and** a future dedicated `declared_forms` list; until that exists, `1.1` ≠ `1.1.0` and Mode 2 fails closed rather than guessing.

Then:

```text
if detection_status in {insufficient, conflict}:
    do not select
    do not run a versioned pack
    do not set detected or verified
    emit selection_status = detection_status
    stop   # fail closed for this service

if exactly one normalized version V from declared/detected:
    if V in OfficialSupported:
        select that pack
        reason = exact_match
    else if V in AvailableOnly:
        selection_status = available_not_supported
        do not select another version
    else:
        selection_status = unknown_to_helix
        do not select

if declared is a list L (e.g. supported_wes_versions), after normalization:
    I = L ∩ OfficialSupported versions
    if I is empty: same as unknown / available_not_supported for those members
    if |I| == 1: select that pack; reason = unique_intersection
    if |I| > 1:
        select the maximum V in I by GA4GH published order recorded in the registry
        reason = highest_supported_intersection
        record the full declared list
        record not_tested = I \ {selected}
        # This is still one pack. It is not Mode 3.

if no versioned pack selected:
    skip that service’s checks with reason selection_failed
    skip is never pass
```

**Published order:** the registry must later expose a total order among OfficialSupported versions of one standard (recommended: an explicit `order` integer on each row, not ad-hoc semver parsing). Until that field exists, Mode 2 **must not** implement “highest” by guessing semver; multiple intersection members then require Mode 1 or Mode 3 (`selection_status: ambiguous`). Prefer adding `order` with the first multi-version packs rather than sorting strings.

### 4.2 What Mode 2 must not do

- Infer version from URL path or from `type.group`.
- Select OfficialSupported **max** when evidence is insufficient.
- Select 1.4.0 because the target declared 1.2.0 and 1.4.0 is “compatible.” Compatibility is Mode 3 or explicit Mode 1.
- Select a ballot/snapshot/development pack.
- Run two packs for one service (that is Mode 3).
- Claim detection when `detection_status` is `insufficient`.

### 4.3 Several TESTABLE services

DRS and WES each get their own `standard_versions[]` entry and their own pack. One `helix verify` may therefore TEST two packs (e.g. DRS 1.4.0 and WES 1.1.0) if both selections succeed. That is not Mode 3: Mode 3 is **one standard, every OfficialSupported version**.

If DRS selection succeeds and WES selection fails, DRS may still run; WES checks are skipped (`selection_failed`). Overall exit follows [CLI_CONTRACT.md](CLI_CONTRACT.md): skip-only is not a pass; a DRS-only pass under `generic` may still exit 0.

---

## 5. Mode 3 — Compatibility

```text
helix verify TARGET --standard drs --all-supported-versions
```

**Meaning:** Run the Helix suite for **every OfficialSupported** DRS pack, **independently**, against the same target.

`--all-supported-versions` **requires** `--standard`. It **conflicts** with `--version`. It **ignores** declared/detected for **selection** (those fields are still **recorded** on every child run). Ballot/snapshot/development are excluded even if `supported`.

### 5.1 Execution

1. List OfficialSupported packs for that standard, in registry `order`.
2. If the list is empty → error (Helix has no official supported pack).
3. For each pack: load that pin, run that suite, set selected = verified = that `pack_id`.
4. Packs must not share compiled schema state. A 1.5.0 OpenAPI must not leak into a 1.4.0 run (today HelixTest uses one `OnceCell` schema — that is an implementation blocker, not a licence to merge versions).
5. Fixture-kind and policy-kind bindings stay labeled. A fixture fail in 1.4.0 does not skip the 1.5.0 pack.

### 5.2 Output honesty (required)

Passing two packs means:

> Helix verification checks for GA4GH DRS 1.4.0 passed.
> Helix verification checks for GA4GH DRS 1.5.0 passed.

It does **not** mean:

> The target officially claims DRS 1.4.0 and 1.5.0.

If the target declared only `1.4.0`, Mode 3 may still run 1.5.0. Every child run must show `declared` vs `selected` and `mismatch` when they differ. Summary text must include: **compatibility matrix, not a claim set.**

### 5.3 Document shape

One `VerificationRun` cannot honestly hold Mode 3 ([STANDARDS_REGISTRY.md](STANDARDS_REGISTRY.md) §8). Proposed parent (new schema when implemented):

```text
helix-verification-set-v1
  mode: compatibility
  target, helix_version, timestamp
  standard: drs
  declared / detected  (once, from discovery)
  runs[]:  each a VerificationRun for one pack_id
  summary: per pack pass|fail|error|skip
```

No HELIOS fields. `helix compare` compares two runs of the **same** `pack_id`; comparing a 1.4.0 run to a 1.5.0 run is `same_measurement: false` (catalog/pack change), not `NEW_FAIL` by itself.

### 5.4 Exit

Print the set, then exit **1** if any child has fail or error, or if every child is skip. Exit **0** only if at least one child has a pass and none have fail/error. That matches today’s verify spirit. A 1.4.0 pass + 1.5.0 fail is exit 1 and a matrix, not “the target is 1.4.0 certified.”

---

## 6. Behavior matrix (required cases)

| Situation | Mode 1 | Mode 2 | Mode 3 |
|-----------|--------|--------|--------|
| **Target declares no version** (no `type.version`, no list; or only 401/403) | Run requested pack. `declared`/`detected` empty. `detection_status=insufficient`. Mismatch recorded. | **No pack.** Do not claim detection. Skip checks (`selection_failed`). Exit per skip-only rules. Tell operator to use Mode 1. | Run all OfficialSupported. Record insufficient declaration on every child. Matrix ≠ claims. |
| **Multiple versions possible** (WES list; or `type.version` plus a list) | Operator names one. List still recorded as declared. | Unique intersection → that pack. Several OfficialSupported in the list → `highest_supported_intersection` **only if** registry `order` exists; else `ambiguous` (require Mode 1 or 3). | Run every OfficialSupported, not only the intersection. |
| **Versions overlap** (1.4 and 1.5 share endpoints; minor “compatible” marketing) | One pack, no subset assumption. | One pack from evidence. Do not assume 1.5 ⊃ 1.4. | Independent suites. Pass/fail per pack. Overlap is why Mode 3 exists. |
| **Target implements an older version** (declares 1.2.0; Helix OfficialSupported is 1.4.0 only) | `--version 1.4.0` allowed; mismatch explicit; no “target is 1.4.0.” `--version 1.2.0` errors if not SUPPORTED. | **Do not** select 1.4.0. `unknown_to_helix` or `available_not_supported`. | Runs 1.4.0 (and any other OfficialSupported). Child shows declared 1.2.0 ≠ selected 1.4.0. |
| **AVAILABLE but not SUPPORTED** | Error: no pack. | Error/skip for that version; **no fallback**. | Omit that version (Mode 3 is OfficialSupported only). Mention in notes if the declared version is AvailableOnly. |
| **SUPPORTED but required fixtures unavailable** | See §6.1. | Same. | Same, per pack. |
| **Breaking changes between GA4GH versions** | Operator opts into one side of the break. | Match declared version only. A break does not auto-select the newer pack. | Both packs run. Bindings/citations are per pack. Shared Helix `id` only if the assertion is the same; otherwise new ids ([TEST_IDENTITY.md](TEST_IDENTITY.md)). |
| **Deprecated version** | See §6.2. | Prefer non-deprecated exact match; if the only match is deprecated, select it and set `reason=deprecated_exact_match`. Never auto-upgrade to a current pack. | Include deprecated OfficialSupported packs (they are still official releases). Label `deprecated` on the child. |

### 6.1 Fixtures unavailable

Distinguish **Helix** fixtures from **target** fixtures.

| Missing | Status | Pack TESTED? | “Verified against” sentence |
|---------|--------|--------------|-----------------------------|
| Helix vendor spec bytes or hash | `error` (runner). Pack not loaded. `verified` empty. | No | Forbidden |
| Helix fixture catalog required by a SUPPORTED pack (Helix bug: SUPPORTED implies step 6) | `error` | No | Forbidden |
| Target lacks `test-object-1` / echo TRS / scatter | Existing fixture contract: those **`kind=fixture`** rows fail or skip. **`kind=normative`** still run if they do not need that object. | Yes, if the pack loaded | Only if **all normative** rows passed. Fixture fails must be disclosed and do not support the sentence by themselves |

Skip is never pass. A pack that cannot execute **any** normative check without a missing Helix artifact is an error, not a green skip-only “verified.”

### 6.2 Deprecated (registry note)

[`helix-standard-version-v1.json`](../schemas/helix-standard-version-v1.json) has no `lifecycle` field yet. Do not implement it in this change.

When added (additive registry schema), recommended values: `current` | `deprecated` | `withdrawn`.

- **deprecated:** still OfficialSupported until Helix removes the pack. Mode 3 includes it. Mode 2 may select it on exact match.
- **withdrawn:** not OfficialSupported; Mode 1 errors like unknown.

Deprecation is a **Helix registry annotation**, not a GA4GH HEAD fetch.

---

## 7. Honest sentences

| Allowed | Not allowed |
|---------|-------------|
| Helix verification checks for GA4GH {product} {version} **passed** (normative bindings, pack SUPPORTED, selected = verified, evidence table complete). | Verified against GA4GH DRS 1.5.0 when no 1.5.0 pack exists. |
| Target **declared** `type.version` = … (raw copy). | Target **is** DRS 1.5.0 because Mode 3 passed 1.5.0. |
| Helix **selected** pack `ga4gh.drs.1.4.0` because … | Helix **detected** 1.0 because the path contains `/v1`. |
| Compatibility: checks for 1.4.0 and 1.5.0 passed. | The target **officially claims** those versions. |
| Declared 1.2.0; selected 1.4.0 (Mode 1). | Silent omit of 1.2.0 while validating 1.4.0 DrsObject. |

`helix security` and `helix bench` do not enter this model.

---

## 8. CLI

| Args | Mode | Today |
|------|------|--------|
| `helix verify TARGET` | unversioned wrap (not Mode 2 labelled as a pack) | Runs HelixTest DRS+WES. `selected_version` / `verified_version` empty. |
| `helix verify TARGET --standard drs` | 2, DRS only | Fail closed: `INSUFFICIENT` (does not guess 1.4.0). |
| `helix verify TARGET --standard drs --version 1.4.0` | 1 | SELECTED. DRS 1.4.0 supported pack executed. `verified_version` empty. |
| `helix verify TARGET --standard drs --version 1.5.0` | 1 | `AVAILABLE_BUT_NOT_SUPPORTED`. No pack executed. |
| `… --release-class ballot` | 1 only | Default class is official. `development` → `DEVELOPMENT_NOT_SELECTABLE`. |
| `helix verify TARGET --standard drs --all-supported-versions` | 3 | OfficialSupported is DRS 1.4.0 only (N=1 → execute). Does not iterate AVAILABLE 1.5.0. |

Usage (exit 2): `--version` without `--standard`; `--all-supported-versions` without `--standard`; `--version` with `--all-supported-versions`; unknown `--release-class`.

`--profile generic|ferrum` stays **fixture/policy**, orthogonal to pack selection ([PROFILES.md](PROFILES.md)). A profile must not pick a standard version.

Do not add automatic testing of every GA4GH release until the registry marks that version **SUPPORTED**.

---

## 9. Profiles, compare, engines

- **HelixTest** remains the engine (D1) but must load **per-pack** schemas. One process-wide `OnceCell` OpenAPI is incompatible with Mode 3 and with Mode 1 ≠ vendored default.
- **`helix compare`:** `same_measurement` should include `pack_id` (and mode). Identity mismatch is not `NEW_FAIL` ([RUN_IDENTITY.md](RUN_IDENTITY.md)).
- **Exit 0** never means certification.

---

## 10. Current tree

| Capability | Today |
|------------|--------|
| `--standard` / `--version` / `--all-supported-versions` | Shipped. Fail closed. |
| Registry OfficialSupported | DRS 1.4.0 only (YAML is not sufficient; `src/standards/support.rs` must pass). DRS 1.5.0 and WES 1.1.0 remain AVAILABLE. |
| Discovery `type.version` | Copied onto `detected_version` when 2xx JSON is sufficient; **not** used to select on the default path |
| URL inference | Rejected |
| Multiple compiled schemas | No (Mode 3 with N>1 is `MULTIPLE_PACKS_NOT_EXECUTABLE`) |
| Report four fields | `requested_version` / `detected_version` / `selected_version` / `verified_version` on every check, plus `standard`, `standards_registry_entry`, `standards_source_commit` |

Mode 2 **must not** be implemented as “keep current verify and label it 1.4.0.” That would bake in the 1.2.0 vs 1.4.0 silence.

---

## 11. What this document does not do

- Does not mark any pack SUPPORTED.
- Does not duplicate HELIOS.
- Does not make Mode 3 a claim that the target implements every version.
- Does not automatically test every GA4GH GitHub tag.
