//! Mycielskian / Mycielski-graph generator benchmarks (ALGO-CN-019).
//!
//! Run: `cargo bench --bench bench_mycielskian`.
//! A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-019.json`.
//!
//! Coverage: the parameter-only chain `mycielski_graph(k)` for
//! `k ∈ {6, 8, 10}` (the recurrence makes k=10 expand to 2047 vertices /
//! ~8.2e6 edges — adequate to stress the resize/assign hot path), and the
//! arbitrary-input form `mycielskian(C_n, 2)` for `n ∈ {32, 64, 128}` so
//! the cost model picks up both axes (input size and iteration count).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{cycle_graph, mycielski_graph, mycielskian};

fn bench_mycielski_graph_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("mycielski_graph/by_k");
    for k in [6u32, 8, 10] {
        // expansion factor: vcount(k) = 2^(k-1) + 1 once past the base cases
        let projected_v = 1u64 << (k - 1);
        group.throughput(Throughput::Elements(projected_v));
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| mycielski_graph(k).unwrap());
        });
    }
    group.finish();
}

fn bench_mycielskian_on_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("mycielskian/cycle_k2");
    for n in [32u32, 64, 128] {
        // two iterations on C_n: vcount = 4n+3, ecount = 9n+n+2n = ...
        // not the point; we report n itself (input size) as the throughput.
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let cyc = cycle_graph(n, false, false).unwrap();
            b.iter(|| mycielskian(&cyc, 2).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_mycielski_graph_k, bench_mycielskian_on_cycle);
criterion_main!(benches);
