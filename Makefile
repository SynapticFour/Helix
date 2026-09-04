# Helix — docs and VERIFY CLI (no local stack)

.PHONY: help prove test

help:
	@echo "Helix — independence of HelixTest (Synaptic Four GA4GH stack)"
	@echo ""
	@echo "  make prove     Zero-risk proof: docs + cargo test (DRS + security + bench mocks; no Ferrum)"
	@echo "  make test      cargo test"
	@echo ""
	@echo "Live target you started:"
	@echo "  cargo run --bin helix -- verify http://127.0.0.1:8080"
	@echo "  cargo run --bin helix -- security http://127.0.0.1:8080"
	@echo "  cargo run --bin helix -- bench --baseline http://127.0.0.1:8080 --candidate http://127.0.0.1:8080"

# Zero-risk customer path. Live Ferrum proof: docs/PROVE.md
prove:
	chmod +x scripts/prove.sh
	./scripts/prove.sh
	$(MAKE) test
	@echo "Helix prove OK."

test:
	cargo test
