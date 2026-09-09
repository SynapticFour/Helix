#!/usr/bin/env bash
# OPTIONAL LIVE VERIFICATION: two independent DRS 1.4.0 origins the operator started.
# Orchestrates existing `helix verify` / `inspect` / `differential` only.
# Not part of make prove. Does not pull Docker images. Does not restamp local/b12/.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

STARTER_URL="${HELIX_STARTER_KIT_URL:-http://127.0.0.1:4500}"
BENTO_URL="${HELIX_BENTO_URL:-http://127.0.0.1:5000}"
STARTER_OBJECT="${HELIX_STARTER_OBJECT_ID:-b8cd0667-2c33-4c9f-967b-161b905932c9}"
BENTO_OBJECT="${HELIX_BENTO_OBJECT_ID:-}"
BENTO_SHA="${HELIX_BENTO_SHA256:-6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1}"
OUT="${HELIX_INDEPENDENT_OUT:-$ROOT/local/independent}"

usage() {
  cat >&2 <<'EOF'
OPTIONAL LIVE VERIFICATION

make verify-independent does not start Docker, does not pull images, and is
not part of make prove. Start the two reviewed DRS implementations yourself.

  docs/INDEPENDENT_DRS.md

Required:
  HELIX_BENTO_OBJECT_ID   UUID printed by `flask ingest` on your Bento database

Optional:
  HELIX_STARTER_KIT_URL    default http://127.0.0.1:4500
  HELIX_BENTO_URL          default http://127.0.0.1:5000
  HELIX_STARTER_OBJECT_ID  default object id shipped in the pinned Starter Kit image
  HELIX_BENTO_SHA256       digest of the ingested blob (4096 ASCII A if you followed the doc)
  HELIX_INDEPENDENT_OUT   default local/independent  (must not be local/b12)
                          writes starter-kit-current.json and bento-current.json

Those files are newly produced. Do not copy or restamp local/b12/.
helix inspect classifies standing. It does not rewrite the JSON.

A missing daemon, image, QEMU support, or stopped process is a setup failure.
It is not a DRS verification FAIL.
EOF
}

if [[ -z "$BENTO_OBJECT" ]]; then
  usage
  echo "error: set HELIX_BENTO_OBJECT_ID to the UUID flask ingest printed." >&2
  exit 2
fi

case "$OUT" in
  *local/b12* | *local/b12)
    echo "error: HELIX_INDEPENDENT_OUT must not be local/b12 (historical evidence; do not restamp)." >&2
    exit 2
    ;;
esac

probe() {
  local name="$1"
  local url="$2"
  local probe_url="${url%/}/ga4gh/drs/v1/service-info"
  if ! command -v curl >/dev/null 2>&1; then
    echo "OPTIONAL LIVE VERIFICATION: curl is required to check that ${name} is reachable." >&2
    echo "Helix will not guess that a target is running. Install curl, or start the target and retry." >&2
    echo "This is not a verification FAIL." >&2
    exit 2
  fi
  if ! curl -sf --max-time 3 "$probe_url" >/dev/null; then
    echo "OPTIONAL LIVE VERIFICATION: ${name} is not reachable at ${url}" >&2
    echo "Start that implementation first (docs/INDEPENDENT_DRS.md)." >&2
    echo "Helix does not start Docker. Unreachable is not a DRS verification FAIL." >&2
    exit 2
  fi
}

probe "GA4GH Starter Kit DRS" "$STARTER_URL"
probe "Bento DRS" "$BENTO_URL"

chmod +x "$ROOT/scripts/require-helixtest.sh"
"$ROOT/scripts/require-helixtest.sh"

mkdir -p "$OUT"
STARTER_JSON="$OUT/starter-kit-current.json"
BENTO_JSON="$OUT/bento-current.json"

helix() {
  cargo run --locked --offline --quiet --bin helix -- "$@"
}

run_verify() {
  set +e
  NO_COLOR=1 helix verify "$@"
  local ec=$?
  set -e
  # 0 = overall pass (not VERIFIED). 1 = recorded FAIL/ERROR/skip-only. Both write --output.
  if [[ "$ec" -eq 0 || "$ec" -eq 1 ]]; then
    return 0
  fi
  echo "helix verify exited ${ec} (usage or runtime). That is not a DRS check outcome." >&2
  exit "$ec"
}

echo "Verifying Starter Kit at ${STARTER_URL} → ${STARTER_JSON}"
run_verify "$STARTER_URL" \
  --standard drs --version 1.4.0 \
  --target-id ga4gh-starter-kit-drs-0.3.2 \
  --target-kind real-independent-local-implementation \
  --implementation-name ga4gh-starter-kit-drs \
  --implementation-version 0.3.2 \
  --drs-object-id "$STARTER_OBJECT" \
  --output "$STARTER_JSON"
echo "Inspect (does not rewrite). Standing is computed; current is not VERIFIED."
NO_COLOR=1 helix inspect "$STARTER_JSON"

echo "Verifying Bento at ${BENTO_URL} → ${BENTO_JSON}"
run_verify "$BENTO_URL" \
  --standard drs --version 1.4.0 \
  --target-id bento-drs-0.21.5 \
  --target-kind real-independent-local-implementation \
  --implementation-name bento_drs \
  --implementation-version 0.21.5 \
  --drs-object-id "$BENTO_OBJECT" \
  --drs-object-sha256 "$BENTO_SHA" \
  --output "$BENTO_JSON"
echo "Inspect (does not rewrite). Standing is computed; current is not VERIFIED."
NO_COLOR=1 helix inspect "$BENTO_JSON"

echo "Differential (does not rank implementations)"
NO_COLOR=1 helix differential "$STARTER_JSON" "$BENTO_JSON"
