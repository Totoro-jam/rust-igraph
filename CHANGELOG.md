# Changelog

All notable changes to **rust-igraph** are recorded here.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning 2.0](https://semver.org/spec/v2.0.0.html).

> Pre-1.0 contract: every minor bump (0.x.y → 0.(x+1).0) may break the
> public API. Patch bumps are bug-fixes / new additive items only.

## [Unreleased]

### Added
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


## [0.0.1-alpha.0] — 2026-05-15

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
