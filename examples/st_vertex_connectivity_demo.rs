//! ALGO-FL-013 example: minimum number of internal vertices whose
//! removal disconnects `source → target`. Computed via the
//! vertex-splitting reduction (Even §5.5) and a unit-cap max-flow on
//! the split graph; behaviour for direct s↔t edges is controlled by
//! [`VconnNei`](rust_igraph::VconnNei).
//!
//! Run: `cargo run --example st_vertex_connectivity_demo`.

use rust_igraph::{Graph, VconnNei, st_vertex_connectivity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) 6-vertex undirected path 0—1—2—3—4—5. Any internal vertex is a
    //    bottleneck → s-t vertex connectivity = 1.
    let mut path = Graph::new(6, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
        path.add_edge(u, v)?;
    }
    println!("6-vertex undirected path 0—1—2—3—4—5");
    println!(
        "  st_vertex_connectivity(0 → 5, ERROR) = {}  (expected 1)",
        st_vertex_connectivity(&path, 0, 5, VconnNei::Error)?
    );

    // 2) 2v with a direct edge: each VconnNei mode produces a different
    //    answer for the direct-edge case.
    let mut two = Graph::new(2, false)?;
    two.add_edge(0, 1)?;
    println!("\n2v undirected with edge (0,1) — direct edge handling");
    println!(
        "  mode=Negative        → {}  (expected -1)",
        st_vertex_connectivity(&two, 0, 1, VconnNei::Negative)?
    );
    println!(
        "  mode=NumberOfNodes   → {}  (expected 2)",
        st_vertex_connectivity(&two, 0, 1, VconnNei::NumberOfNodes)?
    );
    println!(
        "  mode=Ignore          → {}  (expected 0)",
        st_vertex_connectivity(&two, 0, 1, VconnNei::Ignore)?
    );

    // 3) K_6 undirected, IGNORE mode: every internal vertex must be
    //    removed (4 of them) before vertex 0 and 1 become disconnected.
    let mut k6 = Graph::new(6, false)?;
    for i in 0u32..6 {
        for j in (i + 1)..6 {
            k6.add_edge(i, j)?;
        }
    }
    println!("\nK_6 undirected");
    println!(
        "  st_vertex_connectivity(0 → 1, IGNORE) = {}  (expected 4)",
        st_vertex_connectivity(&k6, 0, 1, VconnNei::Ignore)?
    );

    // 4) Bottleneck: two parallel paths 0→1→3 and 0→2→3 — vertices 1
    //    and 2 each lie on one disjoint path, so vc = 2.
    let mut bot = Graph::new(4, true)?;
    for (u, v) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
        bot.add_edge(u, v)?;
    }
    println!("\nTwo parallel directed paths 0→1→3 and 0→2→3");
    println!(
        "  st_vertex_connectivity(0 → 3, ERROR) = {}  (expected 2)",
        st_vertex_connectivity(&bot, 0, 3, VconnNei::Error)?
    );

    // 5) Disconnected endpoints — no path at all → 0.
    let isolated = Graph::new(4, true)?;
    println!("\nDisconnected endpoints (4 isolated vertices)");
    println!(
        "  st_vertex_connectivity(0 → 3, ERROR) = {}  (expected 0)",
        st_vertex_connectivity(&isolated, 0, 3, VconnNei::Error)?
    );

    Ok(())
}
