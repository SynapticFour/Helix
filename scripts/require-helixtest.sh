#!/usr/bin/env bash
# Sibling HelixTest must exist (Cargo path dep). Warn if HEAD ≠ VERSIONS.lock pin.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SIBLING="$ROOT/../HelixTest/helixtest/crates/common/Cargo.toml"
if [[ ! -f "$SIBLING" ]]; then
  echo "Helix needs a sibling HelixTest checkout at ../HelixTest (path dependency)." >&2
  echo "HelixTest is a separate git root. Clone both, then pin HelixTest to VERSIONS.lock:" >&2
  echo "  git clone https://github.com/SynapticFour/Helix.git" >&2
  echo "  git clone https://github.com/SynapticFour/HelixTest.git" >&2
  echo "  git -C HelixTest checkout \"\$(grep '^HELIXTEST_SHA=' Helix/VERSIONS.lock | cut -d= -f2)\"" >&2
  echo "  cd Helix && make prove" >&2
  echo "See docs/INSTALL.md" >&2
  exit 2
fi
WANT="$(grep '^HELIXTEST_SHA=' "$ROOT/VERSIONS.lock" | cut -d= -f2 || true)"
HAVE="$(git -C "$ROOT/../HelixTest" rev-parse HEAD 2>/dev/null || true)"
if [[ -n "$WANT" && -n "$HAVE" && "$HAVE" != "$WANT" ]]; then
  echo "warning: HelixTest HEAD is ${HAVE}; VERSIONS.lock pin is ${WANT} (tag v0.1.3)." >&2
  echo "CI uses the pin. Checkout: git -C ../HelixTest checkout ${WANT}" >&2
fi
