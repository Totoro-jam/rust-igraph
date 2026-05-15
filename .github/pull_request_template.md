<!--
PR template for an Algorithm Work Unit (AWU). For non-AWU PRs (chore, ci,
docs), delete the AWU sections and keep only Summary + Checklist.
-->

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

- [ ] cargo fmt / clippy clean
- [ ] cargo test --workspace --all-features
- [ ] cargo doc + doctest
- [ ] WASM check (when applicable)
- [ ] `.codefuse/tracking/ALGORITHMS.md` status updated
- [ ] `.codefuse/tracking/perf/<ALGO-XXX>.json` committed
- [ ] No new dependencies (or, if any, an ARCHITECTURE.md ADR is included)
