//! ALGO-CN-027 example: Turán graph (`igraph_turan`).
//!
//! The *Turán graph* `T(n, r)` is the unique (up to isomorphism)
//! `n`-vertex graph that maximises the number of edges subject to
//! containing no clique of size `r + 1` — Turán's 1941 theorem. It is
//! the complete `r`-partite graph with maximally balanced partition
//! sizes (sizes differing by at most one). Concretely:
//!
//! * `q = n / r`, `s = n % r`
//! * the first `s` partitions have size `q + 1`
//! * the remaining `r − s` partitions have size `q`
//!
//! This demo walks the canonical shapes covered by the upstream C unit
//! test (`tests/unit/igraph_turan.c`) and confirms the structural
//! invariants the rustdoc advertises.
//!
//! Run: `cargo run --example turan_demo`.

use rust_igraph::{Graph, turan};
use std::collections::BTreeSet;

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn canonical_undirected(g: &Graph) -> BTreeSet<(u32, u32)> {
    dump_edges(g)
        .into_iter()
        .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
        .collect()
}

fn print_summary(label: &str, g: &Graph, types: &[u32]) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  types    = {types:?}");
    println!("  edges    = {:?}", dump_edges(g));
}

fn main() {
    // n = 0 — empty graph regardless of r.
    let empty = turan(0, 10).expect("ok");
    print_summary("T(0, 10) — empty", &empty.graph, &empty.types);
    assert_eq!(empty.graph.vcount(), 0);
    assert!(empty.types.is_empty());

    // r = 1 — n isolated vertices, the only graph with chromatic number 1.
    let isolated = turan(10, 1).expect("ok");
    print_summary(
        "T(10, 1) — 10 isolated vertices",
        &isolated.graph,
        &isolated.types,
    );
    assert_eq!(isolated.graph.ecount(), 0);
    assert_eq!(isolated.types, vec![0u32; 10]);

    // r > n — capped to r = n, yielding K_n (singleton partitions).
    let kn = turan(4, 6).expect("ok");
    print_summary("T(4, 6) — capped to K_4", &kn.graph, &kn.types);
    assert_eq!(kn.graph.vcount(), 4);
    assert_eq!(kn.graph.ecount(), 6);
    assert_eq!(kn.types, vec![0, 1, 2, 3]);

    // T(6, 3) — the octahedron / cocktail-party K_{2,2,2}. The densest
    // triangle-free … wait, T(6,3) contains triangles between partitions
    // (e.g. (0,2,4)); it is K_4-free by construction.
    let oct = turan(6, 3).expect("ok");
    print_summary("T(6, 3) — octahedron K_{2,2,2}", &oct.graph, &oct.types);
    assert_eq!(oct.graph.ecount(), 12);
    assert_eq!(oct.types, vec![0, 0, 1, 1, 2, 2]);

    // T(13, 4) — partitions [4, 3, 3, 3]; 63 inter-partition edges.
    let t13 = turan(13, 4).expect("ok");
    print_summary("T(13, 4) — partitions [4,3,3,3]", &t13.graph, &t13.types);
    assert_eq!(t13.graph.ecount(), 63);
    assert_eq!(t13.types, vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]);

    // Structural invariants for T(13, 4):
    //   (a) always undirected
    //   (b) types are partition-major (non-decreasing)
    //   (c) no edge lives inside a single partition
    //   (d) closed-form edge count E = ½ · Σ n_i · (N − n_i)
    assert!(!t13.graph.is_directed());
    for w in t13.types.windows(2) {
        assert!(w[0] <= w[1], "types must be monotone, got {:?}", t13.types);
    }
    for (u, v) in canonical_undirected(&t13.graph) {
        assert_ne!(
            t13.types[u as usize], t13.types[v as usize],
            "edge ({u}, {v}) crosses a partition boundary"
        );
    }
    let n_total: u32 = 13;
    let sizes: [u32; 4] = [4, 3, 3, 3];
    let twice_e: u32 = sizes.iter().map(|&s| s * (n_total - s)).sum();
    assert_eq!(
        u32::try_from(t13.graph.ecount()).expect("ecount fits u32 in example"),
        twice_e / 2
    );

    println!("\nall structural invariants OK ✓");
}
