//! Erdős–Rényi random graph generator benchmarks (ALGO-GN-001).
//!
//! Run: `cargo bench --bench bench_erdos_renyi`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-001.json`.
//!
//! Coverage:
//!   - G(n,p) sparse — p tuned so that expected `m ≈ 2n` (constant
//!     average degree 4). Exercises the geometric-skip fast path
//!     (`gen_geom` dominates when `n_pairs` · p is moderate).
//!   - G(n,p) dense — p = 0.5. Exercises the Bernoulli-trial path
//!     where the geometric skip is small (~2 on average) and most of
//!     the cost is the per-edge push.
//!   - G(n,m) — Floyd's distinct sampling at `m = 2n`. Exercises the
//!     HashSet+linear-pass distinct draws.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{erdos_renyi_gnm, erdos_renyi_gnp};

fn bench_gnp_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("erdos_renyi_gnp/sparse_avg_deg_4");
    for n in [100u32, 1_000, 10_000] {
        // p picked so E[m] = p · n·(n-1)/2 ≈ 2n  ⇒  p ≈ 4/(n-1).
        let p = 4.0 / f64::from(n - 1);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &(n, p), |b, &(n, p)| {
            b.iter(|| erdos_renyi_gnp(n, p, false, false, 0xCAFE_F00D).unwrap());
        });
    }
    group.finish();
}

fn bench_gnp_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("erdos_renyi_gnp/dense_p_0_5");
    for n in [100u32, 500, 1_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| erdos_renyi_gnp(n, 0.5, false, false, 0xDEAD_BEEF).unwrap());
        });
    }
    group.finish();
}

fn bench_gnm_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("erdos_renyi_gnm/m_eq_2n");
    for n in [100u32, 1_000, 10_000] {
        let m = u64::from(n) * 2;
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(BenchmarkId::from_parameter(n), &(n, m), |b, &(n, m)| {
            b.iter(|| erdos_renyi_gnm(n, m, false, false, 0x1234_5678).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_gnp_sparse, bench_gnp_dense, bench_gnm_sparse);
criterion_main!(benches);
