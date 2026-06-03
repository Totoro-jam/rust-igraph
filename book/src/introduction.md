# rust-igraph

A **pure-Rust port of [igraph](https://igraph.org)**, the network-analysis
library. Targets full API parity with igraph C v1.0.x (≈ 850 public
functions), validated continuously against three official implementations:

- **igraph C** — `tests/unit/*.c` + `*.out`
- **python-igraph** — `tests/test_*.py`
- **R-igraph** (`rigraph`) — `tests/testthat/test-*.R`

> **Status**: 305+ algorithm work units complete across Phases 1–6 and 9.
> 386 public APIs, 7,400+ tests (unit + integration + doctest), 110+
> runnable examples. WASM-compatible (`wasm32-unknown-unknown`). See
> [the master plan](../../docs/plans/MASTER_PLAN.md) for the roadmap, and
> [the algorithm tracker](../../.codefuse/tracking/ALGORITHMS.md) for
> per-algorithm progress.

## What's implemented

| Category | Highlights |
|----------|-----------|
| **Traversal** | BFS, DFS, shortest paths (Dijkstra, Bellman-Ford, Johnson, Floyd-Warshall, A*), widest paths |
| **Centrality** | PageRank, betweenness, closeness, harmonic, eigenvector, HITS, constraint |
| **Community** | Louvain, Leiden, label propagation, fluid communities, fast greedy, edge betweenness, walktrap, leading eigenvector, Voronoi |
| **Connectivity** | Components, articulation points, bridges, biconnected components, cohesive blocks |
| **Flow** | Max flow (Dinic), min cut, Gomory-Hu tree, vertex/edge connectivity, all s-t cuts |
| **Isomorphism** | VF2 (graph + subgraph), BLISS canonical labeling, LAD subgraph isomorphism |
| **Coloring** | Greedy vertex coloring (CN + DSatur), chordality, bipartite matching |
| **Constructors** | 40+ graph generators (Erdős-Rényi, Barabási-Albert, Watts-Strogatz, SBM, lattices, famous graphs, ...) |
| **I/O** | 8 formats: GML, GraphML, DOT, Pajek, NCOL, LGL, DL, LEDA — all attribute-aware |
| **Operators** | Union, intersection, difference, complement, simplify, permute, line graph |

## Why another Rust graph library?

`petgraph` is excellent for general-purpose graph work, but its API does
not express what `igraph_t` expresses (rich attributes, vertex/edge
selectors, full C API parity). Users coming from igraph's C, Python, or R
bindings need a Rust home where:

- function names mirror `igraph_*`
- numerical results match python-igraph within tight tolerance
- the build is WASM-friendly by default (no `unsafe`, no system deps)

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
