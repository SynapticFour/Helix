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
  docs/HELIX_VS_HELIOS.md docs/DECISIONS.md docs/CLI_CONTRACT.md \
  docs/ARCHITECTURE.md docs/VERIFICATION_MODEL.md docs/TEST_IDENTITY.md \
  docs/DISCOVERY.md docs/HELIXTEST_ADAPTER.md docs/DRS_PROFILE.md docs/WES.md \
  docs/PROFILES.md docs/FIXTURES.md docs/REGRESSION.md docs/SECURITY_PROFILE.md docs/CRYPT4GH.md \
  docs/BENCHMARKS.md docs/DIAGNOSTICS.md docs/REPORT.md docs/SCHEMA.md docs/THREAT_MODEL.md \
  docs/EVALUATOR_JOURNEY.md docs/EXTERNAL_TARGET_CONTRACT.md \
  docs/RUN_IDENTITY.md docs/OPEN_SOURCE_RELEASE_CHECKLIST.md \
  docs/evaluator-pack/README.md docs/evaluator-pack/install.md \
  docs/evaluator-pack/explanation.md docs/evaluator-pack/target.md \
  docs/evaluator-pack/fixtures.md docs/evaluator-pack/commands.md \
  docs/evaluator-pack/interpret.md docs/evaluator-pack/FAILURE_REPORT.md \
  docs/evaluator-pack/example-verify.json \
  schemas/helix-verification-v1.json VERSIONS.lock
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
grep -q "Helix tests behavior against the GA4GH spec, independent of implementation. Ferrum is used as a reference target, not a dependency." README.md
grep -q "clinical pilot" docs/IDENTITY.md
grep -q "Prompt B1" INVENTORY.md || grep -q "Coupling to Ferrum" INVENTORY.md
grep -q "Synaptic Four builds the infrastructure. Helix proves it works." docs/HELIX_VISION.md
grep -q "HELIOS stays a separate brand" docs/HELIX_VISION.md || grep -q "not merged into Helix" docs/HELIX_VISION.md
grep -q "Stage 0" docs/HELIX_ROADMAP.md
grep -q "helix verify" docs/HELIX_ROADMAP.md
grep -q "Helix Cloud" docs/HELIX_ROADMAP.md
grep -q "Testet es, OB etwas korrekt/sicher/performant funktioniert?" docs/HELIX_VS_HELIOS.md
grep -q "helios-audit" docs/HELIX_VS_HELIOS.md
grep -q "Helix run identity is not HELIOS evidence" docs/HELIX_VS_HELIOS.md
grep -F -q "| Signing | **No** |" docs/HELIX_VS_HELIOS.md
grep -F -q "| RO-Crate / PDF | **No** |" docs/HELIX_VS_HELIOS.md
grep -F -q "| Scientific reproducibility | **No** |" docs/HELIX_VS_HELIOS.md
grep -q "Not a signed audit trail" docs/RUN_IDENTITY.md
grep -q "Not scientific reproducibility" docs/RUN_IDENTITY.md
grep -q "helix-fixtures-v1" docs/RUN_IDENTITY.md
grep -q "Do not add \`signature\`" docs/RUN_IDENTITY.md
grep -q "pub struct RunIdentity" src/run_identity.rs
grep -q "FIXTURE_VERSION" src/model.rs
grep -q "fixture_version" schemas/helix-verification-v1.json
grep -q "helix-fixtures-v1" schemas/helix-verification-v1.json
if grep -qiE 'Helix (signs|exports RO-Crate|emits a PDF)' docs/RUN_IDENTITY.md docs/HELIX_VS_HELIOS.md; then
  echo "run identity must not claim HELIOS features" >&2
  exit 1
fi
grep -q "Do not tag. Do not publish. Do not announce from this file." docs/OPEN_SOURCE_RELEASE_CHECKLIST.md
grep -q "CODE_OF_CONDUCT" docs/OPEN_SOURCE_RELEASE_CHECKLIST.md
grep -q "Not ready for an external public announcement" docs/OPEN_SOURCE_RELEASE_CHECKLIST.md
grep -q "Keep HelixTest as its own git root" docs/DECISIONS.md
grep -q "helix verify" docs/CLI_CONTRACT.md
grep -q "API compatibility contract" docs/CLI_CONTRACT.md
grep -q "must not write those lines to stdout" docs/CLI_CONTRACT.md
grep -q "helix bench" docs/CLI_CONTRACT.md
grep -q "drs.object.not_found" docs/VERIFICATION_MODEL.md
grep -q "HLX-DRS-005" docs/VERIFICATION_MODEL.md
grep -q "Skip cannot be stored or serialized as pass" docs/VERIFICATION_MODEL.md
grep -q "pub struct VerificationRun" src/model.rs
grep -q "compatibility change" docs/TEST_IDENTITY.md
grep -q "HLX-DRS-005" docs/TEST_IDENTITY.md
grep -q "DRS invalid object id returns 404" docs/TEST_IDENTITY.md
grep -q "pub const SPECS" src/identity.rs
grep -q "HELIXTEST_SHA=1832c043e1679ec283cb2113510ee33684317cce" VERSIONS.lock
grep -q "fn discover" src/discover.rs
grep -q "NOT_DETECTED" docs/DISCOVERY.md
grep -q "DETECTED is not a pass" docs/DISCOVERY.md
grep -q "TESTABLE" docs/DISCOVERY.md
grep -q "Never print" docs/DISCOVERY.md
grep -q "DETECTED is not a pass" docs/DRS_PROFILE.md
grep -q "profile" docs/DRS_PROFILE.md
grep -q "HLX-DRS-001" docs/DRS_PROFILE.md
grep -q "never convert SKIP into PASS" docs/HELIXTEST_ADAPTER.md
grep -q "does not import Ferrum" docs/HELIXTEST_ADAPTER.md
grep -q "separate git root" docs/HELIXTEST_ADAPTER.md
grep -q "v0.1.3" docs/HELIXTEST_ADAPTER.md
grep -q "run_drs_checks" src/adapter/mod.rs
grep -q "run_wes_checks" src/adapter/mod.rs
grep -q "HLX-WES-001" docs/WES.md
grep -q "supports_scatter_gather=false" docs/WES.md
grep -q "trs://test-tool/echo/1.0" docs/WES.md
grep -q "Never choose a profile from service-info" docs/PROFILES.md
grep -q "Mode::Generic" docs/PROFILES.md
grep -q "pub const GENERIC" src/profile.rs
grep -q "pub const FERRUM" src/profile.rs
grep -q "No real secrets" docs/FIXTURES.md
grep -q "start_mock_ga4gh_drs" docs/FIXTURES.md
grep -q "make prove" docs/FIXTURES.md
grep -q "Intentionally invalid" docs/FIXTURES.md
grep -q "start_malformed_json" docs/FIXTURES.md
grep -q "malformed is not PASS" docs/FIXTURES.md
grep -q "fn start_malformed_json" tests/support/mock_adversarial.rs
grep -q "fn start_connection_reset" tests/support/mock_adversarial.rs
grep -q "helix-dummy-hmac-not-for-production-do-not-use" test-fixtures/hmac/shared-secret.txt
if grep -R --include='*.rs' -n 'Mode::Ferrum' src; then
  echo "Helix src must not use Mode::Ferrum; profiles map to Features only" >&2
  exit 1
fi
grep -q "wired in \`helix verify\`" docs/TEST_IDENTITY.md
if grep -q "use framework" src/verify.rs; then
  echo "verify.rs must not import framework::*; use src/adapter" >&2
  exit 1
fi
grep -q "fn verify_json" src/report.rs
grep -q "pub const DRS_PROFILE" src/model.rs
grep -q "D3 revisit" docs/DECISIONS.md
grep -q "fn run_security" src/security/mod.rs
grep -q "fn run_bench" src/bench/mod.rs
grep -q "helix bench" docs/CLI_CONTRACT.md
grep -q "http.drs.smoke.v1" docs/BENCHMARKS.md
grep -q "http.drs.smoke.v1" src/bench/workload.rs
grep -q "not a significance test" docs/BENCHMARKS.md
grep -q "do not fail CI" docs/BENCHMARKS.md
grep -q "GIAB-scale" docs/BENCHMARKS.md
grep -q "fn measure" src/bench/engine.rs
grep -q "P95_MIN_REPETITIONS" src/bench/stats.rs
grep -q "fn analyze" src/bench/analysis.rs
grep -q "Performance changed enough to merit human inspection" docs/BENCHMARKS.md
grep -q "Implementation is incorrect" docs/BENCHMARKS.md
grep -q "verification_failure" src/bench/analysis.rs
grep -q "MIN_DISTRIBUTION_RUNS" src/bench/analysis.rs
if grep -i 'statistically significant' docs/BENCHMARKS.md docs/CLI_CONTRACT.md; then
  echo "bench docs must not claim statistical significance" >&2
  exit 1
fi
grep -q "NICHT FÜR PRODUKTION" test-fixtures/README.md
grep -q "NEW_FAIL" docs/REGRESSION.md
grep -q "SKIP must not silently become PASS" docs/REGRESSION.md
grep -q "helix compare" docs/REGRESSION.md
grep -q "overall score decreased" docs/REGRESSION.md
grep -q "pub fn classify" src/compare.rs
grep -q "fn compare_runs" src/compare.rs
grep -q "not a penetration test" docs/SECURITY_PROFILE.md
grep -F -q "Passing these checks does **not** prove that the implementation is secure" docs/SECURITY_PROFILE.md
grep -q "HLX-AUTH-010" docs/SECURITY_PROFILE.md
grep -q "VerifierPolicy" docs/SECURITY_PROFILE.md
grep -q "pub const HTTP_CASES" src/security/profile.rs
grep -q "SECURITY_BEHAVIOR_DISCLAIMER" src/security/profile.rs
grep -q "fn classify_bearer_with" src/security/jwt.rs
grep -q "What Helix verifies" docs/CRYPT4GH.md
grep -q "What Helix does not verify" docs/CRYPT4GH.md
grep -q "does not mean Crypt4GH is secure" docs/CRYPT4GH.md
grep -q "HLX-AUTH-050" docs/CRYPT4GH.md
grep -q "HLX-AUTH-053" docs/CRYPT4GH.md
grep -q "HLX-AUTH-054" docs/CRYPT4GH.md
grep -F -q "It does not add the \`crypt4gh\` crate" docs/CRYPT4GH.md
grep -q "fn run_crypt4gh_cases" src/security/crypt4gh_header.rs
grep -q "HLX-AUTH-053" src/identity.rs
grep -q "HLX-AUTH-054" src/identity.rs
if grep -E '^crypt4gh\s*=' Cargo.toml; then
  echo "Helix must not depend on the crypt4gh crate (secret-key path stays in HelixTest)" >&2
  exit 1
fi
grep -q "Possible causes" docs/DIAGNOSTICS.md
grep -q "HLX-DRS-005" docs/DIAGNOSTICS.md
grep -q "fn diagnose" src/diagnostics.rs
grep -q "not claiming a root cause" src/diagnostics.rs
grep -q "possible_causes" src/diagnostics.rs
grep -q "possible causes:" src/report.rs
if grep -q 'println!("        Cause:' src/report.rs; then
  echo "text report must print possible causes, not Cause:" >&2
  exit 1
fi
grep -q "HELIX VERIFICATION" docs/REPORT.md
grep -q "This is a technical verification signal" docs/REPORT.md
grep -q "It is not GA4GH certification" docs/REPORT.md
grep -q "Not compared" docs/REPORT.md
grep -q "fn format_verify_text" src/report.rs
grep -q "fn format_compare_text" src/report.rs
if grep -qi 'ro-crate' docs/REPORT.md; then
  :
else
  echo "REPORT.md must mention RO-Crate as out of scope" >&2
  exit 1
fi
grep -q "helix-verification-v1" schemas/helix-verification-v1.json
grep -q "Backwards compatibility" docs/SCHEMA.md
grep -q "schema_version" src/model.rs
grep -q "SCHEMA_VERSION" src/model.rs
grep -q "possible_causes" schemas/helix-verification-v1.json
if grep -q '"cause"' schemas/helix-verification-v1.json; then
  echo "v1 diagnostic schema must not define cause" >&2
  exit 1
fi
grep -q "not a security product" docs/THREAT_MODEL.md
grep -q "HelixTest already runs" docs/THREAT_MODEL.md
grep -q "never accidentally print" docs/THREAT_MODEL.md
grep -q "access_url" docs/THREAT_MODEL.md
grep -q "fn redact_text" src/redact.rs
grep -q "redirect(Policy::none())" src/http_safety.rs
grep -q "MAX_RESPONSE_BYTES" src/http_safety.rs
grep -q "userinfo" src/discover.rs
if ! grep -E '^reqwest\s*=' Cargo.toml | grep -q 'default-features = false'; then
  echo "Helix-owned reqwest must keep default-features = false (no gzip bomb on this client)" >&2
  exit 1
fi
if grep -q "not yet a runnable suite" docs/FOR-EVALUATORS.md; then
  echo "FOR-EVALUATORS.md must not claim the repo is unrunnable" >&2
  exit 1
fi
grep -q "make verify-fixture" docs/FOR-EVALUATORS.md
grep -q "make prove" docs/FOR-EVALUATORS.md
grep -q "Not HELIOS" docs/FOR-EVALUATORS.md
grep -q "Open a GitHub Issue" docs/FOR-EVALUATORS.md
grep -q "make verify-fixture" Makefile
grep -q "verify-fixture" examples/verify_fixture.rs
grep -q "Cloning Helix alone does not build" docs/EVALUATOR_JOURNEY.md
grep -q "default_client_log_filter" src/lib.rs
grep -q "RUST_LOG=error" docs/CLI_CONTRACT.md
grep -q "make verify-fixture" docs/IDENTITY.md
grep -q "make verify-fixture" docs/INSTALL.md
grep -q "make verify-fixture" docs/PROVE.md
grep -q "make verify-fixture" .github/workflows/ci.yml
grep -q "implementation-neutral" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "The target must not need" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "Standard requirements" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "Fixture requirements" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "Optional capabilities" docs/EXTERNAL_TARGET_CONTRACT.md
if grep -qi "must implement Ferrum" docs/EXTERNAL_TARGET_CONTRACT.md; then
  echo "EXTERNAL_TARGET_CONTRACT.md must not require Ferrum" >&2
  exit 1
fi
grep -q "not this contract" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "multipart/form-data" docs/EXTERNAL_TARGET_CONTRACT.md
grep -q "does not send telemetry" docs/evaluator-pack/README.md
grep -q "No Synaptic Four account" docs/evaluator-pack/README.md
grep -q "make verify-fixture" docs/evaluator-pack/commands.md
grep -q "helix-verification-v1" docs/evaluator-pack/example-verify.json
grep -q "Helix failure report" docs/evaluator-pack/FAILURE_REPORT.md
grep -q "What Helix is (one page)" docs/evaluator-pack/explanation.md
if grep -R -qiE 'book a (demo|call)|start a trial|request a quote' docs/evaluator-pack; then
  echo "evaluator-pack must not contain a sales CTA" >&2
  exit 1
fi

echo "prove: docs OK"
