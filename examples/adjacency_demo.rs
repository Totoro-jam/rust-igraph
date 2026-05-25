//! ALGO-CN-029 example: dense integer adjacency-matrix constructor
//! (`igraph_adjacency`).
//!
//! Given an `n × n` integer matrix `A` whose entries are non-negative
//! edge multiplicities, `adjacency` builds a [`Graph`] whose shape is
//! determined by:
//!
//! * an [`AdjacencyMode`] — one of `Directed`, `Undirected`, `Max`,
//!   `Min`, `Plus`, `Upper`, `Lower`;
//! * a [`LoopsMode`] — one of `NoLoops`, `Once`, `Twice`.
//!
//! This demo walks the three small matrices used by the upstream C
//! unit test (`tests/unit/igraph_adjacency.c`) to show how the same
//! input matrix produces very different graphs depending on the
//! `(mode, loops)` pair.
//!
//! Run: `cargo run --example adjacency_demo`.
//!
//! [`Graph`]: rust_igraph::Graph
//! [`AdjacencyMode`]: rust_igraph::AdjacencyMode
//! [`LoopsMode`]: rust_igraph::LoopsMode

use rust_igraph::{AdjacencyMode, Graph, LoopsMode, adjacency};
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
    // Upstream C-test matrices (tests/unit/igraph_adjacency.c L75-89).
    let m3: &[&[i64]] = &[&[4, 2, 0], &[3, 0, 4], &[0, 5, 6]];
    let m3_sym: &[&[i64]] = &[&[4, 2, 0], &[2, 0, 4], &[0, 4, 6]];
    let m3_min: &[&[i64]] = &[&[4, 2, 0], &[3, 0, 5], &[0, 4, 6]];

    // DIRECTED, NoLoops: the diagonal is dropped, every off-diagonal
    // entry becomes that many parallel arcs i → j. 14 edges.
    let g = adjacency(m3, AdjacencyMode::Directed, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 directed no_loops", &g);
    assert_eq!(g.ecount(), 14);
    assert!(g.is_directed());

    // DIRECTED, Once: keep diagonal as self-loop counts (+4 + +6 = 10
    // self-loops). 24 edges total.
    let g = adjacency(m3, AdjacencyMode::Directed, LoopsMode::Once).expect("ok");
    print_summary("M3 directed loops_once", &g);
    assert_eq!(g.ecount(), 24);

    // UNDIRECTED requires symmetry; M3_SYM passes. With Twice, the
    // diagonal counts are halved: A(0,0)=4 → 2 loops, A(2,2)=6 → 3.
    let g = adjacency(m3_sym, AdjacencyMode::Undirected, LoopsMode::Twice).expect("ok");
    print_summary("M3_SYM undirected loops_twice", &g);
    assert!(!g.is_directed());

    // MAX: every off-diagonal pair contributes max(A(i,j), A(j,i)).
    let g = adjacency(m3, AdjacencyMode::Max, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 max no_loops", &g);
    assert_eq!(g.ecount(), 8);
    let ms = canonical_multiset(&g);
    assert_eq!(ms.get(&(0, 1)), Some(&3), "max(2, 3) = 3 on the 0-1 edge");

    // MIN: every off-diagonal pair contributes min(A(i,j), A(j,i)). On
    // M3_MIN we get 2 + 0 + 4 = 6 edges; the {0,2} pair has min(0,0)=0.
    let g = adjacency(m3_min, AdjacencyMode::Min, LoopsMode::NoLoops).expect("ok");
    print_summary("M3_MIN min no_loops", &g);
    assert_eq!(g.ecount(), 6);

    // PLUS: every off-diagonal pair contributes A(i,j) + A(j,i). On
    // M3 this is (2+3) + (0+0) + (4+5) = 14 edges.
    let g = adjacency(m3, AdjacencyMode::Plus, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 plus no_loops", &g);
    assert_eq!(g.ecount(), 14);

    // UPPER, Twice: the strict upper triangle drives edges, plus the
    // diagonal as self-loops. UPPER collapses Twice → Once (upstream
    // line 168), so the diagonal counts pass through unhalved.
    let g = adjacency(m3, AdjacencyMode::Upper, LoopsMode::Twice).expect("ok");
    print_summary("M3 upper loops_twice (collapses to once)", &g);
    assert_eq!(g.ecount(), 16);

    // LOWER, NoLoops: only entries with i > j contribute; diagonal
    // ignored. 3 + 5 + 0 = 8 edges.
    let g = adjacency(m3, AdjacencyMode::Lower, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 lower no_loops", &g);
    assert_eq!(g.ecount(), 8);

    // Empty 0 × 0 matrix → empty graph.
    let empty: &[&[i64]] = &[];
    let g = adjacency(empty, AdjacencyMode::Directed, LoopsMode::NoLoops).expect("ok");
    print_summary("empty 0x0 directed", &g);
    assert_eq!(g.vcount(), 0);
    assert_eq!(g.ecount(), 0);

    println!("\nall mode/loops dispatches behave as upstream igraph");
}
