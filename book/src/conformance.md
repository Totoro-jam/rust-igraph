# Conformance Matrix

rust-igraph is cross-validated against three official igraph implementations. The full conformance matrix is on GitHub:

**[View Conformance Matrix on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/CONFORMANCE.md)**

Conformance testing ensures that rust-igraph produces numerically identical results to:

- **igraph C** (v0.10.x) — the reference implementation
- **python-igraph** (v0.11.x) — Python bindings
- **R-igraph** (rigraph) — R bindings

Test fixtures are extracted from upstream test suites and stored in `tests/conformance/{c,py,r}/`.
