//! ALGO-CN-025 example: full citation graph (`igraph_full_citation`).
//!
//! Walks both `directed` settings across small / medium `n`, asserting
//! every structural invariant the upstream C unit test (and the
//! `igraph_full_citation` rustdoc) advertises:
//!
//! * `n == 0` → empty graph (both directed and undirected variants).
//! * `n == 1` → singleton, no edges.
//! * `directed = true` → complete DAG with arcs `i -> j` for every
//!   `0 ≤ j < i < n`. `ecount == n·(n-1)/2`; arcs are loop-free and
//!   strictly descending (`src > dst`); `in_degree(k) == n - 1 - k`
//!   and `out_degree(k) == k`.
//! * `directed = false` → undirected `K_n` with the same edge multiset
//!   `full_graph(n, false, false)` produces, just in a different emission
//!   order (descending-source-major).
//!
//! Run: `cargo run --example full_citation_demo`.

use rust_igraph::{Graph, full_citation, full_graph};
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

fn in_out_counts(g: &Graph) -> (Vec<usize>, Vec<usize>) {
    let n = g.vcount() as usize;
    let mut in_count = vec![0usize; n];
    let mut out_count = vec![0usize; n];
    for (u, v) in dump_edges(g) {
        out_count[u as usize] += 1;
        in_count[v as usize] += 1;
    }
    (in_count, out_count)
}

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  edges    = {:?}", dump_edges(g));
}

fn main() {
    // Degenerate guards: n=0 → empty, n=1 → singleton.
    let empty = full_citation(0, true).expect("n=0 directed");
    print_summary("n=0 directed — empty", &empty);
    assert_eq!(empty.vcount(), 0);
    assert_eq!(empty.ecount(), 0);

    let singleton = full_citation(1, false).expect("n=1 undirected");
    print_summary("n=1 undirected — singleton", &singleton);
    assert_eq!(singleton.vcount(), 1);
    assert_eq!(singleton.ecount(), 0);

    // Canonical n=4 directed DAG matches the upstream unit-test fixture.
    let dag4 = full_citation(4, true).expect("n=4 directed");
    print_summary("n=4 directed — complete DAG (citation invariant)", &dag4);
    assert_eq!(dag4.vcount(), 4);
    assert_eq!(dag4.ecount(), 6);
    assert!(dag4.is_directed());
    let expected = vec![(1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2)];
    assert_eq!(
        dump_edges(&dag4),
        expected,
        "emission order matches upstream"
    );
    for (u, v) in dump_edges(&dag4) {
        assert!(u > v, "every arc must descend (citation invariant)");
    }

    // In/out degree profile of an n=6 directed DAG.
    let n = 6u32;
    let dag6 = full_citation(n, true).expect("n=6 directed");
    print_summary("n=6 directed — in/out-degree profile", &dag6);
    let (in_d, out_d) = in_out_counts(&dag6);
    for k in 0..n {
        let want_out = k as usize;
        let want_in = (n - 1 - k) as usize;
        assert_eq!(out_d[k as usize], want_out, "out-degree(vertex {k})");
        assert_eq!(in_d[k as usize], want_in, "in-degree(vertex {k})");
    }
    println!("  in-degrees  = {in_d:?}");
    println!("  out-degrees = {out_d:?}");

    // Undirected K_5 — same multiset as full_graph(5, false, false), but
    // a different emission order (full_graph walks ascending-source-major,
    // full_citation walks descending-source-major).
    let k5_cit = full_citation(5, false).expect("n=5 undirected citation");
    let k5_full = full_graph(5, false, false).expect("n=5 undirected full");
    print_summary("n=5 undirected — full_citation (K_5)", &k5_cit);
    print_summary("n=5 undirected — full_graph    (K_5)", &k5_full);
    assert_eq!(k5_cit.ecount(), k5_full.ecount());
    assert_eq!(
        canonical_undirected(&k5_cit),
        canonical_undirected(&k5_full),
        "edge multisets must agree"
    );
    assert_ne!(
        dump_edges(&k5_cit),
        dump_edges(&k5_full),
        "but the emission orders should differ"
    );

    // Closed-form check: ecount = n·(n-1)/2 for both directed settings.
    for n in [0u32, 1, 2, 7, 25] {
        let want = (n as usize) * (n as usize).saturating_sub(1) / 2;
        let dd = full_citation(n, true).expect("ok");
        let du = full_citation(n, false).expect("ok");
        assert_eq!(dd.ecount(), want, "directed n={n}");
        assert_eq!(du.ecount(), want, "undirected n={n}");
    }

    println!("\nall structural invariants OK ✓");
}
