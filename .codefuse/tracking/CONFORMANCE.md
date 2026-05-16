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

## How to add a row

`/awu-conformance ALGO-XXX-NNN` (or follow
[`.claude/skills/awu-conformance/SKILL.md`](../../.claude/skills/awu-conformance/SKILL.md))
extracts fixtures into `tests/conformance/{c,py,r}/<algo>/`. After CI is
green, append a row here with the new counts.

## Phase totals

| Phase | Algorithms in conformance | Total fixtures | C / py / R |
|-------|---------------------------|----------------|------------|
| 0     | 1 (bfs)                   | 4              | 2 / 1 / 1  |
| 1     | 3 (dfs, cc, scc)          | 11             | 5 / 3 / 3  |
