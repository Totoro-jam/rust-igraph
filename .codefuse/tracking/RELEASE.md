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
| `v0.0.1-alpha.1` | 2026-05-21 | Phase 0 ✅ (37/37) · Phase 1 main wave (94 done, 4 todo, 5 native) | First substantive alpha — 94 AWUs landed | Includes BFS/DFS/CC/SCC, shortest paths (BFS, Dijkstra, Bellman-Ford, Johnson, Floyd-Warshall, A\*, widest-path), centrality (closeness, harmonic, betweenness, edge betweenness, PageRank, eigenvector — all weighted variants), connectivity (articulation, bridges, biconnected, decompose), Eulerian existence + path, percolation (edgelist / bond / site), graph properties (girth, diameter, eccentricity, transitivity, density, mean distance, reciprocity, knn, assortativity, is_dag, topological_sorting, is_tree, is_forest, is_complete, neighborhood, convergence_degree, count_loops, count_multiple, count_adjacent_triangles, global/local efficiency), operators (simplify, complementer, disjoint_union, union, intersection, difference, is_same_graph), modularity. CORE-001f cache subsystem and ARPACK-backed solvers (PR-011c, PR-012b) deferred to alpha.2; DS-* families marked `native`. **Follow-up**: `gh release create v0.0.1-alpha.1 --title "..." --notes-file CHANGELOG-section` once `gh auth login` is run — tag is already pushed, so GitHub auto-shows it under "Tags"; the formal Release object can be drafted from the tag at any time. |
| `v0.0.1-alpha.2` | 2026-05-22 | Phase 0 ✅ (37/37) · Phase 1 ✅ (103 done, 0 todo, 5 native) | Phase 1 complete — directed EB community + property cache + efficiency + HITS-weighted | New AWUs since alpha.1: ALGO-CORE-001f (bit-packed property cache with selective invalidation: `is_dag`, `is_forest`, `has_loop`, `has_multi`, `has_mutual`), ALGO-PR-017 (HITS hub/authority scores via power-iter on A·Aᵀ), ALGO-PR-017b (weighted HITS), ALGO-PR-029 (global efficiency, Latora-Marchiori), ALGO-PR-030 (local efficiency + average), ALGO-CO-002 (Louvain multilevel), ALGO-CO-003 (Leiden — Modularity/CPM/ER objectives), ALGO-CO-004 (label propagation — Fast/Dominance/Retention), ALGO-CO-005 (Fluid Communities Parés 2017), ALGO-CO-006 (edge_betweenness_community — Girvan-Newman 2002 unweighted), ALGO-CO-006b (weighted EB community via Brandes-Dijkstra), ALGO-CO-006c (**directed** EB community + directed weighted EB community + new `modularity_weighted_directed` entry), ALGO-CO-007 (fast-greedy modularity Clauset-Newman-Moore 2004), ALGO-PR-012b (directed + weighted eigenvector centrality). Cumulative: 103 Phase 1 AWUs done, 0 todo, 5 native. Phase 2 (PR-011c ARPACK PageRank) deferred / may be redefined under self-roll guidance. **Follow-up**: `gh release create v0.0.1-alpha.2 --title "v0.0.1-alpha.2 — Phase 1 complete" --notes-file CHANGELOG-section` once `gh auth login` is set up — tag will be pushed and visible under "Tags" immediately, the formal Release object can be drafted from the tag at any time. |
