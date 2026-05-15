# Quick start

```bash
git clone https://github.com/Totoro-jam/rust-igraph
cd rust-igraph

# Default loop — passes today.
cargo build --workspace
cargo test  --workspace
cargo run   --example bfs_karate     # loads fixtures/karate.edges, runs BFS
```

You should see something like:

```
loaded fixtures/karate.edges (34 vertices, 78 edges)
BFS from 0: [0, 1, 2, 3, 4, 5, 6, 7, ..., 26]
  visited 34 of 34 vertices
```

That output is verified against python-igraph 0.11.x in CI under the
`oracle` job.

## With the live oracle (optional)

```bash
python3 -m venv .venv
.venv/bin/pip install -r scripts/requirements.txt
cargo test -p igraph --features oracle-tests --test oracle
```

## Where to read next

- [`docs/plans/MASTER_PLAN.md`](../../docs/plans/MASTER_PLAN.md) — what's
  planned, in what order, with what trade-offs.
- [`.codefuse/tracking/ALGORITHMS.md`](../../.codefuse/tracking/ALGORITHMS.md)
  — per-algorithm status; this is where new contributions get listed.
- [`DEVELOPMENT.md`](../../DEVELOPMENT.md) — the AWU workflow and the
  one-time setup steps.
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — alpha-stage external
  contribution policy (currently: not accepting external PRs).
