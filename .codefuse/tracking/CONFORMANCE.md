# Three-source conformance coverage

Auto-checked by `tests/conformance.rs`. Each row is one
algorithm; columns are fixture counts per source plus the Rust pass rate.

> Phase-0 invariant: every `done` AWU must contribute ≥ 1 fixture from
> each of the three sources. CI fails otherwise.

| Algorithm | C | py | R | Rust | Notes |
|-----------|---|----|---|------|-------|
| bfs       | 2 | 1  | 1 | 4/4  | Caught the `igraph_ring(circular=0)` = path bug on first run; see git log of `scripts/test_extract/from_c.py`. |
| dfs       | 1 | 1  | 1 | 3/3  | DFS revealed (a) `neighbors()` was unsorted within from-bucket — fixed `rebuild_indexes` to sort by `(from, to)` lexicographically; (b) `tests/conformance.rs::build_graph` was discarding `directed=true` from R's `make_star(3)` fixture — fixed to honour `payload.directed`. |
| connected_components | 2 | 1  | 1 | 4/4  | First AWU with non-Vec<u32> result — refactored `tests/conformance.rs::run_conformance` to JSON-typed return so a `{membership, count}` shape can be compared structurally. |
| strongly_connected_components | 2 | 1 | 1 | 4/4 | First directed-graph fixture suite. SCC oracle initially failed because `tests/common/mod.rs::GraphPayload::from_graph` hardcoded `directed=false` and dropped reverse edges; fixed by branching on `g.is_directed()` and emitting out-neighbours directly for the directed case. Membership labels match python-igraph exactly because both follow Kosaraju's natural grandfather-pop order (no reindex). |
| distances | 1 | 1 | 1 | 3/3 | Single-source unweighted BFS distances. First AWU returning `Vec<Option<u32>>` — `None` corresponds to upstream's `IGRAPH_INFINITY`. Oracle/wire format encodes `None` as JSON `null` and `Some(d)` as integer. |
| is_eulerian | 3 | 0 | 2 | 5/5 | **Per-source skip**: python-igraph 0.11.x exposes no Eulerian API at all (no `g.is_eulerian` / `has_eulerian_path` etc.) — verified by `[m for m in dir(g) if 'euler' in m.lower()]` returning empty. The conformance harness gained `run_conformance_with_skip(algo, &["py"], …)` for this and similar future cases. Fixtures are pulled from upstream igraph C `tests/unit/igraph_is_eulerian.c` (3) and rigraph `tests/testthat/test-eulerian.R` (2). Re-add `py` once a future python-igraph release exposes the API. |
| articulation_points | 1 | 1 | 1 | 3/3 | DFS-discovery order is implementation-dependent across the three reference impls; the conformance runner sorts the AP vector before comparing. Fixtures: igraph C `igraph_biconnected_components.c` test graph (10v with isolated/disconnected components, expected APs [2,5]); python-igraph `Graph.Tree(5,2)` (APs [0,1]); R `path_graph_impl(n=3)` from `test-aaa-auto.R` (AP [1]). |
| bridges | 2 | 1 | 1 | 4/4 | Edge-id outputs sorted before comparison (DFS discovery order varies). Conformance runs against the fixture's edge-list ordering, so edge ids are stable across the test runner; oracle tests use a `(min,max)` endpoint-pair canonicalisation because `GraphPayload::from_graph` rebuilds python's edge list in a different order. Fixtures: igraph C `igraph_bridges.c` 7v two-triangles + multigraph cases; python-igraph 4-path; R `make_graph("krackhardt_kite")` from `test-components.R`. |
| is_biconnected | 2 | 1 | 1 | 4/4 | Boolean output, no canonicalisation needed. Fixtures: igraph C `igraph_is_biconnected.c` (4-cycle + 3-cycle sharing vertex 2 → false; ring(10) → true); python-igraph K4 → true; R `path_graph(n=3)` from `test-aaa-auto.R` → false. |
| girth | 2 | 1 | 1 | 4/4 | `Option<u32>` encoded as JSON `null` (no cycle) / integer. First fixture suite with a `null`-valued expected — the `tests/common::run_ok` helper was generalised to return `Value::Null` when the oracle's `result` field is null (previously panicked). Fixtures: igraph C `examples/simple/igraph_girth.c` ring(100)+chord and null-graph; python-igraph 5-cycle; R `test-structural-properties.R:'girth() works'` make_ring(100). |
| eccentricity | 1 | 1 | 1 | 3/3 | `Vec<u32>` (per-vertex). Fixtures: 5-path, 4-vertex star, 3-path. All BFS-from-each-vertex consuming SP-006. |
| radius | 1 | 1 | 1 | 3/3 | `Option<u32>` (None for null graph). Fixtures: 5-path → 2, 4-vertex star → 1, 3-path → 1. |
| diameter | 1 | 1 | 1 | 3/3 | `Option<u32>`. Fixtures: directed ring(10) → 9 (igraph_diameter.c); 4-cycle → 2; R disjoint trees (make_tree(7,2) ∪ make_tree(4,3), unconnected=TRUE) → 4. |
| count_triangles | 1 | 1 | 1 | 3/3 | `u64`. python-igraph 0.11 has no direct `count_triangles`; oracle uses `len(g.list_triangles())`. Fixtures: igraph C K4 → 4, python-igraph 5-cycle → 0, R path(3) → 0. |
| transitivity_undirected | 2 | 1 | 1 | 4/4 | `Option<f64>` encoded as JSON null (no triples) or float. The 3*triangles/triples ratio is exactly representable as f64 for integer operands at any practical scale, so direct `==` comparison holds. Fixtures: igraph C `global_transitivity.c` Famous("Zachary") → 0.255681818..., K4 → 1.0; python-igraph K4-minus-edge → 0.75; R path(3) → 0.0. |
| transitivity_local_undirected | 1 | 1 | 1 | 3/3 | Per-vertex `Vec<Option<f64>>`; `None` when degree<2 (matches NaN in upstream's `IGRAPH_TRANSITIVITY_NAN` mode). Karate oracle uses 1e-12 tolerance to absorb f64 round-trip through JSON. Fixtures with rationals that fit exactly use `==`. |
| density | 1 | 1 | 1 | 3/3 | `Option<f64>`. **Triggered the conformance runner's float-tolerance overhaul**: the same f64 `0x1.1cbfa862911ccp-3` round-trips as 17-digit "0.13903743315508021" via Rust's serde_json but as the 16-digit "0.1390374331550802" via Python's `json.dumps`, which serde_json then re-parses as the ULP-different f64. Solution: `tests/conformance::json_approx_eq` does relative-1e-12 comparison on numbers, exact on everything else. |
| mean_distance | 1 | 1 | 1 | 3/3 | `Option<f64>` — None for n<2 or no connected pairs. Same float-tolerance comparison as density. |
| eulerian_path | 1 | 0 | 1 | 2/2 | py-skipped (python-igraph 0.11.x has no Eulerian API). Multiple valid walks exist for a given graph; conformance fixtures compare `len(walk)` only — proptest `eulerian_path_visits_every_edge_once_when_it_exists` enforces the actual structural correctness. |
| count_reachable | 1 | 1 | 1 | 3/3 | `Vec<u32>`. python-igraph 0.11 lacks `count_reachable` directly; oracle uses `len(g.subcomponent(v, mode='out'))` per vertex. |

## How to add a row

`/awu-conformance ALGO-XXX-NNN` (or follow
[`.claude/skills/awu-conformance/SKILL.md`](../../.claude/skills/awu-conformance/SKILL.md))
extracts fixtures into `tests/conformance/{c,py,r}/<algo>/`. After CI is
green, append a row here with the new counts.

## Phase totals

| Phase | Algorithms in conformance | Total fixtures | C / py / R |
|-------|---------------------------|----------------|------------|
| 0     | 1 (bfs)                   | 4              | 2 / 1 / 1  |
| 1     | 19 (+ count_reachable) | 64 | 27 / 17 / 20 |
