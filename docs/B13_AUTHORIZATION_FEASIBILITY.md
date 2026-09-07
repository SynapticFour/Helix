# B13 — Bento authorization feasibility

Helix is HelixTest becoming a standalone VERIFY CLI. This document is a **feasibility investigation**, not authorization verification, not GA4GH certification, and not HELIOS.

**Vertraue mir nicht, vertraue dem Code.**

Inspected source (gitignored operator copies under `Helix/local/b13/source/`, not committed):

```text
bento_drs              v0.21.5  1dc55ebea90185b1fec2c78c8c52909dd0ca889e
bento-lib              17.2.0   wheel sha256:d91f7b3ed42e51bf472e7265fc59cf4a1e65bf66ca1994bacac1d9cedd39e1dd
bento_authorization_service  main@107334104bfb919fcbaa9b808fa4f0384e3588f3
                               (inspection pin; not a Bento DRS release artifact)
```

No live authorization-enabled DRS was started for this investigation. B12 `local/b12/` JSON was not rewritten.

---

## 1. Decision

```text
DEFER
```

```text
B13 — Authorization Evidence Boundary: BLOCKED / DEFERRED
```

This is not a Helix software failure. Existing DRS 1.4.0 B8–B12.1 identities and claims are unchanged. `drs.security.authorization` remains **UNEVALUATED**.

---

## 2. Target

| Field | Value |
|-------|--------|
| Implementation | `bento_drs` (PyPI/package name `chord-drs`) |
| Version | `0.21.5` |
| Source | `https://github.com/bento-platform/bento_drs` tag `v0.21.5` |
| Commit | `1dc55ebea90185b1fec2c78c8c52909dd0ca889e` |
| License | LGPL-3.0 |
| Image/digest | **none for B9/B12.** Those runs were local source + Flask, not a pinned container digest. `Dockerfile` exists (`ghcr.io/bento-platform/bento_base_image:python-debian-2026.08.01`) and was not the B12 artifact. |
| Architecture | B12: local-source on the operator machine (`targets/independence.yaml` `platform: local-source`) |
| Startup (B12) | `FLASK_APP=wsgi:application` → `poetry run flask run --host 127.0.0.1 --port 5000` |
| Runtime configuration (B12) | `AUTHZ_ENABLED=false` (override). `.env-sample` default is `AUTHZ_ENABLED=true` plus `BENTO_AUTHZ_SERVICE_URL`. |

B12 configuration (`AUTHZ_ENABLED=false`) is **not** authorization-enabled configuration. See §4.

---

## 3. Actual authorization architecture

Bento DRS does **not** validate JWTs, evaluate grants, or host an IdP. When authorization is enabled, DRS **forwards** the incoming `Authorization` header to a **separate** Bento Authorization Service and maps that service’s boolean matrix onto GetObject.

```text
HTTP Authorization: Bearer <JWT>   (optional)
        ↓
chord_drs/authz.py
  FlaskAuthMiddleware (bento_lib 17.2.0)
        ↓  POST {AUTHZ_URL}/policy/evaluate
           headers: Authorization forwarded if present
           body: resources + permissions (query:data / download:data / …)
        ↓
bento_authorization_service
  OptionalBearerToken
        ↓  IdPManager.decode (OpenID config + JWKS, RS* in production)
  missing token        → token_data = None  ({"anonymous": true} in engine)
  expired / bad aud    → evaluate as all-false (HTTP 200 from authz)
  non-JWT Bearer       → HTTP 400 from authz  → DRS/bento_lib raises 500
        ↓
Postgres grants/groups
        ↓  policy_engine.evaluation.evaluate
        ↓
boolean matrix
        ↓
chord_drs/routes.py fetch_and_check_object_permissions
  public=true          → allow without looking up that object’s grant
  AUTHZ_ENABLED=false  → all True (no POST)
  False                → werkzeug Forbidden → HTTP 403
  missing object + no “everything” permission → 403 (ID masking)
  missing object + everything permission      → 404
        ↓
GET /ga4gh/drs/v1/objects/{id}  (P_QUERY_DATA)
GET .../objects/{id}/download   (P_DOWNLOAD_DATA)
```

### Code (DRS `1dc55ebe`)

| Step | Path | Function / object |
|------|------|-------------------|
| Toggle | `chord_drs/config.py` | `AUTHZ_ENABLED = str_to_bool(os.environ.get("AUTHZ_ENABLED", "true"))`; if true, `BENTO_AUTHZ_SERVICE_URL` is **required** (`sys.exit(1)` if unset) |
| Middleware | `chord_drs/authz.py` | `FlaskAuthMiddleware(Config.AUTHZ_URL, enabled=Config.AUTHZ_ENABLED)` |
| Attach | `chord_drs/app.py` | `authz_middleware.attach(application)` |
| GetObject | `chord_drs/routes.py` | `object_info` → `fetch_and_check_object_permissions(..., P_QUERY_DATA)` |
| Download | `chord_drs/routes.py` | `P_DOWNLOAD_DATA` |
| Public skip | `chord_drs/models.py` `DrsBlob.public`; `check_objects_permission` | `drs_obj.public or authz_results[...]` |
| Disabled path | `routes.py` `check_objects_permission` | `if not authz_enabled(): return tuple([True] * len(drs_objs))` |
| After-request | `bento_lib.auth.middleware.flask.FlaskAuthMiddleware.middleware_post` | if enabled and authz not marked done → **403** |
| Public endpoint | `routes.py` `service_info` | `@authz_middleware.deco_public_endpoint` |

### Code (bento_lib 17.2.0)

| Step | Path | Behaviour |
|------|------|-----------|
| Forward token | `bento_lib/auth/middleware/base.py` `_extract_token_and_build_headers` | `require_token=False` on `evaluate()` used by DRS: missing header → **empty headers**, not 401 |
| Call | `authz_post` | `POST {url}/policy/evaluate`; non-200 → `BentoAuthException(..., status_code=500)` (does not leak authz body) |
| GetObject evaluate | `evaluate()` / `evaluate_one()` | DRS uses these; `require_token` defaults **false** |

### Code (authorization service, inspected `10733410`)

| Step | Path | Behaviour |
|------|------|-----------|
| Config | `config.py` | `openid_config_url` default `https://bentov2auth.local/realms/bentov2/.well-known/openid-configuration`; `token_audience=account`; **HS256/HS384/HS512 disabled** |
| JWT | `idp_manager.py` `IdPManager.decode` | fetch OpenID config + JWKS; verify `kid`, audience, permitted algs |
| No token | `routers/policy/common.py` `use_token_data_or_return_error_state` | `token_data = None` |
| Bad JWT | same | `DecodeError` → **400** “Bearer token must be a valid JWT” |
| Expired / bad aud | same | return all-false matrix (HTTP **200**) |

DRS tests (`tests/test_routes.py`) never run this service. They stub:

```python
responses.post(f"{AUTHZ_URL}/policy/evaluate", json={"result": [[True] ...]})
```

That stub is **Bento’s unit-test double**. Using the same stub as Helix live evidence would violate this investigation’s “no fake authorization” rule.

---

## 4. Configuration

### B12 (observed, not authorization)

```text
AUTHZ_ENABLED=false
BENTO_DEBUG=true
SERVICE_BASE_URL=http://127.0.0.1:5000
DATABASE=...
DATA=...
# BENTO_AUTHZ_SERVICE_URL not required when AUTHZ_ENABLED=false
```

Effect in code: `check_objects_permission` short-circuits to all-true. Middleware `enabled=false` skips the after-request 403. Objects ingested by `flask ingest` still default `public=false` in the ORM; they were reachable because **authorization was off**, not because they were public.

### Genuine authorization-enabled (from `.env-sample` + authz service README)

```text
AUTHZ_ENABLED=true
BENTO_AUTHZ_SERVICE_URL=http://<real-bento-authorization-service>/
```

plus, **outside DRS**:

```text
# authorization service
DATABASE_URI=postgres://...
OPENID_CONFIG_URL=https://<idp>/.well-known/openid-configuration
# IdP must serve JWKS; HS* algorithms are disabled in production config
# Postgres grant/group rows for query:data on the object’s project/dataset
# disposable OIDC client + user; RS256 (or other permitted) access tokens
```

`AUTHZ_ENABLED=true` **alone is not genuine authorization**. If the URL points at a Helix-written `/policy/evaluate` stub, DRS will still emit 200/403 according to the stub. That is not Bento policy evaluation.

Bento’s documented local enablement is the **Bento node**: Keycloak (HTTPS subdomain via gateway), `./bentoctl.bash init-auth`, `bento_authz create grant`, Postgres. That is platform orchestration, not a DRS-only dotenv.

---

## 5. Credential model

No secrets are recorded. Classes only:

| Class | What Bento actually does |
|-------|--------------------------|
| none | DRS forwards no header; authz treats subject as anonymous; grants for everyone/anonymous apply; typically **403** on a non-public object with no anonymous `query:data` grant |
| valid + `query:data` on the object’s resource | authz returns true; DRS **200** GetObject |
| valid + no grant | authz returns false; DRS **403** |
| expired / wrong audience JWT | authz **200** with all-false; DRS **403** (looks like denial, not 401) |
| non-JWT `Bearer` | authz **400**; DRS/bento_lib **500** (`Error from authz service`) |
| HS256/HS384/HS512 | **rejected** by production `disabled_token_signing_algorithms`; DRS tests of the authz service that mint HS512 tokens are **not** production-equivalent |

Disposable Keycloak users would be sufficient **if** a real IdP + real authz service + grants existed. None of that was instantiated here. Authorization-service tests use `MockIdPManager` + a hard-coded HS512 test secret; that mock is not a local substitute the production `IdPManager` supports.

---

## 6. Protected resource

Preferred operation (already in the DRS catalog’s HTTP surface, not a new Helix check):

| Field | Value |
|-------|--------|
| Endpoint | `GET /ga4gh/drs/v1/objects/{id}` |
| Also | `GET /objects/{id}/download` uses `P_DOWNLOAD_DATA` (stricter than GetObject) |
| Object | B12 ingested UUID `f23b1635-4a65-40fa-8b29-75b4734b602a` (historical). `flask ingest` does not set `public=true` (default false). |
| Authentication required? | **No** at the DRS layer (`require_token=False`). Missing token is anonymous evaluation. |
| Authorization evaluated? | **Only if** `AUTHZ_ENABLED=true` **and** a real authz service answers `/policy/evaluate`. |
| Expected allowed | HTTP 200 JSON DrsObject (GetObject) |
| Expected denied | HTTP **403** Forbidden (not 401; not isolated 404) |
| Decision code | `chord_drs/routes.py` `fetch_and_check_object_permissions` |

`GET /service-info` is explicitly public (`deco_public_endpoint`) and is **not** an authorization-protected DRS object operation.

---

## 7. Feasibility matrix

No live authorization-enabled instance was observed. Expected behaviour is from **source + Bento tests**, not from a Helix live run.

| Scenario | Credential | Expected (from source) | Observed live | Notes |
|----------|------------|------------------------|---------------|--------|
| A | none | 403 on non-public object if authz enabled and no anonymous grant | **not observed** | B12 AUTHZ=false → 200; that is N1, not A |
| B | valid JWT + `query:data` grant | 200 | **not observed** | requires real IdP + grants |
| C | invalid / non-JWT | 400 at authz → **500 at DRS**; expired JWT → 403 | **not observed** | C is **not** a clean 401; do not treat 500 as “denied” without this path |
| D | valid JWT, no `query:data` | 403 | **not observed** | supported by grant model (`iss`/`sub` vs group) |

`public=true` objects skip object-level grant checks and would **invalidate** A/B/D if used as the protected resource.

---

## 8. Attribution

| Observation | Attribution |
|-------------|-------------|
| Connection refused / timeout / TLS | `transport_failure` — not denial |
| Authz service down / non-200 (except mapped 403 from DRS Forbidden) | DRS raises **500** via bento_lib — `target_failure` / `target_configuration_failure`, **not** authorization denial |
| 404 on GetObject with authz on | only if caller has “everything” permission; otherwise missing IDs are **403**. Catalog unknown-id 404 from B12 (`AUTHZ=false`) is **not** authorization evidence |
| 403 from `fetch_and_check_object_permissions` after a **real** `/policy/evaluate` false | authorization denial **attributable to Bento’s DRS+authz path** |
| 403 because Helix/nginx/Traefik returned it | **not** Bento DRS authorization |
| 200 because `AUTHZ_ENABLED=false` | reachability, **not** authorization |
| 200/403 because a stub returned `[[True]]`/`[[False]]` | **test double**, not independent authorization evidence |
| `AUTHZ_ENABLED=true` in env / Docker label | configuration, **not** evidence |

This investigation did **not** obtain a result attributable to Bento’s real authorization service.

---

## 9. Helix implications

Unchanged (must remain):

- DRS 1.4.0 pack / schema / component / binding / catalog / coverage / execution identities
- HelixTest SHA and `checker_id`
- Bento `verified_version = 1.4.0` / `ga4gh_requirement = verified` / `coverage.state = partial` from B12
- Starter Kit `verified_version = null` / `not_verified` / `blocked`
- `drs.security.authorization` class **unevaluated**, `required_for_ga4gh: false`
- No `if target_id == "bento-drs"` in the verifier
- No HELIOS

DRS 1.4.0 `Auth.md` leaves policy to the implementer; GetObject **may** be open. If a live matrix is ever obtained, the coverage row belongs in **`evaluated_non_normative`**, not normative, and **must not** be added silently to `GA4GH_VERIFIED_PREDICATES`. Reclassifying the row **would** change `coverage_id` (canonical bytes include class). That is a future, explicit contract change — not this investigation.

Helix `verify` still has **no** credential field. Even a genuine Bento stack cannot be observed as A/B/C/D through current `helix verify` without a **generic** (not Bento-named) operator credential input. That Helix gap is a future implementation item, not a reason to stub authz.

Existing fail-closed tests (`tests/b13_authorization_boundary.rs`) already encode: configuration is not evidence; mock is not live; forged PASS fails integrity. This investigation does not add a live-matrix test suite.

---

## 10. Decision rationale

**DEFER**, because essential requirements for **clean attributable Helix B13 evidence** are not met:

1. **Authorization is not local to DRS.** Enabling it requires Postgres + Bento Authorization Service + an OIDC IdP with JWKS. DRS has no local identity/policy file mode.
2. **Production IdP path forbids the algorithm used in authz unit tests.** HS* is disabled; `MockIdPManager` is a test double. Substituting it (or a `/policy/evaluate` stub) is forbidden as B13 evidence.
3. **Bento’s own reproducible local recipe is a full node** (`bentoctl init-auth`, Keycloak HTTPS gateway, grant CLI), not a disposable DRS sidecar. That is infrastructure **outside** Helix’s offline `cargo test --locked` boundary and is **not running** in this environment.
4. **Invalid vs missing credentials are not a 401 pair.** Missing → anonymous 403; garbage Bearer → DRS 500. A naive “401 = auth” matrix would mis-attribute.
5. **Helix cannot send credentials today.** Combined with (1–3), a live matrix cannot be honestly recorded.

### What would unblock a later gate (not scheduled, not claimed)

- Operator-run **real** `bento_authorization_service` (not a stub) + Postgres grants + **RS256** OIDC (disposable Keycloak realm/users, not production federation).
- DRS `AUTHZ_ENABLED=true` pointing at that service; **non-public** object; same object for A/B/D.
- Generic Helix-owned credential input on verify (env, never Git, never identity hashes).
- Then reclassify coverage only via the existing machine-controlled mechanism, as **evaluated_non_normative**.

Continuing **without** filling the authorization row is safe and honest: B12 already recorded Bento DRS 1.4.0 catalog verification with authorization **unevaluated**. Filling the row from configuration or a stub would **weaken** the evidence boundary.

---

## 11. Security

- No production credentials used or stored.
- No secrets in this document, Git, fixtures, or B12 JSON.
- Gitignored clones under `Helix/local/b13/source/` contain only public repositories; authorization-service **tests** embed a dummy HS512 string used only in their MockIdP — Helix must not copy it into reports or verifier identity.
- No live tokens. No Helix network trust change. No redirect/limit weakening.
- B12/B12.1 live JSON not overwritten.
- Not HELIOS.

---

## Negative controls (designed, not live-executed)

| ID | Setup | Expected |
|----|--------|----------|
| N1 | `AUTHZ_ENABLED=false` (B12) | no authorization evidence; B13 must not pass |
| N2 | non-JWT Bearer against real authz | DRS 500 (authz 400), **not** a 401 pass |
| N3 | valid identity, no `query:data` grant | 403 |
| N4 | valid identity with grant | 200 GetObject |
| N5 | forge JSON `authorization = PASS` | `validate_claim_integrity` fails (already tested) |
| N6 | mock / stub `/policy/evaluate` | must not count as live independent authorization |

N1 is historically satisfied by B12. N2–N4 were **not** live-run (DEFER). N5–N6 are already encoded in Helix tests.
