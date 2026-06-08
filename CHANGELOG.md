# Changelog

All notable changes to **rust-igraph** are recorded here.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning 2.0](https://semver.org/spec/v2.0.0.html).

> Pre-1.0 contract: every minor bump (0.x.y → 0.(x+1).0) may break the
> public API. Patch bumps are bug-fixes / new additive items only.

> Maintenance: when cutting a tag, rename `[Unreleased]` to
> `[x.y.z] — YYYY-MM-DD` and open a fresh empty `[Unreleased]` block
> above it. See `.codefuse/tracking/RELEASE.md` for the full checklist.

## [Unreleased]

## [0.7.0] — 2026-06-08

### Added
- **Hierarchical Random Graphs** (ALGO-HRG-001..003) — `HrgTree` data
  structure, `hrg_create` / `from_hrg_dendrogram`, `hrg_sample` /
  `hrg_sample_many` / `hrg_game`, `hrg_fit` / `hrg_consensus` /
  `hrg_predict` with MCMC tree-space search.
- **Planarity testing** (ALGO-PR-007) — `is_planar` via Left-Right
  Planarity Test (Brandes 2009).
- **BLISS canonical labeling engine** (ALGO-ISO-003..007) — pure-Rust
  individualization-refinement (I-R) engine for canonical permutation,
  automorphism counting/group, and BLISS-backed isomorphism with vertex
  colour support.
- **WASM bindings expanded to 404** — `@graphrs/igraph-wasm` now exposes
  the full algorithm surface: paths, centrality, community detection,
  isomorphism, flow, generators, layouts, I/O, spectral, HRG, properties,
  and operators. Published to npm via Trusted Publishing (OIDC).
- **Playground expanded to 90+ interactive algorithms** — from 41 to 90+
  demos covering all major algorithm families.
- **Playground dogfooding** — now depends on the real `@graphrs/igraph-wasm`
  npm package instead of building WASM from source. CI copies the published
  binary from `node_modules`.
- **npm Trusted Publishing (OIDC)** — `@graphrs/igraph-wasm` publishes
  with zero static tokens via GitHub Actions OIDC + npm provenance
  signatures. Requires Node 24 (npm 11.5.1+).
- **Bilingual READMEs** — separate `README.md` (English) and
  `README.zh-CN.md` (Chinese) with language switcher links for both the
  root repo and the igraph-wasm crate.
- **Laplacian matrix** (ALGO-LAP-001) and **spectral embeddings**
  (ALGO-EMB-001..003) — adjacency spectral embed, Laplacian spectral
  embed, dimension selection.
- **Chinese cookbook** — full translation of all 13 English cookbook recipes.
- **BLISS tutorial section** — added to both English and Chinese tutorials.
- **Isomorphism example** — `examples/isomorphism_demo.rs` demonstrating
  VF2, BLISS, canonical permutation, automorphism group, and colour-aware
  isomorphism.

### Changed
- **I-R engine performance** — replaced O(|Aut(G)|²) group closure with
  O(n) orbit-based generator collection; flat buffer in refinement loop
  eliminates per-vertex heap allocations. K_8 canonicalization: 79ms → 55ms
  (30% faster), C_30 automorphisms: 1.7ms → 1.0ms (41% faster).
- **igraph-wasm crate refactored** — split monolithic `lib.rs` into 11
  domain modules with `pub(crate)` visibility.
- Pages CI no longer requires `wasm-pack` or Rust toolchain for playground
  WASM (only for rustdoc build).
- Website visual design overhaul — Observatory aesthetic.

### Fixed
- React key collision in playground result table causing DOM reuse bugs
  when algorithms return repeated vertex ids (random walk, paths).
- `isomorphism_demo.rs` used `ring_graph(n, false, false, false)` (path)
  instead of `ring_graph(n, false, false, true)` (cycle).
- 8 duplicate `wasm_bindgen` symbol exports in igraph-wasm removed.
- Clippy lints for Rust 1.96 resolved across all crates.

## [0.6.0] — 2026-06-04

### Added
- **Infomap community detection** (ALGO-CO-018) — two-level map equation
  minimization with merging + single-node moves.
- **Spinglass community detection** (ALGO-CO-019) — Potts model simulated
  annealing with gamma-tunable resolution.
- **Playground rebuilt as React SPA** — CodeMirror 6 editor, D3
  force-directed canvas, Web Worker WASM integration.
- **WASM expansion to 20 algorithms** — first batch of igraph-wasm bindings.
- **mdBook bilingual support** — Chinese (zh) tutorial with language switcher.

### Fixed
- WASM deployment path, canvas flash, dark mode contrast, CI lockfile.

## [0.5.0] — 2026-06-04

Initial feature-complete release covering 269 AWUs. Highlights:

- **Graph core** — `Graph` with incidence-list storage, `GraphBuilder`,
  `TryFrom`/`FromIterator`/`Extend` ergonomics, operator overloads
  (`+`/`|`/`&`/`-`/`!`), `PartialEq`/`Hash`, iterators.
- **Attribute system** — vertex/edge/graph attributes with automatic
  maintenance during structural mutations.
- **I/O** — 8 formats (GraphML, GML, DOT, Pajek, NCOL, LGL, DL, LEDA)
  with attribute round-trip support.
- **Traversal** — BFS, DFS, random walk, topological sort.
- **Shortest paths** — Dijkstra, Bellman-Ford, Johnson, Floyd-Warshall,
  A*, widest paths, diameter/eccentricity/radius.
- **Centrality** — betweenness, closeness, eigenvector, PageRank, HITS,
  Katz, harmonic, constraint, strength. Cutoff and weighted variants.
- **Community detection** — Louvain, Leiden, label propagation, fast
  greedy, Walktrap, edge betweenness, leading eigenvector, fluid, Voronoi.
- **Connectivity** — weak/strong components, biconnected components,
  articulation points, bridges, cohesive blocks, reachability, Eulerian.
- **Flow** — push-relabel max-flow, Gomory-Hu tree, min-cut, all-st-cuts,
  edge/vertex connectivity, disjoint paths.
- **Isomorphism** — VF2 (graph + subgraph), LAD subgraph isomorphism.
- **Generators** — 30+ models (Erdos-Renyi, Barabasi-Albert, Watts-Strogatz,
  SBM, lattice, tree, famous graphs, degree sequence, etc.).
- **Graph properties** — 60+ structural recognizers (`is_bipartite`,
  `is_chordal`, `is_planar`, `is_perfect`, `is_cograph`, etc.).
- **Layouts** — 16 engines (FR, KK, Sugiyama, GEM, DH, GraphOpt, DrL,
  MDS, UMAP, LGL, Reingold-Tilford, circle, star, grid, bipartite).
- **Operators** — union, intersection, difference, complement, compose,
  permute, contract, simplify, reverse, line graph, graph power.
- **Cliques & motifs** — clique number, maximal cliques, independence
  number, triad census, motif search.
- **Eigenvalue solvers** — Lanczos (symmetric), Arnoldi (general).
- **417 convenience methods on `Graph`** — method-style API delegating
  to free functions for the most common operations.
- **mdBook tutorial** — graph construction, centrality, community, paths,
  attributes, I/O, isomorphism, operators, iteration patterns.
- **116 runnable examples** in `examples/`.

> For the full 7,000+ line development history of v0.5.0, see
> `git log v0.4.0..v0.5.0` or the [v0.5.0 release notes][0.5.0-release].

[Unreleased]: https://github.com/Totoro-jam/rust-igraph/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/Totoro-jam/rust-igraph/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Totoro-jam/rust-igraph/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Totoro-jam/rust-igraph/releases/tag/v0.5.0
[0.5.0-release]: https://github.com/Totoro-jam/rust-igraph/releases/tag/v0.5.0
