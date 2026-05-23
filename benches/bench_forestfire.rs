//! Forest-fire random-graph generator benchmarks (ALGO-GN-006).
//!
//! Run: `cargo bench --bench bench_forestfire`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-006.json`.
//!
//! Coverage: directed and undirected at three vertex-count scales with
//! a moderate forward-burning probability (`fw_prob = 0.20`) so the
//! geometric tail stays bounded; `bw_factor = 0.40`; `ambs = 2`. These
//! settings sit in the sparse-but-non-trivial regime that is the
//! intended operating range for the model (per Leskovec et al.,
//! KDD'05).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::forest_fire_game;

const FW_PROB: f64 = 0.20;
const BW_FACTOR: f64 = 0.40;
const AMBS: u32 = 2;

fn bench_forestfire_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("forest_fire_game/directed");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| forest_fire_game(n, FW_PROB, BW_FACTOR, AMBS, true, 0xF1AE_F006).unwrap());
        });
    }
    group.finish();
}

fn bench_forestfire_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("forest_fire_game/undirected");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| forest_fire_game(n, FW_PROB, BW_FACTOR, AMBS, false, 0xF1AE_C006).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_forestfire_directed,
    bench_forestfire_undirected
);
criterion_main!(benches);
