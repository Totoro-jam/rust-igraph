//! ALGO-LO-001 — simple layout algorithms demo.
//!
//! Run: `cargo run --example layout_demo`

use rust_igraph::{
    Graph, layout_circle, layout_grid, layout_grid_3d, layout_random, layout_random_3d,
    layout_sphere, layout_star,
};

fn main() {
    let g = Graph::with_vertices(6);

    println!("=== layout_random (seed=42) ===");
    let coords = layout_random(&g, 42);
    for (i, &(x, y)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4})");
    }

    println!("\n=== layout_random_3d (seed=42) ===");
    let coords = layout_random_3d(&g, 42);
    for (i, &(x, y, z)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4}, {z:.4})");
    }

    println!("\n=== layout_circle ===");
    let coords = layout_circle(&g, None);
    for (i, &(x, y)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4})");
    }

    println!("\n=== layout_star (center=0) ===");
    let coords = layout_star(&g, 0, None).expect("star layout");
    for (i, &(x, y)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4})");
    }

    println!("\n=== layout_grid (width=3) ===");
    let coords = layout_grid(&g, 3);
    for (i, &(x, y)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4})");
    }

    println!("\n=== layout_grid_3d (2x3) ===");
    let g8 = Graph::with_vertices(8);
    let coords = layout_grid_3d(&g8, 2, 2);
    for (i, &(x, y, z)) in coords.iter().enumerate() {
        println!("  v{i}: ({x:.4}, {y:.4}, {z:.4})");
    }

    println!("\n=== layout_sphere (10 vertices) ===");
    let g10 = Graph::with_vertices(10);
    let coords = layout_sphere(&g10);
    for (i, &(x, y, z)) in coords.iter().enumerate() {
        let r = (x * x + y * y + z * z).sqrt();
        println!("  v{i}: ({x:.4}, {y:.4}, {z:.4})  r={r:.4}");
    }
}
