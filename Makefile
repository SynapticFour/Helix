# Helix — VERIFY CLI (HelixTest wrap). Not HELIOS. Not certification.

.PHONY: help prove test test-live verify-fixture install

# HelixTest HttpClient defaults to debug traces. Evaluators need the report, not GET dumps.
RUST_LOG ?= error
export RUST_LOG

help:
	@echo "Helix — GA4GH VERIFY CLI (HelixTest becoming a standalone binary). Not HELIOS."
	@echo ""
	@echo "  make prove            Docs + cargo test (in-process fixtures; no Ferrum)"
	@echo "  make verify-fixture   helix verify against the in-process DRS fixture (no Ferrum)"
	@echo "  make test             cargo test --locked --all-targets"
	@echo "  make install          cargo install --path . --locked (needs sibling HelixTest)"
	@echo "  make test-live        helix verify against HELIX_LIVE_URL (you started the stack)"
	@echo ""
	@echo "First run: docs/FOR-EVALUATORS.md and docs/INSTALL.md"
	@echo "Fixtures: docs/FIXTURES.md. Live Ferrum: docs/PROVE.md (optional)"

# Zero-risk Helix core: honesty docs + all crate tests on deterministic fixtures.
# Does not start Ferrum/Docker. Does not skip, ignore, or exclude tests.
# Live HTTP against a stack you control is make test-live, not this target.
prove:
	chmod +x scripts/prove.sh scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	./scripts/prove.sh
	$(MAKE) test
	@echo "Helix prove OK (in-process fixtures; not Ferrum, not certification)."

test:
	chmod +x scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	cargo test --locked --all-targets

# helix verify against docs/FIXTURES.md §1. Not Ferrum. Not certification.
verify-fixture:
	chmod +x scripts/require-helixtest.sh
	./scripts/require-helixtest.sh
	cargo run --locked --example verify-fixture

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
	cargo run --quiet --bin helix -- verify "$(HELIX_LIVE_URL)" --format json
