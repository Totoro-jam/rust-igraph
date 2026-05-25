//! ALGO-CN-015 example: line graph `L(G)` (`igraph_linegraph`).
//!
//! Walks through the shapes that exercise every interesting branch of
//! `linegraph`: an empty graph, a path, a triangle, a star, an undirected
//! multi-graph, an undirected graph with a self-loop, and a directed
//! chain. For each case we print the input and output graphs and assert
//! the closed-form `(|V(L)|, |E(L)|)`.
//!
//! Key facts from the upstream specification:
//!
//! * `|V(L)| = |E(G)|`. Each edge of the input becomes a vertex of the
//!   line graph.
//! * Undirected branch: two L-vertices `i, j` are connected iff edges
//!   `e_i, e_j` share an endpoint. Each shared endpoint emits exactly one
//!   L-edge; a self-loop `(u, u)` participates **twice** at `u`
//!   (`IGRAPH_LOOPS == IGRAPH_LOOPS_TWICE`), so two parallel non-loop
//!   edges sharing `u` with the loop yield two L-edges.
//! * Directed branch: there is an arc `(i, j)` in `L(G)` iff `head(e_i)
//!   == tail(e_j)` (i.e. edge `e_j` continues out of where edge `e_i`
//!   came in).
//! * `Graph::add_edges` canonicalises undirected edges to `(min, max)`,
//!   so dumped pairs are compared in canonical form.
//!
//! Run: `cargo run --example linegraph_demo`.

#![allow(clippy::similar_names)]

use rust_igraph::{Graph, StarMode, full_graph, linegraph, path_graph, star_graph};
use std::collections::BTreeSet;

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    let mut out = Vec::with_capacity(g.ecount());
    for e in 0..ec {
        let (u, v) = g.edge(e).expect("edge in range");
        out.push((u, v));
    }
    out
}

fn main() {
    // Empty input: L(K_0) is also empty.
    let g_empty = Graph::new(0, false).expect("empty undirected");
    let l_empty = linegraph(&g_empty).expect("L(empty)");
    print_summary("L(empty undirected)", &l_empty);
    assert_eq!(l_empty.vcount(), 0);
    assert_eq!(l_empty.ecount(), 0);

    // Path P_4 (3 edges) — L(P_4) = P_3 (3 vertices, 2 edges).
    let p4 = path_graph(4, false, false).expect("P_4");
    let l_p4 = linegraph(&p4).expect("L(P_4)");
    print_summary("L(P_4)", &l_p4);
    assert_eq!(l_p4.vcount(), 3);
    assert_eq!(l_p4.ecount(), 2);
    let edges_p4: BTreeSet<(u32, u32)> = dump_edges(&l_p4).into_iter().collect();
    let expected_p4: BTreeSet<(u32, u32)> = [(0, 1), (1, 2)].into_iter().collect();
    assert_eq!(
        edges_p4, expected_p4,
        "L(P_4) edge set must be {{(0,1),(1,2)}}"
    );

    // Triangle K_3 — each pair of edges shares a vertex, so L(K_3) = K_3.
    let k3 = full_graph(3, false, false).expect("K_3");
    let l_k3 = linegraph(&k3).expect("L(K_3)");
    print_summary("L(K_3)", &l_k3);
    assert_eq!(l_k3.vcount(), 3);
    assert_eq!(l_k3.ecount(), 3);

    // Star S_5 (centre + 4 leaves) — every leaf-edge meets every other
    // leaf-edge at the centre, so L(S_5) = K_4.
    let s5 = star_graph(5, StarMode::Undirected, 0).expect("S_5");
    let l_s5 = linegraph(&s5).expect("L(S_5)");
    print_summary("L(S_5) — expect K_4", &l_s5);
    assert_eq!(l_s5.vcount(), 4);
    assert_eq!(l_s5.ecount(), 6); // 4 choose 2.

    // Undirected multi-graph: two parallel edges between 0 and 1. Each
    // shared endpoint (0 and 1) emits one L-edge; the two L-vertices are
    // thus connected by two parallel L-edges.
    let mut multi = Graph::new(2, false).expect("multi undirected");
    multi
        .add_edges(vec![(0, 1), (0, 1)])
        .expect("two parallel edges");
    let l_multi = linegraph(&multi).expect("L(multi)");
    print_summary("L(double-edge between 0 and 1)", &l_multi);
    assert_eq!(l_multi.vcount(), 2);
    assert_eq!(l_multi.ecount(), 2);
    assert_eq!(dump_edges(&l_multi), vec![(0, 1), (0, 1)]);

    // Undirected self-loop (LOOPS-TWICE): edge 0 is the self-loop (0,0),
    // edge 1 is the non-loop (0,1). At vertex 0 the incidence list is
    // [0, 0, 1] (loop pushed twice). Cross-edge pairs at 0:
    //   - the two copies of the loop with the non-loop → two L-edges
    //     between vertices 0 and 1, plus a self-loop on L-vertex 0 from
    //     the loop crossing itself. Total: 3 edges.
    let mut sl = Graph::new(2, false).expect("loop+non-loop");
    sl.add_edges(vec![(0, 0), (0, 1)])
        .expect("loop then non-loop");
    let l_sl = linegraph(&sl).expect("L(self_loop+nonloop)");
    print_summary("L(self-loop + edge)", &l_sl);
    assert_eq!(l_sl.vcount(), 2);
    assert_eq!(l_sl.ecount(), 3);
    // Canonical form: (0,0), (0,1), (0,1).
    assert_eq!(dump_edges(&l_sl), vec![(0, 0), (0, 1), (0, 1)]);

    // Directed chain 0 -> 1 -> 2 -> 3 (edges e0, e1, e2).
    // Edge e_i ends where e_{i+1} starts, so L(chain) = chain on 3 nodes:
    // 0 -> 1, 1 -> 2.
    let mut chain = Graph::new(4, true).expect("directed chain");
    chain
        .add_edges(vec![(0, 1), (1, 2), (2, 3)])
        .expect("chain edges");
    let l_chain = linegraph(&chain).expect("L(directed chain)");
    print_summary("L(directed 0->1->2->3)", &l_chain);
    assert_eq!(l_chain.vcount(), 3);
    assert_eq!(l_chain.ecount(), 2);
    assert!(l_chain.is_directed());
    assert_eq!(dump_edges(&l_chain), vec![(0, 1), (1, 2)]);

    println!("\nall linegraph cases OK ✓");
}
