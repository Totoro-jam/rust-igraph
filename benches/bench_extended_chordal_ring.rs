//! Extended chordal-ring constructor benchmarks (ALGO-CN-028).
//!
//! Run: `cargo bench --bench bench_extended_chordal_ring`. Results land
//! under `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-028.json`.
//!
//! `igraph_extended_chordal_ring(nodes, W, directed)` builds the cycle
//! `C_n` and then, for every vertex `i` and every row `r` of `W`, adds
//! the chord `(i, (i + W[r, i mod period]) mod nodes)`. The work is
//! `O(n · m)` with `m = nrow(W)`. The benchmark sweeps three regimes:
//!
//! * **Tiny seed graphs** — the pentagram (n=5, W=[[2]]) and the
//!   12-vertex multigraph (n=12, W=[[4,2],[8,10]]) from
//!   `tests/unit/igraph_extended_chordal_ring.c`. Measures per-call
//!   overhead (matrix-shape validation + cycle emission).
//! * **Single-row chord, large n** (n=4096, W=[[1024]]) — the path that
//!   dominates in practice; throughput is bound by the chord emission
//!   loop (one chord per vertex).
//! * **Multi-row chord, medium n** (n=2048, period=4, 3-row W) —
//!   exercises the multi-row arm where every vertex emits multiple
//!   chord edges per call.
//!
//! Edge counts are pre-computed as `n + n · nrow(W)` so throughput
//! reports remain comparable across shapes.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::extended_chordal_ring;

struct Shape {
    label: &'static str,
    nodes: u32,
    w: &'static [&'static [i64]],
    directed: bool,
    /// Pre-computed edge count for throughput reporting.
    edges: u64,
}

const SHAPES: &[Shape] = &[
    // Tiny: pentagram (n=5, W=[[2]], directed). 5 cycle + 5 chord = 10 edges.
    Shape {
        label: "n5_pentagram_directed",
        nodes: 5,
        w: &[&[2]],
        directed: true,
        edges: 10,
    },
    // Tiny: 12-vertex article-figure multigraph. 12 cycle + 24 chord = 36 edges.
    Shape {
        label: "n12_article_multigraph",
        nodes: 12,
        w: &[&[4, 2], &[8, 10]],
        directed: false,
        edges: 36,
    },
    // Medium: balanced (n=256, period=1) — most-common shape.
    Shape {
        label: "n256_p1_offset_2",
        nodes: 256,
        w: &[&[2]],
        directed: false,
        edges: 512,
    },
    // Medium-multi-row: 3 rows, period 4 (n=512).
    Shape {
        label: "n512_p4_3rows",
        nodes: 512,
        w: &[&[2, 3, 4, 5], &[8, 9, 10, 11], &[64, 65, 66, 67]],
        directed: false,
        edges: 2_048, // 512 + 3 * 512
    },
    // Large: dominant single-row regime (n=4096).
    Shape {
        label: "n4096_p1_offset_1024",
        nodes: 4096,
        w: &[&[1024]],
        directed: false,
        edges: 8_192,
    },
    // Large multi-row: stress chord loop (n=2048, 3 rows, period 4).
    Shape {
        label: "n2048_p4_3rows",
        nodes: 2048,
        w: &[&[2, 3, 4, 5], &[8, 9, 10, 11], &[256, 257, 258, 259]],
        directed: false,
        edges: 8_192, // 2048 + 3 * 2048
    },
];

fn bench_extended_chordal_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("extended_chordal_ring");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| extended_chordal_ring(shape.nodes, shape.w, shape.directed).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_extended_chordal_ring);
criterion_main!(benches);
