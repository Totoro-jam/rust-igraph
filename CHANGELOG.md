# Changelog

All notable changes to **rust-igraph** are recorded here.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning 2.0](https://semver.org/spec/v2.0.0.html).

> Pre-1.0 contract: every minor bump (0.x.y → 0.(x+1).0) may break the
> public API. Patch bumps are bug-fixes / new additive items only.

## [Unreleased]

### Added
- *(core)* **ALGO-CORE-001a**: real `Graph` (replaces the Phase-0 throwaway
  `Graph<u32>`). Indexed-edgelist storage matching upstream igraph's
  `igraph_t` (`from`/`to`/`oi`/`ii`/`os`/`is`). New surface: `Graph::new(n,
  directed)`, `is_directed()`. Phase-0 method signatures
  (`with_vertices` / `add_edge` / `add_edges` / `vcount` / `ecount` /
  `neighbors` / `degree`) preserved so existing callers compile unchanged.
- *(core)* `Graph` derives `Clone` (deep) and `Default`.
- *(core)* Phase-0 split for `igraph_t`: ALGO-CORE-001a..e tracked in
  `.codefuse/tracking/ALGORITHMS.md`. Subsequent AWUs add `incident`,
  edge-id helpers, deletion, edge-list queries, and the property cache.

### Changed
- *(core)* `Graph::neighbors(v)` now returns `Vec<VertexId>` instead of
  `&[VertexId]`. The indexed-edgelist backend cannot offer a contiguous
  slice cheaply (out-neighbours live in `to[oi[os[v]..os[v+1]]]`).
  Iteration call sites no longer use the `&w` pattern; `for w in
  graph.neighbors(v)?` is the new shape. Affects `bfs` and the test
  helper in `tests/common/mod.rs`.
- *(core)* `Graph::degree(v)` for undirected graphs now counts
  self-loops as 2 (matching upstream `IGRAPH_LOOPS_TWICE` default at
  `type_indexededgelist.c:1162`). Phase-0 counted them as 1.
- *(test-helper)* `tests/common/mod.rs::GraphPayload::from_graph`
  rebuilds the edge list from `neighbors()` correctly when self-loops are
  present (was correct before only because `Graph::neighbors` returned a
  single entry per self-loop; the new backend reports 2).
- *(documentation)* `CHANGELOG.md` (Keep a Changelog 1.1.0).
- *(documentation)* `SECURITY.md` — vulnerability reporting via GitHub
  Security Advisory or email. Slim, alpha-appropriate scope.
- *(documentation)* `DEVELOPMENT.md` — maintainer + AI-agent setup notes
  (renamed from the original `CONTRIBUTING.md`, which was honestly never
  an external-contributor guide).
- *(documentation)* New minimal `CONTRIBUTING.md` — "alpha; not accepting
  external PRs yet; here is when that opens up".
- *(tooling)* `.editorconfig` — cross-IDE consistency for files cargo fmt
  does not touch (md / yml / toml / py / sh).
- *(ci)* `.github/dependabot.yml` — weekly cargo + GitHub Actions + pip
  updates; minor/patch grouped, breaking changes split.
- *(documentation)* README badges: crates.io / docs.rs / CI / license / MSRV.

### Changed
- *(claude-code)* `.claude/settings.json` is committed again (Anthropic's
  intended pattern). The `.sample` workaround attempted earlier is
  removed; the `.githooks/pre-commit` hook is the safety net against
  accidental personal-grant commits.
- *(claude-code)* `.githooks/pre-commit` output bug fixed (entries with
  spaces no longer split into multiple "lines"); reference now points to
  DEVELOPMENT.md.

### Explicitly deferred
- `CODE_OF_CONDUCT.md`, `CITATION.cff`, `.github/ISSUE_TEMPLATE/`,
  `.github/CODEOWNERS`, `SUPPORT.md`. These artifacts presume an external
  community / academic citations / multiple reviewers; none of those
  exist at 0.0.1-alpha.0. Add when the corresponding signal arrives.


## [0.0.1-alpha.0] — 2026-05-15

First publish to crates.io. Reserves the `rust-igraph` name and exercises
the release pipeline end-to-end.

### Added
- Walking-skeleton public surface:
  - `Graph` (undirected, unweighted, `u32` vertex ids, adjacency-list
    backed; Phase-0 minimal port of `igraph_t`)
  - `read_edgelist` (Phase-0 port of `igraph_read_graph_edgelist`)
  - `bfs` (single-root variant of `igraph_bfs`)
  - `IgraphError` + `IgraphResult` (initial 8 variants)
- Three-source conformance suite for BFS:
  - igraph C: 2 fixtures (path-10, kary_tree-20-2)
  - python-igraph: 1 fixture (Tree-10-2)
  - R-igraph: 1 fixture (ring-10)
- Live oracle test against python-igraph 0.11.x (BFS on Karate matches
  exactly).
- Property tests (proptest): no-duplicate-visit + reachability symmetry.
- Criterion bench: BFS on Karate ≈ 693 ns.
- mdBook scaffold; rustdoc auto-published to GitHub Pages on every push
  to `main`.
- `examples/bfs_karate.rs` smoke demo.
- AI-engineering scaffolding (committed): `CLAUDE.md`, 7 sub-agents under
  `.claude/agents/`, 9 skills under `.claude/skills/` (the `/awu-*`
  family + `/oracle-add` + `/phase-checkpoint` + `/resume-session`),
  3 hooks under `.claude/hooks/` (block-dangerous-git, post-edit-rust,
  post-tool-bash), prompt cookbook in `.codefuse/tracking/AI_PROMPTS.md`.
- AWU SOP infrastructure: 4 templates (`templates/{algo,test,oracle,bench}.{rs,py}.tpl`),
  ALGORITHMS / ARCHITECTURE / CONFORMANCE / RESUME tracking under
  `.codefuse/tracking/`.
- CI matrix: fmt + clippy (stable + MSRV 1.85) + test (stable on
  Linux/macOS, beta on Linux) + conformance + oracle + proptest +
  cargo-deny + wasm32 check + rustdoc -D warnings + GitHub Pages deploy +
  release workflow (tag-triggered cargo publish).

### Status
- API: alpha. WILL break before 0.1.0.
- 850-API parity: ~0% (only `Graph`, `read_edgelist`, `bfs` shipped).
- See [`docs/plans/MASTER_PLAN.md`](docs/plans/MASTER_PLAN.md) and
  [`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md)
  for the roadmap.

[Unreleased]: https://github.com/Totoro-jam/rust-igraph/compare/v0.0.1-alpha.0...HEAD
[0.0.1-alpha.0]: https://github.com/Totoro-jam/rust-igraph/releases/tag/v0.0.1-alpha.0
