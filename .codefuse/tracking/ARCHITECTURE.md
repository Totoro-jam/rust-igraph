# ARCHITECTURE.md — index of architectural decisions

Architecture Decision Records (ADRs) for rust-igraph. Each ADR is one
binding decision: the *what*, the *why*, the alternatives considered, and
the consequences. New deps, new layers, new license stances all go here
before the code that depends on them lands.

> Naming convention: `ADR-NNNN-kebab-title`. Numbers are monotonic; don't
> reuse a number even if an ADR is superseded — append a new one that
> supersedes the old.

## Status legend

- `accepted` — in force; the code reflects this
- `superseded by ADR-NNNN` — old; cross-link the replacement
- `proposed` — drafted but not yet decided
- `withdrawn` — drafted, then dropped

---

## ADR index

| ID | Title | Status | Source |
|----|-------|--------|--------|
| ADR-0001 | License: GPL-2.0-or-later, matching igraph | accepted | [#below](#adr-0001) |
| ADR-0002 | 3-crate workspace: core / algorithms / facade | accepted | [#below](#adr-0002) |
| ADR-0003 | Linear algebra backend: faer + self-rolled IRLM/IRAM | accepted | [#below](#adr-0003) |
| ADR-0004 | Isomorphism: VF2 first, BLISS C++ → Rust translation, optional nauty FFI | accepted | [#below](#adr-0004) |
| ADR-0005 | Test conformance: integrate all three official test suites | accepted | [#below](#adr-0005) |
| ADR-0006 | AI workflow: AWU SOP + skills + agents + hooks committed in repo | accepted | [#below](#adr-0006) |
| ADR-0007 | Phase 0 Graph<u32> is throwaway; Phase 1 brings the real `igraph_t`-equivalent | accepted | [#below](#adr-0007) |
| ADR-0008 | Banned dependencies: petgraph, graphalgs, scirs2-sparse | accepted | [#below](#adr-0008) |

---

## ADR-0001 — License: GPL-2.0-or-later

**Status**: accepted (2026-05-14, fixed in commit `2ce55aa` and the LICENSE
file from the GitHub-provisioned repo).

**Context**: Reproducing igraph's behavior bit-for-bit is the only way to
serve users coming from the C / Python / R bindings. Doing so without the
ability to reference-translate the upstream source forces a clean-room
re-derivation that costs roughly 3× more.

**Decision**: License this project as **GPL-2.0-or-later**, matching upstream
igraph. Cargo.toml's `license` field declares the "or later" intent; the
LICENSE file itself is the unmodified GPL v2 text from the GitHub template.

**Consequences**:
- We can directly translate from `references/igraph/src/...` line by line.
- Downstream commercial users who need MIT/Apache must use the
  `graph-algo-wasm` follow-on project (not yet started; see
  `references/`-adjacent siblings) which sits on a process boundary so its
  surface API can be relicensed.
- Dependency stance: `cargo-deny` allows GPL-2-or-later compatible licenses
  only; **GPL-3-only** is banned (see ADR-0008).

**Alternatives**:
- MIT/Apache clean-room — rejected, ~3× cost, much higher correctness risk.
- LGPL — rejected, igraph itself isn't LGPL.

---

## ADR-0002 — Three-crate workspace

**Status**: accepted (2026-05-14, fixed in `2ce55aa`).

**Context**: We need data structures that the algorithm crate can depend on
without cycles, and a facade crate that gives end-users a `use igraph::*`
ergonomic surface.

**Decision**: Three crates in one Cargo workspace:

- `igraph-core` — `Graph`, `Vector`, `Matrix`, error types. No algorithms.
- `igraph-algorithms` — depends on `igraph-core`. All ~600 algorithm AWUs.
- `igraph` — depends on both. Re-exports + high-level semantic types
  (`VertexClustering`, `Layout`, `Cut`). Hosts integration tests (oracle,
  conformance, property).

**Consequences**:
- Algorithm authors only need `igraph-core` in scope.
- Tests live with the facade crate so they can use the public API as users
  see it.
- `Cargo.lock` is gitignored (library crate convention); apps using us
  manage their own lockfile.

**Alternatives**:
- Single crate — rejected, mixes concerns and slows incremental builds.
- Per-algorithm-group crates — rejected, premature splitting; can refactor
  later if compile times warrant.

---

## ADR-0003 — Linear algebra backend

**Status**: accepted (2026-05-15; not yet exercised since no eigenvalue
algorithm has landed in code, but locked in MASTER_PLAN.md §2.1).

**Context**: igraph C uses ARPACK (Fortran) for sparse eigenvalue problems.
Pure-Rust ARPACK does not exist. We need WASM-friendly, MIT-compatible
linear algebra plus our own implementations of IRLM (symmetric) and IRAM
(non-symmetric) Krylov solvers.

**Decision**: Three-tier dispatch:
- **Tier A (n ≤ 50)**: faer's dense EVD.
- **Tier B (large sparse)**: hand-rolled IRLM/IRAM, line-by-line translation
  of `references/igraph/src/linalg/arpack.c` (1634 lines). Internal QR /
  three-diagonal EVD calls back into faer.
- **Tier C**: power iteration. PageRank uses this by default (matches
  igraph's preference for PRPACK / power iteration over ARPACK).

`faer 0.24` is the default backend (feature `faer-backend`); `nalgebra` is
an alternative (`nalgebra-backend`). Both expose the same `EigenSolver`
trait so callers don't see the choice.

**Consequences**:
- Numerical-correctness reviews (the `numerical-reviewer` agent) are
  mandatory for any AWU that lives in `linalg/` or transitively depends on
  IRLM (PageRank, eigenvector centrality, leading-eigenvector community,
  spectral embedding, MDS layout).
- WASM works out of the box with `faer-backend`.
- nauty C FFI remains an optional opt-in (`nauty-backend`) for users on
  100K+ vertex graphs who can sacrifice WASM.

**Alternatives**:
- C-FFI to ARPACK — rejected, breaks WASM and adds Fortran toolchain to CI.
- nalgebra-only — rejected, ~2-10× slower than faer on dense EVD.

---

## ADR-0004 — Isomorphism strategy

**Status**: accepted (2026-05-15, locked in MASTER_PLAN.md §2.1 and §5.2).

**Context**: igraph supports VF2, BLISS (canonical permutation), and LAD
(subgraph). BLISS is ~9500 lines of C++ embedded in igraph. nauty/Traces is
faster on huge graphs but Apache-2.0 (compatible) and not WASM-friendly.

**Decision**: Three-stage path:
1. **Phase 1 of isomorphism work**: VF2 (translated from igraph's `vf2.c`)
   plus the `isoclasses.c` lookup table covers the `igraph_isomorphic()`
   common path.
2. **Phase 2**: translate the embedded BLISS C++ to Rust, including the 6
   split heuristics and the `bliss.cc` bridge.
3. **Phase 3 (optional)**: `nauty-backend` feature exposes a C-FFI route
   for users on >100K-vertex graphs.

**Consequences**:
- `isomorphic()` works end-to-end after Phase 1 of isomorphism, even
  before BLISS lands (VF2 is the fallback).
- Default `cargo build` is WASM-friendly because `nauty-backend` is opt-in.
- `automorphism_group_size` returns `BigInt` via `num-bigint` (replacement
  for GMP `mpz_t` used in BLISS C++).

---

## ADR-0005 — Three-source test conformance

**Status**: accepted (2026-05-15, fixed in commit `5779a31`).

**Context**: Each official igraph implementation has a test suite worth
mining: ~425 C tests + .out, ~526 python-igraph methods, ~108 R-igraph
testthat blocks. Picking just one misses the boundary cases the others
catch.

**Decision**: All three are first-class. Per AWU, Step 6b (`/awu-conformance`
skill) extracts at least one fixture from each source under
`tests/conformance/{c,py,r}/<algo>/`. CI fails if any fixture diverges. The
`conformance-extractor` agent is responsible per AWU.

**Consequences**:
- Caught a real bug on day 0: igraph C's `igraph_ring(n, mutual=0,
  circular=0)` is a *path*, not a closed ring. Whichever source you trust,
  the others would diverge if your understanding of the upstream API is
  wrong.
- The `from_r.py` extractor uses a hand-curated manifest in Phase 0 because
  R is not installed locally; the `run_r.R` placeholder is the Phase-1
  upgrade path for full automation.

---

## ADR-0006 — AI workflow as committed repo asset

**Status**: accepted (2026-05-15, fixed in `8d688d8`).

**Context**: Single-developer + AI mode at part-time pace means months can
pass between sessions. Every session needs to recover context fast. The way
the AI works *with* the repo is itself a piece of architecture worth
versioning.

**Decision**: All AI infrastructure lives under `.claude/`, **committed**:
- `.claude/agents/` — 7 focused sub-agents (recon, translator, tester,
  conformance-extractor, numerical-reviewer, perf-bencher, doc-writer).
  Frontmatter pins model preference.
- `.claude/skills/` — 9 skills (the `/awu-*` family + oracle-add +
  phase-checkpoint + resume-session). Pattern borrowed from
  github.com/mattpocock/skills (MIT).
- `.claude/hooks/` — 3 hooks: block-dangerous-git, post-edit-rust,
  post-tool-bash. Hook script for git guardrails adapted from
  mattpocock/skills.

`CLAUDE.md` at repo root is the project-level system prompt. `AI_PROMPTS.md`
in tracking/ is the cookbook of prompts that worked.

**Consequences**:
- `git clone && /resume-session` is the supported path back into the work.
- New contributors (hypothetical) inherit the whole AI workflow without
  per-machine setup beyond `git config core.hooksPath .githooks`.

---

## ADR-0007 — Throwaway Phase-0 Graph

**Status**: accepted (2026-05-15, in force until ALGO-CORE-001 lands).

**Context**: The walking skeleton needed *some* Graph to drive BFS through
the full SOP. The real `igraph_t`-equivalent (CSR storage, directed +
weighted + multigraph support, attribute system) is a ~2000-line AWU.

**Decision**: Ship a deliberately minimal `Graph<u32>` (undirected,
unweighted, `Vec<Vec<u32>>` adjacency, 4 unit tests) for Phase 0 only.
ALGO-CORE-001 in Phase 1 replaces it wholesale; tests pinned to the public
surface (`vcount`, `ecount`, `add_edge`, `neighbors`, `degree`) port without
churn.

**Consequences**:
- The placeholder has no `is_directed()` or weights — algorithms requiring
  them are blocked until Phase 1 lands.
- Avoids over-designing the data structure before any real algorithm
  pressure-tests it.

---

## ADR-0008 — Banned dependencies

**Status**: accepted (2026-05-15; enforced by `deny.toml`).

**Context**: Some natural-looking deps either conflict with our license
stance or with our intent to mirror igraph's API.

**Decision**: The following are banned as direct dependencies:
- `petgraph` — its `Graph` API does not express what `igraph_t` expresses
  (no edge attributes, different selector model). Trying to wrap it would
  cost more than building our own.
- `graphalgs` — GPL-3-only; incompatible with our GPL-2-or-later stance
  (we want options open per ADR-0001).
- `scirs2-sparse` — heavy dependency tree, sparse-matrix API not aligned
  with what `igraph_sparsemat_t` exposes.

**Consequences**:
- `cargo-deny check` fails CI if any of these sneak in transitively. The
  `bans.deny` block in `deny.toml` enforces this.
- Future ADRs can lift these on a per-dep basis with explicit reasoning.
