//! Isomorphism and canonical labeling demo.
//!
//! Demonstrates VF2 isomorphism checking, BLISS canonical permutation,
//! automorphism counting, and automorphism group computation on several
//! small graphs.
//!
//! Run: `cargo run --example isomorphism_demo`.

use rust_igraph::{
    Graph, StarMode, automorphism_group, canonical_permutation, count_automorphisms, full_graph,
    isomorphic, isomorphic_bliss, isomorphic_vf2, permute_vertices, ring_graph, star_graph,
};

fn main() {
    println!("=== Graph Isomorphism Demo ===\n");

    let c4_a = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, None)
        .expect("graph creation succeeds");
    let c4_b = Graph::from_edges(&[(2, 0), (0, 3), (3, 1), (1, 2)], false, None)
        .expect("graph creation succeeds");

    println!("--- VF2 Isomorphism ---");
    let vf2 = isomorphic_vf2(&c4_a, &c4_b, None, None, None, None).expect("VF2 succeeds");
    println!("C4_a ≅ C4_b (VF2): {}", vf2.iso);
    println!("  mapping: {:?}", vf2.map12);

    println!("\n--- BLISS Isomorphism ---");
    let bliss = isomorphic_bliss(&c4_a, &c4_b, None, None).expect("BLISS succeeds");
    println!("C4_a ≅ C4_b (BLISS): {}", bliss.iso);
    println!("  mapping: {:?}", bliss.map12);

    println!("\n--- Generic isomorphic() dispatcher ---");
    let k4 = full_graph(4, false, false).expect("K4 creation succeeds");
    let c4_ring = ring_graph(4, false, false, true).expect("C4 creation succeeds");
    println!("K4 ≅ K4: {}", isomorphic(&k4, &k4).expect("succeeds"));
    println!("K4 ≅ C4: {}", isomorphic(&k4, &c4_ring).expect("succeeds"));

    println!("\n--- Canonical Permutation ---");
    let perm_a = canonical_permutation(&c4_a, None).expect("succeeds");
    let perm_b = canonical_permutation(&c4_b, None).expect("succeeds");
    let canon_a = permute_vertices(&c4_a, &perm_a).expect("permute succeeds");
    let canon_b = permute_vertices(&c4_b, &perm_b).expect("permute succeeds");
    println!("C4_a canonical perm: {perm_a:?}");
    println!("C4_b canonical perm: {perm_b:?}");
    println!(
        "Canonical forms have same edge count: {} == {}",
        canon_a.ecount(),
        canon_b.ecount()
    );

    println!("\n--- Automorphism Counts ---");
    let graphs: Vec<(&str, Graph)> = vec![
        (
            "K4 (complete)",
            full_graph(4, false, false).expect("succeeds"),
        ),
        (
            "C4 (cycle)",
            ring_graph(4, false, false, true).expect("succeeds"),
        ),
        (
            "P4 (path)",
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, None).expect("succeeds"),
        ),
        (
            "Star S5",
            star_graph(5, StarMode::Undirected, 0).expect("succeeds"),
        ),
        (
            "K5 (complete)",
            full_graph(5, false, false).expect("succeeds"),
        ),
    ];

    for (name, g) in &graphs {
        let aut = count_automorphisms(g, None).expect("count succeeds");
        println!("  |Aut({name})| = {aut}");
    }

    println!("\n--- Automorphism Group Generators ---");
    let c5 = ring_graph(5, false, false, true).expect("C5 succeeds");
    let gens = automorphism_group(&c5, None).expect("group succeeds");
    println!("C5 automorphism generators ({} found):", gens.len());
    for (i, generator) in gens.iter().enumerate() {
        println!("  g{i}: {generator:?}");
    }
    let aut = count_automorphisms(&c5, None).expect("count succeeds");
    println!("  |Aut(C5)| = {aut} (expected 10 = 2*5)");

    println!("\n--- Colour-aware Isomorphism ---");
    let g1 = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).expect("succeeds");
    let g2 = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).expect("succeeds");
    let colors_a = vec![0, 0, 1];
    let colors_b = vec![0, 1, 0];
    let result =
        isomorphic_bliss(&g1, &g2, Some(&colors_a), Some(&colors_b)).expect("colour iso succeeds");
    println!("Triangle with colours [0,0,1] ≅ [0,1,0]: {}", result.iso);
    if result.iso {
        println!("  mapping: {:?}", result.map12);
    }

    println!("\nDone.");
}
