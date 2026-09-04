# Helix benchmarks

HelixTest already runs (CI, SF-TR-2026-001/002). Helix productizes a **repeatable measurement engine** around a fixed HTTP smoke workload. It does not replace Demo hap.py, GIAB concordance, or HELIOS evidence.

`helix bench` records client-side timings and compares **distributions** from repeated measurements. It is **not** a publication benchmark. Sample percentiles are not a significance test. Thresholds **do not fail CI**.

---

## Measurement, warning, regression

These are **separate**. A bench result is not a `helix verify` result.

| Concept | Means | Does not mean |
|---------|--------|----------------|
| **Measurement** | A recorded series of repeated runs and the distribution deltas computed from them. Not a verdict. | That the candidate is better or worse. That CI should fail. |
| **Warning** | Performance changed enough to merit human inspection. | **Implementation is incorrect.** A verification FAIL. A red CI X. |
| **Regression** | The candidate **median latency distribution** is worse than baseline beyond the inspect threshold, from a repeated-measurement compare. | A verification failure (`helix compare` `NEW_FAIL`). That the implementation is incorrect. Statistical significance. |

`helix compare` **regression** is PASS→FAIL at a stable check id ([REGRESSION.md](REGRESSION.md)). A bench **regression** is a different word. They are not interchangeable.

`analysis.verification_failure` is always `false`. A bench warning must not become a verification failure. The process still exits 0. helix-action must not fail the job on `warning: true`.

Single wall-clock samples are **not** compared as a distribution. Both series need at least 2 measured runs (`MIN_DISTRIBUTION_RUNS`). `--repetitions 1` is measurement only.

---

## Workload

| | |
|--|--|
| ID | `http.drs.smoke.v1` |
| Version | `1` (the `v1` suffix; bump the id when the request set changes) |
| Requests | `GET /health`, `GET /ga4gh/drs/v1/service-info`, `GET /ga4gh/drs/v1/objects/test-object-1` |
| Scale | Three small GETs (same *count* as Demo DRS micro `n=3`) |

This is the only shipped workload. **GIAB-scale and hap.py workloads are out of scope.** Demo remains the GIAB-slice smoke ([Ferrum-GA4GH-Demo](https://github.com/SynapticFour/Ferrum-GA4GH-Demo)); Helix does not ingest it.

Identities for those three GETs: `bench.get.health` / `HLX-BENCH-001` … `003` ([TEST_IDENTITY.md](TEST_IDENTITY.md)).

---

## Measurement engine

Each target is measured independently, then the two **series** are analysed.

1. **Warm-up runs** — execute the fixed workload `warmup` times and **discard** the results (default `1`).
2. **Measured runs** — execute it `repetitions` times (default `5`). Only these enter stats.
3. **Aggregate** that series (median / min / max / sample p95 at ≥20).
4. **Analyse** the two distributions (`src/bench/analysis.rs`).

```text
helix bench --baseline <url> --candidate <url>
helix bench --baseline <url> --candidate <url> --warmup 1 --repetitions 5 --format json
helix bench --baseline <url> --candidate <url> --repetitions 20   # sample p95 included
```

`--no-rss` skips the optional resource metric.

---

## What the analysis reports

Always (when the metric exists on both series):

| Change | Source |
|--------|--------|
| **median** | `latency.median_ms` vs `latency.median_ms` |
| **p95** | `latency.p95_ms` vs `latency.p95_ms` when **both** have ≥20 measured runs; otherwise omitted |
| **error-rate** | series `error_rate` vs `error_rate` |
| **resource** | median per-run Helix-process `rss_kb` when **both** series recorded RSS; otherwise omitted |

JSON: `analysis.changes[]` plus compatibility `diff` (`wall_ms` is the median change). `analysis.warning` / `analysis.regression` / `analysis.verification_failure`.

A **warning** fires if any available change is worse than `--threshold` (default 10%) on a **comparable** distribution compare. A **regression** fires only when the **median** change is that warning. A p95-only or error-rate-only or RSS-only shift is a warning, not a bench regression.

---

## What Helix records

Per target (`baseline` / `candidate` `Sample` + `metadata`):

| Field | Meaning |
|-------|---------|
| `helix_version` | Helix crate version (`CARGO_PKG_VERSION`) |
| `workload_id` / `workload_version` | `http.drs.smoke.v1` / `1`. Run identity for this measurement ([RUN_IDENTITY.md](RUN_IDENTITY.md)). Not HELIOS |
| `target_url` / `target_label` | Origin Helix reached / CLI label |
| `timestamp` | RFC 3339 UTC when that sample started |
| `os` / `arch` | `std::env::consts` |
| `runtime` | MSRV, HTTP request/connect timeouts (5s / 3s), RSS source |
| `warmup` / `repetitions` | Config actually used |
| `latency.median_ms` / `min_ms` / `max_ms` | Sample stats of **measured** wall times |
| `latency.p95_ms` | Sample 95th percentile when `repetitions >= 20`; otherwise omitted |
| `wall_ms` | Same as median (stable field for helix-action comments) |
| `error_rate` | Failed requests / total measured requests (non-2xx or transport error) |
| `bytes` | Sum of response body lengths over measured runs |
| `rss_kb` | Optional. Peak Linux `VmRSS` of **this Helix process** (`/proc/self/status`), not Ferrum. Absent on macOS/Windows or with `--no-rss` |
| `runs[]` | Each measured run (wall_ms, bytes, errors, optional rss_kb) |

`warning: true` on the outcome is true if `analysis.warning` **or** the environments are incomparable. It does **not** change process exit. After a successful run the CLI exits **0**.

Metric ids: `HLX-BENCH-010`–`016` ([TEST_IDENTITY.md](TEST_IDENTITY.md)).

---

## Limitations

- **Not a statistical test.** Median / p95 / percent change are sample statistics of this series. No t-test, confidence interval, or “significant” finding.
- **Not a single-sample compare.** One measured run per side is recorded and is not a distribution warning or bench regression.
- **Small n is noisy.** Default `repetitions` is 5. p95 is omitted below 20. That floor is not a significance test.
- **Client-side only.** Wall time includes Helix’s HTTP client, not Ferrum’s internal queues alone.
- **RSS is the Helix process** on Linux, not the target’s RSS or CPU.
- **Cross-environment diffs are marked.** Different OS/arch/Helix version/workload/HTTP timeouts → `environment.comparable: false`. Percent diffs are shown; warning/regression **from the threshold** do not fire. `outcome.warning` may still be true so a human notices the mark.
- **Not a verification failure.** Does not fail `helix verify`. Does not fail CI. Does not mean the implementation is incorrect.
- **Not GIAB / hap.py / clinical throughput / production performance.**
- **Not HELIOS** signed evidence, RO-Crate, or PDF.
- **Not GA4GH certification.**

Stage 4 is **started, not exited**: two Ferrum versions on the same runner class are still the roadmap exit ([HELIX_ROADMAP.md](HELIX_ROADMAP.md)).

---

## Environment marking

Live `helix bench` measures both URLs in **one process**, so `environment.comparable` is typically true (`basis` names Helix version, workload, OS, arch, timeouts).

Do not paste two JSON files from different machines into one narrative without that mark. The engine refuses to treat mismatched OS/arch/runtime as an unmarked compare.

---

## CI

`make prove` uses in-process mocks ([FIXTURES.md](FIXTURES.md) §15) and **deterministic synthetic distributions** in `src/bench/analysis.rs`. It does **not** fail on a bench threshold. helix-action appends a warn-only section; `bench-warning` is informational.

Do not add a required check that turns `warning: true` into a red X or a verification FAIL.

---

## Source

`src/bench/` — `engine.rs` (warmup + measured), `analysis.rs` (distribution compare), `stats.rs`, `metadata.rs`, `workload.rs` (`http.drs.smoke.v1`), `rss.rs`. CLI: `helix bench` ([CLI_CONTRACT.md](CLI_CONTRACT.md)).
