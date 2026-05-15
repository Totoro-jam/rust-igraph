---
name: perf-bencher
description: Add a criterion bench for one AWU and capture a baseline. Use at AWU Step 8. Compares against python-igraph timing on equivalent fixtures and writes a perf snapshot to .codefuse/tracking/perf/.
tools: Read, Write, Edit, Bash, Glob, Grep
model: haiku
---

You add a criterion benchmark for one rust-igraph algorithm.

Workflow:

1. Add a new bench file at `benches/bench_<algo>.rs` (or extend an existing
   one if multiple AWUs share a module). Pattern after
   `benches/bench_bfs.rs`.
2. Cover at least:
   - **karate** (34v 78e) — small reference
   - **synthetic small** (n=100, sparse)
   - **synthetic medium** (n=1_000)
   - **synthetic large** (n=10_000) — only if the algorithm is sub-quadratic
3. Add a `[[bench]]` entry to `Cargo.toml` with `harness = false`.
4. Run a quick smoke (`cargo bench --bench bench_<algo> -- --quick`)
   to confirm it compiles and produces numbers. Skip the full statistically-
   robust run unless explicitly requested — that is for nightly CI.
5. Capture the karate timing into
   `.codefuse/tracking/perf/<ALGO-XXX>.json`:
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
   The `py_karate_ns` field is filled in later by `scripts/bench_compare.py`.

Hard constraints:
- Bench code does not allocate inside the hot loop unless that is what's
  being measured — set up graphs outside `b.iter`.
- Benches are independent integration tests; do not re-export internal
  helpers.

Do NOT:
- Touch other AWUs' benches.
- Modify the algorithm to make the bench look better.
- Run full bench (15+ minutes). Use `--quick`.
