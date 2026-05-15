# AI prompt cookbook

Prompts that worked well in practice. Each entry: when to use, the prompt
template, and a "踩过的坑" note for what to avoid.

> Update this file whenever you find a prompt that hits the target on the
> first try. Stale prompts (model upgraded, conventions changed) get a
> ⚠ tag and a note; don't delete unless you have a replacement.

---

## Recon (AWU Step 1)

**Used by**: `igraph-c-recon` agent (BOOT-34).
**Last verified**: 2026-05-15 with sonnet-4-7.

```
You are doing recon for {ALGO-XXX-NNN}: {one-line algo name}.

Read these files (line ranges given) and produce a focused summary:
- C source: references/igraph/{path}:{L1}-{L2}
- Header:   references/igraph/include/{header}
- Test:     references/igraph/tests/unit/{test_file}

Output ≤300 words covering:
1) Recommended Rust signature (vs C signature)
2) Input shape: graph type, parameter ranges, defaults
3) Output shape and ordering guarantees
4) Edge cases (empty / single / self-loop / parallel / unreachable / ...)
5) Numerical concerns (tolerance, convergence)
6) Recommended fixtures for testing
7) `.out` file name(s) for conformance extraction

Quote line numbers, not large code blocks.
```

**踩过的坑**: omitting "≤300 words" leads to a transcription of the C code
and bloated context. Hard cap is essential.

---

## Translate (AWU Step 4)

**Used by**: `awu-translator` agent.
**Last verified**: 2026-05-15.

```
Translate {ALGO-XXX-NNN} from igraph C to Rust.

Inputs:
- C source range: references/igraph/{path}:{L1}-{L2}
- Frozen Rust signature: {file}:{line} (already in target file)
- Style reference: {file_of_a_neighboring_done_AWU}

Constraints (from CLAUDE.md):
- No unsafe, unwrap, expect outside tests
- No new dependencies
- Match IgraphError variants for C error codes
- Float comparisons use tolerance helpers
- Integer arithmetic uses checked_*

Replace `unimplemented!()` with the body. Run cargo build + cargo clippy
-- -D warnings. Output a 5-line summary: data structures, allocations,
deviations from C, anything that needs review.
```

**踩过的坑**: pasting the full C source inline wastes context and tempts
the model to copy comments/structure verbatim. Pass file paths instead.

---

## Numerical review (high-risk AWUs)

**Used by**: `numerical-reviewer` agent.

```
You are an independent reviewer for {ALGO-XXX-NNN}, a {algo class}
algorithm. The implementer just finished Step 4 — you are looking for
issues they may have missed.

Read:
- Rust impl: {file}:{line}
- C reference: references/igraph/{path}:{L1}-{L2}

Check (in this order):
1) Convergence — is there a stopping criterion? What if max_iter is hit?
2) Numerical stability — subtractions of nearly-equal floats?
   Reorthogonalization where needed? NaN/Inf propagation?
3) Edge cases — disconnected, dangling, zero-weight, self-loops, regular
   graphs, strongly-regular graphs (BLISS hard cases)
4) Tolerance choices — bound to a justification? Match igraph C defaults?
5) Integer overflow — checked_* / try_from where it matters?

Output ≤200 words with verdict: OK / OK with notes / Block.
Be precise: cite Rust file:line AND igraph C file:line for every issue.
```

**踩过的坑**: vague verdicts ("could be more stable") are useless. Force
file:line citations.

---

## Conformance extraction (per AWU)

**Used by**: `conformance-extractor` agent.

```
Extend three-source conformance for {ALGO-XXX-NNN} (slug: {algo_slug}).

Sources to mirror:
- igraph C: references/igraph/tests/unit/{file}.c (+ .out)
- python-igraph: references/python-igraph/tests/{file}.py method {name}
- R-igraph: references/rigraph/tests/testthat/{file}.R block {name}

For each source:
1) Append a manifest entry to scripts/test_extract/from_{c,py,r}.py
2) Build the same graph via python-igraph (regardless of source)
3) Use the upstream test's expected value verbatim
4) For R: subtract 1 from all vertex ids (1-based -> 0-based)

Run all three extractors. Run cargo test --test conformance.
Both must be green. Report fixture counts per source.

If a source's test does not have a clean equivalent in our minimal API,
pick a simpler test from the same file rather than fudging values.
```

**踩过的坑**: the BFS C extractor used `circular=True` mistakenly;
conformance test caught it. Always re-read the C test parameters carefully
— igraph's defaults differ from python-igraph's.

---

## Resume after a break

**Used by**: `resume-session` skill.

No fixed prompt — the skill itself drives the work. But the *first message*
back to the user should always be in this shape:

```
Last commit: {sha} — {title} ({date})
Working tree: {clean | dirty list}
Open: {N} wip, {M} blocked
Tests on main: {green | listing failures}

Suggested next step: /awu-{step} {ALGO-XXX-NNN}
Reason: {one line}
```

**踩过的坑**: jumping to "let's start ALGO-X" without verifying tests on
`main` are green can mask a regression that committed broken.

---

## When the AI gets stuck

If three attempts fail to produce a clean result:

```
Stop. Do not retry. Report:
1) What you tried (one line each)
2) Where it broke (file:line, error message)
3) Whether this is a missing prerequisite, a knowledge gap, or a real
   blocker
4) Recommended next action: block on AWU X / ask user / try smaller scope
```

**踩过的坑**: letting the agent retry the same approach 5+ times burns
context and produces increasingly desperate code. Three strikes and you
escalate.

---

## Pre-prepared Phase-1 entry: ALGO-CORE-001 recon brief

**Status**: not yet executed; ready for next session.
**Skill to invoke**: `/awu-start ALGO-CORE-001`.

When the user types that, the recon delegate (`igraph-c-recon` agent,
haiku) should be briefed with:

```
You are doing recon for ALGO-CORE-001: real Graph (igraph_t equivalent).

This is the foundational AWU of Phase 1. It replaces the throwaway
Graph<u32> currently sitting in src/core/graph.rs (Phase-0 placeholder
per ADR-0007). Most subsequent algorithm AWUs depend on this.

Read these files (do NOT inline; quote line numbers):
- C source:   references/igraph/src/graph/type_indexededgelist.c (2013 lines)
- Header:     references/igraph/include/igraph_datatype.h  (struct igraph_t)
- Interface:  references/igraph/include/igraph_interface.h (ops on igraph_t)
- Tests:      references/igraph/tests/unit/igraph_create.c
              references/igraph/tests/unit/igraph_add_*.c
              references/igraph/tests/unit/igraph_delete_*.c

Output ≤500 words (this AWU is bigger than the per-AWU 300-word cap):
1) Recommended Rust struct shape — fields, ownership model. Compare to
   the C `igraph_t` (n, directed, from, to, oi, ii, os, is, attr,
   cached_props).
2) Boundaries for what THIS AWU covers vs what splits into follow-up
   AWUs. Suggested split: ALGO-CORE-001 = struct + create/destroy/copy +
   add/delete vertices/edges; ALGO-CORE-002+ = degree/neighbors/incident
   queries; ALGO-CORE-010+ = property queries (is_directed/is_simple/...).
3) Public API list for ALGO-CORE-001 with proposed Rust signatures.
   Show every method's igraph_t-equivalent and whether it currently
   exists on the Phase-0 `Graph<u32>` placeholder.
4) Storage choice rationale: CSR vs adjacency-list-of-Vec. C uses an
   indexed edge list (`from` / `to` / `os` / `is`). What's the Rust
   equivalent? Trade-offs.
5) Edge cases to test: empty / single / self-loop / parallel edges /
   directed-with-reverse / very-large.
6) Numerical concerns (none expected; confirm).
7) `.out` files relevant for conformance extraction once the AWU lands.
8) Migration plan: how Phase-0 callers (only BFS today) get re-pointed
   from the placeholder to the new Graph without churn during the AWU's
   `wip` window.

If the AWU clearly should be split into multiple, recommend the split
and DO NOT proceed past Step 1. The user will then create the sub-AWUs
in ALGORITHMS.md.
```

**踩过的坑**: ALGO-CORE-001 is the only AWU big enough that the recon
should bust the 300-word cap (raised to 500). Subsequent AWUs go back
to 300.

**踩过的坑**: the Phase-0 `Graph<u32>` placeholder has only 5 methods
(`with_vertices`, `add_edge`, `add_edges`, `vcount`, `ecount`,
`neighbors`, `degree`). Any new public surface in ALGO-CORE-001 that
collides will need a deprecation shim or an `unimplemented!()` placeholder
to keep BFS / EdgeList / oracle tests compiling during the AWU's `wip`
window.
