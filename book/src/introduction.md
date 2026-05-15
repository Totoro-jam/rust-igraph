# rust-igraph

A **pure-Rust port of [igraph](https://igraph.org)**, the network-analysis
library. Targets full API parity with igraph C v1.0.x (≈ 850 public
functions), validated continuously against three official implementations:

- **igraph C** — `tests/unit/*.c` + `*.out`
- **python-igraph** — `tests/test_*.py`
- **R-igraph** (`rigraph`) — `tests/testthat/test-*.R`

> **Status**: Phase 0 complete (walking skeleton + AI-assisted SOP +
> CI/CD). Phase 1 (full data structures) is the next milestone. See
> [the master plan](../../docs/plans/MASTER_PLAN.md) for the roadmap, and
> [the algorithm tracker](../../.codefuse/tracking/ALGORITHMS.md) for
> per-algorithm progress.

## Why another Rust graph library?

`petgraph` is excellent for general-purpose graph work, but its API does
not express what `igraph_t` expresses (rich attributes, vertex/edge
selectors, full C API parity). Users coming from igraph's C, Python, or R
bindings need a Rust home where:

- function names mirror `igraph_*`
- numerical results match python-igraph within tight tolerance
- the build is WASM-friendly by default

## License

GPL-2.0-or-later, matching upstream igraph. The
[architecture decision record](../../.codefuse/tracking/ARCHITECTURE.md)
explains why.

## How this site is built

`mdBook` builds the prose sections; `cargo doc` builds the API rustdoc; CI
publishes both to GitHub Pages. To build locally:

```bash
cargo install mdbook
mdbook serve --open
```
