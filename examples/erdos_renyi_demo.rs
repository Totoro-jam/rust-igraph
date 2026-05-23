//! ALGO-GN-001 example: Erdős–Rényi random graph generators.
//!
//! Demonstrates both classical variants:
//!   * G(n, p) — each potential edge included independently with
//!     probability `p`. Edge count is `Binomial(max_edges, p)`.
//!   * G(n, m) — exactly `m` edges chosen uniformly at random from the
//!     `max_edges` pairs, without replacement.
//!
//! For each call we pass a seed so the run is reproducible. The
//! observed edge count of the G(n,p) sample is compared to the
//! theoretical mean `p · max_edges`; the G(n,m) sample is exact by
//! construction.
//!
//! Run: `cargo run --example erdos_renyi_demo`.
//!
//! Expected output (deterministic given the seeds below):
//!   * G(50, 0.1) undirected, no loops: roughly 122 edges
//!     (mean = 0.1 · 1225 = 122.5)
//!   * G(50, 100) undirected, no loops: exactly 100 edges
//!   * G(20, 0.4) directed, no loops: roughly 152 edges
//!     (mean = 0.4 · 380 = 152.0)

use rust_igraph::{erdos_renyi_gnm, erdos_renyi_gnp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // G(50, 0.1) undirected, no self-loops. max_edges = 50·49/2 = 1225,
    // expected ~ 122.5 edges.
    let gnp_sparse = erdos_renyi_gnp(50, 0.1, false, false, 0x2026_0523)?;
    let max_edges_np = 50 * 49 / 2;
    let expected_mean = 0.1 * f64::from(max_edges_np);
    println!(
        "G(n=50, p=0.1) undirected, no loops: {} vertices, {} edges (mean ≈ {:.1})",
        gnp_sparse.vcount(),
        gnp_sparse.ecount(),
        expected_mean,
    );

    // G(50, 100) undirected, no self-loops. m = 100 is sharp.
    let gnm_exact = erdos_renyi_gnm(50, 100, false, false, 0xCAFE_F00D)?;
    println!(
        "G(n=50, m=100) undirected, no loops: {} vertices, {} edges (exact)",
        gnm_exact.vcount(),
        gnm_exact.ecount(),
    );
    assert_eq!(gnm_exact.ecount(), 100, "G(n,m) ecount must equal m");

    // G(20, 0.4) directed, no self-loops. max_edges = 20·19 = 380,
    // expected ~ 152 edges.
    let gnp_directed = erdos_renyi_gnp(20, 0.4, true, false, 0xDEAD_BEEF)?;
    let max_edges_dir = 20 * 19;
    let expected_dir = 0.4 * f64::from(max_edges_dir);
    println!(
        "G(n=20, p=0.4) directed, no loops:   {} vertices, {} edges (mean ≈ {:.1})",
        gnp_directed.vcount(),
        gnp_directed.ecount(),
        expected_dir,
    );
    assert!(gnp_directed.is_directed(), "directed flag must be honoured");

    Ok(())
}
