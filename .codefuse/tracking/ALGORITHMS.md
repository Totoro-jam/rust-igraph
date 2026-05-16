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
| ALGO-CORE-001c | delete_edges + delete_vertices + delete_vertices_map | type_indexededgelist.c (lines 500-825) | ~400 | adapt | CORE-001a | todo | - | - | - |
| ALGO-CORE-001d | edge/edges + get_eid/get_eids/get_all_eids_between | type_indexededgelist.c (lines 1522-1773) | ~250 | adapt | CORE-001a | todo | - | - | - |
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
| ALGO-TR-001 | BFS (full callback variant) | bfs.c | 300 | adapt | CORE-001 | partial | 5779a31 | 693 ns/karate | C:2 / py:1 / R:1 |
| ALGO-TR-002 | DFS (single-root pre-order) | visitors.c:479 | 200 | adapt | CORE-001a/b | done | (next) | 1.84 µs/karate | C:1 / py:1 / R:1 |
| ALGO-TR-003 | Random walk | random_walk.c | 340 | adapt | CORE-001 | todo | - | - | - |
| ALGO-SP-001 | Dijkstra | distances_dijkstra*.c | 1235 | adapt | TR-001 | todo | - | - | - |
| ALGO-SP-002 | Bellman-Ford | distances_bellman_ford*.c | 591 | adapt | TR-001 | todo | - | - | - |
| ALGO-SP-003 | Johnson | distances_johnson.c | 254 | adapt | SP-001,002 | todo | - | - | - |
| ALGO-SP-004 | Floyd-Warshall | distances_floyd_warshall.c | 365 | adapt | - | todo | - | - | - |
| ALGO-SP-005 | A* | astar.c | 273 | adapt | TR-001 | todo | - | - | - |
| ALGO-SP-006 | BFS shortest paths (unweighted) | shortest_paths*.c | 703 | adapt | TR-001 | todo | - | - | - |
| ALGO-SP-010..014 | Widest paths | widest_paths*.c | 741 | adapt | - | todo | - | - | - |
| ALGO-SP-020..023 | Diameter / eccentricity / radius | diameter.c, eccentricity.c | ~400 | adapt | - | todo | - | - | - |
| ALGO-CC-001 | Weakly connected components | components.c:82-180 | 100 | adapt | TR-001 | done | (next) | 4.1 µs/karate | C:2 / py:1 / R:1 |
| ALGO-CC-002 | Strongly connected components (Kosaraju) | components.c:203-386 | 184 | adapt | CC-001 | done | (next) | 4.49 µs/karate-dir | C:2 / py:1 / R:1 |
| ALGO-CC-003 | Decompose graph by components | components.c:566-732 | 350 | adapt | CC-001,002 | todo | - | - | - |
| ALGO-CC-010..014 | Biconnected / articulation / bridges | biconnected*.c | ~600 | adapt | TR-002 | todo | - | - | - |
| ALGO-CC-020..022 | Reachability | reachability.c | 257 | adapt | TR-001 | todo | - | - | - |
| ALGO-CC-030..032 | Percolation | percolation.c | 404 | adapt | - | todo | - | - | - |
| ALGO-CC-040..042 | Eulerian paths/cycles | eulerian*.c | 681 | adapt | - | todo | - | - | - |

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
| 1 | 5 | 0 | ~80 | ~85 | dfs: 3 (C:1, py:1, R:1); cc: 4 (C:2, py:1, R:1); scc: 4 (C:2, py:1, R:1) |
| 2-10 | 0 | 0 | ~543 | ~543 | - |

**Phase 0 — complete (37/37)**. **Phase 1 underway**: 5/85 done — Graph
core (CORE-001a/b), DFS (TR-002), weak CC (CC-001), strong CC (CC-002).
Next options: SP-001 (Dijkstra; needs weighted-edge extension to Graph),
CC-003 (decompose), or CC-010 (biconnected components, builds on TR-002
DFS).

> Update the counters after every PR merge.
