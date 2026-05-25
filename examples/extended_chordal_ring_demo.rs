//! ALGO-CN-028 example: extended chordal ring
//! (`igraph_extended_chordal_ring`).
//!
//! An *extended chordal ring* is the cycle `C_n` augmented with chord
//! edges defined by an `m × p` integer matrix `W` (where `p` divides
//! `n`). For every vertex `i ∈ [0, n)` and every row `r ∈ [0, m)` of
//! `W`, an edge `(i, (i + W[r, i mod p]) mod n)` is added. Negative
//! offsets are allowed and behave like `n + W` modulo `n`. When `W` is
//! empty (zero rows), the result is the pure cycle.
//!
//! This demo walks the canonical shapes covered by the upstream C unit
//! test (`tests/unit/igraph_extended_chordal_ring.c`) plus a couple of
//! textbook R-help shapes.
//!
//! Run: `cargo run --example extended_chordal_ring_demo`.

use rust_igraph::{Graph, extended_chordal_ring};
use std::collections::BTreeMap;

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn canonical_multiset(g: &Graph) -> BTreeMap<(u32, u32), u32> {
    let mut ms = BTreeMap::new();
    for (u, v) in dump_edges(g) {
        let key = if g.is_directed() || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        *ms.entry(key).or_insert(0) += 1;
    }
    ms
}

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  edges    = {:?}", dump_edges(g));
}

fn main() {
    // Case 1: pentagram (n=5, W=[[+2]], directed) — the 5-cycle plus
    // five chord edges offset by +2. Yields the Petersen-style "star
    // of David" shape on 5 vertices.
    let pentagram = extended_chordal_ring(5, &[&[2]], true).expect("ok");
    print_summary("n=5, W=[[+2]], directed (pentagram)", &pentagram);
    assert_eq!(pentagram.vcount(), 5);
    assert_eq!(pentagram.ecount(), 10);
    assert!(pentagram.is_directed());

    // Case 1b: equivalent shape with a negative offset.
    // (i − 3) ≡ (i + 2) (mod 5), so W=[[-3]] yields the same edges.
    let pentagram_neg = extended_chordal_ring(5, &[&[-3]], true).expect("ok");
    print_summary(
        "n=5, W=[[-3]], directed — equivalent to W=[[+2]]",
        &pentagram_neg,
    );
    assert_eq!(
        canonical_multiset(&pentagram),
        canonical_multiset(&pentagram_neg)
    );

    // Case 2: 12-vertex "article" multigraph (n=12, W=[[4,2],[8,10]],
    // undirected). Period=2 so vertices alternate between rows
    // {4, 8} and {2, 10}; every chord appears twice because (i, j)
    // and (j, i) end up emitted by both endpoints.
    let article = extended_chordal_ring(12, &[&[4, 2], &[8, 10]], false).expect("ok");
    print_summary(
        "n=12, W=[[4,2],[8,10]], undirected (article multigraph)",
        &article,
    );
    assert_eq!(article.vcount(), 12);
    assert_eq!(article.ecount(), 36);
    let ms = canonical_multiset(&article);
    // Backbone edges appear once; chord edges appear twice.
    let backbone_count = (0..12)
        .filter(|&i| {
            let (u, v) = (i, (i + 1) % 12);
            let key = if u <= v { (u, v) } else { (v, u) };
            ms.get(&key).copied().unwrap_or(0) == 1
        })
        .count();
    assert_eq!(
        backbone_count, 12,
        "all 12 cycle edges should have multiplicity 1"
    );

    // R-shape: n=8, W=[[+2]] undirected — the simplest "single-chord"
    // shape from R's `make_chordal_ring(8, matrix(2))` help example.
    let n8 = extended_chordal_ring(8, &[&[2]], false).expect("ok");
    print_summary("n=8, W=[[+2]], undirected (R make_chordal_ring shape)", &n8);
    assert_eq!(n8.vcount(), 8);
    assert_eq!(n8.ecount(), 16);
    assert!(!n8.is_directed());

    // Empty W → pure cycle C_n.
    let cycle10: &[&[i64]] = &[];
    let pure_cycle = extended_chordal_ring(10, cycle10, false).expect("ok");
    print_summary("n=10, W=[] — pure cycle C_10", &pure_cycle);
    assert_eq!(pure_cycle.vcount(), 10);
    assert_eq!(pure_cycle.ecount(), 10);

    println!("\nall structural invariants OK");
}
