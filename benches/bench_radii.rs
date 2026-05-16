//! Eccentricity / radius / diameter benchmark. ALGO-SP-020.
//!
//! Run: `cargo bench --bench bench_radii`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-SP-020.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, diameter, eccentricity, radius, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn bench_eccentricity_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("eccentricity/karate", |b| {
        b.iter(|| eccentricity(&g).unwrap());
    });
}

fn bench_radius_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("radius/karate", |b| {
        b.iter(|| radius(&g).unwrap());
    });
}

fn bench_diameter_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("diameter/karate", |b| {
        b.iter(|| diameter(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_eccentricity_karate,
    bench_radius_karate,
    bench_diameter_karate
);
criterion_main!(benches);
