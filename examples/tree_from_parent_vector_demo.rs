//! ALGO-CN-017 example: decode a parent vector into a rooted tree or
//! forest (`igraph_tree_from_parent_vector`).
//!
//! A parent vector of length `n` represents a rooted forest on `n`
//! vertices: `parents[v]` is `v`'s parent id, or any negative value to
//! mark `v` as a root. Three mode flavours decide arc orientation:
//!
//! * **`TreeMode::Out`** — directed, parent → child arcs.
//! * **`TreeMode::In`** — directed, child → parent arcs.
//! * **`TreeMode::Undirected`** — undirected `(child, parent)` stored
//!   canonically as `(min, max)`.
//!
//! This demo walks every interesting shape:
//!
//! * **single-root chain** in all three modes — shows arc direction.
//! * **two-root forest** — confirms multiple `-1` entries yield two
//!   disconnected paths.
//! * **upstream-C fixture `[4, 4, 1, -2, 3]`** in OUT/IN/Undirected,
//!   exactly as `references/igraph/tests/unit/igraph_tree_from_parent_vector.out`.
//! * **star** centred on vertex 0 in undirected mode — checks the
//!   degree histogram.
//! * **edgeless graph** (all roots) — every vertex isolated, ecount 0.
//! * **rigraph fixture** (after the internal `-1` shift) — the chain
//!   that `tree_from_parent_vector_impl(parents=c(-1,1,2,3))` prints as
//!   `1->2 2->3 3->4` in 1-based labelling.
//!
//! Run: `cargo run --example tree_from_parent_vector_demo`.

use rust_igraph::{Graph, TreeMode, tree_from_parent_vector};
use std::collections::{BTreeMap, BTreeSet};

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn dump_edges_ordered(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    (0..m).map(|e| g.edge(e).expect("edge in range")).collect()
}

fn dump_edges_canonical(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    let mut out: Vec<(u32, u32)> = (0..m)
        .map(|e| g.edge(e).expect("edge"))
        .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
        .collect();
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

fn assert_forest_invariants(label: &str, g: &Graph) {
    let vcount = g.vcount();
    let mut parent: Vec<u32> = (0..vcount).collect();
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for eid in 0..m {
        let (a, b) = g.edge(eid).expect("edge in range");
        assert_ne!(a, b, "{label}: tree must have no self-loops");
        let canon = if a <= b { (a, b) } else { (b, a) };
        if !g.is_directed() {
            assert!(seen.insert(canon), "{label}: duplicate edge {canon:?}");
        }
        let ra = uf_find(&mut parent, a);
        let rb = uf_find(&mut parent, b);
        assert_ne!(ra, rb, "{label}: cycle detected — not a forest");
        parent[ra as usize] = rb;
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

#[allow(clippy::similar_names)]
fn main() {
    // 1) Single-root chain in all three modes.
    let chain_parents: &[i64] = &[-1, 0, 1, 2, 3];
    let g_chain_out = tree_from_parent_vector(chain_parents, TreeMode::Out).expect("chain OUT");
    print_summary("chain OUT — parents=[-1,0,1,2,3]", &g_chain_out);
    assert_forest_invariants("chain OUT", &g_chain_out);
    assert_eq!(
        dump_edges_ordered(&g_chain_out),
        vec![(0, 1), (1, 2), (2, 3), (3, 4)]
    );

    let g_chain_in = tree_from_parent_vector(chain_parents, TreeMode::In).expect("chain IN");
    print_summary("chain IN  — parents=[-1,0,1,2,3]", &g_chain_in);
    assert_forest_invariants("chain IN", &g_chain_in);
    assert_eq!(
        dump_edges_ordered(&g_chain_in),
        vec![(1, 0), (2, 1), (3, 2), (4, 3)]
    );

    let g_chain_un =
        tree_from_parent_vector(chain_parents, TreeMode::Undirected).expect("chain UNDIRECTED");
    print_summary("chain UN  — parents=[-1,0,1,2,3]", &g_chain_un);
    assert_forest_invariants("chain UN", &g_chain_un);
    assert_eq!(
        dump_edges_canonical(&g_chain_un),
        vec![(0, 1), (1, 2), (2, 3), (3, 4)]
    );

    // 2) Two-root forest.
    let g_forest =
        tree_from_parent_vector(&[-1, 0, -1, 2, 3], TreeMode::Out).expect("two-root forest");
    print_summary("two-root forest — parents=[-1,0,-1,2,3], OUT", &g_forest);
    assert_forest_invariants("two-root forest", &g_forest);
    assert_eq!(g_forest.ecount(), 3);

    // 3) Upstream-C fixture in all three modes.
    let upstream: &[i64] = &[4, 4, 1, -2, 3];
    let g_up_out = tree_from_parent_vector(upstream, TreeMode::Out).expect("upstream OUT");
    print_summary("upstream OUT — parents=[4,4,1,-2,3]", &g_up_out);
    assert_eq!(
        dump_edges_ordered(&g_up_out),
        vec![(4, 0), (3, 4), (4, 1), (1, 2)]
    );
    assert_forest_invariants("upstream OUT", &g_up_out);

    let g_up_in = tree_from_parent_vector(upstream, TreeMode::In).expect("upstream IN");
    print_summary("upstream IN  — parents=[4,4,1,-2,3]", &g_up_in);
    assert_eq!(
        dump_edges_ordered(&g_up_in),
        vec![(0, 4), (4, 3), (1, 4), (2, 1)]
    );
    assert_forest_invariants("upstream IN", &g_up_in);

    let g_up_un = tree_from_parent_vector(upstream, TreeMode::Undirected).expect("upstream UN");
    print_summary("upstream UN  — parents=[4,4,1,-2,3]", &g_up_un);
    assert_eq!(
        dump_edges_canonical(&g_up_un),
        vec![(0, 4), (1, 2), (1, 4), (3, 4)]
    );
    assert_forest_invariants("upstream UN", &g_up_un);

    // 4) Star centred on 0 (undirected) — check the degree histogram.
    let g_star = tree_from_parent_vector(&[-1, 0, 0, 0, 0], TreeMode::Undirected).expect("star");
    print_summary("star centre=0 — parents=[-1,0,0,0,0], UN", &g_star);
    assert_forest_invariants("star", &g_star);
    let hist = degree_histogram(&g_star);
    assert_eq!(hist.get(&4), Some(&1), "exactly one vertex of degree 4");
    assert_eq!(hist.get(&1), Some(&4), "exactly four leaves");

    // 5) Edgeless graph — all roots.
    let g_edgeless =
        tree_from_parent_vector(&[-1, -1, -1, -1, -1], TreeMode::Out).expect("edgeless");
    print_summary("edgeless — parents=[-1,-1,-1,-1,-1], OUT", &g_edgeless);
    assert_eq!(g_edgeless.ecount(), 0);

    // 6) rigraph fixture (after the internal -1 shift). The R wrapper
    //    converts user-supplied `c(-1, 1, 2, 3)` to C-level
    //    `[-2, 0, 1, 2]` — a chain 0 → 1 → 2 → 3 that rigraph prints in
    //    1-based form as `1->2 2->3 3->4`.
    let g_r = tree_from_parent_vector(&[-2, 0, 1, 2], TreeMode::Out).expect("rigraph snapshot");
    print_summary("rigraph snapshot — c-level parents=[-2,0,1,2], OUT", &g_r);
    assert_eq!(
        dump_edges_ordered(&g_r),
        vec![(0, 1), (1, 2), (2, 3)],
        "matches rigraph snapshot 1->2 2->3 3->4 in 0-based form"
    );
    assert_forest_invariants("rigraph", &g_r);

    println!("\nall tree_from_parent_vector cases OK ✓");
}
