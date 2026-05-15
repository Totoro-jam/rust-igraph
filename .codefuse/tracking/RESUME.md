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
- BOOT-25: DEVELOPMENT.md (maintainer setup + AWU SOP) + minimal CONTRIBUTING.md
  (alpha-stage external policy) — DONE in this session.
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

---

## 2026-05-15 (later) — single-crate restructure + alpha publish + hygiene reset

**Branch / commit**: `main` @ `40c8206` (pushed to origin).

**Three big arcs landed in this continuation session**:

### 1. ADR-0009: collapse 3-crate workspace → single `rust-igraph` crate
The 3-crate workspace (igraph-core / igraph-algorithms / igraph) made
sense for compile-time isolation but conflicted with the project owner's
"one externally visible package" goal. Code moved into `src/{core,algorithms}/`,
all `igraph_core::Foo` / `igraph_algorithms::Foo` rewrote to
`rust_igraph::Foo`, all `cargo *_p igraph*` flags dropped from CI. ADR-0002
marked superseded; ADR-0009 captures the new shape and what did *not*
change (module boundaries, test layering, AWU SOP).

### 2. v0.0.1-alpha.0 published to crates.io
Reserved the `rust-igraph` name. `igraph` was taken by an unrelated 2021
experiment (162 LOC, 5 recent downloads).

Three release-pipeline gotchas, all fixed:
- `cargo publish --dry-run` locally succeeded but packaged 58 wrong files
  because `LICENSE` / `README.md` patterns in `include` were unanchored
  (matched `.venv/.../LICENSE` etc.). Fixed by leading `/`: `/LICENSE`,
  `/README.md`, `/src/**/*.rs`. Now packs 15 correct files.
- `release.yml` ran `cargo check --target wasm32-unknown-unknown` but
  did not install the target. Added `targets: wasm32-unknown-unknown` to
  the toolchain step.
- crates.io rejected the upload with "verified email required". User had
  to verify email at https://crates.io/settings/profile (one-time).

### 3. Engineering-hygiene reset (40c8206)
Walked back from the "every committed open-source artifact" pile to what
0.0.1-alpha actually needs:
- Added: CHANGELOG (Keep a Changelog), SECURITY (slim), .editorconfig,
  .github/dependabot.yml, README badges (crates.io / docs.rs / CI /
  license / MSRV).
- Renamed: CONTRIBUTING.md → DEVELOPMENT.md (the file was honestly
  maintainer setup notes, not external-contributor docs).
- New minimal CONTRIBUTING.md: "alpha; not accepting external PRs yet;
  here's when that opens up".
- Explicitly NOT added: CODE_OF_CONDUCT, CITATION.cff, ISSUE_TEMPLATE,
  CODEOWNERS, SUPPORT — premature for current scope; revisit at signals
  (first external PR, first paper citation, etc.).

**Settings.json saga (worth remembering)**:
Tried multiple patterns to stop Claude Code's IDE auto-rewriting
`.claude/settings.json` with personal grants:
1. Pre-commit hook to block diff (kept)
2. `.gitignore + .sample` (rejected — diverges from Anthropic's design)
3. **Final**: commit `settings.json` per Anthropic's intent +
   pre-commit hook as safety net + train muscle to pick "Always allow
   (local)" scope when granting. The hook fires correctly when settings
   drift; tested by today's intentional baseline-bump commit, which
   `--no-verify`'d through with explanation.

**GitHub Pages**:
The first attempt deployed via `actions/deploy-pages` succeeded as a
workflow but the actual site served a Jekyll-rendered README.md because
Source was set to "Deploy from a branch". User switched to "GitHub
Actions" mid-session; subsequent push (40c8206) should make rustdoc
actually serve at https://totoro-jam.github.io/rust-igraph/rust_igraph/.

**Next concrete step**:
- Verify Pages now serves rustdoc (after 40c8206 deploys).
- Verify docs.rs build for 0.0.1-alpha.0 finished (was in queue when
  publish landed).
- **Phase 1 entry**: `/awu-start ALGO-CORE-001`. Replaces the throwaway
  `Graph<u32>` (Phase 0 placeholder) with the full igraph_t-equivalent:
  CSR storage, directed/undirected, weighted, multigraph, attribute
  binding. C reference: `references/igraph/src/graph/type_indexededgelist.c`
  (~2013 lines, complexity `adapt`). Big AWU; warrants a focused
  session of its own.

**Don't retry / lessons added**:
- Bash `for f in $FILES; do ...` where `$FILES` is multi-line: pass
  through `find ... -print0 | while IFS= read -r -d ''` instead. The
  for-loop treats the whole var as one token in some shells.
- `cargo` `include` patterns — leading `/` anchors to package root, no
  leading `/` matches recursively. `include = ["LICENSE"]` will pull
  every file named LICENSE in any subdir.
- crates.io requires a verified email on the publishing account before
  the first publish. Set this BEFORE the first tag push so release.yml
  doesn't fail on the wire.
