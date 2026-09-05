# Helix — VERIFY CLI (HelixTest wrap). Not HELIOS. Not certification.

.PHONY: help prove test test-live verify-fixture install fetch independent-verify

# HelixTest HttpClient defaults to debug traces. Evaluators need the report, not GET dumps.
RUST_LOG ?= error
export RUST_LOG

help:
	@echo "Helix — DRS/WES VERIFY CLI wrapping HelixTest. Not HELIOS. Not GA4GH certification."
	@echo ""
	@echo "  make fetch                 cargo fetch --locked (network; crate checksums, not latest GA4GH)"
	@echo "  make prove                 Docs + cargo test --locked --offline (in-process fixtures; no Ferrum)"
	@echo "  make independent-verify    Registry hashes + reproducibility tests (offline)"
	@echo "  make verify-fixture        helix verify against the in-process DRS fixture (no Ferrum)"
	@echo "  make test                  cargo test --locked --offline --all-targets"
	@echo "  make install               cargo install --path . --locked (needs sibling HelixTest)"
	@echo "  make test-live             helix verify against HELIX_LIVE_URL (you started the stack)"
	@echo "  helix matrix               interop matrix (pending without independent runs; see docs/INTEROP.md)"
	@echo ""
	@echo "First run: docs/FOR-EVALUATORS.md, docs/INSTALL.md, docs/INDEPENDENT_VERIFICATION.md"
	@echo "Fixtures: docs/FIXTURES.md. Live Ferrum: docs/PROVE.md (optional)"

# crates.io at Cargo.lock checksums. Explicit network. Not a GA4GH download.
fetch:
	cargo fetch --locked

# Zero-risk Helix core: honesty docs + all crate tests on deterministic fixtures.
# Does not start Ferrum/Docker. Does not skip, ignore, or exclude tests.
# Live HTTP against a stack you control is make test-live, not this target.
# Does not fetch crates; run make fetch first if --offline fails.
prove:
	chmod +x scripts/prove.sh scripts/require-helixtest.sh scripts/independent-verify.sh
	./scripts/require-helixtest.sh
	./scripts/prove.sh
	$(MAKE) test
	@echo "Helix prove OK (in-process fixtures; not Ferrum, not certification)."

test:
	chmod +x scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	@if ! cargo test --locked --offline --all-targets; then \
		echo "cargo test --locked --offline failed. If crates are missing: make fetch (Cargo.lock, network)." >&2; \
		exit 1; \
	fi

independent-verify:
	chmod +x scripts/independent-verify.sh scripts/require-helixtest.sh
	./scripts/independent-verify.sh

# helix verify against docs/FIXTURES.md §1. Not Ferrum. Not certification.
verify-fixture:
	chmod +x scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	cargo run --locked --offline --example verify-fixture

install:
	chmod +x scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	cargo install --path . --locked --force

# Opt-in. Never invoked by prove or GitHub CI. Requires a running origin you started.
test-live:
	@if [ -z "$(HELIX_LIVE_URL)" ]; then \
		echo "test-live: set HELIX_LIVE_URL to a running GA4GH origin (e.g. cd ../Ferrum && make up)." >&2; \
		echo "This target is not part of make prove. First path: make verify-fixture." >&2; \
		exit 2; \
	fi
	cargo run --quiet --locked --bin helix -- verify "$(HELIX_LIVE_URL)" --format json
