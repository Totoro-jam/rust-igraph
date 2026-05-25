//! ALGO-CN-019 example: walk the Mycielski chain `M_2..M_5` and apply the
//! Mycielskian construction to an arbitrary input graph.
//!
//! The Mycielski construction takes a graph `G = (V, E)` and produces a
//! triangle-free larger graph `μ(G)` whose chromatic number is one higher
//! than `G`'s. Iterating from `M_2 = P_2` yields the canonical Mycielski
//! chain `M_2, M_3 = C_5, M_4 = Grötzsch, M_5, …` — each level adds an
//! axiomatic +1 to χ while staying triangle-free.
//!
//! The recurrence on counts is `(v', e') = (2v + 1, 3e + v)`. Mycielski
//! graphs of order ≥3 are exactly what `mycielski_graph(k)` produces; the
//! arbitrary-input form `mycielskian(graph, k)` applies the construction
//! `k` times to any starting graph.
//!
//! Run: `cargo run --example mycielskian_demo`.

use rust_igraph::{Graph, VertexId, cycle_graph, mycielski_graph, mycielskian};
use std::collections::BTreeSet;

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn is_triangle_free(g: &Graph) -> bool {
    let n = g.vcount();
    let nbrs: Vec<BTreeSet<VertexId>> = (0..n)
        .map(|v| g.neighbors(v).expect("nbrs").into_iter().collect())
        .collect();
    for u in 0..n {
        let u_set = &nbrs[u as usize];
        for &v in u_set {
            if v <= u {
                continue;
            }
            for &w in &nbrs[v as usize] {
                if w <= v {
                    continue;
                }
                if u_set.contains(&w) {
                    return false;
                }
            }
        }
    }
    true
}

fn main() {
    // 1) Walk the canonical chain via mycielski_graph(k).
    for k in 2u32..=5 {
        let g = mycielski_graph(k).expect("mycielski_graph");
        print_summary(&format!("M_{k} = mycielski_graph({k})"), &g);
        assert!(is_triangle_free(&g), "M_{k} should be triangle-free");
    }

    // The recurrence: M_3 = C_5, M_4 = Grötzsch (11v/20e), M_5 = 23v/71e.
    let m3 = mycielski_graph(3).expect("M_3");
    assert_eq!((m3.vcount(), m3.ecount()), (5, 5));
    let m4 = mycielski_graph(4).expect("M_4");
    assert_eq!((m4.vcount(), m4.ecount()), (11, 20));
    let m5 = mycielski_graph(5).expect("M_5");
    assert_eq!((m5.vcount(), m5.ecount()), (23, 71));

    // 2) Apply mycielskian to a cycle C_5 directly — that yields the
    //    Grötzsch graph (since C_5 = M_3).
    let c5 = cycle_graph(5, false, false).expect("C_5");
    let grotzsch = mycielskian(&c5, 1).expect("mycielskian(C_5, 1)");
    print_summary("mycielskian(C_5, 1) ≡ Grötzsch", &grotzsch);
    assert_eq!((grotzsch.vcount(), grotzsch.ecount()), (11, 20));
    assert!(is_triangle_free(&grotzsch));

    // 3) Iterate twice on C_5: counts follow (2v+1, 3e+v) twice.
    let two_iters = mycielskian(&c5, 2).expect("mycielskian(C_5, 2)");
    print_summary("mycielskian(C_5, 2)", &two_iters);
    // After 1 iter: (11, 20); after 2 iters: (23, 71).
    assert_eq!((two_iters.vcount(), two_iters.ecount()), (23, 71));

    // 4) Directedness is preserved by the construction.
    let dir = cycle_graph(4, true, true).expect("directed C_4");
    let dir_my = mycielskian(&dir, 1).expect("directed mycielskian");
    print_summary("mycielskian(directed C_4, 1)", &dir_my);
    assert!(dir_my.is_directed(), "directedness preserved");

    // 5) Corner cases: null → singleton → P_2 promotions.
    let null = Graph::new(0, false).expect("null");
    let promo1 = mycielskian(&null, 1).expect("null → singleton");
    print_summary("mycielskian(null, 1) → singleton", &promo1);
    assert_eq!((promo1.vcount(), promo1.ecount()), (1, 0));

    let promo2 = mycielskian(&null, 2).expect("null → P_2");
    print_summary("mycielskian(null, 2) → P_2", &promo2);
    assert_eq!((promo2.vcount(), promo2.ecount()), (2, 1));

    // 6) k=0 identity — input is returned unchanged (modulo a fresh copy).
    let k3 = {
        let mut g = Graph::new(3, false).expect("K_3");
        g.add_edge(0, 1).expect("e1");
        g.add_edge(1, 2).expect("e2");
        g.add_edge(0, 2).expect("e3");
        g
    };
    let identity = mycielskian(&k3, 0).expect("k=0 identity");
    print_summary("mycielskian(K_3, 0) ≡ K_3", &identity);
    assert_eq!((identity.vcount(), identity.ecount()), (3, 3));

    println!("\nall mycielskian cases OK ✓");
}
