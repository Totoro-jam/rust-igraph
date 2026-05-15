---
name: prototype-logic
description: Build a throwaway prototype to validate a tricky algorithm's logic before going through the full AWU SOP. Use for IRLM/IRAM/BLISS/Walktrap-class algorithms where the C source is dense and a 1:1 translation might mask design errors. Use when the user says "prototype this", "let me play with the math first", or "before we commit to the AWU, can we sketch X".
---

# Prototype-logic

A throwaway terminal-app-style prototype that lets you push the
algorithm through hand-picked cases before locking in the AWU's
interface and Step-4 implementation.

> Adapted from <https://github.com/mattpocock/skills/tree/main/skills/engineering/prototype>
> (LOGIC variant only — we have no UI).

## When this is the right move

The standard AWU SOP (`/awu-start` → `/awu-translate` → ...) assumes
the C source is the design. That's true for `add_vertices` / `BFS` /
`Dijkstra` — the C is the authoritative spec.

It's *less* true for:

- **IRLM / IRAM** (`references/igraph/src/linalg/arpack.c`, ~1600 lines
  of dense Fortran-style numerics in C). 1:1 translation will compile
  but may be subtly wrong; debugging numerical drift after the fact is
  expensive (see [diagnose](../diagnose/SKILL.md)).
- **BLISS** (~9500 lines of C++ with custom data structures). Need to
  understand the partition refinement before committing to a Rust shape.
- **Walktrap / Spinglass / DrL / HRG** — C++ ports; the original code
  uses class hierarchies that won't translate idiomatically.
- Anything in MASTER_PLAN tagged `rewrite` complexity (vs `adapt` /
  `copy`).

For these, prototype first. The AWU SOP picks up the validated design
afterwards.

## Process

### 1. Write the question on a Post-it

Before any code, in 1-2 sentences, state:
- The specific question the prototype answers
  ("does the implicit-restart shift correctly recover the next Krylov
  basis after a bad iteration?")
- What "answered" looks like (the prototype's exit predicate)

If the question is fuzzy, the prototype is wasted.

### 2. Pick the smallest possible setting

- For IRLM: a 4×4 symmetric matrix where you can hand-compute eigenvalues.
- For BLISS: a 6-vertex graph with a known automorphism group (e.g.
  Petersen's tiny cousin K_{3,3}).
- For Walktrap: 10 vertices, two clear communities.

### 3. Write a `prototype-<algo>.rs` example

Place under `examples/prototype-<algo>.rs` (NOT `src/`). Naming convention
makes it obvious to readers that this is throwaway. Add to `Cargo.toml`:

```toml
[[example]]
name = "prototype-<algo>"
path = "examples/prototype-<algo>.rs"
```

Inside, a `main()` that:
- Builds the tiny test setting in code (no fixture loading).
- Runs the candidate algorithm step by step.
- After every step, **prints the full relevant state** — eigenvalues so
  far, Krylov basis vectors, partition cells, whatever.
- Runs against expected output (compare to a known-good reference —
  numpy / sympy / python-igraph for the same matrix).

### 4. Iterate by editing the prototype, not the real code

The point is to fail fast on understanding. Don't wire up Cargo
features, don't write tests, don't worry about clippy.

### 5. Capture the answer

When the prototype convinces you the design is right (or wrong), the
**answer is the only thing worth keeping**. Write a short ADR or a
comment at the top of the eventual real implementation. Specifically:
- Which design choice did the prototype validate (or invalidate)?
- What's the smallest case that would have caught the mistake?

That second point becomes a `tests/conformance/c/<algo>/<small_case>.json`
when the AWU lands.

### 6. Delete the prototype

Or move it under `examples/archived/<algo>-<date>.rs` if you genuinely
think someone will want to revisit. Default action: delete. The captured
answer (Step 5) is what survives.

## Anti-patterns

- **Don't prototype `BFS`-class algorithms.** They're simple enough that
  the AWU SOP catches design errors via oracle.
- **Don't keep the prototype around past 1 working day** without a
  reason. Rotting examples confuse the next reader.
- **Don't add tests to the prototype.** Tests are for the real code.
  The prototype's "test" is your own eyes watching state.
- **Don't make the prototype configurable.** One question, one
  hardcoded setting.
