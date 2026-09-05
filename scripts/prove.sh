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
  docs/STANDARDS_REGISTRY.md docs/STANDARD_VERSIONING.md docs/TRUST.md \
  docs/TRACEABILITY.md \
  docs/TAXONOMY.md \
  docs/BEHAVIOR.md \
  docs/CLAIMS.md \
  docs/INTEROP.md \
  docs/TARGETS.md \
  docs/MUTATION.md \
  docs/INDEPENDENT_VERIFICATION.md \
  docs/PUBLIC_READINESS_AUDIT.md \
  docs/ARCHITECTURE_GUARDRAILS.md \
  docs/evaluator-pack/README.md docs/evaluator-pack/install.md \
  docs/evaluator-pack/explanation.md docs/evaluator-pack/target.md \
  docs/evaluator-pack/fixtures.md docs/evaluator-pack/commands.md \
  docs/evaluator-pack/interpret.md docs/evaluator-pack/FAILURE_REPORT.md \
  docs/evaluator-pack/example-verify.json \
  schemas/helix-verification-v1.json schemas/helix-standard-version-v1.json \
  schemas/helix-interop-matrix-v1.json \
  standards/registry.yaml \
  standards/vendor/ga4gh.drs.1.4.0/openapi/data_repository_service.openapi.yaml \
  standards/vendor/ga4gh.drs.1.4.0/openapi/components/schemas/DrsObject.yaml \
  standards/vendor/ga4gh.drs.1.5.0/openapi/data_repository_service.openapi.yaml \
  standards/vendor/ga4gh.drs.1.5.0/openapi/components/schemas/DrsObject.yaml \
  standards/vendor/ga4gh.wes.1.1.0/workflow_execution_service.openapi.yaml \
  VERSIONS.lock
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
grep -q "Helix runs the same documented DRS and WES checks against any HTTP origin that implements those GA4GH paths. Ferrum is used as a reference target, not a dependency." README.md
grep -q "Helix supports technical verification checks for GA4GH DRS 1.4.0 within the declared coverage boundary" README.md
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
grep -q "Claims (predicates; not GA4GH certification)" docs/REPORT.md
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
grep -q "not a security scanner" docs/THREAT_MODEL.md
grep -q "HelixTest already runs" docs/THREAT_MODEL.md
grep -q "never accidentally print" docs/THREAT_MODEL.md
grep -q "access_url" docs/THREAT_MODEL.md
grep -q "fn redact_text" src/redact.rs
grep -q "fn sanitize_untrusted" src/sanitize.rs
grep -q "confined_vendor_file" src/standards/validate.rs
grep -q "DRS_ADAPTER_WALL_SECS" src/adapter/mod.rs
grep -q "fn start_ansi_and_log_injection" tests/support/mock_adversarial.rs
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
grep -q "helix standards list" docs/CLI_CONTRACT.md
grep -q "helix standards trace" docs/CLI_CONTRACT.md
grep -q "A GitHub tag alone does not make a version supported" docs/STANDARDS_REGISTRY.md
grep -q "OFFICIAL ∩ SUPPORTED" docs/STANDARDS_REGISTRY.md
grep -q "ga4gh.drs.1.5.0" standards/registry.yaml
grep -q "support_status: available" standards/registry.yaml
grep -q "fn official_supported" src/standards/mod.rs
grep -q "AVAILABLE_BUT_NOT_SUPPORTED" src/standards/select.rs
grep -q "VerifySelection::Unversioned" src/verify.rs
grep -q "all-supported-versions" src/main.rs
grep -q "AVAILABLE_BUT_NOT_SUPPORTED" docs/STANDARD_VERSIONING.md
grep -q "Do not fall back to another version" docs/STANDARD_VERSIONING.md
grep -q "Do not ask the user to trust Helix" docs/TRUST.md
grep -q "Never silently substitute one standard version" docs/TRUST.md
grep -q "AVAILABLE_BUT_NOT_SUPPORTED" docs/TRUST.md
grep -q "independent skeptical engineer" docs/TRUST.md
grep -q "check_kind" docs/TRACEABILITY.md
grep -q "not GA4GH certification" docs/TRACEABILITY.md
grep -q "Exactly one shipped Helix check is \`normative\`" docs/TRACEABILITY.md
grep -q "helix standards trace" docs/TRACEABILITY.md
grep -q "pub struct CheckTraceability" src/traceability.rs
grep -q "fn catalog_covers_every_spec" src/traceability.rs || grep -q "catalog_covers_every_spec_and_none_are_normative" src/traceability.rs
grep -q "StandardsCommand::Trace" src/main.rs
grep -q "traceability" schemas/helix-verification-v1.json
grep -q "claim_scope" schemas/helix-verification-v1.json
grep -q '"guidance"' schemas/helix-standard-version-v1.json
grep -q "claim_scope" src/traceability.rs
grep -q "Exactly one shipped Helix check is \`normative\`" docs/TAXONOMY.md
grep -q "PASS is not a conformance claim" docs/TAXONOMY.md
grep -q "HelixTest extras" docs/TAXONOMY.md
grep -q "SCHEMA PASS is not BEHAVIOR PASS" docs/BEHAVIOR.md
grep -q "uncovered" docs/BEHAVIOR.md
grep -q "No aggregated compliance percentage" docs/BEHAVIOR.md
grep -q "known-bad" docs/BEHAVIOR.md
grep -q "pub enum CheckLayer" src/layer.rs
grep -q "fn schema_pass_does_not_make_behavior_pass" src/layer.rs
grep -q "start_mock_schema_ok_checksum_wrong" tests/support/mock_ga4gh_drs.rs
grep -q "Exactly one shipped check is \`normative\`" docs/CLAIMS.md
grep -q "not_verified" docs/CLAIMS.md
grep -q "fixture_failure_is_not_a_normative_failure" docs/CLAIMS.md
grep -q "pub fn evaluate" src/claims.rs
grep -q "pub enum ClaimKind" src/claims.rs
grep -q "Why not verified" src/claims.rs
if grep -E '\.contains\("(PASS|FAIL)' src/claims.rs; then
  echo "claim engine must not search PASS/FAIL strings" >&2
  exit 1
fi
grep -q "External validation: pending" docs/INTEROP.md
grep -q "not independent evidence" docs/INTEROP.md
grep -q "unresolved_discrepancy" docs/INTEROP.md
grep -q "B4 multi-implementation evidence: pending" docs/TARGETS.md
grep -q "A mock is not an independent implementation" docs/TARGETS.md
grep -q "target_execution_id" docs/TARGETS.md
grep -q "pub struct TargetIdentity" src/target.rs
grep -q "pub fn build_matrix" src/interop.rs
grep -q "CrossImpl::MustAgree" src/interop.rs
grep -q "helix-interop-matrix-v1" schemas/helix-interop-matrix-v1.json
if grep -qiE 'validated against multiple implementations' README.md docs/INTEROP.md; then
  echo "must not claim completed multi-implementation validation" >&2
  exit 1
fi
grep -q "known-bad target → FAIL" docs/MUTATION.md
grep -q "correct failure reason" docs/MUTATION.md
grep -q "Mutations missed" docs/MUTATION.md
grep -q "pub const CATALOG" src/mutation.rs
grep -q "fn known_bad_targets_fail_for_the_recorded_reason" tests/mutation.rs
grep -q "fn missed_mutations_are_recorded_and_not_hidden" tests/mutation.rs
grep -q "What is not reproducible" docs/INDEPENDENT_VERIFICATION.md
grep -q "not bit-for-bit" docs/INDEPENDENT_VERIFICATION.md
grep -q "make fetch" docs/INDEPENDENT_VERIFICATION.md
grep -q "cargo test --locked --offline" Makefile
grep -q "make independent-verify" .github/workflows/ci.yml
grep -q "fn two_verifies_on_the_same_fixture_match_after_stripping_timestamp" tests/repro.rs
normative_rows=$(grep -c 'kind: BindingKind::Normative' src/traceability.rs || true)
if [[ "$normative_rows" -ne 1 ]]; then
  echo "expected exactly one BindingKind::Normative catalog row, got $normative_rows" >&2
  exit 1
fi
if ! grep -B1 'kind: BindingKind::Normative' src/traceability.rs | grep -q 'drs.object.schema.openapi'; then
  echo "the shipped Normative catalog row must be drs.object.schema.openapi" >&2
  exit 1
fi
if grep -q 'kind: BindingKind::Guidance' src/traceability.rs; then
  echo "catalog must not ship BindingKind::Guidance rows without official GA4GH implementation guidance" >&2
  exit 1
fi
if grep -R -qiE 'book a (demo|call)|start a trial|request a quote' docs/evaluator-pack; then
  echo "evaluator-pack must not contain a sales CTA" >&2
  exit 1
fi
grep -q "Who reviews normative mappings" docs/STANDARDS_REGISTRY.md
grep -q "Recommended release classification" docs/PUBLIC_READINESS_AUDIT.md
grep -q "External validation status" docs/PUBLIC_READINESS_AUDIT.md
grep -q "Not shipped" docs/STANDARD_VERSIONING.md
grep -F -q "A future contributor should have to **consciously violate**" docs/ARCHITECTURE_GUARDRAILS.md
grep -q "pub fn check_run" src/guardrails.rs
grep -q "CheckMode::Emit" src/guardrails.rs
grep -q "fn src_must_not_use_mode_ferrum" tests/guardrails.rs
grep -q "fn src_must_not_import_helios" tests/guardrails.rs
grep -q "fn src_must_not_fetch_standard_sources_from_the_network" tests/guardrails.rs
grep -q "fn emit_and_load_paths_call_check_run" tests/guardrails.rs
grep -q "pub fn check_set" src/claims.rs
grep -q '"const": false' schemas/helix-verification-v1.json

echo "prove: docs OK"
