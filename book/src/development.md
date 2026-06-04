# Development Notes

Detailed development setup and workflow notes are on GitHub:

**[View Development Notes on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/DEVELOPMENT.md)**

## Essential commands

```bash
# Build and test
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Full test suite (including conformance and proptests)
cargo test --workspace --all-features

# WASM compatibility check
cargo check --target wasm32-unknown-unknown
```

## Project structure

```
src/core/           # Graph, Vector, Matrix, attributes
src/algorithms/     # All algorithm implementations
tests/              # Integration, conformance, property tests
fixtures/           # Standard graph datasets
scripts/            # Oracle and test extraction tools
```
