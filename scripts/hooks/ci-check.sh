#!/usr/bin/env bash
# Mirror .github/workflows/ci.yml cargo gates for Helix.
# Helix path-depends on a sibling HelixTest checkout (D1), same as CI.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SIBLING="$(cd "$ROOT/.." && pwd)/HelixTest"
if [[ ! -d "$SIBLING/helixtest/crates/common" ]]; then
  echo "ci-check: missing sibling HelixTest at $SIBLING" >&2
  echo "ci-check: clone it next to Helix (CI uses VERSIONS.lock HELIXTEST_SHA)." >&2
  exit 1
fi

echo "ci-check: cargo fmt --check"
cargo fmt --all -- --check

echo "ci-check: cargo clippy"
# Match .github/workflows/ci.yml (single package, not --workspace)
cargo clippy --all-targets -- -D warnings

echo "ci-check: prove (docs + cargo test)"
make prove

echo "ci-check: verify-fixture (helix verify vs in-process DRS mock)"
make verify-fixture

echo "ci-check: OK"
