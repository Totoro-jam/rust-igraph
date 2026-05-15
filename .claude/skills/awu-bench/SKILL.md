---
name: awu-bench
description: Add a criterion benchmark for one AWU and capture a baseline JSON snapshot. Step 8 of the SOP. Use when user says "bench ALGO-...", "add benchmark", or after `/awu-conformance` passes.
---

# /awu-bench ALGO-XXX-NNN

Step 8 of the AWU SOP: capture a performance baseline so future regressions
are detected automatically.

## Workflow

### 1. Delegate to `perf-bencher`

The agent adds `benches/bench_<algo>.rs` (or extends an existing module),
registers a `[[bench]]` entry in `crates/igraph/Cargo.toml`, and runs a
`--quick` smoke.

### 2. Capture the baseline

After the smoke succeeds, write `.codefuse/tracking/perf/<ALGO-XXX>.json`:

```json
{
  "awu": "ALGO-XXX-NNN",
  "rust_karate_ns": <number>,
  "rust_synthetic_n1000_ns": <number>,
  "py_karate_ns": null,
  "captured_at": "<ISO date>",
  "git_commit": "<short sha>"
}
```

`py_karate_ns` is filled in later by `scripts/bench_compare.py` once the
script is written (BOOT-27).

### 3. Acceptance bar — explicit

A bench is "good enough to merge" if it:
- Compiles with `cargo build --benches`
- Produces stable timings (criterion does this for us)
- The karate result is within a sensible order of magnitude — typically
  ≤ 100 µs

A bench is **not** required to beat python-igraph yet. Phase 0 sets
baselines; performance work is a later, separate effort. If Rust is more
than 10× slower than python-igraph on the same fixture, file a `perf-todo`
issue and continue — do not block the AWU.

### 4. Hand off

Tell the user:

> Baseline captured at `.codefuse/tracking/perf/<ALGO-XXX>.json`. Next:
> `/awu-finish ALGO-XXX-NNN`
