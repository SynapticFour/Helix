#!/usr/bin/env bash
# Independent verification from pinned material. No GA4GH download. Not HELIOS.
# Not bit-for-bit identity of verify JSON (timestamp differs).
set -euo pipefail
export TZ=UTC
export LC_ALL=C
export LANG=C
export RUST_LOG="${RUST_LOG:-error}"
export NO_COLOR=1
export CARGO_TERM_COLOR=never

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

chmod +x "$ROOT/scripts/require-helixtest.sh"
"$ROOT/scripts/require-helixtest.sh"

echo "independent-verify: rustc=$(rustc --version 2>/dev/null || echo missing)"
echo "independent-verify: pin=$(grep '^HELIXTEST_SHA=' VERSIONS.lock)"
echo "independent-verify: this is not bit-for-bit file identity (timestamp is wall clock)."

if ! cargo metadata --locked --offline --format-version 1 >/dev/null 2>&1; then
  echo "crate cache incomplete for Cargo.lock." >&2
  echo "Explicit network step (pinned crates, not latest GA4GH): make fetch" >&2
  exit 2
fi

export CARGO_NET_OFFLINE=true

echo "independent-verify: helix standards validate (vendored bytes + sha256)"
cargo run --locked --offline --quiet --bin helix -- standards validate

echo "independent-verify: reproducibility tests"
cargo test --locked --offline --test repro -- --test-threads=1

echo "independent-verify: OK (offline, Cargo.lock, vendor hashes). Not certification."
