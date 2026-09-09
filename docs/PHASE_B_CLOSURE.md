# Phase B closure

Helix is HelixTest becoming a standalone VERIFY CLI. This document records whether the DRS 1.4.0 productization sequence (B1–B15, plus the P1/P2 operator path) can be formally closed. It is not a new verification architecture. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Product contract: [HELIX_PRODUCT.md](HELIX_PRODUCT.md). Operator path: [OPERATOR_VERIFY.md](OPERATOR_VERIFY.md). First usable release: [P2_FIRST_USABLE_RELEASE.md](P2_FIRST_USABLE_RELEASE.md).

```text
B13 remains closed/deferred.
authorization = UNEVALUATED
```

---

## Executive conclusion

Phase B can close. The current Helix tree implements a coherent, fail-closed DRS 1.4.0 technical verification path: pinned local packs, SpecSource execution, exact checker provenance, target/fixture execution identity, derived claims, partial coverage, durable unsigned evidence, and an operator journey that distinguishes PASS from VERIFIED. Authorization verification is intentionally **DEFERRED**. Unsigned evidence, historical-versus-current standing, partial coverage, and the inability to replay live Docker targets in every environment remain documented findings, not bugs to hide.

---

## What Phase B established

- Helix-owned `helix verify` JSON (`helix-verification-v1`), not a pretty-print of HelixTest `OverallReport`.
- Version-pinned GA4GH packs in `standards/vendor/` with integrity hashes; no runtime fetch of standards.
- Fail-closed selection: no silent version substitution; AVAILABLE is not SUPPORTED; default verify stays unversioned.
- SpecSource-based DRS 1.4.0 execution through the HelixTest adapter (`Mode::Generic`).
- Executed checker identity `helixtest-drs:<source sha256>`, distinct from the HelixTest git checkout pin.
- Target identity and `target_execution_id` distinct from pack `execution_id`.
- Independent-implementation classification and check-level differential (not a ranking).
- Mutation / negative-control revocation of `verified_version` without changing verifier identity.
- Claim join: `verified_version` stamped only when `ga4gh_requirement` predicates hold.
- Coverage accounting: UNEVALUATED ≠ OUT_OF_SCOPE ≠ PASS ≠ VERIFIED; partial coverage is honest.
- Evidence durability: reload, recompute claims/coverage, historical vs current standing, tamper rejection.
- Operator UX: human report, `--output`, `helix inspect`, declared/detected/selected/verified.
- P1/P2 productization: source-build install, one canonical DRS 1.4.0 workflow.

Superseded B2 language (“`verified_version` always empty”) is no longer the executable contract; B8 stamps it only from predicates.

---

## What is verified today

An operator can select **GA4GH DRS 1.4.0** (`--standard drs --version 1.4.0`), run Helix against an HTTP origin or the in-process fixture, retain `helix-verification-v1`, and inspect it.

`ga4gh_requirement` VERIFIED means the Helix DRS 1.4.0 **support contract** was satisfied for that target and fixture: selected supported pack, pinned bytes, integrity, exact checker, normative SpecSource check(s), catalog completeness, claim join, and coverage rules. It does **not** mean full DRS coverage, authorization, reproducibility certification, or official GA4GH certification.

Default `helix verify URL` remains unversioned. Exit 0 is check-PASS (≥1 PASS, no FAIL/ERROR), not VERIFIED.

---

## Evidence boundary

Persisted JSON records target, selected standard/version, checker, checks, derived claims, coverage, and Helix/HelixTest provenance. `helix inspect` recomputes claims, coverage, and standing. Standing is not a JSON field.

The artefact does **not** prove: cryptographic binding of a URL to a Docker digest or git commit; that another Helix binary is the same verifier (per-binary currentness); signatures, RO-Crate, or PDF (HELIOS); authorization; full DRS.

A consistent unsigned restamp (`finalize_run` after changing observations) cannot be distinguished from a genuine re-run. That remains the HELIOS boundary.

---

## Independent implementation evidence

Two reviewed independent **local** DRS implementations were exercised in B9/B12:

| Target | Lineage (operator provenance) | Detected | Selected | `ga4gh_requirement` | Coverage |
|--------|--------------------------------|----------|----------|----------------------|----------|
| GA4GH Starter Kit DRS 0.3.2 | image `ga4gh/ga4gh-starter-kit-drs:0.3.2` | `1.3.0experimental` | 1.4.0 | NOT_VERIFIED | blocked |
| Bento DRS v0.21.5 | source commit `1dc55ebe…` | 1.4.0 | 1.4.0 | VERIFIED | partial |

Same `execution_id` / `coverage_id`; distinct `target_execution_id`. Relabeling one as the other is Invalid. Mocks cannot be upgraded to independent by `--implementation-name`. Live JSON is gitignored (`local/b12/`), historical (`helix_git_sha` absent), and is **not** restamped. The `helix matrix` slots remain pending; that is a different artefact from B12.

Not a ranking. Not certification. Not CI. Authorization unevaluated on both.

---

## Coverage

Current DRS 1.4.0 coverage is **partial**. Required catalog rows can PASS (and `ga4gh_requirement` can be VERIFIED) while pinned OpenAPI operations remain UNEVALUATED. Partial coverage cannot silently become complete: `coverage_id` is compiled from the contract; inflating a row to PASS without execution fails integrity. Authorization (`drs.security.authorization`) stays UNEVALUATED.

---

## B13

**Authorization verification: DEFERRED**

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
```

Bento DRS authorization is a separate OIDC-backed service. B12 ran `AUTHZ_ENABLED=false`. `helix verify` has no credential field and does not send `Authorization`. Configuration labels are not evidence. Reopening B13 requires a real, reproducible, locally runnable authorization-enabled implementation plus a black-box matrix (none / valid / invalid / insufficient). Do not invent a mock as production evidence.

---

## Phase-B invariant

A Helix DRS result cannot honestly become `ga4gh_requirement` VERIFIED unless selected supported version, pinned pack, integrity hashes, exact checker provenance, normative SpecSource execution, target/fixture execution, required checks, claim join, and coverage/claim-integrity rules all agree. `substituted` is always false. Persisted `claims[]` is not authority.

Remaining bypass that Phase B **accepts** (does not close): unsigned consistent restamp. Helix does not sign.

---

## Gate status (B1–B15)

| Gate | Established | Current enforcement | Status |
|------|-------------|---------------------|--------|
| B1 | Non-Ferrum DRS mock / generic adapter | `tests/support/mock_ga4gh_drs.rs`, `Mode::Generic` | Satisfied |
| B2 | Fail-closed version selection; four version facts | `src/standards/select.rs`, `verify_versions.rs` | Satisfied; “always empty verified” **superseded** by B8 |
| B2.5 / B2.6 | Pack load / integrity / SpecSource join | `src/standards/pack.rs`, `spec_join.rs` | Satisfied |
| B3 | Support contract / catalog / binding | `src/standards/support.rs`, `support_gate.rs` | Satisfied |
| B4 | Target identity / `target_execution_id` | `src/target.rs`, `tests/multi_target.rs` | Satisfied |
| B5 / B6 | Fixture object vs standard; operator `--drs-object-id` | `src/fixture.rs`, `tests/drs_fixture.rs` | Satisfied |
| B7 / B7.1 | Checker provenance v2; pin ≠ executed id | `src/checker.rs`, `VERSIONS.lock`, `checker_provenance.rs` | Satisfied |
| B8 | Claim integrity; stamp `verified_version` | `src/claims.rs`, `src/claim_integrity.rs`, `tests/b8_claim_integrity.rs` | Satisfied (24 tests) |
| B9 | Independence + differential | `src/independence.rs`, `src/differential.rs`, `tests/b9_independent_differential.rs` | Satisfied (25 tests) |
| B10 | Negative controls | `tests/b10_negative_control.rs` (M1–M6 + forgeries) | Satisfied (28 tests) |
| B11 | Coverage honesty | `src/coverage.rs`, `tests/b11_coverage_boundary.rs` | Satisfied (34 tests) |
| B12 | Live Starter Kit / Bento | `tests/b12_live_reconciliation.rs`; JSON optional/gitignored | Satisfied; live HTTP not required for prove |
| B12.1 | Helix git provenance | `src/provenance.rs`, `tests/b12_1_verifier_provenance.rs` | Satisfied (9 tests) |
| B13 | Authorization boundary | `src/authorization.rs`, `tests/b13_authorization_boundary.rs` | **DEFERRED** (18 tests keep it closed) |
| B14 | Evidence durability | `src/evidence.rs`, `tests/b14_evidence_durability.rs` | PASS WITH FINDINGS (29 tests) |
| B15 | Operator UX / inspect | `helix inspect`, `tests/b15_operator_ux.rs` | PASS WITH FINDINGS (31 tests) |
| P1 / P2 | Product page + first usable path | `HELIX_PRODUCT.md`, `--output`, `make verify-drs`, `tests/p2_first_usable_release.rs` | PASS WITH FINDINGS (20 tests) |

---

## Accepted findings

Carry-forward (do not hide):

1. **Unsigned evidence.** Consistent restamp is indistinguishable from a genuine run. HELIOS owns signing.
2. **Historical vs current standing** is per-binary. B12 JSON is `historical_observation`. Missing compile-time git ⇒ no Current standing.
3. **CLI exit 0 ≠ VERIFIED.** Frozen; stated in help and report.
4. **Partial DRS coverage.** Honest VERIFIED can still list UNEVALUATED operations.
5. **Authorization UNEVALUATED.** B13 deferred.
6. **No cryptographic URL↔artifact bind.** `live_independent_observation` rejects mock/catalog masquerade only.
7. **Live Docker/qemu targets** are not replayable in CI; B12 JSON is optional operator evidence.
8. **No published binary.** Source build only.
9. **`helix matrix` external slots pending.** Distinct from B12 local observations.
10. **HelixTest checkout** is pin `1baddfd3…` on branch `wip/drs-specsource` (clean). CI clones the SHA, not the branch name.
11. **Productization branch** `wip/drs-140-productization` is not `main`. Do not announce as if `main` already contains this close.
12. **PUBLIC_READINESS_AUDIT.md (2026-09-05)** still contains a “No SUPPORTED pack” blocker row; superseded by this close (banner added on that file).

B14/B15 findings about unsigned restamp, standing, exit 0, and live-target replay are **not** reclassified as resolved.

---

## Validation

Proof time: 2026-09-08. Helix working tree was **clean** at `8ab4b1f15df07e2ed1256a3a87975e7582029030` before this closure document. HelixTest sibling: clean `1baddfd3d75f01dc7c149074a785616fa014c725`. `Cargo.lock` unchanged. `local/` gitignored; not required. Rebuilt binary:

```text
helix 0.1.0
Helix git: 8ab4b1f15df07e2ed1256a3a87975e7582029030
HelixTest pin: v0.1.3
Not GA4GH certification. Not HELIOS.
```

```text
cargo fmt --all -- --check                         # exit 0
cargo clippy --offline --locked --all-targets --all-features -- -D warnings  # exit 0
cargo test --offline --locked --all-targets       # 0 failed
  lib 288; b8 24; b9 25; b10 28; b11 34; b12.1 9; b12 7;
  b13 18; b14 29; b15 31; p2 20
./scripts/prove.sh                                 # prove: docs OK
```

No tests were weakened. HelixTest was not modified. B12 JSON was not restamped. Schema remains `helix-verification-v1`.

---

## Scope confirmation

This close did **not** add: authorization, new standards, coverage expansion, HELIOS, signing, cache, telemetry, upload, ranking, or certification claims.

---

## Phase-B verdict

```text
PASS WITH FINDINGS
```

Phase B is **formally closed** on this branch with the findings above. B13 deferred is not a blocker.

---

## What comes next

Do not start the next engineering gate because this file exists. Review [HELIX_ROADMAP.md](HELIX_ROADMAP.md) and [HELIX_PRODUCT.md](HELIX_PRODUCT.md). The next step is a **product decision**, not the next internal B-gate. Candidates (not started here): merge this branch to `main` if this is the public baseline; a tagged source release; WES as a versioned pack; genuine authorization only if a live stack exists. HELIOS remains out of Helix.
