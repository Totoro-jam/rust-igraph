//! TEMPLATE: criterion benchmark for one AWU.
//!
//! Copy into `benches/bench_<algo>.rs` and add a `[[bench]]` entry to
//! `crates/igraph/Cargo.toml`:
//!
//!     [[bench]]
//!     name = "bench_{{ALGO_SLUG}}"
//!     path = "../../benches/bench_{{ALGO_SLUG}}.rs"
//!     harness = false
//!
//! Placeholders:
//!   {{ALGO_ID}}      e.g. ALGO-CT-002
//!   {{ALGO_SLUG}}    e.g. betweenness
//!   {{FN_NAME}}      e.g. betweenness

use std::fs::File;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use igraph::{read_edgelist, Graph};
// TODO({{ALGO_ID}}): use igraph::{{FN_NAME}};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// Synthetic ER-like sparse graph (placeholder until ALGO-GN-001 lands).
fn synthetic(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    for i in 0..n.saturating_sub(7) {
        g.add_edge(i, i + 7).unwrap();
    }
    g
}

fn bench_{{ALGO_SLUG}}_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("{{ALGO_SLUG}}/karate (34v 78e)", |b| {
        b.iter(|| {
            // TODO({{ALGO_ID}}): call the function under test
            // {{FN_NAME}}(&g, ...).unwrap()
            let _ = &g;
        });
    });
}

fn bench_{{ALGO_SLUG}}_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("{{ALGO_SLUG}}/synthetic");
    for n in [100u32, 1_000, 10_000] {
        let g = synthetic(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| {
                // TODO({{ALGO_ID}}): call the function under test
                let _ = g;
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_{{ALGO_SLUG}}_karate,
    bench_{{ALGO_SLUG}}_synthetic
);
criterion_main!(benches);
