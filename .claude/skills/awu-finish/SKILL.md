---
name: awu-finish
description: Wrap up an AWU — polish rustdoc/doctest, add example, flip status to done, draft the PR description. Step 9 of the SOP. Use when user says "finish ALGO-...", "wrap up ...", or after `/awu-bench` completes.
---

# /awu-finish ALGO-XXX-NNN

Step 9 of the AWU SOP: cross every t before the PR.

## Workflow

### 1. Delegate to `doc-writer`

The agent expands rustdoc to include Arguments / Returns / Errors /
Examples (with a runnable doctest), and adds an `examples/<algo>_demo.rs`
if the AWU is user-facing.

### 2. Run the full local matrix

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace --all-features
cargo doc   --workspace --no-deps
```

Every command must be green. If any fails, do not flip the AWU status.

### 3. Update tracking

In `.codefuse/tracking/ALGORITHMS.md`:
- Flip `ALGO-XXX-NNN` status from `wip` (or `review`) to `done`
- Fill in the `Bench` column with the karate timing
- Fill in the `Conformance` column with `C:<n> / py:<n> / R:<n>`

### 4. Draft the PR description

Use this template (matches `.github/pull_request_template.md`):

```markdown
## ALGO-XXX-NNN: <human title>

### C reference
- File: `references/igraph/src/.../<file>.c` (~<N> lines)
- Tests: `references/igraph/tests/unit/<test>.c`

### Implementation notes
<2-4 lines: data structures used, deviations from C, any reviewer flags>

### Test coverage
- [ ] Unit tests (empty / single / complete / random / error path)
- [ ] proptest invariants (X, Y)
- [ ] Live oracle on karate (matches python-igraph 0.11.x)
- [ ] Conformance fixtures: C:<n> / py:<n> / R:<n>
- [ ] Criterion baseline: <X ns/karate>

### Checklist
- [x] cargo fmt / clippy clean
- [x] cargo test --workspace --all-features
- [x] cargo doc + doctest
- [x] WASM check (when applicable)
- [x] ALGORITHMS.md updated
- [x] perf/<ALGO-XXX>.json committed
```

### 5. Hand off

Tell the user the AWU is ready for PR review. Suggest:

> `git add -A && git commit -m "feat(<scope>): <ALGO-XXX> <title>"`
>
> Open PR titled `ALGO-XXX-NNN: <title>`.

## Anti-patterns

- **Do NOT flip status to `done` if any test is failing or skipped.**
  Use `blocked` instead and explain.
- **Do NOT add unrelated cleanup to the PR.** Each AWU is one focused
  change.
- **Do NOT remove TODO/perf-todo markers** unless they are addressed.
