//! Read-Wilson graph atlas constructor benchmarks (ALGO-CN-021).
//!
//! Run: `cargo bench --bench bench_atlas`.
//! A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-021.json`.
//!
//! Coverage:
//! - `atlas/by_index`: single lookup at five representative entries
//!   spanning the 1253-graph catalogue (null, `K_2`, `K_4`, `K_6`, `K_7`).
//! - `atlas/walk_catalogue`: materialise every one of the 1253 atlas
//!   graphs back-to-back, which is the worst-case throughput probe for
//!   the position-table dispatch + edge-copy loop.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{ATLAS_SIZE, atlas};

fn bench_atlas_by_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("atlas/by_index");
    // (index, ecount) for throughput; ecount is the dominant cost driver.
    for &(idx, ecount) in &[(0u32, 0u64), (3, 1), (18, 6), (208, 15), (1252, 21)] {
        group.throughput(Throughput::Elements(ecount.max(1)));
        group.bench_with_input(BenchmarkId::from_parameter(idx), &idx, |b, &idx| {
            b.iter(|| atlas(idx).unwrap());
        });
    }
    group.finish();
}

fn bench_atlas_walk_catalogue(c: &mut Criterion) {
    let mut group = c.benchmark_group("atlas/walk_catalogue");
    group.throughput(Throughput::Elements(u64::from(ATLAS_SIZE)));
    group.bench_function("all_1253", |b| {
        b.iter(|| {
            for i in 0..ATLAS_SIZE {
                let _ = atlas(i).unwrap();
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_atlas_by_index, bench_atlas_walk_catalogue);
criterion_main!(benches);
