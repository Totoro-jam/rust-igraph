//! Perfect-graph test baseline benchmarks (ALGO-PR-018).
//!
//! Run: `cargo bench --bench bench_perfect`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-PR-018.json`.
//!
//! Two regimes exercise the decision cascade of `is_perfect`:
//! even rings resolve through the bipartite fast path (perfect), while
//! odd rings resolve through the odd-girth fast path (not perfect). Both
//! avoid the exponential odd-hole LAD search, so cost tracks the
//! underlying linear primitives (`is_bipartite`, `girth`, `complementer`).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, is_perfect, ring_graph};

fn ring(n: u32) -> Graph {
    ring_graph(n, false, false, true).expect("ring")
}

/// Even cycle `C_n` is bipartite → perfect via the bipartite fast path.
fn bench_perfect_even_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("perfect/even_ring_bipartite");
    for n in [100u32, 1_000, 5_000] {
        let g = ring(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| is_perfect(g).expect("is_perfect"));
        });
    }
    group.finish();
}

/// Odd cycle `C_n` (n odd) has odd girth > 3 → not perfect, resolved by the
/// odd-girth check. This path first materialises the (dense) complement
/// for the Weak-Perfect-Graph-Theorem probes, so sizes stay modest.
fn bench_perfect_odd_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("perfect/odd_ring_not_perfect");
    for n in [51u32, 101, 501] {
        let g = ring(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| is_perfect(g).expect("is_perfect"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_perfect_even_ring, bench_perfect_odd_ring);
criterion_main!(benches);
