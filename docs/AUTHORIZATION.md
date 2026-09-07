# Helix DRS authorization evidence boundary

Helix is HelixTest becoming a standalone VERIFY CLI. This document records what B13 inspected and what it **did not** claim. It is not GA4GH certification. It is not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Status: **B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED.**

Feasibility investigation (Bento source, not a live matrix): [B13_AUTHORIZATION_FEASIBILITY.md](B13_AUTHORIZATION_FEASIBILITY.md). Decision: **DEFER**.

---

## 1. What authorization evidence would mean

Pinned DRS 1.4.0 `openapi/tags/Auth.md` states that the **implementation** defines and enforces policy. GetObject and GetAccessURL **may** be open or require Basic, Bearer, or Passport. OPTIONS Object is the discovery API for how to authenticate. Helix does not execute OPTIONS (`drs.op.object_options` remains unevaluated).

Therefore a Helix authorization observation would be **evaluated_non_normative** security behaviour, **not** a DRS MUST that every object is protected, and **not** a new `ga4gh_requirement` predicate.

Public scenario id (not a secret): `authorization-policy-v1`.

Minimum black-box matrix against a **protected** resource:

| Scenario | Credential class | Expected |
|----------|------------------|----------|
| A | none | denied (not 404-as-missing unless that is the documented policy) |
| B | valid | allowed |
| C | invalid | denied |
| D | insufficient | denied **only if** the target has a permission/scope model |

HTTP 401 vs 403 is **not** assumed in isolation. The observation must name the resource, the request class, and the expected policy result.

`AUTHZ_ENABLED=true`, Docker labels, implementation name, and the mere presence of an `Authorization` header are **not** evidence (`src/authorization.rs`).

---

## 2. What Helix actually does today

Inspected path (`helix verify`):

```text
CLI → VerifyOptions → TargetIdentity → selection → support contract → pack
  → SpecSource → fixture → HelixTest adapter HTTP → check result → coverage → claims
```

`VerifyOptions` has **no** credential field. The HelixTest adapter used by verify does **not** send `Authorization`. Discovery uses the Helix-owned client (no redirect, 2 MiB cap, rustls). Catalog checks therefore cannot observe authorized vs unauthorized GetObject.

`helix security` is a **different** surface: dummy HMAC JWTs against Helix’s security behaviour profile. Passing it is not DRS authorization evidence for Bento or Starter Kit.

Compiled coverage (`src/coverage.rs`):

```text
id: drs.security.authorization
class: unevaluated
required_for_ga4gh: false
execution_binding: none
```

Reclassifying that row to `evaluated_non_normative` would change `coverage_id` (canonical bytes include `class`). B13 does **not** change the class without a live matrix. `catalog_id` / `execution_id` / `binding_id` stay as B11/B12.

`ga4gh_requirement` predicates (`GA4GH_VERIFIED_PREDICATES`) do not include authorization. Bento `verified_version = 1.4.0` from B12 is **not** an authorization claim.

---

## 3. Bento authorization (source, not a live stack)

Bento DRS `v0.21.5` (`1dc55ebea90185b1fec2c78c8c52909dd0ca889e`):

- `AUTHZ_ENABLED` toggles `bento_lib` `FlaskAuthMiddleware`.
- When enabled, GetObject calls a **separate** Bento Authorization Service (`BENTO_AUTHZ_SERVICE_URL`) for `query:data`.
- Denial is **403**. Missing objects can be **masked as 403** so ids are not leaked. 404 is therefore not a safe “not authorization” signal on that target when authz is on.
- `public=true` objects skip permission checks.
- The authorization service itself wants Postgres + `OPENID_CONFIG_URL` (OIDC / Keycloak) and grant assignment. Anonymous (no Bearer) is `{"anonymous": true}`.

B12 live Bento was `AUTHZ_ENABLED=false`. That observation remains a **pre-commit B12** file under `local/b12/`. It is **not** authorization evidence.

Flipping `AUTHZ_ENABLED=true` without the authorization service and OIDC grants is still not a genuine policy matrix. A stub `/policy/evaluate` would be a **test double**, not independent authorization evidence.

This environment does not have a running Bento Authorization Service + IdP + protected non-public object + disposable operator tokens. Helix therefore does **not** emit live B13 JSON that claims authorization PASS.

---

## 4. Attribution

Existing categories remain:

```text
transport_failure | target_failure | spec_failure | helix_execution_failure
target_configuration_failure | unsupported_test
```

Connection refused / timeout must not become authorization denied (`b13_t5`). Catalog unknown-id 404 (`HLX-DRS-005`) is not authorization denied (`b13_t6`). Bento’s 403-masking is a **future** probe hazard, not a current catalog result.

---

## 5. Credentials

Live tokens, if ever used, exist only in the operator environment. They must not enter Git, fixtures, JSON, logs, or identity hashes. `AuthorizationObservation` stores credential **class**, not values. JWT-shaped resource ids fail closed.

Helix redaction of Authorization / Bearer remains (`src/redact.rs`). Request limits are unchanged.

---

## 6. Tests

`tests/b13_authorization_boundary.rs` (B13-T1–T16). Prove does not require live B13 JSON. Presence of `local/b13/*.json` claiming a matrix is currently forbidden until a genuine target exists.

---

## 7. HELIOS

No signatures, RO-Crate, audit trail, or HELIOS import.
