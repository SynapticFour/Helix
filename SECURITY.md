# Security Policy

## Reporting a Vulnerability

Please do **not** open public GitHub issues for security vulnerabilities.

Report vulnerabilities privately to **contact@synapticfour.com** with:
- affected repository and version/commit
- reproduction steps or proof-of-concept
- impact assessment

We will acknowledge receipt as quickly as possible, triage severity, and coordinate a responsible disclosure timeline.

## Scope and Guarantees

This project is maintained on a best-effort basis (single-steward). Security documentation and test coverage improve over time, but no absolute security guarantee is provided.

Helix is a VERIFY CLI, not a security product. How Helix behaves as an HTTP client (untrusted targets, redaction, redirects, size limits) is documented in [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md). Helix must never accidentally print secrets or `Authorization` header values. Reproducibility / signed evidence stays in HELIOS (`helios-audit`).
