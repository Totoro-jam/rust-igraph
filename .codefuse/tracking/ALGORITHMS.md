# rust-igraph Algorithm Work Unit (AWU) tracker

Single source of truth for per-algorithm progress. **Update on every PR.**
See [docs/plans/MASTER_PLAN.md](../../docs/plans/MASTER_PLAN.md) §4 for the
9-step SOP and §5.2 for the full Phase 1-10 algorithm catalog.

## Status legend

- `todo` — not started
- `wip` — implementation in progress (not on `main`)
- `review` — PR open, waiting for review
- `done` — merged to `main`, all 9 steps green
- `verified` — passed nightly full-conformance for ≥7 days
- `blocked` — waiting on a prerequisite AWU or external decision
- `perf-todo` — functionally `done` but performance > python-igraph × 3

## Complexity legend

- `copy` — 1:1 translation from igraph C, ~30 LOC/h with AI
- `adapt` — needs Rust ownership/lifetimes adaptation, ~20 LOC/h
- `rewrite` — algorithmic re-design (e.g. C++ class hierarchies), ~12 LOC/h
- `novel` — no C reference, design from scratch, ~8 LOC/h

## Phase 0 — Walking skeleton + infrastructure (37 BOOT tasks)

| ID | Task | Status | Commit |
|----|------|--------|--------|
| BOOT-01 | git init + LICENSE + README skeleton | done | 2ce55aa |
| BOOT-02 | Cargo workspace + 3 crate skeleton | done | 2ce55aa |
| BOOT-03 | IgraphError + IgraphResult | done | 2ce55aa |
| BOOT-04 | Minimal Graph<u32> | done | 2ce55aa |
| BOOT-05 | EdgeList reader | done | 2ce55aa |
| BOOT-06 | Minimal BFS | done | 2ce55aa |
| BOOT-07 | example bfs_karate.rs | done | 2ce55aa |
| BOOT-08 | karate.edges fixture | done | 2ce55aa |
| BOOT-09 | scripts/oracle.py + Python venv + requirements.txt | done | 5779a31 |
| BOOT-10 | tests/oracle/ harness (Rust subprocess wrapper) | done | 5779a31 |
| BOOT-11 | First oracle test: BFS on karate | done | 5779a31 |
| BOOT-12 | proptest skeleton + BFS invariant | done | 5779a31 |
| BOOT-13 | criterion bench skeleton + BFS baseline | done | 5779a31 |
| BOOT-14..18 | GitHub Actions CI matrix | done | 8d688d8, a6b1cf8 |
| BOOT-14b | Codecov coverage upload (cargo-llvm-cov + codecov.yml + README badge) | done | (next) |
| BOOT-19..22 | templates/ (algo, test, oracle, bench) | done | (next) |
| BOOT-23 | ALGORITHMS.md (this file) | done | 8d688d8 |
| BOOT-24 | ARCHITECTURE.md | done | (next) |
| BOOT-25 | DEVELOPMENT.md (maintainer setup + AWU SOP) + CONTRIBUTING.md (alpha external policy) | done | (next) |
| BOOT-26 | mdBook docs scaffold | done | (next) |
| BOOT-27 | scripts/bench_compare.py (perf baseline) | done | (next) |
| BOOT-28 | RESUME.md (part-time resumability) | done | (next) |
| BOOT-29 | scripts/test_extract/from_c.py | done | 5779a31 |
| BOOT-30 | scripts/test_extract/from_py.py | done | 5779a31 |
| BOOT-31 | scripts/test_extract/from_r.{R,py} | done | 5779a31 |
| BOOT-32 | tests/conformance/ + three-source BFS test | done | 5779a31 |
| BOOT-33 | CLAUDE.md | done | 8d688d8 |
| BOOT-34 | .claude/agents/ (7 agents) | done | 8d688d8 |
| BOOT-35 | .claude/skills/ (9 skills) | done | 8d688d8 |
| BOOT-36 | .claude/hooks/ + settings.json | done | 8d688d8 |
| BOOT-37 | AI_PROMPTS.md | done | 8d688d8 |

## Phase 1 — Data structures (~80 AWU)

| ID | Task | C source | Lines | Cx | Deps | Status | Commit | Bench | Conformance |
|----|------|----------|-------|----|------|--------|--------|-------|-------------|
| ALGO-CORE-001a | Graph struct + new/with_vertices + add_{vertices,edges,edge} + vcount/ecount/is_directed + neighbors/degree + Clone | type_indexededgelist.c (lines 99-413, 829-1265) | ~700 | adapt | - | done | (next) | - | - |
| ALGO-CORE-001b | incident + edge-id helpers (edge / edge_source / edge_target / edge_other) | type_indexededgelist.c (lines 1775-1912) | ~250 | adapt | CORE-001a | done | (next) | - | - |
| ALGO-CORE-001c | delete_edges + delete_vertices + delete_vertices_map | type_indexededgelist.c (lines 500-825) | ~400 | adapt | CORE-001a | done | (next) | O(V+E) per call (rebuild_indexes) | no fixtures (structural mutation; 16 unit + 2 proptest cover) |
| ALGO-CORE-001d | edge query helpers (`get_eid`, `find_eid`, `get_all_eids_between`) | type_indexededgelist.c:1522-1773 | ~150 | adapt | CORE-001a | done | (next) | O(deg(from)) linear scan | - |
| ALGO-CORE-001e | is_same_graph + property cache subsystem | type_indexededgelist.c + cache_*.c | ~200 | adapt | CORE-001a..d | todo | - | - | - |
| ALGO-CORE-010 | basic queries (vcount, ecount, degree, ...) | basic_query.c | 406 | copy | CORE-001a | partial | 2ce55aa | - | - |
| ALGO-DS-V-001..030 | Vector / VectorInt / VectorBool | vector.c | ~2500 | adapt | - | todo | - | - | - |
| ALGO-DS-M-001..020 | Matrix / MatrixInt | matrix.c | ~1500 | adapt | - | todo | - | - | - |
| ALGO-DS-S-001..010 | SparseMatrix CSR/CSC | sparsemat.c | 3251 | rewrite | - | todo | - | - | - |
| ALGO-DS-SEL-001..010 | VertexSelector / EdgeSelector | iterators.c | 2048 | adapt | - | todo | - | - | - |
| ALGO-DS-ADJ-001..005 | Adjacency lists | adjlist.c | 1328 | adapt | - | todo | - | - | - |

## Phase 2 — Traversal + Shortest Paths + Connectivity (~45 AWU)

| ID | Task | C source | Lines | Cx | Deps | Status | Commit | Bench | Conformance |
|----|------|----------|-------|----|------|--------|--------|-------|-------------|
| ALGO-TR-001 | BFS multi-output (`bfs_tree`: order + distances + parents) | bfs.c | 300 | adapt | CORE-001 | done | (next) | (BFS + bookkeeping) | C:2 / py:1 / R:1 (Phase 0 BFS fixtures cover) |
| ALGO-TR-002 | DFS (single-root pre-order) | visitors.c:479 | 200 | adapt | CORE-001a/b | done | (next) | 1.84 µs/karate | C:1 / py:1 / R:1 |
| ALGO-TR-003 | Random walk | random_walk.c | 340 | adapt | CORE-001 | done | (next) | O(steps · max_deg) per call | no fixtures (RNG-dependent; proptest covers structural invariants) |
| ALGO-SP-001 | Dijkstra single-source distances (`dijkstra_distances`, OUT mode) | paths/dijkstra.c:322-331 | ~250 | adapt | TR-001 | done | (next) | O(E log V + V) heap | C:1 / py:1 / R:1 |
| ALGO-SP-001b | Dijkstra paths/parents + multi-source + cutoff | paths/dijkstra.c | ~400 | adapt | SP-001 | done | (next) | O(E log V + V) heap | C:3 / py:3 / R:3 (paths + path_to + cutoff) |
| ALGO-SP-001c | Dijkstra IN/ALL mode + all-shortest-paths | paths/dijkstra.c | ~250 | adapt | SP-001b | done | (next) | O(E log V + V) heap | C:2 / py:2 / R:2 (dist_with_mode + all_shortest_paths) |
| ALGO-SP-002 | Bellman-Ford | distances_bellman_ford*.c | 591 | adapt | TR-001 | todo | - | - | - |
| ALGO-SP-003 | Johnson | distances_johnson.c | 254 | adapt | SP-001,002 | todo | - | - | - |
| ALGO-SP-004 | Floyd-Warshall (`floyd_warshall_distances`, original variant) | paths/floyd_warshall.c:270-365 | 365 | adapt | - | done | (next) | O(V³) triple-loop | C:1 / py:1 / R:1 |
| ALGO-SP-005 | A* | astar.c | 273 | adapt | TR-001 | done | (next) | O((V+E)log V), better w/ admissible heuristic | C:1 / py:1 / R:1 |
| ALGO-SP-006 | BFS distances (single-source, unweighted, OUT mode) | unweighted.c:273-325 | 240 | adapt | TR-001 | done | (next) | 2.5 µs/karate | C:1 / py:1 / R:1 |
| ALGO-SP-010..014 | Widest paths | widest_paths*.c | 741 | adapt | - | todo | - | - | - |
| ALGO-SP-020 | Eccentricity / radius / diameter (unweighted) | distances.c:257-363, shortest_paths.c:1259 | ~250 | adapt | SP-006 | done | (next) | ecc 92 µs / rad 88 µs / karate | C:3 / py:3 / R:3 (3 algos × 3 sources) |
| ALGO-SP-021abc | Mode-aware (`*_with_mode` accepting OUT/IN/ALL) eccentricity/radius/diameter | distances.c, shortest_paths.c | ~150 | adapt | SP-020 | done | (next) | BFS reuse, O(V·(V+E)) | C:1 / py:1 / R:1 (× 3 algos = 9 fixtures) |
| ALGO-SP-021..023 | Weighted (Dijkstra-based) eccentricity/radius/diameter | distances.c, shortest_paths.c | ~150 | adapt | SP-001, SP-021abc | done | (next) | O(V·(V+E)logV) Dijkstra-from-each | C:1 / py:1 / R:1 (× 3 algos = 9 fixtures) |
| ALGO-CC-001 | Weakly connected components | components.c:82-180 | 100 | adapt | TR-001 | done | (next) | 4.1 µs/karate | C:2 / py:1 / R:1 |
| ALGO-CC-002 | Strongly connected components (Kosaraju) | components.c:203-386 | 184 | adapt | CC-001 | done | (next) | 4.49 µs/karate-dir | C:2 / py:1 / R:1 |
| ALGO-CC-003 | Decompose graph by components (`decompose`, weak only) | components.c:566-732 | 350 | adapt | CC-001,002 | done | (next) | O(V+E) BFS + edge sweep | C:1 / py:1 / R:1 |
| ALGO-CC-010 | Articulation points (`articulation_points`) | components.c:969-972 (driver at 1085-1209) | ~250 | adapt | TR-002 | done | (next) | 3.2 µs/karate | C:1 / py:1 / R:1 |
| ALGO-CC-014 | Bridges (`bridges`) | components.c:1400-1504 | ~200 | adapt | TR-002 | done | (next) | 3.8 µs/karate | C:2 / py:1 / R:1 |
| ALGO-CC-013 | `is_biconnected` (delegate to CC-001 + CC-010) | components.c:1254-1379 | ~80 | copy | CC-001, CC-010 | done | (next) | (delegate; ≈7.3 µs/karate) | C:2 / py:1 / R:1 |
| ALGO-CC-011 | Biconnected components multi-output (`biconnected_components`) | components.c:1032-1227 | ~250 | adapt | CC-010 | done | (next) | O(V+E) DFS | C:1 / py:1 / R:1 |
| ALGO-CC-012 | Biconnected components: explicit `component_edges` output | components.c:1176-1195 | ~80 | adapt | CC-011 | done | (next) | (CC-011 + O(Σ_v deg(v) per comp)) | C:1 / py:1 / R:1 |
| ALGO-CC-020 | Reachability counts (`count_reachable`) | reachability.c:179 | ~80 | adapt | SP-006 | done | (next) | (BFS-from-each, ≈ vcount * SP-006) | C:1 / py:1 / R:1 |
| ALGO-CC-021 | Reachability matrix (`reachability_matrix`) | reachability.c:72-148 | ~80 | adapt | SP-006 | done | (next) | O(V*(V+E)) BFS-from-each | C:1 / py:1 / R:1 |
| ALGO-CC-022 | Transitive closure (`transitive_closure`) | reachability.c:225-257 | ~80 | adapt | CC-021 | done | (next) | (CC-021 + closure ctor) | C:1 / py:1 / R:1 |
| ALGO-CC-030..032 | Percolation | percolation.c | 404 | adapt | - | todo | - | - | - |
| ALGO-CC-040 | Eulerian existence (`is_eulerian`) | eulerian.c:333 (incl. directed/undirected helpers) | ~280 | adapt | CC-001 | done | (next) | 4.7 µs/karate | C:3 / py:0 / R:2 (py skipped — see CONFORMANCE.md) |
| ALGO-CC-041 | Eulerian path/cycle construction, undirected (Hierholzer) | eulerian.c:345-450 | ~200 | adapt | CC-040 | done | (next) | (CC-040 + O(V+E)) | C:1 / py:0 / R:1 (py skipped) |
| ALGO-CC-042 | Eulerian path/cycle construction, directed (Hierholzer) | eulerian.c:453-560 | (folded into CC-041 module) | adapt | CC-041 | done | (next) | (CC-040 + O(V+E)) | C:1 / py:0 / R:0 (py skipped) |
| ALGO-PR-001 | Girth (shortest cycle length) | properties/girth.c:73 | ~200 | adapt | CC-001 | done | (next) | 2.6 µs/karate | C:2 / py:1 / R:1 |
| ALGO-PR-002 | Triangle count + global transitivity | properties/triangles.c:405-630 | ~250 | adapt | - | done | (next) | 2.7 µs/karate | C:3 / py:2 / R:2 (2 algos × 3 sources) |
| ALGO-PR-002b | Local transitivity per-vertex (`transitivity_local_undirected`) | properties/triangles.c:330+185-280 | ~150 | adapt | PR-002 | done | (next) | (shares PR-002 baseline) | C:1 / py:1 / R:1 |
| ALGO-PR-002c | Barrat weighted transitivity (`transitivity_barrat`) | properties/triangles.c:632 | ~150 | adapt | PR-002b | done | (next) | O(V·d²) (shares PR-002 baseline) | C:1 / py:1 / R:1 |
| ALGO-PR-003 | Density + mean shortest-path length (unweighted) | basic_properties.c:71, shortest_paths.c:329 | ~150 | adapt | SP-006 | done | (next) | density O(1); mean_distance ≈ ecc | C:2 / py:2 / R:2 |
| ALGO-PR-004 | Reciprocity (default mode) | basic_properties.c:325 | ~80 | adapt | - | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-004b | Reciprocity ratio mode + ignore_loops (`reciprocity_with_mode`) | basic_properties.c:325 | ~80 | adapt | PR-004 | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-005 | Average nearest-neighbour degree (`avg_nearest_neighbor_degree`) | properties/degrees.c:263 | ~80 | adapt | - | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-005b | knn weighted + per-degree aggregate (`knnk`) | properties/degrees.c:263 | ~120 | adapt | PR-005 | done | (next) | O(V+E) per call | C:3 / py:3 / R:3 (3 algos × 3 sources) |
| ALGO-PR-006 | Degree assortativity (`assortativity_degree`, undirected) | misc/mixing.c:443 + 273 | ~150 | adapt | - | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-006b | Weighted assortativity (`assortativity_degree_weighted`, undirected) | misc/mixing.c | ~120 | adapt | PR-006 | done | (next) | O(V + E) | C:1 / py:1 / R:1 |
| ALGO-PR-006c | Directed assortativity (`assortativity_degree_directed`, unweighted) | misc/mixing.c:351-405 | ~120 | adapt | PR-006b | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-006d | Directed weighted assortativity | misc/mixing.c:351-405 | ~80 | adapt | PR-006c | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-007 | Closeness centrality (`closeness`, unweighted, IGRAPH_OUT/ALL) | centrality/closeness.c:33+ | ~120 | adapt | SP-006 | done | (next) | O(V*(V+E)) BFS | C:1 / py:1 / R:1 |
| ALGO-PR-007b | Weighted closeness (`closeness_weighted`, Dijkstra) | centrality/closeness.c | ~150 | adapt | PR-007, SP-001 | done | (next) | O(V*(V+E)logV) Dijkstra-from-each | C:1 / py:1 / R:1 |
| ALGO-PR-009 | Harmonic centrality (`harmonic_centrality`, unweighted) | centrality/closeness.c:740-805 | ~80 | adapt | SP-006 | done | (next) | O(V*(V+E)) BFS | C:1 / py:1 / R:1 |
| ALGO-PR-009b | Weighted harmonic centrality (`harmonic_centrality_weighted`) | centrality/closeness.c | ~80 | adapt | PR-009, SP-001 | done | (next) | O(V*(V+E)logV) Dijkstra-from-each | C:1 / py:1 / R:1 |
| ALGO-PR-008 | Betweenness centrality (`betweenness`, Brandes unweighted) | centrality/betweenness.c:504+ | ~120 | adapt | SP-006 | done | (next) | O(V*(V+E)) Brandes | C:1 / py:1 / R:1 |
| ALGO-PR-008b | Weighted betweenness (`betweenness_weighted`, Brandes-Dijkstra) | centrality/betweenness.c | ~150 | adapt | PR-008, SP-001 | done | (next) | O(V*(V+E)logV) | C:1 / py:1 / R:1 |
| ALGO-PR-010 | Edge betweenness (`edge_betweenness`, Brandes unweighted) | centrality/betweenness.c:766+ | ~120 | adapt | PR-008 | done | (next) | O(V*(V+E)) Brandes | C:1 / py:1 / R:1 |
| ALGO-PR-010b | Weighted edge betweenness (`edge_betweenness_weighted`, Brandes-Dijkstra) | centrality/betweenness.c | ~150 | adapt | PR-010, SP-001 | done | (next) | O(V*(V+E)logV) | C:1 / py:1 / R:1 |
| ALGO-PR-011 | PageRank (`pagerank`, power iteration, damping=0.85) | centrality/pagerank.c | ~180 | adapt | - | done | (next) | O(iter*(V+E)) power | C:1 / py:1 / R:1 |
| ALGO-PR-011b | Weighted PageRank (`pagerank_weighted`, power iteration) | centrality/pagerank.c | ~180 | adapt | PR-011 | done | (next) | O(iter*(V+E)) power | C:1 / py:1 / R:1 |
| ALGO-PR-011c | PageRank ARPACK backend | centrality/pagerank.c | ~250 | rewrite | PR-011 | todo | - | - | - |
| ALGO-PR-012 | Eigenvector centrality (`eigenvector_centrality`, undirected) | centrality/eigenvector.c | ~120 | adapt | - | done | (next) | shifted power | C:1 / py:1 / R:1 |
| ALGO-PR-012b | Directed eigenvector + ARPACK + weighted | centrality/eigenvector.c | ~250 | rewrite | PR-012 | todo | - | - | - |
| ALGO-OP-001 | Simplify graph (`simplify`: remove loops + multi-edges) | operators/simplify.c | ~200 | adapt | CORE-001a/d | done | (next) | O(V + E log E) | C:1 / py:1 / R:1 |
| ALGO-OP-002 | Disjoint union (`disjoint_union`, two-graph variant) | operators/disjoint_union.c | ~80 | adapt | CORE-001a | done | (next) | O(V + E) | C:1 / py:1 / R:1 |
| ALGO-OP-002b | Multi-arg `disjoint_union_many` | operators/disjoint_union.c | ~80 | adapt | OP-002 | done | (next) | O(ΣV + ΣE) | C:1 / py:1 / R:1 |
| ALGO-OP-003 | Complementer (`complementer`, configurable loops) | operators/complementer.c | ~100 | adapt | CORE-001a/d | done | (next) | O(V² log E) | C:1 / py:1 / R:1 |
| ALGO-OP-004 | Union of two graphs (`union`, max-multiplicity) | operators/union.c:69 | ~150 | adapt | CORE-001a | done | (next) | O((E1+E2) log(E1+E2)) BTreeMap merge | C:1 / py:1 / R:1 |
| ALGO-OP-005 | Intersection of two graphs (`intersection`, min-multiplicity) | operators/intersection.c:71 | ~150 | adapt | CORE-001a | done | (next) | O((E1+E2) log(E1+E2)) BTreeMap lookup | C:1 / py:1 / R:1 |
| ALGO-OP-006 | Difference of two graphs (`difference`, clamped multiset subtract) | operators/difference.c:54 | ~120 | adapt | CORE-001a | done | (next) | O((E1+E2) log(E1+E2)) BTreeMap lookup | C:1 / py:1 / R:1 |
| ALGO-CO-001 | Modularity (`modularity`, undirected, unweighted, configurable γ) | community/modularity.c | ~150 | adapt | CORE-001a/d | done | (next) | O(V + E) | C:1 / py:1 / R:1 |
| ALGO-PR-013 | `is_simple` predicate (no loops, no multi-edges) | properties/multiplicity.c | ~80 | adapt | CORE-001a | done | (next) | O(V + E) | C:1 / py:1 / R:1 |
| ALGO-PR-013b | `is_simple_with_mode` (directed-as-undirected) | properties/multiplicity.c | ~80 | adapt | PR-013 | done | (next) | O(V+E) / O(E log E) | C:1 / py:1 / R:1 |
| ALGO-PR-014 | `has_loop` + `has_multiple` predicates | properties/loops.c + multiplicity.c | ~100 | adapt | CORE-001a/d | done | (next) | O(E) / O(E log E) | C:2 / py:2 / R:2 |
| ALGO-PR-014b | per-edge `is_loop` + `is_multiple` | properties/loops.c + multiplicity.c | ~80 | adapt | PR-014 | done | (next) | O(E) / O(E log E) | C:2 / py:2 / R:2 |
| ALGO-CO-001b | Directed modularity Leicht-Newman (`modularity_directed`) | community/modularity.c | ~120 | adapt | CO-001 | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-CO-001c | Weighted modularity (`modularity_weighted`, undirected) | community/modularity.c | ~150 | adapt | CO-001 | done | (next) | O(V+E) | C:1 / py:1 / R:1 |
| ALGO-PR-015 | Coreness / k-core decomposition (`coreness`, undirected) | centrality/coreness.c | 157 | adapt | - | done | (next) | O(V+E) Batagelj-Zaversnik | C:1 / py:1 / R:1 |
| ALGO-PR-015b | `coreness_with_mode` (directed IN/OUT) | centrality/coreness.c | ~80 | adapt | PR-015 | done | (next) | O(V+E) | C:1 / py:1 / R:1 |

## Phase 3 — Centrality + Eigensolver (~65 AWU)

> Eigensolver scaffolding (ALGO-LA-*) is the prerequisite for the starred (★)
> centrality algorithms. Full per-AWU table to be expanded when Phase 2 nears
> completion. See MASTER_PLAN.md §5.2 Phase 3.

## Phase 4-10 — see MASTER_PLAN.md

Each phase's per-AWU table is materialized here as work approaches.

---

## Counters

| Phase | done | wip | todo | total | Conformance fixtures |
|-------|------|-----|------|-------|----------------------|
| 0 (BOOT) | 37 | 0 | 0 | 37 | bfs: 4 (C:2, py:1, R:1) |
| 1 | 70 | 0 | ~16 | ~85 | dfs: 3; cc: 4; scc: 4; distances: 3; is_eulerian: 5 (py skipped); articulation: 3; bridges: 4; is_biconnected: 4; girth: 4; ecc/radius/diameter: 9; ecc/radius/diameter_with_mode: 9; triangles+transitivity: 10; transitivity_barrat: 3; density+mean_distance: 6; eulerian_path: 3 (py skipped); count_reachable: 3; reciprocity: 3; knn: 3; assortativity: 3; CORE-001c: no fixtures; CORE-001d: no fixtures; reachability_matrix: 3; transitive_closure: 3; closeness: 3; harmonic: 3; betweenness: 3; edge_betweenness: 3; pagerank: 3; biconnected_components: 3; eigenvector: 3; simplify: 3; modularity: 3; is_simple: 3; has_loop+has_multiple: 6; is_loop+is_multiple: 6; disjoint_union: 3; dijkstra_distances: 3; complementer: 3; closeness_weighted: 3; harmonic_centrality_weighted: 3; betweenness_weighted: 3; edge_betweenness_weighted: 3; pagerank_weighted: 3; assortativity_degree_weighted: 3; assortativity_degree_directed_weighted: 3; floyd_warshall_distances: 3; decompose: 3; union: 3; intersection: 3; difference: 3; dijkstra_paths+path_to+cutoff: 9; dijkstra_with_mode+all_shortest_paths: 6; ecc/radius/diameter_weighted_with_mode: 9; a_star_path: 3 |
| 2-10 | 0 | 0 | ~543 | ~543 | - |

**Phase 0 — complete (37/37)**. **Phase 1 underway**: 69/85 done —
Graph core (CORE-001a/b/d), DFS (TR-002), weak CC (CC-001), strong CC
(CC-002), unweighted distances (SP-006), Eulerian existence (CC-040),
articulation points (CC-010), bridges (CC-014), is_biconnected
(CC-013), girth (PR-001), eccentricity/radius/diameter (SP-020),
triangle count + global transitivity (PR-002), local transitivity
per-vertex (PR-002b), density + mean_distance (PR-003), Eulerian
path/cycle (CC-041 + CC-042), reachability counts (CC-020),
reachability matrix (CC-021), transitive closure (CC-022),
reciprocity (PR-004), BFS multi-output (TR-001), knn (PR-005),
degree assortativity (PR-006), edge query helpers (CORE-001d),
closeness centrality (PR-007), harmonic centrality (PR-009), betweenness
centrality (PR-008), edge betweenness (PR-010), PageRank (PR-011),
biconnected components multi-output (CC-011), eigenvector centrality
(PR-012), simplify (OP-001), modularity (CO-001), is_simple
(PR-013), has_loop + has_multiple (PR-014), per-edge is_loop +
is_multiple (PR-014b), disjoint_union (OP-002), dijkstra_distances (SP-001),
complementer (OP-003), weighted closeness (PR-007b),
weighted harmonic (PR-009b), weighted betweenness (PR-008b),
weighted edge_betweenness (PR-010b), weighted PageRank
(PR-011b), weighted assortativity (PR-006b),
Floyd-Warshall all-pairs (SP-004), coreness / k-core
(PR-015), reciprocity ratio mode + ignore_loops
(PR-004b), weighted modularity (CO-001c), is_simple
directed-as-undirected mode (PR-013b),
disjoint_union_many (OP-002b),
directed coreness IN/OUT (PR-015b),
directed assortativity (PR-006c),
directed modularity (CO-001b),
biconnected component_edges output (CC-012),
weighted knn + knnk + knnk_weighted (PR-005b),
Barrat weighted transitivity (PR-002c),
decompose (CC-003 weak slice),
union (OP-004 two-graph max-multiplicity),
intersection (OP-005 two-graph min-multiplicity),
difference (OP-006 clamped multiset subtract),
mode-aware ecc/radius/diameter (SP-021abc),
Dijkstra paths/path_to/cutoff/multi-source (SP-001b),
Dijkstra IN/ALL + all-shortest-paths (SP-001c),
weighted ecc/radius/diameter (SP-021..023 weighted),
directed weighted assortativity (PR-006d),
A* shortest path with admissible heuristic (SP-005),
random walk (TR-003, deterministic SplitMix64 PRNG).
Next options:
CORE-001c (deletion),
hub/auth scores,
SP-002 Bellman-Ford,
SP-003 Johnson all-pairs.

> Update the counters after every PR merge.
