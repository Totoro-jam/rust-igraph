//! ALGO-FL-014 example: maximum number of pairwise internally
//! vertex-disjoint paths from `source` to `target` (Menger 1927).
//! Implementation is a thin wrapper that calls
//! [`st_vertex_connectivity`](rust_igraph::st_vertex_connectivity)
//! under `VconnNei::Ignore` and adds back the count of direct
//! `source → target` edges — every parallel arc contributes one
//! trivially internally-disjoint path of length 1.
//!
//! Run: `cargo run --example vertex_disjoint_paths_demo`.

use rust_igraph::{Graph, vertex_disjoint_paths};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Two parallel directed paths 0→1→3 and 0→2→3 — each interior
    //    vertex lies on exactly one path, no overlap → 2 disjoint paths.
    let mut bot = Graph::new(4, true)?;
    for (u, v) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
        bot.add_edge(u, v)?;
    }
    println!("Two parallel directed paths 0→1→3 and 0→2→3");
    println!(
        "  vertex_disjoint_paths(0 → 3) = {}  (expected 2)",
        vertex_disjoint_paths(&bot, 0, 3)?
    );

    // 2) Direct edge plus one interior detour. `vdp = vc(Ignore) + 1`.
    let mut mix = Graph::new(3, true)?;
    for (u, v) in [(0u32, 1u32), (0, 2), (2, 1)] {
        mix.add_edge(u, v)?;
    }
    println!("\n0→1 direct plus 0→2→1 detour");
    println!(
        "  vertex_disjoint_paths(0 → 1) = {}  (expected 2)",
        vertex_disjoint_paths(&mix, 0, 1)?
    );

    // 3) Four parallel arcs 0→1 — every parallel arc is its own trivial
    //    disjoint path of length 1 → 4.
    let mut parallels = Graph::new(2, true)?;
    for _ in 0..4 {
        parallels.add_edge(0, 1)?;
    }
    println!("\n4 parallel arcs 0→1");
    println!(
        "  vertex_disjoint_paths(0 → 1) = {}  (expected 4)",
        vertex_disjoint_paths(&parallels, 0, 1)?
    );

    // 4) Undirected K_5: every other vertex lies on a disjoint path
    //    (3 of them) plus the direct edge → 4.
    let mut k5 = Graph::new(5, false)?;
    for i in 0u32..5 {
        for j in (i + 1)..5 {
            k5.add_edge(i, j)?;
        }
    }
    println!("\nK_5 undirected");
    println!(
        "  vertex_disjoint_paths(0 → 1) = {}  (expected 4)",
        vertex_disjoint_paths(&k5, 0, 1)?
    );

    // 5) Disconnected endpoints — no path at all → 0.
    let isolated = Graph::new(4, true)?;
    println!("\nDisconnected endpoints (4 isolated vertices)");
    println!(
        "  vertex_disjoint_paths(0 → 3) = {}  (expected 0)",
        vertex_disjoint_paths(&isolated, 0, 3)?
    );

    Ok(())
}
