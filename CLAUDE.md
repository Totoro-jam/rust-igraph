# CLAUDE.md — project-level instructions

This file is loaded by Claude Code as the project system prompt. Keep it
focused: rules, conventions, and pointers — not narrative.

## Project at a glance

- **Goal**: pure-Rust port of [igraph](https://igraph.org) C v1.0.x (~850 public
  APIs). Single-developer + AI, part-time pace.
- **License**: GPL-2.0-or-later (matches upstream igraph; permits direct
  reference-translation of `references/igraph/`).
- **Plan**: [docs/plans/MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) — the
  single source of truth. AWU tracker:
  [.codefuse/tracking/ALGORITHMS.md](.codefuse/tracking/ALGORITHMS.md).

## Hard constraints — never violate

1. **No `unsafe` blocks** anywhere outside an explicit ARCHITECTURE.md ADR.
2. **No `unwrap()` / `expect()`** in non-test code. Tests may use them with
   reason strings.
3. **No new dependencies** unless ARCHITECTURE.md explicitly approves.
   Forbidden: `petgraph` (API mismatch), `graphalgs` (GPL-3 license
   incompatibility), `scirs2-sparse` (heavy dep tree).
4. **Floating-point comparisons** use tolerance helpers
   (`rust_igraph::testutil` once it lands), never `==`.
5. **Integer arithmetic** uses `checked_*` / `try_from` to avoid silent
   overflow.
6. **Public API** requires rustdoc with at least one doctest.
7. **Comments are sparse**: only when the *why* is non-obvious. Don't restate
   what well-named code already says.
8. **All code, comments, identifiers in English**. User-facing chat may be in
   the language the user uses.
9. **Don't write secrets/credentials/tokens** to any tracked file. Personal
   paths (e.g. `/Users/<name>/...`) belong in `.gitignore`d files only.

## AWU workflow (the only way new algorithms land)

Every algorithm is an Algorithm Work Unit (AWU) tracked in
`.codefuse/tracking/ALGORITHMS.md` with a unique `ALGO-XXX-NNN` id. Per AWU,
go through the 9-step SOP defined in MASTER_PLAN.md §4. Use the helper
skills:

```
/awu-start      ALGO-XXX-NNN     # bootstrap (Recon + Interface + Skeleton)
/awu-translate  ALGO-XXX-NNN     # Implementation
/awu-test       ALGO-XXX-NNN     # Unit + proptest
/awu-conformance ALGO-XXX-NNN    # Three-source extraction (igraph C / py / R)
/awu-bench      ALGO-XXX-NNN     # criterion baseline
/awu-finish     ALGO-XXX-NNN     # rustdoc + status update + PR description
```

Skills are defined under `.claude/skills/`; each delegates to the right
sub-agent (`.claude/agents/`).

## Test asset reuse — three official implementations

`references/` (gitignored) holds clones/symlinks of:
- `igraph/` — C core (~425 unit tests + .out)
- `python-igraph/` — Python bindings (~526 test methods)
- `rigraph/` — R bindings (108+ auto-bound tests)

Conformance extractors live in `scripts/test_extract/`. Every AWU's Step 6
adds at least one fixture from each source under
`tests/conformance/{c,py,r}/<algo>/`.

## Commands you may need

```bash
# fast loop
cargo build --workspace
cargo test  --workspace                      # 14 tests today, no oracle
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# full loop (oracle + proptest + conformance)
cargo test  --workspace --all-features

# extract conformance fixtures (per-algo)
.venv/bin/python -m scripts.test_extract.from_c  --algo bfs
.venv/bin/python -m scripts.test_extract.from_py --algo bfs
.venv/bin/python -m scripts.test_extract.from_r  --algo bfs

# WASM check (no external deps — pure Rust)
cargo check --target wasm32-unknown-unknown
```

## Git conventions

- Commit message: `<type>(<scope>): <ALGO-XXX> short why`
  - types: `feat / fix / test / docs / refactor / perf / chore / wip`
  - scope: `core / algo-xxx / oracle / ci / build / templates / docs`
- One AWU = one PR (or a tight series). PR title includes `ALGO-XXX-NNN`.
- Author: this repo uses a personal identity via local `git config`
  (Totoro-jam <moqiuchen66@gmail.com>); the global identity is intentionally
  different.

## Where files live

```
src/core/                   # data structures (Graph, Vector, Matrix, ...)
src/algorithms/             # algorithm implementations
tests/                      # integration tests (oracle / conformance / property)
docs/plans/MASTER_PLAN.md   # the plan (this is THE source of truth)
.codefuse/tracking/         # ALGORITHMS.md, ARCHITECTURE.md, CONFORMANCE.md, ...
.claude/                    # AI infrastructure (agents, skills, hooks)
references/                 # gitignored: igraph C / python-igraph / rigraph
scripts/                    # oracle.py + test_extract/{from_c,from_py,from_r}
templates/                  # AWU templates (Step 3 source)
fixtures/                   # standard graph datasets (karate, dolphins, ...)
tests/conformance/{c,py,r}/ # extracted from upstream test suites
```

## When in doubt

1. Check [docs/plans/MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) — it almost
   always has the answer.
2. Look for a similar AWU that landed already, and copy its style.
3. Ask the user before introducing architectural change. Default to the
   conservative reading of these constraints.
