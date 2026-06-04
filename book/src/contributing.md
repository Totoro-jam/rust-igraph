# Contributing

rust-igraph welcomes contributions! The full contributing guide is on GitHub:

**[View Contributing Guide on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/CONTRIBUTING.md)**

## Quick start

1. Fork and clone the repository
2. Run `cargo test --workspace` to verify your setup
3. Pick an unassigned AWU from the [algorithm tracker](./algorithm-tracker.md)
4. Follow the 9-step AWU SOP described in the [master plan](./master-plan.md)
5. Open a PR with `ALGO-XXX-NNN` in the title

## Key rules

- No `unsafe` blocks
- No new dependencies without an ADR
- All public APIs need rustdoc with a doctest
- Floating-point comparisons use tolerance helpers, never `==`
