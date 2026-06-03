# Contributing to rust-igraph

**Contributions are welcome!** Whether it's a bug fix, new algorithm, documentation
improvement, or performance optimization — we appreciate your help making rust-igraph
the best graph library in the Rust ecosystem.

## Getting started

```bash
git clone https://github.com/Totoro-jam/rust-igraph.git
cd rust-igraph
cargo build
cargo test
```

## How to contribute

1. **Fork** the repository and create a branch from `main`.
2. **Make your changes** — see the guidelines below.
3. **Run the checks**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all --check
   ```
4. **Open a pull request** with a clear description of what and why.

## Guidelines

- **No `unsafe`** — the library is 100% safe Rust and we intend to keep it that way.
- **No `unwrap()`/`expect()`** in library code (tests are fine).
- **No new dependencies** without discussion — we keep the dep tree minimal.
- **Floating-point comparisons** use tolerance helpers, never `==`.
- **Integer arithmetic** uses `checked_*` / `try_from` to avoid silent overflow.
- **Public API** needs rustdoc with at least one doctest.
- **All code, comments, and identifiers in English.**

## What we're looking for

- Bug fixes with regression tests
- New algorithms (see [ALGORITHMS.md](.codefuse/tracking/ALGORITHMS.md) for coverage)
- Performance improvements with benchmarks
- Documentation improvements and examples
- Test coverage for edge cases
- I/O format support (GraphML, GML, GEXF, etc.)

## Algorithm contributions

New algorithms follow our AWU (Algorithm Work Unit) process:

1. Open an issue describing the algorithm and its use case.
2. Reference the igraph C implementation if applicable (see `references/igraph/`).
3. Include unit tests and at least one doctest.
4. Add a benchmark if the algorithm is non-trivial.

See [MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) for architecture context.

## Code style

- `cargo fmt` with default settings.
- `cargo clippy -- -D warnings` must pass.
- Comments only when the *why* is non-obvious — don't restate what code already says.
- Commit messages: `<type>(<scope>): short description`
  - types: `feat / fix / test / docs / refactor / perf / chore`

## Reporting bugs

Open an issue with:
- Minimal reproducer code
- Expected vs. actual behavior
- Rust version (`rustc --version`)

## Reporting security issues

See [SECURITY.md](SECURITY.md). Please do **not** open public issues for vulnerabilities.

## License

By contributing, you agree that your contributions will be licensed under
GPL-2.0-or-later (same as the project).
