//! Line-graph constructor benchmarks (ALGO-CN-015).
//!
//! Run: `cargo bench --bench bench_linegraph`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-015.json`.
//!
//! Coverage spans the realistic shape regimes for `linegraph`:
//!
//! * **path** — sparse, every vertex has degree ≤ 2 so `|E(L)| ≈ |E(G)|`;
//! * **complete `K_n`** — densest case, `|E(L)| = Θ(n^4)` (≈ 12 for `K_4`,
//!   ≈ 600 for `K_8`, ≈ 13230 for `K_16`);
//! * **star** — degree-skewed input where one vertex incidence list
//!   dominates the cost (`|E(L)| = (n−1)(n−2)/2`);
//! * **dense Erdős–Rényi** — random mixed-degree input with a known
//!   expected `|E|` to baseline against;
//! * **directed chain** — long arc chain so the per-source incoming
//!   walk in the directed branch only emits Θ(1) edges per step.
//!
//! Throughput is reported in `|E(G)|` elements per second (i.e. the size
//! of the input edge set), so cross-shape numbers are directly
//! comparable in input-size terms.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, erdos_renyi_gnm, full_graph, linegraph, path_graph, star_graph};

fn bench_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("linegraph/path");
    for n in [64u32, 1_024, 16_384] {
        let g = path_graph(n, false, false).expect("path");
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| linegraph(g).unwrap());
        });
    }
    group.finish();
}

fn bench_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("linegraph/full_kn");
    for n in [8u32, 16, 32, 64] {
        let g = full_graph(n, false, false).expect("K_n");
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| linegraph(g).unwrap());
        });
    }
    group.finish();
}

fn bench_star(c: &mut Criterion) {
    let mut group = c.benchmark_group("linegraph/star");
    for n in [64u32, 512, 2_048] {
        let g = star_graph(n, rust_igraph::StarMode::Undirected, 0).expect("star");
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| linegraph(g).unwrap());
        });
    }
    group.finish();
}

fn bench_erdos_renyi(c: &mut Criterion) {
    let mut group = c.benchmark_group("linegraph/er_gnm");
    // Three (n, m) shapes: sparse, medium, denser.
    for (n, m) in [(512u32, 1_024u64), (1_024, 4_096), (2_048, 8_192)] {
        let g = erdos_renyi_gnm(n, m, false, false, 0xAB_CD_EF_01).expect("ER G(n,m)");
        let label = format!("n{n}_m{m}");
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&label), &g, |b, g| {
            b.iter(|| linegraph(g).unwrap());
        });
    }
    group.finish();
}

fn bench_directed_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("linegraph/directed_chain");
    for n in [1_024u32, 16_384, 131_072] {
        let mut g = Graph::new(n, true).expect("directed");
        // Pre-allocate the edge list so add_edges runs once.
        let edges: Vec<(u32, u32)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        g.add_edges(edges).expect("chain edges");
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| linegraph(g).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_path,
    bench_full,
    bench_star,
    bench_erdos_renyi,
    bench_directed_chain
);
criterion_main!(benches);
