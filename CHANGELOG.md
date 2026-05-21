# Changelog

All notable changes to **rust-igraph** are recorded here.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning 2.0](https://semver.org/spec/v2.0.0.html).

> Pre-1.0 contract: every minor bump (0.x.y → 0.(x+1).0) may break the
> public API. Patch bumps are bug-fixes / new additive items only.

## [Unreleased]

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
