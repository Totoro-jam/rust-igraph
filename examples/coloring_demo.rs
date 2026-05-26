//! ALGO-CL-001 example: greedy vertex coloring demo.
//!
//! Builds several small graphs and colors them with both CN and DSATUR
//! heuristics, printing the number of colors used and validating correctness.
//!
//! Run: `cargo run --example coloring_demo`.

use rust_igraph::{GreedyColoringHeuristic, create, is_vertex_coloring, vertex_coloring_greedy};

fn demo(label: &str, n: u32, edges: &[(u32, u32)]) {
    let g = create(edges, n, false).expect("graph creation succeeds");
    println!("{label} ({n} vertices, {} edges)", g.ecount());

    for (name, heuristic) in [
        ("CN", GreedyColoringHeuristic::ColoredNeighbors),
        ("DSATUR", GreedyColoringHeuristic::DSatur),
    ] {
        let colors = vertex_coloring_greedy(&g, heuristic).expect("coloring succeeds");
        let num_colors = colors.iter().copied().max().map_or(0, |m| m + 1);
        let valid = is_vertex_coloring(&g, &colors).expect("validation succeeds");
        println!("  {name:8} -> {num_colors} colors, valid={valid}");
    }
    println!();
}

fn main() {
    demo("Triangle (K3)", 3, &[(0, 1), (1, 2), (2, 0)]);

    demo(
        "Complete bipartite K3,3",
        6,
        &[
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
        ],
    );

    let mut wheel_edges: Vec<(u32, u32)> = Vec::new();
    for i in 1..=10u32 {
        wheel_edges.push((0, i));
    }
    for i in 1..10u32 {
        wheel_edges.push((i, i + 1));
    }
    wheel_edges.push((10, 1));
    demo("Wheel W11 (odd cycle)", 11, &wheel_edges);

    demo(
        "Petersen-like (8v, chromatic=3)",
        8,
        &[
            (0, 3),
            (0, 4),
            (1, 4),
            (2, 4),
            (1, 5),
            (2, 5),
            (3, 5),
            (0, 6),
            (1, 6),
            (2, 6),
            (3, 6),
            (0, 7),
            (1, 7),
            (2, 7),
            (4, 7),
            (5, 7),
        ],
    );

    demo("Isolated vertices", 5, &[]);

    demo("Path P5", 5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
}
