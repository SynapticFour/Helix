# Failure report

Copy this block. You can keep it locally. Optional: paste into a [GitHub Issue](https://github.com/SynapticFour/Helix/issues) on **Helix** (GitHub account, not a Synaptic Four account). You do not need to contact Synaptic Four to run Helix.

Do not attach production secrets, tokens, or PHI. Redact `Authorization` values. This is not a GA4GH certification result.

Security vulnerabilities: do **not** use a public issue. Follow [SECURITY.md](../../SECURITY.md) (email).

```text
## Helix failure report

### Command
<!-- example: make verify-fixture   or   helix verify http://127.0.0.1:PORT --format json -->


### What I expected
<!-- pass / skip / exit 0 / JSON field … -->


### What I got
<!-- exit code, PASS/FAIL/SKIP/ERROR, or JSON status -->

Exit code:

### Helix
Commit (git -C Helix rev-parse HEAD):
`helix --version` / crate 0.1.0:

### HelixTest
VERSIONS.lock HELIXTEST_SHA:
git -C ../HelixTest rev-parse HEAD:

### Target
<!-- verify-fixture  |  my DRS/WES URL (no credentials) -->
URL or “in-process fixture”:
Layout (prefixed /ga4gh/drs/v1 vs split origin):
DRS fixture object `test-object-1` present? (yes/no/n/a):
WES fixtures mounted? (yes/no/n/a):

### rustc
rustc --version:
which rustc:
<!-- rust-toolchain.toml asks for 1.91.1; Homebrew rustc first on PATH will not match CI -->

### stdout
<!-- HELIX VERIFICATION text, or --format json. Truncate bodies; keep ids/codes. -->


### stderr
<!-- RUST_LOG traces if any. Default is error. -->


### Notes
<!-- OS, anything else. No production data. -->
```
