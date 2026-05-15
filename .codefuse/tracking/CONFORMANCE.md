# Three-source conformance coverage

Auto-checked by `tests/conformance.rs`. Each row is one
algorithm; columns are fixture counts per source plus the Rust pass rate.

> Phase-0 invariant: every `done` AWU must contribute ≥ 1 fixture from
> each of the three sources. CI fails otherwise.

| Algorithm | C | py | R | Rust | Notes |
|-----------|---|----|---|------|-------|
| bfs       | 2 | 1  | 1 | 4/4  | Caught the `igraph_ring(circular=0)` = path bug on first run; see git log of `scripts/test_extract/from_c.py`. |

## How to add a row

`/awu-conformance ALGO-XXX-NNN` (or follow
[`.claude/skills/awu-conformance/SKILL.md`](../../.claude/skills/awu-conformance/SKILL.md))
extracts fixtures into `tests/conformance/{c,py,r}/<algo>/`. After CI is
green, append a row here with the new counts.

## Phase totals

| Phase | Algorithms in conformance | Total fixtures | C / py / R |
|-------|---------------------------|----------------|------------|
| 0     | 1 (bfs)                   | 4              | 2 / 1 / 1  |
