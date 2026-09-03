# Helix — docs and (later) conformance runner (no local stack)

.PHONY: help prove

help:
	@echo "Helix — independence of HelixTest (Synaptic Four GA4GH stack)"
	@echo ""
	@echo "  make prove     Zero-risk proof: required docs + honesty strings"
	@echo ""
	@echo "Helix does not deploy servers. Live suite is still HelixTest:"
	@echo "  cd ../HelixTest && make prove"
	@echo "  cd ../Ferrum && make up"
	@echo "  helixtest --all --mode ferrum"

# Zero-risk customer path. Live Ferrum proof: docs/PROVE.md
prove:
	chmod +x scripts/prove.sh
	./scripts/prove.sh
	@echo "Helix prove OK."
