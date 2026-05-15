---
name: awu-translate
description: Translate the igraph C body of one AWU into Rust. Step 4 of the SOP. Delegates to the awu-translator sub-agent with isolated context. Use when user says "translate ...", "implement ALGO-...", or after `/awu-start` finishes.
---

# /awu-translate ALGO-XXX-NNN

Step 4 of the AWU SOP: replace `unimplemented!()` with a faithful Rust port.

## Pre-checks

- The AWU must be in status `wip` (skeleton compiles, interface frozen)
- The Rust signature is in the target file
- `references/igraph/<C source>` exists

If any pre-check fails: tell the user, suggest `/awu-start ALGO-XXX-NNN`.

## Workflow

### 1. Spawn the translator agent

Use the `awu-translator` agent. Brief it with **only**:
- AWU id and one-line ALGORITHMS.md row
- Path + line range to the C source (do not paste the C inline; let the
  agent read it)
- Path to the target Rust file (already has the frozen signature)
- Path to `src/core/error.rs` so it knows the IgraphError
  variants
- Names of 1-2 already-merged AWU files for style reference (e.g.
  `src/algorithms/traversal/bfs.rs`)

Do NOT pass: full ALGORITHMS.md, other AWU sources, the entire MASTER_PLAN.

### 2. Wait for completion

The agent runs `cargo build` + `cargo clippy -- -D warnings` and reports a
5-line summary of choices made.

### 3. Independent review (high-risk AWUs only)

If the AWU is in one of these classes, also spawn `numerical-reviewer`:
- eigenvalue solvers (ALGO-LA-IRLM, IRAM, EIGEN-*)
- BLISS (ALGO-BLI-*)
- PageRank (ALGO-CT-005)
- spectral methods (ALGO-LO-040 MDS, ALGO-EM-*)
- iterative community algorithms (Louvain, Leiden, Spinglass)
- anything where ALGORITHMS.md complexity is `rewrite` or `novel`

If the reviewer says **Block**, surface the issues and stop. Do not proceed
to Step 5 testing.

### 4. Hand off

Tell the user:

> Implementation landed. Smoke: `cargo test`. Next:
> `/awu-test ALGO-XXX-NNN`

## When the agent gets stuck

The translator may report a translation it cannot finish (typically: needs
a data structure not yet built, or igraph C uses a calling convention that
doesn't map cleanly).

Options:
- **Block on a prerequisite AWU**: flip status to `blocked`, note the
  prerequisite in ALGORITHMS.md.
- **Escalate**: ask the user; the prerequisite may need to be promoted, or
  the AWU split.

Do NOT have the agent retry the same approach. Pick a different path.
