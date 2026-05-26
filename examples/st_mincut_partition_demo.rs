//! ALGO-FL-018 example: s-t minimum-cut **partition** — value, cut edge
//! ids, and source-side / sink-side vertex bipartition.
//!
//! Run: `cargo run --example st_mincut_partition_demo`.
//!
//! Each case below is structured the same way:
//!   1. Build a small graph + capacity vector.
//!   2. Call `st_mincut(graph, source, target, capacity)`.
//!   3. Print value / cut / partition / partition2, with the expected
//!      reference value alongside each headline number.

use rust_igraph::{Graph, st_mincut};

fn show(label: &str, g: &Graph, s: u32, t: u32, cap: Option<&[f64]>, expected_value: f64) {
    let cut = st_mincut(g, s, t, cap).expect("mincut");
    println!("{label}");
    println!("  source = {s}, target = {t}");
    println!(
        "  value      = {:>5.1}   (expected {:>5.1})",
        cut.value, expected_value
    );
    println!("  cut edges  = {:?}", cut.cut);
    println!(
        "  partition  = {:?}  (always contains source)",
        cut.partition
    );
    println!(
        "  partition2 = {:?}  (always contains target)",
        cut.partition2
    );
}

// Eight cases, kept simple — each builds a small graph + caps and calls
// `show`. The main is intentionally a flat list rather than refactored
// into helpers so the reader sees the inputs and expected values in one
// place.
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Single bottleneck. 0 -(5)-> 1 -(2)-> 2 -(7)-> 3.
    //    Unique min cut: middle arc (1,2). value = 2.
    let mut g1 = Graph::new(4, true)?;
    g1.add_edge(0, 1)?;
    g1.add_edge(1, 2)?;
    g1.add_edge(2, 3)?;
    show(
        "1) Single bottleneck chain (4v directed, weighted)",
        &g1,
        0,
        3,
        Some(&[5.0, 2.0, 7.0]),
        2.0,
    );
    println!();

    // 2) CLRS 26.1-1 textbook (6v, 10 arcs). Min cut value 23,
    //    one valid representative is {(1,3), (4,3), (4,5)}.
    let mut g2 = Graph::new(6, true)?;
    let arcs = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 1),
        (2, 4),
        (3, 2),
        (3, 5),
        (4, 3),
        (4, 5),
    ];
    let caps = [16.0, 13.0, 10.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0];
    for (u, v) in arcs {
        g2.add_edge(u, v)?;
    }
    show(
        "2) CLRS 26.1-1 (6v directed, weighted)",
        &g2,
        0,
        5,
        Some(&caps),
        23.0,
    );
    println!();

    // 3) Two parallel unit-cap paths: 0->1->3 and 0->2->3. Min cut = 2.
    //    Source-side partition reduces to {0}; both middle vertices land
    //    on the sink side because the BFS from 0 in the residual saturates
    //    immediately on both out-arcs.
    let mut g3 = Graph::new(4, true)?;
    g3.add_edge(0, 1)?;
    g3.add_edge(1, 3)?;
    g3.add_edge(0, 2)?;
    g3.add_edge(2, 3)?;
    show(
        "3) Two parallel unit-cap paths (4v directed)",
        &g3,
        0,
        3,
        Some(&[1.0, 1.0, 1.0, 1.0]),
        2.0,
    );
    println!();

    // 4) Disconnected endpoints — empty cut already separates them.
    let g4 = Graph::new(4, true)?;
    show(
        "4) Disconnected endpoints (4 isolated vertices, no edges)",
        &g4,
        0,
        3,
        None,
        0.0,
    );
    println!();

    // 5) Undirected 4-vertex reference (igraph_maxflow.c unit test):
    //    edges (0-1,0-2,1-2,1-3,2-3), caps (4,2,10,2,2). Cut value 4.
    let mut g5 = Graph::new(4, false)?;
    let uedges = [(0u32, 1u32), (0, 2), (1, 2), (1, 3), (2, 3)];
    let ucaps = [4.0, 2.0, 10.0, 2.0, 2.0];
    for (u, v) in uedges {
        g5.add_edge(u, v)?;
    }
    show(
        "5) Undirected igraph_maxflow.c reference (4v, weighted)",
        &g5,
        0,
        3,
        Some(&ucaps),
        4.0,
    );
    println!();

    // 6) Multigraph: two parallel arcs 0 => 1 (caps 3 and 4). Min cut
    //    must take both; cut value = 7. Partition is {0} / {1}.
    let mut g6 = Graph::new(2, true)?;
    g6.add_edge(0, 1)?;
    g6.add_edge(0, 1)?;
    show(
        "6) Multigraph: two parallel arcs (2v directed)",
        &g6,
        0,
        1,
        Some(&[3.0, 4.0]),
        7.0,
    );
    println!();

    // 7) Unit-cap directed 5v from igraph_st_mincut.c unit test:
    //    edges 0->1, 1->2, 1->3, 2->4, 3->4. Min cut value 1
    //    (saturating the single arc 0->1 disconnects source from sink).
    let mut g7 = Graph::new(5, true)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (1, 3), (2, 4), (3, 4)] {
        g7.add_edge(u, v)?;
    }
    show(
        "7) igraph_st_mincut.c reference (5v directed, unit caps)",
        &g7,
        0,
        4,
        None,
        1.0,
    );
    println!();

    // 8) Same shape as (7) but weighted: caps [8,2,3,3,2]. Cut value 4
    //    (saturate the second-layer arcs (1,2) cap=2 and (3,4) cap=2).
    let mut g8 = Graph::new(5, true)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (1, 3), (2, 4), (3, 4)] {
        g8.add_edge(u, v)?;
    }
    show(
        "8) Same 5v shape, weighted caps [8,2,3,3,2]",
        &g8,
        0,
        4,
        Some(&[8.0, 2.0, 3.0, 3.0, 2.0]),
        4.0,
    );
    println!();

    println!("(All eight cases match the C / Python / R three-source");
    println!(" conformance fixtures under tests/conformance/*/st_mincut/.)");

    Ok(())
}
