//! ALGO-PR-033 — centralization functions demo.
//!
//! Run: `cargo run --example centralization_demo`

use rust_igraph::{
    CentralizationMode, LoopMode, centralization, centralization_betweenness_tmax,
    centralization_closeness_tmax, centralization_degree_tmax, centralization_eigenvector_tmax,
};

fn main() {
    let n = 5u32;

    println!("=== Star graph (n={n}) degree centralization ===");
    let scores = [4.0, 1.0, 1.0, 1.0, 1.0];
    let tmax = centralization_degree_tmax(n, false, CentralizationMode::All, LoopMode::NoLoops);
    let c = centralization(&scores, tmax, true);
    println!("  scores: {scores:?}");
    println!("  tmax:   {tmax:.2}");
    println!("  C:      {c:.6}  (1.0 = maximally centralized)");

    println!("\n=== Ring graph (n={n}) degree centralization ===");
    let scores = [2.0, 2.0, 2.0, 2.0, 2.0];
    let c = centralization(&scores, tmax, true);
    println!("  scores: {scores:?}");
    println!("  C:      {c:.6}  (0.0 = uniform)");

    println!("\n=== Betweenness tmax ===");
    let bt_u = centralization_betweenness_tmax(n, false);
    let bt_d = centralization_betweenness_tmax(n, true);
    println!("  undirected n={n}: {bt_u:.2}");
    println!("  directed   n={n}: {bt_d:.2}");

    println!("\n=== Closeness tmax ===");
    let ct = centralization_closeness_tmax(n, CentralizationMode::All);
    println!("  undirected n={n}: {ct:.6}");

    println!("\n=== Eigenvector tmax ===");
    let et = centralization_eigenvector_tmax(n, CentralizationMode::All);
    println!("  undirected n={n}: {et:.2}");
}
