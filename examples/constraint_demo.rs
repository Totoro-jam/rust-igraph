//! ALGO-PR-032 example: Burt's constraint demo.
//!
//! Run: `cargo run --example constraint_demo`.

use rust_igraph::{Graph, constraint, create};

fn main() {
    println!("=== Star graph (center + 4 leaves) ===");
    let mut g = Graph::with_vertices(5);
    for i in 1..5 {
        g.add_edge(0, i).unwrap();
    }
    let c = constraint(&g, None).unwrap();
    println!("  Center (v0): {:.4}", c[0]);
    println!("  Leaf   (v1): {:.4}", c[1]);

    println!("\n=== Triangle (0-1-2) ===");
    let g = create(&[(0, 1), (1, 2), (0, 2)], 3, false).unwrap();
    let c = constraint(&g, None).unwrap();
    for (i, val) in c.iter().enumerate() {
        println!("  v{i}: {val:.4}");
    }

    println!("\n=== Path 0-1-2 ===");
    let g = create(&[(0, 1), (1, 2)], 3, false).unwrap();
    let c = constraint(&g, None).unwrap();
    for (i, val) in c.iter().enumerate() {
        println!("  v{i}: {val:.4}");
    }

    println!("\n=== K4 complete ===");
    let g = create(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4, false).unwrap();
    let c = constraint(&g, None).unwrap();
    for (i, val) in c.iter().enumerate() {
        println!("  v{i}: {val:.4}");
    }

    println!("\n=== Weighted path 0-1-2 (w=[1,3]) ===");
    let g = create(&[(0, 1), (1, 2)], 3, false).unwrap();
    let c = constraint(&g, Some(&[1.0, 3.0])).unwrap();
    for (i, val) in c.iter().enumerate() {
        println!("  v{i}: {val:.4}");
    }

    println!("\n=== Directed path 0->1->2 ===");
    let g = create(&[(0, 1), (1, 2)], 3, true).unwrap();
    let c = constraint(&g, None).unwrap();
    for (i, val) in c.iter().enumerate() {
        println!("  v{i}: {val:.4}");
    }
}
