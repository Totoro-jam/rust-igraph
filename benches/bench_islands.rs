//! Simple interconnected islands random-graph benchmarks (ALGO-GN-007).
//!
//! Run: `cargo bench --bench bench_islands`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-007.json`.
//!
//! Coverage: three island counts at fixed island size (50), moderate
//! within-island density (`islands_pin = 0.10`), and a small bipartite
//! slice (`n_inter = 3`). Throughput is reported in total vertices
//! (`islands_n * islands_size`) so wall-clock per element is directly
//! comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::simple_interconnected_islands_game;

const ISLANDS_SIZE: u32 = 50;
const ISLANDS_PIN: f64 = 0.10;
const N_INTER: u32 = 3;

fn bench_islands_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_interconnected_islands_game/n_islands");
    for islands_n in [4u32, 20, 100] {
        let total = u64::from(islands_n) * u64::from(ISLANDS_SIZE);
        group.throughput(Throughput::Elements(total));
        group.bench_with_input(
            BenchmarkId::from_parameter(islands_n),
            &islands_n,
            |b, &islands_n| {
                b.iter(|| {
                    simple_interconnected_islands_game(
                        islands_n,
                        ISLANDS_SIZE,
                        ISLANDS_PIN,
                        N_INTER,
                        0x1514_4AD5,
                    )
                    .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_islands_density(c: &mut Criterion) {
    // Fix the lattice (10 islands × 30 vertices = 300 vertices) and
    // sweep within-island density to capture the geometric-skip cost.
    let mut group = c.benchmark_group("simple_interconnected_islands_game/pin_sweep");
    let islands_n = 10u32;
    let size = 30u32;
    let total = u64::from(islands_n) * u64::from(size);
    for pin in [0.05_f64, 0.20, 0.50] {
        group.throughput(Throughput::Elements(total));
        let id = format!("pin{}", (pin * 100.0) as u32);
        group.bench_with_input(BenchmarkId::from_parameter(id), &pin, |b, &pin| {
            b.iter(|| {
                simple_interconnected_islands_game(islands_n, size, pin, N_INTER, 0x1514_4ED5)
                    .unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_islands_scaling, bench_islands_density);
criterion_main!(benches);
