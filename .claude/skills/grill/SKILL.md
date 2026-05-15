---
name: grill
description: Stress-test an AWU plan by relentlessly interrogating it against MASTER_PLAN.md, ARCHITECTURE.md ADRs, and ALGORITHMS.md state — one question at a time. Use when the user wants to lock down a non-trivial design decision before Step 4 implementation, or says "grill me", "challenge my plan", "are we sure about this", "before we commit to <design>".
---

# Grill

A hostile-questioning interview that walks down the design tree of a
plan one branch at a time, resolving dependencies between decisions
before any code lands. Useful before high-stakes AWUs (data structure
boundaries, BLISS partition design, IRLM convergence criteria, anything
tagged `rewrite` in `ALGORITHMS.md`).

> Adapted from <https://github.com/mattpocock/skills/tree/main/skills/engineering/grill-with-docs>.
> Substitutes our existing tracking docs for the original's `CONTEXT.md`.

## What this skill does

You — the AI — interview the user **relentlessly** about every aspect
of the plan until you reach a shared understanding. Walk down each
branch of the design tree, resolving dependencies between decisions
one-by-one. **For each question, give your recommended answer**
(don't ask open-ended questions; offer a default).

Ask **one question at a time**. Wait for feedback before continuing.

If a question can be answered by exploring the codebase, explore the
codebase instead.

## Where to find the project's existing decisions

Before asking, read what's already settled. The grilling refines the
decisions, not the foundations.

1. **`docs/plans/MASTER_PLAN.md`** — the overall roadmap, Phase
   structure, complexity tags. The plan was already merged of 6 earlier
   plans; treat it as authoritative.
2. **`.codefuse/tracking/ARCHITECTURE.md`** — the ADR index. Each ADR
   is a binding decision (license, crate shape, linear-algebra backend,
   isomorphism path, three-source conformance, AWU SOP). Quote the ADR
   id when an answer is constrained by one.
3. **`.codefuse/tracking/ALGORITHMS.md`** — current AWU statuses + deps.
   If a proposed split conflicts with an existing AWU's deps column,
   surface it.
4. **`.codefuse/tracking/AI_PROMPTS.md`** — captured prompts and
   "踩过的坑" notes. Past traps can save the current plan.
5. **`CLAUDE.md`** — the hard constraints (no unsafe, no unwrap, no
   new deps without ADR). Use these as immediate reject criteria.

If a decision is *not* documented in those, the grilling session might
end with a new ADR being drafted (`ARCHITECTURE.md` gets a new
`ADR-NNNN` row).

## How the grill is shaped

For each branch in the design tree:

1. **Restate the decision being made** — one sentence.
2. **Quote relevant existing constraints** — ADR ids, MASTER_PLAN
   sections, CLAUDE.md rules.
3. **Offer 2-3 options** with trade-offs. Bias the question toward a
   recommendation, with reasoning.
4. **Ask** one specific yes/no or pick-one question.
5. **Wait for the user's answer.** Do not chain.
6. **If the answer creates a new constraint**, capture it inline — add
   to ALGORITHMS.md (status note), MASTER_PLAN.md (a TODO), or draft an
   ADR — before moving on.

## Example (BLISS partition refinement, hypothetical)

> Decision: how to represent BLISS partition cells in Rust.
>
> Constraints:
> - ADR-0004 says we translate BLISS C++ → Rust 1:1 for API parity.
> - C++ uses `std::list<std::list<int>*>*` with custom intrusive
>   pointers, see `references/igraph/src/isomorphism/bliss/partition.cc:42-110`.
> - CLAUDE.md forbids `unsafe`.
>
> Options:
> 1. `Vec<Vec<u32>>` — simplest; loses BLISS's O(1) cell splitting.
> 2. Slab-allocated nodes with indices for "pointers" — preserves O(1)
>    operations; needs a custom slab.
> 3. `Rc<RefCell<Cell>>` linked list — closer to C++ but slower.
>
> Recommended: **2 (slab + indices)**. The 1:1 reasoning in ADR-0004
> only applies to *behaviour*; storage shape is ours to pick. O(1)
> matters for graphs with > 10⁴ vertices, which is the BLISS use case.
>
> Question: do you accept the slab approach, or want option 1's
> simplicity for the first cut and revisit if benchmarks regress?

## When to stop

Stop when:
- The user says "OK, ship that" or moves on.
- The branch is exhausted — every leaf has a one-sentence decision and
  a captured constraint (ADR / ALGORITHMS row / MASTER_PLAN edit).
- A question you'd ask is answerable by `/zoom-out` or by reading the
  C source — go do that instead, then come back with concrete answers.

## Anti-patterns

- **Don't ask open-ended questions** ("what do you think about X?").
  Always offer the recommended default.
- **Don't chain questions** ("and also, …"). One at a time.
- **Don't grill on already-decided things.** ADR-0001 (license) is
  not up for re-litigation in every grilling session. If the user
  *does* want to re-open it, that's a separate ADR-supersedes-ADR
  conversation.
- **Don't grill trivially-decidable things.** "What should the
  function be named?" is not grill material; pick a name and move on.
