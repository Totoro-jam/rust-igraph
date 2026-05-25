//! ALGO-CN-021 example: walk the Read-Wilson graph atlas and verify
//! cell-boundary invariants.
//!
//! `atlas(number)` is the Rust analogue of `igraph_atlas(number)` (and
//! `Graph.Atlas(number)` in python-igraph / `graph_from_atlas(n)` in
//! rigraph). 1253 simple unlabelled undirected graphs on 0..7 vertices
//! ship as a flat-encoded `&[u32]` table in
//! `src/algorithms/constructors/atlas_edges.rs`, transliterated
//! byte-for-byte from upstream `atlas-edges.h` so the per-index dispatch
//! matches the C, Python and R bindings.
//!
//! Run: `cargo run --example atlas_demo`.

use rust_igraph::{ATLAS_SIZE, Graph, atlas};

/// Per-vertex-count starting offsets in the catalogue (and the
/// one-past-the-end sentinel). Documented in the C source at
/// `references/igraph/src/constructors/atlas.c:50-52`.
const STARTS: [u32; 9] = [0, 1, 2, 4, 8, 19, 53, 209, ATLAS_SIZE];

fn degree_sum(g: &Graph) -> usize {
    (0..g.vcount())
        .map(|v| g.neighbors(v).expect("neighbors").len())
        .sum()
}

fn main() {
    // 1) Spot-check the cell boundaries: index N starts a fresh
    //    vertex-count cell, always with the null graph on n vertices.
    for (n, &start) in STARTS.iter().take(8).enumerate() {
        let g = atlas(start).expect("cell start");
        assert_eq!(g.vcount() as usize, n, "cell-start vcount");
        assert_eq!(g.ecount(), 0, "cell-start ecount");
    }
    println!("cell starts: {ATLAS_SIZE} graphs catalogued across vertex counts 0..=7");

    // 2) The last entry in each non-trivial cell is the complete K_n.
    //    `atlas(1252)` is K_7; the rest follow the pattern starts[n+1]-1.
    for n in 2..=7u32 {
        let last = STARTS[n as usize + 1] - 1;
        let g = atlas(last).expect("cell end");
        assert_eq!(g.vcount(), n, "K_{n} vcount");
        let want_e = (n * (n - 1) / 2) as usize;
        assert_eq!(g.ecount() as usize, want_e, "K_{n} ecount");
        // Every vertex sees every other vertex exactly once.
        for v in 0..g.vcount() {
            assert_eq!(g.neighbors(v).expect("nbrs").len(), (n - 1) as usize);
        }
    }
    println!("K_2..K_7 land at indices 3, 7, 18, 52, 208, 1252 — all confirmed regular");

    // 3) The first six graphs by index:
    println!("\n--- first 6 atlas graphs ---");
    for i in 0..6u32 {
        let g = atlas(i).expect("atlas");
        let edges: Vec<(u32, u32)> = (0..u32::try_from(g.ecount()).expect("ecount u32"))
            .map(|eid| g.edge(eid).expect("edge"))
            .collect();
        let vc = g.vcount();
        let ec = g.ecount();
        println!("  atlas({i:>4})  v={vc:>2}  e={ec:>2}  edges={edges:?}");
    }

    // 4) Walk the full 1253-graph catalogue once and check every invariant
    //    we care about (undirected, well-formed, no self-loops,
    //    no exotic degree counts).
    let mut total_edges: u64 = 0;
    for i in 0..ATLAS_SIZE {
        let g = atlas(i).expect("atlas");
        assert!(g.vcount() <= 7, "atlas({i}) vcount exceeds 7");
        assert!(!g.is_directed(), "atlas({i}) should be undirected");
        assert_eq!(
            degree_sum(&g),
            g.ecount() as usize * 2,
            "atlas({i}) degree-sum != 2|E|"
        );
        for v in 0..g.vcount() {
            for &u in &g.neighbors(v).expect("nbrs") {
                assert_ne!(u, v, "atlas({i}) self-loop");
            }
        }
        total_edges += g.ecount() as u64;
    }
    println!("\nwalked entire catalogue: total edges across all 1253 graphs = {total_edges}");

    // 5) Out-of-range numbers yield a typed error rather than panic.
    let err = atlas(ATLAS_SIZE).unwrap_err();
    println!("\nout-of-range error (number = {ATLAS_SIZE}): {err}");

    println!("\nall atlas() invariants OK ✓");
}
