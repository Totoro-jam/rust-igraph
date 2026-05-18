# rust-igraph

[![crates.io](https://img.shields.io/crates/v/rust-igraph.svg?label=crates.io)](https://crates.io/crates/rust-igraph)
[![docs.rs](https://img.shields.io/docsrs/rust-igraph?label=docs.rs)](https://docs.rs/rust-igraph)
[![CI](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml/badge.svg)](https://github.com/Totoro-jam/rust-igraph/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Totoro-jam/rust-igraph/branch/main/graph/badge.svg)](https://codecov.io/gh/Totoro-jam/rust-igraph)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)

A pure-Rust port of [igraph](https://igraph.org/) — the network analysis library.
Targets full API parity with igraph C v1.0.x (~850 public functions).

> **Status**: `0.0.1-alpha.0` — Phase 0 walking skeleton; only `Graph`,
> `read_edgelist`, and `bfs` ship. Not yet usable as a library. See
> [docs/plans/MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) for the roadmap
> and [.codefuse/tracking/ALGORITHMS.md](.codefuse/tracking/ALGORITHMS.md)
> for per-algorithm progress.

## Goals

- 100% public-API parity with `igraph` C core
- Pure Rust (no C FFI in default features); WASM-friendly
- Numerical results match `python-igraph` within tight tolerance
- Test suites from **all three** official implementations integrated:
  - `igraph` C — `tests/unit/*.c` + `*.out`
  - `python-igraph` — `tests/test_*.py`
  - `R-igraph` (`rigraph`) — `tests/testthat/test-*.R`

## License

GPL-2.0-or-later. Same as upstream igraph C, which permits direct reference-translation
of the C source. See [LICENSE](LICENSE).

## Project layout

```
rust-igraph/
├── src/
│   ├── lib.rs                 # crate root, re-exports
│   ├── core/                  # data structures (Graph, Vector, Matrix, ...)
│   └── algorithms/            # algorithm implementations
├── tests/
│   ├── oracle.rs              # live python-igraph oracle
│   ├── conformance.rs         # static fixtures from igraph C / py / R
│   ├── property.rs            # proptest invariants
│   └── conformance/{c,py,r}/  # extracted upstream test fixtures (JSON)
├── benches/                   # criterion benchmarks
├── examples/                  # usage examples
├── fixtures/                  # standard graph data (karate, dolphins, ...)
├── scripts/                   # oracle.py + test extractors
├── templates/                 # AWU templates (Step 3 skeleton source)
├── book/                      # mdBook source for the docs site
├── docs/
│   └── plans/MASTER_PLAN.md   # the engineering plan (single source of truth)
├── references/                # gitignored; clone igraph/python-igraph/rigraph here
└── .codefuse/tracking/        # ALGORITHMS.md, ARCHITECTURE.md, CONFORMANCE.md, ...
```

## Quick start

```bash
# build the crate (Phase 0 alpha — only Graph/read_edgelist/bfs ship today)
cargo build

# run the smoke-test example
cargo run --example bfs_karate

# run all tests including oracle (requires python-igraph installed)
cargo test --features oracle-tests
```

## Development workflow

Each algorithm is an **AWU** (Algorithm Work Unit) tracked in
`.codefuse/tracking/ALGORITHMS.md`. The 9-step SOP is in
[docs/plans/MASTER_PLAN.md §4](docs/plans/MASTER_PLAN.md). AI-assisted execution
uses the agents and skills under `.claude/`:

```
/awu-start    ALGO-XXX-NNN     # bootstrap an AWU
/awu-translate ALGO-XXX-NNN    # C → Rust translation (subagent)
/awu-test     ALGO-XXX-NNN     # unit + oracle + proptest
/awu-conformance ALGO-XXX-NNN  # extract from igraph C / py / R
/awu-bench    ALGO-XXX-NNN     # criterion baseline
/awu-finish   ALGO-XXX-NNN     # rustdoc + status update + PR template
```

## Acknowledgements

- Upstream [igraph](https://github.com/igraph/igraph) by the igraph team
- Reference test assets from [python-igraph](https://github.com/igraph/python-igraph) and [rigraph](https://github.com/igraph/rigraph)
- Linear algebra by [faer](https://github.com/sarah-quinones/faer-rs)
