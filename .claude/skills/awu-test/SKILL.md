---
name: awu-test
description: Add unit tests + property tests + the live oracle test for one AWU. Steps 5-7 of the SOP. Use when user says "add tests for ALGO-...", "test ...", or after `/awu-translate` lands.
---

# /awu-test ALGO-XXX-NNN

Steps 5-7 of the AWU SOP. Goal: green tests at three layers (unit + proptest
+ live oracle) before conformance and bench.

## Workflow

### 1. Unit tests + property tests (delegate to `awu-tester`)

Spawn the `awu-tester` agent. It writes:
- A `#[cfg(test)] mod tests` block in the algorithm file with at least:
  empty graph, single vertex, complete K5, error path, and any algorithm-
  specific edge cases.
- 1-2 proptest invariants in `tests/property.rs`.

Verify both run:
```
cargo test <algo_name>
cargo test --features proptest-harness --test property
```

### 2. Live oracle test (main agent — do NOT delegate)

Add a test to `tests/oracle.rs`:

```rust
#[test]
fn <algo>_<fixture>_matches_python_igraph() {
    let g = /* build or load graph */;
    let rust_result = igraph::<algo>(&g, ...).unwrap();
    let py_result: <type> = serde_json::from_value(
        run_ok("<algo>", &g, json!({...}))
    ).expect("decode");
    assert_eq!(rust_result, py_result);  // or assert_close! for floats
}
```

Then add the corresponding branch in `scripts/oracle.py` if not yet present
(use `/oracle-add <algo>` if the API surface is non-trivial).

Verify:
```
cargo test --features oracle-tests --test oracle <algo>
```

### 3. Diagnose mismatches with discipline

If oracle says the Rust result differs from python-igraph:

1. **Do not weaken the assertion to make it pass.**
2. Build a fast feedback loop: shrink the failing fixture to the smallest
   graph that reproduces the divergence (often n ≤ 10).
3. Compare against `references/igraph/src/<C source>` line by line for
   the divergence point.
4. Common causes (check first):
   - Off-by-one in vertex/edge indexing
   - Float comparison without tolerance
   - Different default for an optional parameter
   - Different graph construction in oracle.py (e.g., `circular=True` vs
     `False` — see scripts/test_extract/from_c.py history for a real
     example)
5. Fix the root cause, not the symptom.

### 4. Hand off

Tell the user:

> Tests green at three layers. Next: `/awu-conformance ALGO-XXX-NNN`
