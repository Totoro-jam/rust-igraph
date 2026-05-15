//! Oracle tests against python-igraph.
//!
//! Run with: `cargo test --features oracle-tests --test oracle`.
//! Requires `.venv/` at the repo root (see scripts/requirements.txt).

#![cfg(feature = "oracle-tests")]

mod common;

use std::fs::File;

use common::{OracleResponse, run_ok};
use rust_igraph::{Graph, bfs, dfs, read_edgelist};

fn workspace_fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn oracle_protocol_smoke_test() {
    // Trivial 2-node path graph; just check the protocol works end to end.
    let mut g = Graph::with_vertices(2);
    g.add_edge(0, 1).unwrap();
    let result = run_ok("bfs", &g, serde_json::json!({"root": 0}));
    let order: Vec<u32> = serde_json::from_value(result).expect("vec<u32>");
    assert_eq!(order, vec![0, 1]);
}

#[test]
fn oracle_reports_failure_for_unknown_algo() {
    let g = Graph::with_vertices(1);
    let resp: OracleResponse = common::run("not_a_real_algo", &g, serde_json::json!({}));
    assert!(!resp.ok);
    assert!(resp.error.unwrap().contains("not_a_real_algo"));
}

#[test]
fn bfs_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust_order = bfs(&g, 0).expect("rust bfs");
    let py_order: Vec<u32> =
        serde_json::from_value(run_ok("bfs", &g, serde_json::json!({"root": 0})))
            .expect("decode python order");

    // BFS visit order is a function of *adjacency-list iteration order*, which
    // both implementations build from the same edge list. We therefore expect
    // exact equality on this fixture. If a future change permutes neighbor
    // order, relax this to a layer-equivalence check.
    assert_eq!(rust_order, py_order, "BFS visit order mismatch");
}

#[test]
fn dfs_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust_order = dfs(&g, 0).expect("rust dfs");
    let py_order: Vec<u32> =
        serde_json::from_value(run_ok("dfs", &g, serde_json::json!({"root": 0})))
            .expect("decode python order");

    // DFS pre-order parity. Like BFS, both implementations consume the
    // same neighbour iteration order; equality holds on this fixture.
    // The reverse-on-push step in `dfs.rs` matches upstream igraph's
    // lazy-adjlist behaviour (see comment there).
    assert_eq!(rust_order, py_order, "DFS visit order mismatch");
}

#[test]
fn dfs_small_synthetic_matches_python_igraph() {
    // Small handcrafted regression case from the AWU:
    // edges (0,1)(0,2)(1,3) — caught the original neighbour-iteration
    // direction mismatch when DFS first landed.
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 3).unwrap();

    let rust_order = dfs(&g, 0).expect("rust dfs");
    let py_order: Vec<u32> =
        serde_json::from_value(run_ok("dfs", &g, serde_json::json!({"root": 0})))
            .expect("decode python order");

    assert_eq!(rust_order, py_order);
}
