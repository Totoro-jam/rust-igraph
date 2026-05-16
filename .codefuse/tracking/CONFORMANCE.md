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

## How to add a row

`/awu-conformance ALGO-XXX-NNN` (or follow
[`.claude/skills/awu-conformance/SKILL.md`](../../.claude/skills/awu-conformance/SKILL.md))
extracts fixtures into `tests/conformance/{c,py,r}/<algo>/`. After CI is
green, append a row here with the new counts.

## Phase totals

| Phase | Algorithms in conformance | Total fixtures | C / py / R |
|-------|---------------------------|----------------|------------|
| 0     | 1 (bfs)                   | 4              | 2 / 1 / 1  |
| 1     | 6 (dfs, cc, scc, distances, is_eulerian, articulation) | 22 | 10 / 5 / 7 |
