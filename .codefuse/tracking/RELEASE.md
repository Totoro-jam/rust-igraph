# Release log

A short ledger of every git tag we cut. **Update when tagging.**

## Cadence

We tag at natural Phase milestones (see
[MASTER_PLAN.md](../../docs/plans/MASTER_PLAN.md) §3 / ALGORITHMS.md
counters), not on a fixed calendar. Pre-1.0, every minor bump may break
the public API; patch bumps are additive / bug-fix only.

We **do not publish to crates.io** until the API is approaching
stability (target: `0.1.0`). Until then, releases live as git tags +
GitHub Releases only.

## Tag-cut checklist

1. Verify `cargo build --workspace`, `cargo test --workspace`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo fmt --all --check` are all green on `main`.
2. Move `[Unreleased]` in `CHANGELOG.md` to `[x.y.z] — YYYY-MM-DD`,
   open a fresh empty `[Unreleased]` above it.
3. Bump `Cargo.toml` `version`. Update its `description` if the Phase
   marker is stale.
4. Commit: `chore(release): vX.Y.Z`.
5. Tag: `git tag -a vX.Y.Z -m "<one-line summary>"`.
6. Push: `git push && git push --tags`.
7. `gh release create vX.Y.Z --title "vX.Y.Z — <one-line>" --notes "$(awk '/^## \[X.Y.Z\]/,/^## \[/' CHANGELOG.md | sed '$d')"`.
8. Append a row to the table below.

## Releases

| Tag | Date | Phase status at cut | Headline | Notes |
|-----|------|---------------------|----------|-------|
| `v0.0.1-alpha.0` | 2026-05-15 | Phase 0 BOOT-01..08 | Initial placeholder — empty walking skeleton | Bootstrapped the repo; no algorithms yet. |
| `v0.0.1-alpha.1` | 2026-05-21 | Phase 0 ✅ (37/37) · Phase 1 main wave (94 done, 4 todo, 5 native) | First substantive alpha — 94 AWUs landed | Includes BFS/DFS/CC/SCC, shortest paths (BFS, Dijkstra, Bellman-Ford, Johnson, Floyd-Warshall, A\*, widest-path), centrality (closeness, harmonic, betweenness, edge betweenness, PageRank, eigenvector — all weighted variants), connectivity (articulation, bridges, biconnected, decompose), Eulerian existence + path, percolation (edgelist / bond / site), graph properties (girth, diameter, eccentricity, transitivity, density, mean distance, reciprocity, knn, assortativity, is_dag, topological_sorting, is_tree, is_forest, is_complete, neighborhood, convergence_degree, count_loops, count_multiple, count_adjacent_triangles, global/local efficiency), operators (simplify, complementer, disjoint_union, union, intersection, difference, is_same_graph), modularity. CORE-001f cache subsystem and ARPACK-backed solvers (PR-011c, PR-012b) deferred to alpha.2; DS-* families marked `native`. |
