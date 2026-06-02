# rust-igraph

[![crates.io](https://img.shields.io/crates/v/rust-igraph.svg?label=crates.io)](https://crates.io/crates/rust-igraph)
[![docs.rs](https://img.shields.io/docsrs/rust-igraph?label=docs.rs)](https://docs.rs/rust-igraph)
[![CI](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml/badge.svg)](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Totoro-jam/rust-igraph/branch/main/graph/badge.svg)](https://codecov.io/gh/Totoro-jam/rust-igraph)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)

**Pure-Rust, high-performance graph and network analysis library.** A faithful port of
[igraph](https://igraph.org/) with 400+ public APIs, zero `unsafe`, and no C/C++ FFI.

Built for researchers, data scientists, and systems engineers who need production-grade
graph algorithms without leaving the Rust ecosystem.

## Why rust-igraph?

| | rust-igraph | petgraph | igraph (C/Python) |
|---|---|---|---|
| **Algorithm coverage** | 400+ APIs (BFS, DFS, shortest paths, community detection, centrality, isomorphism, flows, layouts, graph generators, 60+ graph class recognizers...) | ~30 algorithms | ~850 APIs |
| **Safety** | Zero `unsafe`, zero `unwrap` in library code | Some `unsafe` | C core with FFI |
| **Correctness** | Cross-validated against igraph C, python-igraph, and R-igraph test suites | Independent | Reference implementation |
| **Dependencies** | Minimal (1 runtime dep: `thiserror`) | Minimal | Large C/C++ toolchain |
| **WASM** | Designed for `wasm32-unknown-unknown` | Yes | No |

## Features

- **Traversal**: BFS, DFS, topological sort, random walks
- **Shortest paths**: Dijkstra, Bellman-Ford, A*, all-pairs, widest paths
- **Centrality**: betweenness, closeness, eigenvector, PageRank, HITS, harmonic, constraint
- **Community detection**: Louvain, Leiden, label propagation, Walktrap, edge betweenness, fast greedy, leading eigenvector, fluid communities, Voronoi
- **Connectivity**: connected/biconnected components, articulation points, bridges, separators, cohesive blocks, SCC
- **Network flow**: max-flow (push-relabel), min-cut, Gomory-Hu tree, edge/vertex connectivity, disjoint paths
- **Isomorphism**: VF2 (graph/subgraph), LAD subgraph, BLISS canonical labeling, automorphism groups
- **Graph generators**: Erdos-Renyi, Barabasi-Albert, Watts-Strogatz, SBM, forest fire, geometric random, degree sequence, lattices, famous graphs, and 30+ more
- **Graph properties**: 60+ structural recognizers (`is_bipartite`, `is_chordal`, `is_planar`, `is_perfect`, `is_cograph`, `is_series_parallel`, ...)
- **Eigenvalue solvers**: Lanczos (symmetric), Arnoldi (general), graph adjacency
- **Layout**: Fruchterman-Reingold, Kamada-Kawai, DrL, circle, star, grid, tree, bipartite
- **Spatial**: Delaunay triangulation, Gabriel graph, beta-skeleton, nearest-neighbor graph
- **I/O**: edge list, adjacency matrix, Prufer sequence, LCF notation

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-igraph = "0.0.1-alpha"
```

```rust
use rust_igraph::{Graph, bfs};

fn main() {
    // Build a small social network
    let mut g = Graph::with_vertices(6);
    g.add_edge(0, 1).unwrap(); // Alice - Bob
    g.add_edge(0, 2).unwrap(); // Alice - Carol
    g.add_edge(1, 3).unwrap(); // Bob - Dave
    g.add_edge(2, 4).unwrap(); // Carol - Eve
    g.add_edge(3, 5).unwrap(); // Dave - Frank

    // BFS from Alice
    let order = bfs(&g, 0).unwrap();
    println!("Visit order: {:?}", order);
}
```

### Community detection

```rust
use rust_igraph::{Graph, louvain};

let mut g = Graph::with_vertices(10);
// ... add edges forming two clusters ...
let result = louvain(&g).unwrap();
println!("Communities: {:?}", result.membership);
println!("Modularity: {:.4}", result.modularity);
```

### Centrality analysis

```rust
use rust_igraph::{Graph, pagerank, betweenness};

let mut g = Graph::with_vertices(5);
g.add_edge(0, 1).unwrap();
g.add_edge(1, 2).unwrap();
g.add_edge(2, 3).unwrap();
g.add_edge(3, 4).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
println!("PageRank: {:?}", pr);
println!("Betweenness: {:?}", bc);
```

## Performance

All algorithms are implemented in idiomatic Rust with careful attention to cache locality
and allocation patterns. Benchmarks (via `criterion`) are included for every major algorithm:

```bash
cargo bench --bench bench_louvain     # community detection
cargo bench --bench bench_bfs         # traversal
cargo bench --bench bench_max_flow    # network flow
cargo bench --bench bench_vf2         # isomorphism
```

## Project status

> **Alpha** (`v0.0.1-alpha`) — The API is stabilizing but may change before `v0.1.0`.
> Core algorithms are implemented and tested. Not yet recommended for production use
> without your own validation.

| Category | Status |
|----------|--------|
| Core data structures | Stable |
| Traversal (BFS, DFS) | Stable |
| Shortest paths | Stable |
| Centrality | Stable |
| Community detection | Stable |
| Connectivity | Stable |
| Network flow | Stable |
| Isomorphism | Stable |
| Graph generators | Stable |
| Layout algorithms | Beta |
| I/O formats | Partial |

## Development

```bash
cargo build                          # build
cargo test                           # fast test suite (660+ tests)
cargo test --all-features            # full suite with oracle + proptests
cargo clippy -- -D warnings          # lint
cargo doc --no-deps --open           # browse API docs locally
```

Each algorithm follows a 9-step **AWU** (Algorithm Work Unit) process tracked in
[`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md). The engineering
plan is in [`docs/plans/MASTER_PLAN.md`](docs/plans/MASTER_PLAN.md).

## License

GPL-2.0-or-later. Same as upstream igraph C, which permits direct reference-translation
of the C source. See [LICENSE](LICENSE).

## Acknowledgements

- [igraph](https://github.com/igraph/igraph) by the igraph team (C core)
- [python-igraph](https://github.com/igraph/python-igraph) and [rigraph](https://github.com/igraph/rigraph) (reference test suites)
- [faer](https://github.com/sarah-quinones/faer-rs) (linear algebra backend)
