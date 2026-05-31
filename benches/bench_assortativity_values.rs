//! Value-based assortativity benchmark. ALGO-PR-067.
//!
//! Run: `cargo bench --bench bench_assortativity_values`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-067.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, assortativity, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn ring(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("ring edge");
    }
    g
}

fn degree_values(g: &Graph) -> Vec<f64> {
    #[allow(clippy::cast_precision_loss)]
    (0..g.vcount())
        .map(|v| g.degree(v).expect("degree") as f64)
        .collect()
}

fn bench_karate_normalized(c: &mut Criterion) {
    let g = karate();
    let values = degree_values(&g);
    c.bench_function("assortativity_values/karate normalized (34v 78e)", |b| {
        b.iter(|| assortativity(&g, &values, None, None, false, true).expect("assortativity"));
    });
}

fn bench_ring_1000(c: &mut Criterion) {
    // Pure edge scan: cost is O(E), independent of structure. The 1000-ring
    // exercises the linear pass over the edge list.
    let g = ring(1000);
    let values: Vec<f64> = (0..g.vcount()).map(f64::from).collect();
    c.bench_function("assortativity_values/ring-1000 normalized", |b| {
        b.iter(|| assortativity(&g, &values, None, None, false, true).expect("assortativity"));
    });
}

criterion_group!(benches, bench_karate_normalized, bench_ring_1000);
criterion_main!(benches);
