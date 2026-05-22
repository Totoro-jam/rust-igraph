//! `voronoi` (ALGO-SP-007) — Voronoi cells of Zachary's karate club.
//!
//! Run with `cargo run --release --example voronoi_karate`. Mirrors the
//! reproducible portion of `references/igraph/tests/unit/igraph_voronoi.c`:
//! Zachary's karate club, unweighted, undirected, three generator
//! vertices `[0, 32, 24]` (the two faction leaders + a peripheral
//! member). Prints, for each tiebreaker rule:
//!
//! * the size of each Voronoi cell,
//! * a comma-separated list of the vertices in that cell,
//! * the per-vertex distance to the assigned generator,
//! * the number of unreachable vertices (always 0 here — the graph is
//!   connected).
//!
//! The `FIRST` / `LAST` tiebreakers differ only on equidistant ties;
//! their cell-size totals stay identical because every vertex is
//! reachable from at least one generator at finite distance.

#![allow(clippy::cast_possible_truncation)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{DijkstraMode, VoronoiPartition, VoronoiTiebreaker, read_edgelist, voronoi};

fn report(label: &str, generators: &[u32], partition: &VoronoiPartition) {
    let n = partition.membership.len();
    let mut sizes = vec![0usize; generators.len()];
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); generators.len()];
    let mut unreachable = 0usize;
    for (v, m) in partition.membership.iter().enumerate() {
        match m {
            Some(idx) => {
                let i = *idx as usize;
                sizes[i] += 1;
                buckets[i].push(v as u32);
            }
            None => unreachable += 1,
        }
    }

    println!("\n{label} (generators = {generators:?}):");
    for (i, &g) in generators.iter().enumerate() {
        let members: Vec<String> = buckets[i].iter().map(u32::to_string).collect();
        let dists: Vec<String> = buckets[i]
            .iter()
            .map(|&v| format!("{:.0}", partition.distances[v as usize]))
            .collect();
        println!(
            "  cell {i} (generator {g:>2}): size = {:>2} / {n}",
            sizes[i]
        );
        println!("    vertices : [{}]", members.join(", "));
        println!("    distances: [{}]", dists.join(", "));
    }
    println!("  unreachable: {unreachable}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    println!("Karate club: {} vertices, {} edges", g.vcount(), g.ecount());

    let generators: Vec<u32> = vec![0, 32, 24];

    let first = voronoi(
        &g,
        None,
        DijkstraMode::All,
        &generators,
        VoronoiTiebreaker::First,
        42,
    )?;
    report("FIRST tiebreaker", &generators, &first);

    let last = voronoi(
        &g,
        None,
        DijkstraMode::All,
        &generators,
        VoronoiTiebreaker::Last,
        42,
    )?;
    report("LAST  tiebreaker", &generators, &last);

    // Sanity: FIRST and LAST agree on every non-tied vertex, and the
    // total of cell sizes equals the vertex count under both rules.
    let same: usize = first
        .membership
        .iter()
        .zip(last.membership.iter())
        .filter(|(a, b)| a == b)
        .count();
    println!(
        "\nFIRST vs LAST: {same}/{} vertices agree on cell assignment (remainder were ties).",
        first.membership.len()
    );

    Ok(())
}
