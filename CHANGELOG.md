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

### Added
- **BLISS canonical labeling engine** (ALGO-ISO-003..007) — pure-Rust
  individualization-refinement (I-R) engine for canonical permutation,
  automorphism counting/group, and BLISS-backed isomorphism with vertex
  colour support.
- **WASM expansion to 55 algorithms** — igraph-wasm crate now exposes
  Dijkstra, topological sort, max flow, graph stats, articulation points,
  bridges, degree sequence, SCC, vertex coloring, transitivity, edge
  betweenness centrality, triad census, canonical permutation, count
  automorphisms, BLISS isomorphism, diameter, shortest path, random walk,
  graph generators (Erdos-Renyi, Barabasi-Albert, full, cycle, ring,
  Watts-Strogatz), and layout engines (circle, random, grid, star,
  Kamada-Kawai).
- **Playground: 4 new algorithm categories** — triad census in Structure,
  plus new Isomorphism section with canonical permutation, count
  automorphisms, and BLISS isomorphism check.
- **Playground test suite expanded** — 109 tests (was 98), covering
  all 4 new algorithm demo functions and dispatch.
- **Chinese cookbook** — full translation of all 13 English cookbook
  recipes (was a stub).
- **Chinese WASM tutorial** — isomorphism API section, triad census,
  updated algorithm count to 41.
- **BLISS tutorial section** — added to both English and Chinese tutorials.
- **Isomorphism example** — `examples/isomorphism_demo.rs` demonstrating
  VF2, BLISS, canonical permutation, automorphism group, and colour-aware
  isomorphism (116 examples total).
- **Chinese tutorial sync** — stream-based File I/O example (write_gml /
  read_gml with attribute round-trip) and circle / Kamada-Kawai layout
  examples, matching English tutorial content.

### Changed
- **I-R engine performance** — replaced O(|Aut(G)|²) group closure with
  O(n) orbit-based generator collection; flat buffer in refinement loop
  eliminates per-vertex heap allocations. K_8 canonicalization: 79ms → 55ms
  (30% faster), C_30 automorphisms: 1.7ms → 1.0ms (41% faster).
- Test count updated across all docs: 8,386 → 8,494.
- Example count updated across all docs: 115 → 116.
- Landing page stats synced to current values (8,494 tests).

### Fixed
- `isomorphism_demo.rs` used `ring_graph(n, false, false, false)` (path)
  instead of `ring_graph(n, false, false, true)` (cycle), causing
  incorrect automorphism counts in the example output.

## [0.6.0] — 2026-06-04

### Added
- **Infomap community detection** (ALGO-CO-018) — full AWU with
  two-level map equation minimization, merging + single-node moves,
  module codelength tracking. 7 unit + 3 proptest + 3 conformance.
- **Spinglass community detection** (ALGO-CO-019) — Potts model
  simulated annealing with gamma-tunable resolution and positive/negative
  spin coupling. 7 unit + 3 proptest + 3 conformance.
- **Playground rebuilt as React SPA** — CodeMirror 6 editor, D3
  force-directed canvas with zoom/pan/drag, resizable panels,
  collapsible sections, structured results output with stat badges
  and sortable table.
- **WASM expansion to 20 algorithms** — igraph-wasm crate now exposes:
  PageRank, Louvain, betweenness, closeness, eigenvector, BFS, DFS,
  connected components, Infomap, Spinglass, label propagation, Walktrap,
  Leiden, fast greedy, leading eigenvector, edge betweenness community,
  fluid communities, harmonic centrality, HITS, Katz centrality.
- **Playground Vitest test suite** — 98 tests covering all 20 demo
  fallback algorithms, layout, and dispatch.
- **Landing page enhancements** — community-colored hero animation,
  SVG icons, entrance animations, live algorithm counter, responsive
  design overhaul.
- **mdBook bilingual support** — Chinese (zh) tutorial with language
  switcher toggle.

### Changed
- Playground upgraded from vanilla JS to React SPA architecture with
  Web Worker WASM integration and graceful JS demo fallback.
- Landing page stats updated to reflect actual project counts.
- v0.6.0 roadmap updated: B3-B5 algorithms complete, B6-B10 layouts
  already done.

### Fixed
- WASM deployment path so playground loads real algorithms in production.
- Canvas flash on first load and WASM auto-run infinite loop.
- Node color contrast in dark mode playground.
- CI package-lock.json regenerated with public npm registry.
- Hero canvas node rescaling on window resize.

## [0.5.0] — 2026-06-04

### Added
- **Attribute-aware I/O** — all eight graph formats now read and write
  vertex, edge, and graph-level attributes through the `Graph` attribute
  system:
  - **GraphML**: reads `<key>` definitions and `<data>` elements; writes
    typed `<key>`/`<data>` for all attribute types. Node ID stored as
    `"name"` vertex attribute. 18 attribute tests including roundtrip.
  - **GML**: reads non-structural keys inside `node`/`edge` blocks as
    vertex/edge attributes; top-level keys (e.g. `Creator`) as graph
    attributes. Writer emits attribute key-value pairs. 21 tests.
  - **DOT**: reads `[key=value, ...]` blocks on nodes, edges, and `graph`
    statements; writer emits attribute blocks. Supports numeric, boolean,
    and string values. 34 tests including roundtrip with attributes.
  - **Pajek**: reader stores vertex labels as `"name"` vertex attribute and
    edge weights as `"weight"` edge attribute; writer falls back to these
    attributes when explicit parameters are `None`. 7 new attribute tests.
  - **NCOL**: same attribute integration — `"name"` and `"weight"` stored on
    read, used as fallback on write. 7 new attribute tests.
  - **LGL**: same attribute integration. 7 new attribute tests.
  - **DL (UCINET)**: same attribute integration for labels and weights;
    writer emits `labels:` section from attributes. 7 new attribute tests.
  - **LEDA**: same attribute integration; header type declarations
    (`string`/`double`) automatically set from attributes. 7 new tests.
- **mdBook tutorial** — new user-facing tutorial chapter covering graph
  construction, properties, centrality, community detection, shortest paths,
  attributes, I/O, isomorphism, operators, and iteration patterns.
- `examples/social_network_demo.rs` expanded to demonstrate all 8 I/O
  format round-trips (19 capabilities total).
- **Graph convenience methods batch 8** — 10 new method-style delegates
  bringing the total to 417 `pub fn` methods on `Graph`:
  `st_edge_connectivity`, `vertex_disjoint_paths`, `isomorphic_bliss`,
  `subisomorphic_lad`, `betweenness_subset`, `edge_betweenness_subset`,
  `edge_betweenness_community_weighted`, `modularity_matrix`,
  `is_same_graph`, `mean_distance_weighted`.
- `method_api_demo` expanded from 8 to 11 categories (flow/connectivity,
  subset centrality, BLISS isomorphism).
### Changed
- mdBook introduction updated to reflect current project state.
- ALGORITHMS.md: Phase 5/6/9 headers updated from "(partial)" to complete;
  ALGO-CORE-010 marked done.

### Fixed
- Broken rustdoc link to `AdjacencyType::Both` in `Graph::get_adjacency`
  doc comment — now uses fully-qualified `crate::` path.
- Clippy `similar_names` lint on `k4a`/`k4b` in `method_api_demo.rs`.

### Added
- **Vertex/edge/graph attribute system** — `AttributeValue` enum
  (Numeric/Boolean/String) with 19 methods on `Graph` for setting, getting,
  deleting, and listing attributes. Attribute vectors are automatically
  maintained during structural mutations (add/delete vertices/edges).
- **Ergonomic API improvements** for idiomatic Rust usage:
  - `GraphBuilder` — fluent builder pattern with `.vertices()`, `.edge()`,
    `.edges()`, `.path()`, `.cycle()`, `.clique()`, `.star()`, `.build()`.
  - `TryFrom<&[(u32, u32)]>` and `TryFrom<Vec<(u32, u32)>>` for `Graph` —
    construct undirected graphs directly from edge lists.
  - `std::ops` overloads: `+` (disjoint union), `|` (union), `&`
    (intersection), `-` (difference), `!` (complement) on `Graph`.
  - `IntoIterator for &Graph` and `Graph::iter()` — enables
    `for (u, v) in &graph { ... }` patterns.
  - `FromIterator<(u32, u32)>` — collect edges into a `Graph`.
  - `Extend<(u32, u32)>` — add edges from an iterator, auto-growing vertices.
  - `PartialEq`, `Eq`, `Hash` for `Graph` — structural equality
    (same directedness + vertices + sorted edge set).
  - `EdgeIter` type alias — public named iterator type for `Graph::iter()`.
- **Convenience methods on `Graph`** (method-style API delegating to free
  functions):
  - `density()`, `is_connected()`, `is_simple()`, `connected_components()`
  - `pagerank()`, `betweenness()`, `louvain()`
  - `induced_subgraph()`, `to_dot()`
  - `bfs()`, `dfs()`, `shortest_paths()`, `dijkstra()`
  - `degree_sequence()`, `diameter()`, `transitivity()`, `clique_number()`
  - `articulation_points()`, `topological_sort()`
  - `minimum_spanning_tree()`, `summary()`
- **Expanded prelude** — `EdgeIter`, `leiden`, `articulation_points`,
  `bridges`, `diameter`, `eccentricity`, `radius`, `transitivity_undirected`,
  `clique_number` now included in `rust_igraph::prelude`.
- `examples/quickstart.rs` — complete analysis pipeline example showcasing
  builder, connectivity, centrality, community detection, iteration, and
  graph algebra.
- `examples/community_detection_demo.rs` — Louvain vs Leiden comparison on
  a synthetic two-cluster graph.
- **More convenience methods on `Graph`**:
  - `closeness()`, `eigenvector_centrality()`, `clustering_coefficients()`
  - `complement()`, `leiden()`, `bridges()`, `coreness()`
  - `watts_strogatz(n, k, p, seed)` — small-world graph constructor
  - `strongly_connected_components()`, `shortest_path_to()`,
    `average_path_length()`, `is_bipartite()`
  - `simplify()`, `reverse()`, `contract_vertices()`, `random_walk()`
- `Graph::neighbors_iter()` — zero-allocation iterator over neighbors,
  with `ExactSizeIterator` impl. Ideal for hot-loop traversals.
- `NeighborsIter` type exported from crate root and prelude.
- `Graph::out_degree()` / `Graph::in_degree()` — O(1) per-vertex degree
  queries by index-array subtraction.
- `Graph::max_degree()` / `Graph::min_degree()` — O(V) graph-wide degree
  extrema.
- `line_graph()` — construct L(G) where each edge becomes a vertex.
  Supports directed (head-to-tail adjacency) and undirected graphs.
- `Graph::line_graph()` convenience method.
- `Graph::to_directed()` / `Graph::to_undirected()` — direction-conversion
  convenience methods.
- **More convenience methods on `Graph`** (batch 3):
  - Graph properties: `radius()`, `eccentricity()`, `girth()`, `is_tree()`,
    `is_dag()`, `count_triangles()`, `harmonic_centrality()`,
    `neighborhood_size()`
  - Connectivity: `vertex_connectivity()`, `edge_connectivity()`,
    `subcomponent()`
  - Cliques: `cliques()`, `maximal_cliques()`, `independence_number()`
  - Operators: `permute_vertices()`
- **Prelude expanded** with `erdos_renyi_gnp` and `watts_strogatz_game`.
- Enhanced crate-level documentation with a "Complete workflow" example
  showing load → structural check → centrality → community → shortest path
  → random walk in one coherent snippet.
- **85 convenience methods on `Graph`** (batch 4 — largest expansion):
  - **Structural recognizers** (50+): `is_acyclic()`, `is_antiregular()`,
    `is_bridge_graph()`, `is_caterpillar()`, `is_chordal()`, `is_claw_free()`,
    `is_cluster_graph()`, `is_cograph()`, `is_complete()`, `is_cubic()`,
    `is_distance_regular()`, `is_edgeless()`, `is_eulerian()`, `is_forest()`,
    `is_gem_free()`, `is_hamiltonian()`, `is_interval()`, `is_k_degenerate()`,
    `is_k_regular()`, `is_king()`, `is_kite()`, `is_lobster()`,
    `is_outerplanar()`, `is_paw_free()`, `is_planar()`, `is_power_of_cycle()`,
    `is_pseudo_forest()`, `is_queen()`, `is_rook()`, `is_self_complementary()`,
    `is_semiconnected()`, `is_spider()`, `is_split()`, `is_split_cluster()`,
    `is_threshold()`, `is_tournament()`, `is_traceable()`,
    `is_triangle_free()`, `is_turk()`, `is_unit_interval()`,
    `is_well_colored()`, `is_wheel()`, `satisfies_dirac()`, `satisfies_ore()`
  - **Centrality variants**: `edge_betweenness()`, `betweenness_cutoff()`,
    `betweenness_weighted()`, `closeness_weighted()`,
    `edge_betweenness_cutoff()`, `edge_betweenness_weighted()`,
    `harmonic_centrality_weighted()`, `pagerank_weighted()`
  - **Connectivity**: `cohesive_blocks()`, `count_reachable()`,
    `reachability_matrix()`, `transitive_closure()`
  - **Flow**: `mincut()`, `mincut_value()`, `gomory_hu_tree()`,
    `all_st_cuts()`, `edge_disjoint_paths()`
  - **Paths**: `distances()`, `eulerian_path()`, `mean_distance()`,
    `topological_sorting()`
  - **Structure**: `list_triangles()`, `degree_distribution()`,
    `get_edgelist()`, `regularity()`, `find_cycle()`,
    `joint_degree_matrix()`, `minimum_dominating_set()`, `graph_power()`,
    `trussness()`
- **Expanded prelude** (18 new re-exports): `eigenvector_centrality`,
  `harmonic_centrality`, `HitsScores`, `hub_and_authority_scores`,
  `katz_centrality`, `modularity`, `strongly_connected_components`,
  `mean_distance`, `coreness`, `list_triangles`, `count_triangles`,
  `topological_sorting`, `max_flow`, `MstAlgorithm`, `minimum_spanning_tree`,
  `isomorphic_vf2`, `cycle_graph`, `barabasi_game_bag`.
- Doctests for `famous_names()` and `cache_get`/`cache_set`/`cache_invalidate`/
  `cache_invalidate_all` methods.
- API count updated from 419+ to 850+ across documentation.
- **16 graph operator convenience methods** on `Graph`: `union()`,
  `intersection()`, `difference()`, `disjoint_union()`, `join()`,
  `compose()`, `cartesian_product()`, `tensor_product()`,
  `strong_product()`, `lexicographic_product()`,
  `connect_neighborhood()`, `rewire()`, `rewire_edges()`,
  `subgraph_from_edges()`, `even_tarjan_reduction()`,
  `bipartite_projection()`.
- **13 path/matching/community convenience methods** on `Graph`:
  `bellman_ford_distances()`, `floyd_warshall_distances()`,
  `k_shortest_paths()`, `all_simple_paths()`,
  `maximum_bipartite_matching()`, `vertex_coloring_greedy()`,
  `simple_cycles()`, `leading_eigenvector()`, `fluid_communities()`,
  `motifs_randesu()`, `personalized_pagerank()`.
- **8 isomorphism/analysis convenience methods** on `Graph`:
  `isomorphic_vf2()`, `isomorphic()`, `count_isomorphisms_vf2()`,
  `independent_vertex_sets()`, `global_efficiency()`,
  `local_efficiency()`, `assortativity_degree()`, `diversity()`.
- `examples/method_api_demo.rs` — showcases the method-style API across
  8 categories: construction, structural queries, centrality, community
  detection, shortest paths, generators, isomorphism, graph operations.
- Total public methods on `Graph`: 276 (up from 170).
- **I/O convenience methods on `Graph`** — read/write shortcuts that hide
  wrapper types:
  - `from_ncol_file()` / `to_ncol_file()`, `from_lgl_file()` /
    `to_lgl_file()`, `from_leda_file()` / `to_leda_file()`,
    `from_dl_file()` / `to_dl_file()`, `from_dimacs_file()`.
- **30 structural/analysis convenience methods** on `Graph`:
  - Cliques: `largest_cliques()`, `maximal_cliques_count()`,
    `clique_size_hist()`
  - Independent sets: `maximal_independent_vertex_sets()`,
    `largest_independent_vertex_sets()`
  - Coloring: `edge_coloring_greedy()`, `chromatic_number_upper_bound()`
  - Clustering: `transitivity_avglocal()`
  - Degree: `mean_degree()`, `degeneracy()`, `convergence_degree()`,
    `avg_nearest_neighbor_degree()`
  - Connectivity: `biconnected_components()`,
    `minimum_size_separators()`, `all_minimal_st_separators()`,
    `adhesion()`, `cohesion()`, `count_mutual()`
  - Traversal: `bfs_tree()`, `dfs_tree()`
  - Efficiency: `average_local_efficiency()`
  - Coupling: `bibcoupling()`
  - Predicates: `is_perfect()`, `is_regular()`, `is_strongly_regular()`,
    `is_weakly_chordal()`, `is_well_covered()`, `is_windmill()`,
    `is_complete_multipartite()`, `count_loops()`
- Total public methods on `Graph`: 315 (up from 276).
- **31 more convenience methods on `Graph`** (batch 5):
  - Paths: `distances_all()`, `distances_from()`, `get_shortest_paths()`,
    `get_all_shortest_paths()`, `johnson_distances()`, `widest_paths()`,
    `graph_center()`, `path_length_hist()`, `eulerian_cycle()`
  - Centrality: `hub_and_authority_scores()`, `strength()`, `knnk()`,
    `transitivity_barrat()`, `local_scan_1()`, `max_degree_vertex()`
  - Structure: `maximum_cardinality_search()`, `has_loop()`,
    `has_mutual()`, `is_multiple()`, `is_mutual()`, `isoclass()`
  - Community: `modularity()`
  - Constructors: `mycielskian()`, `to_prufer()`
  - Connectivity: `bond_percolation()`, `site_percolation()`,
    `reachability()`
  - Neighborhoods: `neighborhood_graphs()`
  - Cliques: `weighted_clique_number()`
  - Layout: `layout_mds()`, `layout_sphere()`
- Total public methods on `Graph`: 346 (up from 315).
- **30 more convenience methods on `Graph`** (batch 6):
  - Weighted community: `louvain_weighted()`, `leiden_weighted()`,
    `label_propagation_weighted()`, `walktrap_weighted()`
  - Weighted distances: `diameter_weighted()`, `eccentricity_weighted()`,
    `radius_weighted()`, `knnk_weighted()`
  - Centrality: `pagerank_linsys()`, `local_scan_k()`
  - Validators: `is_loop()`, `is_clique()`,
    `is_independent_vertex_set()`, `is_separator()`,
    `is_minimal_separator()`, `is_vertex_coloring()`,
    `is_edge_coloring()`, `is_vertex_cover()`, `is_edge_cover()`,
    `is_dominating_set()`
  - Operators: `reverse_edges()`, `induced_subgraph_edges()`
  - Similarity: `similarity_dice()`,
    `similarity_inverse_log_weighted()`, `similarity_jaccard_es()`
  - Layout: `layout_lgl()`, `layout_random_3d()`, `layout_grid_3d()`
  - Motifs: `motifs_randesu_no()`
  - Inspection: `graph_summary()`
- Total public methods on `Graph`: 376 (up from 346).
- **12 more convenience methods on `Graph`** (batch 7):
  - Matrix representations: `get_adjacency()`, `get_adjacency_weighted()`,
    `get_laplacian()`, `get_laplacian_full()`, `get_stochastic()`
  - Spectral embedding: `adjacency_spectral_embedding()`,
    `laplacian_spectral_embedding()`, `eigen_adjacency()`
  - Algorithms: `feedback_vertex_set()`, `complementer()`,
    `bipartite_projection_size()`, `unfold_tree()`
- Total public methods on `Graph`: 388 (up from 376).

- **I/O completeness** — all major graph formats now support both reading
  and writing:
  - `read_leda` (ALGO-IO-012) — LEDA native format reader with `LedaGraph`
    struct returning graph + optional labels + optional weights. Full
    round-trip with `write_leda`.
  - `write_dl` (ALGO-IO-013) — UCINET DL writer using edgelist1 format.
    Supports optional labels and edge weights. Full round-trip with `read_dl`.
  - `read_dot` (ALGO-IO-014) and `read_graphml` (ALGO-IO-015) now tracked.
  - Total: 15 I/O functions across 11 formats (GML, GraphML, Pajek, DOT,
    LEDA, DL, DIMACS, edge list, NCOL, LGL, GraphDB).

### Performance
- Switched hot-path inner loops across 16 algorithms from the allocating
  `neighbors()` to zero-allocation `neighbors_iter()`:
  `bfs`, `bfs_tree`, `bfs_simple`, `connected_components`, `distances`,
  `distances_all`, `distances_cutoff`, `distances_from`, `decompose`,
  `cohesive_blocks`, `separators`, `radii`, `eulerian_construct`,
  `pagerank`, `betweenness`, `triangles`, `dfs`, `coloring`.

### Changed
- CONTRIBUTING.md now welcomes external pull requests with clear guidelines.
- README updated with GraphBuilder and operator overloading examples;
  test count corrected to 720+.
- **ADR-0003 revised**: removed `faer` dependency plan; all linear algebra
  is pure hand-rolled Rust (power iteration, future IRLM/IRAM). Zero
  external deps beyond `thiserror`.
- MASTER_PLAN.md §2.3/§2.4 updated to reflect actual single-dependency
  architecture. Feature flags simplified.

### Added
- `Graph::watts_strogatz(n, k, p, seed)` convenience constructor for
  small-world graphs.
- Convenience methods on `Graph`: `closeness()`, `eigenvector_centrality()`,
  `clustering_coefficients()`, `complement()`, `leiden()`, `bridges()`,
  `coreness()`.
- `examples/community_detection_demo.rs` — Louvain vs Leiden showcase.
- Prelude expanded with `erdos_renyi_gnp`, `watts_strogatz_game`.

### Fixed
- Removed stale `faer`/`bliss` comment from CI WASM job.

### Added
- **ALGO-GN-031** — `iea_game`: **Independent Edge Allocation** random
  multigraph generator. Counterpart of `igraph_iea_game()` at
  `references/igraph/src/games/erdos_renyi.c:480-542` (upstream is an
  `IGRAPH_EXPERIMENTAL` wrapper over `igraph_erdos_renyi_game_gnm` with
  `IGRAPH_MULTI_SW | IGRAPH_LOOPS_SW`; the algorithm itself is the
  textbook independent-edge construction).
  - Signature:
    `iea_game(n: u32, m: u64, directed: bool, loops: bool, seed: u64) -> IgraphResult<Graph>`.
  - Sampling: each of the `m` edges is drawn independently and uniformly
    over the ordered pair space — `[0, n)²` when `loops = true`, or
    `[0, n) × ([0, n) \ {u})` when `loops = false`. The no-loops branch
    uses the standard "shift v by 1 past u" remap: sample `v ∈ [0, n-1)`
    and set `v += 1` whenever `v >= u`, yielding a uniform draw of `n-1`
    distinct values with a single PRNG call. The C variant achieves the
    same distribution via the matrix-extension trick, which is needed
    there to amortise an unrelated rejection step; with our cleaner
    Rust API the simpler shift is preferable.
  - Guarantees: result has **exactly `m` edges** (the model is named for
    this property), preserves the requested `directed` flag, and admits
    multi-edges by construction — two edges may map to the same ordered
    pair. The sampler is uniform over edge-labelled graphs; conditional
    on simplicity it coincides with G(n, m) without replacement, but is
    **not** uniform over multigraph realisations. Use
    [`erdos_renyi_gnm`] when uniform sampling over simple graphs is
    required.
  - Validation: rejects `m > u32::MAX`, `m > 0 && n == 0`, and
    `m > 0 && !loops && n < 2`. Empty requests (`m == 0`) succeed for
    any `n >= 0`.
  - Determinism: same `(n, m, directed, loops, seed)` always produces
    the same graph via `SplitMix64`.
  - Time complexity: `O(|V| + |E|)`. One PRNG draw for `loops = true`,
    two draws for `loops = false`, plus one `push` per edge and a final
    `Graph::add_edges` insertion.
  - Conformance: 9 fixtures (3 each from C / py / R) under
    `tests/conformance/{c,py,r}/iea_game/`. RNG state is not portable
    across implementations, so fixtures assert structural invariants
    only — `vcount == n`, `ecount == m` (**exact**, not a band),
    `directed` flag, and `no_self_loops` (every edge has `u != v`)
    when `loops = false`.
  - Bench at `benches/bench_iea_game.rs` covers three groups
    (`directed_loops`, `directed_no_loops`, `undirected_no_loops`) at
    `n = 1 000`, `m ∈ {1k, 10k, 100k}`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-031.json`: 18-24 Melem/s throughput
    across all variants on Darwin/M-series release build; cost is
    dominated by the SplitMix64 draws plus the `Vec::push` of
    `(u32, u32)` into the staging buffer.
  - Example: `examples/iea_game_demo.rs` builds three multigraphs
    illustrating self-loop frequency, multi-edge multiplicity, and
    the uniform marginal endpoint distribution.
  - Re-exported as `rust_igraph::iea_game`.

- **ALGO-GN-030** — `bipartite_game_gnp` / `bipartite_game_gnm` bipartite
  Erdős-Rényi random-graph generators. Counterparts of
  `igraph_bipartite_game_gnp()` / `igraph_bipartite_game_gnm()` in
  `references/igraph/src/games/bipartite.c`. Two sampling families share
  the same partition convention: vertices `0..n1-1` form the *bottom*
  partition (`types[i] = false`), vertices `n1..n1+n2-1` form the *top*
  partition (`types[i] = true`).
  - `bipartite_game_gnp(n1: u32, n2: u32, p: f64, directed: bool, mode: BipartiteMode, seed: u64) -> IgraphResult<BipartiteGraph>`:
    Batagelj-Brandes 2005 geometric-skip sampler over the bipartite pair
    space — `n1·n2` ordered cross-pairs for `Out`/`In`/undirected `All`,
    or `2·n1·n2` for directed `All` (each ordered cross-pair sampled
    independently). Validates `p ∈ [0, 1]`; NaN/Inf rejected.
  - `bipartite_game_gnm(n1: u32, n2: u32, m: u64, directed: bool, mode: BipartiteMode, seed: u64) -> IgraphResult<BipartiteGraph>`:
    Floyd distinct-sample over the same pair space, decodes each sampled
    pair-index back to `(bottom, top)` via `to = idx/n1; from = idx − to·n1; to += n1`
    (swap for `In`; second half of the index space encodes `top → bottom`
    for directed `All`). Rejects `m > cap` where `cap` matches the pair
    count above.
  - `BipartiteMode::{Out, In, All}` mirrors the C `IGRAPH_OUT` / `IGRAPH_IN`
    / `IGRAPH_ALL` family. For undirected graphs every mode produces a
    single edge per cross-pair; for directed graphs `Out` emits
    bottom→top arcs, `In` emits top→bottom arcs, `All` emits both
    directions independently.
  - Returns `BipartiteGraph { graph: Graph, types: Vec<bool> }` where
    `types` is the partition indicator vector.
  - Edge cases: `n1 == 0` or `n2 == 0` returns the requested vertex set
    with zero edges; `p == 0` short-circuits to edgeless; `p == 1` (or
    `m == cap`) produces the complete bipartite graph K_{n1,n2} (or its
    directed counterpart). Deterministic given the input tuple via
    `SplitMix64`.
  - Conformance: 18 fixtures (3 C + 3 py + 3 R per algo) under
    `tests/conformance/{c,py,r}/bipartite_game_{gnp,gnm}/`. RNG state is
    not portable, so fixtures assert structural invariants only —
    `vcount`, `n1`/`n2` partition counts, `directed` flag,
    `types[0..n1] = false` / `types[n1..n1+n2] = true`, every edge
    crosses the bipartition, `is_simple` (no self-loops, no multi-edges
    via canonical-pair `HashSet`), `ecount` band, plus
    `edges_bottom_to_top` / `edges_top_to_bottom` for directed `Out`/`In`
    cases.
  - Bench at `benches/bench_bipartite_game.rs` covers four groups
    (`gnp/undirected`, `gnp/directed_all`, `gnm/undirected`,
    `gnm/directed_out`) at `(n1, n2) ∈ {100, 1 000, 5 000}²`. Baseline
    at `.codefuse/tracking/perf/ALGO-GN-030.json`: gnp at `p = 0.05`
    ~250-600 Melem/s of pair-space throughput; gnm at
    `m = 10·max(n1,n2)` ~14-23 Melem/s of distinct-sample throughput
    (Floyd HashSet dominates).
  - Example: `examples/bipartite_game_demo.rs` builds a sparse undirected
    G(200, 150, p = 0.05) and a directed G(200, 150, m = 600) with
    `mode = Out`, then prints per-side degree means and the top-5 hubs
    on each partition.
  - Re-exported as `rust_igraph::{BipartiteGraph, BipartiteMode,
    bipartite_game_gnp, bipartite_game_gnm}`.

- **ALGO-FL-030** — `dominator_tree`: **Lengauer-Tarjan dominator tree** of
  a directed flowgraph. Given a directed graph and a root `r`, vertex `v`
  *dominates* `w` iff every path `r → w` goes through `v`; the *immediate
  dominator* `idom(w)` is the unique dominator closest to `w`. Mirrors
  `igraph_dominator_tree` at `references/igraph/src/flow/st-cuts.c:434-609`
  (~180 lines including the `dbucket`/`LINK`/`COMPRESS`/`EVAL` helpers at
  `st-cuts.c:286-375`). Rust-native re-implementation:
  1. **DFS phase** — iterative neighbour-cursor DFS from `root` assigns
     DFS numbers `semi[v]`, the inverse `vertex[i]`, and the spanning-tree
     `parent[v]`. Direction is `mode`-dependent: `Out` follows out-edges,
     `In` follows in-edges (post-dominator tree).
  2. **Reverse pass** `i = component_size-1 .. 1` — for each `w = vertex[i]`,
     walks `w`'s opposite-mode neighbours `v`, computes
     `semi[w] = min(semi[w], semi[EVAL(v)])`, inserts `w` into
     `bucket[vertex[semi[w]]]`, then `LINK(parent[w], w)` and drains the
     parent's bucket: for each `v` in the bucket, `u = EVAL(v)` and
     `idom[v] = (semi[u] < semi[v] ? u : parent[w])`.
  3. **Forward fixup** `i = 1 .. component_size-1` — propagates the
     immediate dominator wherever the semi-dominator deferred:
     `if idom[w] != vertex[semi[w]] then idom[w] = idom[idom[w]]`.
  - `pub fn dominator_tree(graph: &Graph, root: VertexId, mode: DominatorMode)
    -> IgraphResult<DominatorTree>`. `DominatorMode::{Out, In}` matches
    `IGRAPH_OUT` / `IGRAPH_IN`; `IGRAPH_ALL` is rejected (the algorithm
    requires a *directed* flowgraph).
  - Returns `DominatorTree { idom: Vec<i32>, tree: Graph, leftout: Vec<u32> }`.
    `idom[v] = -1` for `root`, `-2` for vertices unreachable from `root`,
    else the immediate-dominator vertex id (0-based). `tree` is a directed
    graph with the same `vcount()` as the input — edges go parent→child
    (`Out`) or child→parent (`In`), unreachable vertices appear as isolates.
    `leftout` lists unreachable vertex ids in ascending order.
  - Error contract: `InvalidArgument` on undirected input (matches the
    igraph C behaviour at `st-cuts.c:439`). `VertexOutOfRange` if
    `root >= vcount()`.
  - Tests: unit + proptest random-DAG harness exercising
    `idom == parent on a tree`, `unreachable ⇔ idom == -2`, and the
    `tree.vcount() == g.vcount() ∧ tree.ecount() == #reachable_non_root`
    invariants. Three-source conformance: 4 C fixtures from
    `tests/unit/igraph_dominator_tree.c`, 4 python-igraph fixtures from
    `tests/test_structural.py::testDominators`, 4 R fixtures from
    `tests/testthat/test-flow.R` + `test-aaa-auto.R`.
  - Runnable demo: `cargo run --example dominator_tree_demo` — prints the
    `idom` vector, tree edges, and `leftout` for the canonical 13-vertex
    LT example, the 6-vertex R-igraph DAG, the 20-vertex flowgraph with
    an unreachable component, and the 13-vertex reversed graph with
    `mode=In`. Each reachable vertex's `idom` is brute-force checked
    against every simple `root → w` path.
  - Criterion baseline `.codefuse/tracking/perf/ALGO-FL-030.json`: 13v
    classical at 6.1 µs; complete binary trees `depth_6..12` and a
    reducible-flowgraph stress (`n_64..4096` with deterministic
    `i → i/2` back-edges) capture both pure-DFS-tree and bucket-active
    regimes. Theoretical complexity is
    O(|V| + |E|·α(|E|, |V|)) — near-linear with the inverse Ackermann
    factor from `EVAL` path compression.

- **ALGO-FL-020** — `gomory_hu_tree`: all-pairs minimum-cut **Gomory-Hu
  cut tree** via **Gusfield 1990**. Mirrors `igraph_gomory_hu_tree` at
  `references/igraph/src/flow/flow.c:3079` (a ~70-line driver that loops
  `igraph_maxflow_value` over a partial parent array and accumulates an
  edge list). Rust-native re-implementation (not a line-by-line port):
  1. Single-pass driver over vertices `1..vcount()`. For each `i`,
     `t = parent[i]`, runs `st_mincut(graph, i, t, capacity)` (FL-018)
     once. The returned source-side partition `S` (always containing `i`)
     is used to re-parent every `j > i` that lies in `S` and currently
     points to `t` — exactly the Gusfield update rule, but using the
     already-computed `StMincut` partition instead of a second BFS pass.
  2. Builds the tree edge list as `(i, parent[i])` for `i in 1..n`,
     with `flows[i - 1] = mincut.value` from the `i`-th call.
  3. Returns `GomoryHuTree { tree: Graph, flows: Vec<f64> }`. `tree` has
     the same `vcount()` as the input, exactly `vcount() - 1` edges,
     and is undirected by construction (Gomory-Hu trees are always
     undirected even when the input is undirected — directed inputs are
     rejected at entry to match igraph C semantics and avoid ambiguity
     about which direction the cut should travel).
  - `pub fn gomory_hu_tree(graph: &Graph, capacity: Option<&[f64]>) ->
    IgraphResult<GomoryHuTree>`. The output satisfies the **Gomory-Hu
    property**: for every pair `(u, v)`, `min(flows[e] for e on tree
    path from u to v) == max_flow_value(graph, u, v, capacity)`. This is
    the test invariant exercised by the proptest harness and the runnable
    example.
  - Error contract: `InvalidArgument` on directed input (matches igraph
    C 0.10.x behaviour); `InvalidArgument` if `capacity.len() != ecount`
    or any capacity is negative / non-finite; `VertexOutOfRange` is
    impossible because no caller-supplied vertex enters the path.
  - Empty graph (`vcount == 0`) and single-vertex graph (`vcount == 1`)
    both return a tree with `0` edges and `flows = []`. Disconnected
    inputs return a tree whose inter-component edges carry weight `0.0`
    (the all-pairs min-cut across components is `0`, which is the
    correct degenerate value).
  - Backend reuse: every iteration calls the **same** `st_mincut`
    backend as FL-018, so all FL-020 testing also exercises FL-018's
    Dinic-based max-flow plus partition extraction. Total: 22 unit
    + proptest tests, 12 three-source conformance fixtures (4 each
    from igraph C / python-igraph / rigraph), a 5-case runnable
    `examples/gomory_hu_tree_demo.rs` (K_4, C_5, the canonical C 6v
    weighted fixture, P_4 with caps `[3, 1, 5]`, Petersen), and a
    9-config criterion baseline (`6v_weighted`, `K_8..K_64`,
    `C_16..C_1024`) in `.codefuse/tracking/perf/ALGO-FL-020.json`.
    Observed scaling: K_n grows ~n^3 (10.9µs at n=8, 2.2ms at n=64);
    C_n grows ~n^2 (21µs at n=16, 67ms at n=1024).
- **ALGO-FL-018** — `st_mincut`: full s-t minimum-cut **partition** —
  the value plus the cut edge ids and the source-side / sink-side
  vertex bipartition. Mirrors `igraph_st_mincut` at
  `references/igraph/src/flow/flow.c:1140` (a 47-line wrapper around
  `igraph_maxflow` that requests the optional `cut`, `partition`, and
  `partition2` outputs alongside the flow value). Implementation:
  1. Refactored `src/algorithms/flow/max_flow.rs` so that
     `max_flow_value` is now a thin wrapper over a new crate-private
     entry point `max_flow_with_residual(graph, source, target,
     capacity) -> IgraphResult<(f64, Network)>`. The shared backend
     builds the flat-CSR paired-arc residual network once, runs Dinic,
     and returns both the flow value and the post-augmentation residual
     state. `max_flow_value` simply drops the residual; `st_mincut`
     consumes it for partition extraction.
  2. After Dinic terminates, runs one BFS from `source` in the
     residual graph (following only arcs with strictly positive
     residual capacity — saturated arcs land at exactly `0.0`, not
     within fp noise, because Dinic only subtracts pushed flow from a
     residual that already had at least that much capacity).
  3. The reachable set is the source-side `S` of a minimum s-t cut by
     max-flow / min-cut duality (Ford-Fulkerson 1956); the complement
     is `partition2`.
  4. Walks the original edge list once: an edge `(u, v)` is in the cut
     iff it crosses the partition boundary — directed input requires
     the cut to point `S → V\S` (the reverse direction lies in the
     opposite cut and is not saturated by this flow); undirected input
     accepts either crossing.
  - `pub fn st_mincut(graph: &Graph, source: VertexId, target: VertexId,
    capacity: Option<&[f64]>) -> IgraphResult<StMincut>` and
    `pub struct StMincut { value: f64, cut: Vec<u32>, partition:
    Vec<u32>, partition2: Vec<u32> }`. Both partitions are sorted
    ascending; together they cover every vertex exactly once.
    `partition` always contains `source`; `partition2` always contains
    `target` (unless they are already separated by the empty cut on a
    disconnected input, in which case `partition = {source}`).
  - Sum of `capacity[cut[i]]` (or `cut.len()` when `capacity` is
    `None`) equals `value` within `1e-12` — verified by the proptest
    invariant.
  - Error contract matches `max_flow_value` / `st_mincut_value`:
    `VertexOutOfRange` if `source` or `target` is outside
    `[0, vcount())`, `InvalidArgument` if `source == target`, the
    capacity slice length differs from `ecount()`, or any capacity is
    negative / non-finite. Validation is delegated to the max-flow
    layer so every error path is exercised by the FL-002 test suite.
  - Tests (9 unit + 1 proptest with 4 invariants): single-bottleneck
    chain (unique cut = middle arc), two parallel unit-cap paths
    (cut = both bottlenecks, partition collapses to `{source}`),
    isolated endpoints (empty cut, `partition = {source}`,
    `partition2 = V \ {source}`), single edge graph (cut = edge,
    `partition = {0}` / `partition2 = {1}`), undirected 4-vertex
    igraph reference (cuts an inseparable triangle from the sink),
    multigraph with parallel arcs (cut takes both arcs in sum),
    CLRS 26.1-1 textbook (matches FL-002 / FL-010 numerically; cut
    edge set varies by Dinic's BFS order, structural invariants
    always hold), validation rejection (source == target → `InvalidArgument`),
    and value-parity with `st_mincut_value` / `max_flow_value`
    (peer equivalence across all fixtures). Proptest 50-case rig over
    arbitrary 3-8 vertex directed graphs with random integer capacities
    pins: (a) cover invariant — `partition ∪ partition2 = [0, V)`,
    sorted, disjoint, exactly one contains `source` / `target`;
    (b) sum-of-capacities equals `value`; (c) removing cut edges
    disconnects `source` from `target` via residual BFS in the
    original graph; (d) value matches `st_mincut_value` /
    `max_flow_value` within `1e-12`.
  - Three-source conformance fixtures (10 total) under
    `tests/conformance/{c,py,r}/st_mincut/` exercise both unit and
    weighted paths, directed and undirected, and pin structural
    invariants on every fixture (cover, source/target sidedness,
    capacity sum = value, removing cut disconnects s from t via BFS).
    When the min cut is unique the fixture additionally pins the exact
    `cut` / `partition` / `partition2` lists; otherwise only the value
    + structural invariants are checked, so Dinic's BFS ordering does
    not destabilise CI. C fixtures (4): 5v directed unit caps verbatim
    from `tests/unit/igraph_st_mincut.c:52-58`, 5v directed weighted
    `[8,2,3,3,2]` (same shape, exercises the f64 path), CLRS textbook
    value-only, undirected 4v reference. Python fixtures (3):
    multigraph two-parallel arcs, directed bottleneck weighted,
    disconnected zero-value. R fixtures (3): single directed edge,
    two parallel paths unit caps value-only, undirected K_4 unit caps
    value-only. Harness `st_mincut_three_source_conformance()` parses
    the object-shaped `expected` field and enforces the universal
    structural invariants on every fixture; per-fixture exact-list
    pinning only when the fixture explicitly opts in via `cut` /
    `partition` / `partition2` keys.
  - Bench `benches/bench_st_mincut.rs` adds two FL-018 regimes
    alongside the existing FL-010 ones: `st_mincut/textbook` (6v 10e
    directed, 723 ns vs FL-010's 581 ns — +24% per-call overhead
    dominated by the `StMincut` struct + partition `Vec<u32>`
    allocations on small inputs) and `st_mincut/layered/{L4xW8,
    L6xW16, L8xW32}` (5.86/26.37/103.16 µs vs FL-010's
    5.38/24.92/96.23 µs — +6-9% on larger inputs where Dinic's
    O(V·E) work amortises the post-Dinic BFS over residual arcs and
    the linear edge sweep). The refactor also makes `st_mincut_value`
    measurably faster than its prior snapshot (3× improvement vs the
    pre-FL-018 numbers at `.codefuse/tracking/perf/ALGO-FL-010.json`)
    because the Dinic state is now built once rather than rebuilt
    around a cloned graph.
  - Demo `examples/st_mincut_partition_demo.rs` walks 8 cases
    end-to-end: single bottleneck chain, CLRS 26.1-1 textbook, two
    parallel unit-cap paths, disconnected endpoints, undirected
    igraph_maxflow.c reference, multigraph with parallel arcs, and
    both unit-cap and weighted variants of the
    `igraph_st_mincut.c:52-58` 5-vertex reference.

- **ALGO-FL-017** — `mincut_value`: global minimum-cut **value** —
  weighted generalisation of FL-016. Returns `f64` (total minimum
  capacity of edges whose removal makes a directed graph not strongly
  connected, or an undirected graph not connected). Mirrors
  `igraph_mincut_value` at
  `references/igraph/src/flow/flow.c:1692`. `capacity: Option<&[f64]>`
  matches the igraph C API (`None` = unit capacities). For
  `vcount ≤ 1` returns `f64::INFINITY` (mirrors `IGRAPH_INFINITY` init
  at flow.c:1699). Otherwise runs the fixed-vertex iteration also used
  by FL-016: pick vertex 0, call `max_flow_value(graph, 0, v,
  capacity)` for every `v ≠ 0` (undirected) or both `(0, v)` and
  `(v, 0)` (directed); track the running minimum with early-exit at
  `0.0`. Correctness: every global min-cut separates 0 from at least
  one other vertex, so `mincut = min_{v != 0} max_flow(0, v)`.
  Deliberately does **not** port Stoer-Wagner for the undirected case
  — the fixed-vertex iteration via FL-002 covers both directed and
  undirected with one code path, trading asymptotic optimality on
  dense undirected inputs for code simplicity and exact parity with
  the directed branch.
  - `pub fn mincut_value(graph: &Graph, capacity: Option<&[f64]>) ->
    IgraphResult<f64>`.
  - Unlike FL-016 there is no `checks` parameter, because the igraph C
    `igraph_mincut_value` API does not expose one (the cheap
    short-circuits only apply to the integer-valued unit-cap case).
  - Capacity validation happens up front (length must match `ecount`,
    every entry must be finite and `≥ 0`) so `n = 0`, `n = 1`, and
    `n ≥ 2` all return the same `InvalidArgument` error shape on bad
    capacity inputs instead of `n ≤ 1` silently returning `+∞`.
  - Tests cover the 5 C-fixture cases from
    `tests/unit/igraph_mincut.c` (directed 3-cycle unit caps → 1,
    weighted directed 3-cycle → 1 (bottleneck arc), undirected
    ring `C_5` unit caps → 2, two isolated edges → 0, complete `K_4`
    undirected → 3), 2 py fixtures from `test_flow.py:CutTests`,
    3 R fixtures from `test-flow.R` (including the C_5 weak-edge
    case → 10.5), plus edge cases: empty graph → `+∞`, single vertex
    → `+∞`, K_2 unit caps → 1, multigraph parallel edges → 2 (parallel
    edges raise the cut), C_5 with cheap bridge → 10.5, directed
    out-tree → 0 (sink has no out-arcs), K_5 with cheap edges only on
    vertex 0 → 4 (isolating vertex 0). Proptest pins 4 invariants:
    result is always non-negative and finite-or-`+∞`, `None` matches
    unit-cap vector, `mincut_value(g, None) as i64 ==
    edge_connectivity(g, true)` (unit-cap equivalence), and doubling
    every capacity doubles the mincut value (linearity).
  - Three-source conformance fixtures landed under
    `tests/conformance/{c,py,r}/mincut_value/` (5 + 2 + 3 fixtures)
    exercising both unit and weighted capacity paths in both
    directions; harness asserts at least one fixture from each of
    c/py/r and all 10 pass.
  - Bench `benches/bench_mincut_value.rs` captures five regimes:
    textbook `C_5` (1.7 µs), `layered_unit_caps` (10.4/45.3/199.8 µs
    for L4xW8/L6xW16/L8xW32 — ~4–6× slower than FL-016 on the same
    fixtures because the `checks=true` SCC short-circuit cannot apply
    without a `checks` parameter), `fixed_vertex_ring` (2.3/3.9/8.3 µs
    on C6/C8/C12 — per-call parity with FL-016 on connected inputs),
    `fixed_vertex_complete` (1.1/4.4/8.0 µs on K4/K6/K8), and
    `weighted_ring` (2.6/4.5/9.7 µs on C6/C8/C12 — ~10–15% overhead
    over the unit-cap ring because Dinic's `f64`-min reduction is
    slightly more expensive than the `u32` counter early-exit when
    augmenting paths carry capacity 1 exactly).
  - Demo `examples/mincut_value_demo.rs` walks 8 cases end-to-end:
    empty graph (`+∞`), single vertex (`+∞`), C_5 unit caps (2),
    C_5 with weak bridge (10.5), two isolated edges (0), directed
    3-cycle weighted (1, bottleneck arc), multigraph 2 parallel
    edges (2), K_5 with cheap edges on vertex 0 (4, isolating cost).

- **ALGO-FL-016** — `edge_connectivity` (alias `adhesion`): global edge
  connectivity (adhesion) of a graph — the minimum number of edges
  whose removal disconnects the graph. Mirrors
  `igraph_edge_connectivity` at
  `references/igraph/src/flow/flow.c:2238` and its alias
  `igraph_adhesion` (flow.c:2466). Computes
  `min_{v ≠ 0} st_edge_connectivity(0, v)` for undirected graphs and
  both `(0, v)` and `(v, 0)` for directed graphs — the fixed-vertex
  iteration is correct because any global min-cut separates vertex 0
  from at least one vertex on the other side, and runs in `V - 1`
  max-flow computations instead of the `V(V - 1)` of the pairwise loop
  (matching `igraph_mincut_value`'s directed branch at
  flow.c:1706-1723). When `checks=true` (recommended), runs the cheap
  short-circuits shared with FL-015 (flow.c:2069-2122): (1) empty
  graph → 0; (2) disconnected (weak for undirected, strong for
  directed) → 0; (3) any vertex with `min(in, out) = 1` → 1.
  **No complete-graph shortcut** — flow.c:2168-2180 explicitly notes
  that completeness alone doesn't determine edge connectivity because
  multigraphs admit `lambda > n - 1` (parallel edges raise the min
  cut). Result is `i64` matching upstream's `igraph_integer_t`.
  - `pub fn edge_connectivity(graph: &Graph, checks: bool) ->
    IgraphResult<i64>`.
  - `pub fn adhesion(graph: &Graph, checks: bool) -> IgraphResult<i64>`
    — exact synonym for naming parity with the upstream API and the
    White–Harary (2001) sociological-network literature.
  - Tests cover the two C unit-test fixtures
    (`tests/unit/igraph_edge_connectivity.c`: 7-vertex directed graph
    → 1, 7-vertex undirected graph → 2), R parity from
    `tests/testthat/test-flow.R` (`make_ring(5, circular=F)` → 1,
    `make_graph(edges=c(1,2,3,4), directed=F)` → 0,
    `make_ring(5, circular=T)` → 2), py parity from
    `test_flow.py:27,29-30` (`Tree(10, 3, "out")` directed → 0,
    `Full(4)` undirected → 3), edge cases (empty, single vertex, K_2,
    K_5 undirected, K_4 directed mutual → 3), the no-complete-shortcut
    multigraph case (two parallel edges between 0-1 → `lambda = 2`,
    not 1), undirected tree → 1, directed cycle → 1, cycle-with-chord
    → 2, and `checks=false == checks=true` agreement on a small
    fixture battery. Proptest pins four invariants: `0 ≤ ec ≤ m`,
    `checks=false` agrees with `checks=true`, Whitney's
    `ec ≤ min_degree` (undirected), and `ec ≤ st_edge_connectivity(s,
    t)` for every `s ≠ t`.
  - Three-source conformance fixtures landed under
    `tests/conformance/{c,py,r}/edge_connectivity/` (5 + 2 + 3
    fixtures) covering all three cheap short-circuit branches plus
    the fixed-vertex loop path on K_5 and C_5; harness asserts at
    least one fixture from each of c/py/r and they all pass.
  - Bench `benches/bench_edge_connectivity.rs` captures both regimes:
    cheap short-circuits (textbook 222 ns, layered 2.6–34.3 µs scaling
    with the SCC `O(V+E)` cost) and the fixed-vertex loop on rings
    and complete graphs (`C_6` 2.5 µs / `C_{12}` 8.4 µs linear in V;
    `K_4` 1.3 µs / `K_8` 8.5 µs). The fixed-vertex loop is ~170×
    faster than FL-015's pairwise loop on the same `C_{12}` fixture
    because it runs `V - 1` max-flow calls instead of `V(V - 1)`.
  - Demo `examples/edge_connectivity_demo.rs` exercises every branch
    end-to-end (cycle, path, disconnected, complete, directed out-tree,
    multigraph parallel edges, directed cycle) and prints the
    adhesion alias parity.

- **ALGO-FL-015** — `vertex_connectivity` (alias `cohesion`): global
  vertex connectivity (cohesion) of a graph — the minimum number of
  internal vertices whose removal disconnects some pair of vertices.
  Mirrors `igraph_vertex_connectivity` at
  `references/igraph/src/flow/flow.c:2158` and its alias
  `igraph_cohesion` (flow.c:2470). Computes
  `min_{s ≠ t} st_vertex_connectivity(s, t, NumberOfNodes)` — using
  `NumberOfNodes` mode so direct s→t edges return the n-sentinel and
  never lower the running minimum (matching the upstream loop at
  flow.c:1969-2037). When `checks=true` (recommended), runs the cheap
  short-circuits suggested by Peter McMahan (flow.c:2069-2122): (1)
  empty graph → 0; (2) disconnected (weak for undirected, strong for
  directed) → 0; (3) any vertex with `min(in, out) = 1` → 1; (4)
  complete graph → `n - 1`. Each helper is `O(V + E)` and the
  short-circuits give a 300×-5,800× speed-up over the brute-force
  pairwise loop on the FL-015 bench fixtures. Pairwise loop iterates
  `j > i` for undirected and all `j ≠ i` for directed, with early-exit
  when `min_conn` hits 0. Result is `i64` matching upstream's
  `igraph_integer_t`.
  - `pub fn vertex_connectivity(graph: &Graph, checks: bool) ->
    IgraphResult<i64>`.
  - `pub fn cohesion(graph: &Graph, checks: bool) -> IgraphResult<i64>`
    — exact synonym for naming parity with the upstream API and the
    White–Harary (2001) sociological-network literature.
  - Tests cover the two C unit-test fixtures
    (`tests/unit/igraph_cohesion.c`: 7-vertex directed graph → 1,
    7-vertex undirected graph → 2), R parity from
    `tests/testthat/test-flow.R:130-138` (`make_ring(5, circular=F)` →
    1, `make_graph(edges=c(1,2,3,4), directed=F)` → 0,
    `make_ring(5, circular=T)` → 2), py parity from
    `test_flow.py:27,29-30` (`Tree(10, 3, "out")` directed → 0,
    same tree undirected → 1, `Full(4)` → 3), edge cases (empty,
    single vertex, two disconnected vertices, K_2, K_6, mutual K_5),
    cycle-with-chord (pairwise-loop path), and `checks=false ==
    checks=true` agreement on a small fixture battery. Proptest pins
    three invariants: `0 ≤ vc ≤ n - 1`, Whitney's `vc ≤ min_degree`
    (undirected), and `checks=false` agrees with `checks=true` on
    random small graphs.
  - Three-source conformance fixtures landed under
    `tests/conformance/{c,py,r}/vertex_connectivity/` (5 + 2 + 3
    fixtures) covering all four cheap short-circuit branches plus the
    pairwise-loop path; harness asserts at least one fixture from each
    of c/py/r and they all pass.
  - Bench `benches/bench_vertex_connectivity.rs` captures both regimes:
    cheap short-circuits (textbook 240 ns, layered 2.7–34.9 µs scaling
    with the SCC `O(V+E)` cost) and the brute-force pairwise loop on
    rings (`C_6` 80 µs → `C_{12}` 1.4 ms), confirming the `O(V^5)`
    growth predicted by igraph C's docstring.
  - Demo `examples/vertex_connectivity_demo.rs` exercises every branch
    end-to-end (cycle, path, disconnected, complete, directed out-tree,
    cycle-with-chord) and prints the cohesion alias parity.

- **ALGO-FL-014** — `vertex_disjoint_paths`: maximum number of pairwise
  internally vertex-disjoint paths from `source` to `target`. Mirrors
  `igraph_vertex_disjoint_paths` at
  `references/igraph/src/flow/flow.c:2374`. Thin wrapper that calls
  [`st_vertex_connectivity`] with `VconnNei::Ignore` (which subtracts the
  direct-edge contribution from the split-graph max-flow) and then adds
  the count of direct `source → target` edges back on top: by Menger
  (1927), every parallel arc contributes one trivially internally-
  disjoint path of length 1. The result is `i64` matching upstream's
  `igraph_integer_t`.
  - `pub fn vertex_disjoint_paths(graph: &Graph, source: VertexId,
    target: VertexId) -> IgraphResult<i64>`.
  - Error contract delegates to [`st_vertex_connectivity`]
    (`VertexOutOfRange` for ids ≥ `vcount()`, `InvalidArgument` for
    `source == target` or `vcount() < 2`, `Internal` for the rare
    `i64` overflow on `vc + direct_count`).
  - 11 unit tests: error contract (source == target, both endpoints
    out-of-range), basic correctness (isolated endpoints, single direct
    edge, two parallel directed paths, direct edge + interior detour,
    4 parallel arcs, directed-chain has no reverse path), the C unit
    test's two multigraph cases (directed + undirected on the same 7v
    fixture), and two invariants — `vdp = vc(Ignore) + direct_count`
    when a direct edge exists (plus a sentinel sanity-check that
    `VconnNei::NumberOfNodes` is NOT interchangeable since it returns
    `vcount()` instead of the additive count), and
    `vdp == st_vertex_connectivity(Ignore)` when no direct edge exists.
    2 proptests cross-validate the Menger equality and the
    `vdp ≤ ecount()` slack-free upper bound across random unit-cap
    directed/undirected graphs.
  - Three-source conformance: 5 C fixtures verbatim-replaying
    `tests/unit/igraph_vertex_disjoint_paths.c:32-47` (the directed and
    undirected halves of the 7v multigraph), 2 py fixtures
    (`Graph.vertex_disjoint_paths` aliased to `Graph.vertex_connectivity`
    per `src/igraph/__init__.py:341` — K_5 directed `vdp(0,1)=4` plus
    undirected path `vdp(0,3)=1`), 2 R fixtures
    (`igraph::vertex_disjoint_paths(graph, source =, target =)` per
    `test-flow.R:201-207` — K_5 undirected `vdp(0,1)=4` and directed
    P_5 `vdp(0,2)=1`). All 9 fixtures pass.
  - Bench: `bench_vertex_disjoint_paths` mirrors the FL-013 fixture
    shape (textbook 6v path + layered `L4×W8`, `L6×W16`, `L8×W32`)
    so the JSONs compare cell-for-cell. Headline: textbook = 8.64 µs,
    L4×W8 = 717 µs, L6×W16 = 22.7 ms, L8×W32 = 721 ms. Overhead vs.
    FL-013 is bounded by one `get_all_eids_between` call (O(deg(s)));
    on the layered grids the FL-014 / FL-013 ratio is 1.00× — 1.04×,
    so the wrapper is essentially free past the textbook size.

- **ALGO-FL-013** — `st_vertex_connectivity`: minimum number of internal
  vertices whose removal disconnects `source → target`. Mirrors
  `igraph_st_vertex_connectivity` at
  `references/igraph/src/connectivity/connectivity.c:31`. Algorithm is
  the textbook **vertex-splitting reduction** (Even, *Graph Algorithms*
  §5.5): each vertex `v` is split into `v_out` and `v_in` joined by an
  internal arc of capacity 1; every original arc `u → v` becomes
  `u_out → v_in` with capacity 1; the s-t vertex connectivity then
  equals the unit-cap max-flow from `source_out` to `target_in` on the
  split graph (Menger 1927: max number of internally vertex-disjoint
  paths = min vertex cut). Direct `source → target` arcs are handled
  per the [`VconnNei`] mode (the upstream
  `igraph_vconn_nei_t` enum).
  - `pub fn st_vertex_connectivity(graph: &Graph, source: VertexId,
    target: VertexId, mode: VconnNei) -> IgraphResult<i64>` plus the
    `pub enum VconnNei { Error, Negative, NumberOfNodes, Ignore }`.
    Returns `i64` to match igraph C's `igraph_int_t`.
  - The split-graph layout follows igraph's own convention so the
    disable-incident-arcs phase is bit-exact: vertex `v` maps to
    `v_out = v` (output half, first `n` vertices) and
    `v_in = v + n` (input half, last `n` vertices). For undirected
    input we apply the same `IGRAPH_TO_DIRECTED_MUTUAL` conversion as
    upstream — every edge becomes two arcs.
  - Whitney 1932 bound: `st_vertex_connectivity(s, t) ≤
    st_edge_connectivity(s, t)` on every input, verified by a proptest
    invariant against [`st_edge_connectivity`].
  - 15 unit tests verbatim-replay
    `tests/unit/igraph_st_vertex_connectivity.c` (path 6v `vc(0,5)=1`,
    2v direct-edge under all four `VconnNei` modes, K_6 IGNORE
    `vc(0,1)=4`, parallel directed paths `vc(0,3)=2`, isolated
    endpoints `vc=0`) plus error-contract tests for
    `source == target`, out-of-range endpoints, and `vcount < 2`.
    2 proptests: `vc_le_ec` (Whitney bound) and
    `disconnected_returns_zero`.
  - Three-source conformance: 9 C fixtures (full unit test replay),
    2 py fixtures (`Graph.vertex_connectivity` with a `source`/`target`
    pair), 2 R fixtures (`igraph::vertex_connectivity(graph, source =,
    target =)` per `test-flow.R:138`).
  - Bench: `bench_st_vertex_connectivity` (textbook 6v path + layered
    `L4×W8`, `L6×W16`, `L8×W32`). Headline: textbook = 7.25 µs,
    L4×W8 = 692 µs, L6×W16 = 22 ms, L8×W32 = 728 ms. The blow-up vs.
    FL-012 (130×→8300×) is structural — the split graph doubles
    vertex count and the disable-incident-arcs phase forces zero-cap
    passes that FL-012 skips. See
    `.codefuse/tracking/perf/ALGO-FL-013.json` for the breakdown.
  - Example: `cargo run --example st_vertex_connectivity_demo`.
  - Re-exports: `rust_igraph::{VconnNei, st_vertex_connectivity}` and
    `rust_igraph::algorithms::flow::st_vertex_connectivity::{VconnNei,
    st_vertex_connectivity}`.

- **ALGO-FL-012** — `edge_disjoint_paths`: maximum number of pairwise
  edge-disjoint `source → target` paths. Mirrors
  `igraph_edge_disjoint_paths` at
  `references/igraph/src/flow/flow.c:2326`, a 15-line wrapper that
  rejects `source == target` and then delegates to
  `igraph_maxflow_value` with `NULL` capacity (unit capacities per
  edge), casting the resulting `igraph_real_t` to `igraph_int_t`.
  By **Menger's theorem** (1927) the value equals
  [`st_edge_connectivity`] on every input and equals
  `round(max_flow_value(g, s, t, None))` on unit caps.
  - `pub fn edge_disjoint_paths(graph: &Graph, source: VertexId,
    target: VertexId) -> IgraphResult<i64>`. Returns `i64` to match
    igraph C's `igraph_int_t`. Error contract inherits from
    [`max_flow_value`]: `VertexOutOfRange` for bad source/target,
    `InvalidArgument` for `source == target`, and the same `Internal`
    guard if the unit-cap max-flow value escapes `i64` range
    (unreachable in practice — `ecount() ≤ u32::MAX`).
  - Same wrapper shape as `st_edge_connectivity` (FL-011); kept as a
    separate Rust function for naming parity with igraph C so call
    sites can pick the name that matches their intent
    ("paths" vs. "connectivity").
  - 11 unit tests verbatim-replay `tests/unit/igraph_edge_disjoint_paths.c`
    (6v directed fixture with self-loop at vertex 3, asserts
    ep(0→5)=2, ep(0→3)=1, ep(3→0)=0, ep(3→5)=2; undirected variant
    asserts ep(4→3)=3) plus a belt-and-suspenders
    `matches_st_edge_connectivity` invariant on K_5; 1 proptest
    `menger_equals_st_edge_connectivity` cross-checks Menger equality
    on random unit-cap graphs (transitively inheriting FL-002's
    Edmonds-Karp oracle).
  - Three-source conformance: 5 C fixtures from the dedicated unit
    test, 2 py fixtures via `Graph.edge_disjoint_paths` (which
    python-igraph aliases to `Graph.edge_connectivity` in
    `src/igraph/__init__.py:342`), 2 R fixtures from
    `tests/testthat/test-flow.R:183-189`.
  - Bench: `bench_edge_disjoint_paths` (textbook 6v/8e + layered
    `L4×W8`, `L6×W16`, `L8×W32`). Headline: textbook = 402 ns,
    L4×W8 = 5.3 µs, L6×W16 = 21.3 µs, L8×W32 = 87.8 µs. Same-session
    re-bench of FL-011 confirms the two wrappers run at identical
    speed within criterion's confidence interval, as expected
    (pointwise the same code path).
  - Example: `cargo run --example edge_disjoint_paths_demo`.
  - Re-exports: `rust_igraph::edge_disjoint_paths` and
    `rust_igraph::algorithms::flow::edge_disjoint_paths`.

- **ALGO-FL-011** — `st_edge_connectivity`: minimum number of edges
  whose removal disconnects `source` from `target`. Mirrors
  `igraph_st_edge_connectivity` at
  `references/igraph/src/flow/flow.c:2219`, a 15-line wrapper that
  rejects `source == target` and then delegates to
  `igraph_maxflow_value` with `NULL` capacity (unit capacities per
  edge), casting the resulting `igraph_real_t` to `igraph_int_t`.
  By the **max-flow / min-cut theorem** on unit capacities the value
  equals `round(max_flow_value(g, s, t, None))`.
  - `pub fn st_edge_connectivity(graph: &Graph, source: VertexId,
    target: VertexId) -> IgraphResult<i64>`. Returns `i64` to match
    igraph C's `igraph_int_t`. Error contract inherits from
    [`max_flow_value`]: `VertexOutOfRange` for bad source/target,
    `InvalidArgument` when `source == target`. An `Internal` error
    fires only on the unreachable case of a non-integer / negative /
    `> i64::MAX` flow value (defensive against floating-point drift).
  - **Tests / conformance.** 11 unit tests in
    `src/algorithms/flow/st_edge_connectivity.rs` (input validation,
    isolated endpoints → 0, single edge = 1, two parallel paths = 2,
    three parallel arcs = 3, directed anti-parallel arc isolation,
    undirected bottleneck, `K_5` undirected, igraph C unit-test
    fixture replay). 1 proptest cross-validates
    `st_edge_connectivity(g, s, t) == round(max_flow_value(g, s, t,
    None))` on random unit-cap graphs. 6 three-source conformance
    fixtures wired into `tests/conformance.rs`
    (`st_edge_connectivity_three_source_conformance`):
    - C × 2 — verbatim mirror of
      `tests/unit/igraph_st_edge_connectivity.c:23-38` + an
      undirected-path structural case.
    - py × 2 —
      `tests/test_flow.py:CutTests.testEdgeConnectivity:18`
      (`g.edge_connectivity(0, 3) == 2`) + `Graph.Full(5)` sanity.
    - R × 2 — `test-flow.R:148, 153` (`make_full_graph(5)` →
      ec = 4; directed path `make_ring(5, directed=TRUE,
      circular=FALSE)` source=1→target=3 → ec = 1; both shifted to
      0-indexed for Rust).
  - **Performance.** Criterion at `benches/bench_st_edge_connectivity.rs`.
    Headline numbers (mean, 1.10 GHz · macOS arm64, `lto = thin`,
    `codegen-units = 1`):
    - textbook 6v / 8e directed: **1.10 µs**
    - layered L4 × W8 (74e):  **12.1 µs**
    - layered L6 × W16 (288e):  **46.7 µs**
    - layered L8 × W32 (1088e): **239.9 µs**
    Slightly faster than the equivalent FL-010 mincut bench on the
    textbook (1.57 µs) because unit-capacity `Network::build`
    short-circuits the per-edge `capacity[e]` lookup.
  - **Runnable example.** `examples/st_edge_connectivity_demo.rs` —
    runs the C unit-test fixture, two-parallel-paths, K_5, and the
    disconnected-endpoint case. Run with
    `cargo run --example st_edge_connectivity_demo`.

- **ALGO-FL-010** — `st_mincut_value`: scalar s-t minimum-cut value
  (capacity of the smallest edge set whose removal disconnects
  `source` from `target`). Mirrors `igraph_st_mincut_value` at
  `references/igraph/src/flow/flow.c:1127`, a 5-line wrapper
  delegating to `igraph_maxflow_value`. By the
  **max-flow / min-cut theorem** (Ford-Fulkerson, 1956) the value
  equals [`max_flow_value`]; this function exists for naming parity
  with igraph C and for intent-revealing call sites.
  - `pub fn st_mincut_value(graph: &Graph, source: VertexId, target:
    VertexId, capacity: Option<&[f64]>) -> IgraphResult<f64>`. Same
    arguments, same error contract, same precision as
    [`max_flow_value`] (delegation is verbatim — no extra
    validation, no extra arithmetic).
  - **Tests / conformance.** 10 unit tests in
    `src/algorithms/flow/st_mincut.rs` (source/target validation,
    isolated endpoints → 0, single-edge cut, two parallel paths,
    directed bottleneck, CLRS 26.1-1 = 23, undirected reference,
    `equals_max_flow_value` belt-and-suspenders). 1 proptest
    invariant (`mincut_equals_maxflow`, 256 cases) re-asserts duality
    at the value level on random `Graph` fixtures; combined with the
    Edmonds-Karp cross-validation already on `max_flow_value`, the
    cut value inherits independent reference-checking transitively.
    5 three-source conformance fixtures
    (`tests/conformance/{c,py,r}/st_mincut_value/`): C 6-vertex
    directed weighted from `tests/unit/igraph_st_mincut_value.c:23`
    (cut=7) + 4-vertex undirected unit (cut=2); python-igraph
    `mincut_value(0, 3, ...)` from `test_flow.py:74-75`; R
    `min_cut(g, source, target, capacity)` from
    `test-flow.R:111-128`.
  - **Perf.** Wrapper overhead is measurable as zero: on the
    textbook CLRS fixture, back-to-back criterion runs measure
    `max_flow_value = 1.61 µs` vs `st_mincut_value = 1.57 µs` (well
    within sample variance). Layered fixtures: 4×8 = 16 µs, 6×16 =
    77 µs, 8×32 = 286 µs. Snapshot in
    `.codefuse/tracking/perf/ALGO-FL-010.json`. Runnable demo:
    `cargo run --example st_mincut_demo`.

- **ALGO-FL-002** — `max_flow_value`: scalar maximum-flow value from
  `source` to `target` via **Dinic's algorithm** (BFS level-graph +
  DFS blocking-flow with current-arc pointer optimisation, O(V²·E)
  worst case). igraph C exposes the same function over a
  Goldberg-Tarjan push-relabel solver; the scalar max-flow is unique
  (Ford-Fulkerson + max-flow / min-cut duality), so the value matches
  exactly on integer capacities and within `1e-12` on `f64`.
  - `pub fn max_flow_value(graph: &Graph, source: VertexId, target:
    VertexId, capacity: Option<&[f64]>) -> IgraphResult<f64>`.
    `capacity` defaults to unit on every edge when `None`; otherwise
    its length must equal `graph.ecount()` and entries must be finite
    and `≥ 0`. Errors: [`IgraphError::VertexOutOfRange`] on out-of-
    range endpoints; [`IgraphError::InvalidArgument`] for
    `source == target`, mismatched capacity length, or negative /
    non-finite capacity.
  - **Undirected handling matches igraph C.** Each undirected edge
    `(i, j)` of capacity `c` becomes two opposing directed arcs of
    capacity `c` in the residual network, so e.g. a single
    undirected `0-3` edge with capacity 1 admits unit flow in either
    orientation.
  - **Residual network layout.** Flat arc array stored in pairs
    `(2k, 2k+1)`; the reverse arc of arc `i` is `i ^ 1`. Per-vertex
    `iter[v]` pointer skips saturated arcs during the DFS blocking
    phase so each arc is visited at most twice per phase. No `unsafe`,
    no allocations in the inner loop beyond the BFS queue / DFS
    recursion.
  - **Tests / conformance.** 16 unit tests in
    `src/algorithms/flow/max_flow.rs` (input validation, single-edge
    directed/undirected, parallel-path bottleneck, CLRS 26.1-1
    textbook = 23, multigraph parallel edges = 7, self-loop, weighted
    fractional flow). 2 proptest invariants
    (`matches_edmonds_karp_directed/undirected`, 80 cases each) cross-
    validate against a hand-rolled Edmonds-Karp reference inside the
    test module. 5 three-source conformance fixtures
    (`tests/conformance/{c,py,r}/max_flow_value/`): C 4-vertex
    undirected unit + weighted from `tests/unit/igraph_maxflow.c:213`,
    Python mirror of `test_flow.py:36-40`, R 6-vertex directed from
    `test-flow.R:111-128`.
  - **Perf.** CLRS instance: 5.4 µs; layered 4×8 unit-cap: 53 µs;
    layered 6×16: 414 µs; layered 8×32: 374 µs. Numbers from
    `cargo bench --bench bench_max_flow -- --quick` on Darwin/M-series
    release build. Snapshot in
    `.codefuse/tracking/perf/ALGO-FL-002.json`. Runnable demo:
    `cargo run --example max_flow_demo`.

- **ALGO-PR-011c** — `pagerank_linsys`: second backend for
  [`pagerank`] that casts the same fixed point as a non-singular
  linear system and solves it with restarted GMRES. The Google
  matrix `G = (1 - α)/N · 1·1ᵀ + α · M` gives `pr = G · pr`, which
  rearranges to `(I - α · Mᵀ) · pr = (1 - α)/N · 1`; `α < 1` keeps
  the operator non-singular, so a Krylov solver converges in at most
  `O(spectral_gap)` Arnoldi steps without ever materialising any
  matrix.
  - `pub fn pagerank_linsys(graph: &Graph) -> IgraphResult<Vec<f64>>` —
    same signature as [`pagerank`]. Returns a length-`vcount`
    probability vector summing to ~1, agreeing with the power-iter
    backend within `1e-9` elementwise on every shared conformance
    fixture (the limit comes from PR-011's `eps = 1e-10` stopping
    rule; GMRES itself reaches the true fixed point to ≤ machine
    precision).
  - **Hand-rolled GMRES, no new deps.** Restart dimension `m = 30`,
    `max_restarts = 50`, relative residual tolerance `1e-13`.
    Implementation: modified Gram-Schmidt Arnoldi, Givens rotations
    applied in-place to a flat row-major Hessenberg buffer, back-sub
    on the triangulated system, and a single
    `(m + 1) · |V|` flat `Vec<f64>` for the Krylov basis (no
    per-iteration `vec!`). Matvec is `O(|V| + |E|)`, fused-multiply-
    add over a pre-computed `inv_out_deg` lookup; dangling-vertex
    correction is baked in (one `O(|D|)` sweep per matvec).
  - **Edge cases match PR-011.** Empty graph → `vec![]`; singleton →
    `vec![1.0]`. Directed and undirected both supported (undirected
    self-loops contribute twice, identical to PR-011's convention).
  - **Conformance / tests.** 9 unit tests in
    `src/algorithms/properties/pagerank_linsys.rs` (incl. parity vs.
    `pagerank` on K4-minus-edge, 2-vertex dangling, and 7-leaf star);
    1 proptest (`pagerank_linsys_matches_power_iter` over
    `arb_graph(8)`) in `tests/property.rs`; 1 conformance test
    (`pagerank_linsys_three_source_conformance` in
    `tests/conformance.rs`) reusing the PR-011 `tests/conformance/
    {c,py,r}/pagerank/` fixtures (`1e-6` tolerance vs.
    python-igraph's ARPACK).
  - **Performance.** GMRES is `1.16–1.63×` slower than power
    iteration at `α = 0.85` (`bench_pagerank_linsys`):
    karate 18 µs / 12 µs, grid-30×30 769 µs / 472 µs, gnp(1000,
    p=0.05) 1.17 ms / 1.01 ms. The trade-off is genuinely
    algorithmic: power iteration's contraction factor at `α = 0.85`
    is fast enough that 142 matvecs are competitive with GMRES's
    12–30 Arnoldi steps plus `m²` orthogonalisation. The GMRES
    backend is the right pick when (a) you need a tighter residual
    than `eps = 1e-10` without inflating the iteration count, or
    (b) the spectral gap shrinks (e.g. `α → 1`); both regimes
    require parameterising `α`, which lands separately.
  - **Scope.** Unweighted only; a weighted GMRES backend would be
    PR-011d.
  - Bench scenarios: karate, 30×30 grid, gnp(1000, 0.05), each with
    a power-iter sibling for direct comparison. Perf snapshot lands
    in `.codefuse/tracking/perf/ALGO-PR-011c.json`. Example:
    `cargo run --release --example pagerank_linsys_demo` prints the
    top-10 vertices of karate under both backends with the max
    elementwise difference (`2.6e-11` on this fixture).

- **ALGO-PR-040** — `rich_club_sequence` per-vertex rich-club coefficient
  sequence over a user-supplied vertex peeling order. Faithful port of
  `igraph_rich_club_sequence` in
  `references/igraph/src/properties/rich_club.c:91-166`.
  - `pub fn rich_club_sequence(graph: &Graph, weights: Option<&[f64]>, vertex_order: &[VertexId], normalized: bool, loops: bool, directed: bool) -> IgraphResult<Vec<f64>>` —
    returns a `Vec<f64>` of length `vcount` whose `i`-th entry is the
    surviving edge count (or summed edge weight when `weights` is
    `Some`) of the subgraph that remains after peeling the first `i`
    vertices of `vertex_order` from the graph. With `normalized = true`,
    each entry is divided by `total_possible_edges(vcount - i, directed,
    loops)` — the same density denominator used by `igraph_density`.
  - **Strictly O(|V| + |E|), no actual peeling.** Inverts
    `vertex_order` once into `order_of[v]`, then for every edge `(v1,
    v2)` adds its weight to `res[min(order_of[v1], order_of[v2])]` —
    that bucket index is precisely the step at which the edge first
    disappears. One reverse cumulative pass turns the "removed-at-step"
    tally into the "surviving-after-step" sequence; an optional
    `O(|V|)` per-step normalization closes it out.
  - **`directed` flag silently coerced to `false` on undirected
    graphs**, matching upstream's `if (!igraph_is_directed(graph))
    directed = false;` line at `rich_club.c:118`. This keeps the
    denominator consistent with the (already undirected) edge tally.
  - **NaN as legitimate trailing sentinel.** In normalized mode the
    final entry `res[vcount - 1]` is `0 / 0 = f64::NAN` whenever the
    trailing single-vertex subgraph has zero possible edges (i.e.
    `!loops`), matching upstream's `print_vector` `( NaN )` output in
    `tests/unit/rich_club.out`. Conformance fixtures encode this as
    JSON `null`; the runner converts NaN→null on actual values before
    comparing.
  - **Permutation validation in the inverse-build pass.** Any
    out-of-range entry, duplicate, or omission is caught while building
    `order_of[]` and returned as `IgraphError::InvalidArgument`; no
    separate validation loop is needed.
  - **Loops + normalization edge case.** When `normalized && !loops`
    and the graph contains a self-loop, upstream issues a `Warning`
    (not an error) and continues with the loopless denominator. This
    port silently follows the same behaviour: the caller opted in to
    "assume no loops", so the responsibility for that assumption is on
    them.
  - Full 9-step AWU SOP. **Unit tests** (16 cases): triangle K3 raw/
    normalized; path P3 in-order/reverse-order; weighted contribution;
    undirected coercion of `directed`; permutation validation
    (length/out-of-range/duplicate); `weights` length check;
    `vcount == 0` empty; `vcount == 1` NaN; `total_possible_edges`
    coverage of all four (loops, directed) corners. **Proptest
    invariants**: order-irrelevance of edge enumeration (any vertex
    re-labeling that preserves degree sequence yields the same
    sequence), monotone non-increasing raw counts as `i` grows, and
    consistency between weighted-by-ones and unweighted modes.
  - **Three-source conformance** (9 fixtures, all pass):
    `tests/conformance/c/rich_club_sequence/` mirrors `rich_club.out`
    Tests 3a (undirected/no-loop/in-order), 6a (directed/loop/in-order),
    and 7a (weighted ×2 all-ones); `tests/conformance/py/` carries
    three hand-derived parity cases (K3, P4, weighted star K_{1,3})
    since python-igraph 0.11.9 does not expose a binding for
    `rich_club_sequence`; `tests/conformance/r/` mirrors
    `_snaps/aaa-auto.md:3068-3081` for the path-3 normalized and
    unnormalized-with-loops snapshots plus a hand-derived K4 reverse
    order case.
  - **Criterion bench** at `benches/bench_rich_club.rs` with karate
    (~336 ns), 30×30 grid (~5.6 µs), and G(1000, 0.05) (~33.7 µs).
    Baseline snapshot at `.codefuse/tracking/perf/ALGO-PR-040.json`.
  - **Runnable demo** at `examples/rich_club_karate.rs` — peels karate
    by ascending degree and prints the rising rich-club density curve.
- **ALGO-GN-029** — `citing_cited_type_game` citing×cited-type growing
  citation network. **Twenty-ninth member of the `games/` family** and
  a faithful port of `igraph_citing_cited_type_game` in
  `references/igraph/src/games/citations.c:340-499`.
  - `pub fn citing_cited_type_game(nodes: u32, types: &[u32], pref: &[&[f64]], edges_per_step: u32, directed: bool, seed: u64) -> IgraphResult<Graph>` —
    generalises `cited_type_game` by letting the **citing** vertex's
    type also matter: vertex `i` cites `edges_per_step` earlier
    vertices, drawing each target from a per-citing-type Fenwick BIT
    weighted by `pref[type[i]][type[j]]`. Maintains `T = max(types)+1`
    parallel `PsumTree`s of size `nodes`; pre-seeds vertex 0 into all
    trees so the first step has a non-empty draw pool, then per step
    `i` does `edges_per_step` `search_bounded` calls on
    `sumtrees[type[i]]` (or uniform `[0, i)` when `sums[type[i]] == 0`,
    matching the C reference fallback at `citations.c:444-460`), and
    finally inserts vertex `i` into all `T` trees with weight
    `pref[j][type[i]]`. Cost per step: `O(eps · log n)` for sampling +
    `O(T · log n)` for the per-step append.
  - **No self-loops, structurally**: even in the uniform-fallback
    regime the RNG samples `[0, i)` BEFORE vertex `i` is appended to
    the trees, so the citing vertex can never select itself. Asserted
    in unit `positive_pref_never_self_loops` and proptest
    `no_self_loops_when_pref_positive`.
  - **Multi-edges ARE allowed** when `edges_per_step ≥ 2` (two draws
    at one step may pick the same target); simplicity is therefore
    NOT asserted on output and not enforced by `add_edges` (which is
    a multigraph append in this crate).
  - **Strict input validation up front**, mirroring the C reference:
    `types.len() == nodes`, `pref` has `T` rows × `T` cols (square),
    every `pref[j][k]` is finite (rejects `NaN`/`±Inf`) and
    non-negative, and `edges_per_step == 0` short-circuits to an empty
    graph. All error paths covered by dedicated unit tests.
  - **Per-module private `PsumTree`** following the established crate
    pattern (see `barabasi_psumtree.rs:96-97`) — the Fenwick BIT is
    self-contained per game module to keep diffs focused and avoid
    premature abstraction.
  - **Conformance**: three-source structural fixtures (3 from `c`, 3
    from `py`, 3 from `r`) under
    `tests/conformance/{c,py,r}/citing_cited_type_game/`. RNG state is
    not portable across implementations, so each fixture pins
    parameters and asserts `vcount` exact, `directed` flag exact,
    `ecount` in band `[min, max]` (both bounds equal to `(n-1)·eps`
    for positive pref), and `no_self_loops` whenever flagged.
  - **Coverage**: 25 in-module tests (20 unit + 5 proptest) hit error
    paths (length, shape, NaN, Inf, negative), structural invariants
    (`source = i`, `dst < src` for directed), regime checks
    (assortative concentration, off-diagonal cross-type, all-zero-row
    fallback to uniform). Plus 1 integration test on 9 fixtures.
  - Demo: `cargo run --example citing_cited_type_demo` shows the
    citation-flow matrix concentrating ≈ 98 % on the diagonal under a
    `diag(10) + off(0.1)` 3×3 pref — within 0.4 pp of the steady-state
    target `10 / (10 + 0.1·2) = 98.0 %`.
  - Bench: `cargo bench --bench bench_citing_cited_type` — at the
    `n=5_000`, `eps=3`, `T=4` operating point the generator runs at
    `≈ 2.88 Melem/s`; per-step cost grows linearly in `eps` and only
    weakly (`~T·log n`) in the number of types. Baseline snapshot at
    `.codefuse/tracking/perf/ALGO-GN-029.json`.

- **ALGO-CN-030** — `weighted_adjacency` dense `f64` adjacency-matrix
  constructor returning `(Graph, Vec<f64>)`. **Thirtieth member of the
  `constructors/` family** and a faithful port of
  `igraph_weighted_adjacency` in
  `references/igraph/src/constructors/adjacency.c:472-787`.
  - `pub fn weighted_adjacency(matrix: &[&[f64]], mode: AdjacencyMode, loops: LoopsMode) -> IgraphResult<(Graph, Vec<f64>)>` —
    given an `n × n` real-valued matrix interpreted as weighted
    adjacency entries (each non-zero value is a single weighted edge,
    not a multiplicity), an [`AdjacencyMode`] selecting one of seven
    dispatch flavours (`Directed`, `Undirected`, `Max`, `Min`, `Plus`,
    `Upper`, `Lower`), and a [`LoopsMode`] controlling self-loop
    emission, builds the unique weighted graph whose `(edge, weight)`
    multiset matches the per-mode rule, returning both the topology
    and the per-edge weight vector aligned with `Graph::edge(eid)`.
  - **Shares `AdjacencyMode` / `LoopsMode` with CN-029** — same enum
    semantics, same `Twice → Once` per-mode collapse (upstream
    `adjacency.c` line 558, 619, 654 for `Directed`/`Upper`/`Lower`),
    same `Undirected` symmetry guard. The only error path beyond
    CN-029's structural set is **NaN-tolerant asymmetry**: under
    `Undirected`, two entries `m[i][j]` and `m[j][i]` that are both
    `NaN` count as symmetric (matching IEEE 754 weighted-edge
    convention where `NaN ≡ NaN` for "no information"), while one
    `NaN` against any non-`NaN` triggers `InvalidArgument`.
  - **NaN propagation in `Max` / `Min`**: weight comparison via
    `m1 < m2 || m2.is_nan()` (so any `NaN` operand wins) — matches
    upstream `adjacency.c:586-594` exactly. `Plus` mode just sums
    `m[i][j] + m[j][i]`; `NaN + finite = NaN` is the standard IEEE
    propagation.
  - **Loop weight adjustment**: `NoLoops` zeroes the diagonal in
    place ahead of the off-diagonal walk; `Once` passes the diagonal
    entry through unhalved; `Twice` halves the diagonal weight (`w/2`)
    for `Undirected` / `Max` / `Min` / `Plus`, then becomes `Once`
    semantics (unhalved) for `Directed` / `Upper` / `Lower` per the
    upstream collapse — odd-integer diagonals on symmetric modes
    therefore do **not** error here (unlike CN-029's integer-only
    `Twice`-divisibility guard) because real-valued halving is
    well-defined.
  - **Type-level error elimination**: accepting `&[&[f64]]` rather
    than `igraph_matrix_t` makes "wrong element type" statically
    unreachable; the surviving structural errors are ragged-matrix,
    non-square (caught by `if row.len() != nrow`), `NaN`-vs-finite
    asymmetry under `Undirected`, and `usize::MAX`-class overflow on
    the edge-buffer pre-size.
  - **Overflow protection**: `u32::try_from(nrow)` so adversarial
    `nrow > u32::MAX` reports `InvalidArgument` rather than
    truncating; `usize::checked_mul(nrow, nrow)` on the maximum
    possible edge count; `u32::try_from` on every `usize → u32`
    cast for edge endpoints.
  - **Tests**: 29 unit + 6 proptest covering all 7 × 3 mode/loops
    combinations on the M3 family from
    `references/igraph/tests/unit/igraph_weighted_adjacency.c`, the
    empty `0 × 0` and singleton `1 × 1` shapes, the `Twice → Once`
    collapse on `Directed`/`Upper`/`Lower`, the `Twice` halving on
    `Undirected`/`Max`/`Min`/`Plus`, all 4 error paths
    (ragged / non-square / `NaN`-vs-finite asymmetric `Undirected` /
    `nrow > u32::MAX`), and proptest invariants (`Max(A, A) == A`
    weighted, `Min(A, A) == A` weighted, `Undirected ≡ Max(A, Aᵀ)`
    on symmetric `A`, `Plus(A) sums to 2·trace(A)` on the diagonal
    under `Twice`, etc.).
  - **Conformance**: 18 fixtures across all 3 source bindings
    (C:11 / py:4 / R:3) — same fixture density as CN-029.
    python-igraph and R-igraph expose only the `Once` semantics
    (no native `Twice` mode), so those lanes only cover the
    `NoLoops` / `Once` paths. Comparison is via directed-aware
    canonical `((u, v), weight_bits)` multiset, with `NaN` weights
    bit-compared via `f64::to_bits()` + `u64::MAX` sentinel so two
    `NaN`s count as equal.
  - **Bench**: M3 (3×3) shapes 343-668 ns (~5-10 Melem/s — per-call
    overhead dominates); n=128 sparse 14.7 µs (~8.7 Melem/s);
    n=128 dense directed ~468 µs (~35 Melem/s — L1 sweet spot);
    n=128 dense `Max` ~211 µs (~78 Melem/s — symmetric walk emits
    half the edges); n=512 dense `Max` (~130k edges) ~4.25 ms
    (~31 Melem/s). Perf snapshot at
    `.codefuse/tracking/perf/ALGO-CN-030.json`.
  - **Example**: `cargo run --example weighted_adjacency_demo` walks
    a small weighted matrix through every mode/loops combination,
    illustrating how `Max` vs `Min` vs `Plus` produce very different
    weight aggregations from the same input.

- **ALGO-CN-029** — `adjacency` dense integer adjacency-matrix
  constructor. **Twenty-ninth member of the `constructors/` family**
  and a faithful port of `igraph_adjacency` in
  `references/igraph/src/constructors/adjacency.c:75-386`.
  - `pub fn adjacency(matrix: &[&[i64]], mode: AdjacencyMode, loops: LoopsMode) -> IgraphResult<Graph>` —
    given an `n × n` integer matrix whose entries are non-negative
    edge multiplicities, an [`AdjacencyMode`] selecting one of seven
    dispatch flavours (`Directed`, `Undirected`, `Max`, `Min`, `Plus`,
    `Upper`, `Lower`), and a [`LoopsMode`] controlling how diagonal
    entries become self-loops (`NoLoops` / `Once` / `Twice`), builds
    the unique graph whose edge multiset matches the per-mode rule.
  - **Type-level error elimination**: the upstream C entry takes
    `igraph_matrix_t` (a dense `igraph_real_t` matrix that may carry
    NaN or non-integer entries despite being interpreted as
    multiplicities); accepting `&[&[i64]]` makes both paths
    statically unreachable. The surviving structural errors are
    ragged-matrix, negative-entry, asymmetric-matrix (`Undirected`
    only), and odd-diagonal under `Twice` on symmetric modes — all
    returning `IgraphError::InvalidArgument`.
  - **Per-mode `Twice → Once` collapse**: matches upstream
    `adjacency.c` lines 84, 168, 209 — for `Directed`, `Upper`, and
    `Lower` modes the matrix only stores one copy of each loop, so
    `LoopsMode::Twice` is silently treated as `LoopsMode::Once` for
    those layouts (the diagonal entry passes through unhalved).
  - **`Undirected` mode** requires `A == Aᵀ` and then delegates to
    `Max` after the symmetry check (consumers explicitly opting into
    `multiplicity = max(A(i,j), A(j,i))` semantics for the
    undirected case).
  - **Overflow protection**: `u32::try_from(nrow)` so adversarial
    `nrow > u32::MAX` reports `InvalidArgument` rather than
    truncating; `usize::checked_add` / `usize::checked_mul` on the
    edge buffer pre-size; `u32::try_from` on every cross-domain
    `i64 → u32` cast.
  - **Tests**: 20 unit + 6 proptest covering all 7 × 3 mode/loops
    combinations on the M3 family from
    `references/igraph/tests/unit/igraph_adjacency.c`, the empty
    `0 × 0` and singleton `1 × 1` shapes, the `Twice → Once` collapse
    on `Directed`/`Upper`/`Lower`, all 4 error paths
    (ragged / negative / asymmetric `Undirected` / odd-diagonal
    `Twice` on symmetric mode), and proptest invariants
    (`Max(A, A) == A`, `Min(A, A) == A`, `Upper + Lower` reproduces
    asymmetric `Directed` off-diagonal walk, etc.).
  - **Conformance**: 18 fixtures across all 3 source bindings
    (C:11 / py:4 / R:3); python-igraph and R-igraph expose only the
    `Once` semantics (no native `Twice` mode), so those lanes only
    cover the `NoLoops` / `Once` paths. Comparison is via
    directed-aware canonical edge multiset.
  - **Bench**: M3 (3×3) shapes 550-860 ns (~10-20 Melem/s — per-call
    overhead dominates); n=128 sparse directed 27.3 µs
    (~4.7 Melem/s — n² traversal dominates over emission); n=128
    dense directed ~466 µs / dense MAX ~217 µs (~35-37 Melem/s — L1
    sweet spot); n=512 dense MAX (~130k edges) ~4.32 ms
    (~30 Melem/s). Perf snapshot at
    `.codefuse/tracking/perf/ALGO-CN-029.json`.
  - **Example**: `cargo run --example adjacency_demo` walks the three
    M3 matrices through every mode/loops combination, illustrating
    how the same matrix produces very different graphs depending on
    the dispatch.

- **ALGO-CN-028** — `extended_chordal_ring` extended-chordal-ring
  constructor. **Twenty-eighth member of the `constructors/` family**
  and a faithful port of `igraph_extended_chordal_ring` in
  `references/igraph/src/constructors/regular.c:868-963`.
  - `pub fn extended_chordal_ring(nodes: u32, w: &[&[i64]], directed: bool) -> IgraphResult<Graph>` —
    given `nodes`, a chord-offset matrix `w` (each row defines a
    family of chord offsets with row length = "period" `p`, which
    must divide `nodes`), and a `directed` flag, builds the union of
    (a) the cycle `C_nodes` and (b) for every vertex `i ∈ [0, nodes)`
    and every row `r ∈ [0, m)` of `w`, the chord edge
    `(i, (i + w[r][i mod p]) mod nodes)`. Negative offsets are
    accepted and handled via `i64::rem_euclid`, so `w = [[-3]]` for
    `n = 5` produces the same chord set as `w = [[+2]]`.
  - **Type-level error elimination**: the upstream C entry takes
    `igraph_matrix_t` (a 2-D dense matrix accepting non-integer / NaN
    entries that are then interpreted as offsets); accepting
    `&[&[i64]]` makes that path statically unreachable. The only
    structural error that survives is "ragged matrix" (rows of
    different length), which returns
    `IgraphError::InvalidArgument("extended_chordal_ring: matrix W
    must have rows of equal length")`.
  - **Degenerate inputs handled inline ahead of any work**:
    - `nodes < 3` → `InvalidArgument("extended_chordal_ring: nodes
      must be at least 3")` (upstream cycle-too-short guard).
    - `nrow(w) == 0` (empty matrix) → pure cycle `C_nodes`; chord
      pass is skipped.
    - `period == 0` (row exists but has length 0) →
      `InvalidArgument("extended_chordal_ring: period (W row length)
      must be positive when W has rows")`.
    - `nodes % period != 0` →
      `InvalidArgument("extended_chordal_ring: period must divide
      nodes")`.
  - **Overflow protection**: `usize::checked_mul(nodes as usize,
    nrow + 1)` for the edge buffer capacity (adversarial
    `nodes = u32::MAX` reports `Overflow` rather than silently
    truncating); `u32::try_from` on every chord target; `i64::rem_euclid`
    for safe negative-offset modular arithmetic.
  - **Tests**: 17 unit + 6 proptest covering pentagram pos/neg
    equivalence (`(5, [[+2]], dir) ≡ (5, [[-3]], dir)` byte-for-byte
    on canonical multiset), article-12-multigraph 36-edge byte-for-byte
    reproduction of upstream
    `references/igraph/tests/unit/igraph_extended_chordal_ring.c`
    case 2, pure-cycle invariant when `W = []` across
    `n ∈ {3, 5, 8, 12}`, every guard path (`nodes < 3`, `period == 0`,
    `period ∤ nodes`, ragged `W`), and the closed-form
    `ecount == nodes + nodes · nrow(w)` invariant in proptest sweeps
    over `nodes ∈ [3, 20], period ∈ [1, nodes], nrow ∈ [0, 3]`,
    `offsets ∈ [-50, 50]`.
  - **Conformance**: python-igraph 0.11.x exposes **no**
    `Graph.Extended_Chordal_Ring` binding (verified via
    `hasattr(igraph.Graph, 'Extended_Chordal_Ring') == False`
    against the live `.venv/` install), so this AWU runs a
    **two-source** conformance test (the precedent set by
    CN-024 `hexagonal_lattice` and CN-027 `turan` for absent-binding
    lanes). R-igraph exposes `make_chordal_ring(n, w, directed = FALSE)`
    sharing the upstream C entry. Fixture catalogue: C:3 (the three
    canonical cases from upstream `igraph_extended_chordal_ring.c` —
    pentagram-positive, pentagram-negative-equivalent, article
    multigraph), py:0 (no binding), R:3
    (`make_chordal_ring(8, matrix(2))`,
    `make_chordal_ring(9, matrix(c(2,3,4), 1, 3))`,
    `make_chordal_ring(10, matrix(c(2,3,4,5), 2, 2))`).
  - **Performance**: O(|V| + |E|) — single cycle emission + a two-deep
    per-vertex per-row chord pass. Throughput sweep on M-series
    silicon: n=5 pentagram (10 edges) 11.6 Melem/s, n=12 article
    multigraph (36 edges) 18.6 Melem/s, n=256 single-row (512 edges)
    27.8 Melem/s, n=512 period-4 3-row (2048 edges) 28.5 Melem/s
    (sweet spot — edge buffer L1-resident), n=4096 single-row
    (8192 edges) 24.3 Melem/s, n=2048 period-4 3-row (8192 edges)
    24.5 Melem/s (~8k-edge buffer pushes out of L1 but stays
    L2-resident). No `python-igraph` baseline by construction.

- **ALGO-CN-027** — `turan` Turán graph constructor returning
  `FullMultipartite { graph, types }` so the partition layout is
  recoverable from the result. **Twenty-seventh member of the
  `constructors/` family** and a thin balanced wrapper over
  [`full_multipartite`]. Counterpart of `igraph_turan` in
  `references/igraph/src/constructors/full.c:281-325`.
  - `pub fn turan(n: u32, r: u32) -> IgraphResult<FullMultipartite>` —
    builds the unique (up to isomorphism) `n`-vertex graph that
    maximises the number of edges subject to containing no clique of
    size `r + 1` (Turán's 1941 theorem). Concretely the complete
    `r'`-partite graph with maximally balanced partition sizes, where
    `r' = min(r, n)`, `q = n / r'`, `s = n % r'`, the first `s`
    partitions get size `q + 1`, and the remaining `r' − s` partitions
    get size `q`. Always undirected (matching upstream — Turán graphs
    are not defined as directed objects).
  - **Implementation**: thin wrapper over
    `full_multipartite(&partitions, false, MultipartiteMode::All)`. The
    only original work is the `Vec<u32>` partition build via a single
    `for i in 0..r_effective` loop that pushes `q + 1` when
    `i < remainder` and `q` otherwise (matching upstream's
    `igraph_vector_int_fill(quotient)` + per-index `++` post-loop). All
    inter-partition edge emission is delegated to CN-026, so the inner
    work is byte-for-byte identical to `full_multipartite`'s 4-deep
    source-major walk.
  - **Degenerate inputs handled inline ahead of any work**:
    - `n == 0` short-circuits to `full_multipartite(&[], false, All)`
      returning the empty graph with empty `types`, regardless of `r`
      (matching upstream's `if (n == 0) return IGRAPH_SUCCESS` after
      `igraph_empty`).
    - `r == 0 ∧ n > 0` returns
      `IgraphError::InvalidArgument("turan: number of partitions must
      be positive when n > 0")` (mirroring upstream's `IGRAPH_EINVAL`).
    - `r > n` silently caps to `r_effective = n` yielding `K_n` with
      `r' = n` singleton partitions and `types = [0, 1, …, n − 1]`
      (matching upstream's `r = n` reassignment).
  - **Overflow protection**: `quotient.checked_add(1)` for the
    `q + 1` partition size so adversarial inputs cannot silently wrap;
    the wrapped `full_multipartite` carries the heavier
    `u64.checked_add` for `n_acc` prefix sums and `usize::checked_mul`
    for the edge buffer capacity.
  - **Tests**: 15 unit + 6 proptest covering (a) all 6 upstream `.out`
    cases from `references/igraph/tests/unit/igraph_turan.out` byte-
    for-byte (`igraph_turan(0, 10)` empty guard, `igraph_turan(10, 1)`
    10 isolated vertices, `igraph_turan(4, 6)` capped to `K_4`,
    `igraph_turan(13, 4)` with `types = [0,0,0,0,1,1,1,2,2,2,3,3,3]`
    and 63 edges, `igraph_turan(8, 3)` with `types = [0,0,0,1,1,1,2,2]`
    and 21 edges, `igraph_turan(6, 3)` the octahedron `K_{2,2,2}` with
    12 edges), (b) the closed-form `E = ½ · Σ n_i · (N − n_i)` cross-
    check for `T(13, 4) = 63`, `T(8, 3) = 21`, `T(6, 3) = 12`,
    `T(12, 4) = 54`, (c) the partition-size-differs-by-at-most-one
    Turán-balance invariant, (d) the types-vector-is-monotone partition-
    major invariant, (e) the no-edge-lives-inside-a-single-partition
    multipartite invariant, (f) the always-undirected invariant, (g)
    `T(n, n + extra) ≡ T(n, n)` for every `extra ≥ 0` (the r-capping
    invariant), (h) `T(13, 4)` is isomorphic to
    `full_multipartite(&[4, 3, 3, 3], false, All)` byte-for-byte (same
    edge multiset + same types), plus a proptest sweep across
    `n ∈ [0, 30], r ∈ [1, 10]`.
  - **Two-source conformance** (no `Graph.Turan` in python-igraph):
    9 fixtures.
    - python-igraph 0.11.x exposes NO `Graph.Turan` binding —
      `hasattr(igraph.Graph, 'Turan') == False` against the live
      `.venv/` installation — so this AWU runs the C + R two-lane
      pattern (mirroring the precedent set by CN-024 hexagonal_lattice
      for absent-binding lanes).
    - C lane (6) — the 6 `.out` cases from
      `references/igraph/tests/unit/igraph_turan.out`
      (`igraph_turan(0, 10)` empty, `igraph_turan(10, 1)` 10 isolated,
      `igraph_turan(4, 6)` capped to `K_4`, `igraph_turan(13, 4)` 63-
      edge canonical, `igraph_turan(8, 3)` 21-edge, `igraph_turan(6, 3)`
      octahedron).
    - py lane (0) — no binding.
    - R lane (3) — `make_turan(10, 3)` with sizes `[4, 3, 3]`,
      `make_turan(7, 4)` with sizes `[2, 2, 2, 1]` and 18 edges,
      `make_turan(9, 3)` balanced `K_{3,3,3}` — all dispatching to
      `turan_impl() → igraph_turan()`.
    - Conformance test (`tests/conformance.rs::check_turan_fixture`
      + `turan_two_source_conformance`) uses canonical `(min, max)`
      edge multiset compare plus the exact types vector. Walks both
      `tests/conformance/c/turan/` and `tests/conformance/r/turan/`
      directories.
  - **Bench** (`benches/bench_turan.rs`): 6 shapes covering tiny seed
    graphs (`T(6, 3)` octahedron, `T(13, 4)`), dense balanced
    (`T(96, 3)`, `T(128, 4)`), and skewed remainder (`T(101, 3)`,
    `T(200, 7)`). Throughput climbs from ~12.6 Melem/s on `T(6, 3)`
    (12 edges — overhead-dominated by `Graph::new` + `add_edges` + the
    `Vec<u32>` partition allocation) to ~42 Melem/s on the medium-
    balanced `T(96, 3)` / `T(128, 4)` (~3-6k edges, edge buffer L1-
    resident). Skewed-remainder `T(101, 3)` is on par with balanced
    shapes (~42 Melem/s — the per-partition branch is a single index
    compare, not a hot path). Larger `T(200, 7)` drops to ~35 Melem/s
    as the 17k-edge buffer spills out of L1 but stays L2-resident.
    `py_baseline_ns` is null by construction.
  - **Example** (`examples/turan_demo.rs`): walks the canonical shapes
    `T(0, 10)`, `T(10, 1)`, `T(4, 6)` cap, `T(6, 3)` octahedron,
    `T(13, 4)` partitions `[4, 3, 3, 3]` and verifies the structural
    invariants (always-undirected, monotone types, no intra-partition
    edges, closed-form ecount).
- **ALGO-CN-026** — `full_multipartite` complete multipartite (and
  bipartite) constructor returning `FullMultipartite { graph, types }`
  with the `types: Vec<u32>` partition labels. **Twenty-sixth member of
  the `constructors/` family** and the cross-partition sibling of
  [`full_graph`] / [`full_citation`]. Counterpart of
  `igraph_full_multipartite` in
  `references/igraph/src/constructors/full.c:154-247`.
  - `pub fn full_multipartite(partitions: &[u32], directed: bool, mode:
    MultipartiteMode) -> IgraphResult<FullMultipartite>` — byte-for-byte
    port of the upstream 4-deep emission walk
    `for from_type in 0..k { for i in n_acc[from_type]..n_acc[from_type
    + 1] { for to_type in from_type + 1..k { for j in
    n_acc[to_type]..n_acc[to_type + 1] { emit … } } } }`. The edge
    buffer is pre-sized via `Vec::with_capacity(E)` where `E` is derived
    from the upstream `IGRAPH_SAFE_MULT` formula `(1/2) · Σ_i n_i · (N
    − n_i)` (mode-aware: `All` doubles, `In`/`Out` matches undirected),
    so the bulk `Graph::add_edges` call sees no reallocation.
  - `pub enum MultipartiteMode { All, Out, In }` — local mode enum
    (matching the per-constructor `StarMode`/`WheelMode`/`TreeMode`
    pattern) mapping directly to `IGRAPH_OUT`/`IGRAPH_IN`/`IGRAPH_ALL`.
  - **Empty-middle-partition rule**: the types vector is built with a
    `while v < no_of_types && (i as u64) == n_acc[v] { v += 1; }` loop
    (the `while` — not `if` — is critical: consecutive empty partitions
    like `partitions = [2, 0, 3]` produce two adjacent equal `n_acc`
    entries, and the empty middle partition still occupies a label
    slot, yielding `types = [0, 0, 2, 2, 2]` not `[0, 0, 1, 1, 1]`).
    The 12-arc count for `K_{2,3}` directed-All comes out correctly
    because the empty partition is skipped at the edge level while its
    label slot is preserved.
  - **Overflow protection**: `u64.checked_add` on the cumulative
    `n_acc` (so adversarial `partitions = [u32::MAX, u32::MAX]` reports
    `Overflow` rather than silently wrapping), `u32::try_from` on the
    final `N = n_acc[k]`, `usize::checked_mul` on the edge buffer
    capacity.
  - **Degenerate inputs handled inline**: `partitions = []` returns the
    empty graph with empty types regardless of `directed`/`mode`,
    `partitions = [n]` returns `n` isolated vertices in partition 0 (no
    edges, types `[0; n]`), `partitions` containing zeros simply skips
    those partitions at the edge level while still occupying label
    slots.
  - **Mode dispatch**: undirected collapses to `Out` semantics (the
    directedness flag is encoded only at `Graph::new` time);
    `directed && mode == All` doubles the edge count vs `Out`/`In`;
    `Out` and `In` are reverse-of-each-other when intersected as
    ordered arc sets.
  - **Tests**: 14 unit + 6 proptest covering (a) all 7 upstream
    `.out` cases byte-for-byte (`[2,0,3]` undirected, `[2,0,3]`
    directed-All, `[2,3,4,2]` undirected/All/In/Out, `[]` directed),
    (b) the empty-middle-partition `[2,0,3]` skip yielding
    `types = [0, 0, 2, 2, 2]` and `ecount = 12`, (c) the every-emitted-
    edge-crosses-a-partition-boundary invariant, (d) the
    types-vector-is-monotone partition-major invariant, (e) the
    `directed && mode == All` doubles `Out` ecount invariant, (f) the
    undirected canonical multiset equals directed-Out canonical
    multiset invariant, (g) the `Out` and `In` are reverse-of-each-other
    invariant, plus a proptest sweep validating the invariants across
    random `partitions` shapes (`k ∈ [0, 5]`, per-partition size
    `∈ [0, 6]`, both directednesses, all three modes).
  - **Three-source conformance**: 13 fixtures
    - C lane (7) — the seven `igraph_full_multipartite.out` cases from
      `references/igraph/tests/unit/igraph_full_multipartite.c` (full
      coverage of every mode × shape the upstream test exercises,
      including the empty-graph guard and the empty-middle-partition
      skip; **test 4 has 44 arcs**, not 45 as an early summary claimed
      — verified by direct `.out` read).
    - py lane (3) — `Full_Multipartite([2, 3, 4], False, 'all')` from
      `references/python-igraph/tests/test_constructors.py::testFullMultipartite`,
      `Full_Bipartite(3, 4, False)` from `testBipartite` exercising
      the bipartite shortcut, `Full_Multipartite([2, 3, 4, 2], True,
      'out')` for the directed-out cross-mode check.
    - R lane (3) — `make_full_multipartite(c(2, 3, 4), False, 'all')`,
      `make_full_bipartite_graph(3, 4, False, 'all')`,
      `make_full_multipartite(c(2, 3, 4, 2), True, 'all')` from the
      rigraph `make_full_multipartite` snapshots.
    - Conformance test (`tests/conformance.rs::check_full_multipartite_fixture`)
      uses directed-aware multiset (`(u, v)` distinct from `(v, u)`
      when directed, canonical `(min, max)` when undirected) plus the
      exact types vector since R re-canonicalises endpoints and
      cross-source emission order is not byte-stable.
  - **Bench** (`benches/bench_full_multipartite.rs`): 7 shapes covering
    K_{2,3} bipartite × {undirected ALL, directed OUT}, K_{8,8,8}
    tripartite × {undirected ALL, directed ALL}, K_{32}^4 × {undirected
    ALL, directed OUT}, K_{16}^6 undirected ALL. Throughput grows from
    ~8.5–8.8 Melem/s at K_{2,3} (Graph::new + add_edges overhead
    dominates the 6-edge buffer), climbs to ~33–38 Melem/s at
    K_{8,8,8}, peaks at ~38–42 Melem/s on K_{32}^4 (sweet spot —
    6144-edge buffer comfortably L1-resident), and stays at ~42 Melem/s
    on K_{16}^6 (most balanced six-partite shape spread across
    C(6,2)=15 partition pairs). Directed-All is slightly slower than
    Out/In on the same shape because it emits 2× edges per pair.
  - **Example** (`examples/full_multipartite_demo.rs`): walks the
    degenerate cases (empty partitions, singleton partition `[4]`),
    the empty-middle-partition skip `[2, 0, 3]` showing
    `types = [0, 0, 2, 2, 2]` and 12 mutual arcs, and the richer
    `K_{2,3,4,2}` four-partite shape cross-mode-checked across
    undirected ALL / directed OUT / directed IN / directed ALL with
    five structural invariants asserted (every edge crosses a
    partition boundary, undirected canonical multiset equals
    directed-OUT canonical multiset, ALL ecount = 2 · OUT ecount,
    types is partition-major monotone, types length equals vcount).
  - **Re-exports**: `rust_igraph::{FullMultipartite, MultipartiteMode,
    full_multipartite}` lifted to the crate root.
- **ALGO-CN-025** — `full_citation` complete-graph constructor with the
  citation-order emission. **Twenty-fifth member of the `constructors/`
  family** and the descending-source-major sibling of
  [`full_graph`]. Counterpart of `igraph_full_citation` in
  `references/igraph/src/constructors/full.c:348-376`.
  - `pub fn full_citation(n: u32, directed: bool) -> IgraphResult<Graph>`
    — single byte-for-byte port of the upstream emission walk
    `for i in 1..n { for j in 0..i { push (i, j) } }`, pre-sized via
    `Vec::with_capacity(n·(n-1)/2)` so the bulk `Graph::add_edges` call
    sees no reallocation. `usize::checked_mul` for the `n·(n-1)` product
    mirrors upstream's `IGRAPH_SAFE_MULT(n, n-1, ...)` guard so an
    adversarial `n = u32::MAX` reports `IgraphError::Overflow` rather
    than silently truncating the buffer length.
  - **Citation invariant**: every arc `(u, v)` in the directed variant
    satisfies `u > v` — newer paper cites older — yielding a complete
    DAG with `n·(n-1)/2` arcs and zero self-loops. `in_degree(k) ==
    n - 1 - k`, `out_degree(k) == k` for every `k ∈ [0, n)`.
  - **K_n sibling cross-check**: `directed = false` yields the
    undirected K_n whose edge multiset is identical to
    `full_graph(n, false, false)` but in descending-source-major
    emission order (vs `full_graph`'s ascending-source-major) — the
    n=5 example demonstrates the multiset equality + emission-order
    inequality directly.
  - **Degenerate inputs handled inline**: `n == 0` returns the empty
    graph regardless of flag, `n == 1` returns a singleton with no
    edges.
  - **Tests**: 12 unit + 6 proptest covering (a) `n=0/1` degenerate
    cells for both directedness settings, (b) `n=4` directed canonical
    DAG byte-match against the upstream `.out` fixture, (c) `n=6`
    in/out-degree profile via local `in_out_counts` helper, (d) the
    K_5 multiset cross-check against `full_graph(5, false, false)`,
    (e) emission-order inequality, (f) closed-form ecount sweep across
    `n ∈ {0, 1, 2, 7, 25}`, (g) directed-undirected ecount equality
    invariant.
  - **Three-source conformance**: 10 fixtures
    - C lane (4) — `n=4` directed canonical DAG matching
      `references/igraph/tests/unit/igraph_full_citation.c`, `n=4`
      undirected K_4, `n=1` directed singleton, `n=0` directed empty.
    - py lane (3) — `n=6` directed/undirected from
      `references/python-igraph/tests/test_constructors.py::testFullCitation`,
      `n=2` directed degenerate single arc `1 → 0`.
    - R lane (3) — `n=4` directed/undirected from
      `references/rigraph/R/make_full_citation_graph`, `n=1` directed
      singleton.
    - Conformance test uses canonical `(min, max)` multiset for
      undirected and the raw `(u, v)` multiset for directed
      (mirroring `check_linegraph_fixture`'s precedent) since R
      re-canonicalises endpoints and emission order is not byte-stable
      across the three sources.
  - **Bench** (`benches/bench_full_citation.rs`): 6 shapes covering
    n=8 / 64 / 512 × directed / undirected. Throughput grows from
    ~20 Melem/s at n=8 (Graph::new + add_edges overhead dominates) to
    ~44 Melem/s at n=64 (sweet spot where the inner emit loop is
    L1-resident yet the per-call setup amortizes) and tapers to
    ~31–32 Melem/s at n=512 (130 816 edges; cost dominated by
    `Graph::add_edges` populating from/to/oi/ii arrays and sort-by-key
    into the CSR-like adjacency). Directed vs undirected are
    indistinguishable since the emission body is identical.
  - **Example** (`examples/full_citation_demo.rs`): walks the
    degenerate cases (n=0, n=1), the canonical n=4 directed DAG
    byte-match, the n=6 in/out-degree profile, the K_5 multiset
    cross-check vs `full_graph(5, false, false)`, and the closed-form
    ecount sweep — demonstrating that `full_citation` and `full_graph`
    produce the same undirected K_5 multiset but in different emission
    orders.

- **ALGO-CN-024** — `hexagonal_lattice` planar hexagonal (honeycomb)
  lattice constructor. **Twenty-fourth member of the `constructors/`
  family** and the planar dual of [`triangular_lattice`]. Counterpart of
  `igraph_hexagonal_lattice` in
  `references/igraph/src/constructors/lattices.c:350-601`.
  - `pub fn hexagonal_lattice(dims: &[u32], directed: bool, mutual: bool) -> IgraphResult<Graph>`
    — single `dims`-length dispatch (1=triangle outline, 2=quasi-rectangle,
    3=hexagon) with a shared emitter that emits up-to-two local neighbour
    candidates per lattice site: right `(k + 1, j)` always, and up-left
    `(k - 1, j + 1)` only when `k` is odd. Vertex indexing follows
    upstream's `lex_ordering = false` convention via per-row prefix sums.
  - **Three shape branches** with a single emitter:
    - `dims=[n]` → triangular outline of `n` hexagons per side. The
      shape helper uses upstream's exact `row_count_raw = size + 2; for i
      in 0..(row_count_raw - 1)` loop with `len = 2 · (row_count_raw - i)
      − (3 if i == 0 else 1)` — subtle off-by-one trap (the loop runs
      over `size + 1` rows, not `size`, and the first row sheds 3 instead
      of 1). `dims=[5]` gives the canonical 46-vertex / 60-arc .out
      fixture.
    - `dims=[size_x, size_y]` → quasi-rectangle of `size_x × size_y`
      hexagons. `dims=[2, 2]` matches the python-igraph
      `testHexagonalLattice` 16-vertex / 19-edge fixture.
    - `dims=[size_x, size_y, size_z]` → hexagonal outline with three
      sides carrying `size_x, size_y, size_z` hexagons respectively.
      `dims=[3, 4, 5]` gives the canonical 94-vertex / 129-edge .out
      fixture. Zero in any dim collapses to the empty graph (upstream
      `igraph_vector_int_any_smaller(dims, 1)` guard).
  - **Degree ≤ 3 invariant**: every vertex carries at most three
    neighbours (vs the planar dual's ≤ 6) — the honeycomb tiling
    structural property that distinguishes hexagonal from triangular
    lattices.
  - **Directed-mutual matrix**: `directed=false` ignores `mutual`;
    `directed=true, mutual=false` emits one arc per edge; `directed=true,
    mutual=true` doubles emission so each undirected edge becomes a
    forward+reverse pair. Comparison in the conformance lane uses
    directed-aware multiset equality.
  - **Tests**: 12 unit + 6 proptest covering (a) every dim arity branch,
    (b) the directed/mutual cross-product, (c) `dims=[1]` single-hexagon
    C_6, (d) zero-dim guard, (e) max-degree-≤-3 structural invariant,
    (f) the upstream `.out`-derived `dims=[5]` triangle side 5 fixture
    (46 vertices, 60 arcs), (g) the canonical `dims=[3, 4, 5]` hexagon
    fixture (94 vertices, 129 edges).
  - **Two-source conformance**: 10 fixtures
    (`tests/conformance/{c,py}/hexagonal_lattice/`). **R lane skipped**:
    R-igraph 2.x exposes no `hexagonal_lattice` wrapper (verified via
    `references/rigraph/R/`), so the integration test compares C and py
    only. C lane (7): single hexagon directed, triangle side 5 directed
    full 60-arc reproduction, rectangle 4×5 directed-mutual 154-arc,
    hexagon (3,4,5) undirected-mutual 129-edge match, two zero-dim
    guards, rectangle 2×2 undirected. py lane (3): `Graph.Hexagonal_Lattice([2, 2], ...)`
    in all three mode combinations.
  - **Bench** (`benches/bench_hexagonal_lattice.rs`, baseline at
    `.codefuse/tracking/perf/ALGO-CN-024.json`): five shapes across
    both flag groups. `triangle_50` ≈ 133 µs (~30 Medge/s),
    `triangle_100` ≈ 575 µs (~27 Medge/s), `rect_100x100` ≈ 1.19 ms,
    `rect_200x100` ≈ 2.52 ms, `hex_30_30_30` ≈ 303 µs. Directed-mutual
    variants land in the 17–22 Medge/s band. Throughput edges out
    `triangular_lattice` on similar `|V|+|E|` budgets because hexagonal
    vertices carry degree ≤ 3 (vs triangular's ≤ 6), so per-vertex
    candidate evaluations are fewer.
  - Example: `cargo run --example hexagonal_lattice_demo` walks a
    7-case truth table (single hexagon, triangle side 5, rect 2×2
    plain/mutual, rect 4×5 mutual, hexagon (3,4,5), zero-dim guard)
    asserting vcount, ecount, and the degree-≤-3 invariant.
  - **Translation bug caught + fixed during Step 5**: my first draft of
    `triangle_shape` used `row_count = size + 1` (reading the upstream
    loop bound as `row_count` rather than `row_count_raw - 1`), which
    produced 20 vertices / 12 arcs for `dims=[5]` instead of the
    canonical 46 / 60. The unit-test sweep against the upstream `.out`
    file (`references/igraph/tests/unit/igraph_hexagonal_lattice.out`)
    surfaced the mismatch immediately; fixed by transcribing upstream's
    `row_count_raw = size + 2; loop 0..row_count_raw - 1` verbatim.
- **ALGO-CN-023** — `triangular_lattice` planar triangular lattice
  constructor. **Twenty-third member of the `constructors/` family**
  and the planar dual of the hexagonal lattice. Counterpart of
  `igraph_triangular_lattice` in
  `references/igraph/src/constructors/lattices.c:431-569`.
  - `pub fn triangular_lattice(dims: &[u32], directed: bool, mutual: bool) -> IgraphResult<Graph>`
    — single `dims`-length dispatch (1=triangle, 2=quasi-rectangle,
    3=hexagonal) with a shared `add_if_in_bounds` closure that emits
    up-to-three local neighbour candidates (right, up-right, up-left)
    per lattice site. Vertex indexing follows upstream's
    `lex_ordering = false` convention via per-row prefix sums.
  - **Three shape branches** with a single emitter:
    - `dims=[k]` → triangle side `k`: `k*(k+1)/2` vertices,
      `3*k*(k-1)/2` undirected edges. Each row `j` has `k - j`
      vertices and starts at id `prefix_sum[j]`.
    - `dims=[m, n]` → `m × n` quasi-rectangle: `m*n` vertices and a
      triangular tessellation of the rectangle.
    - `dims=[a, b, c]` → hexagonal lattice with sides `a, b, c`:
      a `2·max(a,b,c) − 1`-row jagged tile. Zero in any dim collapses
      to the empty graph (upstream `IGRAPH_ERROR` guard mapped to a
      successful empty return).
  - **Directed-mutual matrix**: `directed=false` ignores `mutual`;
    `directed=true, mutual=false` emits one arc per edge; `directed=true,
    mutual=true` doubles emission so each undirected edge becomes a
    forward+reverse pair. Comparison in the conformance lane uses
    directed-aware multiset equality so the mutual flag legitimately
    distinguishes `(a, b)` from `(b, a)`.
  - **Tests**: 12 unit + 6 proptest covering (a) every dim arity branch,
    (b) the directed/mutual cross-product, (c) `dims=[1]` single-vertex
    edge case, (d) zero-dim guard, (e) max-degree-≤-6 structural
    invariant for interior vertices, (f) the upstream `.out`-derived
    triangle side 5 fixture (15 vertices, 30 arcs).
  - **Three-source conformance**: 13 fixtures (`tests/conformance/{c,py,r}/triangular_lattice/`).
    C lane (7): triangle side 1/3/5, 2×2 rectangle undirected/directed/mutual,
    hex (3,4,5). py lane (3): `testTriangularLattice` directly from
    `python-igraph/tests/test_constructors.py`. R lane (3): two from
    `rigraph/tests/testthat/_snaps/aaa-auto.md` (rectangle undirected +
    directed-mutual) plus one synthetic triangle-side-3 fixture cross
    -checked against C upstream.
  - **Bench** (`benches/bench_triangular_lattice.rs`, baseline at
    `.codefuse/tracking/perf/ALGO-CN-023.json`): five shapes across
    both flag groups. `triangle_100` ≈ 693 µs (~21 Medge/s),
    `triangle_200` ≈ 2.85 ms, `rect_100x100` ≈ 1.43 ms,
    `rect_200x100` ≈ 3.03 ms, `hex_30_30_30` ≈ 330 µs (~23 Medge/s).
    Directed-mutual variants double both edge count and wall time at
    ~19 Medge/s. All shapes land in the 18-23 Medge/s band; the ~30%
    gap vs `square_lattice` is the extra up-left candidate plus the
    per-edge i64→u32 conversion that keeps mode dispatch overflow-safe.
  - Example: `cargo run --example triangular_lattice_demo` walks the
    7-case truth table (triangle 1/3/5, rect 2×2 plain/mutual, hex
    (3,4,5), zero-dim guard) with assertions on vcount, ecount, and
    max degree ≤ 6.
- **ALGO-CN-022** — `create` foundational edge-list constructor.
  **Twenty-second member of the `constructors/` family** and the
  universal hand-rolled entry point: every other generator-like API in
  the project ultimately reduces to it. Counterpart of `igraph_create`
  in `references/igraph/src/constructors/create.c:47-95`.
  - `pub fn create(edges: &[(u32, u32)], n: u32, directed: bool) -> IgraphResult<Graph>`
    — single max-scan over the input slice + `Graph::new(vcount, directed)`
    + bulk `Graph::add_edges`. No intermediate buffers, no dedup, no
    canonicalisation: self-loops `(u, u)` and parallel edges survive
    verbatim, and emission order is preserved byte-for-byte.
  - **Type-level error elimination.** The upstream C entry takes
    `igraph_vector_int_t` of length `2·|E|` and can return
    `IGRAPH_EINVAL` (odd-length edge buffer) and `IGRAPH_EINVVID`
    (negative vertex ids). Both become statically unreachable once
    the input is `&[(u32, u32)]`: odd-length cannot exist (pairs are
    tuples) and negative ids cannot exist (`u32`).
  - **Semantics** (matches upstream exactly):
    - `n == 0` infers vcount from `max(u, v) + 1` over the edge list,
      returning the empty graph when `edges` is empty.
    - `n < max_endpoint + 1` silently extends to `max_endpoint + 1`
      via the upstream `igraph_add_vertices(graph, delta)` path.
    - `n ≥ max_endpoint + 1` keeps the requested vertex count verbatim;
      any unreferenced vertex stays isolated.
  - **Runtime overflow protection**: `max_endpoint.checked_add(1)`
    returns `IgraphError::Overflow` rather than wrapping when an
    endpoint is `u32::MAX` (the demo shows
    `create(&[(0, u32::MAX)], 0, false)` triggering this path). A
    saturating `n.max(max_endpoint + 1)` widens to `u32` before
    `Graph::new` so the "silent extend" rule cannot truncate.
  - **Tests**: 12 unit + 4 proptest covering (a) the upstream-C example
    from `references/igraph/examples/simple/igraph_create.c`
    (`[(0,1), (1,2), (2,3), (2,2)]`, `n=0` → vcount 4 / ecount 4 /
    1 self-loop on vertex 2), (b) `n == 0` empty-edges null graph,
    (c) `n == 0` self-loop inference, (d) `n > max+1` keeps padding,
    (e) `n < max+1` silent extend, (f) directed arc-order preserved,
    (g) self-loops and parallels survive, (h) `u32::MAX` endpoint
    triggers `Overflow`. Proptest sweep checks edge multiset equality,
    vcount projection, ecount equals input length, and edge-id order
    preservation across random `n ∈ [0, 50]`, edge counts `∈ [0, 100]`,
    and both directed flags.
  - **Three-source conformance fixtures: 19 total** under
    `tests/conformance/{c,py,r}/create/`. C:7 (upstream example,
    n=10 padded, n=3 silent extend, directed arc-order, empty null,
    empty 5-isolated, self-loops+parallels), py:6 (simple n=0,
    explicit n keeps, directed arcs, empty null, star via edges,
    self-loop), R:6 (make_graph 0-based variants: n=0 infers,
    explicit n kept, two arcs directed, empty isolated, parallel
    edges, P_4 path). Conformance test decodes
    `params.edges/n/directed` and compares vcount + ecount + directed
    flag + canonical edge multiset (directed-aware key:
    `if directed || u <= w { (u,w) } else { (w,u) }`).
  - **Demo**: `examples/create_demo.rs` walks seven invariants —
    upstream n=0 infer, n=10 padded, n=3 extends to max+1=7, directed
    cycle pair, multigraph self-loops/parallels, K_{1,7} star via
    `create`, u32::MAX `Overflow` error path.
  - **Bench**: `benches/bench_create.rs` covers three workloads
    scanning the per-edge cost surface: path (single-degree vertices),
    star (one hub holds all incidences), dense K_n (quadratic |E|).
    Rust per-edge cost is ~15 ns at the path/star scale (n=100k:
    1.53 ms ≈ 15.3 ns/edge); 31.0× faster than python-igraph
    `Graph(edges, n, directed)` at path n=100 (90 µs Cython entry
    overhead amortises away at scale: 8.4× / 3.2× / 2.6× across n ∈
    {1k, 10k, 100k}). Dense narrows to 4.8×→1.4× as the Cython
    per-call cost becomes negligible vs the already-optimal upstream
    C edge-array packing. Snapshot in
    `.codefuse/tracking/perf/ALGO-CN-022.json`.

- **ALGO-CN-021** — `atlas` + `ATLAS_SIZE` Read-Wilson graph atlas
  constructor. **Twenty-first member of the `constructors/` family.**
  Counterpart of `igraph_atlas` in
  `references/igraph/src/constructors/atlas.c:50-87`. Given an index
  `number ∈ [0, 1253)` returns the `number`-th of the 1253 simple
  unlabelled undirected graphs on 0..7 vertices in the ordering of
  Ronald C. Read and Robin J. Wilson, *An Atlas of Graphs* (Oxford
  University Press, 1998).
  - `pub fn atlas(number: u32) -> IgraphResult<Graph>` — single
    position-table indirection + bulk `add_edges`. Out-of-range indices
    return `InvalidArgument`.
  - `pub const ATLAS_SIZE: u32 = 1253` — total catalogue size.
  - Ordering convention (canonical, matches upstream byte-for-byte):
    (1) by ascending vertex count, (2) for fixed vertex count by
    ascending edge count, (3) for fixed vertex/edge count by ascending
    lexicographic degree sequence, (4) for fixed degree sequence by
    ascending automorphism count.
  - Per-vertex-count starting offsets: `STARTS = [0, 1, 2, 4, 8, 19,
    53, 209, 1253]`. The first index in each cell is the null graph
    on `n` vertices; the last index in each non-trivial cell is the
    complete graph `K_n`: `atlas(3) = K_2`, `atlas(7) = K_3`,
    `atlas(18) = K_4`, `atlas(52) = K_5`, `atlas(208) = K_6`,
    `atlas(1252) = K_7`.
  - Storage: two `#[rustfmt::skip] pub(super) const &[u32]` arrays in
    `src/algorithms/constructors/atlas_edges.rs` — `ATLAS_EDGES_POS`
    (length 1253, starting offsets into the flat data block) and
    `ATLAS_EDGES` (length 27146, flat header+edge data with layout
    `[vcount, ecount, edge0_a, edge0_b, …]` per graph). Both
    transliterated byte-for-byte from
    `references/igraph/src/constructors/atlas-edges.h` so the
    per-index dispatch reproduces the C, python-igraph
    (`Graph.Atlas(number)`), and R-igraph (`graph_from_atlas(n)` →
    `igraph_atlas`) emission identically.
  - Overflow protection: `usize::checked_mul(ecount, 2)` guards the
    edge buffer length (per-graph upper bound is `2·21 = 42` so
    overflow is impossible in practice but the check stays for
    adversarial position-table corruption), defensive bounds check
    on `pos + 2 ≤ ATLAS_EDGES.len()` and
    `pos + 2 + 2·ecount ≤ ATLAS_EDGES.len()` returns
    `InvalidArgument` rather than panicking on a corrupt table.
  - 8 unit tests cover (a) out-of-range error path (number = 1253,
    1254, `u32::MAX`), (b) `ATLAS_SIZE` matches upstream count,
    (c) null-graph cell-start indices for all 8 vertex counts,
    (d) small-known-graph identities for `atlas(3) = K_2`,
    `atlas(7) = K_3`, `atlas(18) = K_4` with canonical edge multiset
    compare, (e) `atlas(1252) = K_7` with 6-regularity check,
    (f) no self-loops or repeated edges across a 19-index
    representative spread, (g) exhaustive 1253-graph walk for
    degree-sum invariant + undirected flag, (h) every index in
    `start[n]..start[n+1]` has exactly `n` vertices. The exhaustive
    sweep is strictly stronger than any sample-based proptest, so
    no proptest was added.
  - 19 conformance fixtures across the three sources: C:8 (null0,
    k2_single_edge, triangle, k4, v6_null, k6_last_6v, v7_null,
    k7_last_atlas_graph), py:7 (null0, k2, k4, idx70_skipped_in_upstream,
    idx180_skipped_in_upstream, k6, k7_last — python-igraph cells skip
    a few interior indices the C test catalogue omits), R:4 (null0,
    triangle, k4, k7_last). Cross-source emission order is byte-stable
    because all three bindings read the same on-disk `atlas-edges.h`
    table, but R-igraph re-canonicalises 1-based→0-based, so the
    conformance test compares canonical `(min, max)` edge multisets.
  - Runnable example: `cargo run --example atlas_demo` walks the
    cell-boundary invariants, verifies `K_2..K_7` complete-graph
    landings at the published indices, lists the first six graphs by
    index, walks the entire 1253-graph catalogue verifying undirected /
    well-formed / no self-loops / degree-sum invariants, and exercises
    the out-of-range error path.
  - Criterion bench at `benches/bench_atlas.rs`; baseline in
    `.codefuse/tracking/perf/ALGO-CN-021.json`. Per-call cost: 226 ns
    (null graph at idx 0) → 1141 ns (K_7 at idx 1252) — dominated by
    `add_edges` since position-table indirection is two array indexes.
    Full 1253-graph catalogue walk costs 1.06 ms. python-igraph's
    `Graph.Atlas(number)` baseline sits at ~85 μs/call regardless of
    graph size (fixed Cython entry + Graph object construction
    overhead), so Rust is 87.7×–375.8× faster (375.8× on the null
    graph, 101.4× on the full 1253-graph walk).

- **ALGO-CN-020** — `famous` + `famous_names` named-graph catalogue.
  **Twentieth member of the `constructors/` family.** Counterpart of
  `igraph_famous` in `references/igraph/src/constructors/famous.c`. Given
  a case-insensitive ASCII name returns the canonical labelled copy of
  one of 31 small graphs that recur throughout graph theory.
  - `pub fn famous(name: &str) -> IgraphResult<Graph>` — case-insensitive
    name dispatch over the static catalogue; returns `InvalidArgument`
    with a helpful message for unknown names.
  - `pub fn famous_names() -> &'static [&'static str]` — returns the
    31-entry canonical-name slice for catalogue walks.
  - Catalogue (alphabetical): Bull, Chvátal, Coxeter, Cubical, Diamond,
    Dodecahedron, Folkman, Franklin, Frucht, Grötzsch, Heawood, Herschel,
    House, HouseX, Icosahedron, Krackhardt Kite, Levi, McGee, Meredith,
    Noperfectmatching, Nonline, Octahedron, Petersen, Robertson,
    Smallestcyclicgroup, Tetrahedron, Thomassen, Tutte, Uniquely3colorable,
    Walther, Zachary.
  - Aliases follow upstream: `dodecahedral ≡ dodecahedron`, `grotzsch ≡
    groetzsch` (German/English spelling), `icosahedral ≡ icosahedron`,
    `octahedral ≡ octahedron`, `tetrahedral ≡ tetrahedron`.
  - Storage: each graph is a `#[rustfmt::skip] const DATA_<NAME>: &'static
    [u32]` array transliterated byte-for-byte from
    `references/igraph/src/constructors/famous.c:26-249`. Layout is
    `[vcount, ecount, directed_flag, edge0_a, edge0_b, …]`. Dispatch is
    a linear scan over a 36-entry static `(name, data)` table using
    `eq_ignore_ascii_case` so the whole table fits in L1 and a lookup
    stays sub-microsecond.
  - Size envelope: smallest is Diamond (4 v / 5 e), largest is Meredith
    (70 v / 140 e). All 31 graphs are simple undirected.
  - 10 unit tests cover every catalogue entry's published `(vcount,
    ecount)`, case-insensitive dispatch, every alias collapsing to its
    canonical name, the unknown-name error path, Petersen's 3-regularity,
    Zachary's published (34 v / 78 e) shape, and the simple-undirected
    invariant across the full catalogue.
  - 16 conformance fixtures across the three sources: C:7 (Bull, Petersen,
    Meredith counts, Zachary counts, dodecahedron case-insensitive,
    grotzsch alias, tetrahedron alias), py:5 (Bull, Petersen, Meredith
    counts, Zachary counts, krackhardt kite lowercase), R:4 (Bull,
    Petersen, Zachary counts, tetrahedral alias). All three bindings
    dispatch to the same C entry point (python-igraph `Graph.Famous(name)`,
    R-igraph `make_graph("<name>")` → `famous_impl()` → `igraph_famous`),
    so cross-source edge multisets match — conformance test compares
    canonical `(min, max)` edge multisets.
  - Runnable example: `cargo run --example famous_demo` walks every
    `famous_names()` entry, validates simple-undirected, asserts
    canonical counts for Petersen / Zachary / Tutte / Meredith, and
    exercises case-insensitive dispatch + the German/English Grötzsch
    alias + the unknown-name error path.
  - Criterion bench at `benches/bench_famous.rs`; baseline in
    `.codefuse/tracking/perf/ALGO-CN-020.json`. Per-call cost: Bull
    619 ns, Petersen 982 ns, Meredith 5.4 μs; walking the full
    31-graph catalogue costs 50.6 μs. The python-igraph baseline
    (`Graph.Famous(name)`) sits at ~85 μs/call because the per-call
    cost is dominated by the Cython entry path and Graph object
    construction; Rust avoids both layers and runs 15.7×–207× faster
    (52.4× speedup on the full 31-graph walk).

- **ALGO-CN-019** — `mycielskian` + `mycielski_graph` Mycielski-construction
  pair. **Nineteenth member of the `constructors/` family.** Counterpart
  of `igraph_mycielskian` + `igraph_mycielski_graph` in
  `references/igraph/src/constructors/mycielskian.c`. Given a graph
  `G = (V, E)` and a non-negative iteration count `k`, the Mycielski
  construction `μ(G)` produces a triangle-free larger graph whose
  chromatic number is one higher than `G`'s. Per-iteration recurrence
  `(v', e') = (2v + 1, 3e + v)` is exact, and the construction preserves
  directedness.
  - `pub fn mycielskian(graph: &Graph, k: u32) -> IgraphResult<Graph>` —
    apply the construction `k` times to an arbitrary input graph.
  - `pub fn mycielski_graph(k: u32) -> IgraphResult<Graph>` — produce the
    canonical Mycielski chain element `M_k`, with `M_2 = P_2`, `M_3 = C_5`,
    `M_4 = Grötzsch` (11 vertices, 20 edges, triangle-free, χ = 4),
    `M_5 = (23 vertices, 71 edges)`, etc. `k ∈ {0, 1}` returns null /
    singleton in-band per project convention.
  - Single-iteration emission: for input with `n` vertices and edge list
    `[(u_0, v_0), …, (u_{m-1}, v_{m-1})]`, output has `2n + 1` vertices
    labelled `0..n` (originals) ∪ `n..2n` (shadow copies) ∪ `{2n}` (apex),
    and edges (1) originals reproduced verbatim, (2) `(u_i, v_i + n)` and
    `(u_i + n, v_i)` for each original edge (`2m` cross edges), (3)
    `(j + n, 2n)` for `j ∈ [0, n)` (`n` apex edges).
  - Per-iteration cost `O(e + v)` — `stage_input_edges` copies the current
    row once, `project_counts` pre-flight grows the output buffer via
    `Vec::resize` so all subsequent work is in-place pointer arithmetic,
    and the final `Graph::add_edges` call is a single bulk insertion.
  - Promotion chain handles degenerate inputs in-band:
    `null → singleton (k=1) → P_2 (k=2)`.
  - Overflow protection: `u32::checked_mul(2)` + `checked_add(1)` for
    `2v + 1`, `usize::checked_mul(3)` + `checked_add(v)` for `3e + v`,
    `try_from` on every cross-domain boundary so adversarial sizes report
    `Overflow` rather than silently wrapping.
  - 18 unit tests + 6 proptest invariants covering M_2..M_5 chain
    counts, Grötzsch construction from `C_5`, directed preservation,
    null/singleton/P_2 promotion chain, `k=0` identity, recurrence
    on arbitrary inputs, shadow-degree and apex-degree invariants,
    overflow paths.
  - 17 conformance fixtures: C:10 (`k0_null`, `k1_singleton`, `k2_p2`,
    `k3_c5`, `k4_grotzsch`, `k5_recurrence` for `mycielski_graph`;
    `p3_one_iteration`, `c5_one_iteration`, `null_two_iterations`,
    `k0_identity` for `mycielskian`), py:4 (`k3_c5`, `k4_grotzsch_counts`
    for `mycielski_graph`; `p3_one_iteration`, `singleton_one_iteration_is_p2`
    for `mycielskian` — flagged "no upstream binding in python-igraph 0.11.x"),
    R:3 (`k3_c5` for `mycielski_graph`; `p3_one_iteration`,
    `p3_two_iterations` for `mycielskian` from rigraph snapshot).
  - igraph C has no dedicated unit test for mycielskian; C-lane fixtures
    use synthetic-but-canonical recurrence-derived shapes (same pattern
    as ALGO-CN-018 lcf for the bug #996 regression).
  - Runnable example: `cargo run --example mycielskian_demo` walks
    `M_2..M_5`, applies the construction to `C_5` to recover Grötzsch,
    demonstrates directed preservation, and confirms the
    `null → singleton → P_2` promotion chain.
  - Criterion bench at `benches/bench_mycielskian.rs`; baseline in
    `.codefuse/tracking/perf/ALGO-CN-019.json`. `mycielski_graph(10)`
    builds a 513-vertex/4099-edge graph in ~1.05 ms; `mycielskian(C_n, 2)`
    scales linearly in n at ~0.51 μs/vertex (n=32: 15 μs, n=64: 31 μs,
    n=128: 65 μs). No external baseline — python-igraph 0.11.x exposes
    neither binding.

- **ALGO-CN-018** — `lcf` LCF (Lederberg-Coxeter-Frucht) cubic-graph
  constructor. **Eighteenth member of the `constructors/` family.**
  Counterpart of `igraph_lcf` in `references/igraph/src/constructors/lcf.c`.
  Given vertex count `n`, an integer shift pattern `shifts[0..k]`
  (positive or negative offsets relative to a Hamilton cycle), and a
  `repeats` count, materialises the unique 3-regular simple undirected
  graph encoded by the LCF notation `[shifts]^repeats`.
  - `pub fn lcf(n: u32, shifts: &[i64], repeats: u32) -> IgraphResult<Graph>`.
    Algorithm: (1) emit Hamilton-cycle backbone `0–1–2–…–(n−1)–0`
    (skipped when `n ≤ 1`), (2) chord pass — for each
    `sptr ∈ [0, repeats·shifts.len())` compute
    `u = sptr mod n`, `s = shifts[sptr mod shifts.len()]`,
    `v = ((u as i64) + s).rem_euclid(n as i64)`, emit candidate chord
    `(u, v)`, (3) inline simplify via `BTreeSet<(u32, u32)>` with
    canonical `(min, max)` insert — self-loops where `u == v` are
    silently dropped and any chord already in the seen-set is a no-op,
    mirroring upstream's terminal `igraph_simplify(loops=true, multi=true)`
    collapse. Result is always `Graph::new(n, false)` (the C constructor
    is fixed `IGRAPH_UNDIRECTED`). Overflow protection: `i64` widening
    for the chord arithmetic so `shifts = [i64::MAX]` cannot silently
    wrap before the `rem_euclid` modular reduction.
  - 15 unit tests covering: Franklin `[5,-5]^6` n=12 (18 edges, bipartite
    cubic), Heawood `[5,-5]^7` n=14 (21 edges, girth-6 cage), Frucht
    `[-5,-2,-4,2,5,-2,2,5,-2,-5,4,2]^1` n=12 (18 edges, only 3-regular
    planar graph with trivial automorphism group), Truncated tetrahedron
    `[2,6,-2,-6]^3` n=12, Truncated octahedron `[3,-7,7,-3]^6` n=24 (36
    edges), pure-Hamilton-cycle when `shifts` is empty or `repeats == 0`,
    n=2 self-loop collapse (every chord is a self-loop; only backbone
    survives, ecount=1), `lcf(0, [], 0)` → empty graph (upstream bug #996
    regression), negative-shift wrap via `rem_euclid`, large-shift wrap
    `|s| ≥ n`, duplicate-chord collapse, mixed-shift smoke. All assert
    cubic-regularity, simple-undirected, and exact edge multiset where
    possible.
  - 7 proptest invariants (`proptest-harness` feature) over
    `n ∈ [0, 30]`, `shifts.len() ∈ [0, 6]`, `repeats ∈ [0, 4]`: result
    always undirected, no self-loops survive, no parallel edges survive
    (canonical multiset uniqueness); edge count ≤ `n + repeats·shifts.len()`
    (chord-pass upper bound before simplify); pure-cycle invariant when
    `repeats == 0` or `shifts` is empty; `n == 0` always yields empty
    regardless of pattern; `n == 1` always yields singleton; unrolled-shifts
    equivalence (`lcf(n, shifts, k) ≡ lcf(n, shifts.repeat(k), 1)` as
    edge multisets); maximum degree bounded by `2 + 2·shifts.len()` per
    vertex.
  - 14 three-source conformance fixtures (`C:6 / py:5 / R:3`) under
    `tests/conformance/{c,py,r}/lcf/`: C — Franklin, three_minus2_repeats_4_n8,
    two_minus2_repeats_2_n2 (self-loop collapse), two_repeats_2_n2,
    null_graph_bug_996, Heawood; py — Franklin, Heawood, Truncated
    tetrahedron, empty-shifts pure-cycle, single-shift repeats=3;
    R — Franklin, Heawood, repeats_zero_pure_cycle. Conformance test
    compares canonical `(min, max)` edge multisets (cross-source
    emission order is not byte-stable; upstream's terminal simplify
    rewrites the edge list).
  - `examples/lcf_demo.rs` — walks Franklin, Heawood, Frucht, Truncated
    tetrahedron, Truncated octahedron, pure-cycle, n=2 self-loop
    collapse, bug #996 null-graph; asserts cubic-regularity and
    simple-undirectedness for each.
  - `benches/bench_lcf.rs` (criterion) — `famous` group over the five
    catalogue graphs, `cycle_only` sweep `n ∈ {64, 1024, 16384, 131072}`,
    `chord_heavy` sweep same `n` with `shifts = [3,-5,7,-11,13]` and
    `repeats = n/5`. Snapshot at `.codefuse/tracking/perf/ALGO-CN-018.json`:
    famous shapes ~7.8–9.4 Melem/s, cycle-only `n=131072` 12.578 ms
    (10.4 Melem/s), chord-heavy `n=131072` 27.531 ms (4.8 Melem/s);
    cycle-only scaling 17.9×/20.7×/9.0× and chord-heavy 23.7×/20.0×/9.0×
    per 16×/16×/8× `n` step confirm near-linear with mild log overhead
    from `BTreeSet` inserts (matches upstream's `igraph_simplify`
    envelope).

- **ALGO-CN-017** — `tree_from_parent_vector` parent-vector tree/forest
  decoder. **Seventeenth member of the `constructors/` family.**
  Counterpart of `igraph_tree_from_parent_vector` in
  `references/igraph/src/constructors/tree.c`. Decodes a length-`n`
  parent vector — where `parents[v]` names `v`'s parent vertex id and
  any negative entry marks `v` as a root — into the unique rooted tree
  or forest on `n` vertices, with three possible arc orientations
  selected by `TreeMode` (reused from `kary_tree`): `Out` (directed
  parent→child), `In` (directed child→parent), `Undirected` (single
  canonical `(min, max)` edge per parent link).
  - `pub fn tree_from_parent_vector(parents: &[i64], mode: TreeMode) ->
    IgraphResult<Graph>`. Linear-time `O(n)`: each vertex is visited at
    most twice across all rounds — once as the outer index `i`, once on
    the way up its parent chain — with a `seen[]` array short-circuiting
    revisits. Emission order matches upstream byte-for-byte. Validation
    rejects any parent id `>= n` with `InvalidArgument`; any negative
    value is a valid root sentinel (matches C semantics where igraph's
    internal `-2` and the user's `-1` are both negative). Uses
    `u32::try_from(...).map_err(...)` rather than `as u32` to stay free
    of unwrap/expect per project conventions.
  - 13 unit tests covering: empty parent vector → empty graph,
    all-roots → isolated-vertex edgeless graph in all three modes,
    single-root chain in all three modes, two-root forest, the upstream
    `[4, 4, 1, -2, 3]` fixture in OUT/IN/Undirected with exact
    edge-order assertions, star centred on vertex 0, the rigraph
    snapshot chain (C-level `[-2, 0, 1, 2]` after the R wrapper's `-1`
    shift), out-of-range parent error path.
  - 7 proptest invariants over `n ∈ [1, 30)`: result is acyclic
    (union-find probe), edge count equals number of non-root entries,
    zero self-loops, undirected mode keeps the multiset deduplicated,
    OUT/IN modes produce mirror-image arc sets (transposing flips them),
    Undirected mode equals the canonical multiset of either directed
    mode, and an out-of-range `parents[v] >= n` strategy always errors.
  - Three-source conformance: 6 igraph C fixtures (upstream `[4,4,1,-2,3]`
    in OUT/IN/Undirected, two-root forest, edgeless all-roots, null
    graph), 5 synthetic python-igraph fixtures (python-igraph does NOT
    bind `igraph_tree_from_parent_vector`; fixtures use identical C
    semantics — chain undirected, star OUT/IN, two-root forest,
    singleton root), 3 rigraph snapshot fixtures (chain in all three
    modes using the C-level shifted parents `[-2, 0, 1, 2]`). Directed
    fixtures compare the ordered edge multiset (upstream emission order
    is the contract); undirected fixtures compare the canonical
    `(min, max)` multiset (rigraph re-canonicalises edges).
  - `examples/tree_from_parent_vector_demo.rs` walks every interesting
    shape — chain in all three modes, two-root forest, upstream-C
    fixture in OUT/IN/Undirected, star centred on vertex 0, edgeless
    graph (all roots), rigraph snapshot — with forest-invariant
    assertions on each (no self-loops, no cycles, no duplicate edges).
  - `benches/bench_tree_from_parent_vector.rs` sweeps four topologies
    (chain / star / forest / random) over `n ∈ {64, 1 024, 16 384,
    131 072}`. Best throughput ~53 Melem/s on `forest` (short
    alternating chains stay L1-resident); worst ~18 Melem/s on `random`
    (uniformly random parent ids defeat both prefetcher and per-vertex
    `seen[]` locality). `time(n=131072)/time(n=16384)` ratios of
    4.94/5.56/5.80/7.20 across chain/star/forest/random vs ideal 8.0
    confirm `O(n)` scaling with sub-linear cache effects beyond L2.
    Baseline snapshot at `.codefuse/tracking/perf/ALGO-CN-017.json`.

- **ALGO-CN-016** — `from_prufer` linear-time Prüfer-sequence decoder.
  **Sixteenth member of the `constructors/` family.** Counterpart of
  `igraph_from_prufer` in `references/igraph/src/constructors/prufer.c`.
  Decodes a length-`L` Prüfer sequence over `{0, 1, …, n-1}` (with
  `n = L + 2`) into the unique labelled undirected tree on `n` vertices
  it encodes, completing one half of the Prüfer bijection between
  sequences and trees. Empty input yields the 2-vertex path graph `P_2`
  (single edge `0—1`).
  - `pub fn from_prufer(prufer: &[u32]) -> IgraphResult<Graph>`.
    Linear-time `O(n)` Micikevičius–Caminiti–Deo algorithm: pre-fill
    `degree[w]` = residual occurrences of vertex `w` in the unread tail,
    then for each `i ∈ [0, n)` cascade through leaves discovered as a
    side-effect of edge emission (walk `u`, while `degree[u] == 0 ∧
    u ≤ i ∧ k < L`, emit `(prufer[k], u)`, decrement `degree[prufer[k]]`,
    advance `u ← prufer[k]`), then scan for the lone surviving leaf to
    emit the final closing edge. `u32::try_from(L) + checked_add(2)`
    rejects sequences whose `L + 2` would overflow `u32`, and a single
    pre-decode pass validates every entry is `< n`.
  - 9 unit tests covering: empty Prüfer → `P_2`, both upstream-C
    `igraph_from_prufer.out` fixtures byte-for-byte (`[2,3,2,3]` and
    `[0,2,4,1,1,0]`), out-of-range entry error, constant sequence →
    star, ascending sequence → path, singleton entry, always-undirected
    invariant, no-self-loops + no-duplicates invariant.
  - 5 proptest invariants over `n ∈ [2, 30)`: result has exactly `n`
    vertices + `n − 1` edges + is undirected; no self-loops; no
    duplicate edges; single connected component (union-find with path
    compression — equivalent to "is a tree" since edge count is already
    pinned); out-of-range entry always returns `InvalidArgument`.
  - 11 cross-source conformance fixtures (3 C, 5 py, 3 R) covering the
    two upstream-C `igraph_from_prufer.out` cases plus the empty
    sequence across all three sources, with additional py coverage of
    singleton/constant/ascending/mixed shapes and an R-side round-trip
    from `make_tree(13, 3, undirected) → to_prufer → from_prufer`
    confirming the bijection. Cross-source edge emission order is **not**
    byte-stable (R-igraph re-canonicalises edges), so the conformance
    test compares canonical `(min, max)` edge multisets.
  - Throughput baseline (`benches/bench_prufer.rs`, perf snapshot
    `.codefuse/tracking/perf/ALGO-CN-016.json`): three topology sweeps
    over `n ∈ {64, 1024, 16384, 131072}` — `path` (worst-case cascade)
    ~60 Melem/s, `star` (no cascade) ~51 Melem/s, `random` (cache-
    unfriendly `degree[]` access from random parents) ~16 Melem/s, all
    at the largest size. Time ratios `t(131072)/t(16384)` of 7.83/7.81/
    9.14 confirm near-linear `O(n)` scaling across topologies.
  - Example `cargo run --example prufer_demo` walks through all six
    canonical sequences (empty, two upstream-C fixtures, constant/star,
    ascending/path, mixed) and asserts the tree invariants (vcount/
    ecount, undirected, no self-loops, no duplicates, connected via
    union-find).

- **ALGO-CN-015** — `linegraph` constructor over an existing graph.
  **Fifteenth member of the `constructors/` family.** Counterpart of
  `igraph_linegraph` in `references/igraph/src/constructors/linegraph.c`.
  Produces the line graph `L(G)` of an input `Graph`, where each input
  edge becomes an L-vertex and two L-vertices are joined iff the
  corresponding input edges share an endpoint (undirected) or chain head
  → tail (directed). Self-loops follow upstream `IGRAPH_LOOPS ==
  IGRAPH_LOOPS_TWICE` semantics — a self-loop `(u, u)` appears **twice**
  in the incidence list of `u`, so a self-loop together with `k`
  non-loop edges at `u` emits `2k + 1` L-edges from that vertex.
  - `pub fn linegraph(graph: &Graph) -> IgraphResult<Graph>`. Dispatches
    on `graph.is_directed()`. Undirected branch builds per-vertex
    incidence lists (loops pushed twice) and emits one L-edge per
    `j < k` index pair in each vertex's incidence vector. Directed
    branch builds per-vertex IN-incidence lists once and for each source
    vertex `u` walks `u`'s OUT-incidence × `IN[u]`, emitting
    `(out_eid, in_eid)` for every (out, in) pair whose chaining
    condition `head(out) == tail(in)` is satisfied. `u32::try_from`
    guards the edge → L-vertex id reuse, `usize::checked_mul` /
    `usize::checked_add` size the L-edge buffer before allocation.
    Time complexity `O(|V(G)| + Σ_v deg(v)²)` undirected,
    `O(|V(G)| + Σ_v deg_in(v)·deg_out(v))` directed.
  - 13 unit tests covering: empty graph, isolated vertex, path P_4,
    triangle K_3, K_4, star, ring C_5, multi-edge, undirected self-loop
    (loop-only, loop-then-edge, edge-then-loop), directed chain,
    directed cycle.
  - 4 proptest invariants: vcount(L) = ecount(G); every L-edge
    references a valid L-vertex; undirected loopless input produces
    loopless L; multi-edge multiplicity preserved through L.
  - 13 cross-source conformance fixtures (5 C, 5 py, 3 R) covering
    undirected canonical input from the upstream C unit test, directed
    canonical input, the no-edges edge case, P_4, K_3, K_4, C_5,
    directed P_4, directed 3-cycle, and star S_5. Test compares L-edge
    **multiset** canonicalised to `(min, max)` for undirected and
    as-ordered for directed, matching upstream `igraph_is_same_graph`
    semantics (L-edge emission ordering is not portable across
    py-igraph and R-igraph minor versions).
  - Throughput baseline (`benches/bench_linegraph.rs`, perf snapshot
    `.codefuse/tracking/perf/ALGO-CN-015.json`): path inputs
    ~16 Melem/s of input (also ~16 Melem/s of output), dense K_n inputs
    ~20 M L-edges/s of output, star inputs ~24 M L-edges/s of output,
    directed chains ~26 M arcs/s of input.

- **ALGO-CN-014** — `full_graph` deterministic constructor.
  **Fourteenth member of the `constructors/` family.** Counterpart of
  `igraph_full` in `references/igraph/src/constructors/full.c:54-124`.
  Builds the complete graph `K_n` on `n` vertices in any of the four
  `(directed, loops)` cells of the truth table, with ecounts
  `n·(n−1)/2`, `n·(n+1)/2`, `n·(n−1)`, `n²` respectively. Emission
  order matches upstream byte-for-byte (source-major, target-ascending;
  for the `directed && !loops` cell the inner loop walks `j ∈ [0, i)`
  then `j ∈ (i, n)` so the diagonal is skipped without a per-edge
  branch).
  - `pub fn full_graph(n: u32, directed: bool, loops: bool) -> IgraphResult<Graph>`.
    A four-arm `match (directed, loops)` dispatches to the correct
    emission loop; the loop bodies are kept as a tight pair of nested
    `for` ranges over `u32` so the compiler vectorises the inner
    iteration. `usize::checked_mul` / `usize::checked_add` guards the
    edge-count product before any `Vec::with_capacity`, returning
    `IgraphError::InvalidArgument` on overflow rather than aborting.
    Time complexity `O(|V| + |E|) = O(n²)`.
  - Degenerate inputs handled inline ahead of any buffer allocation:
    `n == 0` → empty graph (vcount = 0, ecount = 0); `n == 1 ∧ !loops`
    → singleton with no edges; `n == 1 ∧ loops` → singleton with a
    single self-loop `(0, 0)`.
  - **Tests**: 9 unit cases (null, singleton ± loops, `K_10` in each
    of the 4 cells with edge-emission spot-checks, ecount-overflow
    rejection, cross-check that `full_graph(4, true, false)`
    reproduces the directed-`K_4` arc set inlined by `kautz(3, 0)`)
    + 3 proptest invariants (ecount matches the closed form for every
    `(n ≤ 12, directed, loops)`; the `loops` flag exactly toggles
    self-loop count; the `(false, false)` cell yields the unordered
    pair set `{ (i, j) : i < j }`).
  - **Three-source conformance**: `tests/conformance/c/full_graph/`
    (7 fixtures: null, singleton ± loops, `K_10` in all 4 cells, with
    full ordered edge lists from `tests/unit/full.out`),
    `tests/conformance/py/full_graph/` (4 fixtures from
    `Graph.Full(n, directed, loops)` covering `K_4` ud-noloops,
    `K_4` ud-loops, `K_3` d-noloops, `K_3` d-loops),
    `tests/conformance/r/full_graph/` (3 fixtures from
    `make_full_graph` covering `K_4` ud-noloops, `K_3` d-loops,
    `K_5` ud-loops); `full_graph_three_source_conformance` walks
    each source directory and rebuilds the graph, asserting the
    ordered edge list matches byte-for-byte.
  - **Bench** (`benches/bench_full.rs`): eight shapes spanning
    `n ∈ {8, 64, 512}` across all four cells. On Apple Silicon the
    throughput hovers at 30-42 Melem/s — at `n = 512` the directed +
    loops cell takes ~9 ms for 262 144 arcs while the undirected +
    no-loops cell takes ~4.1 ms for 130 816 edges, ~1.8× faster than
    python-igraph `Graph.Full(512, True, True)` on the same host.
  - **Example** (`examples/full_demo.rs`): walks every cell of the
    truth table (`K_0`, `K_1` ± loops, `K_4` in all four cells) and
    asserts vcount, ecount, self-loop count, and the unordered-pair
    set for the undirected variant.
- **ALGO-CN-013** — `kautz` deterministic constructor.
  **Thirteenth member of the `constructors/` family.** Counterpart of
  `igraph_kautz` in `references/igraph/src/constructors/kautz.c:61-210`.
  Builds the **directed** Kautz graph `K(m, n)` on `(m+1)·m^n`
  vertices whose labels are length-`n+1` strings over an alphabet of
  `m+1` letters subject to the constraint that no two consecutive
  letters in the string may be equal. An arc goes from
  `v = (a_0, …, a_n)` to every `w = (a_1, …, a_n, b)` with `b ≠ a_n`,
  giving out-degree exactly `m` and in-degree exactly `m` for
  `n ≥ 1`.
  - `pub fn kautz(m: u32, n: u32) -> IgraphResult<Graph>`. Result is
    always directed (matches `IGRAPH_DIRECTED` in C). The sparse
    base-`(m+1)` string codes are mapped to dense vertex ids via two
    inverse tables (`index1` of size `(m+1)^(n+1)`, 1-based sentinel;
    `index2` of size `vcount`) built by a single in-order cursor walk
    over the valid strings using `digits[]` + a positional-weight
    `table[i] = (m+1)^(n-i)`. Arc emission is source-major and
    target-ascending: `basis = (fromvalue·(m+1)) mod allstrings`
    hoisted once per source vertex, then for each `digit ∈ [0, m]`
    with `digit ≠ lastdigit` an arc is emitted to
    `index1[basis + digit] − 1`.
  - Degenerate inputs dispatched ahead of any table allocation:
    `n == 0` → directed `K_{m+1}` with no self-loops (the C source
    delegates to `igraph_full`; this crate inlines the emission to
    avoid pulling in a not-yet-ported helper); `m == 0 ∧ n ≥ 1` →
    empty 0-vertex graph (single-symbol alphabet cannot satisfy the
    no-consecutive-equal constraint); `m == 0 ∧ n == 0` → singleton.
  - Overflow protection: `u32::checked_pow` / `u32::checked_mul` /
    `u32::checked_add` on every dimension (`m+1`, `n+1`, `m^n`,
    `(m+1)·m^n`, `(m+1)^(n+1)`); `usize::checked_mul` for the edge
    buffer length; and a `u64` widening for the
    `(fromvalue·(m+1)) mod allstrings` basis so the intermediate
    product never silently truncates. Out-of-range inputs return
    `IgraphError::InvalidArgument` rather than panicking.
  - Conformance: python-igraph (`Graph.Kautz(m, n)`) and R-igraph
    (`make_kautz_graph(m, n)`) both dispatch to the same upstream
    `igraph_kautz()` C entry point, so the ordered arc list is
    identical across all three sources. The conformance test compares
    the raw ordered arc list rather than the canonicalised multiset —
    any drift in emission order would be a real regression. Fixture
    catalogue: C oracle covers `K(2,1)`, `K(0,10)`, `K(0,0)`,
    `K(5,0)`, `K(3,1)`, `K(2,2)`, `K(3,2)` (7 cases including the
    `tests/unit/igraph_kautz.c` canonical scenarios plus the
    upstream-style degenerate-input gates); python-igraph oracle adds
    `K(2,1)`, `K(3,2)` (2 cases); R-igraph oracle adds `K(2,1)`,
    `K(2,2)` (2 cases) — 11 fixtures total.
  - Structural properties verified by proptest sweep: exactly
    `(m+1)·m^n` vertices, exactly `m·(m+1)·m^n` arcs, every vertex
    has out-degree `m` and in-degree `m`, and the loopless-and-simple
    invariant holds (no `(v, v)` arcs and no duplicated `(u, v)`
    pairs across the full edge list).
  - Performance: criterion bench `bench_kautz` covers six shapes
    spanning the tiny canonical cases through `K(2, 14)` (98 304
    arcs) and `K(5, 5)` (93 750 arcs). Peak throughput
    ~30–38 Marc/s in the L2-friendly band (`k_4_4` / `k_5_5`) and
    ~24 Marc/s at the largest binary shape `k_2_14` where the
    `(m+1)/m × vcount` sparse table dominates allocator pressure.
    Baseline captured at `.codefuse/tracking/perf/ALGO-CN-013.json`.
  - Runnable example: `cargo run --example kautz_demo` walks the
    canonical specialisations including both degenerate paths
    (`K(0, 0)`, `K(0, 5)`, `K(5, 0)`) and the `tests/unit/igraph_kautz.c`
    canonical case `K(2, 1)`, asserting vcount/ecount/total-degree
    and the loopless invariant for each one.

- **ALGO-CN-012** — `de_bruijn` deterministic constructor.
  **Twelfth member of the `constructors/` family.** Counterpart of
  `igraph_de_bruijn` in
  `references/igraph/src/constructors/de_bruijn.c:58-113`. Builds the
  **directed** De Bruijn graph `B(m, n)` on `m^n` vertices whose
  vertices are length-`n` strings drawn from an alphabet of `m`
  symbols. There is an arc from `v = (a_1, …, a_n)` to
  `w = (a_2, …, a_n, b)` for every alphabet symbol `b` — i.e. `w` is
  obtained from `v` by dropping the first symbol and appending one.
  - `pub fn de_bruijn(m: u32, n: u32) -> IgraphResult<Graph>`. Result
    is always directed (matches `IGRAPH_DIRECTED` in C). Vertices are
    encoded as integers in `[0, m^n)` via little-endian base-`m`, and
    the arc rewrite reduces to integer arithmetic: from vertex `i` the
    `m` successors are `((i * m) mod m^n) + b` for `b ∈ [0, m)`.
  - **Arithmetic encoding with basis-hoisting**: the inner loop hoists
    `basis = (i * m) mod m^n` (computed once per source vertex) and
    only adds `b` for each successor — `O(|V| + |E|) = O(m^(n+1))`
    total work with no division or modulus inside the inner loop.
  - **Overflow protection**: `u32::checked_pow(n)` validates the
    vertex count; the edge count `m · m^n` is then checked via
    `usize::checked_mul`. Both surface as
    `IgraphError::InvalidArgument` with a deterministic message
    (`"de_bruijn: m^n overflows u32 …"` or `"de_bruijn: m^(n+1)
    overflows usize …"`) — never produces a silently-wrapped graph.
  - **Degenerate forms** (all match upstream):
    * `n == 0` → singleton directed graph with no arcs (the unique
      length-0 string maps nowhere)
    * `m == 0` → empty directed graph
    * `m == 1, n == 1` → single vertex with one self-loop `(0, 0)`
    * `B(2, 1)` → directed `K_2` plus both self-loops (4 arcs total)
  - **Structural invariants** (verified by test): for `n ≥ 1` every
    vertex has out-degree exactly `m` and in-degree exactly `m`; the
    arc list is emitted in source-major order with successors in
    ascending `b`; every arc `(u, v)` satisfies
    `v ∈ [(u·m) mod m^n, (u·m) mod m^n + m)`.
  - **Tests** (`src/algorithms/constructors/de_bruijn.rs`):
    12 unit tests covering `n=0` singleton, `m=0` null, `m=1 n=1`
    self-loop, `B(2,1)` directed-`K_2`-with-loops, exact arc list of
    `B(2,2)`, `B(3,2)` per-vertex in/out degree audit, `B(2,3)` /
    `B(4,3)` count check, the rewrite rule on `B(3,2)`, the
    `(m, n) = (2, 32)` overflow rejection, and uniform in/out-degree
    sweep over `(m, n) ∈ [1..=4] × [1..=3]`. 3 proptest invariants
    (`proptest-harness` feature) check `vcount == m^n` and
    `ecount == m^(n+1)` exactness, per-vertex in/out-degree equals
    `m`, and the rewrite rule across random valid inputs.
  - **Conformance** (`tests/conformance/{c,py,r}/de_bruijn/`):
    10 fixtures total — 6 from igraph C (`b_2_2`, `b_2_3`, `b_3_2`,
    `b_4_3`, `b_2_1_with_loops`, `b_1_1_singleton_loop`), 2 from
    python-igraph (`b_2_2_py`, `b_3_2_py` via `Graph.De_Bruijn(m, n)`),
    and 2 from R-igraph (`b_2_2_r`, `b_3_2_r` via
    `make_de_bruijn_graph(m, n)`). **Note**: python-igraph and rigraph
    both dispatch directly to the upstream C entry point, so the arc
    emission order is identical across all three sources — the
    cross-source test therefore compares the raw ordered arc list
    rather than canonicalising into a multiset.
  - **Bench** (`benches/bench_de_bruijn.rs` /
    `.codefuse/tracking/perf/ALGO-CN-012.json`): 6 shapes from
    `B(2, 2)` through `B(4, 7)` (16 384 vertices, 65 536 arcs).
    Throughput plateaus around 11-23 Marc/s for the larger shapes;
    `B(2, 2)` clocks ~370 ns end-to-end. The inner loop is dominated
    by the `Vec::push` and final `Graph::add_edges` copy — the
    arithmetic itself is essentially free.
  - **Example** (`examples/de_bruijn_demo.rs`): walks the canonical
    specialisations `B(1, 1)`, `B(2, 1)`, `B(2, 2)`, `B(2, 3)`, and
    `B(3, 2)`; verifies the standard structural invariants
    (vcount/ecount/total degree) for each one; and demonstrates the
    rewrite rule by iterating every arc of `B(3, 2)` and asserting
    `v ∈ [(u·3) mod 9, (u·3) mod 9 + 3)`.

- **ALGO-CN-011** — `circulant` deterministic constructor.
  **Eleventh member of the `constructors/` family.** Counterpart of
  `igraph_circulant` in
  `references/igraph/src/constructors/circulant.c:70-170`. Builds the
  circulant graph `G(n, shifts)` on `n` vertices where for each
  `s ∈ shifts` (after normalisation) edge `(v_j, v_{(j+s) mod n})` is
  emitted for every `j ∈ [0, n)`.
  - `pub fn circulant(n: u32, shifts: &[i64], directed: bool) -> IgraphResult<Graph>`.
    Supports both directed and undirected output; accepts negative
    shifts (lifted into `[0, n)` via signed `mod`); overflow guarded
    via `checked_mul` on the edge-`Vec` pre-allocation.
  - **Shift normalisation**: `s := s mod n` lifted to `[0, n)`; then for
    `!directed`, shifts `s ≥ (n + 1) / 2` fold to `n − s` (so shift 3 on
    `n = 5` collapses to shift 2, avoiding duplicate undirected edges);
    zero shift always suppressed (would emit self-loops); duplicates
    deduped via linear `seen.contains` scan (kept linear because
    `|shifts|` is typically small).
  - **Edge emission**: per canonical shift, push `(j, (j + s) mod n)`
    for `j ∈ [0, limit)`, where `limit = n / 2` when
    `!directed && n is even && s == n / 2` (even-`n` antipodal shift
    halves to avoid duplicating the perfect matching), else `limit = n`.
    Total work `O(n · |canonical shifts|)` plus `Graph::add_edges`.
  - **Canonical generalisations** verified by test:

    | call                                  | equivalent                       | note                          |
    |---------------------------------------|----------------------------------|-------------------------------|
    | `circulant(n, &[1], false)`           | `ring_graph(n)` = `C_n`          | basic cycle                   |
    | `circulant(n, &[k], false)`           | inner layer of `G(n, k)`         | for `0 < k < n/2`             |
    | `circulant(n, &[1, …, n/2], false)`   | `K_n`                            | every distinct undirected shift |
    | `circulant(n, &[s], true)`            | directed cycle of step `s`       | per-shift orientation kept    |

  - **Structural properties** (covered by unit + proptest sweep):
    every `circulant(n, shifts, false)` is simple (no loops, no
    parallel edges) and vertex-regular with degree
    `2·|canonical shifts after halving|`; negative shifts always
    equivalent to their positive normalisation.
  - **Bounds & overflow**: shift normalisation uses signed `i64` mod;
    `u32::try_from` guards the lifted value; `n * |canonical shifts|`
    edge capacity uses `checked_mul`.
  - **Tests** (`src/algorithms/constructors/circulant.rs`):
    17 unit tests covering the empty graph, `n = 1`, ring equivalence,
    directed-cycle equivalence, suppression of shift = 0, duplicate
    shifts, the complementary-shift collapse, even-`n` antipodal
    halving, odd-`n` no-shortcut, the complete-graph specialisation,
    negative-shift normalisation, simplicity, and equivalence with the
    inner layer of `generalized_petersen`. 4 proptest invariants
    (`proptest-harness` feature) check simplicity of undirected
    output, vertex-regularity, negative-shift equivalence, and the
    `K_n` specialisation across random valid inputs.
  - **Conformance** (`tests/conformance/{c,py,r}/circulant/`):
    10 fixtures total — 6 from igraph C (`c_5_shifts_1`,
    `c_6_shifts_1_3_antipodal`, `c_7_shifts_1_2_squared_cycle`,
    `k4_shifts_1_2_complete`, `c_5_shifts_neg1_directed`,
    `c_8_shifts_1_3_directed`), 2 from python-igraph
    (`circulant_py_c_5_shifts_1_ring` via `Graph.Ring(5)` and
    `circulant_py_k4_shifts_1_2_tetrahedral` via
    `Graph.Famous('Tetrahedral')`), and 2 from R-igraph
    (`circulant_r_c_6_shifts_1_ring` via `make_ring(6)` and
    `circulant_r_k5_shifts_1_2_full` via `make_full_graph(5)`).
    **Note**: python-igraph 0.11.x and rigraph do NOT expose a direct
    `Graph.Circulant` constructor, so py/R fixtures use the canonical
    equivalents from the truth table above; the cross-source test
    compares canonicalised `(lo, hi)` edge multisets for undirected
    fixtures and preserves orientation for directed ones.
  - **Bench**
    (`benches/bench_circulant.rs` /
    `.codefuse/tracking/perf/ALGO-CN-011.json`): 6 shapes from `C_5`
    through `circulant(10000, [1, 5, 17, 101])`. Throughput tops out
    around 27 Medge/s for large `n`; the even-`n` antipodal shape
    halves the per-shift work as expected. Small-`n` shapes pay
    600 ns – 2.2 µs in `Graph::new` + `add_edges`.
  - **Example** (`examples/circulant_demo.rs`): walks five canonical
    specialisations — `C_5`, `C_6 + matching` (shift `[1, 3]`),
    squared cycle on 7 vertices, `K_4` via `[1, 2]`, and a directed
    8-vertex circulant — and verifies the standard structural
    invariants (vcount/ecount/regularity) for each one.

- **ALGO-CN-010** — `generalized_petersen` deterministic constructor.
  **Tenth member of the `constructors/` family.** Counterpart of
  `igraph_generalized_petersen` in
  `references/igraph/src/constructors/generalized_petersen.c:58-95`.
  Builds the generalized Petersen graph `G(n, k)` on `2n` vertices in
  two layers: outer cycle `v_0..v_{n-1}` on ids `0..n` and inner
  circulant `u_0..u_{n-1}` of shift `k` on ids `n..2n`, joined by `n`
  rung edges `v_i — u_i` — exactly `3n` edges.
  - `pub fn generalized_petersen(n: u32, k: u32) -> IgraphResult<Graph>`.
    Always undirected (the C constructor is fixed `IGRAPH_UNDIRECTED`).
    Constraints `n ≥ 3` and `0 < k < n/2` (strict — `2k < n`; `2k = n`
    would emit parallel inner edges); overflow guarded via
    `checked_mul` on both `2*n` and `3*n`.
  - **Algorithm**: for each `i ∈ [0, n)` push the outer cycle edge
    `(i, (i+1) mod n)`, the rung `(i, i+n)`, and the inner circulant
    edge `(i+n, ((i+k) mod n) + n)`, in that order. Total work
    `O(n)` plus the cost of `Graph::add_edges`.
  - **Famous specializations**:

    | n  | k | name                          | vcount | ecount | note                |
    |----|---|-------------------------------|--------|--------|---------------------|
    | 5  | 2 | Petersen                      | 10     | 15     | 3-reg, girth 5      |
    | 8  | 3 | Möbius–Kantor                 | 16     | 24     | bipartite, girth 6  |
    | 10 | 3 | Desargues                     | 20     | 30     | bipartite           |
    | 12 | 5 | Nauru                         | 24     | 36     | symmetric           |

  - **Structural properties** (covered by unit + proptest sweep):
    every `G(n,k)` is 3-regular and simple; the outer-layer subgraph
    is exactly `C_n`; rungs cover every `i`; the inner layer is
    exactly the circulant with shift `k`; closed-form `|V| = 2n`,
    `|E| = 3n`.
  - **Bounds & overflow**: `n < 3` rejected; `k = 0` rejected; `2k ≥ n`
    rejected (covers both `2k = n` parallel-edge case and `k > n/2`);
    `2*n` and `3*n` overflow checked.
  - **Tests** (`src/algorithms/constructors/generalized_petersen.rs`):
    14 unit tests covering Petersen, Möbius–Kantor, Desargues, Nauru,
    a 3-regularity sweep over all valid `(n, k)` pairs with
    `n ∈ [3, 20]`, the canonical edge-emission order for `G(5, 2)`,
    presence of outer-cycle / rung / inner-circulant edges, and every
    validation error path. 5 proptest invariants
    (`proptest-harness` feature) check `|V| = 2n`, `|E| = 3n`,
    3-regularity + simplicity, the outer cycle, the rungs, and the
    inner-circulant layer across random valid `(n, k)`.
  - **Conformance** (`tests/conformance/{c,py,r}/generalized_petersen/`):
    8 fixtures total — 6 from igraph C
    (`g_3_1`, `g_4_1`, `g_5_2_petersen`, `g_6_2`, `g_7_2`,
    `g_8_3_mobius_kantor`), 1 from python-igraph
    (`Graph.Famous('Petersen')` = G(5,2)), and 1 from R-igraph
    (`make_graph('Petersen')`). Emission order is internal-loop-driven
    and differs from the upstream Famous-database layouts, so the
    cross-source test compares canonicalised edge multisets — the
    same strategy used by `square_lattice` above. Famous layouts for
    Dodecahedron / Möbius–Kantor / Desargues / Nauru are isomorphic
    to GPGs but use polytope-embedded vertex labellings that the
    multiset compare cannot reconcile without isomorphism, so they
    are deliberately out of py/R scope.
  - **Bench**
    (`benches/bench_generalized_petersen.rs` /
    `.codefuse/tracking/perf/ALGO-CN-010.json`): 6 shapes from the
    eponymous Petersen up through `G(10000, 101)`. Throughput tops out
    around 24 Medge/s for large `n`; small-`n` shapes pay 1–2 µs in
    `Graph::new` + `add_edges`. Work per call is exactly `3n` edge
    pushes — a single linear loop.
  - **Example** (`examples/generalized_petersen_demo.rs`): walks the
    four canonical specializations (Petersen, Möbius–Kantor,
    Desargues, Nauru) and verifies the standard structural invariants.

- **ALGO-CN-009** — `square_lattice` deterministic constructor.
  **Ninth member of the `constructors/` family.** Counterpart of
  `igraph_square_lattice` in
  `references/igraph/src/constructors/lattice.c:60-298`. Builds a
  d-dimensional Cartesian-product grid over a shape
  `[n_0, n_1, …, n_{d-1}]` with optional per-axis torus wrap and
  optional bidirectional arc emission. Vertex ids follow the
  little-endian convention: lattice site `(i_0, i_1, …, i_{d-1})`
  carries id `i_0 + n_0·i_1 + n_0·n_1·i_2 + …`, matching upstream.
  - `pub fn square_lattice(dim: &[u32], nei: u32, directed: bool, mutual: bool, periodic: Option<&[bool]>) -> IgraphResult<Graph>`.
    No artificial cap on `dim.len()` or per-axis `n_d`: vertex-count
    product is chained through `checked_mul`, stride walk uses
    `checked_mul`, and a slice length over `u32::MAX` is rejected with
    `InvalidArgument`. `nei = 1` is the only currently supported
    neighbourhood radius — `nei >= 2` (handled in C by a post-hoc
    `igraph_connect_neighborhood` pass) returns `InvalidArgument` and
    defers to a future AWU; `nei = 0` is also rejected.
  - **Algorithm**: for each vertex and each axis `d` emit the forward
    neighbour `(v, v + stride_d)` whenever `coord_d + 1 < n_d`, plus
    the wrap edge `(v, v − (n_d − 1)·stride_d)` when `periodic[d]` is
    set. Wrap edges are **suppressed for dimension size 2** so the
    cycle does not duplicate the forward edge into a parallel pair —
    same rule as upstream. `directed && mutual` additionally emits the
    symmetric back arc for each forward and wrap edge so undirected
    edges become bidirectional arc pairs; `directed && !mutual`
    produces the same forward set without back arcs. Total work:
    `O(|V| · d) = O(|E|)`.
  - **Truth table** for `square_lattice(dim, 1, directed, mutual, periodic)`:

    | dim          | periodic         | directed | mutual | vcount | ecount | shape                |
    |--------------|------------------|----------|--------|--------|--------|----------------------|
    | `[]`         | n/a              | false    | false  | 1      | 0      | singleton (0-d)      |
    | `[3]`        | none             | false    | false  | 3      | 2      | path `P_3`           |
    | `[3]`        | `[true]`         | false    | false  | 3      | 3      | cycle `C_3`          |
    | `[3, 3]`     | none             | false    | false  | 9      | 12     | 2-D grid             |
    | `[3, 3]`     | `[true, true]`   | false    | false  | 9      | 18     | 2-D torus (4-reg.)   |
    | `[2, 2, 2]`  | none             | false    | false  | 8      | 12     | cube `Q_3`           |
    | `[3]`        | none             | true     | true   | 3      | 4      | bidir. `P_3` arcs    |

  - **Structural properties** (covered by both unit and proptest):
    undirected non-periodic has `Σ_d (Π_{i≠d} n_i)·(n_d − 1)` edges;
    undirected periodic doubles each cycle dim subject to the
    `n_d > 2` suppression rule; 2-D `n × n` torus is 4-regular when
    `n ≥ 3`; `[2, 2, 2]` reproduces the hypercube `Q_3` edge-by-edge
    against [`hypercube`]; `directed && mutual` doubles every
    undirected edge count exactly.
  - **Bounds & overflow**: dimension count over `u32::MAX` rejected;
    vertex-count and stride products use `checked_mul`; `nei == 0` or
    `nei >= 2` rejected with `InvalidArgument`; mismatched
    `periodic.len() != dim.len()` rejected.
  - **Tests** (`src/algorithms/constructors/square_lattice.rs`): 17
    unit tests covering 0-d singleton, 1-d path, 1-d periodic cycle
    (with the wrap canonicalisation that turns `(2, 0)` into
    `(0, 2)`), 2-d 3×3 grid, 2-d 3×3 torus regularity, 2-D dim-2
    wrap suppression, 3-D `[2, 2, 2]` ≡ `Q_3` edge-by-edge, directed
    only, directed+mutual edge count doubling on `P_3`, and all
    validation error paths. 4 proptest invariants
    (`proptest-harness` feature) check edge-count formula across
    random shapes, every emitted edge connects coords that differ in
    exactly one position by 1 (mod `n_d` when periodic), wrap
    suppression for dim 2, and the cube ≡ `Q_3` equivalence on random
    `[2, 2, k]` shapes.
  - **Conformance** (`tests/conformance/{c,py,r}/square_lattice/`):
    8 C cases (zero-singleton, 1-d path, 1-d periodic cycle,
    2×2 four-cycle, 3×3 grid, 3×3 torus, `[2, 2, 2]` cube,
    1-d directed+mutual), 4 python-igraph cases, and 4 R-igraph
    cases. Because the lattice emission order is **not portable
    across upstream sources** (python, R, and our port produce
    different edge orderings for the same shape — especially with
    torus wraps), the conformance test compares the **canonicalised
    edge multiset** for both directed and undirected variants. Total:
    8 + 4 + 4 = 16 fixtures.
  - **Bench** (`benches/bench_square_lattice.rs`,
    `.codefuse/tracking/perf/ALGO-CN-009.json`): 7 bench points across
    4 shapes (2-d 100×100, 2-d 200×200, 3-d 30×30×30, 2-d 100×100
    torus). Throughput stays in the **19.5 – 33.0 Medge/s** band:
    undirected non-periodic ≈ 30 – 33 Medge/s, 2-D torus drops to
    ≈ 22 Medge/s due to the branchy wrap path, `directed && mutual`
    drops to ≈ 20 Medge/s due to the symmetric back-arc emission.
  - **Example** (`examples/square_lattice_demo.rs`): walks 1-D path,
    1-D periodic cycle, 2-D 3×3 grid, 2-D 3×3 torus, 3-D `[2, 2, 2]`
    cube ≡ `Q_3`, and 1-D directed+mutual with structural assertions
    on every variant.
- **ALGO-CN-008** — `hamming` deterministic constructor.
  **Eighth member of the `constructors/` family.** Counterpart of
  `igraph_hamming` in
  `references/igraph/src/constructors/regular.c:1070-1153`. The
  d-dimensional **Hamming graph** `H(n, q)` over an alphabet of size
  `q` has `q^n` vertices indexed by length-`n` strings in
  `{0, 1, …, q-1}^n`. Two vertices are connected iff their strings
  differ in **exactly one position**. The string `(x_0, x_1, …, x_{n-1})`
  maps to vertex id `Σ_i x_i · q^i` (little-endian base-`q`), matching
  the upstream convention.
  - `pub fn hamming(n: u32, q: u32, directed: bool) -> IgraphResult<Graph>`.
    No artificial cap on `(n, q)`: the vertex count `q^n` is guarded at
    runtime via `u32::checked_pow`, and the edge count
    `q^n · (q − 1) · n / 2` is guarded via chained `checked_mul`. The
    degenerate cases `n = 0` (singleton, even when `q = 0`), `n > 0` ∧
    `q = 0` (null graph), and `q = 1` (singleton) match upstream.
  - **Algorithm**: for each `v ∈ [0, q^n)` walk the digits in base `q`
    by stepping `pos = 1, q, q^2, …`. At each digit position extract
    `dig = (v / pos) mod q`, then for every higher value
    `dig + j` with `j ∈ [1, q − dig)` emit the canonical edge
    `(v, v + j · pos)`. Walking upward in every digit means each edge
    is produced exactly once (lower endpoint first). Mirrors the
    upstream C loop. Total work: `O(n · q^(n+1))`.
  - **Truth table** for `hamming(n, q, directed)`:

    | n | q | directed | vcount | ecount | shape                            |
    |---|---|----------|--------|--------|----------------------------------|
    | 0 | * | false    | 1      | 0      | singleton (any `q`, even `q=0`)  |
    | 2 | 0 | false    | 0      | 0      | null graph                       |
    | 1 | 3 | false    | 3      | 3      | complete `K_3`                   |
    | 2 | 3 | false    | 9      | 18     | 4-regular                        |
    | 2 | 4 | false    | 16     | 48     | 6-regular                        |
    | 3 | 2 | false    | 8      | 12     | equivalent to hypercube `Q_3`    |
    | 3 | 2 | true     | 8      | 12     | low → high orientation           |

  - **Structural properties** (covered by both unit and proptest):
    `n·(q−1)`-regular (every vertex has degree `n · (q − 1)`); edge
    count exactly `q^n · (q − 1) · n / 2`; every edge differs in
    exactly one base-`q` digit; every emitted pair is canonically
    ordered `u < v`; `q = 2` coincides with the hypercube `Q_n`
    (asserted edge-by-edge against [`hypercube`]).
  - **Bounds & overflow**: vertex-count overflow (e.g. `n = 32, q = 2`
    or `n = 5, q = 100`) rejected with `InvalidArgument`; edge-count
    computation uses chained `checked_mul` so future formula changes
    cannot silently wrap around.
  - Conformance: 14 fixtures (C × 6 covering `n = 0` singleton, `q = 0`
    null graph, `H(1, 3) = K_3`, `H(2, 3)` with 18 edges, `H(3, 2) ≡ Q_3`,
    and `H(2, 4)` with 48 edges; py × 4 including a directed variant;
    R × 4) under `tests/conformance/{c,py,r}/hamming/`, comparing exact
    edge sequences (directed mode) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_hamming.rs` with two groups
    (undirected / directed) across `(n, q) ∈ {(3, 3), (4, 4), (5, 4)}`
    (27 / 256 / 1024 vertices respectively). Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-CN-008.json` — ~25–39 Medge/s, with
    throughput consistent across shapes (the base-`q` digit walk does
    exactly the formula's emission count, with a small per-edge cost
    that does not depend on `q`).

- **ALGO-CN-007** — `hypercube` deterministic constructor.
  **Seventh member of the `constructors/` family.** Counterpart of
  `igraph_hypercube` in
  `references/igraph/src/constructors/regular.c:983-1003`. The
  `n`-dimensional hypercube graph `Q_n` has `2^n` vertices and
  `2^(n-1) · n` edges; two vertices are connected iff their zero-based
  IDs differ in **exactly one bit**.
  - `pub fn hypercube(n: u32, directed: bool) -> IgraphResult<Graph>`
    and `pub const MAX_HYPERCUBE_DIMENSION: u32 = 30`. The cap keeps
    `1u32 << n` always well-defined, `2^n` comfortably within `u32`,
    and the edge `Vec` allocation within 64-bit `usize` even at the
    upper bound.
  - **Algorithm**: enumerate every vertex `v ∈ [0, 2^n)`; for each bit
    `i ∈ [0, n)` toggle to get `u = v ^ (1 << i)` and emit the
    canonical edge `(v, u)` only when `v < u` so each edge is produced
    exactly once. Mirrors the upstream C loop. Total work
    `O(2^n · n) = O(|E|)`.
  - **Truth table** for `hypercube(n, directed)`:

    | n | directed | vcount | ecount | shape                       |
    |---|----------|--------|--------|-----------------------------|
    | 0 | false    | 1      | 0      | singleton                   |
    | 1 | false    | 2      | 1      | K_2                         |
    | 2 | false    | 4      | 4      | 4-cycle                     |
    | 3 | false    | 8      | 12     | cube (3-regular, bipartite) |
    | 3 | true     | 8      | 12     | cube, low → high arcs       |
    | 4 | false    | 16     | 32     | tesseract                   |

  - **Structural properties** (covered by both unit and proptest):
    `n`-regular (every vertex has degree `n`), bipartite split by
    parity of `popcount(id)`, every edge has Hamming distance 1, and
    every emitted pair is canonically ordered `u < v`.
  - **Bounds & overflow**: `n > MAX_HYPERCUBE_DIMENSION (30)` rejected
    with `InvalidArgument`; edge-count computation uses
    `checked_shl` / `checked_mul` so any future bound change cannot
    silently wrap around.
  - Conformance: 14 fixtures (C × 6 covering `n ∈ {0, 1, 2, 3, 4}` and
    the directed variant at `n = 3`, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/hypercube/`, comparing exact edge
    sequences (directed mode) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_hypercube.rs` with two groups
    (undirected / directed) across `n ∈ {6, 10, 14}` (64 / 1024 / 16384
    vertices respectively). Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-CN-007.json` — ~27–34 Medge/s,
    indistinguishable across the two variants (orientation flag does
    not alter enumeration work).

- **ALGO-CN-006** — `regular_tree` deterministic constructor.
  **Sixth member of the `constructors/` family.** Counterpart of
  `igraph_regular_tree` in
  `references/igraph/src/constructors/regular.c:843-865`. A regular
  tree (Bethe lattice) of height `h` and degree `k` is a rooted tree
  where every non-leaf vertex has total degree exactly `k`:
  - the root has `k` children (its total degree is `k`),
  - internal non-root vertices have `k − 1` children plus their parent,
    so their total degree is also `k`,
  - leaves sit at depth `h` and have degree 1.
  - `pub fn regular_tree(h: u32, k: u32, mode: TreeMode) -> IgraphResult<Graph>`
    reusing the shared `TreeMode ∈ {Out, In, Undirected}` enum.
  - **Algorithm**: thin wrapper that builds
    `branches = [k, k-1, k-1, ..., k-1]` of length `h` and delegates to
    `symmetric_tree` (CN-005). Total work is O(|V|) edge enumeration +
    `Graph::add_edges`. Overflow propagates via `symmetric_tree`'s
    `checked_mul` / `checked_add` so pathological inputs surface as
    `InvalidArgument` rather than panicking. Rejects `h < 1` or `k < 2`
    with `InvalidArgument`.
  - **Three-mode truth table** for `regular_tree(h, k, mode)`:

    | h | k | mode        | directed | vcount | ecount | shape                |
    |---|---|-------------|----------|--------|--------|----------------------|
    | 1 | 3 | Out         | true     | 4      | 3      | star K1,3            |
    | 2 | 3 | Out         | true     | 10     | 9      | Bethe `[3, 2]`       |
    | 2 | 3 | In          | true     | 10     | 9      | Bethe (reversed)     |
    | 2 | 3 | Undirected  | false    | 10     | 9      | undirected Bethe     |
    | 2 | 4 | Out         | true     | 17     | 16     | Bethe `[4, 3]`       |
    | 3 | 2 | Out         | true     | 7      | 6      | degenerate `[2,1,1]` |

  - **Degenerate cases**: `h = 1` → star K1,k anchored at vertex 0 with
    `k` leaves; `k = 2` → linear chain because `branches = [2, 1, 1, …]`
    collapses to a path after the root.
  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/regular_tree/`, comparing exact edge
    sequences (directed modes) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_regular_tree.rs` with three groups
    (Out / In / Undirected) across `(h, k) ∈ {(3, 3), (4, 4), (5, 5)}`
    (22 / 365 / 6831 vertices respectively). Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-CN-006.json` — ~16-240 Melem/s,
    indistinguishable across the three modes; tracks the
    `symmetric_tree` baseline because this is a thin wrapper.
  - Example: `examples/regular_tree_demo.rs` walks star K1,3 (h=1,
    k=3), all three modes of the canonical Bethe lattice (h=2, k=3),
    the denser variant (h=2, k=4), and the degenerate chain (h=3,
    k=2).
  - 12 unit + 3 proptest checks in
    `src/algorithms/constructors/regular_tree.rs` covering the star
    K1,k case, the canonical Bethe lattice, the In/Undirected
    orientations, the degenerate k=2 chain, the h=0 / k<2 rejections,
    overflow propagation, and the structural invariants
    `deg(root) = k` and `deg(internal non-root) = k`.
- **ALGO-CN-005** — `symmetric_tree` deterministic constructor.
  **Fifth member of the `constructors/` family.** Counterpart of
  `igraph_symmetric_tree` in
  `references/igraph/src/constructors/regular.c:706-808`. Whereas
  `kary_tree` uses a single uniform branching factor, this constructor
  takes a slice of branching factors and applies one per BFS depth:
  `branches[d]` children at depth `d`.
  - `pub fn symmetric_tree(branches: &[u32], mode: TreeMode) -> IgraphResult<Graph>`
    reusing the shared `TreeMode ∈ {Out, In, Undirected}` enum.
  - **Algorithm**: O(|V|) — BFS level walk that mirrors the upstream C
    loop. At each level `k`, every parent in `[parent..level_end)` is
    expanded by pushing exactly `branches[k]` arcs apiece;
    `level_end = child` is snapshotted before the inner loop so only
    current-level parents are expanded. Vertex count is computed via
    `checked_mul`/`checked_add` so pathological inputs (e.g. `[2; 32]`)
    surface as `InvalidArgument` rather than panicking. Rejects any
    `branches[d] == 0` with `InvalidArgument`.
  - **Three-mode truth table** for `branches = [2, 2]` (mirrors
    `kary_tree(7, 2, ...)`):

    | mode        | directed | vcount | ecount | first edge | last edge   |
    |-------------|----------|--------|--------|------------|-------------|
    | Out         | true     | 7      | n-1=6  | (0, 1)     | (2, 6)      |
    | In          | true     | 7      | n-1=6  | (1, 0)     | (6, 2)      |
    | Undirected  | false    | 7      | n-1=6  | (0, 1)*    | (2, 6)*     |

    `*` Undirected storage canonicalises endpoints as `(min, max)`.
  - **Degenerate cases**: `branches = []` → singleton root (1 vertex, 0
    edges); `branches = [1, 1, ...]` → linear chain (path);
    `branches = [k]` → star K1,k anchored at vertex 0;
    `branches[d] == 0` → `InvalidArgument`.
  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/symmetric_tree/`, comparing exact edge
    sequences (directed modes) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_symmetric_tree.rs` with three groups
    (Out / In / Undirected) across `branches ∈ {[3,3,3], [4,4,4,4],
    [5,5,5,5,5]}` (40 / 341 / 3906 vertices respectively). Baseline
    snapshot: `.codefuse/tracking/perf/ALGO-CN-005.json` —
    ~21-58 Melem/s, indistinguishable across the three modes.
  - Example: `examples/symmetric_tree_demo.rs` walks through all three
    modes plus mixed `[3,2]`, deep `[3,2,1]`, linear chain `[1,1,1]`,
    star-collapse `[3]`, and the singleton corner case.
  - 12 unit + 3 proptest checks in
    `src/algorithms/constructors/symmetric_tree.rs` covering the
    3-mode truth table, mixed branching, linear chain, single-level
    star, overflow rejection, and the bounds / degenerate / error
    cases.
- **ALGO-CN-004** — `kary_tree` deterministic constructor.
  **Fourth member of the `constructors/` family.** Counterpart of
  `igraph_kary_tree` in `references/igraph/src/constructors/regular.c`.
  Emits a rooted tree on `n` vertices in BFS order: vertex `0` is the
  root, and at each parent index `p`, up to `children` consecutive
  `(parent, to)` pairs are pushed while `to` advances by one each push;
  the last parent may emit a short batch.
  - `pub fn kary_tree(n: u32, children: u32, mode: TreeMode) -> IgraphResult<Graph>`
    with `TreeMode ∈ {Out, In, Undirected}`.
  - **Algorithm**: O(|V|) single sweep — one
    `Vec::with_capacity(n - 1)` allocation plus a single
    `Graph::add_edges` call. Edge orientation flips on `TreeMode::In`
    ((to, parent)) and stays parent-child on `Out` and `Undirected`.
    Rejects `children == 0` with `InvalidArgument`.
  - **Three-mode truth table** for `n = 7, children = 2`:

    | mode        | directed | ecount  | first edge | last edge   |
    |-------------|----------|---------|------------|-------------|
    | Out         | true     | n-1 = 6 | (0, 1)     | (2, 6)      |
    | In          | true     | n-1 = 6 | (1, 0)     | (6, 2)      |
    | Undirected  | false    | n-1 = 6 | (0, 1)*    | (2, 6)*     |

    `*` Undirected storage canonicalises endpoints as `(min, max)`.
  - **Degenerate cases**: `n = 0` → empty graph; `n = 1` → singleton;
    `children >= n - 1` → collapses to a star anchored at vertex 0;
    `children = 1` → linear chain (path).
  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/kary_tree/`, comparing exact edge
    sequences (directed modes) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_kary_tree.rs` with three groups
    (Out / In / Undirected) across `n ∈ {100, 10_000, 1_000_000}`.
    Baseline snapshot: `.codefuse/tracking/perf/ALGO-CN-004.json`.
  - Example: `examples/kary_tree_demo.rs` walks through all three
    modes plus partial-batch ternary, linear chain, and star-collapse
    shapes.
  - 11 unit + 3 proptest checks in
    `src/algorithms/constructors/kary_tree.rs` covering the 3-mode
    truth table, partial last batch, linear chain, star collapse and
    the bounds / degenerate / error cases.
- **ALGO-CN-003** — `wheel_graph` deterministic constructor.
  **Third member of the `constructors/` family.** Counterpart of
  `igraph_wheel` in `references/igraph/src/constructors/regular.c`.
  Built as a star plus a rim cycle: the centre is connected to every
  rim vertex via spokes, and the rim vertices are linked in a cycle.
  - `pub fn wheel_graph(n: u32, mode: WheelMode, center: u32) -> IgraphResult<Graph>`
    with `WheelMode ∈ {Out, In, Mutual, Undirected}` (one-to-one with
    [`StarMode`]).
  - **Algorithm**: O(|V|) — delegates the star layer to `star_graph`,
    then sweeps the rim in raw vertex-id order, skipping `center`.
    `Out`/`In`/`Undirected` emit each rim edge once (`prev → next`);
    `Mutual` appends the reverse of every forward rim arc in
    **reverse-discovery order**, matching the upstream C loop. The
    wrap-around rim edge connects the largest rim vertex back to the
    smallest.
  - **Four-mode truth table** for `n = 5, center = 0`:

    | mode        | directed | ecount        |
    |-------------|----------|---------------|
    | Out         | true     | 2(n-1) = 8    |
    | In          | true     | 2(n-1) = 8    |
    | Mutual      | true     | 4(n-1) = 16   |
    | Undirected  | false    | 2(n-1) = 8    |

  - **Degenerate cases** (documented and intentional, matching
    upstream): `n=2` produces a self-loop on the only rim vertex
    (rim collapses to a 1-cycle); `n=3` produces parallel rim edges
    `(1,2)` and `(2,1)` (rim collapses to a 2-cycle). `n=0` and
    `n=1` reduce to the empty/singleton star.
  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/wheel_graph/`, comparing exact edge
    sequences (directed modes) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_wheel.rs` with four groups (Out / In /
    Mutual / Undirected) across `n ∈ {100, 10_000, 1_000_000}`.
    Baseline snapshot: `.codefuse/tracking/perf/ALGO-CN-003.json`.
  - Example: `examples/wheel_demo.rs` walks through all four modes
    plus a non-zero centre and the two degenerate small-`n` shapes.
  - 10 unit + 3 proptest checks in
    `src/algorithms/constructors/wheel.rs` covering the 4-mode truth
    table, centre placement (zero / interior), bounds checking, rim
    cycle topology and the degenerate small-`n` shapes.
- **ALGO-CN-002** — `star_graph` deterministic constructor.
  **Second member of the `constructors/` family.** Counterpart of
  `igraph_star` in `references/igraph/src/constructors/regular.c`.
  - `pub fn star_graph(n: u32, mode: StarMode, center: u32) -> IgraphResult<Graph>`
    with `StarMode ∈ {Out, In, Mutual, Undirected}`.
  - **Algorithm**: O(|V|) single sweep over leaves visited in raw
    vertex-id order `[0, center) ∪ (center, n)`. Per leaf,
    `Out` emits `(center, leaf)`; `In` and `Undirected` emit
    `(leaf, center)`; `Mutual` emits both `(center, leaf)` then
    `(leaf, center)` (forward arc always first, matching the upstream
    C loop). Degenerate cases mirror upstream: `n=0` → empty graph;
    `n=1` → no edges; `center == n - 1` → leaves
    `0..center` only.
  - **Four-mode truth table** for `n = 5, center = 0`:

    | mode        | directed | ecount |
    |-------------|----------|--------|
    | Out         | true     | 4      |
    | In          | true     | 4      |
    | Mutual      | true     | 8      |
    | Undirected  | false    | 4      |

  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/star_graph/`, comparing exact edge
    sequences (directed modes) or canonicalised edge multisets
    (undirected mode).
  - Benchmarks: `benches/bench_star.rs` with four groups (Out / In /
    Mutual / Undirected) across `n ∈ {100, 10_000, 1_000_000}`.
    Baseline snapshot: `.codefuse/tracking/perf/ALGO-CN-002.json`.
  - Example: `examples/star_demo.rs` walks through all four modes plus
    a non-zero centre and prints the per-mode edge list.
  - 13 unit + 3 proptest checks in
    `src/algorithms/constructors/star.rs` covering the 4-mode truth
    table, centre placement (zero / interior / last), bounds checking,
    and edge-count formula.
- **ALGO-CN-001** — `ring_graph` deterministic constructor with the
  `path_graph` / `cycle_graph` convenience wrappers. **First member of
  the `constructors/` family.** Counterpart of `igraph_ring`,
  `igraph_path_graph` and `igraph_cycle_graph` in
  `references/igraph/src/constructors/regular.c`.
  - `pub fn ring_graph(n: u32, directed: bool, mutual: bool, circular: bool) -> IgraphResult<Graph>`
    plus the `path_graph(n, directed, mutual)` (alias for
    `circular = false`) and `cycle_graph(n, directed, mutual)` (alias
    for `circular = true`) one-line wrappers.
  - **Algorithm**: lay `n` vertices in a sequence `0, 1, …, n-1`,
    emit forward arcs `(i, i+1)`. When `directed && mutual` also emit
    the back-arc `(i+1, i)`. When `circular` close with `(n-1, 0)` and
    (if also mutual) `(0, n-1)`. The undirected case ignores the
    `mutual` flag entirely (matches upstream). Degenerate cases mirror
    upstream verbatim: `n=0` → empty graph; `n=1, circular` → self-loop
    `(0, 0)`; `n=2, circular, undirected` → two parallel edges.
  - **When to choose it**: deterministic, `O(|V|)`, no RNG involved —
    use as a building block for benchmarks, conformance fixtures, or
    anywhere a `P_n` / `C_n` is needed by name.
  - Conformance: 14 fixtures (C × 6, py × 4, R × 4) under
    `tests/conformance/{c,py,r}/ring_graph/`, comparing exact edge
    sequences (directed) or canonicalised edge multisets (undirected).
  - Benchmarks: `benches/bench_ring.rs` with three groups
    (undirected path, undirected cycle, directed-mutual cycle) across
    `n ∈ {100, 10_000, 1_000_000}`. Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-CN-001.json`.
  - Example: `examples/ring_demo.rs` walks through the common shapes
    and prints vcount / ecount / degree sequence / edge list.
  - 18 unit + proptest checks in `src/algorithms/constructors/ring.rs`
    covering the 4-flag truth table plus path/cycle wrapper agreement.
- **ALGO-GN-028** — `degree_sequence_game_edge_switching_simple` sampled
  *simple* graph for a prescribed degree sequence via a deterministic
  Havel–Hakimi (undirected) / Kleitman–Wang (directed) INDEX seed
  followed by `10 · |E|` degree-preserving edge-switching MCMC trials.
  Counterpart of the `IGRAPH_DEGSEQ_EDGE_SWITCHING_SIMPLE` branch of
  `igraph_degree_sequence_game()` in
  `references/igraph/src/games/degree_sequence.c`. **Fifth and final**
  of the 5-AWU split of `igraph_degree_sequence_game`
  (GN-024..028).
  - `pub fn degree_sequence_game_edge_switching_simple(out_degrees: &[u32], in_degrees: Option<&[u32]>, seed: u64) -> IgraphResult<Graph>`
    — `in_degrees = None` produces an undirected simple graph; `Some(in_seq)`
    produces a directed simple graph. Rejects non-graphical inputs up front
    via the same EG/FCA pre-check helpers shared from
    [ALGO-GN-026] / [ALGO-GN-027].
  - **Algorithm**: two-phase pipeline. (1) Build a deterministic seed
    graph that exactly realises the sequence using the INDEX variant of
    Havel–Hakimi (undirected) or Kleitman–Wang (directed) — hubs are
    processed by INDEX order with residual sort and stable tie-breaking
    by vertex index. (2) Run `10 · |E|` edge-switching MCMC trials:
    pick two distinct edges uniformly, undirected branch additionally
    flips the second edge's orientation with probability ½, then test
    the rewired pair `(a,b),(c,d) → (a,d),(c,b)` for no-op,
    self-loop, and multi-edge — apply iff all pass. Both successful
    *and* failed trials count toward the budget (detailed-balance
    requirement). Multi-edge detection uses `Vec<HashSet<u32>>`
    adjacency tables (undirected stores both directions; directed
    stores out-adjacency only).
  - **When to choose it**: best default for *dense* or *skewed* degree
    sequences. Cost is `O(n² + |E|)` independent of density, so it
    avoids the exponential restart cliff that hits
    [ALGO-GN-027]'s rejection sampler at `Σd/n ≳ 4`. Drops the
    connectivity guarantee of [ALGO-GN-025] (use that one if
    connectivity is required) and offers stronger MCMC mixing
    guarantees than the heuristic [ALGO-GN-026]. Completes the
    5-AWU split — `igraph_degree_sequence_game` is now fully ported.
  - Constants: `REWIRE_TRIALS_PER_EDGE: u64 = 10` (mirrors upstream
    `10 * igraph_ecount(graph)` chain length).
  - Conformance: 10 fixtures (C × 4, py × 3, R × 3) under
    `tests/conformance/{c,py,r}/degree_sequence_game_edge_switching_simple/`.
  - Benchmarks: `benches/bench_degree_sequence_edge_switching_simple.rs`
    with three regimes (size sweep on 3-regular, dense 5-regular n=100,
    directed balanced n=100 d=2). Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-GN-028.json`.
  - Example: `cargo run --example degree_sequence_edge_switching_simple_demo --release`.
- **ALGO-GN-027** — `degree_sequence_game_configuration_simple` uniformly
  sampled *simple* graph for a prescribed degree sequence via
  stub-matching with two-swap-per-edge incremental Fisher–Yates and
  restart-on-collision. Counterpart of the
  `IGRAPH_DEGSEQ_CONFIGURATION_SIMPLE` branch of
  `igraph_degree_sequence_game()` in
  `references/igraph/src/games/degree_sequence.c`.
  - `pub fn degree_sequence_game_configuration_simple(out_degrees: &[u32], in_degrees: Option<&[u32]>, seed: u64) -> IgraphResult<Graph>`
    — `in_degrees = None` produces an undirected simple graph; `Some(in_seq)`
    produces a directed simple graph. Rejects non-graphical inputs up front
    (Erdős–Gallai for the undirected branch, Fulkerson–Chen–Anstee for the
    directed branch; helpers shared with [ALGO-GN-026] via crate-local
    visibility).
  - **Algorithm**: build a flat stub bag of size `Σd`. Undirected: at edge
    index `i` pick two indices `k1 = RNG(2i, sc-1)` and
    `k2 = RNG(2i+1, sc-1)`, swap into positions `2i` and `2i+1`, take the
    pair as a candidate edge; reject and restart the whole attempt on a
    self-loop or multi-edge (tracked via `Vec<HashSet<u32>>` adjacency).
    Directed: shuffle the out-stub bag with one FY swap per edge, pair
    against in-stubs in construction order; multi-arc detection uses a
    bumped-counter trick (per-attempt mark in a `Vec<u64>` indexed by
    target vertex) for `O(1)` checks without per-target clear. Restart
    budget capped at `MAX_OUTER_ATTEMPTS = 1024`.
  - **Guarantees**: exact degree preservation, no self-loops, no multi-edges
    / multi-arcs, *uniform* distribution over the space of simple
    realisations. **NOT** guaranteed connected (use `_vl` for connectivity);
    expected number of attempts grows as `exp(O((Σd/n)²))` so the sampler
    is best suited to bounded-density sequences.
  - **Performance**: undirected 3-regular size sweep (medians, release
    profile) — `n = 100`: 14.5 µs, `n = 300`: 228 µs, `n = 600`: 116 µs
    (rejection-sampler variance: the n=300 seed lands on a long restart
    tail); moderate-skew `n = 100`: 13.0 µs; directed balanced `n = 100`,
    d=2: 11.4 µs. Snapshot: `.codefuse/tracking/perf/ALGO-GN-027.json`.
  - **Determinism**: single [`SplitMix64`] seed drives every FY swap and
    every restart. Not bitwise-portable to igraph C / NumPy / R, so the
    three-source conformance harness asserts structural invariants only
    (vcount, ecount = Σd/2 or Σout, exact out/in degree match, simplicity).
  - **Tests**: 16 unit tests + 3 proptest invariants
    (`degrees_preserved_undirected`, `simple_no_loops_no_multi_undirected`,
    `same_seed_same_graph`) + 1 doctest;
    `degree_sequence_game_configuration_simple_three_source_conformance`
    runs against 10 fixtures (4 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/degree_sequence_game_configuration_simple/`.
  - **Bench**: `benches/bench_degree_sequence_configuration_simple.rs`
    covers (a) 3-regular size sweep at `n ∈ {100, 300, 600}`,
    (b) moderate-skew `n = 100`, Σd = 258, and
    (c) directed balanced `n = 100`, d = 2. Example demo:
    `cargo run --example degree_sequence_configuration_simple_demo --release`.
- **ALGO-GN-026** — `degree_sequence_game_fast_heur_simple` fast-heuristic
  *simple* graph sampler for a prescribed degree sequence. Counterpart of
  the `IGRAPH_DEGSEQ_FAST_HEUR_SIMPLE` branch of
  `igraph_degree_sequence_game()` in
  `references/igraph/src/games/degree_sequence.c`.
  - `pub fn degree_sequence_game_fast_heur_simple(out_degrees: &[u32], in_degrees: Option<&[u32]>, seed: u64) -> IgraphResult<Graph>`
    — `in_degrees = None` produces an undirected simple graph; `Some(in_seq)`
    produces a directed simple graph. Rejects non-graphical inputs up front
    (Erdős–Gallai for the undirected branch, Fulkerson–Chen–Anstee for the
    directed branch).
  - **Algorithm**: build a stub bag of size `Σd`, Fisher–Yates shuffle once,
    then walk pairs left-to-right. For each candidate pair, reject if it
    is a self-loop or duplicates an existing edge (sorted-Vec adjacency
    `O(log Δ)` lookup); on rejection bump residual back and continue.
    On stuck (no further legal pair) restart from scratch, capped at
    `MAX_OUTER_ATTEMPTS = 1024`. No MCMC, no per-window connectivity
    recheck — this is the deliberate trade vs. `degree_sequence_game_vl`.
  - **Guarantees**: exact degree preservation, no self-loops, no multi-edges
    / multi-arcs. **NOT** guaranteed connected (use `_vl` for connectivity).
  - **Performance**: 21–54× faster than the VL sampler on the same fixtures.
    Baseline at `n = 1_200`, 3-regular undirected: 213 µs (fast-heur)
    vs. 11.5 ms (VL). Snapshot: `.codefuse/tracking/perf/ALGO-GN-026.json`.
  - **Determinism**: single [`SplitMix64`] seed drives every shuffle and
    every retry. Not bitwise-portable to igraph C / NumPy / R, so the
    three-source conformance harness asserts structural invariants only
    (vcount, ecount = Σd/2 or Σout, exact out/in degree match, simplicity).
  - **Tests**: 17 unit tests + 4 proptest invariants
    (`degrees_preserved_undirected`, `simple_no_loops_no_multi_undirected`,
    `degrees_preserved_directed`, `same_seed_same_graph`) + 1 doctest;
    `degree_sequence_game_fast_heur_simple_three_source_conformance` runs
    against 10 fixtures (4 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/degree_sequence_game_fast_heur_simple/`.
  - **Bench**: `benches/bench_degree_sequence_fast_heur.rs` covers
    (a) 3-regular size sweep at `n ∈ {200, 600, 1_200}`,
    (b) skewed power-law-tailed `n = 200`, Σd = 600, and
    (c) directed balanced `n = 200`, d = 4. Example demo:
    `cargo run --example degree_sequence_fast_heur_demo --release`.
- **ALGO-GN-025** — `degree_sequence_game_vl` Viger–Latapy uniform sampler
  for **simple connected** graphs with a prescribed degree sequence.
  Counterpart of the `IGRAPH_DEGSEQ_VL` branch of
  `igraph_degree_sequence_game()` in
  `references/igraph/src/games/degree_sequence_vl.c`.
  - `pub fn degree_sequence_game_vl(degrees: &[u32], seed: u64) -> IgraphResult<Graph>`
    — undirected only; rejects sequences that fail Erdős–Gallai or violate
    Hakimi's connected-graphical bound (Σd ≥ 2(n−1) when all degrees > 0).
  - **Algorithm**: (1) Hakimi-style greedy realisation seeded by the
    largest-degree-first heuristic, (2) `make_connected` 2-swap merging
    over the components produced by Step 1, (3) edge-switch MCMC with
    `total = 5 · 2 · |E|` proposals, with a windowed snapshot/restore
    rollback if the chain disconnects the graph mid-flight
    (window = `max(16, 2|E|)`).
  - **Guarantees**: exact degree preservation, no self-loops,
    no multi-edges, weak connectivity for every positive-degree component.
  - **Determinism**: a single [`SplitMix64`] seed drives every draw and
    every accept/reject decision; the PRNG is *not* portable to igraph C /
    NumPy / R, so the three-source conformance harness asserts structural
    invariants only (vcount, ecount = Σd/2, exact degree match, simplicity,
    weak connectivity).
  - **Tests**: 18 unit tests + 4 proptest invariants
    (`degree_sequence_preserved_when_graphical`, `simple_no_loops_or_multi`,
    `weakly_connected_when_positive_degrees`, `same_seed_same_graph`) + 1
    doctest; `degree_sequence_game_vl_three_source_conformance` runs
    against 8 fixtures (2 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/degree_sequence_game_vl/`.
  - **Bench**: `benches/bench_degree_sequence_vl.rs` measures (a) 3-regular
    size sweep at `n ∈ {200, 600, 1_200}` and (b) a skewed
    `[5,4,4,3,3,3,2,2,2,2]` tail. Baseline snapshot:
    `.codefuse/tracking/perf/ALGO-GN-025.json`.
- **ALGO-GN-024** — `degree_sequence_game_configuration` configuration-model
  random multigraph generator. Counterpart of the
  `IGRAPH_DEGSEQ_CONFIGURATION` branch of `igraph_degree_sequence_game()` in
  `references/igraph/src/games/degree_sequence.c` (function `configuration`,
  lines 37-123). Realises a prescribed degree sequence by classical
  stub matching (Bender–Canfield 1978 / Bollobás 1980):
  expand each vertex `i` into `d_i` half-edge stubs, then repeatedly draw
  two stubs uniformly at random and pair them into an edge. The result
  is a **multigraph** — self-loops and multi-edges are admitted by
  construction whenever the degree sequence permits them.
  - `pub fn degree_sequence_game_configuration(out_degrees: &[u32], in_degrees: Option<&[u32]>, seed: u64) -> IgraphResult<Graph>`
    — `in_degrees = None` produces an undirected graph (requires
    `Σ out_degrees` even); `in_degrees = Some(in_seq)` produces a
    directed graph (requires `Σ out = Σ in` and the two slices to be
    the same length).
  - **Stub-bag layout**: a single `Vec<VertexId>` of size `Σ d_i` for
    the undirected case, two bags (out + in) for the directed case.
    `Vec::swap_remove` gives `O(1)` random-access pop, so the pairing
    loop is `Θ(|E|)` with no inner allocation after the initial
    `Vec::with_capacity`.
  - **Complexity**: `Θ(n + Σ d_i)` total time and `Θ(Σ d_i)` peak
    memory; for `n = 100_000` at `d = 4` this is ~12.4 ms on Apple
    silicon (see `.codefuse/tracking/perf/ALGO-GN-024.json`).
  - **Determinism**: a single [`SplitMix64`] seed drives every draw, so
    `(out_degrees, in_degrees, seed)` always produces the same graph.
    The PRNG is *not* bitwise portable to igraph C / NumPy / R, so the
    three-source conformance harness asserts structural invariants only
    (vcount, ecount, exact degree match, directed flag).
  - **Tests**: 21 unit tests + 4 proptest invariants
    (`undirected_degree_match`, `directed_degree_match`,
    `deterministic_same_seed`, `odd_sum_rejected`) + 1 doctest;
    `degree_sequence_game_configuration_three_source_conformance` runs
    against 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/degree_sequence_game_configuration/`.
  - **Scope note**: this AWU lands the CONFIGURATION method only.
    Follow-up AWUs split out the remaining four methods of
    `igraph_degree_sequence_game()` as separate generators:
    `VL` (uniform sample of simple graphs, GN-025),
    `FAST_HEUR_SIMPLE` (GN-026),
    `CONFIGURATION_SIMPLE` (GN-027),
    `EDGE_SWITCHING_SIMPLE` (GN-028). Each variant has its own
    correctness model and benchmarking profile, so keeping them as
    independent AWUs preserves the 1-AWU-1-PR discipline.

- **ALGO-GN-023** — `correlated_game` + `correlated_pair_game` correlated
  Erdős–Rényi graph generators. Counterparts of `igraph_correlated_game()`
  and `igraph_correlated_pair_game()` in
  `references/igraph/src/games/correlated.c` (lines 90-338). Given a
  simple input graph on `n` vertices with target marginal density `p`,
  `correlated_game` returns a new simple graph whose adjacency vector has
  Pearson correlation `corr ∈ [0, 1]` with the input's. The construction
  solves the elementary 2×2 contingency table —
  `q = p + corr·(1−p)`, `p_del = 1−q` (drop an existing edge),
  `p_add = (1−q)·p/(1−p)` (create a non-edge) — so the marginal density
  is preserved exactly in expectation.
  - `pub fn correlated_game(old_graph: &Graph, corr: f64, p: f64, permutation: Option<&[VertexId]>, seed: u64) -> IgraphResult<Graph>`
    — relabels endpoints through `permutation` (interpreted as in the
    upstream C: `perm[i]` names the old vertex that becomes new vertex
    `i`, so we apply its inverse).
  - `pub fn correlated_pair_game(n: u32, corr: f64, p: f64, directed: bool, permutation: Option<&[VertexId]>, seed: u64) -> IgraphResult<(Graph, Graph)>`
    — convenience wrapper that samples a fresh `ER(n, p)` graph and a
    correlated counterpart from a single user seed.
  - **Sampling strategy**: the per-position Bernoullis are sampled via
    the same Batagelj–Brandes RNG_GEOM geometric-skip trick used in
    [`erdos_renyi_gnp`] — two independent geometric streams skip directly
    over kept-or-skipped edge slots without visiting every `O(n²)`
    position. A 3-way merge interleaves the kept edges, the deletes, and
    the additions in canonical-code order. Undirected codes use the
    upper-triangular `to·(to−1)/2 + from` encoding; directed codes use
    the diagonal-hole `D_CODE` encoding from upstream
    (`to·n + from`, with the `to == n−1` edges filed into the
    would-be diagonal hole of column `from`).
  - **Construction guarantees**: simple-by-construction (no self-loops,
    no parallel edges); `directed` flag of the output matches the input;
    deterministic in `seed`; `corr = 0` ⇒ independent ER(n, p) sample
    (delegated to `erdos_renyi_gnp` for the zero-corr fast path);
    `corr = 1` ⇒ exact copy of the old graph (`p_del = p_add = 0`, no
    RNG draws consumed).
  - **Validation**: `corr ∈ [0, 1]`; `p ∈ (0, 1)`; `permutation` must be
    a permutation of `[0, n)` (each vertex appears exactly once);
    `old_graph` must be simple. All four error paths covered.
  - **Edge cases**: `n = 0` empty input; `n = 1` singleton (no candidate
    pairs); `directed` input preserved through the output.
  - **Coverage**: 20 unit tests + 5 proptests + 2 doctests covering
    code/decode round-trip / bijection / determinism / seed-divergence /
    `corr = 0` and `corr = 1` special cases / directed handling /
    permutation correctness / simple-by-construction / `correlated_pair_game`
    correlation baseline / all four validation error paths.
  - **Three-source conformance**: 12 JSON fixtures under
    `tests/conformance/{c,py,r}/{correlated_game,correlated_pair_game}/`
    — `corr = 1` exact-copy on a `P4` path and a `C5` cycle (the latter
    with reverse permutation) for `correlated_game`; `n = 30` undirected
    and `n = 20` directed `correlated_pair_game` with 6σ Binomial bands
    on both returned graphs. RNG state is not portable across
    implementations (MT in C, R RNGkind, NumPy in py), so conformance
    asserts structural invariants only (vcount, directed flag, ecount
    band, no self-loops, `is_simple`).
  - **Bench + example**: Criterion bench `benches/bench_correlated.rs`
    sweeps `corr ∈ {0, 0.25, 0.5, 0.75, 1.0}` at `n = 800`, vertex count
    `n ∈ {200, 800, 3 200}` at `corr = 0.5`, and directed vs undirected
    at `n = 800`. The corr sweep traces the expected U-shape: `corr = 0`
    (2.67 ms) ⇒ peaks at `corr = 0.25` (3.64 ms, mixed delete + add
    activity) ⇒ `corr = 1` minimum (2.20 ms, zero RNG draws). Runnable
    example `examples/correlated_pair_demo.rs` samples a `(g1, g2)` pair
    at six `corr` values on `n = 200, p = 0.1`, reporting ecounts,
    intersection size, Jaccard overlap, and the empirical Pearson
    correlation — the empirical ρ̂ tracks the requested `corr` to within
    a few percent, and at `corr = 1.0` both graphs have exactly the same
    edge set (Jaccard = 1.0).

- **ALGO-GN-022** — `dot_product_game` random dot-product graph
  generator. Counterpart of `igraph_dot_product_game()` in
  `references/igraph/src/games/dotproduct.c` (lines 59-102). For each
  unordered (or ordered, when `directed=true`) pair of distinct vertices
  `(i, j)`, computes `prob = <v_i, v_j>` and adds the edge with that
  probability under three regimes matching the C kernel exactly:
  `prob > 1` → unconditional edge + `had_over_one` warning flag (no RNG
  draw consumed); `prob < 0` → skip + `had_negative` warning flag (also
  no draw); otherwise Bernoulli via `rng.gen_unit() < prob` (strict `<`,
  so `prob = 0` never fires).
  - `pub fn dot_product_game(vecs: &[Vec<f64>], directed: bool, seed: u64) -> IgraphResult<Graph>`
    — convenience wrapper that discards the warnings struct.
  - `pub fn dot_product_game_with_warnings(vecs: &[Vec<f64>], directed: bool, seed: u64) -> IgraphResult<(Graph, DotProductWarnings)>`
    — returns `DotProductWarnings { had_negative, had_over_one }` so
    callers can detect when the latent vectors push the inner product
    outside `[0, 1]`.
  - **Construction guarantees**: NEVER produces self-loops (undirected
    loop starts at `j = i + 1`; directed loop short-circuits on `i == j`),
    ALWAYS simple (each pair inspected exactly once), deterministic in
    `seed`.
  - **Validation**: all latent vectors must share the same dimension `d`;
    `d` may be `0` (every dot product is `0.0`, every pair is skipped);
    every component must be finite.
  - **Edge cases**: `n = 0` empty graph; `n = 1` singleton; `d = 0` →
    edge-free graph with `had_negative = had_over_one = false`; all-ones
    vectors at `d = 1` → complete graph with `had_over_one = true`.
  - **Coverage**: 14 unit tests + 5 proptests + 2 doctests covering
    exact-vcount / no-self-loops / determinism / seed-divergence /
    directed-vs-undirected counts / `d = 0` and `n = 0` boundaries /
    warning-flag triggers / validation error paths / structural
    completeness on all-ones inputs.
  - **Three-source conformance**: 9 JSON fixtures under
    `tests/conformance/{c,py,r}/dot_product_game/` — all-ones complete
    `n = 8` undirected, orthogonal blocks `n = 8` undirected, mixed-clamp
    `n = 10` directed. RNG state is not portable across implementations
    (Mersenne Twister in C, `R_unif_index` in R, NumPy in py), so
    conformance asserts structural invariants only (vcount, directed
    flag, ecount band, no self-loops).
  - **Bench + example**: Criterion bench `benches/bench_dot_product.rs`
    sweeps latent dimension `d ∈ {1, 4, 16, 64}` at `n = 400`, vertex
    count `n ∈ {100, 400, 1 600}` at `d = 8`, and directed vs undirected
    at `n = 400, d = 8`. Runnable example `examples/dot_product_demo.rs`
    plants three communities by concentrating each cohort's latent
    vector near a different basis vector — intra-community connection
    probability lands at ~0.747 (theory 0.743), inter-community at
    ~0.178 (theory 0.180), a ~4.2× contrast. Perf snapshot at
    `.codefuse/tracking/perf/ALGO-GN-022.json` shows ~2.06× directed /
    undirected ratio matching the `n(n-1)` vs `n(n-1)/2` pair count.

- **ALGO-GN-021** — `barabasi_aging_game` Barabási–Albert
  preferential-attachment with vertex aging. Counterpart of
  `igraph_barabasi_aging_game()` in
  `references/igraph/src/games/barabasi.c` (lines ~606-841). Each step
  `i ≥ 1` adds a fresh vertex and attaches `m` (or `outseq[i]`)
  outgoing edges, with targets drawn from a Fenwick BIT (`PsumTree`)
  weighted by a product of a degree term and an age term:
  `weight(v) = (deg_coef · pow(deg(v), pa_exp) + zero_deg_appeal)
  · (age_coef · pow(age(v), aging_exp) + zero_age_appeal)`.
  `age(v) = (i − v)/binwidth + 1` with `binwidth = nodes/aging_bins + 1`;
  the C kernel's `pow(0, 0) == 1` special case is preserved on the
  degree term only.
  - `pub fn barabasi_aging_game(nodes: u32, m: u32, outseq: Option<&[u32]>, outpref: bool, pa_exp: f64, aging_exp: f64, aging_bins: u32, zero_deg_appeal: f64, zero_age_appeal: f64, deg_coef: f64, age_coef: f64, directed: bool, seed: u64) -> IgraphResult<Graph>`.
  - **Mechanics**: each step does `m` draws against the live BIT
    *without* zeroing picks between draws (matching upstream — within-step
    multi-edges are possible when `m ≥ 2`); chosen vertices have their
    weights refreshed at end-of-step with the new degree; the new vertex
    `i` is inserted into the BIT at age `1`; and an age sweep at every
    `k · binwidth ≤ i` boundary refreshes the vertex at position
    `i − k · binwidth` with the new age `k + 2`.
  - **Construction guarantees**: NEVER produces self-loops (the new
    vertex `i` is added to the BIT *after* its `m` outgoing draws and
    `search_bounded(target, i)` rules out FP-drift over-advancement);
    zero-sum uniform fallback over `[0, i)` (only fires when the seed
    vertex's weight has decayed to zero); `outpref` feeds the new
    vertex's own out-degree back into its weight at end-of-step.
  - **Validation**: `pa_exp` / `aging_exp` finite; `aging_bins ≥ 1`;
    `deg_coef` / `age_coef` / `zero_deg_appeal` / `zero_age_appeal`
    finite and non-negative; `outseq.len() == nodes` when provided.
  - **Edge cases**: `nodes = 0` empty graph; `nodes = 1` singleton;
    `m = 0` (no `outseq`) returns a vertex-only graph; without `outseq`,
    ecount = `(nodes − 1) · m` exactly (no saturation branch — the C
    kernel writes one edge per attempted draw regardless of within-step
    collisions).
  - **Coverage**: 23 unit tests + 5 proptests covering exact-vcount /
    exact-ecount / determinism / seed-divergence / no-self-loops /
    source-is-step-index / directed-and-undirected propagation / outpref
    behaviour / age-sweep correctness / `pow(0, 0)` branch / validation
    error paths / aging directional comparison (sharp aging shifts
    in-degree mass to the young half vs. classical baseline).
  - **Three-source conformance**: 9 JSON fixtures under
    `tests/conformance/{c,py,r}/barabasi_aging_game/` — classical
    (`aging_exp=0` degenerate), aging-heavy (`aging_exp=-1`), and
    outpref-undirected cases per source. RNG state is not portable
    across implementations (Mersenne Twister in C, `R_unif_index` in R,
    NumPy in py), so conformance asserts structural invariants only
    (vcount, directed flag, exact ecount, no self-loops).
  - **Bench + example**: Criterion bench
    `benches/bench_barabasi_aging.rs` covering classical (no aging),
    aging-heavy, and outpref-undirected cases at `n ∈ {100, 1k, 10k}`.
    Runnable example `examples/barabasi_aging_demo.rs` contrasts a
    classical baseline (`aging_exp = 0`) with a sharp-aging run
    (`aging_exp = -1, zero_age_appeal = 0`) and reports the young-half
    in-degree share — sharp aging lifts it from ~5% to ~21% at
    `n = 2 000`. Perf snapshot at
    `.codefuse/tracking/perf/ALGO-GN-021.json` shows ~2.8 Melem/s
    (classical, n=10k), ~2.5 Melem/s (aging-heavy, n=10k),
    ~2.5 Melem/s (outpref-undirected, n=10k) — consistent with the
    predicted `O((n + n/aging_bins) · log n + |E|)` bound.

- **ALGO-GN-020** — `barabasi_game_psumtree` +
  `barabasi_game_psumtree_multiple` Barabási–Albert preferential-attachment
  variants. Counterparts of `igraph_i_barabasi_game_psumtree` and
  `igraph_i_barabasi_game_psumtree_multiple` in
  `references/igraph/src/games/barabasi.c` (lines ~166-414). Both variants
  attach `m` (or `outseq[i]`) outgoing edges from every new vertex `i`,
  with destinations weighted by `attraction(deg) = pow(deg, power) + A`
  (preserving C's `pow(0, 0) == 1` semantics). Sampling shares the inline
  Fenwick BIT (`PsumTree`) used by `lastcit_game` /
  `recent_degree_game` and uses a new `search_bounded(target, bound)`
  routine that clamps the binary-lifting result to `[0, bound)`.
  - `pub fn barabasi_game_psumtree(nodes: u32, power: f64, m: u32, outseq: Option<&[u32]>, outpref: bool, a: f64, directed: bool, seed: u64) -> IgraphResult<Graph>`
    — **simple** variant: each of the `m` draws temporarily zeros the
    chosen target's weight in the BIT so the same target cannot be drawn
    twice within one step; weights are refreshed at end-of-step.
    Cannot produce within-step multi-edges; source `i` is fresh each step
    so cross-step duplicates are impossible too — output is simple.
  - `pub fn barabasi_game_psumtree_multiple(nodes, power, m, outseq, outpref, a, directed, seed) -> IgraphResult<Graph>`
    — **multiple** variant: the BIT sum is snapshotted once per step and
    all `m` draws sample against the unchanged tree, so the same target
    may be picked twice within one step (multi-edges allowed); explicit
    `m >= i` always-cite saturation branch emits `i` edges (one per
    previously-added vertex) when there are fewer prior vertices than
    requested edges.
  - Construction guarantees: NEVER produces self-loops (the new vertex
    `i` is added to the BIT *after* its `m` outgoing draws complete, and
    `search_bounded(target, i)` rules out the FP-drift over-advancement
    that could otherwise return `i` itself); zero-sum fallback uniform
    over `[0, i)` (only fires in the very first step when
    `outpref = true, A > 0` because the seed vertex's weight is set to
    `1.0`); `outpref` (forced `true` when `directed = false`) feeds the
    new vertex's own out-degree back into its attraction at end-of-step.
  - Validation: `nodes` finite; `power` finite and non-NaN; `m > 0` when
    `outseq = None`; `outseq.len() == nodes` when provided; `a > 0`
    (strict) when `outpref = false` so zero-degree vertices have non-zero
    probability; `a >= 0` (non-strict) when `outpref = true`.
  - Edge cases: `nodes = 0` empty graph; `nodes < 2` vertex-only graph;
    edge count formulas — simple: `(n-1) · m`; multiple:
    `(n-1) · m - m · (m-1) / 2` when `n > m` (early-cite saturation).
  - Coverage: unit tests for vcount/ecount exactness (both variants,
    constant-`m` and `outseq`), determinism, seed divergence,
    no-self-loops invariant, source-is-always-step-index invariant,
    directed/undirected flag propagation, `outpref` behaviour, `pow(0,0)`
    branch, validation error paths; 5 proptests covering ecount,
    self-loop-freedom, source invariant, and parameter robustness with
    seed = (n, m, power, seed, outpref).
  - Three-source conformance: 9 JSON fixtures under
    `tests/conformance/{c,py,r}/barabasi_game_psumtree/` — three classic /
    multiple-pow15 / undirected-outpref cases per source. RNG state
    (Mersenne-Twister in C, `R_unif_index` in R, NumPy in py) is not
    portable to our SplitMix64, so conformance asserts structural
    invariants only (vcount, directed flag, ecount band,
    no-self-loops).
  - Criterion bench `benches/bench_barabasi_psumtree.rs` covering simple
    directed classical, simple undirected `pow=1.5`, and multiple
    directed `m=3` at `n ∈ {100, 1k, 10k}`. Runnable example
    `examples/barabasi_psumtree_demo.rs` contrasts linear vs sub-linear
    BA degree-distribution tails. Perf snapshot at
    `.codefuse/tracking/perf/ALGO-GN-020.json` shows ~3.6 Melem/s
    (simple, n=10k), ~4.7 Melem/s (undirected pow=1.5, n=10k),
    ~2.4 Melem/s (multiple m=3, n=10k) — consistent with the predicted
    `O(n · m · log n)` slope.

- **ALGO-GN-019** — `recent_degree_game` sliding-window preferential
  attachment. Counterpart of `igraph_recent_degree_game()` in
  `references/igraph/src/games/recent_degree.c:24-200`. Models a growing
  graph where each new vertex `i ≥ 1` emits `m` (or `outseq[i]`)
  outgoing edges, with each target drawn proportionally to
  `pow(recent_in_degree, power) + zero_appeal`. The "recent" in-degree
  only counts edges added within the last `time_window` steps; older
  edges are expired from a BIT-tree (Fenwick) weight store via a
  `VecDeque<Option<u32>>` history queue (per-step `None` sentinels).
  - `pub fn recent_degree_game(nodes: u32, power: f64, time_window: u32, m: u32, outseq: Option<&[u32]>, outpref: bool, zero_appeal: f64, directed: bool, seed: u64) -> IgraphResult<Graph>`.
  - Sampling uses an inline Fenwick BIT (`PsumTree`) with
    binary-lifting prefix-search — `O(log n)` for both `set` and
    `search`. Per-step cost is `O(m · log n)` for draws + amortized
    `O(m)` for window expiry, giving a total `O(n · m · log n)` runtime;
    batched `add_edges` keeps the edge-insert phase out of the
    `add_edge` / rebuild_indexes `O(m²)` trap.
  - Construction guarantees: NEVER produces self-loops (the psumtree
    only ranges over previously-added vertices, the new vertex is
    seeded *after* its outgoing edges are drawn); multi-edges are
    allowed when `m ≥ 2` (independent draws can collide); the
    cumulative weight only falls to zero during the warm-up phase
    (or when `time_window = 0` and `zero_appeal = 0`), in which case
    the algorithm falls back to a uniform draw over `[0, i)`.
  - `outpref` toggles whether the source vertex's own emitted citations
    also feed back into its recent in-degree (matching igraph's natural
    choice for the undirected variant).
  - Validation: `power` finite and non-NaN; `zero_appeal` finite,
    non-NaN, and non-negative; `outseq.len() == nodes` when provided.
  - Edge cases: `nodes = 0` returns an empty graph; `nodes < 2` or
    `m = 0` returns the vertex-only graph; `time_window = 0` collapses
    the BIT-tree to zero_appeal-only weights ⇒ uniform draws (or full
    fallback when `zero_appeal = 0` too).
  - Coverage: 22 unit tests (vcount/ecount exactness for both
    constant-`m` and `outseq` variants, determinism, seed divergence,
    NEVER-self-loops with positive `zero_appeal`,
    `source_is_always_step_index` and `target_in_zero_to_step_minus_one`
    invariants, directed/undirected flag propagation,
    `outpref_changes_graph`, `time_window_zero_uses_zero_appeal_only`
    degeneration, `large_time_window_concentrates_on_early_vertices`
    sanity check, 5 validation error paths, 3 `weight_from_degree`
    edge-case unit tests covering the `pow(0,0)=1` branch and the
    positive-power-on-zero-degree branch) + 5 proptests
    (`ecount_exact_constant_m`, `no_self_loops_when_zero_appeal_positive`,
    `source_is_always_step_index`, `target_in_zero_to_step_minus_one`,
    `determinism_under_proptest`) under `--features proptest-harness` +
    9 three-source conformance fixtures (3 each from C / py / R) under
    `tests/conformance/{c,py,r}/recent_degree_game/` asserting
    structural invariants only (vcount exact, directed flag exact,
    ecount exact = `(n−1)·m`, optional `no_self_loops`). RNG state is
    not portable across SplitMix64 vs igraph's GLIBC RNG, so we do not
    assert per-edge endpoint equality and do not assert `is_simple`
    (recent_degree allows multi-edges when `m ≥ 2` by construction).
  - Bench: criterion baseline at
    `.codefuse/tracking/perf/ALGO-GN-019.json` covering four groups —
    size_scaling (139µs @ n=500, 2.02ms @ n=5000 at m=3, window=10),
    m_count (1/4/16 at n=1000, power=1.5, window=20 — confirms linear
    scaling in `m`), window_count (2/20/200 at n=1000 — shows mild
    ~10% slowdown across two orders of magnitude of window size,
    confirming O(log n) per-op cost amortizes), and an undirected
    outpref-enabled case. Slightly slower than GN-018 lastcit
    (~2.02ms vs ~1.65ms at n=5000) because the per-step VecDeque-pop
    + psumtree refresh runs unconditionally whereas lastcit's age
    sweep only fires at bin boundaries.
  - Example: `cargo run --example recent_degree_demo` builds a
    2000-vertex directed graph with `m = 3`, `power = 1`,
    `time_window = 25`, `zero_appeal = 1` and prints in-degree
    quartiles, uncited count, and top-10 vertices by in-degree with
    cohort labels demonstrating that the highest in-degrees skew
    toward the *earliest* cohorts (rich-get-richer amplifies through
    each window refresh — once a vertex is hot it keeps being cited,
    refreshing its window timer).
- **ALGO-GN-018** — `lastcit_game` recency-decay citation network.
  Counterpart of `igraph_lastcit_game()` in
  `references/igraph/src/games/citations.c:28-92`. Models a growing
  directed citation graph where each new vertex `i ≥ 1` emits
  `edges_per_node` outgoing citations weighted by how recently the
  target was last cited: `bucket = min(⌊(i − lastcit[v]) / binwidth⌋,
  agebins − 1)` for already-cited vertices (with
  `binwidth = nodes / agebins + 1`), and `bucket = agebins` for the
  "never-cited" pool. Citation events refresh the target back to
  `preference[0]`; a per-step age-sweep demotes vertices that just
  crossed a bin boundary.
  - `pub fn lastcit_game(nodes: u32, edges_per_node: u32, agebins: u32, preference: &[f64], directed: bool, seed: u64) -> IgraphResult<Graph>`.
  - Sampling uses an inline Fenwick BIT (`PsumTree`) with
    binary-lifting prefix-search — `O(log n)` for both `set` and
    `search`. Overall step cost is
    `O((edges_per_node + agebins) · log n)`, giving a total
    `O(n · (eps + agebins) · log n)` runtime; batched `add_edges`
    keeps the edge-insert phase out of the `add_edge` / rebuild_indexes
    `O(m²)` trap.
  - Construction guarantees: NEVER produces self-loops (the psumtree
    only ranges over previously-added vertices, the new vertex is
    seeded as never-cited *after* its citations are drawn);
    multi-edges are allowed when `edges_per_node ≥ 2` (independent
    draws can collide); the cumulative preference can only fall to
    zero on the trivial degenerate input `preference = [0; agebins+1]`
    where the model produces zero edges.
  - Validation: `preference.len() == agebins + 1`; `agebins ≥ 1`;
    every `preference[i]` finite, non-NaN, and non-negative.
  - Edge cases: `nodes = 0` returns an empty graph; `nodes < 2` or
    `edges_per_node = 0` returns the vertex-only graph;
    all-zero `preference` returns the vertex-only graph (no edges).
  - Coverage: 19 unit tests (vcount/ecount exactness, determinism,
    seed divergence, all-zero-pref empty result, NEVER-self-loops on
    non-trivial preferences, directed/undirected flag propagation,
    every validation error path, uniform vs steep-decay sanity
    checks, age-sweep correctness, single-agebin degenerate case) +
    5 proptests (`ecount_exact_when_pref_positive`,
    `no_self_loops_always`, `source_is_always_step_index`,
    `target_in_zero_to_step_minus_one`, `determinism_under_proptest`)
    under `--features proptest-harness` + 9 three-source conformance
    fixtures (3 each from C / py / R) under
    `tests/conformance/{c,py,r}/lastcit_game/` asserting structural
    invariants only (vcount exact, directed flag exact, ecount exact
    = `(n−1)·eps`, optional `no_self_loops`). RNG state is not
    portable across SplitMix64 vs igraph's GLIBC RNG, so we do not
    assert per-edge endpoint equality and do not assert `is_simple`
    (lastcit allows multi-edges when `eps ≥ 2` by construction).
  - Bench: criterion baseline at
    `.codefuse/tracking/perf/ALGO-GN-018.json` covering four groups —
    size_scaling (95µs @ n=500, 1.65ms @ n=5000 at eps=3, agebins=4),
    eps_count (1/4/16 at n=1000), agebins_count (1/4/16 at n=1000,
    eps=2 — the new dimension vs cited_type), and an undirected
    uniform-preference case. The psumtree O(log n) drives the
    expected ~3.6× speedup over GN-017 at n=5000 despite the extra
    per-step age-sweep work.
  - Example: `cargo run --example lastcit_demo` builds a 2000-vertex
    directed citation graph with `edges_per_node = 3`, `agebins = 4`,
    `preference = [8, 4, 2, 1, 0.5]` and prints in-degree quartiles
    (mean ≈ 3.0, max ≈ 80), uncited count (~66% of vertices), and
    the top-10 vertices by in-degree with cohort labels demonstrating
    that the highest in-degrees skew toward the oldest cohorts —
    because each citation refreshes a vertex to bucket 0 and old
    vertices have had the most opportunities to accumulate refreshes.
- **ALGO-GN-017** — `cited_type_game` cited-type / type-weighted
  growing citation network. Counterpart of `igraph_cited_type_game()`
  in `references/igraph/src/games/citations.c:246-335`. Models a
  growing citation network where vertex types are **pre-assigned by
  the caller** (not sampled internally — the key contrast vs
  `establishment_game` and `callaway_traits_game`): for each new
  vertex `i ∈ [1, nodes)`, draw `edges_per_step` outgoing citations
  independently, each targeting a previously-added vertex `v` with
  probability proportional to `pref[type[v]]`. Sampling uses an
  incrementally-grown cumulative-sum vector and `partition_point`
  inverse-transform — `O((n−1)·eps·log n)` overall, with a single
  batched `add_edges` to avoid the `add_edge`/`rebuild_indexes`
  `O(m²)` trap. Multi-edges are allowed when `edges_per_step ≥ 2`
  (independent draws can collide); self-loops only appear via the
  `sum = 0` fallback path (when every assigned type has zero
  attractivity).
  - `pub fn cited_type_game(nodes: u32, types: &[u32], pref: &[f64], edges_per_step: u32, directed: bool, seed: u64) -> IgraphResult<Graph>`.
  - Validation: `types.len() == nodes`; `pref.len() ≥ max(types) + 1`
    (with overflow check); every `pref[i]` finite, non-NaN, and
    non-negative.
  - Edge cases: `nodes = 0` returns an empty graph; `nodes < 2` or
    `edges_per_step = 0` returns the vertex-only graph; when
    `pref` is identically zero (or every assigned type has
    `pref = 0` so far), each step falls back to emitting a self-loop
    on the step vertex — matching the C reference behaviour.
  - Coverage: 18 unit tests (vcount/ecount exactness when pref > 0,
    determinism, seed divergence, all-zero-pref self-loop fallback,
    positive-pref no-self-loops, directed/undirected flag
    propagation, all 6 validation error paths, concentration on
    non-zero-pref types, heavy-skew concentration) + 5 proptests
    (`ecount_exact_when_pref_positive`,
    `no_self_loops_when_pref_positive`,
    `source_is_always_step_index`,
    `target_in_zero_to_step_minus_one`,
    `determinism_under_proptest`) under `--features proptest-harness`
    + 9 three-source conformance fixtures (3 each from C / py / R)
    under `tests/conformance/{c,py,r}/cited_type_game/` asserting
    structural invariants only — RNG state is not portable across
    SplitMix64 vs igraph's GLIBC RNG, so we assert `vcount = nodes`
    (exact), `directed` flag (exact), `ecount` exact = `(n−1)·eps`
    when `nodes ≥ 2` and `eps > 0`, `max(types)` bound, and the
    optional `no_self_loops` (when all pref > 0) / `all_self_loops`
    (under the sum=0 fallback) flags. Notably we do NOT assert
    `is_simple` — multi-edges are part of the model.
  - Bench at `benches/bench_cited_type.rs`: a
    `size_scaling/eps3_uniform` sweep at fixed `types = 4,
    edges_per_step = 3` with uniform pref (`n ∈ {500, 5_000}`), an
    `eps_count/n1000_skewed` sweep at `n = 1_000, types = 3,
    pref = [10.0, 1.0, 0.05]` over `edges_per_step ∈ {1, 4, 16}`,
    and an `undirected/n1000_uniform` point at `n = 1_000, types = 2,
    edges_per_step = 4`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-017.json`: the size-scaling axis
    holds near 6-8 Melem/s (60 µs / 806 µs for n ∈ {500, 5_000}),
    the eps-sweep scales linearly in `eps` (36 / 175 / 826 µs for
    `eps ∈ {1, 4, 16}` at `n = 1_000`) confirming the
    `(n − 1) · eps` candidate-edge bound, the undirected variant
    matches the directed `eps = 4` baseline at 174 µs (the
    `directed` flag only affects canonicalisation in `Graph`
    storage, not the citation logic). Crucial perf win during
    implementation: switching from per-edge `add_edge` (which rebuilds
    the edge index `O(m)` times → `O(m²)` overall, ~4.6 s for
    `n = 5_000` `eps = 3`) to a single batched `add_edges` yields a
    5_700× speed-up.
  - Example: `examples/cited_type_demo.rs` builds a 2 000-vertex
    directed graph with round-robin `types = 3,
    edges_per_step = 3`, sharply skewed `pref = [10.0, 1.0, 0.05]`;
    prints exact ecount, per-type vertex counts, self-loop and
    multi-bundle counts, and per-type in-degree share. The observed
    in-degree share (90.1% / 9.6% / 0.4%) tracks the pref-implied
    target (90.5% / 9.0% / 0.5%) closely — empirical confirmation
    that the cumulative-sum binsearch implements the type-weighted
    distribution correctly.
  - Re-exported as `rust_igraph::cited_type_game`.

- **ALGO-GN-016** — `callaway_traits_game` Callaway et al. (2001)
  growing-traits random graph generator. Counterpart of
  `igraph_callaway_traits_game()` in
  `references/igraph/src/games/citations.c:95-156`. Differs from
  `establishment_game` in two structural ways: (1) all `n` vertex
  types are categorical-sampled up front (uniform when
  `type_dist = None`), not just for `i ≥ k`; (2) on each step
  `i ∈ [1, n)` BOTH endpoints of each candidate edge are drawn
  uniformly from the existing population `[0, i]` *inclusive* — and
  the candidate is accepted with `pref_matrix[t_a][t_b]` —
  `edges_per_step` independent attempts per step. As a consequence
  self-loops and multi-edges ARE allowed by construction (the C model
  draws both endpoints with `RNG_INTEGER(0, i)` independently).
  - `pub fn callaway_traits_game(nodes: u32, types: u32, edges_per_step: u32, type_dist: Option<&[f64]>, pref_matrix: &[Vec<f64>], directed: bool, seed: u64) -> IgraphResult<(Graph, Vec<u32>)>`.
  - Validation: `types ≥ 1`; `pref_matrix` is `types × types`,
    finite, non-NaN, entries in `[0, 1]`; when `directed = false`,
    `pref_matrix` must be symmetric; `type_dist` (when set) length
    matches `types`, entries finite and non-negative.
  - Edge cases: `nodes = 0` returns an empty `(Graph, vec![])`;
    `nodes = 1` returns a single-vertex edgeless graph (the loop body
    runs only for `i ≥ 1`); `edges_per_step = 0` returns an edgeless
    graph regardless of `pref_matrix`. Per-vertex types are still
    assigned in every case so the returned `Vec<u32>` always has
    length `nodes`.
  - Coverage: 25 unit tests + 5 proptests
    (`ecount_bounded_by_full_accept`, `types_in_range`,
    `determinism`, `p1_full_pref_yields_exact_max_ecount`,
    `p0_yields_no_edges`) under `--features proptest-harness` + 9
    three-source conformance fixtures (3 each from C / py / R) under
    `tests/conformance/{c,py,r}/callaway_traits_game/` asserting
    structural invariants only — RNG state is not portable across
    SplitMix64 vs igraph's GLIBC RNG, so we assert `vcount = nodes`
    (exact), `directed` flag (exact), `ecount` band (hand-derived
    from `(n-1)·eps · p_avg` plus tolerance), `max_type < types`, and
    (where applicable) the `diagonal_only_pref` / `cross_only_pref`
    flags. Notably we do NOT assert `is_simple` — that contrasts with
    `establishment_game` and is the model's defining property.
  - Bench at `benches/bench_callaway_traits.rs`: a
    `size_scaling/eps3_diag` sweep at fixed `types = 4,
    edges_per_step = 3` with a diagonal `p = 0.20` pref matrix
    (`n ∈ {500, 5_000}`), an `eps_count/n1000_full` sweep at
    `n = 1_000, types = 2, p = 1.0` over
    `edges_per_step ∈ {1, 4, 16}`, and a `directed/n1000_3types`
    point with an asymmetric `3 × 3` pref matrix at `n = 1_000,
    edges_per_step = 3`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-016.json`: the size-scaling axis
    sits near 24-40 Melem/s (linear in `n · eps`); the eps-sweep
    scales roughly linearly in `eps` (44.5 / 177.5 / 833.3 µs for
    `eps ∈ {1, 4, 16}` at `n = 1_000`), confirming the bound
    `(n - 1) · eps` total candidate edges; the directed asymmetric
    variant holds 28.3 Melem/s.
  - Example: `examples/callaway_traits_demo.rs` builds a 2 000-vertex
    undirected graph with `types = 3, edges_per_step = 4`, an
    assortative pref matrix (0.30 within / 0.02 across), and a skewed
    `type_dist = [0.50, 0.25, 0.25]`; prints per-type vertex counts,
    the within-vs-cross-type edge split, the per-type mean degree,
    AND the count of self-loops + edges in multi-bundles — the
    planted assortative structure shows up as > 90% within-type
    edges, while the loop / multi counters make the
    not-simple-by-construction property concrete versus
    `establishment_game`.
  - Re-exported as `rust_igraph::callaway_traits_game`.

- **ALGO-GN-015** — `establishment_game` Caldarelli et al. (2002)
  sample-traits growing random graph generator. Counterpart of
  `igraph_establishment_game()` in
  `references/igraph/src/games/citations.c:159-274`. The generator
  starts from `k` empty seed vertices; on each subsequent step a fresh
  vertex `i` is assigned a categorical type from `type_dist` (uniform
  when `None`), Floyd-distinct samples `k` previous vertices, and emits
  each candidate edge `(i, j)` independently with probability
  `pref_matrix[type[i]][type[j]]`. Output is always simple by
  construction — `floyd_distinct_sample` guarantees the `k` neighbours
  are distinct, and the candidate set is always strictly previous
  vertices, so no self-loops or multi-edges can ever appear.
  - `pub fn establishment_game(nodes: u32, types: u32, k: u32, type_dist: Option<&[f64]>, pref_matrix: &[Vec<f64>], directed: bool, seed: u64) -> IgraphResult<(Graph, Vec<u32>)>`.
  - Validation: `types ≥ 1`; `pref_matrix` is `types × types`,
    finite, non-NaN, entries in `[0, 1]`; when `directed = false`,
    `pref_matrix` must be symmetric; `type_dist` (when set) length
    matches `types`, entries finite and non-negative.
  - Edge cases: `nodes = 0` returns an empty `(Graph, vec![])`;
    `nodes ≤ k` returns a `nodes`-vertex edgeless graph (the loop body
    runs only for `i ≥ k`); `k = 0` returns an edgeless graph
    regardless of `pref_matrix`. Per-vertex types are still assigned in
    every case so the returned `Vec<u32>` always has length `nodes`.
  - Coverage: 21 unit tests + 5 proptests (`types_in_range`,
    `ecount_band`, `deterministic`, `diagonal_pref_stays_within_types`,
    `cross_only_pref_yields_cross_edges`) under
    `--features proptest-harness` + 9 three-source conformance
    fixtures (3 each from C / py / R) under
    `tests/conformance/{c,py,r}/establishment_game/` asserting
    structural invariants only — RNG state is not portable across
    SplitMix64 vs igraph's GLIBC RNG, so we assert `vcount = nodes`
    (exact), `directed` flag (exact), `ecount` band (hand-derived
    from `Σ pair-cells × p_avg` plus tolerance), `is_simple` via
    canonical-pair `HashSet`, `max_type < types`, and (where
    applicable) the `diagonal_only_pref` / `cross_only_pref` flags
    that pin the edge-coloring shape.
  - Bench at `benches/bench_establishment.rs`: a
    `size_scaling/k4_diag` sweep at fixed `types = 4, k = 4` with a
    diagonal `p = 0.20` pref matrix (`n ∈ {500, 5_000}`), a
    `k_count/n1000_full` sweep at `n = 1_000, types = 2, p = 1.0`
    over `k ∈ {1, 4, 16}`, and a `directed/n1000_3types` point with
    an asymmetric `3 × 3` pref matrix at `n = 1_000, k = 4`.
    Baseline at `.codefuse/tracking/perf/ALGO-GN-015.json`: the
    size-scaling axis sits near 7.3 Melem/s (effectively linear in
    `n`); the k-sweep degrades roughly linearly in `k` (11.5 → 4.6 →
    1.27 Melem/s for `k = 1 → 16`); the directed asymmetric variant
    holds 7.1 Melem/s.
  - Example: `examples/establishment_demo.rs` builds a 2 000-vertex
    undirected graph with `types = 3`, an assortative pref matrix
    (0.30 within / 0.02 across), and a skewed `type_dist =
    [0.50, 0.25, 0.25]`; prints per-type vertex counts, the
    within-vs-cross-type edge split, and the per-type mean degree —
    the planted assortative structure shows up as
    > 90% within-type edges with the diagonal-heavy pref matrix.
  - Re-exported as `rust_igraph::establishment_game`.

- **ALGO-GN-014** — `preference_game` + `asymmetric_preference_game`
  block-model random-graph generators. Counterparts of
  `igraph_preference_game()` and `igraph_asymmetric_preference_game()`
  in `references/igraph/src/games/preference.c`. The symmetric variant
  draws a single per-vertex type from a categorical `type_dist` (or
  partitions vertices deterministically when `fixed_sizes=true`), then
  for each block pair `(i, j)` runs the Batagelj–Brandes geometric-skip
  sampler over the `vids_by_type[i] × vids_by_type[j]` indirection.
  The asymmetric variant draws each vertex's `(out_type, in_type)` from
  a joint cumulative distribution over `type_dist_matrix` and is always
  directed; it reuses the `PairShape` enum from `sbm.rs` to iterate
  every `(out_type, in_type)` cell.
  - `pub fn preference_game(nodes: u32, types: u32, type_dist: Option<&[f64]>, fixed_sizes: bool, pref_matrix: &[Vec<f64>], directed: bool, loops: bool, seed: u64) -> IgraphResult<(Graph, Vec<u32>)>`.
  - `pub fn asymmetric_preference_game(nodes: u32, no_out_types: u32, no_in_types: u32, type_dist_matrix: Option<&[Vec<f64>]>, pref_matrix: &[Vec<f64>], loops: bool, seed: u64) -> IgraphResult<(Graph, Vec<u32>, Vec<u32>)>`.
  - Validation: `pref_matrix` square (symmetric variant) or
    `no_out_types × no_in_types` (asymmetric); when undirected,
    `pref_matrix` must be symmetric; `type_dist` length matches `types`
    and entries non-negative; when `fixed_sizes=true` and `type_dist`
    is provided, the entries are interpreted as integer block sizes
    that must sum to `nodes`; `type_dist_matrix` (when set) is a
    `no_out_types × no_in_types` non-negative matrix.
  - Coverage: 39 unit tests + 8 proptests (vcount / directed flag /
    no-self-loops when `loops=false` / diagonal-pref keeps edges
    in-block / `fixed_sizes` equal-split counts / determinism per
    seed for both shapes / asymmetric vcount / asymmetric
    no-self-loops / asymmetric determinism) + 18 three-source
    conformance fixtures (3 C + 3 py + 3 R per algorithm) asserting
    structural invariants only — RNG state is not portable across
    SplitMix64 vs igraph's GLIBC RNG. Bench snapshot under
    `.codefuse/tracking/perf/ALGO-GN-014.json` (size scaling at
    `n=5_000, k=4` ≈ 19.6 ms; `k=2 → 8` sweep at `n=1_000` shows
    cost shrinks as block size drops; asymmetric at `n=1_000, 2×3` ≈
    2.07 ms no-loops). Example under `examples/preference_demo.rs`.
- **ALGO-GN-013** — `static_fitness_game` + `static_power_law_game`
  static-fitness / static-power-law random-graph generators. Counterparts
  of `igraph_static_fitness_game()` and `igraph_static_power_law_game()`
  in `references/igraph/src/games/static_fitness.c`. Both place edges by
  drawing endpoints proportional to a per-vertex *fitness* score; the
  power-law variant synthesises that fitness as `f_i = j^α` with
  `α = -1/(γ-1)` (so the marginal degree distribution is asymptotically
  `P(k) ∝ k^-γ`).
  - `pub fn static_fitness_game(no_of_edges: u32, fitness_out: &[f64], fitness_in: Option<&[f64]>, loops: bool, multiple: bool, seed: u64) -> IgraphResult<Graph>`.
  - `pub fn static_power_law_game(no_of_nodes: u32, no_of_edges: u32, exponent_out: f64, exponent_in: Option<f64>, loops: bool, multiple: bool, finite_size_correction: bool, seed: u64) -> IgraphResult<Graph>`.
  - Sampler: build the cumulative-fitness prefix sum once (`O(n)`), then
    sample each edge endpoint via `binsearch_cum_first_ge` (`O(log n)`)
    against `U(0, Σf)`. Loops/multi-edge filtering uses an explicit retry
    loop with a `HashSet<(u32, u32)>` deduplicator and a
    `max_edges()`-validated capacity check upfront so termination is
    guaranteed. Power-law also exposes the **Cho et al. (2009)
    finite-size correction**: when `α < -0.5` the fitness offset is
    shifted by an analytic factor that compensates for the heavy-tail
    cut-off at finite `n`.
  - Validation: fitness scores must be finite and non-negative; vertex
    count bounded by `2^53` (`IGRAPH_MAX_EXACT_REAL`); `fitness_in.len()`
    must match `fitness_out.len()` for the directed shape; for the
    power-law variant `γ ≥ 2` (in **and** out) and not NaN; the
    requested edge count must not exceed `max_edges(...)` capacity given
    `loops` / `multiple` toggles.
  - Coverage: 24 unit tests + 5 proptests (vcount / ecount / directed
    flag invariants, `multiple=false` ⇒ no duplicate canonical pairs,
    `loops=false` ⇒ no self-loops, `multiple=false ⇒ ecount = m`
    exactly, exponent-NaN-rejected, fitness-correlates-with-degree at
    `n=300`, FSC-on/off both produce valid graphs) + 36 three-source
    conformance fixtures (16 C + 10 py + 10 R, evenly split across both
    algorithms) asserting vcount / ecount / directedness / `is_simple` /
    `no_multi_edges` semantics — RNG state is not portable across
    SplitMix64 vs igraph's GLIBC RNG, so we assert structural invariants
    only. Bench snapshot under `.codefuse/tracking/perf/ALGO-GN-013.json`
    (e.g. uniform `f` at `n=5_000` ≈ 3.78 ms; γ-sweep at `n=2_000` ≈
    0.95 ms across γ ∈ [2.1, 4.0]). Example under
    `examples/static_fitness_demo.rs`.
- **ALGO-GN-012** — `chung_lu_game` Chung–Lu expected-degree random graph
  generator. Counterpart of `igraph_chung_lu_game()` in
  `references/igraph/src/games/chung_lu.c`. Given a per-vertex weight
  vector `w` (and optional `in_weights` for the directed shape), the
  base connection probability is `q_ij = w_i · w_j / Σ w_k`; one of
  three closed-form transforms picks the actual edge probability:
  - `pub fn chung_lu_game(out_weights: &[f64], in_weights: Option<&[f64]>, loops: bool, variant: ChungLuVariant, seed: u64) -> IgraphResult<Graph>`.
  - `pub enum ChungLuVariant { Original, Maxent, Nr }`:
    - `Original` — `p_ij = min(q_ij, 1)` (Chung–Lu 2002),
    - `Maxent` — `p_ij = q_ij / (1 + q_ij)` (Park & Newman 2004),
    - `Nr` — `p_ij = 1 − exp(−q_ij)` (Norros–Reittu 2006).
  - Miller–Hagberg O(|V| + |E|) sampler: vertices are sorted descending
    by in-weight, then a per-source row sweep uses a running upper-bound
    probability `p` and `gen_geom(p)` to skip non-edge slots without
    visiting every candidate pair. The descending sort guarantees `q ≤
    p` along the row so the rejection ratio `q / p` is always in `[0,
    1]`.
  - Validation: weights must be finite and non-negative; in the directed
    case `in_weights.len() == out_weights.len()` and the two sums are
    bit-exactly equal (matching the C reference's `IGRAPH_EQUALITY`
    contract); the all-zero-weight input short-circuits to an empty
    graph regardless of variant; vertex count is bounded by `2^53`
    (`IGRAPH_MAX_EXACT_REAL`).
  - Coverage: 16 unit tests + 3 proptests (`vcount`/`directed`
    invariants, `loops=false` → no self-loops, `Original ≥ Maxent ≥ Nr`
    edge-count ordering on the same seed) + 27 three-source conformance
    fixtures (9 each from C / py / R) asserting vcount / directedness /
    ecount band / simple-no-loops semantics — direct sample comparison
    is not portable across SplitMix64 vs igraph's GLIBC RNG, so we
    assert structural invariants only. Bench snapshot under
    `.codefuse/tracking/perf/ALGO-GN-012.json` (e.g. uniform `w=10` at
    `n=5_000` ≈ 1.71 ms). Example under `examples/chung_lu_demo.rs`.
- **ALGO-GN-011** — `hsbm_game` + `hsbm_list_game` Hierarchical Stochastic
  Block Model random graph generator. Counterparts of
  `igraph_hsbm_game()` and `igraph_hsbm_list_game()` in
  `references/igraph/src/games/sbm.c:267-481`. A two-level community
  structure: `K` macro-blocks, each running its own internal SBM over
  `k_b` micro-clusters, plus a single Bernoulli rate `p` for every edge
  crossing two distinct macro-blocks.
  - `hsbm_game(n: u32, m: u32, rho: &[f64], c: &[Vec<f64>], p: f64, seed: u64) -> IgraphResult<Graph>`
    (uniform variant — every macro has size `m`, the same `rho`, the
    same `c`).
  - `hsbm_list_game(n: u32, m_list: &[u32], rho_list: &[Vec<f64>], c_list: &[Vec<Vec<f64>>], p: f64, seed: u64) -> IgraphResult<Graph>`
    (list variant — every macro carries its own size, `rho`, and `c`).
  - Phase 1 generates intra-macro edges by running an SBM independently
    inside each macro over its `csizes = round(rho_b * m_b)` micro
    clusters with pref matrix `c_b`. Phase 2 emits inter-macro edges by
    one cross-pair Batagelj–Brandes geometric-skip sampler per unordered
    macro pair. Total cost is
    `O(Σ_b m_b² · c_max + Σ_{b<b'} m_b · m_{b'} · p + macros²)`.
  - Validation: `n ≥ 1`; for the uniform variant `n % m == 0`; `rho`
    sums to 1 within `√DBL_EPSILON ≈ 1.49e-8`; every `rho[j] * m` rounds
    to an integer within the same tolerance; `c` is `k × k`, symmetric,
    entries in `[0, 1]`; `p ∈ [0, 1]`. The list variant additionally
    requires `m_list.len() == rho_list.len() == c_list.len()`,
    `sum(m_list) == n`, and every `m_list[b] ≥ 1`.
  - Output is always undirected and (modulo Bernoulli realization)
    simple — Phase 1 produces a simple graph and Phase 2 only adds
    edges across different vertex groups.
  - Deterministic given `(n, m, rho, c, p, seed)` (uniform) or
    `(n, m_list, rho_list, c_list, p, seed)` (list) via `SplitMix64`.
  - Conformance: 18 fixtures total — 9 for `hsbm_game` (3 C + 3 py +
    3 R) under `tests/conformance/{c,py,r}/hsbm_game/` and 9 for
    `hsbm_list_game` under `tests/conformance/{c,py,r}/hsbm_list_game/`.
    RNG state is not portable across implementations, so fixtures
    assert structural invariants only — `vcount` exact, `directed = false`,
    `ecount` band (exact for the `p = 0` and `p = 1` corner cases like
    the C `igraph_hsbm_game.out` K_{6,4} `n=10/m=10/rho=[0.6,0.4]`
    bipartite shape with `c = [[0,1],[1,0]]` giving exactly `6·4 = 24`
    edges, or the `p = 1 / c = 0` complete-`K_n`-between-macros R
    `g_hsbm5` giving exactly `m·m'` edges), and `is_simple` (canonical-
    pair `HashSet`) for the deterministic fixtures. The list-variant
    test additionally asserts `vcount == sum(m_list)` to catch manifest
    typos.
  - Bench at `benches/bench_hsbm.rs`: a
    `size_scaling/uniform_k2` sweep (`n ∈ {200, 1 000, 4 000}` at fixed
    `m = 50`, two-cluster `c` with `p_in = 0.1 / p_off = 0.02`,
    inter-macro `p = 0.01`), a `density_sweep/uniform_n2000_m50` over
    in-cluster `p ∈ {0.01, 0.05, 0.10, 0.30}`, a
    `p_sweep/uniform_n2000_m100` over inter-macro
    `p ∈ {0.0, 0.01, 0.10, 0.50, 1.00}`, a `list_vs_uniform/n2000_k20`
    pair comparing the uniform and list APIs on the same shape, and a
    `list_irregular/n2200_jagged` point with non-uniform macro sizes
    `m_list = [40, 80, 120, 160, 200, 240, 280, 320, 360, 400]` and
    per-macro variable `rho`/`c`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-011.json`: at `n = 200`,
    `m = 50`, uniform runs at 19.5 µs; scales to 5.24 ms at `n = 4000`
    (80 macros, ~3160 inter-macro pairs). The `p_sweep` at
    `n = 2000 / m = 100` reveals the inter-macro Bernoulli sweep cost:
    246 µs at `p = 0` (pure intra-macro) up to 123.8 ms at `p = 1`
    (dense — ~1.9M cross-macro edges). The list variant
    `list_equivalent` runs at 1.37 ms vs the uniform `1.40 ms` —
    confirms the per-macro dispatcher overhead is essentially free.
  - Example: `examples/hsbm_demo.rs` generates a 3-level hierarchy
    (`n = 120`, `4` macros of `m = 30`, each with `3` equal
    micro-clusters of `10`, `p_in = 0.40 / p_off = 0.05` within macros,
    inter-macro `p = 0.01`) and prints the `4 × 4` macro-block edge
    matrix, per-macro `3 × 3` micro-cluster edge matrices, and mean
    degree per macro — the planted three-level community structure is
    visible at the command line. The same shape is replayed through
    `hsbm_list_game` to confirm the APIs are interchangeable.
  - Re-exported as `rust_igraph::{hsbm_game, hsbm_list_game}`.

- **ALGO-GN-010** — `sbm_game` Stochastic Block Model random graph
  generator. Counterpart of `igraph_sbm_game()` in
  `references/igraph/src/games/sbm.c:78`. Given a `k × k` preference
  matrix `P` and a `k`-vector of `block_sizes` summing to `n`, every
  pair `(u, v)` is connected independently with probability
  `P[block(u)][block(v)]` (or, in multigraph mode, with
  `Pascal`-distributed multiplicity of mean `P[..][..]`).
  - `sbm_game(pref_matrix: &[Vec<f64>], block_sizes: &[u32], directed: bool, loops: bool, multiple: bool, seed: u64) -> IgraphResult<Graph>`.
  - Each block-pair is sampled independently with a Batagelj–Brandes
    geometric-skip sampler over the local pair-index space. Four
    `PairShape` decoders select the right index → `(vfrom, vto)`
    formula per pair: `Rect` for off-diagonal (or directed-with-loops
    diagonal) blocks, `RectNoDiag` for the directed-no-loops diagonal
    (the diagonal element is folded into the last column), `TriInclDiag`
    for the undirected-with-loops diagonal, and `TriExclDiag` for the
    undirected-no-loops diagonal. Total cost is `O(n + m + k²)` where
    `m` is the realised edge count.
  - Multigraph mode replaces the per-pair connection probability `p`
    with `p / (1 + p)` and drops the `+1` step in the geometric-skip
    loop. This is exact Pascal-distributed multiplicity sampling — no
    rejection, no per-pair retry; the geometric draws give edge
    *multiplicities* directly.
  - Validation: every pref-matrix row must have length `k`, every entry
    must be finite, in `[0, 1]` (probability mode) or non-negative
    (multigraph mode); `pref_matrix` must be symmetric when `!directed`;
    `sum(block_sizes)` must fit in `u32`.
  - Edge cases: empty `block_sizes` or `n = 0` returns an empty graph;
    zero-sized intermediate blocks are skipped; `pref_matrix[i][j] = 0`
    short-circuits the corresponding sampler entirely so the seed is
    irrelevant for that block-pair.
  - Deterministic given `(pref_matrix, block_sizes, directed, loops,
    multiple, seed)` via `SplitMix64`.
  - Conformance: 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/sbm_game/`. RNG state is not portable
    across implementations, so fixtures assert structural invariants
    only — `vcount = sum(block_sizes)` (exact), `directed` flag (exact),
    `ecount` band (hand-derived from the expected `Σ P[i][j] · pairs(i,j)`
    plus tolerance), `is_simple` via canonical-pair `HashSet` when
    `multiple = false` and `loops = false`, and `diagonal_only_pref`
    which asserts no between-block edges exist when the pref matrix is
    block-diagonal (every cross-block edge would imply a sampling bug).
  - Bench at `benches/bench_sbm.rs`: a
    `size_scaling/balanced_k4` sweep (`total ∈ {500, 5_000}`,
    assortative `p_in = 0.05 / p_off = 0.005`), a
    `sparsity/off_diagonal_k4_n2000` sweep over `p_off ∈ {0.0, 0.001,
    0.005, 0.05}` at fixed `p_in = 0.05`, a `density_sweep/k4_n2000`
    over `p_in ∈ {0.01, 0.05, 0.10, 0.30}` at fixed `p_off = 0.005`, a
    `directed/k4_n2000_asym` point with an asymmetric pref matrix, and
    a `multigraph/k4_n2000_dense` point exercising the Pascal fast
    path. Baseline at `.codefuse/tracking/perf/ALGO-GN-010.json`: the
    n=500 / k=4 / assortative path runs at ~90 µs; n=5000 at ~12.4 ms
    (≈10× edges scale n²); the density sweep grows roughly linearly
    with within-block edges as expected from the geometric-skip cost
    being dominated by `Vec::push` + `gen_geom`; the directed
    asymmetric variant takes ~2.2× the equivalent undirected because
    directed touches twice as many pair shapes.
  - Example: `examples/sbm_demo.rs` generates a 4-community SBM on 100
    vertices (within-block `p = 0.30`, between-block `p = 0.02`) and
    prints per-block edge counts, density, mean degree, the symmetric
    `k × k` block-edge matrix, and the top-3 highest-degree vertices in
    each block — the planted communities are visible at a glance.
  - Re-exported as `rust_igraph::sbm_game`.

- **ALGO-GN-009** — `watts_strogatz_game` Watts–Strogatz 1-D small-world
  random graph generator. Counterpart of `igraph_watts_strogatz_game()`
  in `references/igraph/src/games/watts_strogatz.c:75-118` for the
  `dim = 1` case — by far the dominant use case and the original
  Nature '98 Watts & Strogatz model. The upstream C entry point
  delegates to `igraph_square_lattice` + `igraph_rewire_edges`; both
  helpers are folded into a single `src/algorithms/games/watts.rs`
  rather than ported as standalone public APIs.
  - `watts_strogatz_game(size: u32, nei: u32, p: f64, loops: bool, multiple: bool, seed: u64) -> IgraphResult<Graph>`.
  - Step 1 builds a periodic 1-D ring lattice: every vertex `v` emits
    forward edges to `(v + 1) % size, …, (v + nei) % size`. Total degree
    per vertex is `2 · nei`; total edges is `size · nei`.
  - Step 2 walks the edge list twice (one pass per endpoint side, mirroring
    upstream's `igraph_rewire_edges`) and rewires each endpoint with
    probability `p`, sampled via geometric skips (`SplitMix64::gen_geom`).
    Replacement vertices are drawn uniformly from `[0, size)` with optional
    rejection — `loops = false` uses the C "draw from `[0, size-1)` and
    remap forbidden→`size-1`" trick for O(1) uniform sampling minus one
    vertex; `multiple = false` rejects candidates that would create a
    duplicate edge via a `HashSet<(u32, u32)>` of canonical pairs (C uses
    a stub linked-list with the same asymptotics).
  - Validation: `size ≥ 1`, `2 · nei + 1 ≤ size` (so the ring lattice
    does not self-overlap), `p ∈ [0, 1]` and finite.
  - Edge invariants: rewire never adds or drops edges, only mutates
    endpoints. Edge count `size · nei` is preserved for all `p`. Output is
    always undirected.
  - Edge cases: `nei = 0` returns an edgeless graph; `2 · nei + 1 = size`
    (the upper-bound case) is the complete graph `K_size`; `p = 0`
    short-circuits the rewire entirely so the seed is irrelevant.
  - Deterministic given `(size, nei, p, loops, multiple, seed)` via
    `SplitMix64`.
  - Conformance: 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/watts_strogatz_game/`. RNG state is not
    portable across implementations, so fixtures assert structural
    invariants only — `vcount = size`, `directed = false`, `ecount`
    exact (`size · nei`), `every_degree` per-vertex (for `p = 0`
    fixtures), and `is_simple` (canonical-pair `HashSet`) for fixtures
    with `multiple = false` and `loops = false`.
  - Bench at `benches/bench_watts.rs`: a `size_scaling/p0_ring` sweep at
    fixed `nei = 4` (`size ∈ {100, 1 000, 10 000}`), a
    `size_scaling/p_half_simple` sweep at the same sizes with `p = 0.5`,
    a `p_sweep/size1000_nei4` curve over `p ∈ {0.01, 0.10, 0.50, 1.00}`,
    and a `multigraph/fast_path` point at `size = 10 000, nei = 4,
    p = 0.5, loops = true, multiple = true`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-009.json`: the pure ring path
    (`p = 0`) runs at 5.8–7.4 Melem/s; the simple-graph small-world
    (`p = 0.5`) is ~3.5× slower (1.6 Melem/s) because of the canonical-
    pair `HashSet` work per rewire; the multigraph fast path runs at
    3.9 Melem/s — ~2.5× faster than the simple-graph equivalent because
    the rejection set is skipped entirely.
  - Example: `examples/watts_demo.rs` sweeps `p ∈ {0.0, 0.05, 0.3, 1.0}`
    on a 32-vertex `nei = 2` ring and prints the count of "long-range"
    edges (ring-distance > `nei`) at each `p`, tracing the lattice →
    small-world → random transition in plain text.
  - Re-exported as `rust_igraph::watts_strogatz_game`.

- **ALGO-GN-008** — `k_regular_game` random graph generator (k-regular
  sampler). Counterpart of `igraph_k_regular_game()` in
  `references/igraph/src/games/degree_sequence.c:38-122` which itself
  thinly wraps `igraph_degree_sequence_game()` on a length-`n` constant
  degree sequence (CONFIGURATION for multigraph, FAST_HEUR_SIMPLE for
  simple). Self-rolled rather than porting the 8-mode 864-line
  `degree_sequence` machinery — only the two paths the public API needs
  are implemented.
  - `k_regular_game(n: u32, k: u32, directed: bool, multiple: bool, seed: u64) -> IgraphResult<Graph>`.
  - Multigraph path (`multiple = true`) is the configuration model —
    build `n · k` stubs, Fisher-Yates shuffle once, pair adjacent. For
    directed graphs there are separate out-bag and in-bag, each shuffled
    independently and zipped. Output is allowed to have self-loops and
    parallel edges; total cost `O(n · k)`.
  - Simple path (`multiple = false`) is the fast-heur sampler — same
    stub bag, but each pair is rejected if it would create a self-loop
    or duplicate (`HashSet` adjacency check). Rejected stubs are carried
    over to the next sweep; when no more feasible pairs remain, the run
    restarts from scratch. Capped at 1024 restarts to guarantee
    termination on pathological inputs.
  - Validation: `n · k` must fit in `u32`; for undirected sampling
    `n · k` must be even (handshake parity); for simple sampling
    `k ≤ n − 1` (so the sampler is not asked to draw more distinct
    neighbours than exist).
  - Edge cases: `n = 0` returns an empty graph; `k = 0` returns an
    edgeless `n`-vertex graph; `k = n − 1` on the simple path is a
    deterministic complete graph.
  - Deterministic given `(n, k, directed, multiple, seed)` via
    `SplitMix64`.
  - Conformance: 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/k_regular_game/`. RNG state is not
    portable across implementations, so fixtures assert structural
    invariants only — `vcount = n`, `directed` flag matches, `ecount`
    exact band (`n · k / 2` undirected, `n · k` directed),
    `every_degree` / `every_out_degree` / `every_in_degree` per-vertex
    assertions, and `is_simple` (canonical-pair `HashSet`) for the
    simple-path fixtures.
  - Bench at `benches/bench_k_regular.rs`: an `n_scaling/simple` sweep
    at fixed `k = 6` (`n ∈ {50, 200, 1 000}`), a `k_sweep/simple` at
    `n = 200` (`k ∈ {4, 16, 64}`), a `multigraph/configuration` point
    at `n = 500, k = 10`, and a `directed/simple` point at
    `n = 200, k = 8`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-008.json`: simple n-scaling sits
    around 4 Melem/s up to `n = 1 000`; the simple k-sweep falls from
    4.5 Melem/s at `k = 4` to 232 Kelem/s at `k = 64` because the
    HashSet-based rejection rate rises with density; the configuration
    multigraph path runs at ~4.8 Melem/s.
  - Example: `examples/k_regular_demo.rs` generates one undirected
    simple 4-regular, one directed simple 3-regular, and one undirected
    multigraph 6-regular sample, then prints per-vertex degree (or
    `(out, in)` pair) so the regularity invariant is visible in plain
    text.
  - Re-exported as `rust_igraph::k_regular_game`.

- **ALGO-GN-007** — `simple_interconnected_islands_game` random graph
  generator. Counterpart of
  `igraph_simple_interconnected_islands_game()` in
  `references/igraph/src/games/islands.c:55-176`. Builds `islands_n`
  Erdős–Rényi G(n, p) islands of equal size and connects every pair of
  islands with exactly `n_inter` bipartite edges sampled uniformly at
  random from the `islands_size × islands_size` cell.
  - `simple_interconnected_islands_game(islands_n: u32, islands_size: u32, islands_pin: f64, n_inter: u32, seed: u64) -> IgraphResult<Graph>`.
  - Intra-island sampling uses Batagelj–Brandes geometric-skip over the
    strict upper triangle (cost `O(islands_size + |E_i|)`). Inter-island
    sampling uses Floyd's distinct-sample to draw `n_inter` edges from
    `islands_size²` cells (expected `O(n_inter)`).
  - Validation: `islands_pin ∈ [0, 1]` (NaN/Inf rejected),
    `n_inter ≤ islands_size²`, and `islands_n · islands_size ≤ u32::MAX`.
  - Output is always undirected and always simple — the intra slice is
    strictly upper-triangular, and the inter slice samples a bipartite
    cell whose index space is disjoint from every intra slice by
    construction.
  - Edge cases: `islands_n = 0` or `islands_size = 0` returns an empty
    graph; `islands_pin = 0` skips intra sampling entirely; the
    `n_inter = islands_size²` saturation case is exercised by tests.
  - Deterministic given `(islands_n, islands_size, islands_pin, n_inter,
    seed)` via `SplitMix64`.
  - Conformance: 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/simple_interconnected_islands_game/`.
    RNG state is not portable across implementations, so fixtures
    assert structural invariants only — `vcount`, `directed = false`,
    `is_simple` (canonical-pair `HashSet`), and an `ecount` band built
    from `E[intra] = islands_n · C(size, 2) · pin` plus
    `exact_inter = C(islands_n, 2) · n_inter`.
  - Bench at `benches/bench_islands.rs`: an `islands_n` scaling sweep
    at fixed `islands_size = 50, islands_pin = 0.10, n_inter = 3`, plus
    a `pin_sweep` at `islands_n = 10, islands_size = 30`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-007.json`: 8.5 / 3.3 / 2.7 Melem/s
    at `islands_n = 4 / 20 / 100`; pin sweep 15.0 / 6.6 / 2.9 Melem/s at
    `pin = 0.05 / 0.20 / 0.50`.
  - Example: `examples/islands_demo.rs` builds a 6 × 25 lattice and
    prints the per-island intra-edge counts plus the per-pair
    inter-edge counts so the block structure is visible in plain text;
    every bipartite slice should hit exactly `n_inter`.
  - Re-exported as `rust_igraph::simple_interconnected_islands_game`.

- **ALGO-GN-006** — `forest_fire_game` Leskovec–Kleinberg–Faloutsos
  forest-fire random graph generator (KDD'05). Counterpart of
  `igraph_forest_fire_game()` in
  `references/igraph/src/games/forestfire.c:106-257`. Nodes arrive one
  at a time; each new node cites `ambs` ambassador vertices and then
  BFS-burns outward, drawing `Geom(1 − fw_prob)` outgoing and
  `Geom(1 − fw_prob · bw_factor)` incoming neighbours per ambassador on
  the burn frontier.
  - `forest_fire_game(n: u32, fw_prob: f64, bw_factor: f64, ambs: u32, directed: bool, seed: u64) -> IgraphResult<Graph>`.
  - Mirrors the corrected variant from the CMU tech report — a single
    geometric draw rather than the published paper's `mean = p/(1−p)`.
  - Validation: `fw_prob ∈ [0, 1)` (strict upper bound so the geometric
    draw is finite) and `bw_factor · fw_prob ∈ [0, 1)`; NaN/Inf rejected.
  - Edge cases: `n = 0` returns an empty graph; `n = 1` returns a
    singleton; `ambs = 0` shortcuts to an edgeless `n`-vertex graph.
  - Output is loop-free and parallel-edge-free by construction (the
    per-actnode `visited` stamp prevents both); when `directed = true`
    every emitted edge has `src > dst`, so the graph is a DAG.
  - Deterministic given `(n, fw_prob, bw_factor, ambs, directed, seed)`
    via `SplitMix64`.
  - Conformance: 9 fixtures (3 C + 3 py + 3 R) under
    `tests/conformance/{c,py,r}/forest_fire_game/`. RNG state is not
    portable, so fixtures assert structural invariants only — `vcount`,
    `directed` flag, `is_simple` (no self-loops, no multi-edges via
    canonical-pair `HashSet`), and a loose `ecount` band (lower
    bound = `n − 1` when `ambs > 0` since each new node contributes at
    least one fresh citation).
  - Bench at `benches/bench_forestfire.rs` (directed + undirected groups,
    `n ∈ {100, 1 000, 10 000}`) at the operating point
    `fw_prob = 0.20, bw_factor = 0.40, ambs = 2`. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-006.json`: 4.0 / 2.2 / 1.6 Melem/s
    directed, 2.2 / 1.9 / 1.6 Melem/s undirected.
  - Example: `examples/forestfire_demo.rs` runs the model on 2 000
    vertices and reports the top-10 in-degree hubs plus a power-of-2
    in-degree histogram showing the heavy-tailed degree distribution
    that motivated the model in the first place.
  - Re-exported as `rust_igraph::forest_fire_game`.

- **ALGO-GN-005** — `grg_game` geometric random graph generator.
  Counterpart of `igraph_grg_game()` in
  `references/igraph/src/games/grg.c:56-174`. Drops `n` uniform points
  on the unit square and connects every pair within Euclidean distance
  strictly less than `radius`, with optional periodic boundary
  conditions (`torus = true`).
  - `grg_game(n: u32, radius: f64, torus: bool, seed: u64) -> IgraphResult<Graph>`.
  - `grg_game_with_coords(n, radius, torus, seed) -> IgraphResult<(Graph, Vec<f64>, Vec<f64>)>`
    returns the sorted `xs` and the original-order `ys`, faithfully
    mirroring upstream's `igraph_vector_sort(xx)` which sorts only the
    x-array (the (x, y) pairing is intentionally broken — the joint
    marginal is still uniform because the two axes were independent).
  - Algorithm: O(n log n) sort + O(n + |E|) x-sweep with a width-`radius`
    window — each candidate pair is inspected exactly once. Torus mode
    adds a wrap-around tail gated on `j == n` and the
    `xi − xs[k] >= radius` guard to avoid double-counting.
  - Output is always undirected and simple — the sweep starts at
    `j = i + 1` (no self-loops) and visits each pair once (no
    multi-edges). Negative or zero radius yields an empty graph
    (matching upstream's strict `<` comparison).
  - Deterministic given `(n, radius, torus, seed)` via `SplitMix64`.
  - Conformance: 10 fixtures (3 C + 4 py + 3 R) under
    `tests/conformance/{c,py,r}/grg_game/`. RNG state is not portable,
    so fixtures assert structural invariants only — `vcount`,
    `directed == false`, `is_simple` (no loops, no multi-edges via
    canonical-pair `HashSet`), and a loose `ecount` band derived from
    `E[edges] = C(n,2) · π · r²` (interior bulk) with ±50 % tolerance.
    The dense fixture (`r > sqrt(2)`) pins the count to `C(n,2)` exactly.
  - Bench at `benches/bench_grg.rs` (plane + torus groups,
    `n ∈ {100, 1 000, 10 000}`) with the radius auto-tuned for an
    expected average degree of ~10. Baseline at
    `.codefuse/tracking/perf/ALGO-GN-005.json`: 8.1 Melem/s at
    `n = 100`, 4.6 Melem/s at `n = 1 000`, 2.7 Melem/s at `n = 10 000`
    on the plane; torus runs ~20-35 % slower from the per-pair y-wrap
    fold and the wrap-around tail.
  - Example: `examples/grg_demo.rs` contrasting plane vs. torus mean
    degree against the `E[deg] = (n − 1) · π · r²` bulk and reporting
    the largest connected component.
  - Re-exported as `rust_igraph::{grg_game, grg_game_with_coords}`.

- **ALGO-GN-004** — `tree_game_lerw` uniform random labelled-tree generator
  via Wilson's loop-erased random walk. Counterpart of
  `igraph_tree_game(..., IGRAPH_RANDOM_TREE_LERW)` in
  `references/igraph/src/games/tree.c:72-139`. The Prüfer-sequence
  alternative (`IGRAPH_RANDOM_TREE_PRUFER`) is deferred — it depends on
  `igraph_from_prufer`, which is not yet ported.
  - `tree_game_lerw(n: u32, directed: bool, seed: u64) -> IgraphResult<Graph>`.
    Samples a labelled tree on `n` vertices uniformly at random
    (every one of Cayley's `n^{n-2}` trees has equal probability) by
    performing a loop-erased random walk on the complete graph `K_n`.
    The classical visited/unvisited partition trick (`vertices[0..k)`
    visited, `[k, n)` unvisited; resample from the unvisited tail when
    the first draw hits a visited slot) collapses the walk into a
    single linear pass — every iteration emits exactly one edge with
    at most two RNG draws and no rejection loop.
  - Cost: `O(n)` time, `O(n)` auxiliary memory (one `Vec<u32>`, one
    `Vec<bool>`, one `Vec<(u32, u32)>`); the graph itself is never
    touched during sampling.
  - Edge count is deterministic: exactly `max(0, n − 1)` edges. Output
    is always a spanning tree (acyclic + connected). In `directed`
    mode the tree is out-rooted at the random initial vertex; every
    non-root vertex has in-degree exactly 1.
  - Deterministic given `(n, directed, seed)` via the `SplitMix64`
    PRNG shared with the other generators.
  - Allocation-safe: caps `n` at `u32::MAX` and reserves all buffers
    exactly so absurd inputs error cleanly rather than panic.
  - Conformance: 10 fixtures (3 C + 4 py + 3 R) under
    `tests/conformance/{c,py,r}/tree_game_lerw/`. RNG state is not
    portable across implementations, so fixtures are hand-derived and
    assert structural invariants only — `vcount == n`,
    `ecount == max(0, n − 1)`, and (for `is_tree == true` cases) a
    union-find pass verifies acyclic + connected.
  - Bench at `benches/bench_tree_game.rs` (undirected + directed
    groups, `n ∈ {100, 1 000, 10 000}`). Baseline at
    `.codefuse/tracking/perf/ALGO-GN-004.json`: ≈ 22-25 Melem/s
    vertices for `n ≤ 1 000` (the bool-visited array fits in L1),
    dropping to ≈ 17.6 Melem/s at `n = 10 000` once buffers cross
    L1/L2. Directed and undirected paths are within 1 %.
  - Example: `examples/tree_game_demo.rs` showing the spanning-tree
    invariants and a BFS-depth histogram from the random root.
  - Re-exported as `rust_igraph::tree_game_lerw`.

- **ALGO-GN-003** — `growing_random_game` uniform-kernel growing random
  graph generator. Counterpart of `igraph_growing_random_game()` in
  `references/igraph/src/games/growing_random.c:55-105`.
  - `growing_random_game(n: u32, m: u32, directed: bool, citation: bool,
    seed: u64) -> IgraphResult<Graph>`. The graph starts as a single
    seed vertex; on each of the remaining `n - 1` steps a fresh vertex
    arrives together with exactly `m` new edges. Two endpoint-selection
    rules are exposed:
    * **Citation** (`citation = true`): every new edge originates at
      the freshly-added vertex `i` and lands on a uniformly chosen
      earlier vertex `to ∈ [0, i - 1]`. Result is strictly
      time-ordered (directed: `dst < src` for every edge), free of
      self-loops, and vertex 0 never appears as a source.
    * **Free** (`citation = false`): both endpoints are uniformly
      sampled within the current frontier — `from ∈ [0, i]` (new vertex
      allowed) and `to ∈ [1, i]` (vertex 0 excluded from sinks),
      mirroring the asymmetric closed intervals upstream.
  - Edge count is deterministic given `(n, m)`: exactly `(n - 1) · m`
    edges. Total cost: `O(n · m)` work, `O(n · m)` memory for the edge
    list only — no degree bookkeeping (the kernel is uniform, not
    preferential).
  - Deterministic given `(n, m, directed, citation, seed)` via the
    `SplitMix64` PRNG shared with the other generators.
  - Allocation-safe: `validate_inputs` caps `(n - 1) · m` at
    `u32::MAX` (`IGRAPH_ECOUNT_MAX` convention) so absurd inputs error
    cleanly rather than panic at `Vec::with_capacity`.
  - Conformance: 10 fixtures (3 C + 4 py + 3 R) under
    `tests/conformance/{c,py,r}/growing_random_game/`. RNG state is
    not portable across implementations, so fixtures are hand-derived
    and assert structural invariants only — `vcount`, `ecount`, and
    `is_directed` are exact, and (when applicable) the BA-style
    temporal-ordering assertion is reused (`dst < src` directed,
    `src != dst` undirected).
  - Bench at `benches/bench_growing_random.rs` (citation `m = 2`, free
    `m = 2`, citation `m = 5`). Baseline at
    `.codefuse/tracking/perf/ALGO-GN-003.json`: ≈ 14 Melem/s vertices
    for `m = 2` at `n ≤ 1 000` (citation, single RNG draw per edge),
    dropping to ≈ 11 Melem/s at `n = 10 000` as the edge buffer leaves
    L2. Free mode is ≈ 15 % slower because each edge takes two RNG
    draws.
  - Example: `examples/growing_random_demo.rs` showing the
    citation-mode DAG structure and the in-degree distribution.
  - Re-exported as `rust_igraph::growing_random_game`.
- **ALGO-GN-002** — `barabasi_game_bag` preferential-attachment random
  graph generator (BAG variant). Counterpart of the
  `IGRAPH_BARABASI_BAG` branch of `igraph_barabasi_game()` in
  `references/igraph/src/games/barabasi.c:67-178`.
  - `barabasi_game_bag(n: u32, m: u32, outpref: bool, directed: bool,
    seed: u64) -> IgraphResult<Graph>`. Classical Albert–Barabási "bag"
    mechanism: a multiset whose multiplicity of vertex `v` equals
    `deg(v) + 1`, so a uniform draw is proportional to degree. MVP
    scope hardcodes `power = 1`, `A = 1` (the only setting BAG
    supports per upstream `barabasi.c:567-574`), constant `m` (no
    `outseq`), and seeds with the singleton `[0]` (no `start_from`).
    `outpref` is forced to `true` for undirected graphs to match
    upstream `barabasi.c:83-85`. Total cost: `O(n · m)` work, `O(n ·
    m)` memory for bag + edge list.
  - Edge count is deterministic given `(n, m)`: exactly `(n - 1) · m`
    edges. Since the bag is sampled with replacement, the output may
    contain multi-edges. Self-loops are *not* produced because the
    algorithm pushes vertex `i` to the bag *after* its own draws.
  - Deterministic given `(n, m, outpref, directed, seed)` via the
    `SplitMix64` PRNG shared with the other generators.
  - Conformance: 10 fixtures (3 C + 4 py + 3 R) under
    `tests/conformance/{c,py,r}/barabasi_game_bag/`. Since
    cross-implementation RNG state is not portable, the fixtures are
    hand-derived and assert structural invariants only — `vcount` and
    `ecount` are exact, `is_directed` is exact, and the BA temporal
    ordering (`dst < src` for every edge in the directed case;
    no-self-loop in the undirected case where the storage layer
    canonicalises endpoints) is verified per-edge.
  - Bench at `benches/bench_barabasi.rs` (directed `m = 2`, directed
    `m = 5`, undirected `m = 2` with auto-promoted `outpref`). Baseline
    at `.codefuse/tracking/perf/ALGO-GN-002.json`: ≈ 13–14 Melem/s
    vertices for `m = 2` at `n ≤ 1 000` (cache-resident bag), dropping
    to ~10 Melem/s at `n = 10 000` once the bag overflows L2.
  - Example: `examples/barabasi_demo.rs` showing the degree-hub
    concentration (vertex 0 dominates as expected).
  - Re-exported as `rust_igraph::barabasi_game_bag`.
- **ALGO-GN-001** — `erdos_renyi_gnp` + `erdos_renyi_gnm` random
  graph generators. Counterparts of `igraph_erdos_renyi_game_gnp` /
  `igraph_erdos_renyi_game_gnm` in
  `references/igraph/src/games/erdos_renyi.c`. Both honour the four
  (directed × loops) cases.
  - `erdos_renyi_gnp(n: u32, p: f64, directed: bool, loops: bool, seed:
    u64) -> IgraphResult<Graph>`. Batagelj–Brandes geometric-skip
    sampling: O(n + m_expected) work, where `m_expected = p ·
    max_edges`. `p` must lie in `[0, 1]` and be non-NaN; `n = 0`
    returns an empty graph; `p = 0` returns a graph with no edges;
    `p = 1` returns the complete graph (with self-loops if `loops`).
  - `erdos_renyi_gnm(n: u32, m: u64, directed: bool, loops: bool, seed:
    u64) -> IgraphResult<Graph>`. Floyd distinct-sample: O(n + m) work
    selecting `m` edges uniformly without replacement from the
    `max_edges` possible pairs. Errors when `m > max_edges`.
  - Both accept an explicit `seed: u64` for reproducibility — the
    generators are deterministic given `(n, p|m, directed, loops,
    seed)`. SplitMix64 PRNG promoted to `src/core/rng.rs` and is
    available to subsequent generator AWUs.
  - Conformance: 20 fixtures (3 C + 4 py + 3 R for `gnp`, 3 C + 4 py +
    3 R for `gnm`) under
    `tests/conformance/{c,py,r}/erdos_renyi_{gnp,gnm}/`. Since the
    upstream C/py/R RNGs are not portable, each fixture is
    hand-derived and asserts the structural invariants only —
    `vcount` is exact, `is_directed` is exact, `gnp` ecount is bounded
    by a ±6σ Binomial band around `µ = p · max_edges`, `gnm` ecount is
    exact.
  - Benches at `benches/bench_erdos_renyi.rs` (sparse `gnp` at average
    degree ≈ 4, dense `gnp` at `p = 0.5`, sparse `gnm` at `m = 2n`).
    Baseline at `.codefuse/tracking/perf/ALGO-GN-001.json`: ≈ 13–15
    Melem/s edges for sparse `gnp` and ≈ 19–22 Melem/s for `gnm`
    across `n ∈ {100, 1 000, 10 000}` on Apple Silicon.
  - Example: `examples/erdos_renyi_demo.rs`.
  - The placeholder synthetic generator in `benches/bench_bfs.rs` now
    delegates to `erdos_renyi_gnp`, eliminating the BFS bench TODO.
- **ALGO-MST-001** — `minimum_spanning_tree` (Prim / Kruskal /
  Unweighted / Automatic). Counterpart of
  `igraph_minimum_spanning_tree` in
  `references/igraph/src/misc/spanning_trees.c`. Returns the IDs of the
  edges that constitute the minimum spanning tree (or forest when the
  input is disconnected); directed graphs are treated as undirected.
  - `minimum_spanning_tree(graph: &Graph, weights: Option<&[f64]>,
    method: MstAlgorithm) -> IgraphResult<Vec<EdgeId>>`. `weights`
    must match `ecount()` and contain no NaN when supplied. Required
    for `Prim` / `Kruskal`; `Unweighted` ignores it; `Automatic`
    dispatches to `Unweighted` when `weights` is `None`, otherwise
    `Kruskal`.
  - `MstAlgorithm` selector enum mirroring upstream
    `igraph_mst_algorithm_t`: `Automatic`, `Unweighted` (BFS spanning
    forest), `Prim` (eager binary-heap, `f64::total_cmp` ordering with
    edge-ID tiebreak for determinism), `Kruskal` (sort once + path-
    compressed union-find).
  - Conformance: 9 fixtures (3 each from C / py / R) under
    `tests/conformance/minimum_spanning_tree/`. Each asserts the
    matroid invariant (total weight + edge count) rather than exact
    edge IDs to absorb tiebreak differences across variants. The
    upstream C test uses an RNG-seeded Erdős–Rényi graph that does not
    port to Rust; we instead exercise the three dispatch branches on
    small hand-derived graphs.
  - Benches at `benches/bench_mst.rs` (karate / sparse synthetic /
    `K_n`); baseline at `.codefuse/tracking/perf/ALGO-MST-001.json`.
    Kruskal beats Prim 10–25× on every input shape — the heap-based
    Prim pays log-m per push and incurs cache misses, while Kruskal's
    sort-once is cache-friendly. A future perf pass could swap the
    binary heap for a d-ary or indexed heap (mirroring upstream
    `igraph_d_indheap` at `spanning_trees.c:176`); not on the v0.0.1
    critical path.
  - Example: `examples/mst_karate.rs`.
- **ALGO-CO-009** — Voronoi-based community detection (Deritei et al.
  2014, Molnár et al. 2024). Counterpart of
  `igraph_community_voronoi` in
  `references/igraph/src/community/voronoi.c`. Greedy generator picking
  via Local Relative Density (LRD): each unassigned vertex's LRD is the
  mean shortest-path distance to all reachable vertices; the
  lowest-LRD vertex (smallest vertex id on ties — fully deterministic,
  no RNG) becomes a generator and claims every vertex within radius
  `r · d̄` where `d̄` is the global mean distance. Repeats until all
  vertices are assigned, then the final cell membership is computed by
  `voronoi()` (ALGO-SP-007).
  - `community_voronoi(graph: &Graph, lengths: Option<&[f64]>, weights:
    Option<&[f64]>, mode: DijkstraMode, r: f64) -> IgraphResult<
    CommunityVoronoiResult>`. `lengths` are edge costs for the LRD /
    Voronoi step (default 1.0); `weights` are edge weights for the
    modularity computation that auto-r maximises. `mode` is honoured
    only when the graph is directed (undirected collapses to `All`).
  - **Auto-r**: passing `r = -1.0` wraps the inner pick+assign in a
    Brent quadratic-fit 1D optimizer over `r ∈ (0.001·d̄,
    100·d̄)` with up to 25 iterations, returning the `r` that
    maximises Newman-Girvan modularity (weighted when `weights` is
    `Some`). The result's `modularity` field is `Some(q)` for auto-r and
    fixed `r` runs where modularity is well-defined, `None` for the
    degenerate single-community case.
  - **Determinism**: unlike the C reference which uses
    `igraph_rng_get_integer` to break LRD ties, our implementation
    breaks ties by smallest vertex id — strictly deterministic, no RNG
    parameter. On Zachary karate club with `r = -1` (auto-r), this
    still picks the same generator set `[33, 0, 24]` and produces
    3 communities as the C reference.
  - Errors: `InvalidArgument` for `lengths`/`weights` length ≠ `ecount`
    or containing non-finite/negative values; `Internal` for Brent
    optimizer degenerate cases (`f1 > f3` at start, or drift outside
    the initial interval — both indicate a flat / monotone modularity
    surface on degenerate inputs).
  - **Perf** (`benches/bench_community_voronoi.rs`, Apple Silicon):
    - karate fixed-r=1 (34v 78e): **~14.8 µs**;
    - karate auto-r (34v 78e): **~39.6 µs** (~3× the fixed-r cost for
      the Brent outer loop);
    - 20×20 weighted grid fixed-r=1 (400v): **~540 µs**.
    `python-igraph` 0.11.9 and `rigraph` do not bind
    `community_voronoi`, so no cross-language baseline is published.
  - Conformance: 4 fixtures per source (C / py / R, 12 total) under
    `tests/conformance/{c,py,r}/community_voronoi/`. The runner asserts
    only on (a) exact picked-generator list and (b) distinct-community
    count — never on raw membership labels — because the inner
    `voronoi()` Voronoi-cell tiebreak is RNG-driven in the C reference
    (`MT19937`) but vertex-id-deterministic here (intentional, to keep
    the public API RNG-free).
  - File: `src/algorithms/community/community_voronoi.rs` (~750 LOC).
  - Example: `examples/community_voronoi_karate.rs`.
- **ALGO-PR-031** — `ecc` (Radicchi 2004 edge clustering coefficient).
  Counterpart of `igraph_ecc` in `references/igraph/src/properties/ecc.c`
  (lines 33-385). For each edge `(i, j)` returns
  `C^(k)_ij = (z^(k)_ij + offset) / s^(k)_ij`, where `z` counts the
  number of `k`-cycles the edge belongs to and `s` is the maximum
  number of such cycles allowed by the endpoint degrees:
  - `k = 3` (triangle): `s = min(d_i, d_j) - 1`,
  - `k = 4` (square): `s = (d_i - 1) · (d_j - 1)`.
  - `ecc(graph: &Graph, eids: Option<&[EdgeId]>, k: u32, offset: bool,
    normalize: bool) -> IgraphResult<Vec<f64>>`. `eids = None` walks
    every edge in id order; `Some(&[...])` walks just those edges, in
    the order given. `offset` toggles the canonical Radicchi `+1`;
    `normalize` toggles the division by `s`. Passing `(true, true)`
    reproduces the paper's `C^(k)_ij = (z + 1) / s` exactly.
  - Self-loop semantics match the C reference: cycle counts `z` ignore
    multi-edges and self-loops (the dedup'd simple-adjacency view is
    used), but the normaliser `s` uses the **loop-aware** degree (each
    self-loop contributes 2 to the undirected degree, i.e.
    `IGRAPH_LOOPS` mode). A self-loop edge yields `NaN` when
    normalising; any edge with `s ≤ 0` (e.g. a degree-1 endpoint of a
    star or `P_2`) also yields `NaN`.
  - Errors: `InvalidArgument` for `k < 3`, `Unsupported` for `k > 4`
    (Radicchi only defines 3 and 4), `EdgeOutOfRange` for invalid
    `eids` entries.
  - Complexity: O(E · d̄) for k=3 (one sorted-merge intersection per
    edge), O(E · d̄²) worst-case for k=4 (iterates the smaller-degree
    endpoint and intersects per intermediate). The k=4 path picks the
    smaller-degree endpoint as the iterator to keep cost bounded by
    `min(d_i, d_j)`, mirroring `igraph_i_ecc4_*`.
  - Tests: 18 unit cases + 3 proptest invariants
    (non-negative-integer `z` without normalisation; `(offset=true) -
    (offset=false) = 1` for finite edges; subset-eids preserves order
    against the full sweep) under `--features proptest-harness`.
  - Conformance: 13 fixtures across all three sources — 5 from the C
    reference (`tests/unit/igraph_ecc.out` lines 11-85: K_5 k=3 / k=4,
    K_5 + self-loops k=3 / k=4, and a multigraph), 4 hand-derived
    Python fixtures (python-igraph 0.11 has no bound `Graph.ecc()`),
    and 4 hand-derived R fixtures (R only exports the internal
    `ecc_impl`, not as a user-facing API).
  - Performance (criterion, release, `git d06e63a`): karate k=3
    `~4.1 µs`, karate k=4 `~8.5 µs`, 30×30 grid k=3 `~74 µs`. Sits in
    the same micro-bench class as `count_triangles` (`~2.7 µs` on
    karate) since both are essentially one adjlist + intersection
    sweep.
- **ALGO-SP-007** — `voronoi` (multi-source Voronoi cells via BFS or
  Dijkstra). Counterpart of `igraph_voronoi` in
  `references/igraph/src/paths/voronoi.c`. Given a set of *generator*
  vertices, assigns every other vertex to the generator from / to which
  the shortest-path distance is smallest (under [`DijkstraMode::Out` /
  `In` / `All`]) and returns the per-vertex `(generator-index, distance)`
  pair.
  - `voronoi(graph: &Graph, weights: Option<&[f64]>, mode: DijkstraMode,
    generators: &[VertexId], tiebreaker: VoronoiTiebreaker, seed: u64)
    -> IgraphResult<VoronoiPartition>` where `VoronoiPartition {
    membership: Vec<Option<u32>>, distances: Vec<f64> }`. `None` /
    `f64::INFINITY` mark vertices unreachable from every generator.
  - `VoronoiTiebreaker { First, Last, Random }` matches
    `IGRAPH_VORONOI_{FIRST,LAST,RANDOM}`. `Random` uses a self-rolled
    seeded `SplitMix64` reservoir sampler (probability `1/k` of replacing
    after the `k`-th tied generator) — no `rand` crate dependency, fully
    reproducible per `seed`.
  - Unweighted graphs take the BFS inner loop; weighted graphs take a
    binary-heap Dijkstra inner loop (same edge-weight validation as
    SP-001b: rejects `NaN` and negative weights, accepts `+∞` as
    "unusable edge"). Both inner loops share a `mindist`-aware early
    subtree prune — when expanding from generator `g`, a vertex whose
    current `mindist` already beats the distance it was reached at via
    `g` is skipped, so subtrees that are dominated by another generator
    are not explored at all. This is what beats the python emulation by
    7–10×.
  - Complexity: O(k · (V + E)) for unweighted, O(k · (V + E) log V) for
    weighted (`k` = generator count), but the inner-loop pruning makes
    it sub-additive in practice as soon as cells start overlapping.
  - Errors: empty `generators` slice → `EmptyGenerators`; out-of-range
    or duplicate generator id → `InvalidGenerator`; `weights.len() !=
    ecount()` → `InvalidLength`; `weights` containing `NaN` or a
    negative entry → `InvalidWeight`.
  - Test coverage: 15 unit tests (single-generator full-coverage, two
    generators on the path graph for both tiebreakers, weighted
    triangle with asymmetric weights, unreachable-vertex handling on a
    disconnected graph, directed in/out mode parity vs reversed graph,
    `+∞`-weight edge skipped, `Random` tiebreaker determinism for a
    fixed seed, all four error paths) + 3 proptests (membership is
    `None` ⇔ distance is `+∞`, distance(g) = 0 for every generator,
    `Last` ≥ `First` in lexicographic membership ordering on ties).
  - Three-source conformance (10 fixtures: 4 C, 3 py, 3 R) covers the
    disconnected directed multigraph from `igraph_voronoi.c` (FIRST +
    LAST), the karate club at 3 generators (FIRST + LAST), the
    path-graph endpoints split (FIRST + LAST from py + R), the karate
    club FIRST tiebreaker through python-igraph's emulated reference
    (`Graph.distances()` per generator + manual min/tiebreaker — the
    upstream python-igraph 0.11 has no bound `voronoi()`), and the
    R-igraph star-centre singleton case. `Random` tiebreaker fixtures
    are intentionally omitted (RNG divergence: C uses Mersenne Twister
    seeded 42, ours uses `SplitMix64`).
  - Bench: 4.63 µs / karate (34v 78e, 3 generators), 185.77 µs /
    weighted 30×30 grid (900v 1740e, 3 generators), 614.76 µs / same
    grid with 10 generators (LAST tiebreaker) — 10.06× / 8.18× / 7.30×
    faster than python-igraph emulated via `Graph.distances()` per
    generator. The pruning amortises across generators so even the
    10-generator dense regime stays linear in `k`.
  - Runnable example `examples/voronoi_karate.rs` runs FIRST and LAST
    tiebreakers on Zachary's karate club with generators `[0, 32, 24]`,
    prints per-cell vertex lists + distances, and confirms 29/34
    vertices stay on the same cell under both rules (the remaining 5
    are equidistant ties).

- **ALGO-CM-016** — `split_join_distance` (asymmetric projection-distance
  pair). Pure-function helper mirroring `igraph_split_join_distance` in
  `references/igraph/src/community/community_misc.c`. Returns the
  asymmetric `SplitJoinDistance { d12, d21 }` pair the C reference
  exposes alongside the symmetric scalar reported by
  `compare_communities(_, _, SplitJoin)`.
  - `split_join_distance(comm1: &[u32], comm2: &[u32]) -> IgraphResult<SplitJoinDistance>`
    with `SplitJoinDistance::total() -> u64` returning `d12 + d21`
    (matches the CM-015 `SplitJoin` scalar).
  - `d12 == 0` ⇔ `comm1` is a sub-partition of `comm2`; `d21 == 0` ⇔
    `comm2` is a sub-partition of `comm1`; both zero ⇔ identical
    partitions. The asymmetric pair preserves the refinement-relationship
    information that the symmetric scalar collapses.
  - Reuses CM-014's `reindex_membership` for densification and CM-015's
    shared `split_join_distances(pub(crate))` confusion-matrix walk
    (promoted from `fn` to `pub(crate) fn` so the sibling module imports
    it without duplicating logic). Complexity: `O(n + S)` where `S` is
    the number of observed `(comm1, comm2)` cell pairs (≤ `min(k₁·k₂, n)`).
  - 8 unit tests (empty pair, length-mismatch error, identical zero,
    full-disagreement 2x2 → `(2, 2)` summing to 4 = CM-015's `SplitJoin`,
    sub-partition asymmetry → `(0, 1)`, cross-check vs
    `compare_communities`, relabel-invariance, all-singletons) + 5
    proptests (non-negative ≤ n, total matches `compare_communities`,
    relabel-invariance under arbitrary remap, identical → `(0, 0)`,
    coarsened `b = a/2` ⇒ `d12 == 0`).
  - Three-source conformance (6 fixtures, 2 each from c/py/r) covering
    refinement and full-disagreement scenarios. Expected emitted as
    `{"d12", "d21"}`; conformance test asserts exact `u64` equality on
    both components. The R-igraph source uses `split_join_distance()`
    directly; the python-igraph entries are fixed references derived
    from the upstream confusion-matrix decomposition (python-igraph
    exposes only the symmetric scalar via
    `compare_communities(method='split_join')`).
  - Bench: small 256v/8c ~5.29µs, medium 10000v/100c ~174.74µs,
    subpartition coarsening 10000v/100c ~172.92µs — 3.31×/3.94×/3.57×
    faster than python-igraph `compare_communities(method='split_join')`.
    Speedup is consistent with CM-015 (the asymmetric pair has the same
    internal cost as the symmetric scalar; the FFI boundary dominates
    the python-side cost).
  - Runnable example
    `examples/split_join_distance_louvain_vs_leiden_karate.rs` runs
    louvain + leiden + leiden(`n_iterations=6`) on Zachary's karate
    club, prints `(d12, d21, total)` with a one-line interpretation per
    pair, demonstrates louvain vs leiden(default) yields
    `(3, 3, 6)` (neither refines the other) while leiden(default) and
    leiden(`n_iterations=6`) produce identical partitions.

- **ALGO-CM-015** — `compare_communities` (5 partition-distance metrics).
  Pure-function helper mirroring `igraph_compare_communities` in
  `references/igraph/src/community/community_misc.c`. Given two membership
  vectors of equal length, computes one of five partition-distance
  measures and returns it as a single `f64`.
  - `compare_communities(comm1: &[u32], comm2: &[u32], method: CommunityComparison) -> IgraphResult<f64>`
    with `CommunityComparison { VariationOfInformation,
    NormalizedMutualInformation, SplitJoin, Rand, AdjustedRand }`.
  - Methods: `VariationOfInformation` (Meilă 2003, `VI = H(C₁|C₂) +
    H(C₂|C₁)`, natural-log basis matching igraph C / python-igraph);
    `NormalizedMutualInformation` (Danon 2005, `2·I(C₁,C₂)/(H₁+H₂)`
    clamped to `[0,1]`; the `H₁ = H₂ = 0` degenerate case returns 1);
    `SplitJoin` (van Dongen 2000, `2n − Σᵢmax_j n_ij − Σⱼmax_i n_ij`);
    `Rand` (Rand 1971, agreeing pairs / total pairs in `[0,1]`);
    `AdjustedRand` (Hubert-Arabie 1985, `(RI − E[RI])/(max RI − E[RI])`
    with `0/0 → 1` per sklearn for single-cluster pairs).
  - Algorithm: densifies both inputs via `reindex_membership` (CM-014),
    then builds a sparse `HashMap<(u32, u32), u32>` confusion matrix
    (O(observed) memory, not dense O(k₁·k₂)). Per-method post-processing
    walks the dense `p₁`/`p₂` marginals plus a single sparse-matrix
    pass.
  - Complexity: O(n + S) where S is the number of observed sparse cells;
    allocates the two densified vectors plus the confusion-matrix
    `HashMap`.
  - Empty inputs: VI=0 / NMI=1 / SJ=0; Rand and AdjustedRand error
    (pair-count denominator is 0).
  - Test coverage: 11 unit tests (identical 6v partition with relabel,
    two-class disagreement closed-form `VI = 2·ln(2)`, four-vertex
    full disagreement closed-form `Rand = 1/3` `AR = −0.5` `SJ = 4`,
    single-cluster vs single-cluster `NMI = 1` and `AR = 1` 0/0 case,
    three-way subpartition split-join, empty input degenerate cases,
    the three error paths) + 7 proptests over 64 cases each
    (NMI ∈ [0,1], VI ≥ 0, Rand ∈ [0,1], AR ≤ 1, relabel-invariance
    under arbitrary id remap, NMI symmetric in arguments,
    identical-partition extremals: VI=0 / NMI=1 / SJ=0 / Rand=1 /
    AR=1) + 6 three-source conformance fixtures (2 each from c/py/r
    covering all 5 methods at least once; expected scalar comparison
    within 1e-9 to closed-form arithmetic; the conformance test accepts
    both `{"value": f64}` (C/py) and bare scalar (R), and both
    snake_case (`nmi`, `split_join`, ...) and Rust CamelCase
    (`NormalizedMutualInformation`, ...) method names).
  - Bench (release, criterion `--quick`, Apple Silicon): VI/NMI small
    256v/8c ~6.4 µs, medium 10000v/100c ~197 µs; Rand/AdjustedRand
    small ~5.5 µs, medium ~183 µs; SplitJoin small ~5.5 µs, medium
    ~208 µs. Vs python-igraph 0.11.9 `igraph.compare_communities`:
    2.5–3× faster across all 5 methods at both sizes (modest because
    python-igraph itself delegates to the C core; the gap mostly
    reflects FFI overhead avoided).
  - Example: `cargo run --release --example
    compare_communities_walktrap_louvain_karate` runs walktrap and
    louvain on Zachary's karate club, prints all 5 metrics for the
    walktrap-vs-louvain pair (NMI ≈ 0.82, AR ≈ 0.77) plus a
    walktrap-stability cell at steps=4 vs steps=8 (AR ≈ 0.53).

- **ALGO-CM-014** — `reindex_membership` (densify membership labels).
  Pure-function helper mirroring `igraph_reindex_membership` +
  `igraph_i_reindex_membership_large` in
  `references/igraph/src/community/community_misc.c`. Relabels a
  `membership[v] = c` vector with arbitrary `u32` ids into a contiguous
  `0..k` labelling assigned in first-occurrence order; also reports the
  original id behind each new id so callers can round-trip.
  - `reindex_membership(membership: &[u32]) -> Result<ReindexMembershipResult>` —
    `ReindexMembershipResult { membership, new_to_old }` where
    `membership[v] ∈ [0, k)` and `new_to_old[i]` is the original id that
    now maps to new id `i`; `nb_clusters() -> u32` is a convenience
    accessor.
  - Algorithm: two-branch single pass. Fast path when `max_id < n` uses
    a flat `Vec<u32>` lookup with 0 as the "unseen" sentinel and stores
    `new_id + 1` so the zero-init is reusable. Sparse fallback uses a
    `BTreeMap<u32, u32>` keyed by old cluster id when ids overflow the
    `[0, n)` window so the O(n) flat-Vec path stays bounded. Both
    branches preserve first-occurrence ordering.
  - Complexity: O(n) average / O(n log k) sparse; allocates one
    `Vec<u32>` of size `n` (output) plus the lookup and the `new_to_old`
    vector of size `k`.
  - Infallible today; returns `IgraphResult` for API symmetry with
    other community helpers so future fallible checks can land without
    a breaking change.
  - Test coverage: 11 unit tests (empty, identity, singleton collapse,
    gaps compressed, reverse-order, all-singletons, large-id triggers
    BTreeMap, fast-path edge `max == n−1`, sparse-path edge
    `max == n`, partition-preserved over 3 mixed inputs including
    `u32::MAX`, fast-path-equals-sparse-path bookkeeping check) +
    4 proptests over 80 cases each (partition preserved across all
    id-kind branches, ids contiguous in `[0, k)`, `new_to_old`
    round-trips so `r.new_to_old[r.membership[i]] == input[i]`,
    idempotent when already dense) + 6 three-source conformance
    fixtures (2 each from c/py/r, partition-equivalence comparison via
    canonical first-occurrence relabel + `new_to_old` round-trip check
    because cluster numbering can differ between C/py/R impls).
  - Bench (release, criterion 1s warmup / 2s measurement / 20 samples,
    Apple Silicon): fast 256v/8c 0.32 µs, fast 1024v/32c 0.98 µs,
    fast 10000v/100c 8.23 µs, sparse 1024v/32c 5.00 µs,
    sparse 10000v/100c 63.44 µs. Vs python-igraph 0.11.9
    (`VertexClustering(g, membership=...).membership`): 320×/160×/107×
    faster on the fast path; 32×/13× on the sparse path.
  - Example: `cargo run --release --example reindex_membership_walktrap_karate`
    builds a messy karate membership with cluster ids 1_000 / 4_242 /
    9_999 (forcing the sparse branch), densifies to `0..k-1`, and
    verifies modularity Q is preserved bit-for-bit before and after.

- **ALGO-CM-013** — `community_to_membership` (binary dendrogram → membership).
  Pure-function helper mirroring `igraph_community_to_membership` in
  `references/igraph/src/community/community_misc.c`. Cuts a binary
  dendrogram at `steps` merges and returns a densified `0..k` membership
  vector plus per-cluster size vector.
  - `community_to_membership(merges, nodes, steps) -> Result<CommunityToMembershipResult>` —
    `merges[i] = [c1, c2]` combines dendrogram nodes `c1` and `c2` into
    `nodes + i`; leaves are `0..nodes`.
  - `CommunityToMembershipResult { membership, csize }` — `membership[v] ∈ [0, k)`,
    `csize[c]` is the number of vertices in cluster `c`.
  - Algorithm: walks merges top-down (`steps-1 → 0`) assigning 1-based
    supercluster ids via a `tmp` slot per row, marks `already_merged`
    to detect double-merges, propagates ids down through internal
    nodes; final pass densifies leaves with cid==0 as fresh singletons.
    O(steps + n) total work; allocates a `Vec<u32>` of size `n`, a
    `Vec<bool>` of size `n + steps`, and a `Vec<u32>` of size `steps`.
  - Errors: `InvalidArgument` when `steps > merges.len()`, when a
    dendrogram node is merged twice, when a row references a node
    `≥ nodes + i` (i.e. itself or a future merge), or on usize
    overflow of `nodes + steps`.
  - Test coverage: 11 unit tests (zero steps, full collapse,
    intermediate cut, untouched leaves, chained merges, empty graph,
    all 3 error paths, partial dendrogram, end-to-end against
    `walktrap` on two K4 + bridge) + 3 proptests (`zero_steps` yields
    singletons, `full_collapse` yields one cluster, cut invariants:
    csize sums to n, cluster count = n − steps, every membership entry
    < cluster count) + 6 three-source conformance fixtures (2 each
    from c/py/r, partition-equivalence comparison because cluster
    labels can differ between implementations).
  - Bench (release, criterion 1s warmup / 2s measurement / 20 samples,
    Apple Silicon): chain-64 full collapse 470 ns,
    balanced-256 full collapse 1.7 µs, balanced-1024 full collapse
    6.6 µs, balanced-1024 half cut 2.95 µs, balanced-1024 zero
    steps 1.26 µs. Vs python-igraph 0.11.9
    (`VertexDendrogram.as_clustering`): ~28–30× faster on full
    collapses, ~106× on a half cut, ~342× at zero steps.
  - Example: `cargo run --release --example community_to_membership_walktrap_karate`
    drives Walktrap on the karate club then re-cuts the resulting
    dendrogram at 0, 8, 16, 29, 33 steps and reports modularity at
    each depth.

- **ALGO-CO-008** — Walktrap community detection (Pons P., Latapy M.
  2005, *Computing communities in large networks using random walks*).
  Random walks of length t (default 4) define a between-vertex distance
  r²_{ij} = Σ_k (P^t_{ik} − P^t_{jk})²/deg(k); the agglomerative loop
  greedily merges the pair minimising Δσ — the increase in
  within-community squared distance — and the dendrogram step with the
  best Newman modularity Q is returned as the final partition.
  - `walktrap(graph) -> Result<WalktrapResult>` — unweighted, steps=4.
  - `walktrap_weighted(graph, weights) -> Result<WalktrapResult>` —
    edge weights enter both the transition matrix (P_{ij} = w_{ij}/s_i,
    where s_i is the weighted degree including the synthesized
    self-loop) and the Newman-Girvan Q used for the best-cut choice.
  - `walktrap_with_options(graph, weights, opts)` — full control over
    `steps` (1..=u32::MAX; default `WALKTRAP_DEFAULT_STEPS = 4`).
  - `WalktrapResult { membership, nb_clusters, merges, modularity }`
    mirrors `igraph_community_walktrap`: dendrogram `[c1, c2]` rows for
    the merge tree, full Q trajectory from singletons to one community,
    and a densified `[0, k)` best-Q membership.
  - Internal graph adapter synthesizes a per-vertex self-loop with
    weight = mean incident edge weight (1.0 if isolated) — same fix as
    upstream issue #2043 — so the random-walk probability vectors
    handle leaf/isolated vertices without numerical pathology.
  - Hand-rolled binary min-heap with a lazy-refine pattern (entries
    with `!exact` are popped, refined via the triangle Δσ formula or a
    chain lower bound, marked exact, re-inserted) — no external heap
    dependency, no `unsafe`. Triangle update:
    `Δσ_new = ((s₁+s_k)·d₁ + (s₂+s_k)·d₂ − s_k·Δσ_old) / (s₁+s₂+s_k)`;
    chain fallback: `Δσ_new = −1 / min(|adj(new)|, |adj(k)|)`.
  - Modularity trajectory matches the C `community_walktrap.out`
    reference (triangle / bug-2042 / 6-ring weighted / isolated) to
    ~1e-15; conformance tolerance is 1e-12.
  - Tests (CO-008): 13 unit tests + 2 proptests cover the four C
    reference cases (exact match), error cases (directed input,
    `steps=0`, NaN/negative/wrong-length weights), edge cases
    (empty/single-vertex), and structural cases (two K4 bridge,
    multi-edge folding). Three-source conformance test covers Zachary
    karate (Q ∈ [0.30, 0.45], k ∈ [3, 6]), K5+K5+bridge / two-K4-bridge
    (k=2), ring-of-4-cliques (k=4), and the weighted 6-ring (k=3,
    Q ≈ 0.146) across `c/`, `py/`, `r/`.
  - Bench (`benches/bench_walktrap.rs`): triangle ≈ 11 µs, ring-6
    weighted ≈ 26 µs, ring-of-cliques 4×5 ≈ 262 µs, karate ≈ 932 µs.
    Rust beats python-igraph at the smallest scales (triangle ~2×,
    ring-6 weighted ~7×); python-igraph still wins on karate /
    ring-of-cliques (~3-5×) — gap is from dense Vec<f64> probability
    vectors (vs the C reference's sparse Map switchover) and per-merge
    Δσ recomputation; tracked for alpha.3.
  - Example: `examples/walktrap_karate.rs` (karate club → 4
    communities at Q ≈ 0.420).

- **ALGO-EIG-001/002/003** — Eigenvalue solvers cluster:
  `eigen_matrix_symmetric` (Lanczos), `eigen_matrix` (Arnoldi),
  `eigen_adjacency` (adjacency spectrum wrapper). Supports
  `LargestAlgebraic`, `SmallestAlgebraic`, `LargestMagnitude` selection.

- **Core ergonomics** — `Graph::from_edges(&[(u32,u32)], directed, n_override)`
  constructor infers vertex count from the highest endpoint. `Display` impl
  for `Graph` shows `"Directed/Undirected graph with N vertices and M edges"`.

### Changed
- Eliminated all `unwrap()`/`expect()` from production code — library is now
  fully panic-free outside tests.
- Added doctests to every public `Graph` method and all 400+ public API
  functions (test count: 640 → 661).
- Fixed stale README examples to match current API signatures.
- Removed stale `#[allow(dead_code)]` from Lanczos module.

### Added
- **ALGO-CO-006c** — Directed Girvan-Newman edge betweenness community
  detection (Leicht E. A., Newman M. E. J. 2008, *Community structure in
  directed networks*, Phys. Rev. Lett. 100, 118703 for the directed
  modularity objective; Girvan M., Newman M. E. J. 2002 / Newman M. E.
  J., Girvan M. 2004 for the underlying edge-removal framework). Both
  existing entrypoints now accept directed graphs:
  - `edge_betweenness_community(graph) -> Result<EdgeBetweennessResult>`
    and `edge_betweenness_community_weighted(graph, weights) ->
    Result<EdgeBetweennessResult>` — the per-removal Brandes /
    Brandes-Dijkstra pass now walks `Graph::incident(_, Out)` on the
    forward sweep, the back-pass is driven by predecessor lists
    populated during the forward sweep, and the per-pair edge
    contribution is **not halved** for directed graphs (matching the C
    reference rule `if (!directed) eb /= 2.0;`).
  - Per-level modularity dispatch: directed graphs use
    `modularity_directed` (unweighted) or `modularity_weighted_directed`
    (weighted), so the best-Q cut reflects the Leicht-Newman 2008
    directed objective `Σ_c [e_{cc}/m − (s^{out}_c/m)·(s^{in}_c/m)]`
    rather than the symmetric undirected sum.
  - New public function `modularity_weighted_directed(graph, membership,
    resolution, weights) -> Result<Option<f64>>` — directed-aware
    weighted modularity, falling through to `modularity_weighted` when
    the graph is undirected so callers have a single weighted entry.
  - Same `EdgeBetweennessResult` shape, same per-vertex `Vec<EdgeId>`
    masking pattern, same lowest-eid tie-break (documented under
    3-way Brandes ties via a smoke test).
  - Tests: directed 4-path eb=4.0 (un-halved) sanity, directed 6-path
    0→1→2→3→4→5 cuts the middle directed edge first (Q=8/25=0.32, k=2),
    directed two-triangles+bridge runs cleanly under lowest-eid tie
    break; weighted unit-weights through the directed entry reproduce
    the unweighted directed dendrogram bit-for-bit (proptest).
  - Three-source conformance: 3 directed_path_6 fixtures from each of
    C / Python / R (Q ∈ [0.31, 0.33], k = 2) for both algos.
  - Bench: directed-path-10 ~16.4 µs / directed-two-triangles-bridge
    ~8.3 µs (unweighted), directed-path-10 unit ~11.6 µs /
    directed-two-triangles-cheap-bridge ~7.5 µs (weighted). Undirected
    cells unchanged.

- **ALGO-CO-006b** — Weighted edge betweenness community detection
  (Newman M. E. J., Girvan M. 2004, *Finding and evaluating community
  structure in networks*, Phys. Rev. E 69, 026113 — weighted variant of
  the Girvan-Newman 2002 algorithm). New public entrypoint:
  - `edge_betweenness_community_weighted(graph, weights) ->
    Result<EdgeBetweennessResult>` — reuses the same
    `EdgeBetweennessResult { membership, nb_clusters, removed_edges,
    edge_betweenness, merges, bridges, modularity }` as the unweighted
    CO-006 slice. Accepts an explicit `&[f64]` of length `ecount`;
    rejects directed graphs with `IgraphError::Unsupported(...)`
    (delegated to ALGO-CO-006c) and rejects any NaN / negative /
    non-finite weight with `IgraphError::InvalidArgument(...)`.
  - Per removal: weighted Brandes pass over Dijkstra shortest paths
    (`BinaryHeap<Frontier>` min-heap; `Frontier` ordering uses
    `f64::total_cmp` after weight validation so the heap stays
    deterministic) restricted to the *active* edge mask. Tie-break is
    the smallest active edge id, mirroring the upstream linear scan.
    `edge_betweenness[i]` is the weighted betweenness of the *i*-th
    removed edge at the moment of removal (halved for undirected to
    match the centrality convention).
  - Stage 2 replays removals in reverse to build the binary dendrogram
    and recomputes modularity at every merge via the standalone
    `modularity_weighted` (so `m = Σ w_e`, not `m = ecount`); best-Q
    membership is densified to `0..nb_clusters` and returned.
  - Edges are masked via private per-vertex `Vec<EdgeId>` incidence
    lists with `retain`, never via `graph.delete_edges`, so the
    original `EdgeId`s stay stable across the whole run and callers
    can replay any cut from `removed_edges`.
  - Invariant: with `weights = [1.0; ecount]` this entrypoint must
    reproduce the unweighted CO-006 dendrogram bit-for-bit (membership,
    nb_clusters, removed_edges, merges identical; modularity within
    1e-9). Enforced by 3 integration tests
    (`unit_weights_match_unweighted_on_karate`,
    `unit_weights_match_unweighted_on_two_triangles_bridge`,
    `unit_weights_match_unweighted_path_5`) and 2 proptests on
    `arb_graph(10)` (`edge_betweenness_community_weighted_unit_matches_unweighted`
    + `edge_betweenness_community_weighted_deterministic`).
  - Cheap-bridge fixtures: two K4 + bridge w=0.1 (the bridge sits on
    every cross-clique shortest path → carries the largest weighted
    betweenness → first removed, K4s stay internally connected at the
    best cut, k = 2); two K3 + bridge w=0.1 (analogous on triangles);
    karate weighted-Q ≥ 0.35 self-consistent with `modularity_weighted`
    of the returned partition.
  - Complexity: `O(|V| · |E| · (|E| + |V| log |V|))` — the per-removal
    Dijkstra-Brandes pass dominates.
  - Test surface: 11 unit tests in module + 13 integration tests in
    `tests/edge_betweenness_community_weighted.rs` + 2 proptests in
    `tests/property.rs` + 6 conformance fixtures across C/py/R
    (`tests/conformance/{c,py,r}/edge_betweenness_community_weighted/`).
    Conformance test exercises the Q envelope + k range via
    `modularity_weighted` of the returned partition (same shape as the
    fast_greedy and unweighted-EB conformance oracles).
  - Bench (`benches/bench_eb_community_weighted.rs`, Apple Silicon):
    path-10 unit ~35.7 µs / two-K4-bridge unit ~36.4 µs /
    ring-of-cliques(4×5) unit ~218 µs / karate unit ~1.76 ms; baseline
    via `python-igraph 0.11.9
    Graph.community_edge_betweenness(weights=[1.0]*ecount)` — Rust
    runs karate in ~79% of python-igraph wall time and matches
    ring-of-cliques(4×5) within noise. ~2.6× slower than the
    unweighted CO-006 on karate as expected from the Dijkstra-vs-BFS
    heap overhead. Perf snapshot:
    `.codefuse/tracking/perf/ALGO-CO-006b.json`.
  - Example: `examples/eb_community_weighted_karate.rs` runs the karate
    fixture once with unit weights, once with a tilted weight vector,
    and prints the best partition + first few removals.

- **ALGO-CO-007** — Fast greedy modularity community detection
  (Clauset A., Newman M. E. J., Moore C. 2004, *Finding community
  structure in very large networks*, Phys. Rev. E 70, 066111). Two
  public entrypoints sit on a shared kernel:
  - `fast_greedy_modularity(graph) -> Result<FastGreedyResult>` —
    unweighted convenience (every edge counted with weight 1).
  - `fast_greedy_modularity_weighted(graph, weights) ->
    Result<FastGreedyResult>` — accepts an explicit `&[f64]` of
    length `ecount` (non-negative; finite). Both reject directed
    graphs and multigraphs with `IgraphError::InvalidArgument(...)`
    matching the upstream C contract.
  - `FastGreedyResult { membership, nb_clusters, merges, modularity }`
    carries the best-modularity dense-labelled partition
    (`0..nb_clusters`), the full binary merge dendrogram
    (`merges[i] = [c1, c2]` produces the new super-cluster id
    `n + i` in classical igraph dendrogram encoding), and the
    per-step modularity trajectory of length `merges.len() + 1`
    (`modularity[0]` = singleton Q, `modularity[i+1]` = Q after the
    *i*-th merge), so callers can re-cut the dendrogram at any
    level.

  Numerical contract: each community keeps a
  `BTreeMap<u32 community_id, f64 ΔQ>` of its alive neighbours;
  a global lazy-deletion `BinaryHeap<HeapEntry { dq, c1, c2 }>`
  drives the merge order. On each iteration we pop the entry with
  the largest ΔQ, validate that *both* endpoints are still alive
  and that the stored ΔQ still matches the live BTreeMap value
  (stale entries from previous updates are silently discarded),
  merge `c2` into `c1`, update `a[c1] += a[c2]`, then for every
  shared neighbour `k` apply the three Clauset-Newman-Moore update
  rules — Triangle `ΔQ'(c1,k) = ΔQ(c1,k) + ΔQ(c2,k)`, Chain-1
  `ΔQ'(c1,k) = ΔQ(c1,k) − 2·a[c2]·a[k]`, Chain-2
  `ΔQ'(c1,k) = ΔQ(c2,k) − 2·a[c1]·a[k]` — mirror the updated value
  back into `k`'s BTreeMap, and push the fresh entry onto the heap.
  Modularity per merge is tracked incrementally as
  `Q += 2·ΔQ_merged`, matching standalone `modularity(g, &mem, 1.0)`
  at every dendrogram cut to floating-point exactness. Best-Q
  membership is densified (first-appearance reindex to
  `0..nb_clusters`) and returned. Complexity is
  `O(|V|·|E|·log²|V|)` worst-case — the lazy-heap approach trades
  one log factor versus the upstream Wakita-Tsurumi indexed heap
  for safe-Rust simplicity; at Phase-1 graph scales the constant
  factor stays well under python-igraph's Python overhead.

  Best-Q values (all match upstream igraph C unit-test numbers
  bit-for-bit on the best dendrogram cut):
  - karate (`fixtures/karate.edges`, 34v 78e): Q = 0.380671, k = 3
  - two-K5-bridge (10v 21e): Q = 0.452381, k = 2
  - K4+K4+isolate (9v 12e): Q = 0.5, k = 3
  - two-disjoint-10-rings (20v 20e): Q = 0.54, k = 4
  - 6v8e small undirected (the upstream unit-test exemplar):
    Q = 0.179688
  - 2v with two self-loops: Q = 0.5

  Test surface: 12 unit tests (in-module) + 14 integration tests
  (`tests/fast_greedy_modularity.rs`) cover the six Q-value
  exemplars above plus error paths (directed rejection /
  multi-edge rejection / negative-weight rejection / weight-length
  mismatch / non-finite weight), uniform-unit-weight ≡ unweighted
  agreement, dendrogram total-merges invariant, and
  modularity-trajectory monotone-up-to-best-cut. Two proptests
  (`tests/property.rs`):
  `fast_greedy_modularity_partition_well_formed` checks label
  contiguity, merge well-formedness (`c1 ≠ c2`, both `< n + i`),
  and Q ↔ `modularity()` agreement within 1e-9 on `arb_graph(12)`;
  `fast_greedy_modularity_deterministic` checks bit-for-bit
  reproducibility on `arb_graph(10)`. Three-source conformance: 2
  fixtures each from C / py / R under
  `tests/conformance/{c,py,r}/fast_greedy_modularity/` — karate +
  two-K5-bridge — asserting Q ∈ [q_min, q_max] and `nb_clusters`
  ∈ [k_min, k_max] (envelope form because the upstream C and
  python-igraph implementations sometimes choose a marginally
  different best-Q cut on graphs with flat-top modularity
  trajectories, but the dendrogram itself is identical).

  Bench (`benches/bench_fast_greedy.rs`): path-10 ~2.54 µs,
  two-K5-bridge ~4.72 µs, ring-of-cliques(4×5) ~10.33 µs,
  karate ~36.57 µs on darwin-aarch64. python-igraph 0.11.9's
  `Graph.community_fastgreedy().as_clustering()` baseline:
  karate ~45.17 µs (Rust ~81% wall time), ring-of-cliques(4×5)
  ~33.67 µs (Rust ~3.3× faster), two-K5-bridge ~22.69 µs
  (Rust ~4.8× faster). Numbers checked into
  `.codefuse/tracking/perf/ALGO-CO-007.json`.

  Runnable demo (`examples/fast_greedy_karate.rs`): loads
  `fixtures/karate.edges`, prints the best-Q partition
  (k = 3, Q = 0.380671), the first five merges with the
  modularity after each, and where in the dendrogram the
  best-Q cut sits. Run with
  `cargo run --example fast_greedy_karate`.

- **ALGO-CO-006** — Edge betweenness community detection
  (Girvan M., Newman M. E. J. 2002, *Community Structure in Social and
  Biological Networks*, PNAS 99 (12) 7821–7826). One public entrypoint:
  - `edge_betweenness_community(graph) -> Result<EdgeBetweennessResult>`
    — undirected, unweighted Phase-1 slice. Directed graphs are
    rejected with
    `IgraphError::Unsupported("directed edge_betweenness_community is ALGO-CO-006c; not yet ported")`
    as a follow-up AWU.
  - `EdgeBetweennessResult { membership, nb_clusters, removed_edges,
    edge_betweenness, merges, bridges, modularity }` carries the
    best-modularity dense-labelled partition (`0..nb_clusters`),
    the full edge-removal order (length = `ecount`) and the
    betweenness of each edge at the moment it was removed (halved
    for undirected to match the centrality convention), plus the
    binary merge dendrogram (`igraph_community_eb_get_merges()`
    encoding: cluster IDs `[0, n)` are vertex-singletons and each
    merge produces the new cluster id `n + merge_index`), the
    "bridges" indices into `removed_edges` where a removal first
    disconnects a component, and the per-merge modularity
    trajectory (so callers can replay any cut point).

  Numerical contract: each outer iteration runs a Brandes
  unweighted edge-betweenness pass *over the active edge set*
  (BFS layering → reverse dependency accumulation), picks the edge
  with the largest current betweenness with ties broken by smallest
  edge id (matching upstream `igraph_i_which_max_active_ratio`),
  records it, and masks it from both endpoints' incidence lists. The
  removal loop never mutates the original `Graph` — masking happens
  in private per-vertex `Vec<EdgeId>` lists via `retain` — so the
  original `EdgeId`s remain stable for the entire run and can be
  replayed by callers. Stage 2 walks `removed_edges` in reverse and
  rebuilds membership by re-joining components: each removal that
  re-joins two distinct components is a *merge*, modularity is
  recomputed via the standalone `modularity()` function at every
  merge, and the partition with the highest Q is densified (first-
  appearance reindex to `0..nb_clusters`) and returned. The returned
  `modularity` value matches a fresh `modularity(graph, &membership, 1.0)`
  to 1e-9 on every test graph. Complexity is the canonical
  `O(|V|·|E|²)` — the per-removal Brandes pass dominates.

  Test surface: 9 unit tests + 12 integration tests
  (`tests/edge_betweenness_community.rs`) cover empty graph /
  edgeless / single-vertex / two-K4-bridge (bridge removed first)
  / karate (best Q ≥ 0.35 with `modularity()` self-consistency) /
  ring-of-4-K5-cliques (recovers k=4) / path-5 (first removal is a
  middle edge) / cycle-4 (modularity monotone at singletons) /
  already-disconnected components / determinism (repeated calls
  bit-identical) / dendrogram total-merges invariant / directed
  rejection. Two proptests (`tests/property.rs`):
  `edge_betweenness_community_partition_well_formed` checks label
  contiguity, edge-id uniqueness, and Q ↔ `modularity()` agreement
  within 1e-9 on `arb_graph(12)`;
  `edge_betweenness_community_deterministic` checks bit-for-bit
  reproducibility on `arb_graph(10)`. Three-source conformance: 2
  fixtures each from C / py / R under
  `tests/conformance/{c,py,r}/edge_betweenness_community/` —
  karate / two-K4-bridge / K5+K5+bridge / ring-of-4-K5-cliques,
  asserting Q ∈ [q_min, q_max] and `nb_clusters` ∈ [k_min, k_max]
  (envelope form because tie-breaking can drift across upstreams).

  Bench (`benches/bench_eb_community.rs`): path-10 ~13.5 µs,
  two-K4-bridge ~14.0 µs, ring-of-cliques(4×5) ~115.5 µs,
  karate ~998 µs on darwin-aarch64. python-igraph 0.11.9's
  `Graph.community_edge_betweenness()` baseline:
  karate ~1400 µs, ring-of-cliques(4×5) ~128 µs — Rust runs the
  karate fixture in ~71% of python-igraph's median wall time and
  is within noise on the ring-of-cliques. Numbers checked into
  `.codefuse/tracking/perf/ALGO-CO-006.json`.

  Runnable demo (`examples/eb_community_karate.rs`): loads
  `fixtures/karate.edges`, prints the best-Q partition (k = 5,
  Q ≈ 0.40), the first five edge removals with their betweenness
  at removal, and the tail of the merge dendrogram with per-merge
  Q. Run with `cargo run --example eb_community_karate`.

- **ALGO-CO-005** — Fluid Communities community detection (Parés F.,
  Gasulla D.G. *et al.* 2018, *Fluid Communities: A Competitive,
  Scalable and Diverse Community Detection Algorithm*). Two public
  entrypoints sit on a shared kernel:
  - `fluid_communities(graph, k) -> Result<FluidResult>` —
    convenience: `seed = 0`, `max_iterations =
    FLUID_DEFAULT_MAX_ITERATIONS` (1000).
  - `fluid_communities_with_options(graph, k, &FluidOptions) ->
    Result<FluidResult>` — full control via `FluidOptions { seed,
    max_iterations }`.
  - `FluidResult { membership, nb_clusters, n_iterations_run }`
    exposes the final dense-labelled partition (`0..k`), the
    actual community count (usually exactly `k`, but a community
    can in principle vanish), and how many outer iterations the
    convergence loop spent.

  Numerical contract: the algorithm seeds `k` fluids at the first
  `k` vertices of a shuffled order, each with density `1/size`. Each
  iteration re-shuffles the visit order and re-evaluates every
  vertex's label by summing the density contributions of itself and
  its neighbours, picking the dominant label with an ε = 1e-4 tie
  band and uniform random tie-break. The vertex's *current* label is
  pre-loaded into the dominant set so that ties retain it — matching
  upstream's same-label-stickiness. A vanished community recovers
  automatically through IEEE-754 `1/0 = +inf` arithmetic: the spike
  pulls a neighbour back on the next sweep. Iterations stop when no
  vertex changes label, or at `max_iterations`. Validation rejects
  `k = 0`, `k > vcount`, non-simple graphs (self-loops / parallel
  edges) and disconnected graphs.

  Determinism: SplitMix64 + Fisher-Yates duplicated within the
  module (each AWU keeps its own copy to stay independent of the
  others). Identical `(graph, k, seed)` triples produce bit-for-bit
  identical membership vectors.

  Test surface: 14 unit tests + 15 integration tests
  (`tests/fluid_communities.rs`) cover k=1/k=n/two-K4-bridge/
  ring-of-4-K5-cliques/empty/single-vertex/all four error paths/
  determinism/max-iterations cap. Two proptests
  (`tests/property.rs::fluid_partition_well_formed`,
  `fluid_determinism_under_seed`). Three-source conformance: 2
  fixtures each from C / py / R (karate k=2, two-K4-bridge / K5+K5
  bridge / ring-of-4-K5-cliques k=4) verifying Q ∈ [q_min, q_max]
  and the user-pinned `k`.

  Bench (`benches/bench_fluid.rs`): karate k=2 ~12.8 µs, karate k=4
  ~9.6 µs, karate k=3 fixed seed ~9.6 µs, ring-of-cliques(8×10) k=8
  ~44.6 µs on darwin-aarch64. python-igraph 0.11.9 does not expose
  `community_fluid_communities`, so no cross-language baseline was
  captured.

- **ALGO-CO-004** — Label propagation community detection
  (Raghavan, Albert, Kumara 2007 *Near linear time algorithm to detect
  community structures in large-scale networks*; with the Traag &
  Šubelj 2023 *Fast LPA* improvement). Three public entrypoints sit
  on a shared variant-dispatched kernel:
  - `label_propagation(graph) -> Result<LpaResult>` — undirected,
    unit weights, Fast variant (Traag-Šubelj 2023), seed `0`.
  - `label_propagation_weighted(graph, weights) -> Result<LpaResult>`
    — weighted variant with the same defaults; validates
    `weights.len() == ecount()` and rejects negative / non-finite
    weights.
  - `label_propagation_with_options(graph, weights, &LpaOptions) ->
    Result<LpaResult>` — full control via `LpaOptions`: `variant`
    (`LpaVariant::Fast` / `Dominance` / `Retention`), `seed`
    (SplitMix64), optional `initial` membership (with `-1` flagging
    an unlabelled vertex), optional `fixed` mask for semi-supervised
    LPA where flagged vertices keep their initial label.
  - `LpaResult { membership, nb_clusters }` exposes the final
    partition (dense labels in `0..k`) and the community count;
    quality is reported via standalone `modularity()` since LPA does
    not optimise an objective.

  Numerical contract: self-loops follow the **IGRAPH_LOOPS_ONCE**
  convention so they contribute exactly once to a vertex's own label
  sum (matching upstream `igraph_community_label_propagation`); after
  the main loop a final dense-relabel + BFS-fill pass ensures every
  unlabelled connected component receives its own fresh contiguous
  label. The Fast variant uses a queue + in-queue bitmap to converge
  in a single sweep over the work-set; Dominance alternates
  control/update iterations until no vertex's label flips; Retention
  keeps the current label when it is still tied for the dominant
  weight, otherwise picks a random majority-label neighbour.

  Determinism: `label_propagation_with_options` is fully reproducible
  for a fixed `(graph, weights, options)` tuple — same SplitMix64 +
  Fisher-Yates shuffle story as Louvain / Leiden.

  Conformance & tests: 16 unit tests + 15 integration tests covering
  all three variants, weight validation, seed determinism, fixed
  vertices preserving co-membership, unlabelled isolates becoming
  singletons, and the full error surface; plus 6 three-source
  conformance fixtures (2 each from the C, Python and R upstream
  test suites) with a Q-range + k-window oracle that tolerates
  shuffle-order drift across implementations; 4 proptest invariants
  gating partition well-formedness, unit-weighted vs unweighted
  equivalence, seed determinism across all three variants, and the
  no-unlabelled-left guarantee.

  Benchmark vs python-igraph (release build): karate Fast 3.9 µs vs
  python-igraph 17.5 µs (4.49× faster); karate Dominance 8.8 µs;
  karate Retention 6.1 µs; ring-of-cliques(8×10) Fast 19.2 µs vs
  24.3 µs (1.27× faster). See `.codefuse/tracking/perf/ALGO-CO-004.json`.

  Example: `cargo run --example lpa_karate` demonstrates all three
  variants on the karate club graph with the modularity score
  computed from the partition.

- **ALGO-CO-003** — Leiden community detection (Traag, Waltman, van Eck
  2019 *From Louvain to Leiden: guaranteeing well-connected
  communities*), the second Phase-4 algorithm. Three public entrypoints
  sit on a shared three-phase loop (fastmove → refinement →
  aggregation):
  - `leiden(graph) -> Result<LeidenResult>` — undirected, unit
    weights, modularity objective with `γ = 1.0`, `β = 0.01`, two
    iterations, seed `0`.
  - `leiden_weighted(graph, weights) -> Result<LeidenResult>` —
    weighted variant with the same defaults; validates `weights.len()
    == ecount()` and rejects negative / non-finite weights for the
    Modularity and ER objectives (CPM allows negative weights).
  - `leiden_with_options(graph, weights, &LeidenOptions) ->
    Result<LeidenResult>` — full control via `LeidenOptions`:
    `objective` (`Modularity` / `Cpm` / `Er`), `resolution` (γ),
    `beta` (refinement randomness, default
    `LEIDEN_DEFAULT_BETA = 0.01`), `n_iterations` (negative ⇒
    iterate until stable; default `LEIDEN_DEFAULT_ITERATIONS = 2`),
    `seed` (SplitMix64), `start` (optional initial membership).
  - `LeidenResult { membership, quality, nb_clusters,
    n_iterations_run, qualities }` exposes the final partition (dense
    labels in `0..k`), the chosen objective's value, the community
    count, the number of outer iterations actually executed, and the
    per-iteration quality history.

  Numerical contract: local-moving computes the chosen objective's
  gain in the generic Reichardt-Bornholdt form
  `Q = (1/2m) Σ_c (e_c − γ · N_c²)`; refinement is a singleton-init
  pass with merges sampled with probability `∝ exp(diff/β)` over the
  non-negative diffs (the Leiden-vs-Louvain fix that guarantees
  connected, well-separated communities). Aggregation is the standard
  Leiden novelty: super-vertices are the **refined** subclusters (not
  the original clusters); the initial partition of the super-graph
  maps each refined subcluster back to its parent cluster, so later
  iterations can recover the splits introduced by refinement.
  Self-loops follow the **IGRAPH_LOOPS** convention (matching upstream
  `igraph_community_leiden`), so on loop-free graphs Leiden's internal
  Q equals the standalone `modularity()` to f64 precision.

  Determinism: `leiden_with_options` is fully reproducible for a fixed
  `(graph, weights, options)` tuple — same SplitMix64 + Fisher-Yates
  story as Louvain.

  Conformance & tests: 25 integration tests covering all three
  objectives, weight validation, seed determinism, start-membership
  honouring, n_iterations<0 stable-until-no-change semantics, and the
  full error surface; plus 6 three-source conformance fixtures (2 each
  from the C, Python and R upstream test suites) with a Q-range +
  k-window oracle that tolerates shuffle-order drift across
  implementations; 4 proptest invariants gating partition shape,
  unit-weighted vs unweighted equivalence, seed determinism, and CPM
  γ→∞ ⇒ singletons.

  Benchmark vs python-igraph (release build): karate unweighted
  37.5 µs vs python-igraph 56.0 µs (1.49× faster); karate weighted
  43.9 µs vs 46.5 µs (1.06×); ring-of-cliques(8×10) 93.4 µs vs
  78.7 µs (0.84× — hand-tuned igraph C `igraph_community_leiden`
  still leads on the larger fixture). See
  `.codefuse/tracking/perf/ALGO-CO-003.json` for the next-step
  optimisation levers.

  Example: `cargo run --example leiden_karate` demonstrates all three
  objectives on the karate club graph, including the `modularity()`
  cross-check.

- **ALGO-CO-002** — Louvain multilevel community detection
  (Blondel, Guillaume, Lambiotte, Lefebvre 2008 *fast unfolding*),
  the first Phase-4 algorithm to land. Three public entrypoints sit
  on a shared pass-loop / aggregation kernel:
  - `louvain(graph) -> Result<LouvainResult>` — undirected, unit
    weights, default seed (`0`), resolution `γ = 1.0`.
  - `louvain_weighted(graph, weights) -> Result<LouvainResult>` —
    weighted undirected variant; validates `weights.len() == ecount()`
    and rejects negative or non-finite values up-front.
  - `louvain_with_options(graph, weights, resolution, seed) ->
    Result<LouvainResult>` — full control over γ and the SplitMix64
    seed driving the per-pass node shuffle.
  - `LouvainResult { membership, modularity, modularities, levels }`
    exposes the final partition, its internal Q (matches the
    standalone `modularity()` to `1e-9` by construction), and the
    per-level history (super-graph snapshots one entry per pass).

  Numerical contract: the per-pass local-moves loop computes the
  modularity gain in the same form as upstream
  `community_multilevel.c`, accepting only strictly improving moves.
  Self-loops follow the **IGRAPH_LOOPS_TWICE** convention through
  every level — a loop of weight `w` contributes `2w` to `k_v` both
  at level 0 and in every aggregated super-graph, so the cross-check
  vs `modularity()` is exact regardless of whether the input graph
  starts with loops. The aggregation step re-uses
  `Graph::init_from_edges` shared with level-0 initialisation, so
  there is exactly one code path for parallel-edge collapsing.

  Determinism: `louvain_with_options` is fully reproducible for a
  fixed `(graph, weights, resolution, seed)` tuple. The SplitMix64
  PRNG + Fisher-Yates shuffle gives stable iteration order across
  platforms; the convenience entrypoints `louvain` and
  `louvain_weighted` pin `seed = 0`.

  Errors: rejects directed inputs (`SemanticError("undirected only")`),
  weight/ecount mismatches, negative weights, non-finite weights,
  negative/non-finite γ. Empty graphs return an empty membership and
  `modularity == 0.0`; isolated vertices remain singletons.

  Tests: 19 integration tests in `tests/louvain.rs` cover karate
  (Q ≈ 0.39..0.42, k ≈ 2..8), ring-of-cliques(4×5) (Q > 0.60, k = 4),
  weighted-equals-unweighted under unit weights, heavy/thin bridge
  splits, seed-determinism, γ extremes, every documented error path,
  empty / isolated / self-loop graphs, and the level-history
  invariant. 4 proptest invariants
  (`tests/property.rs::louvain_*`) generalise the well-formedness,
  non-decreasing per-level Q, unit-weighted equivalence, and
  seed-determinism guarantees. Conformance: 6 fixtures
  (C:2, py:2, R:2) at `tests/conformance/{c,py,r}/louvain/*.json`
  with a Q-range + k-window oracle, since Louvain output varies by
  shuffle order across implementations.

  Bench (`benches/bench_louvain.rs`, criterion):
  - karate (34 V, 78 E, unweighted)   — 16.8µs, **2.73×** vs python-igraph
  - karate (34 V, 78 E, unit-weighted) — 12.1µs, **2.47×**
  - karate, fixed-seed (`with_options`) — 14.8µs
  - ring-of-cliques 8×10 (80 V, 288 E) — 29.0µs, **3.61×**

  Snapshot: `.codefuse/tracking/perf/ALGO-CO-002.json`. Runnable demo
  at `examples/louvain_karate.rs` (loads `fixtures/karate.edges`,
  runs Louvain, cross-checks vs `modularity()`, prints per-level Q
  and community memberships).

- **ALGO-PR-012b** — directed + weighted eigenvector centrality. Adds
  four public functions and two types built on a self-rolled
  shifted-power-iter kernel:
  - `EigenvectorMode { Out, In, All }` — selects which arc direction
    is followed for the dominant left/right eigenvector of the
    adjacency matrix (matches upstream's `IGRAPH_REVERSE_MODE`
    semantics at the directed entrypoint).
  - `EigenvectorScores { vector, eigenvalue, options }` — return type
    parallel to `HitsScores`.
  - `eigenvector_centrality_weighted(graph, weights) -> EigenvectorScores`
    — undirected weighted, validates `weights.len() == ecount()` and
    short-circuits all-zero weights / empty edges to ones.
  - `eigenvector_centrality_directed(graph, mode) -> EigenvectorScores`
    — directed unweighted, runs shifted power-iter on the adjacency
    walked in `mode`. For DAGs (non-negative weights) the spectrum
    collapses to zero, so we return a sentinel: ones on sinks (Out) /
    sources (In), with `eigenvalue = 0.0`.
  - `eigenvector_centrality_directed_weighted(graph, mode, weights)
    -> EigenvectorScores` — directed weighted; same shifted-pivot
    kernel parameterised by per-edge weights.
  - `eigenvector_centrality_full(graph, mode, weights) -> EigenvectorScores`
    — single master entrypoint that dispatches to the right
    undirected/directed × unweighted/weighted leaf based on
    `graph.is_directed()` and `weights`.

  Numerical contract: iterates on `(M + σI)` where
  `σ = max_row_norm + 1`. For a non-negative `M` Perron-Frobenius
  guarantees the largest-real eigenvalue is real and non-negative;
  the shift then makes it the **unique** largest-magnitude eigenvalue
  of `M + σI`, so plain max-norm power-iter converges without ARPACK.
  The dominant eigenvalue of `M` itself is recovered via the Rayleigh
  quotient `xᵀMx / xᵀx` after vector convergence. For the
  negative-weight branch we track the signed-pivot component of
  greatest magnitude (mirrors C's `which='LA'`). Tolerance for
  conformance against ARPACK is `1e-9`.

  Counterpart of the weighted + directed branches in
  `references/igraph/src/centrality/eigenvector.c`. 20 unit tests
  (lib) + 18 integration tests (`tests/eigenvector_weighted.rs`,
  `tests/eigenvector_directed.rs`) + 2 proptest invariants
  (weighted-unit parity with unweighted; directed-finite + max-1
  normalisation + non-negative λ) + 6 three-source conformance
  fixtures (1 C / 1 py / 1 R per signature, including the
  cycle-with-chord ARPACK golden at 16-digit precision and the
  DAG out-star sentinel) under
  `tests/conformance/{c,py,r}/eigenvector_centrality_{weighted,directed}/`.
  Bench at `benches/bench_eigenvector.rs`, perf snapshot at
  `.codefuse/tracking/perf/ALGO-PR-012b.json`: karate undirected
  ~15µs (unit) / ~28µs (varied), directed ring(500) ~17µs unweighted
  / ~6.2ms weighted.

- **ALGO-PR-017b** — `hub_and_authority_scores_weighted(graph, weights)
  -> HitsScores` (Kleinberg HITS, weighted). Builds the weighted matrix
  `W[i,j] = Σ_{e: i→j} w_e` implicitly: each power-iter step walks
  every incident edge with multiplicative `w_e * h[other]`, avoiding
  any dense materialisation. On directed graphs, runs power iteration
  on `W·Wᵀ` with hub seeded from weighted out-strength (falling back
  to out-degree when negative weights are present, matching the
  upstream contract). On undirected graphs, runs a self-rolled shifted
  power-iter on `W + I` and recovers the dominant eigenvalue via the
  Rayleigh quotient — this private helper will be promoted to a public
  `eigenvector_centrality_weighted` once PR-012b lands. For non-negative
  weights, sign cleanup keeps both vectors elementwise `>= 0`; with
  negative weights we track the signed component of greatest magnitude
  (matches C's `which='LA'`). Validates `weights.len() == ecount()` and
  returns `InvalidArgument` otherwise. Empty-edge graphs and all-zero
  weights both fill both vectors with `1.0` and report eigenvalue `0.0`.
  Counterpart of the weighted branch in
  `references/igraph/src/centrality/hub_authority.c` (lines 130-176,
  333-505). 10 unit tests + 8 integration tests in
  `tests/hits_weighted.rs` (parity with unweighted under unit weights
  on a directed chain + karate, both cross-relations
  `hub ∝ W·authority` and `authority ∝ Wᵀ·hub`, length-mismatch error,
  positive-only non-negativity, empty/all-zero sentinels) + 3
  three-source conformance fixtures (1 C / 1 py / 1 R, each a
  two-hubs-one-authority configuration with closed-form expected
  values) under
  `tests/conformance/{c,py,r}/hub_and_authority_scores_weighted/`.

- **ALGO-PR-017** — `hub_and_authority_scores(graph) -> HitsScores`
  (Kleinberg HITS, unweighted). On directed graphs, runs power
  iteration on `A·Aᵀ`: hub vector seeded from out-degrees, max-norm
  rescaling each iteration, eigenvalue read off as `max|A·Aᵀ·h|` at
  convergence; `authority` recovered as `Aᵀ·h` then max-normed.
  Sources have authority `0`, sinks have hub `0` by construction.
  On undirected graphs, delegates to `eigenvector_centrality` per the
  upstream contract; reported `eigenvalue` is the squared dominant
  adjacency-matrix eigenvalue. Empty-edge directed graphs fill both
  vectors with `1.0` and report eigenvalue `0.0`. Counterpart of
  `igraph_hub_and_authority_scores()` from
  `references/igraph/src/centrality/hub_authority.c` (unweighted slice;
  ARPACK weighted variant ships with PR-017b/c). 10 unit tests + 6
  integration tests in `tests/hits.rs` (including a karate-club
  identity check `hub == authority == eigenvector_centrality`) + 6
  three-source conformance fixtures (2 C / 2 py / 2 R) under
  `tests/conformance/{c,py,r}/hub_and_authority_scores/`. Runnable
  example at `examples/hits_karate.rs`.

- **ALGO-CORE-001f** — Boolean property cache subsystem.
  - New `core::cache::CachedProperty` enum (7 variants: `HasLoop`,
    `HasMulti`, `HasMutual`, `IsWeaklyConnected`, `IsStronglyConnected`,
    `IsDag`, `IsForest`) and bit-packed `PropertyCache` (interior-mut
    via `Cell<u32>`) on every `Graph`. Counterpart of
    `igraph_i_property_cache_t` from `references/igraph/src/graph/caching.{c,h}`.
  - `Graph::cache_get` / `cache_set` / `cache_invalidate` /
    `cache_invalidate_all` public helpers — compute functions
    consult and populate the cache without holding `&mut Graph`,
    matching igraph C's "compute-is-not-modification" semantics.
  - Selective invalidation policies (`invalidate_after_add_vertices`,
    `invalidate_after_add_edges`) preserve cached values that the
    mutation provably cannot change — e.g. adding an edge keeps
    `IS_DAG=false`, and adding isolated vertices keeps every edge-level
    property. Mirrors `type_indexededgelist.c:341-364`. Deletes do a
    full `invalidate_all` (conservative).
  - `is_dag`, `is_forest` (in `All` mode), `has_loop`, `has_multiple`
    now read-or-compute through the cache → repeated calls are O(1).
  - 12 integration tests in `tests/cache.rs` covering hit/miss across
    every mutation path; 11 unit tests on the bitfield logic.

### Changed
- *(ci)* Coverage + test analytics moved out of `.github/workflows/ci.yml`
  into a dedicated `.github/workflows/coverage.yml`. The new workflow
  runs two jobs in parallel and both upload to Codecov:
  1. `llvm-cov` (line coverage via `cargo-llvm-cov` → `lcov.info`),
  2. `test-results` (JUnit XML via `cargo-nextest --profile ci` →
     `target/nextest/ci/junit.xml`, consumed by Codecov Test Analytics).
  The `ci.yml` workflow stays Python-free and fast; coverage / test
  analytics own their own python-igraph venv setup. New nextest profile
  lives in `.config/nextest.toml`.

### Added
- *(properties)* **ALGO-PR-030**: `local_efficiency(graph) -> Vec<f64>`
  and `average_local_efficiency(graph) -> f64` — per-vertex local
  efficiency (Vragović–Louis–Díaz-Guilera, 2005) and its mean. For
  each vertex `v`, the local efficiency is the average inverse
  shortest-path distance between every ordered pair of distinct
  neighbours of `v`, computed in the subgraph `G \ {v}` (paths must
  not pass through `v`); pairs unreachable in `G \ {v}` contribute 0.
  Vertices with fewer than two unique non-self neighbours yield 0 by
  upstream convention. `average_local_efficiency` returns 0 when
  `vcount < 3`. Implemented as one BFS per source-neighbour with `v`
  excluded from traversal — O(V·|N(v)|·(V+E)) overall. Counterpart of
  `igraph_local_efficiency()` and `igraph_average_local_efficiency()`
  from `references/igraph/src/paths/shortest_paths.c:688-867` (the
  `directed=true, mode=OUT` slice exposed in this Phase). Coverage: 16
  unit tests (empty/singleton/n<3 zero baseline; isolated; path-3
  zero; triangle 1.0 at every vertex; K4 1.0 at every vertex; star
  zero at every vertex; diamond [5/6, 1, 5/6, 1]; self-loops + parallel
  edges collapse correctly; `average_local_efficiency` empty/n<3=0,
  K4=1, diamond=11/12, path-4=0), 8 conformance fixtures (C: K4 +
  path-3 for `local_efficiency`, K4 + diamond for
  `average_local_efficiency`; R: triangle + star K_{1,3} for
  `local_efficiency`, triangle + path-4 for `average_local_efficiency`;
  py-skipped — python-igraph 0.11 exposes no `local_efficiency` API),
  and 1 proptest invariant (each per-vertex value in `[0, 1]`; length
  matches `vcount`; mean matches `average_local_efficiency`; vertices
  with `<2` unique non-self neighbours contribute 0).
- *(properties)* **ALGO-PR-029**: `global_efficiency(graph) ->
  Option<f64>` — Latora–Marchiori average inverse pairwise distance,
  defined as `(1/(n*(n-1))) * sum_{i!=j} 1/d(i,j)`, where `d(i,j)` is
  the unweighted shortest-path distance and unreachable pairs
  contribute `0`. Returns `None` when `vcount() < 2` (no ordered pair
  exists), and lies in `[0, 1]` for graphs with at least one edge.
  Implemented as one BFS per source via the existing
  `distances()` primitive — the same primitive used by
  `mean_distance` (PR-003) and structurally identical to
  `harmonic_centrality(graph).iter().sum::<f64>() / n` (PR-009)
  modulo the leading `1/n` normalisation, which is asserted as a
  proptest invariant. Counterpart of `igraph_global_efficiency()` from
  `references/igraph/src/paths/shortest_paths.c:392-486`. Coverage:
  12 unit tests (empty / singleton → `None`; K3 = K4 = 1.0; path-3 =
  5/6; path-4 = 13/18; star K_{1,3} = 0.75; disconnected two-island =
  1/3; directed-path OUT-only; result ∈ [0,1]; matches harmonic
  average), 6 conformance fixtures (C: K4 + path-3; py: star K_{1,3} +
  disconnected; R: triangle + undirected path-3), and 1 proptest
  invariant (`None` iff `vcount < 2`; otherwise value ∈ [0,1] and
  agrees with mean of `harmonic_centrality` to 1e-9).
- *(properties)* **ALGO-PR-002d**: `count_adjacent_triangles(graph) ->
  Vec<u64>` — per-vertex adjacent-triangle count, completing the
  triangle / transitivity family alongside PR-002 (scalar
  `count_triangles` + global `transitivity_undirected`), PR-002b
  (`transitivity_local_undirected`) and PR-002c (`transitivity_barrat`).
  Thin wrapper exposing the existing `per_vertex_triangle_stats` helper:
  for each vertex `v` it returns the number of triangles containing
  `v`. Self-loops and parallel edges are ignored (the simple graph is
  used). Counterpart of `igraph_count_adjacent_triangles()` from
  `references/igraph/src/properties/triangles.c:522`. Coverage: 11 unit
  tests (empty / isolated vertices / single triangle / K4 / diamond /
  star / self-loops / parallel edges / two disjoint triangles / sum
  invariant / consistency with `transitivity_local_undirected`), 6
  conformance fixtures (C: K4 + K4-minus-edge; py: triangle +
  star-no-triangles; R: two-disjoint-triangles + 4-path), and one
  proptest invariant (length matches `vcount`, sum equals
  `3 * count_triangles`, each entry ≤ `C(simple_degree, 2)`).
- *(properties)* **ALGO-PR-014c**: `count_loops(graph) -> usize` and
  `count_multiple(graph) -> Vec<usize>`, completing the loops /
  multiplicity trio alongside PR-014 (`has_*` predicates) and PR-014b
  (`is_*` per-edge boolean vectors). `count_loops` is a linear scan
  counting edges where `from == to`; `count_multiple` returns each
  edge's multiplicity (the size of its endpoint-pair equivalence class)
  via O(|E| log |E|) sort-and-group. Storage canonicalises undirected
  pairs to `(min, max)`, so undirected `(a,b)` and `(b,a)` collapse
  correctly; directed pairs stay ordered. Self-loops at the same
  vertex are grouped per upstream's `IGRAPH_LOOPS_ONCE` semantics — each
  loop's multiplicity is the number of loops at that vertex.
  Counterpart of `igraph_count_loops()` from
  `references/igraph/src/properties/loops.c:137` and
  `igraph_count_multiple()` from
  `references/igraph/src/properties/multiplicity.c:313`. Coverage:
  12 unit tests (empty/no-loop/no-multi/mixed cases, parallel /
  loop-grouping, directed mutual-pair distinction, length and
  consistency invariants vs `is_loop` / `is_multiple` /
  `has_multiple`), 12 conformance fixtures (C/py/R, 4 each — two for
  `count_loops`, two for `count_multiple`; `count_multiple` fixtures
  store pre-sorted multisets since wire-format edge ids permute), and
  3 proptest invariants (`count_loops` agrees with `is_loop` count;
  `count_multiple` per-edge ≥ 1 and `> 1` agrees with `has_multiple`;
  directed-graph variant of the latter).
- *(properties)* **ALGO-PR-028**: `convergence_degree(graph)` and
  `convergence_degree_full(graph)` — per-edge convergence value in
  `[-1, 1]` (directed) or `[0, 1]` (undirected) measuring whether the
  shortest paths through the edge originate from a larger or smaller
  vertex set than they terminate in. For each edge `e`,
  `In(e)` counts source vertices whose BFS reaches `e`'s tail-side
  endpoint and `Out(e)` counts vertices whose IN-BFS reaches `e`'s
  head-side endpoint; the result is `(In − Out) / (In + Out)` (with
  the absolute value taken in the undirected variant per upstream).
  `_full` additionally returns the raw `In` / `Out` counts for
  diagnostics, mirroring python-igraph's `convergence_field_size`.
  Edges that lie on no shortest path produce `NaN` (matches upstream's
  `0/0` semantics). Counterpart of `igraph_convergence_degree` from
  `references/igraph/src/properties/convergence_degree.c:21`.
  Algorithm: directed graphs run two BFS-per-source passes (OUT pass
  feeds `ins`, IN pass feeds `outs`); undirected graphs use a single
  pass and decide tree-edge orientation by endpoint comparison
  (`actnode < neighbor` ⇒ `ins`, else `outs`). Both upstream `.out`
  reference cases (n=7 undirected two-triangle, n=6 directed star)
  reproduced bit-for-bit in unit tests. Coverage: 17 unit tests
  (upstream cases, K_4 / K_3 symmetric, balanced cycles, isolated
  vertices, parallel edges, single-edge sanity, NaN self-loop), 3
  oracle tests against python-igraph (with dummy weights vector to
  pin stored-edge-order encoding through the wire), 6 conformance
  fixtures (C/py/R, 2 each), and 3 proptest invariants
  (result length matches ecount; values within `[-1,1]` or `[0,1]`;
  `convergence_degree` agrees with `convergence_degree_full().0`).
  `O(V·(V+E))` per call.

- *(properties)* **ALGO-PR-027b**: `neighborhood(graph, order)` and
  `neighborhood_with_mode(graph, order, mode, mindist)` — k-hop
  neighbourhood vertex lists for every vertex (sibling of PR-027's
  size variant). For each source `v` returns a `Vec<u32>` of vertices
  `w` with `mindist <= dist(v, w) <= order`, in BFS visitation order;
  with `mindist = 0` the source vertex is the first element. Mode +
  mindist semantics identical to PR-027 and reuse the same parameter
  validation (`mindist < 0` and finite `mindist > order` both yield
  `InvalidArgument`). Counterpart of `igraph_neighborhood` from
  `references/igraph/src/properties/neighborhood.c:208`. Coverage:
  20 unit tests covering the full C reference fixture
  (`igraph_neighborhood.c` .out file, set-equality comparison),
  3 oracle tests, 6 conformance fixtures (C/py/R, 2 each, sorted
  lists), and 5 proptest invariants (list length == neighborhood_size,
  mindist=0 includes self, mindist=1 excludes self, IDs in range and
  unique, monotone-in-order set inclusion). Reuses the BFS-per-source
  marker array approach from PR-027 with vertex push instead of size
  increment. `O(V·(V+E))` per call.

- *(properties)* **ALGO-PR-027**: `neighborhood_size(graph, order)` and
  `neighborhood_size_with_mode(graph, order, mode, mindist)` —
  k-hop neighbourhood size for every vertex. For each vertex `v`
  returns the number of vertices `w` with `mindist <= dist(v, w) <=
  order`. Counterpart of `igraph_neighborhood_size` from
  `references/igraph/src/properties/neighborhood.c:70`. New
  `NeighborhoodMode` enum (`Out` / `In` / `All`) mirrors
  `igraph_neimode_t` and is ignored on undirected graphs. Negative
  `order` is treated as infinity (every reachable vertex within
  `mindist`+ is counted). Validation: `mindist < 0` and (finite)
  `mindist > order` both yield `InvalidArgument`. Algorithm: direct
  BFS-per-source port of the C reference, using an integer
  "added by source `i+1`" marker array to avoid per-source
  re-allocation. Loops and multi-edges are tolerated and do not
  inflate the count. Coverage: 24 unit tests covering the entire
  C reference fixture (`igraph_neighborhood_size.c` .out file) +
  the python `Ring(10)` fixture + 3 oracle tests +
  6 conformance fixtures (C/py/R, 2 each) + 4 proptest invariants
  (order-0 returns 1, monotone non-decreasing, bounded by vcount,
  mindist-1 + 1 = mindist-0). `O(V·(V+E))` BFS-per-vertex.

- *(properties)* **ALGO-PR-016**: `is_complete(graph: &Graph) ->
  IgraphResult<bool>` — true iff every distinct pair of vertices
  is adjacent. Counterpart of `igraph_is_complete` from
  `references/igraph/src/properties/complete.c:43`. Null and
  singleton graphs are complete (matches upstream convention).
  Algorithm: cardinality short-circuit (`ecount` vs.
  `n*(n-1)` directed / `n*(n-1)/2` undirected), simple-graph
  fast path that returns `ecount == target` directly, and a
  unique-neighbour scan for graphs that have loops or parallel
  edges padding the edge count. On directed graphs both arcs
  must be present for every pair (calls `is_simple_with_mode(_,
  DirectedAsDirected)` exactly like the C reference uses
  `IGRAPH_DIRECTED`). Coverage: 20 unit tests + 3 oracle tests
  cross-checked against `python-igraph`'s `Graph.is_complete()` +
  6 conformance fixtures (C/py/R, 2 each) + 2 proptest
  invariants (`complete_simple_graph_has_full_edge_count` and
  `complete_implies_every_vertex_sees_n_minus_1`). `O(V + E)`
  worst case.

- *(properties)* **ALGO-PR-024**: `is_forest(graph: &Graph, mode:
  DijkstraMode) -> IgraphResult<Option<Vec<VertexId>>>` —
  mode-aware forest predicate. Counterpart of `igraph_is_forest`
  from `references/igraph/src/properties/trees.c:520`. Returns
  `Some(roots)` iff the graph is a forest (a disjoint union of
  trees) under `mode`, otherwise `None`. Unlike [`is_tree`], the
  null graph **is** considered a forest with empty roots
  (matches upstream's convention). For undirected graphs the
  mode argument is ignored; for directed graphs:
  - `DijkstraMode::Out`: out-forest — every tree is an
    out-arborescence; roots are vertices with in-degree 0.
  - `DijkstraMode::In`: in-forest — every tree is an
    in-arborescence; roots are vertices with out-degree 0.
  - `DijkstraMode::All`: orientation ignored; roots are the
    canonical (lowest-id) vertex of each connected component.
  Implementation pairs a fast `ecount ≤ vcount - 1` cardinality
  bound with a per-component DFS visitor that detects cycles by
  popping a vertex that's already marked visited (mirrors
  upstream's `igraph_i_is_forest_visitor`). Self-loops in the
  undirected case are caught explicitly. Time `O(V + E)`.
  17 unit tests + 3 oracle tests vs an inline Python reference
  (python-igraph does not expose this predicate; the oracle
  replicates the C contract for both `mode=all` and `mode=out`)
  + 6 conformance fixtures (2 each C/Python/R, mixing modes and
  true/false outcomes; payload shape `{is_forest, roots[]}`)
  + 2 proptest invariants
  (`is_forest_implies_acyclic_with_forest_edge_count` — every
  forest under `All` has a root per CC and satisfies the
  identity `m == n - cc`; `tree_is_forest_with_one_root` —
  every tree is a forest with exactly the tree root).
- *(properties)* **ALGO-PR-023**: `is_tree(graph: &Graph, mode:
  DijkstraMode) -> IgraphResult<Option<VertexId>>` — mode-aware
  tree predicate. Counterpart of `igraph_is_tree` from
  `references/igraph/src/properties/trees.c:251`. Returns
  `Some(root)` iff the graph is a tree under `mode`, otherwise
  `None`. The null graph (`vcount == 0`) is by convention **not**
  a tree. For undirected graphs the mode argument is ignored and
  the canonical root is `0`; for directed graphs:
  - `DijkstraMode::Out`: out-arborescence — every edge points
    away from the root (the unique vertex with in-degree 0).
  - `DijkstraMode::In`: in-arborescence — every edge points
    towards the root (the unique vertex with out-degree 0).
  - `DijkstraMode::All`: orientation ignored; canonical root 0.
  Single-vertex graphs are trees in every mode. The implementation
  combines a fast `ecount == vcount - 1` check with a DFS
  reach-all pass from the canonical root, which together imply
  acyclicity and connectedness in `O(V + E)`.
  15 unit tests + 3 oracle tests vs `python-igraph`'s
  `Graph.is_tree(mode=...)` (mode='all' for undirected path/
  triangle, mode='out' for an out-arborescence) + 6 conformance
  fixtures (2 each C/Python/R, mixing modes and true/false
  outcomes) + 2 proptest invariants
  (`is_tree_implies_acyclic_connected_and_correct_edge_count` —
  whenever `is_tree(All)` is `Some`, the graph is acyclic, has a
  single connected component, and `m == n - 1`;
  `directed_out_tree_is_undirected_tree` — every out-tree is an
  undirected tree).
- *(properties)* **ALGO-PR-022**: `is_acyclic(graph: &Graph) ->
  bool` — generic acyclic predicate. Counterpart of
  `igraph_is_acyclic` from
  `references/igraph/src/properties/trees.c:753`. For directed
  graphs delegates to [`crate::is_dag`]; for undirected graphs
  runs a union-find pass over the edge list and returns false as
  soon as an edge re-connects two already-unioned vertices.
  Self-loops and parallel undirected edges count as cycles.
  Time `O(V + E·α(E))`.
  13 unit tests + 3 oracle tests vs an inline Python reference
  (python-igraph does not expose this directly; mirrors the
  upstream contract) + 6 conformance fixtures (2 each C/Python/R)
  + 2 proptest invariants
  (`is_acyclic_matches_is_dag_for_directed`,
  `is_acyclic_undirected_implies_forest_edge_count` — every
  acyclic undirected graph satisfies the forest identity
  `m == n - cc`).
- *(properties)* **ALGO-PR-021**: `topological_sorting(graph,
  mode)` — returns a topological ordering of a directed graph's
  vertices. Counterpart of `igraph_topological_sorting` from
  `references/igraph/src/properties/dag.c:54`. Same Kahn-peel
  inner loop as [`crate::is_dag`] but recording each popped vertex
  in the output. Uses `DijkstraMode` to mirror the IN/OUT mode
  selection (ALL rejected with `InvalidArgument`).
  Key contract difference from `is_dag`: **self-loops are
  ignored** when computing degrees (matches upstream's
  `IGRAPH_NO_LOOPS` flag). A vertex with only a self-loop can
  still be sorted; only non-loop cycles cause the function to
  error.
  12 unit tests + 3 oracle tests vs `python-igraph`'s
  `Graph.topological_sorting()` (one direct-equality on a
  uniquely-ordered chain; one partial-order check on a diamond
  DAG since tie-breaking may differ) + 6 conformance fixtures
  (2 each C/Python/R, all on uniquely-ordered cases for
  element-wise comparison) + 1 proptest invariant
  (`topological_sorting_respects_every_directed_edge` —
  partial-order respect, permutation, and consistency with
  `is_dag`/SCC structure).
- *(properties)* **ALGO-PR-020**: `is_dag(graph: &Graph) -> bool`
  — directed acyclic graph predicate. Counterpart of `igraph_is_dag`
  from `references/igraph/src/properties/dag.c:151`. Returns
  `false` for undirected graphs (matches upstream — DAGs are
  directed by definition); for directed graphs, runs Kahn's
  topological peel: queue zero-in-degree vertices, pop and
  decrement their out-neighbours' in-degrees, queue any that hit
  zero. If all vertices peel away the graph is a DAG; otherwise
  a cycle remains. Self-loops short-circuit to `false`.
  Time `O(V + E)`. The C version caches the result on the graph
  object; our Rust impl recomputes on each call until the
  property-cache subsystem (CORE-001f) lands.
  12 unit tests + 3 oracle tests vs `python-igraph`'s
  `Graph.is_dag()` + 6 conformance fixtures (2 each C/Python/R) +
  1 proptest invariant (`is_dag_consistent_with_scc` — every DAG
  has SCC count equal to vcount and no self-loops; every non-DAG
  either has a self-loop or a multi-vertex SCC).
- *(operators)* **ALGO-CORE-001e**: `is_same_graph(g1: &Graph, g2:
  &Graph) -> bool` — structural equality on labelled vertex/edge
  sets. Counterpart of `igraph_is_same_graph` from
  `references/igraph/src/graph/type_indexededgelist.c:1947`. Two
  graphs are "the same" iff they have the same vertex count, the
  same directedness, and the same edge multiset (regardless of
  insertion order; for undirected, endpoint orientation doesn't
  matter). Distinct from isomorphism — vertex labels matter.
  Algorithm: canonicalise both edge lists (undirected edges are
  already stored with `from <= to` by `Graph::add_edge`; directed
  pairs already canonical), lex-sort, compare. Time `O(E log E)`
  — slightly worse than upstream's `O(E)` walk over pre-sorted
  index vectors (private to `Graph`), well under a second up to a
  few million edges.
  12 unit tests + 3 oracle tests vs an inline Python reference
  (python-igraph does not expose this predicate) + 6 conformance
  fixtures (2 each C/Python/R, with `params.other` carrying the
  second graph payload) + 1 proptest invariant
  (`is_same_graph_reflexivity_and_symmetry` — reflexivity,
  symmetry, vcount-bump breaks equality).
  Row split: CORE-001e (this) covers the `is_same_graph` slice;
  remaining property-cache subsystem moves to CORE-001f (todo).
- *(connectivity)* **ALGO-CC-032**: Site percolation
  (`site_percolation(graph: &Graph, vertex_order: &[VertexId]) ->
  IgraphResult<SitePercolation>`). Counterpart of
  `igraph_site_percolation` from
  `references/igraph/src/connectivity/percolation.c:328`. Activates
  sites (vertices) in the given order; each activation step adds
  every edge that now connects two activated vertices, unioning
  the corresponding trees in a shared union-find.
  Output struct `SitePercolation` carries `giant_size[i]` (size of
  the largest component after the i-th vertex activates) and
  `edge_count[i]` (cumulative count of edges activated through step
  i). Self-loops on the just-activated vertex contribute 2 to
  `edge_count` (loop appears twice in the all-neighbor walk per
  `IGRAPH_LOOPS` semantics); parallel edges each count separately
  per `IGRAPH_MULTIPLE`. Edge direction is ignored (percolation is
  pure connectivity).
  Validates up front: ids in range and no duplicates. `vertex_order`
  may be a strict subset — unlisted vertices stay inactive.
  11 unit tests + 3 oracle tests vs an inline Python union-find
  reference + 6 conformance fixtures (2 each C/Python/R) +
  1 proptest invariant
  (`site_percolation_monotone_and_matches_components`: both curves
  monotone non-decreasing; final giant after activating every
  vertex equals the full-graph largest CC).
  **Percolation series complete** — CC-030 (edgelist) + CC-031
  (bond) + CC-032 (site) cover the upstream `percolation.c` module
  in full.
- *(connectivity)* **ALGO-CC-031**: Bond percolation
  (`bond_percolation(graph: &Graph, edge_order: &[EdgeId]) ->
  IgraphResult<EdgelistPercolation>`). Counterpart of
  `igraph_bond_percolation` from
  `references/igraph/src/connectivity/percolation.c:214`. Thin
  wrapper over [`edgelist_percolation`] that resolves the given
  edge ids into `(u, v)` pairs through the graph before delegating
  to the union-find core. Edge direction is ignored (matches
  upstream).
  Validates the order up front: every id must be `< graph.ecount()`
  and must not repeat (duplicates return
  `IgraphError::InvalidArgument`, out-of-range returns
  `IgraphError::EdgeOutOfRange`). The order does not have to cover
  every edge — pass a subset to percolate just a slice. Unlike the
  C version, no random shuffle option is exposed — callers who
  want randomness shuffle the order with their own RNG and pass it
  in; this keeps the API deterministic and dependency-free.
  8 unit tests + 3 oracle tests that resolve the order Rust-side
  and cross-check against the [`edgelist_percolation`] oracle
  (avoids python-igraph's undirected edge canonicalisation) +
  6 conformance fixtures (2 each C/Python/R) + 1 proptest
  invariant (`bond_percolation_natural_order_matches_edgelist` —
  natural id order produces the same curves as the equivalent
  direct edgelist call).
- *(connectivity)* **ALGO-CC-030**: Edge-list percolation
  (`edgelist_percolation(edges: &[(VertexId, VertexId)]) ->
  IgraphResult<EdgelistPercolation>`). Counterpart of
  `igraph_edgelist_percolation` from
  `references/igraph/src/connectivity/percolation.c:105`. Given a
  sequence of vertex-pair edges, returns two parallel curves:
  `giant_size[i]` (size of the largest connected component after
  edge `i` is added) and `vertex_count[i]` (cumulative count of
  distinct vertices touched by any edge up through `i`). Classic
  network-resilience / phase-transition primitive.
  Algorithm: union-find with path compression (`links[a] =
  links[links[a]]`) and union-by-size. Time complexity
  `O(|E| · α(|E|))` where `α` is the inverse Ackermann function.
  Vertex ids are inferred from the edge list (implicit vcount =
  max id + 1). Self-loops and parallel edges are tolerated.
  10 unit tests + 2 oracle tests vs an inline Python union-find
  reference (python-igraph does not bind percolation) + 6
  conformance fixtures (2 each C/Python/R, hand-computed) +
  1 proptest invariant
  (`edgelist_percolation_monotone_and_matches_components` —
  monotonicity of both curves; final `giant_size` matches the
  largest CC restricted to touched vertices).
  Order-sensitive note: the oracle passes the edge sequence
  through `params` (not `g.es`) because python-igraph reorders
  edges internally; the conformance runner reads
  `case.graph.edges` directly to preserve JSON insertion order.
- *(paths)* **ALGO-SP-014**: Widest-paths SPT sidecar
  (`widest_paths(graph, from, weights) -> IgraphResult<WidestPaths>`
  plus the mode-aware `widest_paths_with_mode`). Counterpart of
  `igraph_get_widest_paths(_, NULL, NULL, source, vss_all(),
  weights, mode, parents, inbound_edges)` from
  `references/igraph/src/paths/widest_paths.c:102`. Returns a
  `WidestPaths` struct exposing all three SPT outputs in one call:
  `widths`, `parents` (predecessor vertex per node), and
  `inbound_edges` (the edge id each vertex was reached through) —
  mirroring `DijkstraPaths`'s shape so callers don't have to re-run
  the SPT loop for the parent-pointer view.
  Implementation reuses `widest_inner` and derives `parents` from
  `inbound_edges` via `graph.edge_other(eid, v)`. Source itself
  has `widths[source] == Some(f64::INFINITY)`,
  `parents[source] == None`, `inbound_edges[source] == None`.
  Unreachable vertices: all three fields `None`. Disambiguate
  source vs unreachable via the `widths` field.
  7 unit tests + 2 oracle tests vs an inline Python reference +
  6 conformance fixtures (2 each C/Python/R, hand-computed) +
  1 proptest invariant (`widest_paths_spt_consistent_with_widths`:
  fields agree, walking back via `parents` reaches source in ≤ vcount
  steps with distinct vertices).
- *(paths)* **ALGO-SP-013**: Multi-target widest paths
  (`widest_paths_to(graph, from, targets, weights) ->
  IgraphResult<Vec<WidestPathResult>>` plus the mode-aware
  `widest_paths_to_with_mode`). Counterpart of
  `igraph_get_widest_paths` from
  `references/igraph/src/paths/widest_paths.c:102`. Returns one
  `Option<(vertices, edges)>` per element of `targets`, in the
  same order. SP-011's single-target `widest_path` is now a thin
  wrapper around the shared `widest_inner` SPT loop and the new
  `reconstruct_one` helper; this AWU exposes the multi-target
  variant. Duplicate target ids are allowed (each gets the same
  path); self-targets return the trivial `(vec![from], vec![])`.
  Added `pub type WidestPathResult = Option<(Vec<VertexId>,
  Vec<EdgeId>)>` to keep the public signature readable (clippy's
  `type_complexity` lint).
  8 unit tests + 2 oracle tests vs an inline Python reference +
  6 conformance fixtures (2 each C/Python/R, hand-computed since
  neither python-igraph nor rigraph bind widest paths) +
  1 proptest invariant (`widest_paths_to_consistent_with_widths` —
  reachability and chain bottleneck match the single-call API).
- *(paths)* **ALGO-SP-012**: All-pairs widest-path widths via
  Floyd-Warshall (`widest_path_widths_floyd_warshall(graph, weights)
  -> IgraphResult<Vec<Vec<Option<f64>>>>` plus the mode-aware
  `widest_path_widths_floyd_warshall_with_mode`). Counterpart of
  `igraph_widest_path_widths_floyd_warshall` from
  `references/igraph/src/paths/widest_paths.c:451`. Returns the
  `vcount × vcount` bottleneck-width matrix between every pair of
  vertices. Useful on **dense** graphs; for sparse graphs running
  `widest_path_widths` from every source is asymptotically the same
  in practice but avoids the V³ wall.
  Algorithm: standard FW shape with the widest-paths recurrence
  `M[i][j] = max(M[i][j], min(M[i][k], M[k][j]))`. Diagonal is
  `Some(f64::INFINITY)` (source to self). Mode controls how
  directed edges seed the matrix: OUT populates `M[s][t]`, IN
  populates `M[t][s]`, ALL does both. On undirected graphs every
  mode collapses to ALL. Parallel edges merge by wider-wins;
  `-f64::INFINITY` weights are ignored.
  8 unit tests + 2 oracle tests vs an inline FW Python reference +
  6 conformance fixtures (2 each C/Python/R, hand-computed) +
  1 proptest invariant (`fw_widest_matches_pairwise_dijkstra` —
  every row matches the Dijkstra-based `widest_path_widths` from
  that source).
- *(paths)* **ALGO-SP-011**: Single-source single-target widest
  path (`widest_path(graph, from, to, weights) ->
  IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>>` plus the
  mode-aware `widest_path_with_mode`). Counterpart of
  `igraph_get_widest_path` from
  `references/igraph/src/paths/widest_paths.c:365`. Returns the
  actual path (vertex chain + edge chain) along the
  maximum-bottleneck `from → to` route, or `None` if unreachable.
  Self-target (`from == to`) returns the trivial `(vec![from],
  vec![])`.
  Implementation refactors SP-010's loop into a private
  `widest_inner` helper that returns both widths and parent edges;
  `widest_path_widths` strips parents, `widest_path` walks them
  back from the target. Reusing the same core keeps the algorithm
  in one place.
  9 unit tests + 2 oracle tests vs an inline Python reference +
  6 conformance fixtures (2 each C/Python/R, hand-computed since
  python-igraph/rigraph do not bind widest paths) + 1 proptest
  invariant (`widest_path_chain_is_well_formed` — chain validity,
  endpoint anchoring, bottleneck consistency with
  `widest_path_widths`).
- *(paths)* **ALGO-SP-010**: Single-source widest-path widths
  (`widest_path_widths(graph, source, weights) ->
  IgraphResult<Vec<Option<f64>>>` plus the mode-aware
  `widest_path_widths_with_mode`). Counterpart of
  `igraph_widest_path_widths_dijkstra` from
  `references/igraph/src/paths/widest_paths.c:596`. Returns the
  maximum bottleneck width of any `source → v` path for each
  vertex `v` — useful for network-capacity / max-flow heuristics.
  Algorithm: Dijkstra with a max-priority queue keyed by width.
  Relaxation uses `width[u] = max(width[u], min(width[v], edge_w))`.
  Source's own width is `Some(f64::INFINITY)` by convention;
  unreachable vertices are `None`; edges with weight
  `-f64::INFINITY` are treated as "edge absent" (matches upstream).
  Uses Rust's `BinaryHeap` directly (no indexed heap needed —
  lazy stale-entry skip on pop).
  12 unit tests + 2 oracle tests vs an inline Python reference
  (python-igraph does not bind widest paths) + 6 conformance
  fixtures (2 each C/Python/R, hand-computed expected values
  since the C extractor cannot drive a non-bound function) +
  1 proptest invariant (`widest_path_invariants` — bottleneck
  bounded by max edge weight; reachability matches Dijkstra).
- *(paths)* **ALGO-SP-003**: Johnson's all-pairs shortest distances
  (`johnson_distances(graph, weights) -> IgraphResult<Vec<Vec<Option<f64>>>>`).
  Counterpart of `igraph_distances_johnson` from
  `references/igraph/src/paths/johnson.c:83`. Computes the full
  `vcount × vcount` distance matrix with support for negative edge
  weights on directed graphs.
  Algorithm: fast-path to V independent Dijkstras when all weights
  are non-negative (matches upstream short-circuit). Slow path:
  compute Johnson potentials `h[v]` via virtual-source SPFA (the
  standard trick — initialising every dist[v]=0 in SPFA is
  equivalent to attaching a virtual vertex with zero-weight
  outgoing edges), reweight each edge to `w' = w + h[u] - h[v]`,
  snap roundoff negatives to 0, run Dijkstra from each source on
  the reweighted graph, and recover `d[u][v] = d'[u][v] - h[u] +
  h[v]`. Time complexity `O(V·E + V·(V+E)·log V)`.
  Constraints (match upstream): undirected graphs with any negative
  weight are rejected (an undirected negative edge is itself a
  length-2 negative cycle); negative cycles reachable from any
  vertex are surfaced as `IgraphError::InvalidArgument`.
  11 unit tests + 2 oracle tests vs python-igraph + 6 conformance
  fixtures (2 each C/Python/R) + 1 proptest invariant
  (`johnson_matches_pairwise_dijkstra_on_nonneg_weights`).
- *(paths)* **ALGO-SP-002**: Bellman-Ford single-source shortest
  distances (`bellman_ford_distances(graph, source, weights) ->
  IgraphResult<Vec<Option<f64>>>` plus the mode-aware
  `bellman_ford_distances_with_mode`). Counterpart of
  `igraph_distances_bellman_ford` from
  `references/igraph/src/paths/bellman_ford.c:69`. Algorithm: SPFA
  (Shortest Path Faster Algorithm), the queue-based BF variant
  upstream uses. Initial queue contains every vertex; relaxation
  marks targets dirty and re-queues. Negative cycle detected when a
  vertex is popped more than `vcount` times → returns
  `IgraphError::InvalidArgument` (matches upstream's
  `IGRAPH_ENEGCYCLE`). Positive-infinite weights are ignored
  (matches upstream); NaN weights and size mismatch are rejected.
  Use this when edge weights may be negative; for non-negative
  weights `dijkstra_distances` is asymptotically faster (`O((V+E)
  log V)` vs `O(V·E)`).
  12 unit tests + 3 oracle tests vs python-igraph + 9 conformance
  fixtures (3 each C/Python/R) + 1 proptest invariant
  (`bellman_ford_matches_dijkstra_on_nonneg_weights`).
- *(core)* **ALGO-CORE-001c**: Structural mutators
  `Graph::delete_edges(&[EdgeId])`, `Graph::delete_vertices(&[VertexId])`,
  and `Graph::delete_vertices_map(&[VertexId]) -> (Vec<Option<VertexId>>,
  Vec<VertexId>)` returning `(map, invmap)`. Counterparts of
  `igraph_delete_edges` and `igraph_delete_vertices_map` from
  `references/igraph/src/graph/type_indexededgelist.c:500-825`. Both
  validate ids up-front (errors leave graph state untouched), tolerate
  duplicate ids, and re-index `oi/ii/os/is` via the existing
  `rebuild_indexes` helper — no manual sort/rebuild needed. 16 unit
  tests (empty input, duplicates, all-removal, out-of-range, self-loops,
  parallel edges, directed direction preservation, post-delete
  add_edges round-trip) + 2 proptest invariants
  (`delete_{edges,vertices}_preserves_invariants` over arbitrary
  graphs ≤ 8 vertices). No oracle/conformance: structural mutation,
  not numerical algorithm output (parity with CORE-001b/d).
- *(paths)* **ALGO-TR-003**: Random walk on a graph
  (`random_walk(graph, weights, start, mode, steps, seed) ->
  IgraphResult<(Vec<VertexId>, Vec<EdgeId>)>`). Counterpart of
  `igraph_random_walk(_, &weights, _, _, start, mode, steps,
  IGRAPH_RANDOM_WALK_STUCK_RETURN)` from
  `references/igraph/src/paths/random_walk.c:288`. New module
  `src/algorithms/paths/random_walk.rs`.

  Behaviour matches upstream's `IGRAPH_RANDOM_WALK_STUCK_RETURN`
  variant (returns the truncated chain when the walk reaches a
  vertex with no admissible outgoing neighbours). `weights = None`
  selects neighbours uniformly; `weights = Some(_)` selects each
  candidate edge with probability proportional to its weight (zero
  and non-finite weights are skipped). Negative or NaN weights
  reject at validation time. Mode follows the same `DijkstraMode`
  convention as SP-001b/c (on undirected graphs every mode collapses
  to ALL).

  PRNG: deterministic inline SplitMix64 seeded by the user-supplied
  `seed: u64` — no external `rand` dependency. Same
  `(graph, weights, start, mode, steps, seed)` always produces the
  same chain; callers wanting non-deterministic behaviour can derive
  `seed` from `std::time::SystemTime` etc. 13 new unit tests + 1
  doctest covering: 4-cycle walk length; sink stops early; zero-step
  singleton; isolated vertex stuck immediately; deterministic same-
  seed repeatability; different-seed divergence; weighted walk picks
  only positive-weight edges; weighted zero-total stops early;
  negative / NaN / size-mismatch / out-of-range errors; directed
  IN-mode reverse-edge walk. 1 new proptest invariant: chain is
  well-formed (`vs[0] == start`; `len(vs) ≤ steps + 1`;
  `len(es) == len(vs) - 1`; consecutive vertices are connected by
  the recorded edge id; same seed yields identical chain).

  No oracle / conformance fixtures: cross-implementation chain
  comparison isn't meaningful (each impl uses its own RNG). The
  AWU's correctness is anchored by the structural proptest invariant
  plus the unit-test seed-reproducibility checks.
- *(paths)* **ALGO-SP-005**: A* shortest path with admissible heuristic.
  Adds `a_star_path<H: Fn(VertexId, VertexId) -> f64>(graph, from, to,
  weights: Option<&[f64]>, mode: DijkstraMode, heuristic: H) ->
  IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>>` (single-source
  single-target). Counterpart of `igraph_get_shortest_path_astar()`
  (`paths/astar.c:93`). New module
  `src/algorithms/paths/astar.rs`.

  Behaviour mirrors upstream:
  - `weights = None` is treated as unit-weights (BFS-equivalent).
  - `weights[e] = INFINITY` skips the edge during relaxation.
  - Unreachable target → `Ok(None)`; `from == to` → vertex chain
    `[from]` and empty edge chain.
  - With null heuristic (`|_, _| 0.0`), A* reduces to Dijkstra and is
    guaranteed correct.
  - Heuristic must return non-negative non-NaN values, else
    [`IgraphError::InvalidArgument`]. Negative / NaN edge weights
    rejected at validation time.

  Internals: BinaryHeap with `(f_score, tiebreaker, vertex)` entries;
  closed-set tracking; mode-aware incidence (private
  `incident_for_mode` helper kept local to keep cross-module
  dependencies tight). 12 new unit tests (BFS-equivalent on chain;
  weighted triangle shortcut; unreachable target; from == to
  singleton; admissible heuristic equals null-heuristic length;
  directed IN-mode reverses; negative weight error; NaN weight error;
  size mismatch error; out-of-range source/target errors; negative
  heuristic error; INF weight skipped). 1 new doctest, 2 new oracle
  tests vs python-igraph (unit-weight chain; weighted triangle
  shortcut), 3 three-source conformance fixtures, 1 new proptest
  invariant (A* with null heuristic produces a path with edge-weight
  sum equal to `dijkstra_distances` from source to target).
- *(properties)* **ALGO-PR-006d**: Directed weighted assortativity
  (`assortativity_degree_directed_weighted(graph, weights) ->
  Option<f64>`), counterpart of `igraph_assortativity_degree(_, _,
  /*directed=*/true, &weights)` (`misc/mixing.c:351-405`). Pearson
  correlation between out-strength of source and in-strength of target,
  each edge weighted by `w`. Returns `None` for graphs with no edges,
  zero total weight, or zero variance (matches upstream's `IGRAPH_NAN`).
  Undirected graphs route to the symmetric formula via
  [`assortativity_degree_weighted`]. Edge weights validated as
  non-negative, finite, not NaN. 6 new unit tests (3-cycle uniform
  → None; chain with unit weights ≡ unweighted directed; undirected
  routes to undirected weighted; empty graph None; negative weight error;
  size mismatch error), 1 new doctest, 1 new oracle test (directed chain
  unit weights ≡ unweighted directed), 3 three-source conformance fixtures
  (C chain-with-branch unit → -0.5 formula collapse; py DAG diamond unit
  → -1.0 hand-computed; R directed 3-cycle weights (1,2,4) → 1.0
  hand-computed perfect alignment between out-strength of source and
  in-strength of target).
- *(paths)* **ALGO-SP-021..023 (weighted)**: Dijkstra-based eccentricity
  / radius / diameter for weighted graphs. Adds six new public items
  to `src/algorithms/paths/radii.rs`:
  - `eccentricity_weighted_with_mode(graph, weights, mode) -> Vec<f64>`
    — counterpart of `igraph_eccentricity(_, weights, _, vss_all(), mode)`.
  - `radius_weighted_with_mode(graph, weights, mode) -> Option<f64>`
    — counterpart of `igraph_radius(_, weights, _, mode)`.
  - `diameter_weighted_with_mode(graph, weights, mode) -> Option<f64>`
    — counterpart of `igraph_diameter(_, weights, _, NULL, NULL, NULL,
    NULL, mode == directed ? IGRAPH_OUT : IGRAPH_ALL, /*unconn=*/true)`.
  - `eccentricity_weighted` / `radius_weighted` / `diameter_weighted`
    — OUT-mode-default thin wrappers (matching upstream's default
    semantics when `mode` is omitted from the call site).

  Implementation reuses [`dijkstra_distances_with_mode`] from SP-001c:
  for each vertex `v`, run weighted single-source Dijkstra, then fold
  the max-of-finite distances (matches upstream's `unconn=true`/ignore
  -unreachable semantics; isolated vertices have eccentricity `0.0`).
  Radius / diameter are min / max over the eccentricity vector. Edge
  weights validated by the underlying Dijkstra (non-negative, finite,
  not NaN). 9 new unit tests (path eccentricity; singleton zero;
  isolated vertices zero; disconnected unconn-true semantics; directed
  IN/ALL reachability; undirected modes agree; negative weight error;
  empty graph None; with_mode/Out matches default), 1 new doctest
  (`eccentricity_weighted_with_mode`), 3 new oracle tests vs python-
  igraph (P3 ecc; directed P3 radius across modes; undirected triangle
  diameter), 9 three-source conformance fixtures (3 algos × C/py/R)
  with bespoke fixture-walking runners that thread `case.graph.weights`,
  2 new proptest invariants (weighted ecc/radius/diameter consistency
  per mode; unit-weight weighted ecc agrees with unweighted ecc cast to
  f64).
- *(paths)* **ALGO-SP-001c**: Dijkstra IN/ALL mode plus all-shortest-paths.
  Adds the [`DijkstraMode`] enum (`Out` / `In` / `All`), the
  `incident_for_mode` private helper that selects per-vertex
  incident-edge lists by mode (with a new `pub(crate)` `Graph::incident_in`
  for in-incident edge ids on directed graphs), and six new public
  items in `src/algorithms/paths/dijkstra.rs`:
  - `dijkstra_distances_with_mode`,
  - `dijkstra_paths_with_mode`,
  - `dijkstra_path_to_with_mode`,
  - `dijkstra_distances_cutoff_with_mode`,
  - `dijkstra_distances_multi_with_mode`,
  - `dijkstra_all_shortest_paths(graph, source, weights, mode) ->
    DijkstraAllPaths`.

  `DijkstraAllPaths { vertex_paths, edge_paths, nrgeo }` carries
  every distinct shortest source→v path (vertex chain + parallel
  edge chain) plus `nrgeo[v]` = the geodesic count. Tie detection
  uses an internal `cmp_eps` helper mirroring upstream's
  `igraph_cmp_epsilon` (epsilon 1e-10, scale-relative). Equal-cost
  alternative paths are recorded only when the *connecting edge has
  positive weight* — matches upstream's zero-weight loop guard
  (avoids infinite enumeration through 0-weight edges in undirected
  graphs). `nrgeo` is computed in heap-settle topological order in
  linear time after the BFS. Path reconstruction is a depth-first
  enumeration through the predecessor DAG; output ordering is
  heap-dependent and not stable across impls — conformance
  comparisons cover `distances` + `nrgeo` only.

  Mode is threaded through the existing `dijkstra_inner` helper, so
  the legacy `dijkstra_distances` / `dijkstra_paths` /
  `dijkstra_path_to` / `dijkstra_distances_cutoff` /
  `dijkstra_distances_multi` continue to behave identically (they
  delegate with `DijkstraMode::Out`). For undirected graphs every
  mode collapses to ALL (every edge is bidirectional). 14 new unit
  tests (mode-default agreement; directed P3 IN/ALL reachability;
  undirected modes agree; paths_with_mode IN parents; path_to_with_mode
  unreachable via OUT; cutoff_with_mode mask; multi_with_mode rows;
  diamond all-paths two geodesics; unique chain single path;
  unreachable empty paths; directed IN all-paths; invalid source
  errors; zero-weight guard drops alt path), 2 new doctests
  (`dijkstra_distances_with_mode`, `dijkstra_all_shortest_paths`),
  3 new oracle tests vs python-igraph (directed IN distances; ALL
  distances; diamond all-paths nrgeo), 6 three-source conformance
  fixtures (2 algos × C/py/R), 2 new proptest invariants
  (`dijkstra_distances_with_mode(_, Out) ≡ dijkstra_distances`;
  `dijkstra_all_shortest_paths` consistency: nrgeo[source]=1, every
  emitted path is a valid weighted geodesic of length distances[v]).
- *(paths)* **ALGO-SP-001b**: Dijkstra paths / parents / cutoff /
  multi-source. Adds four public items to
  `src/algorithms/paths/dijkstra.rs` on top of the existing SP-001
  `dijkstra_distances`:
  - `dijkstra_paths(graph, source, weights) -> DijkstraPaths` —
    counterpart of `igraph_get_shortest_paths_dijkstra`. Returns
    `{ distances, parents, inbound_edges }` where each vector is
    `Vec<Option<...>>` of length `vcount`. The source has
    `parents[source] = None`; unreachable vertices share that sentinel
    (caller can disambiguate via `distances[v]`).
  - `dijkstra_path_to(graph, source, target, weights) ->
    Option<(Vec<VertexId>, Vec<EdgeId>)>` — counterpart of
    `igraph_get_shortest_path_dijkstra`. Returns `None` for
    unreachable target; otherwise the vertex chain (including source
    and target) and the parallel edge id chain.
  - `dijkstra_distances_cutoff(graph, source, weights, cutoff:
    Option<f64>) -> Vec<Option<f64>>` — counterpart of
    `igraph_distances_dijkstra_cutoff`. `cutoff = None` is identical
    to `dijkstra_distances`; `cutoff = Some(c)` masks every vertex
    with `dist > c` to `None` (matches upstream's "distances within
    cutoff only" semantics).
  - `dijkstra_distances_multi(graph, sources, weights, cutoff) ->
    Vec<Vec<Option<f64>>>` — multi-source row-per-source variant.
    Each source is run independently (matches upstream's `fromvit`
    iteration).

  Implementation factors out a private `dijkstra_inner` that runs the
  binary-heap loop with multiple seed sources at distance 0, optional
  cutoff, and optional inbound-edge bookkeeping; `validate_weights`
  enforces non-negative finite weights (matches SP-001 contract);
  `INFINITY`-weight edges are skipped during relax. Mode is `OUT`
  only; `IN`/`ALL` plus all-shortest-paths ship with SP-001c. 15 new
  unit tests (paths SPT relaxation; parent/inbound consistency;
  unreachable parent is `None`; path_to vertex+edge chain; path_to to
  self is singleton; path_to unreachable; cutoff masks above; cutoff
  None ≡ unbounded; cutoff zero keeps source only; cutoff NaN errors;
  multi yields per-source distances; multi empty list yields empty;
  multi propagates cutoff; multi rejects out-of-range source). 2 new
  doctests (`dijkstra_paths`, `dijkstra_path_to`). 3 new oracle tests
  vs python-igraph (triangle with shortcut paths; directed chain
  path_to; cutoff masks above 2.5). 9 three-source conformance
  fixtures (3 algos × C/py/R; conformance runners walk fixtures
  directly to access `case.graph.weights`). 3 proptest invariants
  (paths SPT relaxation `dist[parent] + w(eid) == dist[v]` for every
  reachable non-source; path_to edge weights sum to dist[target];
  cutoff is monotone — more permissive cutoff produces a superset of
  reachable vertices and never disagrees on retained distances).
- *(paths)* **ALGO-SP-021abc**: mode-aware
  `eccentricity_with_mode(graph, mode) -> Vec<u32>`,
  `radius_with_mode(graph, mode) -> Option<u32>`,
  `diameter_with_mode(graph, mode) -> Option<u32>` plus the
  `EccMode { Out, In, All }` enum. Counterparts of
  `igraph_eccentricity / igraph_radius / igraph_diameter` with the
  `mode` parameter wired through. For directed graphs the BFS follows
  `Out`-edges (default; matches the legacy `eccentricity / radius /
  diameter`), `In`-edges, or `All` (treat every edge as
  bidirectional). For undirected graphs every mode reduces to `All`
  (every edge is bidirectional). Implemented via an inlined BFS that
  picks per-vertex neighbour lists via `out_neighbors_vec` /
  `in_neighbors_vec` / their concatenation — no allocation reuse
  beyond the distance vector. The legacy `eccentricity / radius /
  diameter` APIs are unchanged and remain `Out`-mode for directed
  graphs. python-igraph's `Graph.diameter` has no IN-mode, so the
  oracle handler reverses edges for `mode="in"` on directed inputs
  and uses `directed=True`; the `Out`/`All` modes map to
  `directed=True`/`directed=False`. 8 new unit tests (legacy-API
  agreement; undirected modes agree; directed P3 IN reverses BFS;
  directed P3 ALL collapses to undirected; directed K3-cycle modes;
  sources/sinks have zero ecc in OUT/IN; min/max ↔ radius/diameter;
  empty-graph None for every mode), 1 doctest, 3 oracle tests vs
  python-igraph (directed P4 ecc all modes; directed K3 cycle radius
  all modes; directed DAG diamond diameter all modes), 9 three-source
  conformance fixtures (C: directed P4 IN ecc/radius/diameter →
  [0,1,2,3]/0/3; py: directed 3-cycle ALL → [1,1,1]/1/1; R: directed
  out-star OUT → [1,0,0,0]/0/1), 3 proptest invariants
  (min/max ↔ radius/diameter consistency for every mode; undirected
  modes agree; `eccentricity_with_mode(_, Out) ≡ eccentricity`).
- *(operators)* **ALGO-OP-006**: difference of two graphs
  (`difference(orig, sub) -> Graph`), counterpart of
  `igraph_difference(_, &orig, &sub)` (`operators/difference.c:54`).
  Per canonicalised endpoint pair the result keeps
  `max(0, count_orig − count_sub)` copies, i.e. clamped multiset
  subtraction; pairs unique to `sub` drop out entirely (difference
  cannot synthesise edges). **vcount = orig.vcount() only** —
  asymmetric, unlike [`union`] / [`intersection`] which take
  `max(left, right)`. Same `BTreeMap` template as OP-004/OP-005
  (O((E1+E2) log(E1+E2))). Errors on directedness mismatch.
  python-igraph exposes `g.difference(h)` as a Graph method (not
  module-level `ig.difference`), so the oracle handler calls it that
  way. 16 unit tests (empty±empty, vcount-orig-only, vcount when sub
  larger / smaller, doc example, self-difference is empty,
  identity-with-empty, empty-minus-anything, sub-only pair drops,
  multiplicity clamp to zero, partial subtraction, directed
  orientations independent, directed per-orientation multiplicity,
  loops, mixed-directedness errors, undirected canonicalisation,
  high-index sub vertex ignored), 1 doctest, 3 oracle tests vs
  python-igraph's `g.difference(h)` (undirected triangle\path; directed
  unmatched orientation; multiplicity clamps to zero), 3 three-source
  conformance fixtures (C `igraph_union.c` BINARY VERSION inputs but
  asking difference 4v 1e; py K3 \ P4 → 3v 1e; R K4 \ K3-on-{0,1,2} →
  4v 3e star at vertex 3), 3 proptest invariants
  (`difference(g, g) ≡ empty`; `difference(g, empty) ≡ g`;
  per-pair clamped subtract + no-edge-synthesis).
- *(operators)* **ALGO-OP-005**: intersection of two graphs
  (`intersection(left, right) -> Graph`), counterpart of
  `igraph_intersection(_, &left, &right, NULL, NULL)`
  (`operators/intersection.c:71`). Phase-1 two-graph slice; multi-arg
  `intersection_many` and edge-mapping outputs ship later. Vertex sets
  aligned by index (`vcount = max(left.vcount(), right.vcount())` —
  matches upstream's "common edges, larger vertex set" semantics);
  edges intersected by min-multiplicity per canonicalised endpoint
  pair (so pairs unique to either side drop out entirely). Iterates
  the smaller of the two count-BTreeMaps and looks up matches in the
  other (O((E1+E2) log(E1+E2))). Errors on directedness mismatch. 14
  unit tests (empty, vcount-max, doc example, both-sides multiplicity,
  idempotent, directed orientation separation, loops, undirected
  canonicalisation, commutative), 1 doctest, 3 oracle tests vs
  python-igraph's `igraph.intersection([g1, g2])` (undirected triangle
  ∩ path; directed overlap; min-multiplicity), 3 three-source
  conformance fixtures (C `igraph_union.c` BINARY VERSION inputs but
  asking intersection 5v 3e; py K3 ∩ P4 sharing vertices 4v 2e; R K4 ∩
  K3-on-{0,1,2} 4v 3e), 2 proptest invariants (idempotence;
  per-pair min-multiplicity + commutativity).
- *(operators)* **ALGO-OP-004**: union of two graphs
  (`union(left, right) -> Graph`), counterpart of
  `igraph_union(_, &left, &right, NULL, NULL)`
  (`operators/union.c:69`). Phase-1 two-graph slice; multi-arg
  `union_many` and edge-mapping outputs ship later. Vertex sets are
  aligned by index (output `vcount = max(left.vcount(),
  right.vcount())`); edges are unioned by max-multiplicity per
  endpoint pair, with undirected pairs canonicalised to `(min, max)`
  and directed pairs counted per orientation. Implementation merges
  per-pair counts with two `BTreeMap`s (deterministic, O((E1+E2)
  log(E1+E2))). Errors when directedness diverges. 14 unit tests
  (empty, vcount-max, doc example, both-sides multiplicity, idempotent
  with self, directed orientation separation, loops, undirected
  canonicalisation, swap-endpoints invariance), 1 doctest, 3 oracle
  tests vs python-igraph's `igraph.union([g1, g2])` (undirected
  triangle ∪ path; directed opposing paths; max-multiplicity), 3
  three-source conformance fixtures (C upstream `igraph_union.c`
  BINARY VERSION 5v 5e directed-with-loop; py K3 ∪ P4 sharing vertices
  4v 4e; R directed opposite paths 3v 4e), 2 proptest invariants
  (idempotence; per-pair max-multiplicity + ecount = Σ_pairs max).
- *(connectivity)* **ALGO-CC-003**: weak graph decomposition
  (`decompose(graph) -> Vec<Graph>`), counterpart of
  `igraph_decompose(_, _, IGRAPH_WEAK, -1, 1)` (`components.c:603`).
  Phase-1 minimal slice covers the weak (BFS-by-actstart) branch only;
  strong decomposition is a follow-up AWU. Each component subgraph has
  vertices renumbered to `0..k` in BFS visit order, matching upstream's
  `IGRAPH_SUBGRAPH_AUTO` semantics. Loops and parallel edges within a
  component are preserved; on directed input the components are
  detected by weak connectivity but the subgraph keeps every edge's
  original orientation. Single edge sweep places each edge in its
  pre-computed component subgraph (O(V+E) total). 9 unit tests
  (empty/null/single/multi-component, vertex remapping, directed
  orientation, loops + parallel edges, cross-check vs CC-001), 1
  proptest invariant (component count + vcount/ecount partition + each
  subgraph is single-cc), 2 oracle tests (deterministic 5-vertex
  2-component fixture for exact structural match; karate
  size-only cross-check — BFS remapping order differs between
  python-igraph and our impl when neighbour iteration order matters,
  documented), 3 three-source conformance fixtures (C triangle ∪ edge
  → `[K3, K2]`; py `K3 ∪ P3 ∪ {6}` with isolated vertex; R `K4` single
  component).
- *(properties)* **ALGO-PR-002c**: Barrat's weighted local transitivity
  (`transitivity_barrat(graph, weights)`), counterpart of
  `igraph_transitivity_barrat()` (`triangles.c:874`). Per-vertex
  `Vec<Option<f64>>`; `None` for degree<2 / strength==0 (matches
  upstream `IGRAPH_TRANSITIVITY_NAN`). Implements equation (5) of Barrat
  et al., PNAS 101 3747 (2004): for each vertex `v`,
  `Σ_t (w(v,u_t) + w(v,u_t')) / (s_v · (deg_v − 1))`. Uses upstream's
  sentinel-marker pattern (`nei_mark[u] == v+1` to avoid per-iteration
  resets). Phase-1 minimal slice — rejects directed graphs and any
  non-simple input (multi-edges or self-loops); upstream documents the
  function as undefined for non-simple graphs. Re-applies the
  `incident()`-not-`neighbors()` weight-handling pattern validated in
  PR-005b: positional alignment between the two iterators is not
  guaranteed. Full 9-step SOP: 12 new unit tests (unit weights match
  unweighted local on K4 and K4-minus-edge, hand-checked triangle
  unequal weights, path/isolated → None, all four input validations,
  directed/loop/multi rejection), 1 oracle test against python-igraph's
  `Graph.transitivity_local_undirected(weights="weight", mode="nan")`,
  3 three-source conformance fixtures (1 per source: C weighted
  triangle (1,2,4) all-1.0; py K4-minus-edge unit → [1, 2/3, 2/3, 1];
  R K4 unit → all-1.0), 2 proptest invariants (unit weights match
  unweighted local; values bounded in [0, 1]). Verified against
  python-igraph: weighted karate.edges with cyclic 1-4 weights matches
  to 1e-9.
- *(properties)* **ALGO-PR-005b**: weighted variant + per-degree
  aggregate of `avg_nearest_neighbor_degree`. Three new public entries:
  - `avg_nearest_neighbor_degree_weighted(graph, weights)` — Barrat
    formula `k_nn(v) = (1/s_v) Σ_{u∼v} w_{vu} k_u` with `s_v` being
    the strength of `v`. Returns `None` for vertices with strength 0.
  - `knnk(graph)` — unweighted per-degree aggregate `k_nn(k)`.
    Output indexed by degree `k`; `result[k-1]` is the mean of
    `knn[v]` over all vertices with `deg(v)=k`. `None` for unused
    degrees (matches upstream's `IGRAPH_NAN`).
  - `knnk_weighted(graph, weights)` — weighted per-degree aggregate;
    pools `Σ_{deg(v)=k} sum_v / Σ_{deg(v)=k} strength_v` per upstream
    (degrees.c:155 — `knnk[nv-1]` accumulates raw `sum`, `deghist`
    accumulates `strength`).
  Implementation note: the weighted code path iterates over
  `incident()` (edge ids) and resolves the neighbour via
  `edge_other()`. `neighbors()` and `incident()` are NOT positionally
  aligned for undirected graphs (neighbors merges out/in lists in
  ascending order; incident concatenates), so a join-by-edge is
  required. Weights are validated as finite, non-negative, with
  `len == ecount`.
  Full 9-step SOP: 11 new unit tests on top of PR-005's 6 (covering
  uniform=unweighted, hand-checked triangle Q=2.0 invariant under any
  weights, isolated/zero-strength=None, validation), 3 new oracle
  tests against python-igraph's `Graph.knn(weights="weight")` (uniform
  + non-uniform karate weights, knn_w + knnk + knnk_w), 9 three-source
  conformance fixtures (3 per source × 3 algos: weighted-triangle hand
  check, K4 unit collapse, 5-path arithmetic), 2 proptest invariants:
  unit-weight knn_w/knnk_w must equal unweighted on every graph.

- *(connectivity)* **ALGO-CC-012**: `BiconnectedComponents.component_edges`
  — explicit per-component edge-id list (companion of CC-011's
  `tree_edges`). Counterpart of upstream's `component_edges` output
  argument at `components.c:1216` of `igraph_biconnected_components()`.
  Implementation: after the spanning-tree edges of a biconnected
  component are popped off the DFS edge stack, re-scan each component
  vertex's incidence list and pick edges whose other endpoint is also
  in the component (using the `nei < vert` guard for canonicalisation,
  matching upstream). Self-loops are skipped by that guard (also
  matches upstream). Loop edges of the same biconnected component are
  partitioned across the new field.
  Full 9-step SOP: 3 new unit tests (k4_complete_component_edges_has_all_six_edges,
  pendant_component_edges_match_tree, component_edges_partition_non_bridge_edges)
  on top of CC-011's 9, doctest extended to assert partitioning,
  oracle test extended to compare canonicalised endpoint pairs against
  python-igraph (computed from `g.es` filtered by component membership),
  3 three-source conformance fixtures (C 10v upstream + py triangle
  +pendant + R K4) under new `biconnected_component_edges` algo, and
  1 proptest invariant: per-component edge sets partition all non-loop
  edges, tree_edges ⊆ component_edges, and all endpoints stay within
  the component's vertex set.

- *(community)* **ALGO-CO-001b**: `modularity_directed` — directed
  Newman-Girvan modularity (Leicht-Newman 2008). Counterpart of
  `igraph_modularity(_, _, NULL_weights, resolution, /*directed=*/true, _)`.
  Formula: `Q = (1/m) Σ (A_ij − γ k_out_i k_in_j / m) δ(c_i, c_j)`.
  Per-partition `k_out` and `k_in` are tracked separately (vs the
  undirected case where they collapse). Undirected graphs route to
  [`modularity`] (matches python-igraph's "ignored on undirected"
  semantics).
  Full 9-step SOP: 6 new unit tests (23 total in module — incl.
  two-triangles+bridge with hand-checked Q=18/49, 3-cycle single
  partition Q=0, undirected routing, no-edges None, validation),
  3 oracle tests, 3 three-source conformance fixtures, 1 proptest
  invariant: undirected routes to canonical formula.

- *(properties)* **ALGO-PR-006c**: `assortativity_degree_directed` —
  directed Pearson correlation of source out-degree vs target in-degree.
  Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/true)`.
  Different formula from the symmetric undirected case:
  `r = (num1 − num2*num3/m) / (sqrt(den1 − num2²/m) * sqrt(den2 − num3²/m))`.
  Returns `None` when either variance term collapses (regular
  in-/out-degrees, e.g. directed 3-cycle).
  Existing `assortativity_degree` now routes directed inputs through
  `assortativity_degree_directed` (matches python-igraph's default
  behaviour where the `directed` arg is "do the natural thing").
  Full 9-step SOP: 5 new unit tests (15 total in module — incl. 3-cycle
  None, path-3 None due to vanishing variance, well-defined chain+branch
  with hand-checked r=-0.5, undirected routing), 3 oracle tests,
  3 three-source conformance fixtures, 1 proptest invariant: undirected
  graphs route to the canonical formula.

- *(properties)* **ALGO-PR-015b**: `coreness_with_mode` +
  `CorenessMode { All, In, Out }`. Counterpart of
  `igraph_coreness(_, _, mode)`. The peeling loop walks **reverse-mode**
  neighbours: `Out` peels via in-neighbours, `In` peels via
  out-neighbours, `All` walks the merged adjacency view. Existing
  `coreness()` is now a thin wrapper around
  `coreness_with_mode(_, All)` and accepts directed graphs (which it
  used to reject as unsupported).
  Full 9-step SOP: 6 new unit tests (19 total in module), 3 oracle
  tests, 3 three-source conformance fixtures, 1 proptest invariant:
  on undirected graphs every mode agrees with the canonical entry.

  *(Breaking-ish.)* `coreness()` no longer returns `Unsupported` for
  directed graphs; it now returns the undirected projection
  (`CorenessMode::All`), matching python-igraph's default behaviour.

- *(operators)* **ALGO-OP-002b**: `disjoint_union_many` — variadic
  disjoint union over a slice of graphs. Counterpart of
  `igraph_disjoint_union_many()`. Vertices of the i-th graph shift by
  the cumulative `vcount` of all preceding inputs; existing
  `disjoint_union(left, right)` is now a thin wrapper around
  `disjoint_union_many(&[left, right])`. Empty slice → null graph;
  single-input case yields a clone.
  Full 9-step SOP: 6 new unit tests (15 total in module — incl.
  empty-slice/null, single-input clone, three-triangles shift,
  variadic-vs-pairwise associativity, mixed-directedness rejection,
  directed-orientation preservation), 3 oracle tests (3 triangles,
  mixed sizes, directed chain), 3 three-source conformance fixtures,
  1 proptest invariant: `disjoint_union_many(&[a, b])` matches
  `disjoint_union(a, b)` exactly.

- *(properties)* **ALGO-PR-013b**: `is_simple_with_mode` +
  `SimpleMode { DirectedAsDirected, DirectedAsUndirected }`. Counterpart
  of `igraph_is_simple(_, _, /*directed=*/dir)`. The undirected view
  canonicalises each directed edge to its `(min, max)` endpoint pair
  and reports a mutual pair `{a→b, b→a}` as a multi-edge. Existing
  `is_simple()` is now a thin wrapper around
  `is_simple_with_mode(_, DirectedAsDirected)`.
  Full 9-step SOP: 5 new unit tests (16 total — incl. mutual-pair
  divergence, default-mode equivalence, directed-3-cycle simple,
  self-loop disqualifies in both modes, undirected mode-equivalence),
  3 oracle tests, 3 three-source conformance fixtures, 1 proptest
  invariant: undirected modes always agree.

- *(community)* **ALGO-CO-001c**: `modularity_weighted` — Newman-Girvan
  modularity of a partition with edge weights. Counterpart of
  `igraph_modularity(_, &membership, &weights, resolution,
  /*directed=*/false, _)`. Uses strength `s_v = Σ w_e` (self-loops
  contribute 2w per IGRAPH_LOOPS) instead of degree, and W (sum of
  weights) replaces m. Phase-1 minimal: undirected only; directed
  weighted ships with the future PR-006c-style adaptation. Negative,
  NaN and infinite weights rejected.
  Full 9-step SOP: 7 new unit tests (17 total in module — incl.
  unit-equivalence to PR-008's modularity, balanced-heavy boost,
  asymmetric-heavy-drags-Q-down documented case, zero-total-weight
  → None, two-disjoint-edges hand-checked Q=0.5),
  3 oracle tests (unit-equivalence, heavy-balanced, resolution=0
  density-only mode),
  3 three-source conformance fixtures (C balanced-heavy ≈0.498,
  py unit weights = 6/7-0.5, R two disjoint edges = 0.5),
  1 proptest invariant: unit-weight `modularity_weighted` agrees with
  unweighted `modularity` on every graph + 2-block partition.

- *(properties)* **ALGO-PR-004b**: `reciprocity_with_mode` +
  `ReciprocityMode { Default, Ratio }`. Counterpart of
  `igraph_reciprocity(_, _, ignore_loops, mode)`. Ratio mode is
  `rec / (rec + nonrec)` and counts non-mutual one-way edges twice
  (once at source, once at destination); the two formulas only
  agree on fully-mutual graphs. `ignore_loops` drops self-loops
  from both numerator and denominator in Default; in Ratio they
  drop only from numerator (no `nonrec` increment for self-loops).
  Existing `reciprocity()` is now a thin wrapper around
  `reciprocity_with_mode(_, false, Default)`.
  Full 9-step SOP: 8 new unit tests (16 total in module),
  3 oracle tests (ratio + ignore_loops + undirected),
  3 three-source conformance fixtures (C ratio mode partial,
  py ignore_loops + self-loop, R directed 3-cycle ratio),
  1 proptest invariant: the wrapper agrees with the parametric
  function under default args.

- *(properties)* **ALGO-PR-015**: `coreness` — k-core decomposition
  per vertex via Batagelj & Zaversnik's O(|E|) "An O(m) Algorithm for
  Cores Decomposition of Networks". Counterpart of `igraph_coreness(_,
  _, IGRAPH_ALL)`. Phase-1 minimal slice: undirected graphs only
  (directed IN/OUT modes ship as PR-015b). Self-loops contribute 2 to
  a vertex's degree (matches upstream `IGRAPH_LOOPS`).
  Full 9-step SOP: 13 unit tests (incl. empty/singleton/isolated, K3,
  K4, path, star, triangle+pendant, 2-component disjoint, self-loop
  semantics, directed-graph rejection, coreness-≤-degree bound),
  3 oracle tests (triangle+pendant, 2 components, karate),
  3 three-source conformance fixtures (C triangle+pendant → [2,2,2,1],
  py K4-minus-edge → [2,2,2,2], R 5-path → [1,1,1,1,1]),
  1 proptest invariant: `coreness(v) ≤ degree(v)` on arbitrary
  undirected graphs.

- *(paths)* **ALGO-SP-004**: `floyd_warshall_distances` — all-pairs
  shortest distances via the textbook Floyd-Warshall O(V³) variant.
  Counterpart of `igraph_distances_floyd_warshall(_, _, vss_all,
  vss_all, &weights, IGRAPH_OUT, IGRAPH_FLOYD_WARSHALL_ORIGINAL)`.
  Returns a `vcount × vcount` `Vec<Vec<Option<f64>>>`, with `None` for
  unreachable pairs. Negative weights are accepted on directed graphs
  (rejected at upload for self-loops; rejected at relaxation if the
  diagonal goes negative); rejected outright on undirected graphs
  because every undirected edge induces a 2-cycle. `+inf` weights are
  silently ignored to match upstream igraph C behaviour. Power-of-two
  multi-edges are taken at their minimum.
  Full 9-step SOP: 16 unit tests (incl. unweighted/weighted, unreachable,
  directed/undirected orientation, negative-weight rejection,
  parallel-edges-pick-min, NaN/inf handling, and unit-weight
  equivalence to BFS), 3 oracle tests (unweighted path, weighted
  triangle shortcut, directed chain-with-shortcut), 3 three-source
  conformance fixtures (C directed chain, py 4-path unit weights, R
  undirected weighted triangle), 1 proptest invariant: unit weights
  yield a symmetric matrix whose row-0 matches BFS distances.

- *(properties)* **ALGO-PR-006b**: `assortativity_degree_weighted` —
  weighted Pearson correlation of endpoint strengths. Counterpart of
  `igraph_assortativity_degree(_, _, /*directed=*/false, &weights)`.
  Strength replaces degree (`s_v = Σ w_e` over incident edges, with
  self-loops contributing `2w`); each edge weighted by `w` in the
  Pearson sum. Returns `None` for empty / zero-total-weight / regular
  graphs (zero-variance denominator).
  python-igraph 0.11 has no Python-level weighted assortativity
  (`Graph.assortativity` lacks a `weights` kwarg), so the oracle uses
  unit-weight equivalence to `assortativity_degree`; non-unit cases
  validate via 3-source conformance with hand-computed reference
  values (formula derivation lives in the manifest comments).
  Full 9-step SOP: 11 unit tests (incl. unit-weight equivalence to
  PR-006, weighted-path breaking perfect disassortativity,
  zero-total-weight → None, weight rejection, directed unsupported),
  3 oracle tests (unit-weight cases), 3 three-source conformance
  fixtures (C 3-path non-uniform, py 4-path unit equivalence, R
  diamond non-uniform), 1 proptest invariant: unit weights collapse
  to unweighted assortativity_degree.

- *(properties)* **ALGO-PR-011b**: `pagerank_weighted` — power-iteration
  weighted `PageRank`. Counterpart of `igraph_pagerank(_,
  IGRAPH_PAGERANK_ALGO_POWER, _, _, vss_all(), directed, 0.85,
  &weights, NULL_options)`. Uses out-strength (`Σ w(u → x)`) instead
  of out-degree, and the in-flow term becomes `Σ w(u → v) * PR[u] /
  out_strength(u)`. Dangling-vertex redistribution preserved.
  Weights must be non-negative + finite.
  Full 9-step SOP: 11 unit tests (incl. unit-weight equivalence to
  PR-011 on triangle / directed 4-cycle, heavy-edge concentration,
  star centre dominance, weight rejection), 3 oracle tests against
  python-igraph (1e-6 tolerance for ARPACK drift), 3 three-source
  conformance fixtures (C directed 4-cycle uniform, py heavy-edge
  asymmetric, R undirected triangle uniform), 1 proptest invariant:
  unit weights collapse to unweighted PageRank + sum-to-1.

- *(properties)* **ALGO-PR-010b**: `edge_betweenness_weighted` —
  Brandes-Dijkstra weighted edge-betweenness centrality. Counterpart
  of `igraph_edge_betweenness(_, _, all_eids, directed, &weights)`.
  Same Brandes-Dijkstra framework as PR-008b but the dependency is
  deposited on edges (predecessor list now stores `(vertex, edge_id)`
  tuples). Undirected results halved.
  Full 9-step SOP: 10 unit tests (incl. unit-weight equivalence to
  PR-010, weighted shortcut routing, K4 short-circuit, weight
  rejection), 3 oracle tests, 3 three-source conformance fixtures
  (C 4-path, py triangle with heavy chord, R directed chain with
  shortcut), 1 proptest invariant: unit weights collapse to
  unweighted edge_betweenness.

- *(properties)* **ALGO-PR-008b**: `betweenness_weighted` — Brandes-
  Dijkstra weighted betweenness centrality. Counterpart of
  `igraph_betweenness(_, _, vss_all(), directed, &weights)`.
  Replaces BFS in PR-008 with Dijkstra: relaxation step now
  manages `sigma`/`pred` lists with strict-less-than vs equal-distance
  branches; vertices are pushed onto a stack in final-distance order
  during heap pops, then processed in reverse for dependency
  accumulation. Undirected results halved to match upstream's
  raw-count convention. Weights must be non-negative + finite.
  Full 9-step SOP: 10 unit tests (incl. unit-weight equivalence to
  PR-008, weighted shortcut routing, directed-OUT, K4 short-circuit,
  weight rejection), 3 oracle tests, 3 three-source conformance
  fixtures (C 3-vertex routing through middle, py 5-path unit
  weights matching PR-008, R directed chain with shortcut → 1, 2 each
  carry betweenness 2.0), 1 proptest invariant: unit weights collapse
  to unweighted betweenness.

- *(properties)* **ALGO-PR-009b**: `harmonic_centrality_weighted` —
  Dijkstra-based weighted harmonic centrality. Counterpart of
  `igraph_harmonic_centrality(_, _, vss_all(), IGRAPH_OUT, &weights,
  /*normalized=*/true)`. Returns `Vec<f64>`: `(1/(n-1)) * sum 1/d_w`.
  Defined on disconnected graphs (unlike weighted closeness — `1/inf
  = 0`).
  Full 9-step SOP: 9 unit tests (incl. unit-weight equivalence to
  PR-009, weighted path-with-doubled-edge, directed, disconnected),
  3 oracle tests, 3 three-source conformance fixtures, 1 proptest
  invariant: unit weights match unweighted harmonic.

- *(properties)* **ALGO-PR-007b**: `closeness_weighted` — Dijkstra-
  based weighted closeness centrality. Counterpart of
  `igraph_closeness(_, _, _, _, vss_all(), IGRAPH_OUT, &weights,
  /*normalized=*/true)`. Returns `Vec<Option<f64>>`: `Some(reach /
  sum_dist)` per vertex; `None` for isolated. Reuses
  [`crate::dijkstra_distances`] (SP-001) so unit weights collapse
  exactly to the unweighted PR-007 result.
  Full 9-step SOP: 10 unit tests (incl. unit-weight equivalence to
  PR-007, non-uniform star, directed path, weight rejection
  forwarding from SP-001), 3 oracle tests, 3 three-source
  conformance fixtures (C 4-star non-uniform, py directed path,
  R undirected 4-path), 1 proptest invariant: unit weights match
  unweighted closeness.

- *(operators)* **ALGO-OP-003**: `complementer`. Returns a new graph
  containing every `(u, v)` edge the input does not. Toggles
  self-loops via the `loops` flag. Counterpart of
  `igraph_complementer()` from
  `references/igraph/src/operators/complementer.c`. Phase-1 minimal
  slice drops attributes. Uses a sorted-set lookup for O(|V|² log |V|)
  in the worst case; complete graphs short-circuit quickly because
  the iteration only touches `n²` candidate pairs.
  Full 9-step SOP: 11 unit tests (incl. K_n round-trip, double-complement
  identity, directed with loops, parallel-edge collapse), 3 oracle
  tests, 3 three-source conformance fixtures (C 3-path no-loops, py
  3 isolated with loops, R directed single edge), 1 proptest
  invariant cross-checking the edge-count identity `m + m_complement
  = n(n-1)/2` for simple inputs and that double-complement of
  `simplify(g)` recovers it.

- *(paths)* **ALGO-SP-001**: `dijkstra_distances` — single-source
  weighted shortest distances via Dijkstra's algorithm with a binary
  heap. Returns `Vec<Option<f64>>` (None = unreachable). Counterpart
  of `igraph_distances_dijkstra()` from
  `references/igraph/src/paths/dijkstra.c`. Phase-1 minimal slice:
  single source, `IGRAPH_OUT` mode (directed); paths / parents /
  cutoff / multi-source / `IN` / `ALL` variants ship later
  (SP-001b/c). Weights must be non-negative, finite, and exactly
  `ecount()` long; violations return
  [`IgraphError::InvalidArgument`](crate::IgraphError).
  Adds `tests/common::run_with_weights` (and `run_ok_with_weights`)
  for oracle requests that carry a per-edge weights vector; aligned
  to **stored edge order** so `weights[e]` stays paired with the
  right edge after the python-igraph round-trip. Also extends the
  three-source extractors with a `graph_weights` manifest key.
  Full 9-step SOP: 12 unit tests (shortcut paths, unreachable,
  negative/NaN/infinity rejection, BFS equivalence at unit weights,
  parallel-min-weight, directed mode), 3 oracle tests, 3 three-source
  conformance fixtures (C triangle shortcut, py directed chain
  shortcut, R disconnected pair with unreachable), 1 proptest
  invariant cross-checking BFS equivalence at unit weights +
  source-distance-zero + non-negativity.

- *(operators)* **ALGO-OP-002**: `disjoint_union` (two-graph variant).
  Counterpart of `igraph_disjoint_union()` from
  `references/igraph/src/operators/disjoint_union.c`. Vertices of
  `right` are shifted by `left.vcount()`; edges are concatenated in
  original order. Returns
  [`IgraphError::InvalidArgument`](crate::IgraphError) if the two
  graphs differ in directedness. Phase-1 minimal slice: two-graph
  variant only — multi-arg `disjoint_union_many` ships in OP-002b.
  Edge / vertex attributes are dropped.
  Full 9-step SOP: 9 unit tests (incl. directed both, mixed-directedness
  rejection, self-disjoint-union doubling), 3 oracle tests against
  python-igraph's `+` operator, 3 three-source conformance fixtures
  (C two triangles, py path-plus-path with vertex shift, rigraph-style
  directed two paths), 1 proptest invariant cross-checking
  count-preservation + endpoint-shift formula.

- *(properties)* **ALGO-PR-014b**: per-edge `is_loop` + `is_multiple`.
  `is_loop[e]` is the trivial self-loop check; `is_multiple[e]` is
  `true` only for the **second-or-more** appearances of a parallel
  pair (the canonical/first edge id stays `false`) — matches upstream
  contract from `igraph_is_multiple()` (loops.c:230). Sort by
  canonical (from, to) with edge-id tiebreaker to assign the
  "canonical" status to the lowest id in each group.
  Full 9-step SOP: 6 unit tests, 3 oracle tests (multiset-compared
  since edge ids reorder over the wire), 6 three-source conformance
  fixtures (2 algos × 3 sources, multiset compare), 1 proptest
  invariant cross-checking length, per-edge correctness, the algebraic
  link to PR-014's bulk predicates, and the count-of-trues identity.

- *(properties)* **ALGO-PR-014**: `has_loop` + `has_multiple`
  companion predicates. `has_loop` is the O(|E|) self-loop scan;
  `has_multiple` sorts canonicalised endpoint pairs and looks for
  adjacent duplicates (O(|E| log |E|)). Counterparts of
  `igraph_has_loop()` / `igraph_has_multiple()` from
  `references/igraph/src/properties/loops.c` /
  `properties/multiplicity.c`. Two self-loops at the same vertex
  count as parallel (matches upstream).
  Full 9-step SOP: 10 unit tests (incl. directed mutual pair, two
  self-loops at same vertex), 4 oracle tests, 6 three-source
  conformance fixtures (2 algos × 3 sources), 1 proptest invariant
  cross-checking against the structural definition + the
  `is_simple ⇔ ¬has_loop ∧ ¬has_multiple` algebraic identity.

- *(properties)* **ALGO-PR-013**: `is_simple` (predicate — no self-loops
  and no parallel edges). Counterpart of `igraph_is_simple()` from
  `references/igraph/src/properties/multiplicity.c`. Phase-1 minimal
  slice treats directed graphs structurally — `(a,b)` and `(b,a)` are
  distinct, matching upstream's `directed=IGRAPH_DIRECTED`. The
  "treat directed graph as undirected" mode (mutual pairs flag as
  parallel) ships in PR-013b. O(|V| + |E|) using the already-sorted
  out-neighbour lists exposed by `Graph::neighbors`.
  Full 9-step SOP: 10 unit tests (incl. directed mutual pair stays
  simple, simplify makes any graph simple), 3 oracle tests (path,
  self-loop, parallels), 3 three-source conformance fixtures, 1
  proptest invariant cross-checking against the structural definition
  + asserting simplify(g) is always simple + idempotence on
  already-simple graphs.

- *(community)* **ALGO-CO-001**: `modularity` (Newman-Girvan modularity
  of a partition). Returns `Option<f64>` — `None` for graphs with no
  edges (matches upstream's NaN). Counterpart of `igraph_modularity()`
  from `references/igraph/src/community/modularity.c`. Phase-1
  minimal slice: undirected, unweighted; resolution parameter γ is
  configurable. Directed (Leicht-Newman 2008) + weighted variants
  ship later (CO-001b/c). Membership labels need not be consecutive
  (we reindex internally).
  Full 9-step SOP: 10 unit tests (incl. K3 ∪ K3 + bridge canonical case
  with hand-checked Q = 6/7 − 1/2, K4 singletons giving Q = −1/4,
  γ = 0 reduction to e/2m, label reindexing, error paths),
  3 oracle tests (synthetic 6v case, karate-by-id split, K4 γ = 0),
  3 three-source conformance fixtures (C K3 ∪ K3 + bridge; py K5 ∪ K5
  + bridge from `test_structural.py:127 testModularity` Q ≈ 0.4523;
  rigraph-style path(3) with [0,1,0] giving Q = −1/2),
  1 proptest invariant (finite, bounded; Q(all-same) = 0; Q(singletons)
  ≤ 0 when no self-loops; |Q| ≤ 1).

- *(operators)* **ALGO-OP-001**: `simplify` (remove self-loops and/or
  parallel edges). Returns a new [`Graph`] (upstream igraph mutates in
  place; we prefer immutability). Counterpart of `igraph_simplify()`
  from `references/igraph/src/operators/simplify.c`. Phase-1 minimal
  slice ignores edge-attribute combination (`edge_comb` argument);
  attributes ship later under ALGO-AT-*. Both directed and undirected
  graphs are supported.
  Full 9-step SOP: 10 unit tests (no-op, loops only, multi only,
  loops+multi, idempotence on simple graph, empty graph, isolated
  vertices, two igraph_simplify.c example regression cases),
  3 oracle tests on synthetic graphs, 3 three-source conformance
  fixtures (igraph C `igraph_simplify.c` 5x parallel undirected;
  python-igraph `test_operators.py` loops-no-multi; rigraph-style
  directed-loops-only), 1 proptest invariant (vcount/directedness
  preserved, ecount monotone, no surviving loops/parallels per flag,
  idempotent under the same flags).

- *(properties)* **ALGO-PR-012**: eigenvector centrality
  (`eigenvector_centrality`, undirected). Returns `Vec<f64>` —
  dominant-eigenvector entries normalized so `max == 1`. Implemented
  via shifted power iteration `(A + I) · x` to break the `±λ` symmetry
  of bipartite adjacency matrices that would trap plain power
  iteration in a 2-cycle (caught by 4-star unit test). Tighter
  convergence (`eps = 1e-14`, max 5000 iter) so f64 results match
  python-igraph's ARPACK output within 1e-12 relative tolerance.
  Counterpart of `igraph_eigenvector_centrality()` from
  `references/igraph/src/centrality/eigenvector.c`. Phase-1 minimal
  slice: undirected, unweighted; directed mode returns
  `IgraphError::Unsupported` until PR-012b lands.
  Full 9-step SOP: 7 unit tests, 1 oracle test on karate (1e-6 tol),
  3 three-source conformance fixtures, 1 proptest invariant
  (max == 1, nonneg, finite).

- *(connectivity)* **ALGO-CC-011**: biconnected components multi-output
  (`biconnected_components`). Returns `BiconnectedComponents { count,
  components, tree_edges, articulation_points }`. Counterpart of
  `igraph_biconnected_components()` from
  `references/igraph/src/connectivity/components.c:1032-1227`. Same
  iterative DFS with low-link tracking that powers
  `articulation_points` (CC-010), extended to also collect the vertex
  set and tree edges of every biconnected component. Phase-1 minimal
  slice ships components/tree_edges/APs; the separate `component_edges`
  output (O(|V|²·d)) is deferred to CC-012.
  Full 9-step SOP: 9 unit tests (incl. cross-check vs `articulation_points`
  on a 7-vertex fixture), 1 oracle test on karate, 3 three-source
  conformance fixtures, 1 proptest invariant (count consistency,
  component sizes ≥ 2, AP set matches CC-010).

- *(properties)* **ALGO-PR-011**: PageRank (`pagerank`). Power-iteration
  implementation with damping `0.85`, `eps = 1e-10`,
  `max_iter = 1000`. Counterpart of `igraph_pagerank()` from
  `references/igraph/src/centrality/pagerank.c` (the
  `IGRAPH_PAGERANK_ALGO_POWER` branch). Handles dangling vertices via
  uniform redistribution. Phase-1 minimal slice: undirected/IGRAPH_OUT,
  unweighted, default damping; ARPACK-based variant + weighted ship in
  PR-011b.
  Full 9-step SOP: 8 unit tests, 1 oracle test on karate (1e-6
  tolerance — python-igraph defaults to ARPACK so fp drift differs but
  both converge to the same fixed point), 3 three-source conformance
  fixtures (directed 4-cycle uniform 0.25; triangle uniform 1/3; K4
  uniform 0.25), 1 proptest invariant (probability distribution:
  nonneg, finite, sums to 1).

- *(properties)* **ALGO-PR-010**: edge betweenness centrality
  (`edge_betweenness`). Brandes' framework but accumulates dependency
  on edges. Counterpart of `igraph_edge_betweenness()` from
  `references/igraph/src/centrality/betweenness.c:766+`. Phase-1
  minimal slice: undirected/IGRAPH_OUT, unweighted, raw counts.
  Full 9-step SOP: 8 unit tests, 1 oracle test (uses endpoint-pair
  canonicalisation since edge ids change across the wire), 3
  three-source conformance fixtures, 1 proptest invariant
  (nonneg + finite + length match).

- *(properties)* **ALGO-PR-008**: betweenness centrality (`betweenness`).
  Returns `Vec<f64>` — Brandes' (2001) BFS-based algorithm for the
  unweighted case. Counterpart of `igraph_betweenness()` from
  `references/igraph/src/centrality/betweenness.c:504+`. Phase-1
  minimal slice: undirected/IGRAPH_OUT, unweighted, raw counts
  (`normalized = false`); weighted Dijkstra-based variant ships in
  PR-008b.
  Full 9-step SOP: 8 unit tests (empty / isolated / K3 / 5-path /
  4-star / K4 / 4-cycle / directed-3-path), 1 oracle test on karate
  (1e-10 tolerance — Zachary has values ≈ 231 amplifying f64 noise),
  3 three-source conformance fixtures, 1 proptest invariant
  (nonneg, finite, ≤ pair count).

- *(properties)* **ALGO-PR-009**: harmonic centrality
  (`harmonic_centrality`). Returns `Vec<f64>` — `(1/(n-1)) * sum 1/d`
  with `1/inf == 0` for unreachable pairs (always finite, defined on
  disconnected graphs). Counterpart of `igraph_harmonic_centrality()`
  from `references/igraph/src/centrality/closeness.c:800`. Phase-1
  minimal slice: undirected/IGRAPH_OUT, unweighted, normalized.
  Full 9-step SOP: 8 unit tests, 1 oracle test, 3 three-source
  conformance fixtures, 1 proptest invariant (0 ≤ h ≤ 1).

- *(properties)* **ALGO-PR-007**: closeness centrality (`closeness`).
  Returns `Vec<Option<f64>>` — `None` for isolated vertices, otherwise
  the normalized `reach / sum_dist` score (matches upstream's
  `normalized=true` default). Counterpart of `igraph_closeness()` from
  `references/igraph/src/centrality/closeness.c:33+`. Phase-1 minimal
  slice: undirected/IGRAPH_OUT, unweighted; weighted Dijkstra version
  ships in PR-007b.
  Full 9-step SOP: 8 unit tests, 1 oracle test on karate, 3
  three-source conformance fixtures, 1 proptest invariant
  (`0 < x ≤ 1` when defined).

- *(connectivity)* **ALGO-CC-022**: transitive closure
  (`transitive_closure`). Returns a new `Graph` with the same vcount
  and directedness, edges added for every off-diagonal reachable pair
  (ordered for directed, unordered for undirected). Counterpart of
  `igraph_transitive_closure()` from
  `references/igraph/src/connectivity/reachability.c:225-257`. Built
  on top of CC-021 reachability matrix.
  Full 9-step SOP: 7 unit tests, 2 oracle tests against python-igraph
  0.11 (uses per-vertex `subcomponent`), 3 three-source conformance
  fixtures, 1 proptest invariant (closure edge set equals reachability
  matrix off-diagonal pairs; no self-loops; closure is idempotent
  by construction).

- *(connectivity)* **ALGO-CC-021**: reachability matrix
  (`reachability_matrix`). Returns `Vec<Vec<bool>>` where `r[i][j]`
  is `true` iff vertex `j` is reachable from `i` in 0+ steps. Counterpart
  of `igraph_reachability(_, _, _, _, _, IGRAPH_OUT)` from
  `references/igraph/src/connectivity/reachability.c:72-148`. Phase-1
  minimal slice uses BFS-from-each-vertex (O(|V|*(|V|+|E|))); upstream's
  SCC-condensation optimisation lands later.
  Full 9-step SOP: 7 unit tests, 2 oracle tests against python-igraph
  0.11 (uses `g.subcomponent(v, mode='out')` per row), 3 three-source
  conformance fixtures (directed 3-cycle / undirected path-3 / 2
  disjoint edges), 1 proptest invariant (n×n shape, diagonal true,
  symmetric on undirected, row-sums match `count_reachable`).

- *(core)* **ALGO-CORE-001d**: edge query helpers on `Graph`:
  - `get_eid(from, to) -> IgraphResult<EdgeId>` — error if no edge.
  - `find_eid(from, to) -> IgraphResult<Option<EdgeId>>` — None if no edge.
  - `get_all_eids_between(from, to) -> IgraphResult<Vec<EdgeId>>` —
    all parallel edges, sorted ascending.
  Counterparts of `igraph_get_eid()` / `igraph_get_eids()` /
  `igraph_get_all_eids_between()` from
  `references/igraph/src/graph/type_indexededgelist.c:1522-1773`.
  Phase-1 minimal slice: linear scan across the from-bucket
  (O(deg(from))). Upstream's binary-search optimisation lands in a
  perf pass.
  Undirected lookup canonicalises the pair to (min, max) and searches
  the bucket of the smaller endpoint. Self-loops handled correctly.
  7 new unit tests (undirected/directed lookup, parallel edges,
  self-loop, missing-edge None/err, deg-direction respected).

- *(properties)* **ALGO-PR-006**: degree assortativity coefficient
  (`assortativity_degree`). Returns `Option<f64>` — Pearson
  correlation of endpoint degrees over the edge list, or `None` for
  regular graphs (variance denominator zero, matching upstream NaN).
  Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/false)`
  from `references/igraph/src/misc/mixing.c:443` and the underlying
  `igraph_assortativity()` (`mixing.c:273`). Single O(V+E) pass:
  build degree[], sum num1/num2/den1 over edges. Phase-1 minimal:
  undirected, unweighted only — directed input returns
  `IgraphError::Unsupported`. Float-arithmetic ordering matches
  upstream so f64 results agree to the bit with python-igraph.
  Full 9-step SOP: 10 unit tests (empty / isolated / regular / K4 /
  path-3 / star / disjoint pair / directed-rejected / diamond /
  two-triangles-bridge), 1 oracle test on karate (matches python),
  3 three-source conformance fixtures (igraph C Zachary → -0.4756,
  python-igraph K4-minus-edge → -2/3, R path(3) → -1.0), 1 proptest
  bounds invariant (-1 ≤ r ≤ 1).

- *(properties)* **ALGO-PR-005**: average nearest-neighbour degree
  (`avg_nearest_neighbor_degree`). Returns `Vec<Option<f64>>` —
  `result[v]` is the mean degree over `v`'s neighbours, or `None` if
  `v` is isolated. Counterpart of
  `igraph_avg_nearest_neighbor_degree(_, vss_all(), IGRAPH_ALL,
  IGRAPH_ALL, &knn, NULL, NULL)` from
  `references/igraph/src/properties/degrees.c:263`. Self-loops counted
  per upstream's `IGRAPH_LOOPS` (each loop contributes twice to
  undirected degree). Phase-1 minimal slice: unweighted, undirected
  (or `IGRAPH_ALL` mode for directed input). The per-degree aggregate
  (upstream's `knnk`) and weighted/mode-aware variants ship as PR-005b.
  9-step SOP minus bench: 6 unit tests, 1 oracle test on karate
  (1e-12 tolerance), 3 three-source conformance fixtures.

- *(traversal)* **ALGO-TR-001**: multi-output BFS (`bfs_tree`). Returns
  `BfsTree { order, distances, parents }` in a single pass — the visit
  order, per-vertex distance from the root, and the BFS-tree parent
  pointer (None for root and unreachable vertices). Counterpart of the
  common subset of `igraph_bfs(_, root, _, _, &order, _, &father,
  &dist, _, _)` from `references/igraph/src/properties/bfs.c`. Existing
  `bfs` (visit order only) and `distances` (single-purpose) remain;
  callers that want both plus parents can now avoid duplicate BFS work.
  9-step SOP minus separate conformance: 5 unit tests, 1 proptest
  invariant (consistency: bfs_tree.order == bfs(); distances match
  SP-006; parent[v] is one step closer to root for all reachable v).
  Inherits the Phase-0 BFS conformance fixtures via the proptest
  consistency invariant.

- *(properties)* **ALGO-PR-004**: reciprocity (`reciprocity`). Returns
  `Option<f64>` — `None` for graphs with no edges (matches upstream's
  `IGRAPH_NAN`), `Some(1.0)` for undirected graphs (by definition),
  otherwise (number of edges with a reverse counterpart) / (total edges).
  Counterpart of `igraph_reciprocity(_, _, false, IGRAPH_RECIPROCITY_DEFAULT)`
  from `references/igraph/src/properties/basic_properties.c:325`.
  Two-pointer merge over sorted in/out neighbour lists per vertex.
  Self-loops counted as mutual (matches `ignore_loops=false`).
  9-step SOP minus bench: 8 unit tests, 1 oracle test (3v partial
  reciprocity → 2/3), 3 three-source conformance fixtures.
  Ratio mode + `ignore_loops=true` ship as PR-004b.

- *(paths)* **ALGO-CC-042**: directed Eulerian path/cycle construction.
  Folded into the existing `eulerian_path` function — directed graphs
  now use out-edges only, traverse via `edge_target` (always points
  away from `curr`), and pick the start vertex by `out_degree -
  in_degree == 1` for a path or any non-isolated vertex for a cycle.
  3 new unit tests (directed 3-cycle, directed path, directed
  no-Eulerian), 1 new conformance fixture (directed 3-cycle, walk
  length 3). The `Unsupported` error path on directed input is gone.

- *(connectivity)* **ALGO-CC-020**: reachability counts
  (`count_reachable`). Returns `Vec<u32>` where `result[v]` is the
  number of vertices reachable from `v` (including `v` itself).
  Counterpart of `igraph_count_reachable()` from
  `references/igraph/src/connectivity/reachability.c:179`.
  Phase-1 minimal slice does BFS-from-each-vertex via the existing
  SP-006 `distances` primitive — O(|V|*(|V|+|E|)). Upstream's
  SCC-condensation approach (O(|C||V|/w + |V| + |E|)) is a future perf
  pass once `igraph_reachability` lands as CC-021.
  9-step SOP: 7 unit tests, 2 oracle tests against python-igraph
  (using `len(g.subcomponent(v, mode='out'))` since python-igraph 0.11
  doesn't expose `count_reachable` directly), 3 three-source
  conformance fixtures, 1 proptest invariant (count[v] == size of v's
  weak component, on undirected graphs).

- *(paths)* **ALGO-CC-041**: Eulerian path / cycle construction
  (undirected) via Hierholzer's algorithm (`eulerian_path`). Returns
  `Option<Vec<EdgeId>>` — `Some(walk)` when an Eulerian walk exists
  (every edge visited exactly once, walk is consecutively connected),
  `None` otherwise. Counterpart of `igraph_eulerian_path()` from
  `references/igraph/src/paths/eulerian.c:345-450`.
  Phase-1 minimal slice: undirected only. Directed Hierholzer (CC-042)
  uses different adjacency tracking; ships separately.
  Calls existing CC-040 `is_eulerian` to detect existence and pick a
  valid start vertex. Returns `IgraphError::Unsupported` for directed
  input.
  Full 9-step SOP: 9 unit tests (empty / isolated / triangle /
  path-3 / K4 / disconnected / ring-5 / directed-rejected /
  test-eulerian.R complex case), oracle DEFERRED (python-igraph 0.11
  has no Eulerian API), 2 three-source conformance fixtures (C
  triangle walk-len 3; R 4-cycle walk-len 4) with py-skipped, 1
  proptest invariant (visits-every-edge-once-and-connected when
  has_path).

- *(properties)* **ALGO-PR-003**: density and mean shortest-path length
  (`density`, `mean_distance`). Counterparts of `igraph_density()`
  (`basic_properties.c:71`) and `igraph_average_path_length()`
  (`shortest_paths.c:329`). Both return `Option<f64>` — `None` when
  the value is undefined (n<2 for density without loops, no connected
  pairs for mean_distance). Density's float-arithmetic ordering exactly
  matches upstream's `m / n * 2 / (n - 1)` form so f64 results agree
  bit-for-bit with python-igraph.
  Full 9-step SOP: 12 unit tests (mostly small-graph corner cases),
  1 oracle test on karate (density ≈ 0.139, mean ≈ 2.408), 6
  three-source conformance fixtures, 2 proptest invariants
  (`density >= 0`, `mean_distance >= 1.0`).
- *(test-helpers)* `tests/conformance::json_approx_eq` replaces the
  strict `assert_eq!` on `serde_json::Value`. Number nodes compare with
  a 1e-12 relative-or-absolute tolerance; everything else (booleans,
  arrays, objects, strings) still compares exactly. Resolves a
  cross-language f64 JSON round-trip mismatch where the same f64
  prints as 17 digits via Rust serde_json and 16 digits via Python
  `json.dumps`, with the 16-digit form re-parsing to a ULP-different
  f64. Caught by density's Zachary fixture.

- *(properties)* **ALGO-PR-002b**: local transitivity per-vertex
  (`transitivity_local_undirected`). Returns `Vec<Option<f64>>` —
  `None` when a vertex has simple-degree < 2 (matches upstream's
  `IGRAPH_TRANSITIVITY_NAN` mode). Counterpart of
  `igraph_transitivity_local_undirected()` from
  `references/igraph/src/properties/triangles.c:369`.
  Implementation reuses the PR-002 acyclic-orientation triangle scan
  but tallies `+1` to all three vertices on each detected triangle,
  divided afterwards by `d * (d - 1) / 2` per vertex.
  Full 9-step SOP: 4 unit tests (triangle / star / isolated /
  diamond), 1 oracle test on karate (with 1e-12 tolerance to absorb
  JSON f64 round-trip), 3 three-source conformance fixtures (igraph C
  K4 → all 1.0; python-igraph star → centre 0, leaves None; R triangle
  → all 1.0), 1 proptest invariant (sum of per-vertex triangle counts
  equals `3 * total_triangles`), no separate bench (shares PR-002's
  helper).

- *(properties)* **ALGO-PR-002**: triangle count + global transitivity
  (`count_triangles`, `transitivity_undirected`). Counterparts of
  `igraph_count_triangles()` and `igraph_transitivity_undirected()` from
  `references/igraph/src/properties/triangles.c`. Acyclic-orientation
  algorithm (`v < u` trick) counts each triangle exactly once in
  `O(|V|*d^2)`. Self-loops, parallel edges, and edge directions ignored
  (matches upstream `IGRAPH_NO_LOOPS, IGRAPH_NO_MULTIPLE` adjlist).
  `transitivity_undirected` returns `Option<f64>` — `None` for "no
  connected triples" (upstream's `IGRAPH_TRANSITIVITY_NAN` mode); the
  `IGRAPH_TRANSITIVITY_ZERO` behaviour is `result.unwrap_or(0.0)`.
  Full 9-step SOP: 11 unit tests (empty / isolated / triangle / K4 /
  cycle-4 / star / path / self-loop / parallel-edges / disjoint-pair /
  diamond), 1 oracle test on karate (`triangles == 45`,
  `transitivity ≈ 0.2557`) against python-igraph 0.11
  (using `len(g.list_triangles())` since `count_triangles` isn't
  exposed), 7 three-source conformance fixtures (igraph C K4 + Zachary
  karate from `global_transitivity.c`; python-igraph 5-cycle, K4-minus-edge;
  R path-3 from `test-aaa-auto.R`), 1 proptest coherence invariant
  (`3 * triangles == 3 * triangles ≤ triples` and
  `transitivity == 3 * triangles / triples`), criterion baseline
  ≈ 2.7 µs on karate.

- *(paths)* **ALGO-SP-020**: eccentricity / radius / diameter (unweighted,
  undirected or `IGRAPH_OUT` mode). Returns `Vec<u32>` / `Option<u32>` /
  `Option<u32>` respectively. Counterparts of `igraph_eccentricity()`
  (`distances.c:257`), `igraph_radius()` (`distances.c:345`), and
  `igraph_diameter()` (`shortest_paths.c:1259`). All three are
  BFS-from-each-vertex driven by the existing SP-006 `distances`
  primitive; unreachable pairs ignored (upstream's `unconn=true` default).
  Full 9-step SOP: 9 unit tests (empty / singleton / isolated /
  path-5 / cycle-4 / star / disconnected / directed-path /
  self-loop), 1 oracle test sweeping eccentricity/radius/diameter on
  karate against python-igraph 0.11, 9 three-source conformance
  fixtures (3 algos × 3 sources, including igraph C
  `igraph_diameter.c` directed-ring(10) and R `test-structural-properties.R`
  disjoint-trees `unconnected=TRUE` case), 1 proptest coherence
  invariant (`radius == min(ecc) ∧ diameter == max(ecc)`), criterion
  baselines ≈ 92 µs / 88 µs on karate (34 calls to `distances` at
  ~2.5 µs each).

- *(properties)* **ALGO-PR-001**: girth (`girth`). Returns `Option<u32>` —
  `None` for acyclic graphs (mapped from upstream's `IGRAPH_INFINITY`),
  `Some(k)` for the shortest cycle length. Counterpart of
  `igraph_girth()` from `references/igraph/src/properties/girth.c:73`.
  Itai-Rodeh BFS-from-every-vertex with early termination on triangle
  (girth = 3). Self-loops and parallel edges ignored.
  Full 9-step SOP: 12 unit tests (empty / singleton / isolated / tree
  / triangle / 4-cycle / pentagon / K4 / self-loop / parallel-edges /
  two-components / pendant), 2 oracle tests against python-igraph 0.11
  (small sweep + karate), 4 three-source conformance fixtures
  (igraph C examples/simple/igraph_girth.c ring(100)+chord and
  null-graph; python-igraph 5-cycle; R `test-structural-properties.R`
  make_ring(100)), 1 proptest bounds-and-forest invariant, criterion
  baseline ≈ 2.6 µs on karate, ≈ 79 µs on ring-100 (worst case).
  New top-level module `src/algorithms/properties/`.
- *(test-helpers)* `tests/common::run_ok` now returns `Value::Null` when
  the oracle's `result` field is JSON `null` (previously panicked via
  `Option::expect`). First needed by girth's "no cycle" → `null` wire
  format; future `Option<T>`-returning AWUs benefit automatically.

- *(connectivity)* **ALGO-CC-013**: `is_biconnected`. Returns `bool`.
  Counterpart of `igraph_is_biconnected()` from
  `references/igraph/src/connectivity/components.c:1254-1379`.
  Phase-1 minimal slice delegates to existing `connected_components` +
  `articulation_points` for n ≥ 3, with explicit n < 2 / n == 2 special
  cases (matches upstream's "two-vertex graph with one connecting edge
  is biconnected" convention).
  Full 9-step SOP: 12 unit tests (empty / singleton / two-no-edge /
  two-with-edge / triangle / path-3 / 4-cycle / K4 / disconnected /
  star / cycle+pendant / triangle+isolate), 1 oracle test sweeping
  4 graphs against python-igraph 0.11, 4 three-source conformance
  fixtures (igraph C `igraph_is_biconnected.c` two-triangles-share-vertex
  + ring-10; python-igraph K4; R `path_graph(n=3)` from `test-aaa-auto.R`),
  1 proptest invariant (n ≥ 3: `is_biconnected ≡ cc.count == 1 ∧ aps.is_empty()`),
  no standalone bench (cost is sum of CC-001 + CC-010 ≈ 7.3 µs on karate).
  A bespoke single-DFS-with-early-exit ports later as a perf pass.

- *(connectivity)* **ALGO-CC-014**: bridges (`bridges`). Returns
  `Vec<EdgeId>` of edges whose removal would increase the number of
  weak connected components. Counterpart of `igraph_bridges()` from
  `references/igraph/src/connectivity/components.c:1400-1504`.
  Tarjan-style iterative DFS with low-link tracking + per-vertex
  *incoming-edge* tracking (rather than parent-vertex), so multigraphs
  with parallel edges are handled correctly. Treats input as
  undirected (matches upstream `IGRAPH_ALL` mode default).
  Full 9-step SOP: 10 unit tests (empty / isolated / cycle / path /
  cycle-with-pendant / parallel-edges / self-loop / two-components /
  two-triangles-joined-by-bridge / star), 2 oracle tests against
  python-igraph 0.11 (karate, two-triangles-joined), 4 three-source
  conformance fixtures (igraph C `igraph_bridges.c` 7v two-triangles
  + multigraph; python-igraph 4-path; R
  `make_graph("krackhardt_kite")` from `test-components.R`),
  1 brute-force proptest invariant (an edge is a bridge iff removing
  it splits its endpoints across distinct weak components on small
  random graphs), criterion baseline ≈ 3.8 µs on karate, ≈ 73 µs on
  path-1k.

  Oracle helper: `rust_bridge_pairs` / `py_bridge_pairs` in
  `tests/oracle.rs` resolve edge ids to canonical `(min, max)`
  endpoint pairs because `GraphPayload::from_graph` re-numbers edges
  on the wire.

- *(connectivity)* **ALGO-CC-010**: articulation points
  (`articulation_points`). Iterative DFS with low-link tracking,
  mirroring `igraph_articulation_points()` (which itself reduces to
  `igraph_biconnected_components(_, NULL, NULL, NULL, NULL, &result)`)
  from `references/igraph/src/connectivity/components.c:969-1209`.
  Treats input as undirected (matches `IGRAPH_ALL` mode default at
  `components.c:1060`). Returns vertex ids in upstream's
  DFS-discovery order; conformance runner sorts before comparing
  because the order differs across reference impls.
  Full 9-step SOP: 11 unit tests (empty / isolated / cycle / path /
  star / cycle-with-pendant / multi-component / upstream
  biconnected_components fixture / self-loop / parallel edges /
  two-triangles-sharing-vertex), 2 oracle tests against python-igraph
  0.11 (karate + cycle-with-pendant), 3 three-source conformance
  fixtures (igraph C `igraph_biconnected_components.c` 10-vertex
  graph; python-igraph Tree(5,2); R `path_graph(n=3)` from
  `test-aaa-auto.R`), 1 brute-force proptest invariant (a vertex is
  an articulation point iff removing it splits its neighbourhood
  across multiple weak components), criterion baseline ≈ 3.2 µs on
  karate, ≈ 71 µs on a 1000-path. Full multi-output
  biconnected_components (vertex sets, edge sets, spanning trees)
  ships separately as CC-011.

- *(paths)* **ALGO-CC-040**: Eulerian path/cycle existence test
  (`is_eulerian`). Returns `EulerianClassification { has_path, has_cycle }`.
  Counterpart of `igraph_is_eulerian()` from
  `references/igraph/src/paths/eulerian.c:333` (incl. its undirected /
  directed helpers). Per-vertex degree balance + weak-connectivity
  precondition; correctly handles isolated vertices, singletons with
  self-loops, parallel edges, multiple disconnected self-loops, etc.
  Full 9-step SOP: 11 unit tests (empty / single / isolated / undirected
  path / triangle / disconnected components / K4 / triangle-with-self-loop
  / directed cycle / directed path / directed imbalanced), oracle
  **deferred** because python-igraph 0.11.x exposes no Eulerian API at
  all, 5 three-source conformance fixtures (igraph C: undirected path-3 /
  triangle / two disconnected directed edges from `igraph_is_eulerian.c`;
  R-igraph: 4-cycle and 6-vertex non-cycle from `test-eulerian.R`),
  1 proptest invariant (cycle ⇒ path), criterion baseline ≈ 4.7 µs on
  karate, ≈ 94 µs on a 1000-vertex cycle.
  Concrete path/cycle construction (Hierholzer) is deferred to CC-041/042.
- *(test-helpers)* `tests/conformance.rs::run_conformance_with_skip`
  allows omitting a specific source from the "all three sources must
  contribute" assertion. Used by CC-040 (`skip_sources = ["py"]`)
  and documented in `.codefuse/tracking/CONFORMANCE.md`.

- *(paths)* **ALGO-SP-006**: single-source unweighted shortest-path
  distances (`distances`). Returns `Vec<Option<u32>>` where `None`
  corresponds to upstream's `IGRAPH_INFINITY` (vertex unreachable).
  Counterpart of
  `igraph_distances(_, NULL_weights, _, single_from, all_to, IGRAPH_OUT)`
  from `references/igraph/src/paths/unweighted.c:273-325` — BFS scan
  where the first dequeue of a vertex records its shortest-path length.
  Full 9-step SOP: 9 unit tests (empty / single / invalid-source / path
  / disjoint-components / self-loop / parallel edges / directed-out /
  cycle minimum), 2 oracle tests (karate single-source + directed
  3-chain), 3 three-source conformance fixtures (igraph C: kary_tree
  20/2 from bfs_simple.c; python-igraph: Tree(10,2) from
  test_iterators.testBFS; R-igraph: ring(10) from
  test-structural-properties.R), 2 proptest invariants (BFS-reachability
  parity; triangle inequality on every edge), criterion baseline
  ≈ 2.5 µs on karate, ~52 ns/vertex on path graphs to n=10k.
- *(module)* `src/algorithms/paths/` — new top-level subtree mirroring
  upstream `references/igraph/src/paths/`. Houses SP-* AWUs.

- *(connectivity)* **ALGO-CC-002**: strongly connected components
  (`strongly_connected_components`). Returns the same
  `ConnectedComponents { membership, count }` shape as weak components.
  Counterpart of
  `igraph_connected_components(_, _, _, _, IGRAPH_STRONG)` from
  `references/igraph/src/connectivity/components.c:203-386` —
  iterative two-pass Kosaraju (forward DFS → post-order; reverse via
  in-edges → SCCs). Membership labels match
  `python-igraph.connected_components(mode='strong')` exactly because
  both implementations follow Kosaraju's natural grandfather-pop order.
  Undirected graphs delegate to `connected_components` (their SCCs equal
  their WCCs). Full 9-step SOP: 11 unit tests, 2 oracle tests
  (two-disjoint-3-cycles + cycle-with-tail), 4 three-source conformance
  fixtures (igraph C: components.c two-3-cycles + directed-2-path;
  python-igraph: directed-4cycle-with-tail; R-igraph: test-components.R
  literal-graph A→B→C→A→D, isolate E), 2 proptest invariants (dense
  ids; SCC partition refines the underlying weak components), criterion
  baseline ≈ 4.49 µs on karate-as-directed, 2.44 µs on karate-undirected
  (delegate path), ≈ 86 ns/vertex on directed cycles up to n=10000.
- *(test-helper)* `tests/common/mod.rs::GraphPayload::from_graph` now
  honours `g.is_directed()` and reconstructs directed edge lists via
  out-neighbours (was hardcoded `directed=false`, dropping reverse
  edges). Caught by SCC's first oracle run.
- *(core)* internal helpers `Graph::out_neighbors_vec(v)` /
  `Graph::in_neighbors_vec(v)` (`pub(crate)`) used by direction-aware
  algorithms. Public mode-aware `neighbors(_, mode)` ships in a future
  AWU.

- *(connectivity)* **ALGO-CC-001**: weakly connected components
  (`connected_components`). Returns `ConnectedComponents { membership,
  count }`. Counterpart of
  `igraph_connected_components(_, _, _, _, IGRAPH_WEAK)` —
  BFS-based per-component scan, dense ids assigned in vertex-id order.
  Full 9-step SOP: 7 unit tests (empty / isolated / path / disjoint /
  self-loop / directed-weak / dense-ids invariant), 2 oracle tests
  against python-igraph (karate single component, two-disjoint), 4
  three-source conformance fixtures (igraph C: 2 isolated + path-5;
  python-igraph: two K3 cliques; R-igraph: two K5 cliques from
  test-components.R), 2 proptest invariants
  (membership length + dense ids; CC component of vertex 0 == BFS
  reachable set), criterion baseline ≈ 4.1 µs on karate.

- *(test-helpers)* `tests/conformance.rs::run_conformance` now takes
  a `serde_json::Value` for both `actual` and `expected` so non-list
  result shapes compare structurally. The previous BFS/DFS Vec<u32>
  shape is wrapped via `serde_json::json!(order)` in the per-test
  closure. CC is the first AWU with `{membership, count}` result.

- *(traversal)* **ALGO-TR-002**: depth-first search (`dfs`). Pre-order
  visit, single root, reachable component only. Counterpart of
  `igraph_dfs()` in `references/igraph/src/graph/visitors.c:479`. Full
  9-step AWU SOP run: 7 unit tests, 2 oracle tests against
  python-igraph 0.11.x (synthetic 4-node + karate full match), 3-source
  conformance (igraph C kary_tree(20,2) / python-igraph testDFS
  Tree(10,2) / R-igraph make_star(3) directed), 2 proptest invariants
  (no-duplicate visit, BFS/DFS reach the same set), criterion baseline
  ≈ 1.84 µs on karate.
- *(core)* `Graph::neighbors(v)` now returns neighbours sorted
  ascending by id, matching upstream igraph's
  `igraph_neighbors(_, _, _, IGRAPH_ALL)` and python-igraph's
  `Graph.neighbors(v)`. Implementation: `rebuild_indexes` does a stable
  pair-sort `(from, to)` (was counting-sort by `from` only), and the
  undirected branch of `neighbors()` does a merge over the two
  pre-sorted sublists. Caught by the DFS oracle test on the synthetic
  4-vertex case during the AWU.
- *(test-helper)* `tests/conformance.rs::build_graph` now honours
  `payload.directed` (was always undirected). Caught by R's
  `make_star(3)` DFS fixture.
- *(core)* **ALGO-CORE-001b**: edge-id helpers and incident-edges query.
  `edge(eid)` / `edge_source(eid)` / `edge_target(eid)` / `edge_other(eid, vid)`
  / `incident(v)`. Counterparts of `igraph_edge` and the `IGRAPH_FROM` /
  `IGRAPH_TO` / `IGRAPH_OTHER` macros from `igraph_interface.h`, and a
  simplified `igraph_incident` (default `IGRAPH_LOOPS_TWICE` semantics
  matching `neighbors`). `IgraphError::EdgeOutOfRange` surfaces invalid
  edge ids; `edge_other` errors with `InvalidArgument` when the supplied
  vertex is not actually an endpoint. The full mode-aware `incident`
  (IGRAPH_IN / IGRAPH_OUT / IGRAPH_ALL) ships in a later AWU once
  algorithms need it.
- *(core)* **ALGO-CORE-001a**: real `Graph` (replaces the Phase-0 throwaway
  `Graph<u32>`). Indexed-edgelist storage matching upstream igraph's
  `igraph_t` (`from`/`to`/`oi`/`ii`/`os`/`is`). New surface: `Graph::new(n,
  directed)`, `is_directed()`. Phase-0 method signatures
  (`with_vertices` / `add_edge` / `add_edges` / `vcount` / `ecount` /
  `neighbors` / `degree`) preserved so existing callers compile unchanged.
- *(core)* `Graph` derives `Clone` (deep) and `Default`.
- *(core)* Phase-0 split for `igraph_t`: ALGO-CORE-001a..e tracked in
  `.codefuse/tracking/ALGORITHMS.md`. Subsequent AWUs add `incident`,
  edge-id helpers, deletion, edge-list queries, and the property cache.

### Changed
- *(core)* `Graph::neighbors(v)` now returns `Vec<VertexId>` instead of
  `&[VertexId]`. The indexed-edgelist backend cannot offer a contiguous
  slice cheaply (out-neighbours live in `to[oi[os[v]..os[v+1]]]`).
  Iteration call sites no longer use the `&w` pattern; `for w in
  graph.neighbors(v)?` is the new shape. Affects `bfs` and the test
  helper in `tests/common/mod.rs`.
- *(core)* `Graph::degree(v)` for undirected graphs now counts
  self-loops as 2 (matching upstream `IGRAPH_LOOPS_TWICE` default at
  `type_indexededgelist.c:1162`). Phase-0 counted them as 1.
- *(test-helper)* `tests/common/mod.rs::GraphPayload::from_graph`
  rebuilds the edge list from `neighbors()` correctly when self-loops are
  present (was correct before only because `Graph::neighbors` returned a
  single entry per self-loop; the new backend reports 2).
- *(documentation)* `CHANGELOG.md` (Keep a Changelog 1.1.0).
- *(documentation)* `SECURITY.md` — vulnerability reporting via GitHub
  Security Advisory or email. Slim, alpha-appropriate scope.
- *(documentation)* `DEVELOPMENT.md` — maintainer + AI-agent setup notes
  (renamed from the original `CONTRIBUTING.md`, which was honestly never
  an external-contributor guide).
- *(documentation)* New minimal `CONTRIBUTING.md` — "alpha; not accepting
  external PRs yet; here is when that opens up".
- *(tooling)* `.editorconfig` — cross-IDE consistency for files cargo fmt
  does not touch (md / yml / toml / py / sh).
- *(ci)* `.github/dependabot.yml` — weekly cargo + GitHub Actions + pip
  updates; minor/patch grouped, breaking changes split.
- *(documentation)* README badges: crates.io / docs.rs / CI / license / MSRV.

### Changed
- *(claude-code)* `.claude/settings.json` is committed again (Anthropic's
  intended pattern). The `.sample` workaround attempted earlier is
  removed; the `.githooks/pre-commit` hook is the safety net against
  accidental personal-grant commits.
- *(claude-code)* `.githooks/pre-commit` output bug fixed (entries with
  spaces no longer split into multiple "lines"); reference now points to
  DEVELOPMENT.md.

### Explicitly deferred
- `CODE_OF_CONDUCT.md`, `CITATION.cff`, `.github/ISSUE_TEMPLATE/`,
  `.github/CODEOWNERS`, `SUPPORT.md`. These artifacts presume an external
  community / academic citations / multiple reviewers; none of those
  exist at 0.0.1-alpha.0. Add when the corresponding signal arrives.


First publish to crates.io. Reserves the `rust-igraph` name and exercises
the release pipeline end-to-end.

### Added
- Walking-skeleton public surface:
  - `Graph` (undirected, unweighted, `u32` vertex ids, adjacency-list
    backed; Phase-0 minimal port of `igraph_t`)
  - `read_edgelist` (Phase-0 port of `igraph_read_graph_edgelist`)
  - `bfs` (single-root variant of `igraph_bfs`)
  - `IgraphError` + `IgraphResult` (initial 8 variants)
- Three-source conformance suite for BFS:
  - igraph C: 2 fixtures (path-10, kary_tree-20-2)
  - python-igraph: 1 fixture (Tree-10-2)
  - R-igraph: 1 fixture (ring-10)
- Live oracle test against python-igraph 0.11.x (BFS on Karate matches
  exactly).
- Property tests (proptest): no-duplicate-visit + reachability symmetry.
- Criterion bench: BFS on Karate ≈ 693 ns.
- mdBook scaffold; rustdoc auto-published to GitHub Pages on every push
  to `main`.
- `examples/bfs_karate.rs` smoke demo.
- AI-engineering scaffolding (committed): `CLAUDE.md`, 7 sub-agents under
  `.claude/agents/`, 9 skills under `.claude/skills/` (the `/awu-*`
  family + `/oracle-add` + `/phase-checkpoint` + `/resume-session`),
  3 hooks under `.claude/hooks/` (block-dangerous-git, post-edit-rust,
  post-tool-bash), prompt cookbook in `.codefuse/tracking/AI_PROMPTS.md`.
- AWU SOP infrastructure: 4 templates (`templates/{algo,test,oracle,bench}.{rs,py}.tpl`),
  ALGORITHMS / ARCHITECTURE / CONFORMANCE / RESUME tracking under
  `.codefuse/tracking/`.
- CI matrix: fmt + clippy (stable + MSRV 1.85) + test (stable on
  Linux/macOS, beta on Linux) + conformance + oracle + proptest +
  cargo-deny + wasm32 check + rustdoc -D warnings + GitHub Pages deploy +
  release workflow (tag-triggered cargo publish).

### Status
- API: alpha. WILL break before 0.1.0.
- 850-API parity: ~0% (only `Graph`, `read_edgelist`, `bfs` shipped).
- See [`docs/plans/MASTER_PLAN.md`](docs/plans/MASTER_PLAN.md) and
  [`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md)
  for the roadmap.

[Unreleased]: https://github.com/Totoro-jam/rust-igraph/compare/v0.0.1-alpha.0...HEAD
[0.0.1-alpha.0]: https://github.com/Totoro-jam/rust-igraph/releases/tag/v0.0.1-alpha.0
