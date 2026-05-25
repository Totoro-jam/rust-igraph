//! ALGO-CN-014 example: full (complete) graph `K_n` (`igraph_full`).
//!
//! Walks through every `(directed, loops)` cell of the truth table and
//! verifies the standard structural invariants for each one. `K_n` is
//! the densest deterministic graph on `n` vertices — every pair (and
//! optionally every self-pair) is connected.
//!
//! | n | directed | loops | result          | vcount | ecount             | note                              |
//! |---|----------|-------|-----------------|--------|--------------------|-----------------------------------|
//! | 0 | false    | false | null graph      | 0      | 0                  | degenerate empty input            |
//! | 1 | false    | false | singleton       | 1      | 0                  | nothing to connect                |
//! | 1 | false    | true  | self-loop only  | 1      | 1                  | single self-loop `(0, 0)`         |
//! | 4 | false    | false | `K_4` undirected| 4      | 6 (`n·(n−1)/2`)    | every undirected pair             |
//! | 4 | false    | true  | `K_4` + loops   | 4      | 10 (`n·(n+1)/2`)   | add every self-loop too           |
//! | 4 | true     | false | directed `K_4`  | 4      | 12 (`n·(n−1)`)     | both `(i, j)` and `(j, i)`        |
//! | 4 | true     | true  | directed + loops| 4      | 16 (`n²`)          | full Cartesian product            |
//!
//! Run: `cargo run --example full_demo`.
//!
//! The example asserts:
//! * vcount and ecount match the closed-form for each cell;
//! * the loops flag exactly toggles the presence of `(v, v)` arcs;
//! * for the undirected non-looped variant, every unordered pair appears
//!   exactly once.

use rust_igraph::{Graph, full_graph};
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

fn assert_self_loop_count(g: &Graph, expected: u32) {
    let mut loops = 0u32;
    for (u, v) in dump_edges(g) {
        if u == v {
            loops += 1;
        }
    }
    assert_eq!(
        loops, expected,
        "self-loop count mismatch (got {loops}, want {expected})"
    );
}

fn main() {
    // K_0 — empty graph, the degenerate input.
    let k0 = full_graph(0, false, false).expect("K_0");
    print_summary("K_0 = full_graph(0, false, false)", &k0);
    assert_eq!(k0.vcount(), 0);
    assert_eq!(k0.ecount(), 0);
    assert!(!k0.is_directed());

    // K_1 no loops — singleton, no edges.
    let k1_noloops = full_graph(1, false, false).expect("K_1 no loops");
    print_summary("K_1 = full_graph(1, false, false)", &k1_noloops);
    assert_eq!(k1_noloops.vcount(), 1);
    assert_eq!(k1_noloops.ecount(), 0);

    // K_1 with loops — single self-loop on vertex 0.
    let k1_loops = full_graph(1, false, true).expect("K_1 with loops");
    print_summary("K_1 (loops) = full_graph(1, false, true)", &k1_loops);
    assert_eq!(k1_loops.vcount(), 1);
    assert_eq!(k1_loops.ecount(), 1);
    assert_eq!(dump_edges(&k1_loops), vec![(0, 0)]);

    // K_4 undirected no loops — 4·3/2 = 6 edges.
    let k4_ud_noloops = full_graph(4, false, false).expect("K_4");
    print_summary("K_4 = full_graph(4, false, false)", &k4_ud_noloops);
    assert_eq!(k4_ud_noloops.vcount(), 4);
    assert_eq!(k4_ud_noloops.ecount(), 6);
    assert!(!k4_ud_noloops.is_directed());
    assert_self_loop_count(&k4_ud_noloops, 0);

    // K_4 undirected with loops — 4·5/2 = 10 edges.
    let k4_ud_loops = full_graph(4, false, true).expect("K_4 + loops");
    print_summary("K_4 (loops) = full_graph(4, false, true)", &k4_ud_loops);
    assert_eq!(k4_ud_loops.vcount(), 4);
    assert_eq!(k4_ud_loops.ecount(), 10);
    assert_self_loop_count(&k4_ud_loops, 4);

    // K_4 directed no loops — 4·3 = 12 arcs.
    let k4_dir_noloops = full_graph(4, true, false).expect("directed K_4");
    print_summary("directed K_4 = full_graph(4, true, false)", &k4_dir_noloops);
    assert_eq!(k4_dir_noloops.vcount(), 4);
    assert_eq!(k4_dir_noloops.ecount(), 12);
    assert!(k4_dir_noloops.is_directed());
    assert_self_loop_count(&k4_dir_noloops, 0);

    // K_4 directed with loops — 4² = 16 arcs.
    let k4_dir_loops = full_graph(4, true, true).expect("directed K_4 + loops");
    print_summary(
        "directed K_4 + loops = full_graph(4, true, true)",
        &k4_dir_loops,
    );
    assert_eq!(k4_dir_loops.vcount(), 4);
    assert_eq!(k4_dir_loops.ecount(), 16);
    assert!(k4_dir_loops.is_directed());
    assert_self_loop_count(&k4_dir_loops, 4);

    // For undirected no-loops, every unordered pair {i, j} with i < j
    // appears exactly once. Verify via set equality on the K_4 case.
    let got: BTreeSet<(u32, u32)> = dump_edges(&k4_ud_noloops).into_iter().collect();
    let mut expected: BTreeSet<(u32, u32)> = BTreeSet::new();
    for i in 0u32..4 {
        for j in (i + 1)..4 {
            expected.insert((i, j));
        }
    }
    assert_eq!(
        got, expected,
        "K_4 (undirected, no loops) must equal the unordered pair set"
    );

    println!("\nall structural invariants OK ✓");
}
