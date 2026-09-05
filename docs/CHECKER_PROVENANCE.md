# Helix checker provenance (B7)

Helix is HelixTest becoming a standalone VERIFY CLI. This document is how Helix proves **which DRS checker it actually executed** (the executed checker identity). It is not HELIOS (no signatures, RO-Crate, PDF). It is not GA4GH certification. Trust: [TRUST.md](TRUST.md).

HelixTest already runs. Helix productizes that engine as a path dependency (`../HelixTest/helixtest/crates/{common,framework}`). Cargo compiles **whatever is on disk**, not a git SHA written in a lockfile.

---

## 1. Four identities (do not collapse them)

| Identity | What it names | Where |
|----------|---------------|--------|
| Standard | GA4GH DRS 1.4.0 official pack (commit, pack hash, schema hashes) | `standards/registry.yaml`, `standard_selection` |
| Verifier / checker | Compiled HelixTest DRS checker sources | `standard_selection.checker_id` = `helixtest-drs:<sha256>` |
| Target | HTTP origin under test | `target.identity` |
| Fixture | Operator or default catalog object id | `drs_fixture` |

`test-object-1` is a Helix test fixture. It is not a DRS 1.4.0 requirement.

`HELIXTEST_SHA` in [VERSIONS.lock](../VERSIONS.lock) is the **git commit CI should check out**. It is not the executed checker.

---

## 2. How executed identity is computed

Compile-time (HelixTest `crates/framework/build.rs` and Helix `build.rs`, same manifest):

```text
helix-drs-checker-v1
file=crates/framework/src/drs.rs
sha256=<file>
file=crates/common/src/ga4gh_schemas.rs
sha256=<file>
file=crates/common/src/spec_source.rs
sha256=<file>
```

SHA-256 of that UTF-8 is `HELIXTEST_CHECKER_SOURCE_SHA256`. HelixTest embeds it as `HELIXTEST_DRS_CHECKER_SOURCE_SHA256`. Helix panics at build if [VERSIONS.lock](../VERSIONS.lock) disagrees with the sibling files Cargo will compile.

Runtime: `framework::drs::executed_checker_id()` returns `helixtest-drs:` plus that digest. `helix verify` records it. Callers cannot supply a checker id. YAML cannot supply it. The report serializer does not invent it.

Thought experiment:

```text
VERSIONS.lock git SHA = X
compiled checker sources = Y
→ Helix reports helixtest-drs:Y, never X as the executed checker.
If the lock digest is not Y, the build fails; if a test lies about the lock digest, verify fails closed.
```

---

## 3. What changes which id

| Change | `execution_id` | `target_execution_id` |
|--------|----------------|------------------------|
| Standard / pack / schema / catalog / checker | changes | changes (includes spec-join) |
| Target | stable | changes |
| Fixture object id / expected SHA-256 | stable | changes |

---

## 4. Reproduce the verifier identity

From a Helix checkout with sibling HelixTest:

```bash
grep '^HELIXTEST_CHECKER_SOURCE_SHA256=' VERSIONS.lock
cargo test --offline --locked --test checker_provenance
# JSON: standard_selection.checker_id == helixtest-drs:$(that digest)
```

Live target evidence is **not** stored under `/tmp`. See [EXTERNAL_EVIDENCE.md](EXTERNAL_EVIDENCE.md).
