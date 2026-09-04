# Helix vs HELIOS

Internal gate for feature decisions. If a proposal could live in either repo, it does not. Apply the table, then the rule. Do not “just add it to Helix” because VERIFY sounds like audit.

Canonical positioning: [HELIX_VISION.md](HELIX_VISION.md) §3. HELIOS stays a separate brand. It is not merged into Helix. HelixTest absorption is a different question ([HELIX_VISION.md](HELIX_VISION.md) §7) and does not move HELIOS work.

Neither green result is certification. Ferrum has no real clinical pilot; do not use either tool’s output as a production-deployment claim.

---

## 1. Ownership table

| Topic | Belongs to Helix | Belongs to HELIOS |
|-------|------------------|-------------------|
| API conformance | Yes. HelixTest heritage: does the GA4GH HTTP contract behave (DRS/WES/TES/TRS/htsget/Beacon, …). | No. |
| Security behaviour | Yes. Fail-closed API behaviour (HMAC/Passport/OIDC/Crypt4GH as exercised against a running target). | No. HELIOS does not replace ga4gh-infra or Ferrum auth tests. |
| Performance / regression | Yes. Repeatable scores, fail-level, Demo-docked runtime/resource compare ([HELIX_ROADMAP.md](HELIX_ROADMAP.md) Stages 2 and 4). | No. |
| Reproducibility | No. | Yes. Was this pipeline run attestably reproducible (Nextflow/Snakemake envelope). |
| Signed audit trails | No. | Yes. Signed evidence chain. |
| RO-Crate / PDF export | No. | Yes. HELIOS already exports RO-Crate 1.1 and PDF. |
| Compliance checklists ISO 15189 / EU AI Act | No. Helix does not map clauses, emit QMS checklists, or score accreditation. | **Orientation only.** HELIOS already has engineering mappings (`docs/compliance/iso15189.md`, `docs/compliance/ai_act.md`). A green HELIOS score is not ISO 15189 accreditation or an AI Act decision. The lab QMS still interprets. Do not grow a second checklist product in Helix. |

If a row is “No” for Helix, implementing it here is in scope-error, even as a “small export.”

---

## 2. Rule of thumb

**Testet es, OB etwas korrekt/sicher/performant funktioniert? → Helix.**
**Belegt es, WAS gelaufen ist und WIE es reproduziert werden kann? → HELIOS.**

English: Helix answers *whether* a running system behaves. HELIOS answers *what* ran and *how* to reproduce it.

Still unsure → default **neither**, or two artefacts that stay file-compatible (a Helix JSON report is not a HELIOS evidence pack unless HELIOS later ingests it as a file on the HELIOS side). Do not share a CLI, a report schema that pretends to be both, or a “compliance score” that mixes fail-level with signed provenance.

---

## 3. Helix run identity is not HELIOS evidence

Helix records a **lightweight verification-run identity** so two `helix verify` JSON files can be compared (`helix compare`). Fields: Helix version, HelixTest version, profile, test ids, target URL, fixture version, schema version, timestamp, and (for `helix bench` only) workload id/version. Spec: [RUN_IDENTITY.md](RUN_IDENTITY.md).

That identity answers: *are these two technical signals the same kind of measurement?* It does **not** answer: *what pipeline ran, who signed it, or how to reproduce it.*

| | Helix run identity | HELIOS |
|--|--------------------|--------|
| Purpose | Pair two verification results | Attest what ran and how to reproduce it |
| Signing | **No** | Yes (signed evidence chain) |
| RO-Crate / PDF | **No** | Yes |
| Scientific reproducibility | **No** | Yes (pipeline envelope) |
| Compare by check `id` | Yes (`NEW_FAIL`) | No |

Do not grow Helix identity into an evidence pack. Do not put HELIOS signatures on `VerificationRun`. A timestamp is a wall clock, not a signature.

---

## 4. Cross-link, keep separate

Both tools **must** point at each other in READMEs and identity docs so users are not sold the wrong question.

They **keep**:

| | Helix (and HelixTest until decided otherwise) | HELIOS |
|--|-----------------------------------------------|--------|
| GitHub repo | `SynapticFour/Helix` (CLI today: `SynapticFour/HelixTest`) | `SynapticFour/HELIOS` |
| CLI name | `helixtest` today; `helix verify` is Stage 1. Never `helios`. | `helios` / PyPI `helios-audit`. Never `helix`. |
| Pricing | Separate. Both are Apache-2.0 ambassadors today (not paid SKUs). If support/time is sold later, Helix and HELIOS stay distinct line items — not one licence that includes the other. | Same. |

Do not: merge repos, share a binary name, bundle prices, or put RO-Crate/PDF/sign/ISO-checklist work on the Helix roadmap.
