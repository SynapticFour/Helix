# Security Behavior Profile

HelixTest already runs HMAC JWT fixtures (`framework/src/auth.rs`). This document productizes a named Helix surface: **`helix security`**. Five black-box HTTP invariants against an auth-gated DRS object. Dummy HS256 only.

**Helix verifies selected security behavior. It is not a penetration test, security audit, or certification.**

Passing these checks does **not** prove that the implementation is secure. It does not prove OIDC, GA4GH Passport, or Crypt4GH encryption. It is not a hospital IdP test. Ferrum is a reference target, not a dependency. There is no real Ferrum clinical pilot (DIZ / genomDE). HELIOS (`helios-audit`) stays out of this profile (no RO-Crate, signatures, or PDF).

Source: `src/security/profile.rs`, `src/security/http_cases.rs`. Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md) `auth.helix.token.*` / `HLX-AUTH-010`–`014`. Fixtures: [FIXTURES.md](FIXTURES.md) §9–10. CLI: [CLI_CONTRACT.md](CLI_CONTRACT.md).

---

## What this profile is

| | |
|--|--|
| Command | `helix security <url>` |
| Target | Gateway-style origin. Helix discovers DRS, then `GET {drs_base}/objects/test-object-1` with `Authorization: Bearer …` |
| Engine | Helix-owned HTTP. Does **not** import Ferrum or ga4gh-infra. Does **not** wrap HelixTest `auth.rs` (those ids are `HLX-AUTH-001`–`006`) |
| Secret | Test-only HMAC in `test-fixtures/hmac/shared-secret.txt` (or `HELIX_HMAC_SECRET` / `--hmac-secret-file`). **NICHT FÜR PRODUKTION.** |
| JSON | HelixTest `OverallReport` (D3). Human text prints the disclaimer above |
| Exit | 0 if no executed `status: fail`; skip-only HTTP (no secret) is not a fail |

Skip is never pass. Missing dummy secret → five HTTP rows **skip**. DRS not discovered → five HTTP rows **fail**.

---

## Invariants (HTTP)

Execution order is catalog order (`HLX-AUTH-010` then `011` … `014`). The numbered list below is the behaviour set, not a score.

Every case:

| Field | Meaning |
|-------|---------|
| Invariant | What must hold on the target |
| Request | Method, path, Bearer |
| Expected HTTP | What a fail-closed DRS does |
| Acceptable status classes | What Helix counts as pass |
| Failure code | Helix `code` on fail (`HLX-AUTH-0xx`) |
| Severity | Catalog severity if the case **fails** |
| Fixture | Dummy HMAC file + JWT claims. **No production credentials.** |

Shared request (except the token):

```text
GET {drs_base}/objects/test-object-1
Authorization: Bearer <token>
```

`{drs_base}` is the discovered DRS base (`…/ga4gh/drs/v1` or a split origin). Object id is the same deterministic fixture as DRS verify (`test-object-1`).

Issuer/subject on minted tokens: `https://helix.test.invalid` / `helix-stage3-fixture-user`. Algorithm: HS256. Helix never logs the secret.

### 1. Valid token accepted

| | |
|--|--|
| **id / code** | `auth.helix.token.valid` / **`HLX-AUTH-010`** |
| **Invariant** | A correctly scoped, unexpired dummy token is accepted on the protected DRS object. |
| **Request** | Bearer: aud=`drs`, scope=`drs.read`, `exp` ~ now+5m |
| **Expected HTTP** | Access allowed |
| **Acceptable status classes** | **2xx** |
| **Failure code** | `HLX-AUTH-010` |
| **Severity** | `error` |
| **Fixture** | `test-fixtures/hmac/shared-secret.txt` (`TestJwtSpec::valid_drs`) |

Without this case, a verifier that rejects everyone looks the same as a verifier that works.

### 2. Invalid token rejected

| | |
|--|--|
| **id / code** | `auth.helix.token.manipulated` / **`HLX-AUTH-013`** |
| **Invariant** | A forged or garbage Bearer is rejected (token integrity). |
| **Request** | Two Bearers: (1) valid JWT with **flipped** HS256 signature; (2) literal `not-a-jwt` |
| **Expected HTTP** | Not authenticated |
| **Acceptable status classes** | **401** (not 403) |
| **Failure code** | `HLX-AUTH-013` |
| **Severity** | `error` |
| **Fixture** | Same HMAC file; Helix mints a valid token then corrupts it. No extra secret. |

Both requests must be 401.

### 3. Expired token rejected

| | |
|--|--|
| **id / code** | `auth.helix.token.expired` / **`HLX-AUTH-011`** |
| **Invariant** | An expired dummy token is rejected (must not keep access after `exp`). |
| **Request** | Bearer: aud=`drs`, scope=`drs.read`, `exp` in the past (~ now−5m) |
| **Expected HTTP** | Not authenticated |
| **Acceptable status classes** | **401** |
| **Failure code** | `HLX-AUTH-011` |
| **Severity** | `error` |
| **Fixture** | Same HMAC file (`TestJwtSpec::expired_drs`) |

### 4. Wrong scope denied

| | |
|--|--|
| **id / code** | `auth.helix.token.wrong_scope` / **`HLX-AUTH-012`** |
| **Invariant** | A dummy token with the wrong scope is denied on this DRS object. |
| **Request** | Bearer: aud=`drs`, scope=`wes.run` (unexpired) |
| **Expected HTTP** | Not authorized for this resource |
| **Acceptable status classes** | **401 or 403** |
| **Failure code** | `HLX-AUTH-012` |
| **Severity** | `error` |
| **Fixture** | Same HMAC file (`TestJwtSpec::wrong_scope`) |

A WES-run token must not read DRS objects.

### 5. Wrong audience / service denied

| | |
|--|--|
| **id / code** | `auth.helix.token.wrong_audience` / **`HLX-AUTH-014`** |
| **Invariant** | A dummy token minted for another service (WES) is denied on DRS. |
| **Request** | Bearer: aud=`wes`, scope=`drs.read` (unexpired) |
| **Expected HTTP** | Not authorized for this service |
| **Acceptable status classes** | **401 or 403** |
| **Failure code** | `HLX-AUTH-014` |
| **Severity** | `error` |
| **Fixture** | Same HMAC file (`TestJwtSpec::wes_audience`) |

Audience confusion: a WES access token must not unlock DRS.

---

## Negative tests (broken mocks)

A profile that only passes against a correct mock is incomplete. Helix ships **intentionally broken** in-process gates (`VerifierPolicy` in `src/security/jwt.rs`). Prove CI (`make prove`) asserts Helix **fails** the matching case:

| Broken mock | Policy | Case that must FAIL | What it proves |
|-------------|--------|---------------------|----------------|
| Closed gate | `reject_all` (always 401) | `HLX-AUTH-010` valid | Helix notices “nobody gets in” |
| Ignore expiry | `ignore_expiry` | `HLX-AUTH-011` expired | Helix notices missing `exp` |
| Ignore scope | `ignore_scope` | `HLX-AUTH-012` wrong scope | Helix notices missing scope |
| Ignore signature | `ignore_signature` | `HLX-AUTH-013` manipulated | Helix notices missing HMAC check |
| Ignore audience | `ignore_audience` | `HLX-AUTH-014` wrong audience | Helix notices missing `aud` |

On a closed gate, denial cases still pass (they expect 401). On ignore-one-check, the valid-token case still passes. That is the point: Helix names **which** invariant broke.

These mocks are not Ferrum. They are not production verifiers.

---

## Crypt4GH protocol layout (after the five HTTP cases)

Crypt4GH is **not** one of the five HTTP token invariants. Helix runs it only after those cases.

Contract: [CRYPT4GH.md](CRYPT4GH.md). Helix verifies **header framing** (`HLX-AUTH-050`, `HLX-AUTH-053`) and, if a 2xx DRS body starts with Crypt4GH magic, that same layout on the wire (`HLX-AUTH-054`). It does **not** decrypt, load private keys, or implement X25519/ChaCha20.

**A Crypt4GH pass does not mean the implementation is secure.**

---

## What a pass is not

- Not a penetration test
- Not a security audit
- Not GA4GH certification or Passport certification
- Not proof that Ferrum (or any stack) is production-hardened
- Not HELIOS evidence
- Not a reason to put the dummy HMAC in a real deployment

Green CI is a technical signal that these selected behaviours held against the target Helix reached.

---

## How to run

```text
helix security http://127.0.0.1:8080
helix security http://127.0.0.1:8080 --format json \
  --hmac-secret-file test-fixtures/hmac/shared-secret.txt
```

In-process prove: `make prove` (no Ferrum, no real secrets). Live stack is opt-in `make test-live` for **verify**; pointing `helix security` at Ferrum HMAC-on is documented, not the CI default.
