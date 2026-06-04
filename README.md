# rust-igraph

[![crates.io](https://img.shields.io/crates/v/rust-igraph.svg?label=crates.io)](https://crates.io/crates/rust-igraph)
[![docs.rs](https://img.shields.io/docsrs/rust-igraph?label=docs.rs)](https://docs.rs/rust-igraph)
[![CI](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml/badge.svg)](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Totoro-jam/rust-igraph/branch/main/graph/badge.svg)](https://codecov.io/gh/Totoro-jam/rust-igraph)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Pure-Rust, high-performance graph and network analysis library.** A faithful port of
[igraph](https://igraph.org/) with 1200+ public APIs, zero `unsafe`, and no C/C++ FFI.

Built for researchers, data scientists, and systems engineers who need production-grade
graph algorithms without leaving the Rust ecosystem.

## Why rust-igraph?

| | rust-igraph | petgraph | igraph (C/Python) |
|---|---|---|---|
| **Algorithm coverage** | 1200+ APIs (BFS, DFS, shortest paths, community detection, centrality, isomorphism, flows, layouts, graph generators, 60+ graph class recognizers...) | ~50 (composable) | ~850 APIs (reference) |
| **Safety** | Zero `unsafe`, zero `unwrap` in library code | Minimal `unsafe` | C core + bindings |
| **Correctness** | Cross-validated against igraph C, python-igraph, and R-igraph test suites | Independent | Reference implementation |
| **Dependencies** | Minimal (1 runtime dep: `thiserror`) | Minimal | Large C/C++ toolchain |
| **WASM** | Designed for `wasm32-unknown-unknown` | Yes | No |

## Features

- **Traversal**: BFS, DFS, topological sort, random walks
- **Shortest paths**: Dijkstra, Bellman-Ford, A*, all-pairs, widest paths
- **Centrality**: betweenness, closeness, eigenvector, PageRank, HITS, Katz, harmonic, constraint
- **Community detection**: Louvain, Leiden, label propagation, Walktrap, edge betweenness, fast greedy, leading eigenvector, fluid communities, Voronoi
- **Connectivity**: connected/biconnected components, articulation points, bridges, separators, cohesive blocks, SCC
- **Network flow**: max-flow (push-relabel), min-cut, Gomory-Hu tree, edge/vertex connectivity, disjoint paths
- **Isomorphism**: VF2 (graph/subgraph), LAD subgraph, BLISS canonical labeling, automorphism groups
- **Graph generators**: Erdos-Renyi, Barabasi-Albert, Watts-Strogatz, SBM, forest fire, geometric random, degree sequence, lattices, famous graphs, and 30+ more
- **Graph properties**: 60+ structural recognizers (`is_bipartite`, `is_chordal`, `is_planar`, `is_perfect`, `is_cograph`, `is_series_parallel`, ...)
- **Eigenvalue solvers**: Lanczos (symmetric), Arnoldi (general), graph adjacency
- **Layout**: Fruchterman-Reingold, Kamada-Kawai, DrL, Sugiyama, GEM, Davidson-Harel, GraphOpt, MDS, LGL, UMAP, Reingold-Tilford, circle, star, grid, bipartite (16 engines, 2D+3D)
- **Spatial**: Delaunay triangulation, Gabriel graph, beta-skeleton, nearest-neighbor graph
- **I/O**: GML, GraphML, Pajek, DOT/Graphviz, LEDA, UCINET DL, DIMACS, edge list, NCOL, LGL, GraphDB (15 read/write functions)

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-igraph = "0.5"
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

### Graph construction

```rust
use rust_igraph::{Graph, GraphBuilder};

// Fluent builder pattern
let g = GraphBuilder::undirected()
    .vertices(5)
    .edges(&[(0,1), (1,2), (2,3), (3,4)])
    .cycle(&[0, 1, 2, 3, 4])
    .build()
    .unwrap();

// From an edge list (auto-infers vertex count)
let g = Graph::from_edges(&[(0,1), (1,2), (2,0)], false, None).unwrap();

// From a slice via TryFrom
let g = Graph::try_from(vec![(0u32, 1), (1, 2), (2, 0)].as_slice()).unwrap();

// From a string (great for tests)
let g = Graph::from_edge_list_str("0 1\n1 2\n2 0").unwrap();

// Classic generators
let k5 = rust_igraph::full_graph(5, false, false).unwrap();
let ring = rust_igraph::cycle_graph(10, false, false).unwrap();
```

### Graph algebra (operator overloading)

```rust
use rust_igraph::Graph;

let a = Graph::from_edges(&[(0,1), (1,2)], false, None).unwrap();
let b = Graph::from_edges(&[(1,2), (2,0)], false, None).unwrap();

let union = &a | &b;          // edges in either graph
let intersection = &a & &b;   // edges in both graphs
let difference = &a - &b;     // edges in a but not b
let complement = !&a;          // all missing edges
let disjoint = &a + &b;       // concatenated (6 vertices)
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
use rust_igraph::{Graph, pagerank, betweenness, katz_centrality};

let g = Graph::from_edges(
    &[(0,1), (1,2), (2,3), (3,4)], false, None
).unwrap();

let pr = pagerank(&g).unwrap();
let bc = betweenness(&g).unwrap();
let katz = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
println!("PageRank: {:?}", pr);
println!("Betweenness: {:?}", bc);
println!("Katz: {:?}", katz);
```

### Method-style API

The most common operations are available directly on `Graph`:

```rust
use rust_igraph::Graph;

let g = Graph::from_edges(
    &[(0,1), (1,2), (2,0), (2,3), (3,4), (4,5), (5,3)],
    false, None
).unwrap();

// Structural queries
assert!(g.is_connected().unwrap());
println!("Diameter: {:?}", g.diameter().unwrap());
println!("Girth: {:?}", g.girth().unwrap());
println!("Triangles: {}", g.count_triangles().unwrap());

// Centrality
let pr = g.pagerank().unwrap();
let bc = g.betweenness().unwrap();
let hc = g.harmonic_centrality().unwrap();

// Community detection
let communities = g.louvain().unwrap();
println!("Modularity: {:.4}", communities.modularity);

// Graph construction
let er = Graph::erdos_renyi(1000, 0.01, 42).unwrap();
let ws = Graph::watts_strogatz(1000, 6, 0.1, 42).unwrap();
let ba = Graph::barabasi_albert(1000, 3, 42).unwrap();
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

> **v0.5.0** — 306 algorithm work units complete, 687 public functions,
> 7,574 tests. API stabilizing toward `v1.0.0`.

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
| Layout algorithms | Stable (16 engines) |
| I/O formats | Stable (15 functions) |
| Spatial algorithms | Stable |

## Documentation

- **[Tutorial & Guide](https://Totoro-jam.github.io/rust-igraph/book/)** — mdBook with getting started, cookbook, and architecture overview
- **[API Reference](https://Totoro-jam.github.io/rust-igraph/rust_igraph/)** — full rustdoc for all 1,200+ public items
- **[docs.rs](https://docs.rs/rust-igraph)** — auto-published on every crates.io release

## Development

```bash
cargo build                          # build
cargo test                           # fast test suite (7,574 tests)
cargo test --all-features            # full suite with oracle + proptests
cargo clippy -- -D warnings          # lint
cargo doc --no-deps --open           # browse API docs locally
```

Each algorithm follows a 9-step **AWU** (Algorithm Work Unit) process tracked in
[`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md). The engineering
plan is in [`docs/plans/MASTER_PLAN.md`](docs/plans/MASTER_PLAN.md).

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Whether you want to fix a bug, add an algorithm, improve docs, or optimize performance —
we'd love your help. Open an issue to discuss larger changes before starting.

## License

GPL-2.0-or-later. Same as upstream igraph C, which permits direct reference-translation
of the C source. See [LICENSE](LICENSE).

## Acknowledgements

- [igraph](https://github.com/igraph/igraph) by the igraph team (C core)
- [python-igraph](https://github.com/igraph/python-igraph) and [rigraph](https://github.com/igraph/rigraph) (reference test suites)
