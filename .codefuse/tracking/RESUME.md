# Session log

> Append a block on the way out, read the most recent block on the way in.
> The `resume-session` skill automates both ends of this loop.

The format is intentionally loose. Capture: what's mid-flight, what surprised
you, what *not* to retry, what to do next. **Past-you is talking to
present-you across an unknown gap.**

---

## 2026-05-15 — Phase 0 walking-skeleton + BFS SOP + AI infra + CI all landed

**Branch / commit**: `main` @ `a6b1cf8` (pushed to origin).

**What landed this session**:
- Walking skeleton (BOOT-01..08): cargo workspace, Graph<u32>, EdgeList, BFS,
  karate fixture, bfs_karate example.
- Live oracle (BOOT-09..11): `scripts/oracle.py` + `tests/oracle.rs`
  + Python venv, BFS karate ↔ python-igraph 0.11.9 numerically equal.
- proptest (BOOT-12) + criterion bench (BOOT-13).
- Three-source conformance (BOOT-29..32): igraph C / python-igraph / R-igraph
  fixtures all green for BFS (4 fixtures total). **Real bug caught**: igraph
  C's `igraph_ring(n, mutual=0, circular=0)` is a *path*, not a closed ring.
  See `scripts/test_extract/from_c.py` for the corrected manifest entry.
- AI infra (BOOT-23, 33-37): CLAUDE.md, ALGORITHMS.md tracker, AI_PROMPTS.md
  cookbook, 7 agents under `.claude/agents/`, 9 skills under `.claude/skills/`,
  3 hooks under `.claude/hooks/`. Skill structure borrowed from
  github.com/mattpocock/skills (MIT) — terse, opinionated, description-first.
- GitHub Actions CI (BOOT-14..18): 12 jobs all green on first push.
  `Deploy docs to GitHub Pages` failed because Pages source needed switching
  from "Deploy from a branch" to "GitHub Actions".

**What surprised me**:
- The 3-source conformance test caught a bug on the first run (the igraph C
  `circular=0` mistake). Justifies the cost of wiring all three sources.
- Cargo dev-dependencies cannot be `optional`. The `proptest-harness` feature
  has to be a marker-only feature; proptest itself is always in dev-deps.
- macOS sandbox blocks any write under `.git/` and `.claude/`. Need
  `dangerouslyDisableSandbox` for git ops and `.claude/` edits.

**What's still pending in Phase 0** (not blocking any AWU):
- BOOT-19..22: 4 templates under `templates/` — DONE in this session.
- BOOT-24: ARCHITECTURE.md (ADR index) — DONE in this session.
- BOOT-25: CONTRIBUTING.md — DONE in this session.
- BOOT-26: mdBook scaffold — TODO (CI already publishes rustdoc).
- BOOT-27: scripts/bench_compare.py — DONE in this session.
- BOOT-28: this RESUME.md — DONE.

**Next concrete step** (when picking back up):
- Pages: switch repo Settings → Pages → Source to "GitHub Actions" (one-time
  UI), then re-trigger the pages workflow.
- Phase 1 entry: `/awu-start ALGO-CORE-001` to begin the real Graph type
  (CSR, igraph_t-equivalent).

**Don't retry**:
- Translating algorithms inline in CLAUDE Code conversation. Always go through
  the AWU SOP — even small algorithms benefit from oracle + conformance
  catching subtle bugs (see the `circular=0` story above).
- Putting `${{ matrix.X }}` inside flow-style YAML mappings — the YAML scanner
  collides with the `${{` opening brace. Use block style.

**Identity sanity check**:
- Local repo `git config user.email` should be `moqiuchen66@gmail.com`
  (Totoro-jam personal). Global is intentionally different (`digital-engine.com`,
  work). Run `git config user.email` to verify before committing.
- All commits in this repo carry `Co-Authored-By: Claude Opus 4.7
  <noreply@anthropic.com>` via the `prepare-commit-msg` hook in `.githooks/`.
  If commits are missing it, run `git config core.hooksPath .githooks`.
