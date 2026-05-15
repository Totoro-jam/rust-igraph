//! TEMPLATE: AWU test scaffold (unit + oracle + proptest in one place).
//!
//! Copy into `src/algorithms/<group>/<name>_tests.rs` (or a
//! sibling module) for unit tests, OR into `tests/` for
//! oracle + proptest blocks. The awu-tester skill fills in the bodies.
//!
//! Placeholders:
//!   {{ALGO_ID}}         e.g. ALGO-CT-002
//!   {{FN_NAME}}         e.g. betweenness
//!   {{ORACLE_SLUG}}     e.g. "betweenness"

#![allow(unused_imports)]

use rust_igraph::/* TODO({{ALGO_ID}}): module path */::{{FN_NAME}};
use rust_igraph::Graph;

// ===================================================================
// Step 5 — Unit tests (in the algorithm crate)
// ===================================================================

#[test]
fn empty_graph() {
    let g = Graph::with_vertices(0);
    // TODO({{ALGO_ID}}): expected behavior on n=0 — sane empty result OR
    // a specific IgraphError variant
    let _ = {{FN_NAME}}(&g /* TODO({{ALGO_ID}}): params */);
}

#[test]
fn single_vertex() {
    let g = Graph::with_vertices(1);
    let _ = {{FN_NAME}}(&g /* TODO({{ALGO_ID}}): params */);
}

#[test]
fn complete_k5() {
    let mut g = Graph::with_vertices(5);
    for u in 0..5 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).unwrap();
        }
    }
    // TODO({{ALGO_ID}}): assert against well-known K5 result
    let _ = {{FN_NAME}}(&g /* TODO({{ALGO_ID}}): params */);
}

#[test]
fn error_path_for_invalid_input() {
    // TODO({{ALGO_ID}}): construct an input that should error and assert
    // the IgraphError variant matches
}

// ===================================================================
// Step 6 — Live oracle (in tests/oracle.rs)
//   Cut the block below into oracle.rs; do NOT compile it here.
// ===================================================================
//
// #[test]
// fn {{FN_NAME}}_karate_matches_python_igraph() {
//     use std::fs::File;
//     let g = igraph::read_edgelist(File::open(
//         std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
//             .ancestors().nth(2).unwrap()
//             .join("fixtures/karate.edges"),
//     ).unwrap()).unwrap();
//
//     let rust = igraph::{{FN_NAME}}(&g /* TODO */).unwrap();
//     let py: Vec</* TODO */> = serde_json::from_value(common::run_ok(
//         "{{ORACLE_SLUG}}", &g, serde_json::json!({/* TODO */}),
//     )).expect("decode python result");
//
//     assert_eq!(rust, py);  // or assert_close! for floats
// }

// ===================================================================
// Step 7 — Property tests (in tests/property.rs)
// ===================================================================
//
// proptest! {
//     #[test]
//     fn {{FN_NAME}}_invariant_xxx(g in arb_graph(0..30)) {
//         // TODO({{ALGO_ID}}): pick an invariant. Common patterns:
//         // - symmetric on undirected: f(u,v) == f(v,u)
//         // - sum/normalization: |sum(values) - 1.0| < tol
//         // - covering: every vertex appears exactly once
//     }
// }
