# NICHT FÜR PRODUKTION

**NOT FOR PRODUCTION.** Dummy files for Helix Stage 3 only. Catalog of **all** Helix fixtures (HTTP mocks + these files): [docs/FIXTURES.md](../docs/FIXTURES.md).

Helix security tests are the **Security Behavior Profile** (black-box HTTP JWT behaviour) then Crypt4GH **protocol layout** ([docs/CRYPT4GH.md](../docs/CRYPT4GH.md)). They do not implement ga4gh-infra or Ferrum. They do not decrypt Crypt4GH. They do not produce HELIOS evidence. They are **not** a security audit. A Crypt4GH pass is not “secure”. **No real secrets.**

Do **not** use these values as Ferrum, ga4gh-infra, or hospital secrets. Do **not** copy them into `.env` for a real deployment. Ferrum has no real clinical pilot; a green Helix run against a dummy HMAC is not production hardening and not GA4GH certification.

Do **not** use these values as Ferrum, ga4gh-infra, or hospital secrets. Do **not** copy them into `.env` for a real deployment. Ferrum has no real clinical pilot; a green Helix run against a dummy HMAC is not production hardening and not GA4GH certification.

## What is here

| Path | What it is | What it is not |
|------|------------|----------------|
| `hmac/shared-secret.txt` | Dummy HS256 shared secret so CI can mint and verify JWTs without a live IdP | Not an OIDC client secret, not a Passport broker key |
| `crypt4gh/well-formed.c4gh` | Synthetic Crypt4GH header (magic `crypt4gh`, version 1, one dummy packet). **No private key material.** | Not a real encrypted genome file |
| `crypt4gh/wrong-magic.c4gh` | Same layout with a bad magic — must fail structure checks | — |
| `crypt4gh/truncated.c4gh` | Truncated header — must fail | — |
| `crypt4gh/dummy-x25519.placeholder` | Explicitly fake keypair **placeholder** (all-zero / labeled dummy). Header tests **do not read** this file. | Not a Crypt4GH X25519 secret; never use to encrypt data |

HelixTest already has HMAC JWT fixtures (`framework/src/auth.rs`, `HELIXTEST_SHARED_SECRET`). This directory is Helix’s Stage 3 copy so Helix CI never needs production keys. HelixTest stays a separate git root ([docs/DECISIONS.md](../docs/DECISIONS.md) D1).
