# Contributing to rust-igraph

This is a single-developer + AI project, so the main audience for this file
is **future-you returning from a break** and **any AI agent assigned a task**.
Read [CLAUDE.md](CLAUDE.md) and [docs/plans/MASTER_PLAN.md](docs/plans/MASTER_PLAN.md)
first; this file fills in the practical setup.

## One-time setup (per clone)

```bash
# 1. Fetch the three official igraph implementations as references.
#    These are gitignored; see references/README.md for the exact commands.
cd references && cat README.md

# 2. Activate the in-repo git hooks (CC attribution + format-on-edit).
git config core.hooksPath .githooks

# 3. Set a personal git identity for this repo (do NOT touch global).
git config user.name  "Your Name"
git config user.email "your@personal.email"

# 4. Set up the Python venv used by the live oracle.
python3 -m venv .venv
.venv/bin/pip install -r scripts/requirements.txt

# 5. Smoke-test everything.
cargo test --workspace
.venv/bin/python -m scripts.test_extract.from_c  --algo bfs
cargo test -p igraph --features oracle-tests --test oracle
```

## Daily loop

```bash
cargo test --workspace                                # 0.x s
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Before opening a PR:

```bash
cargo test --workspace --all-features                 # incl. oracle + proptest
cargo doc --workspace --no-deps                       # catches broken doclinks
cargo bench --no-run                                  # benches at least compile
```

CI (GitHub Actions) re-runs all of the above plus `wasm32` check, `cargo-deny`,
and three-source conformance.

## How new algorithms land — the AWU workflow

Every algorithm is one Algorithm Work Unit (AWU) with id `ALGO-XXX-NNN`,
tracked in [.codefuse/tracking/ALGORITHMS.md](.codefuse/tracking/ALGORITHMS.md).
Each AWU goes through the 9-step SOP defined in the
[master plan](docs/plans/MASTER_PLAN.md#四算法移植-sop核心可复用流程).

There are two ways to drive the SOP — **pick one, do not mix**:

### Skill-driven (recommended; uses the in-repo Claude Code skills)

```
/awu-start       ALGO-XXX-NNN     # Steps 1-3 (Recon + Interface + Skeleton)
/awu-translate   ALGO-XXX-NNN     # Step 4
/awu-test        ALGO-XXX-NNN     # Steps 5-6a + 7
/awu-conformance ALGO-XXX-NNN     # Step 6b
/awu-bench       ALGO-XXX-NNN     # Step 8
/awu-finish     ALGO-XXX-NNN     # Step 9
```

Skills live under `.claude/skills/` and delegate to focused sub-agents in
`.claude/agents/`.

### Manual

Read [`docs/plans/MASTER_PLAN.md` §4](docs/plans/MASTER_PLAN.md) and copy the
templates under [`templates/`](templates/) into the right module:

- [`templates/algo.rs.tpl`](templates/algo.rs.tpl) → `src/algorithms/...`
- [`templates/test.rs.tpl`](templates/test.rs.tpl) → unit / oracle / proptest
- [`templates/oracle.py.tpl`](templates/oracle.py.tpl) → branch in `scripts/oracle.py`
- [`templates/bench.rs.tpl`](templates/bench.rs.tpl) → `benches/bench_<algo>.rs`

In either case, the AWU is `done` only when:

- [ ] cargo fmt / clippy clean
- [ ] cargo test --workspace --all-features green
- [ ] At least one fixture from each of igraph C / python-igraph / R-igraph
      under `tests/conformance/{c,py,r}/<algo>/`
- [ ] Criterion baseline captured under `.codefuse/tracking/perf/`
- [ ] Public API has rustdoc with at least one doctest
- [ ] [`.codefuse/tracking/ALGORITHMS.md`](.codefuse/tracking/ALGORITHMS.md)
      status flipped to `done` in the same PR

## Commit conventions

```
<type>(<scope>): <ALGO-XXX-NNN> short why

Body describing the *why*, never the *what*.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
```

Types: `feat`, `fix`, `test`, `docs`, `refactor`, `perf`, `chore`, `wip`.
Scopes: `core`, `algo-xxx`, `oracle`, `ci`, `build`, `templates`, `docs`.

The `Co-Authored-By` trailer is appended automatically by the
`prepare-commit-msg` hook in `.githooks/`. (Run the one-time setup above to
activate.)

## Hard constraints — never violate

Mirrored from [CLAUDE.md](CLAUDE.md):

1. No `unsafe` blocks outside an explicit ARCHITECTURE.md ADR.
2. No `unwrap()` / `expect()` outside tests (with reason strings).
3. No new dependencies unless ARCHITECTURE.md approves. **Forbidden**:
   `petgraph` (API mismatch), `graphalgs` (GPL-3 license incompatibility),
   `scirs2-sparse` (heavy dep tree).
4. Float comparisons use tolerance helpers, never `==`.
5. Integer arithmetic uses `checked_*` / `try_from` to avoid silent overflow.
6. Public API requires rustdoc with at least one doctest.
7. Comments are sparse: only when the *why* is non-obvious.
8. **All code, comments, identifiers in English.** Chat / commit body may
   match the user's language.
9. Never write secrets, credentials, or personal paths to a tracked file.

## Claude Code settings — committed vs personal

There are **two** Claude Code settings files; they look similar but have
different roles:

| File | In commit? | Purpose |
|------|-----------|---------|
| `.claude/settings.json` | **yes** | Team-shared: hooks, statusline, durable rules everyone gets |
| `.claude/settings.local.json` | **no** (gitignored) | Personal: session "Always allow" grants, per-machine overrides |

Claude Code auto-appends to **whichever file already exists** when you
click "Always allow". To keep `settings.json` clean and avoid noisy diffs:

1. Make sure `.claude/settings.local.json` exists in your clone (an empty
   `{}` works — it just needs to be there).
2. If you ever see a `permissions:` block sneak into `.claude/settings.json`,
   cut-and-paste it into `settings.local.json` and re-commit the clean
   `settings.json`.

The reason: `settings.local.json` is gitignored, so personal grants stay
local; the committed `settings.json` only carries the hooks block.

## Returning after a break

If it's been more than a few days, run the `resume-session` skill (or read
its [SKILL.md](.claude/skills/resume-session/SKILL.md) and follow it
manually). It catches stale `wip` AWUs and proposes the next concrete step
in under 5 minutes.

[`.codefuse/tracking/RESUME.md`](.codefuse/tracking/RESUME.md) is where
session-by-session notes accumulate — append to it on the way out, read it
on the way in.
