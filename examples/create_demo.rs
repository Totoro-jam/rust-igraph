//! ALGO-CN-022 example: build small graphs from edge lists with
//! `create()`, the Rust analogue of `igraph_create()`.
//!
//! `create(edges, n, directed)` is the universal hand-rolled
//! constructor: every other generator-like API in the project
//! ultimately reduces to it. Slice-of-pairs `&[(u32, u32)]` replaces
//! the upstream flat `igraph_vector_int_t` of length `2·|E|`, which
//! statically rules out the two C-side errors (`IGRAPH_EINVAL` for
//! odd-length, `IGRAPH_EINVVID` for negative IDs).
//!
//! Run: `cargo run --example create_demo`.

use rust_igraph::{Graph, create};

fn show(label: &str, g: &Graph) {
    let edges: Vec<(u32, u32)> = (0..u32::try_from(g.ecount()).expect("ecount fits u32"))
        .map(|eid| g.edge(eid).expect("edge"))
        .collect();
    println!(
        "  {label:<30}  v={vc}  e={ec}  directed={d}  edges={edges:?}",
        vc = g.vcount(),
        ec = g.ecount(),
        d = g.is_directed(),
    );
}

fn main() {
    // 1) The upstream example from references/igraph/examples/simple/igraph_create.c:
    //    edges = [0,1, 1,2, 2,3, 2,2], n=0 yields 4 vertices.
    let upstream = create(&[(0, 1), (1, 2), (2, 3), (2, 2)], 0, false).expect("create");
    show("upstream n=0 infer", &upstream);

    // 2) Same edges with n=10 keeps all 10 vertices (4 isolated).
    let padded = create(&[(0, 1), (1, 2), (2, 3), (2, 2)], 10, false).expect("create");
    show("upstream n=10 padded", &padded);

    // 3) n smaller than the largest endpoint silently extends.
    let auto_extend = create(&[(0, 1), (5, 6)], 3, false).expect("create");
    show("n=3 extends to max+1=7", &auto_extend);

    // 4) Directed: each (u, v) becomes a single arc; arc order is preserved.
    let directed = create(&[(0, 1), (1, 0), (1, 2), (2, 1)], 3, true).expect("create");
    show("directed cycle pair", &directed);
    assert!(directed.is_directed());
    // First arc out of vertex 0 goes to 1.
    let nbrs0: Vec<u32> = directed.neighbors(0).expect("nbrs").clone();
    assert_eq!(nbrs0, vec![1]);

    // 5) Self-loops and parallels survive (no canonicalisation).
    let multigraph = create(&[(0, 0), (1, 1), (0, 1), (0, 1)], 0, false).expect("create");
    show("multigraph", &multigraph);
    assert_eq!(multigraph.ecount(), 4);
    assert_eq!(multigraph.degree(0).expect("deg"), 4); // 2 self-loop counted twice + 2 parallels

    // 6) A small star, equivalent to `star_graph(StarMode::Out, 8, 0)`
    //    when you only want the topology and not the typed entry point.
    let n_star: u32 = 8;
    let star_edges: Vec<(u32, u32)> = (1..n_star).map(|i| (0, i)).collect();
    let star = create(&star_edges, n_star, false).expect("create");
    show("K_{1,7} star via create", &star);
    assert_eq!(star.degree(0).expect("hub"), (n_star - 1) as usize);

    // 7) Error path: a u32::MAX endpoint cannot be promoted to a vcount.
    let err = create(&[(0, u32::MAX)], 0, false).unwrap_err();
    println!("\nerror at u32::MAX endpoint: {err}");

    println!("\nall create() invariants OK ✓");
}
