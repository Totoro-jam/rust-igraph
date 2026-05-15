---
name: oracle-add
description: Extend scripts/oracle.py with a new algorithm branch so Rust tests can compare against python-igraph. Use when user says "add oracle for ...", "wire <algo> into oracle.py", or when `/awu-test` needs a new oracle case.
---

# /oracle-add <algo-slug>

Adds one algorithm to `scripts/oracle.py` and verifies the wire works.

## Workflow

### 1. Locate the python-igraph equivalent

Search `references/python-igraph/src/igraph/` (or
`references/python-igraph/tests/`) for the call you want to mirror. Note:
- The exact `Graph` method name (e.g. `g.bfs(root)`)
- Its return shape (often a tuple — extract just the part the Rust port
  produces)
- Default values for optional parameters

### 2. Add the branch

Open `scripts/oracle.py` and append a branch to `run()`:

```python
if algo == "<slug>":
    # Counterpart of igraph_<algo>(). Returns <shape>.
    <param1> = <type>(params["<param1>"])
    result = g.<method>(<args>)
    return <list/dict/scalar conversion>
```

Convention:
- Always return JSON-serializable values (`list`, `dict`, `int`, `float`, `bool`).
- For tuples, return the slice the Rust function exposes (often just one
  field — see how BFS does it).
- Comment briefly with `Counterpart of igraph_<algo>().` so future readers
  can find the C reference.

### 3. Smoke

```
echo '{"graph":{"n":5,"edges":[[0,1],[1,2],[2,3],[3,4]],"directed":false},"algo":"<slug>","params":{...}}' \
  | .venv/bin/python scripts/oracle.py
```

Must print `{"ok": true, "result": ..., "version": "0.11.x"}`.

### 4. Hand off

Tell the user oracle is ready; the next call to `cargo test --features
oracle-tests` will exercise it.

## Anti-patterns

- **Do NOT add a branch you cannot smoke-test.** If python-igraph throws
  on your example input, the Rust side will also fail; figure out why now,
  not later.
- **Do NOT silently change return shapes** of existing branches. If two
  AWUs need different shapes from the same algorithm, use distinct slugs
  (e.g. `bfs_order` vs `bfs_layers`).
