//! Turán graph constructor benchmarks (ALGO-CN-027).
//!
//! Run: `cargo bench --bench bench_turan`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-027.json`.
//!
//! `igraph_turan(n, r)` is a thin wrapper over
//! `igraph_full_multipartite` with maximally balanced partition sizes.
//! The benchmark sweeps three regimes:
//!
//! * **Tiny seed graphs** (octahedron, K_{4,3,3,3}) — measures
//!   per-call overhead (partition-vector allocation + dispatch).
//! * **Dense balanced** (T(96, 3), T(128, 4)) — the path most users
//!   exercise; throughput is bound by inter-partition edge emission.
//! * **Skewed remainder** (T(101, 3), T(200, 7)) — exercises the
//!   `remainder` arm where the first `n%r` partitions get an extra
//!   vertex.
//!
//! Edge counts are pre-computed via the closed-form
//! `E = ½ · Σ n_i · (N − n_i)` so throughput reports remain comparable
//! across shapes.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::turan;

struct Shape {
    label: &'static str,
    n: u32,
    r: u32,
    /// Closed-form edge count, used for throughput reporting.
    edges: u64,
}

const SHAPES: &[Shape] = &[
    // Tiny: T(6, 3) — the octahedron / K_{2,2,2}. 12 edges.
    Shape {
        label: "t_6_3_octahedron",
        n: 6,
        r: 3,
        edges: 12,
    },
    // Small: T(13, 4) — sizes [4,3,3,3]. 63 edges.
    Shape {
        label: "t_13_4",
        n: 13,
        r: 4,
        edges: 63,
    },
    // Medium-balanced: T(96, 3) — sizes [32,32,32]. E = ½·3·32·64 = 3072.
    Shape {
        label: "t_96_3_balanced",
        n: 96,
        r: 3,
        edges: 3_072,
    },
    // Medium-balanced: T(128, 4) — sizes [32,32,32,32]. E = ½·4·32·96 = 6144.
    Shape {
        label: "t_128_4_balanced",
        n: 128,
        r: 4,
        edges: 6_144,
    },
    // Skewed remainder: T(101, 3) — sizes [34,34,33]. E = ½·(34·67+34·67+33·68) = ½·(4556+2244) = 3400.
    Shape {
        label: "t_101_3_skewed",
        n: 101,
        r: 3,
        edges: 3_400,
    },
    // Larger skewed: T(200, 7) — sizes [29,29,29,29,28,28,28].
    // E = ½·(4·29·171 + 3·28·172) = ½·(19836 + 14448) = 17142.
    Shape {
        label: "t_200_7_skewed",
        n: 200,
        r: 7,
        edges: 17_142,
    },
];

fn bench_turan(c: &mut Criterion) {
    let mut group = c.benchmark_group("turan");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| turan(shape.n, shape.r).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_turan);
criterion_main!(benches);
