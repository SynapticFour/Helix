# Crypt4GH protocol behaviour

HelixTest already has local **age** checks and an env-gated Ferrum **Crypt4GH HTTP** path that decrypts with a client secret key (`crypt4gh` crate, `HELIXTEST_FEATURE_CRYPT4GH_REWRAP`). Helix does **not** take that path.

This document is the Crypt4GH slice of `helix security`, run **after** the five HTTP Security Behavior Profile cases ([SECURITY_PROFILE.md](SECURITY_PROFILE.md)). It is a **narrow protocol-layout test**. It is not encryption verification.

**A pass does not mean Crypt4GH is secure. It does not mean encryption, rewrap, or key hygiene is correct.**

Source: `src/security/crypt4gh_header.rs`. Identities: [TEST_IDENTITY.md](TEST_IDENTITY.md). Fixtures: [FIXTURES.md](FIXTURES.md) §11–14.

---

## What Helix verifies

Helix can check these invariants from a **safe fixture** or a **black-box HTTP body**. No private key is loaded. Packet payloads are not decrypted.

| id | code | Source | Invariant |
|----|------|--------|-----------|
| `auth.helix.crypt4gh.header` | **`HLX-AUTH-050`** | Fixture (`test-fixtures/crypt4gh/well-formed.c4gh` or `--crypt4gh-file`) | Bytes are a Crypt4GH **unencrypted header**: magic `crypt4gh`, version **1**, packet count 1–1024, packet length fields do not overrun. |
| `auth.helix.crypt4gh.invalid_rejected` | **`HLX-AUTH-053`** | Embedded negative fixtures (wrong magic, truncated, version 2, zero packets) | Those envelopes are **rejected**. If Helix accepted them, this case **fails**. |
| `auth.helix.crypt4gh.http_envelope` | **`HLX-AUTH-054`** | Black-box `GET {drs_base}/objects/test-object-1` | If the 2xx body **starts with** magic `crypt4gh`, the same layout rules as 050 hold. If there is no magic, or GET is not 2xx, **skip** (plaintext DRS is not a fail). |

Severity on fail: catalog `error`. Errors never dump envelope bytes or keys.

Layout rules (GA4GH Crypt4GH unencrypted header, framing only):

```text
magic     8 bytes  "crypt4gh"
version   u32le    1
npackets  u32le    1..=1024
packets   n times  length u32le (≥ 8, includes the length field) + body
```

That is **protocol framing**. It is not X25519, ChaCha20-Poly1305, or a MAC check.

---

## What Helix does not verify

| Out of scope | Where it lives instead |
|--------------|------------------------|
| Decrypting header packets or data segments | GA4GH [`crypt4gh`](https://crates.io/crates/crypt4gh) crate (HelixTest env-gated HTTP) |
| Proving the payload was encrypted **to** a given public key | Needs the matching **secret** key — Helix will not hold one |
| DRS rewrap (`X-Crypt4GH-Public-Key`) round-trip | HelixTest `HLX-AUTH-051` placeholder — **not wired**; needs `CRYPT4GH_CLIENT_SECRET_KEY_PATH` |
| Plain download matches rewrap plaintext | HelixTest `HLX-AUTH-052` placeholder — **not wired** |
| Local **age** encrypt/decrypt | HelixTest `crypt4gh-tests` / `run_age_checks` (age is not Crypt4GH) |
| Key generation, rotation, storage, or a KMS | Nowhere in Helix. `test-fixtures/crypt4gh/dummy-x25519.placeholder` is **not read** |
| “This stack is secure” / production hardening | Not a Helix claim. Ferrum has no clinical pilot |
| HELIOS evidence of the encrypt dance | HELIOS (`helios-audit`) |

Helix does not implement Crypt4GH cryptography. It does not add the `crypt4gh` crate. HelixTest already compiles that crate for the secret-key HTTP path; Helix does not call it.

---

## Fixtures (no production credentials)

| File | Role |
|------|------|
| `well-formed.c4gh` | Valid **layout** (magic, version 1, one dummy packet). Not a real encrypted genome. |
| `wrong-magic.c4gh` | Invalid. Must be rejected (`HLX-AUTH-053`). |
| `truncated.c4gh` | Invalid. Must be rejected. |
| `dummy-x25519.placeholder` | Explicit non-key. **Never loaded.** |

NICHT FÜR PRODUKTION. No private keys in output.

---

## How this relates to `helix security`

```text
helix security <url>
        │
        ├─ five HTTP token cases (HLX-AUTH-010–014)
        └─ then Crypt4GH layout (HLX-AUTH-050, 053, 054)
```

JSON stays HelixTest `OverallReport` (D3). Skip on 054 is not a pass and does not fail the job. Fail on 050/053/054 does.

Passing Crypt4GH cases means: Helix recognized a well-formed header, rejected known-bad headers, and (if the target returned Crypt4GH magic) the HTTP body was framed like Crypt4GH. **It does not mean the implementation is secure.**
