# Changelog

All notable changes to **rust-igraph** are recorded here.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning 2.0](https://semver.org/spec/v2.0.0.html).

> Pre-1.0 contract: every minor bump (0.x.y → 0.(x+1).0) may break the
> public API. Patch bumps are bug-fixes / new additive items only.

## [Unreleased]

### Added
- *(documentation)* `CHANGELOG.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`,
  `CITATION.cff`, `.editorconfig`, GitHub issue templates, Dependabot
  config, `CODEOWNERS`. README gains crates.io / docs.rs / CI / license
  badges.

### Changed
- *(claude-code)* `.claude/settings.json` is committed again (Anthropic's
  intended pattern). The `.sample` workaround is removed; the pre-commit
  hook in `.githooks/` is the safety net for accidental personal grants.

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
