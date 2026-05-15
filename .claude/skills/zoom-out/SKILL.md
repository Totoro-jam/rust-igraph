---
name: zoom-out
description: Pull back from a section of code or an algorithm to give a higher-level map. Use when the user says "zoom out", "what's the big picture", "where does this fit", or when the AI is stuck deep inside igraph C internals and needs perspective on the surrounding AWU dependency graph.
disable-model-invocation: true
---

I'm too deep in this. Go up a layer of abstraction.

Give me a one-screen map covering:

1. **Which AWU is this?** Quote the row from
   `.codefuse/tracking/ALGORITHMS.md` (id, status, deps).
2. **What sits above** — which algorithm AWUs will consume the function/
   module I'm in, once they land. Use the dependency column.
3. **What sits below** — which already-merged AWUs this one builds on.
4. **The igraph C neighbourhood** — the directory in
   `references/igraph/src/` we are currently in, and the 2-3 sibling C
   files most relevant to whatever I'm reading. Quote line ranges.
5. **The relevant ADR** — if any decision in
   `.codefuse/tracking/ARCHITECTURE.md` constrains this AWU, name the
   ADR id and one-line summary. (If none does, say so explicitly.)

Use the project's vocabulary — `igraph_t`, `IRLM`, `BLISS`, `oracle`,
`conformance`, `AWU` — not generic terms. Skip implementation details.
The output is for refocusing, not for implementation.

Adapted from <https://github.com/mattpocock/skills/tree/main/skills/engineering/zoom-out>.
