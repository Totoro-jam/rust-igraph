---
name: awu-start
description: Bootstrap a new Algorithm Work Unit. Reads ALGORITHMS.md to find the AWU, recons the upstream igraph C source, drafts a Rust interface for user approval, then writes a skeleton file and flips the AWU status to wip. Use when user says "start AWU ...", "begin ALGO-...", "let's implement ...", or pastes an ALGO-XXX-NNN id.
---

# /awu-start ALGO-XXX-NNN

Step 1-3 of the 9-step AWU SOP. Goal: get to a compiling skeleton with a
frozen interface, ready for `/awu-translate`.

## Prerequisites

- Repo at workspace root
- `references/igraph/` is populated (see `references/README.md`)
- `templates/algo.rs.tpl` and `templates/test.rs.tpl` exist (BOOT-19/20)

If any prerequisite is missing, **stop and tell the user**. Do not try to
synthesize a template inline.

## Workflow

### 1. Locate the AWU

Read `.codefuse/tracking/ALGORITHMS.md`. Find the row for
`ALGO-XXX-NNN`. If status is not `todo` or `blocked`, ask the user before
proceeding (someone else may have started it).

### 2. Recon (delegate to `igraph-c-recon`)

Spawn the `igraph-c-recon` agent. Brief:
- igraph C source path + line range from ALGORITHMS.md
- relevant header in `references/igraph/include/`
- one or two unit tests under `references/igraph/tests/unit/` matching the
  algorithm name

Wait for its summary (≤300 words). Surface the summary to the user.

### 3. Draft the interface

From the recon summary, draft **one Rust signature**. Format:

```rust
pub fn <algo_name>(
    graph: &Graph,
    /* params, types, doc-style names */
) -> IgraphResult</* return type */>;
```

Show this to the user. **Wait for explicit approval** before writing files.
Default position: match igraph C's parameter shape; deviate only when Rust
ownership requires it (and explain the deviation).

### 4. Write the skeleton

Once approved:

1. Copy `templates/algo.rs.tpl` to the target module path. Fill in:
   - `name` field in the `ALGO-XXX-NNN` doc comment
   - the C source reference
   - the public signature
   - body: `unimplemented!("ALGO-XXX-NNN")`
2. Copy `templates/test.rs.tpl` to the test path. Empty test bodies.
3. Add a placeholder branch (commented out) in `scripts/oracle.py` so the
   user remembers to wire it in `/awu-conformance`.
4. Run `cargo build`. Must compile.

### 5. Update tracking

In `.codefuse/tracking/ALGORITHMS.md`:
- Flip `ALGO-XXX-NNN` status from `todo` to `wip`
- Note the date and which agent started it

### 6. Hand off

Tell the user:

> Interface frozen, skeleton compiles. Next: `/awu-translate ALGO-XXX-NNN`

## Anti-patterns

- **Do NOT translate the C now.** That's `/awu-translate`. Premature
  translation pollutes context.
- **Do NOT design generics or abstractions speculatively.** Match igraph C
  first; refactor only if a real second use case appears.
- **Do NOT update ALGORITHMS.md status until the skeleton actually
  compiles.** A broken `wip` is worse than a `todo`.
