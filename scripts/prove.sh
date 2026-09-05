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
  docs/FOR-EVALUATORS.md docs/INSTALL.md docs/HELIX_VISION.md docs/HELIX_ROADMAP.md \
  docs/HELIX_VS_HELIOS.md docs/DECISIONS.md docs/CLI_CONTRACT.md VERSIONS.lock
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
grep -q "Helix is HelixTest becoming a standalone VERIFY CLI. The binary is \`helix\`. Not HELIOS. Not GA4GH certification. Ferrum is a reference target, not a dependency." README.md
grep -q "docs/STATUS.md" README.md
grep -q "clinical pilot" docs/IDENTITY.md
grep -q "Prompt B1" INVENTORY.md || grep -q "Coupling to Ferrum" INVENTORY.md
grep -q "Synaptic Four builds the infrastructure. Helix proves it works." docs/HELIX_VISION.md
grep -q "HELIOS stays a separate brand" docs/HELIX_VISION.md || grep -q "not merged into Helix" docs/HELIX_VISION.md
grep -q "Stage 0" docs/HELIX_ROADMAP.md
grep -q "helix verify" docs/HELIX_ROADMAP.md
grep -q "Helix Cloud" docs/HELIX_ROADMAP.md
grep -q "Testet es, OB etwas korrekt/sicher/performant funktioniert?" docs/HELIX_VS_HELIOS.md
grep -q "helios-audit" docs/HELIX_VS_HELIOS.md
grep -q "Keep HelixTest as its own git root" docs/DECISIONS.md
grep -q "helix verify" docs/CLI_CONTRACT.md
grep -q "HELIXTEST_SHA=1832c043e1679ec283cb2113510ee33684317cce" VERSIONS.lock
grep -q "fn discover" src/discover.rs
grep -q "run_drs_checks" src/verify.rs
grep -q "fn overall_report" src/report.rs
grep -q "fn run_security" src/security/mod.rs
grep -q "fn run_bench" src/bench/mod.rs
grep -q "helix bench" docs/CLI_CONTRACT.md
grep -q "NICHT FÜR PRODUKTION" test-fixtures/README.md

echo "prove: docs OK"
