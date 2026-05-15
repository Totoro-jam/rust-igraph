---
name: awu-conformance
description: Add three-source conformance fixtures (igraph C / python-igraph / R-igraph) for one AWU and verify they all pass. Step 6b of the SOP. Use when user says "add conformance for ALGO-...", "wire up C/py/R tests", or after `/awu-test` passes.
---

# /awu-conformance ALGO-XXX-NNN

Step 6b of the AWU SOP. Wires the AWU into the three-source conformance
test so any future drift from the official implementations breaks CI.

## Pre-checks

- The AWU's live oracle test (Step 6a) is green
- `references/{igraph,python-igraph,rigraph}` are populated
- `.venv/` exists with python-igraph installed

## Workflow

### 1. Delegate to `conformance-extractor`

Brief the agent with:
- AWU id and algorithm slug (e.g. `betweenness`)
- Specific upstream test sources to mirror:
  - igraph C: list `references/igraph/tests/unit/<algo>*.c` and the
    corresponding `.out` files
  - python-igraph: list test methods in
    `references/python-igraph/tests/test_*.py` that exercise the algorithm
  - R-igraph: list `expect_equal` blocks in
    `references/rigraph/tests/testthat/test-*.R`

The agent extends the manifests in `scripts/test_extract/from_{c,py,r}.py`
and runs each extractor.

### 2. Verify

```
.venv/bin/python -m scripts.test_extract.from_c  --algo <slug>
.venv/bin/python -m scripts.test_extract.from_py --algo <slug>
.venv/bin/python -m scripts.test_extract.from_r  --algo <slug>
ls tests/conformance/{c,py,r}/<slug>/
cargo test -p igraph --test conformance
```

Hard requirement: **at least one fixture from each of the three sources**
(c, py, r). Phase 0 gate.

### 3. Update CONFORMANCE.md

Append a row to `.codefuse/tracking/CONFORMANCE.md`:

```
| <algo> | <C count> | <py count> | <R count> | <rust pass / total> | - |
```

### 4. Diagnose conformance failures

If the conformance test fails (Rust output ≠ upstream expected):

1. Check the failing source first: C fixtures often catch parameter-meaning
   bugs (e.g. `circular=0` is a path, not a ring); py fixtures catch
   high-level API mismatches; R fixtures catch 1-vs-0-based indexing slips.
2. Reproduce on the smallest failing fixture.
3. **Trust the upstream value.** All three official implementations agree
   on igraph's reference behavior; if Rust diverges, Rust is wrong.
4. Fix the root cause in the algorithm, not in the fixture.

### 5. Hand off

Tell the user:

> Three-source conformance green. Next: `/awu-bench ALGO-XXX-NNN`
