#!/usr/bin/env bash
# Zero-risk proof for the Helix docs repo (no Ferrum, no HELIOS, no live stack).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

missing=0
for f in \
  README.md LICENSE CONTRIBUTING.md SECURITY.md CHANGELOG.md \
  INVENTORY.md \
  docs/IDENTITY.md docs/ECOSYSTEM.md docs/DEPENDENCY.md docs/PROVE.md \
  docs/FOR-EVALUATORS.md docs/INSTALL.md
do
  if [[ ! -f "$f" ]]; then
    echo "missing $f" >&2
    missing=1
  fi
done
if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

grep -q "Not a product SKU" docs/IDENTITY.md
grep -q "HELIOS" docs/IDENTITY.md
grep -q "GA4GH certification" README.md
grep -q "clinical pilot" docs/IDENTITY.md
grep -q "Prompt B1" INVENTORY.md || grep -q "Coupling to Ferrum" INVENTORY.md

echo "prove: docs OK"
