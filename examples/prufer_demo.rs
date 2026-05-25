//! ALGO-CN-016 example: decode a Prüfer sequence into a labelled tree
//! (`igraph_from_prufer`).
//!
//! A Prüfer sequence of length `n - 2` over `{0, 1, …, n-1}` is in
//! bijection with the set of labelled trees on `n` vertices. This demo
//! walks through every interesting shape:
//!
//! * **empty** → `P_2`, the 2-vertex path graph (single edge `0—1`).
//! * **upstream-C fixture `[2,3,2,3]`** (`n = 6`) and **`[0,2,4,1,1,0]`**
//!   (`n = 8`), exactly as printed by
//!   `references/igraph/tests/unit/igraph_from_prufer.out`.
//! * **constant sequence `[0,0,0,0]`** → star centred on vertex 0.
//! * **ascending sequence `[1,2,3,4]`** → path `0-1-2-3-4-5`.
//! * **mixed `[3,0,2,1,3]`** → a generic tree, used here to show that
//!   every output is undirected, connected, has `n - 1` edges, and no
//!   self-loops/duplicates.
//!
//! Run: `cargo run --example prufer_demo`.

use rust_igraph::{Graph, from_prufer};
use std::collections::{BTreeMap, BTreeSet};

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn dump_edges_canonical(g: &Graph) -> Vec<(u32, u32)> {
    let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    let mut out = Vec::with_capacity(g.ecount());
    for e in 0..ec {
        let (u, v) = g.edge(e).expect("edge in range");
        out.push(if u <= v { (u, v) } else { (v, u) });
    }
    out.sort_unstable();
    out
}

fn uf_find(parent: &mut [u32], mut node: u32) -> u32 {
    while parent[node as usize] != node {
        let grand = parent[parent[node as usize] as usize];
        parent[node as usize] = grand;
        node = grand;
    }
    node
}

fn assert_tree_invariants(label: &str, g: &Graph) {
    let vcount = g.vcount();
    let ecount = g.ecount();
    assert_eq!(
        ecount,
        (vcount as usize) - 1,
        "{label}: ecount must equal vcount - 1"
    );
    assert!(!g.is_directed(), "{label}: output is always undirected");

    let mut parent: Vec<u32> = (0..vcount).collect();
    let em = u32::try_from(ecount).expect("ecount fits u32 in example");
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for eid in 0..em {
        let (a, b) = g.edge(eid).expect("edge in range");
        assert_ne!(a, b, "{label}: tree must have no self-loops");
        let canon = if a <= b { (a, b) } else { (b, a) };
        assert!(seen.insert(canon), "{label}: duplicate edge {canon:?}");
        let ra = uf_find(&mut parent, a);
        let rb = uf_find(&mut parent, b);
        assert_ne!(ra, rb, "{label}: cycle detected — not a tree");
        parent[ra as usize] = rb;
    }
    let root = uf_find(&mut parent, 0);
    for v in 1..vcount {
        assert_eq!(
            uf_find(&mut parent, v),
            root,
            "{label}: tree must be connected"
        );
    }
}

fn degree_histogram(g: &Graph) -> BTreeMap<usize, u32> {
    let mut hist: BTreeMap<usize, u32> = BTreeMap::new();
    for v in 0..g.vcount() {
        let d = g.neighbors(v).expect("neighbors").len();
        *hist.entry(d).or_insert(0) += 1;
    }
    hist
}

fn main() {
    // 1) Empty Prüfer → P_2 (the 2-vertex path; one edge 0—1).
    let g_empty = from_prufer(&[]).expect("empty");
    print_summary("from_prufer([]) — expect P_2", &g_empty);
    assert_tree_invariants("empty", &g_empty);
    assert_eq!(dump_edges_canonical(&g_empty), vec![(0, 1)]);

    // 2) Upstream C fixture #1 from igraph_from_prufer.out.
    let g1 = from_prufer(&[2, 3, 2, 3]).expect("prufer1");
    print_summary("from_prufer([2,3,2,3]) — upstream-C", &g1);
    assert_tree_invariants("[2,3,2,3]", &g1);
    let expected_g1: Vec<(u32, u32)> = [(0, 2), (1, 3), (2, 4), (2, 3), (3, 5)]
        .into_iter()
        .collect();
    let mut expected_g1 = expected_g1;
    expected_g1.sort_unstable();
    assert_eq!(dump_edges_canonical(&g1), expected_g1);

    // 3) Upstream C fixture #2 from igraph_from_prufer.out.
    let g2 = from_prufer(&[0, 2, 4, 1, 1, 0]).expect("prufer2");
    print_summary("from_prufer([0,2,4,1,1,0]) — upstream-C", &g2);
    assert_tree_invariants("[0,2,4,1,1,0]", &g2);
    let mut expected_g2: Vec<(u32, u32)> = [(0, 3), (2, 5), (2, 4), (1, 4), (1, 6), (0, 1), (0, 7)]
        .into_iter()
        .collect();
    expected_g2.sort_unstable();
    assert_eq!(dump_edges_canonical(&g2), expected_g2);

    // 4) Constant sequence → star centred on the repeated vertex.
    let g_star = from_prufer(&[0, 0, 0, 0]).expect("star");
    print_summary("from_prufer([0,0,0,0]) — expect S_6 (centre 0)", &g_star);
    assert_tree_invariants("constant", &g_star);
    // Centre 0 has degree 5, every other vertex has degree 1.
    let hist = degree_histogram(&g_star);
    assert_eq!(hist.get(&5), Some(&1), "exactly one vertex of degree 5");
    assert_eq!(hist.get(&1), Some(&5), "exactly five leaves");

    // 5) Ascending sequence → straight path 0-1-2-3-4-5.
    let g_path = from_prufer(&[1, 2, 3, 4]).expect("path");
    print_summary("from_prufer([1,2,3,4]) — expect P_6", &g_path);
    assert_tree_invariants("ascending", &g_path);
    let hist = degree_histogram(&g_path);
    assert_eq!(hist.get(&1), Some(&2), "two endpoints of degree 1");
    assert_eq!(hist.get(&2), Some(&4), "four interior vertices of degree 2");

    // 6) Mixed sequence — generic tree, no specific topology asserted.
    let g_mix = from_prufer(&[3, 0, 2, 1, 3]).expect("mixed");
    print_summary("from_prufer([3,0,2,1,3]) — generic tree", &g_mix);
    assert_tree_invariants("mixed", &g_mix);
    println!("  edges    = {:?}", dump_edges_canonical(&g_mix));

    println!("\nall from_prufer cases OK ✓");
}
