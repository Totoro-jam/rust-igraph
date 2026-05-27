//! ALGO-SP-031 example: all simple paths demo.
//!
//! Run: `cargo run --example simple_paths_demo`.

use rust_igraph::{Graph, SimplePathMode, all_simple_paths, create};

fn main() {
    println!("=== Path graph 0-1-2-3 ===");
    let g = create(&[(0, 1), (1, 2), (2, 3)], 4, false).unwrap();
    let paths = all_simple_paths(&g, 0, None, SimplePathMode::Out, 0, -1, -1).unwrap();
    for p in &paths {
        println!("  {p:?}");
    }

    println!("\n=== Triangle 0-1-2, from 0 ===");
    let g = create(&[(0, 1), (1, 2), (0, 2)], 3, false).unwrap();
    let paths = all_simple_paths(&g, 0, None, SimplePathMode::Out, 0, -1, -1).unwrap();
    for p in &paths {
        println!("  {p:?}");
    }

    println!("\n=== Directed cycle 0->1->2->0, from 0, OUT mode ===");
    let g = create(&[(0, 1), (1, 2), (2, 0)], 3, true).unwrap();
    let paths = all_simple_paths(&g, 0, None, SimplePathMode::Out, 0, -1, -1).unwrap();
    for p in &paths {
        println!("  {p:?}");
    }

    println!("\n=== Same cycle, IN mode ===");
    let g = create(&[(0, 1), (1, 2), (2, 0)], 3, true).unwrap();
    let paths = all_simple_paths(&g, 0, None, SimplePathMode::In, 0, -1, -1).unwrap();
    for p in &paths {
        println!("  {p:?}");
    }

    println!("\n=== Path 0-1-2-3, target=[3], min_len=2 ===");
    let g = create(&[(0, 1), (1, 2), (2, 3)], 4, false).unwrap();
    let paths = all_simple_paths(&g, 0, Some(&[3]), SimplePathMode::Out, 2, -1, -1).unwrap();
    for p in &paths {
        println!("  {p:?}");
    }

    println!("\n=== K4, from 0, max_results=5 ===");
    let mut g = Graph::with_vertices(4);
    for i in 0..4u32 {
        for j in (i + 1)..4 {
            g.add_edge(i, j).unwrap();
        }
    }
    let paths = all_simple_paths(&g, 0, None, SimplePathMode::Out, 0, -1, 5).unwrap();
    println!("  ({} paths, capped at 5)", paths.len());
    for p in &paths {
        println!("  {p:?}");
    }
}
