---
name: phase-checkpoint
description: Run the full local Phase exit gate (fmt, clippy, test all features, conformance, bench compile, doc, WASM check) and write a retro entry. Use when user says "phase checkpoint", "ready to bump version", "phase X done?", or wants to verify Phase 0/1/2/... is complete.
---

# /phase-checkpoint [<phase-id>]

Verify all the gates a Phase must pass before declaring it complete and
bumping the version. Catches drift early — every Phase should pass this
once near completion.

## Pre-checks

- Working tree clean (committed everything)
- On `main`

## Phase exit gate (the checks)

Run each of these. **All must pass.**

```bash
# Formatting + lint
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings

# Default test loop
cargo test --workspace

# Full feature loop (oracle + proptest)
cargo test --workspace --all-features

# Conformance must include all three sources for every done AWU
cargo test --workspace --test conformance

# Benches compile (do not run full)
cargo build --benches

# Doc build (catches broken doc links / missing exports)
cargo doc --workspace --no-deps

# WASM target compiles (zero external deps — pure Rust)
cargo check --target wasm32-unknown-unknown 2>&1 || echo "(WASM check — install target with rustup target add wasm32-unknown-unknown)"

# Cargo deny (license + advisories)
cargo deny check 2>&1 || echo "(install cargo-deny if missing)"
```

## ALGORITHMS.md audit

Open `.codefuse/tracking/ALGORITHMS.md`. Verify:

- [ ] No `wip` AWU older than 4 weeks (flip stale ones to `todo` and open
      an issue per RESUME.md guidance)
- [ ] No `done` AWU missing a Bench column entry
- [ ] No `done` AWU missing a Conformance column entry
- [ ] Phase counters at the bottom match reality

## Coverage check

Phase exit threshold: **≥ 85% of the Phase's AWUs are `done`**, and
**100% of P0 AWUs in that Phase are `done`**.

If under threshold, list the missing AWUs and stop. Do not write a retro
yet.

## Write the retro

Append to `.codefuse/tracking/RETRO.md`:

```markdown
## Phase <N> — <date>

**Status**: <complete / near-complete (XX%) / blocked>

### What landed
- <bullets>

### What surprised me
- <unexpected findings — these inform plan adjustments>

### What to defer
- <perf-todo / blocked AWUs to revisit in Phase <N+1>>

### Next Phase
- Start: <next ALGO ids>
- Risks: <call out any prerequisite gaps>
```

## Bump version

If the gate passes and the retro is written, suggest the version bump:

```
# Cargo.toml
[workspace.package]
version = "0.<N+1>.0"
```

Do NOT bump version yourself; tag and crates.io release are user actions.

## Anti-patterns

- **Do NOT skip a check** because "it has always passed." Phase
  checkpoints catch the slow drift that everyday `cargo test` misses.
- **Do NOT relax the 85% threshold** to declare a Phase done. Move the
  remaining AWUs to the next Phase as `todo` instead, with a brief note.
