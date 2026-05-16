//! Oracle tests against python-igraph.
//!
//! Run with: `cargo test --features oracle-tests --test oracle`.
//! Requires `.venv/` at the repo root (see scripts/requirements.txt).

#![cfg(feature = "oracle-tests")]

mod common;

use std::fs::File;

use common::{OracleResponse, run_ok};
use rust_igraph::{
    Graph, articulation_points, bfs, connected_components, dfs, distances, read_edgelist,
    strongly_connected_components,
};

fn workspace_fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

/// Wire-format payload returned by the oracle for both weak (`cc`) and
/// strong (`scc`) connected-components algos: the result is structurally
/// identical (`{"membership": [...], "count": N}`).
#[derive(serde::Deserialize)]
struct PyComponents {
    membership: Vec<u32>,
    count: u32,
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
fn connected_components_karate_matches_python_igraph() {
    // Karate is a single connected component; verifies degenerate
    // n=1 result and BFS-based traversal correctness.
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust_cc = connected_components(&g).expect("rust cc");

    let py_cc: PyComponents =
        serde_json::from_value(run_ok("connected_components", &g, serde_json::json!({})))
            .expect("decode python cc");
    assert_eq!(rust_cc.membership, py_cc.membership);
    assert_eq!(rust_cc.count, py_cc.count);
    assert_eq!(rust_cc.count, 1);
}

#[test]
fn connected_components_two_components() {
    // 5 vertices, 2 components: {0,1,2}, {3,4}.
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(3, 4).unwrap();
    let rust_cc = connected_components(&g).expect("rust cc");

    let py_cc: PyComponents =
        serde_json::from_value(run_ok("connected_components", &g, serde_json::json!({})))
            .expect("decode python cc");
    assert_eq!(rust_cc.membership, py_cc.membership);
    assert_eq!(rust_cc.count, py_cc.count);
    assert_eq!(rust_cc.count, 2);
}

#[test]
fn strongly_connected_components_two_disjoint_cycles_matches_python_igraph() {
    // Two disjoint directed 3-cycles: 0->1->2->0 and 3->4->5->3.
    // SCC count = 2; rust + python-igraph follow the same Kosaraju
    // grandfather-pop labelling so membership vectors must agree exactly.
    let mut g = Graph::new(6, true).expect("new directed");
    for (u, v) in [(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)] {
        g.add_edge(u, v).expect("add edge");
    }
    let rust_scc = strongly_connected_components(&g).expect("rust scc");

    let py_scc: PyComponents = serde_json::from_value(run_ok(
        "strongly_connected_components",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode python scc");
    assert_eq!(rust_scc.membership, py_scc.membership);
    assert_eq!(rust_scc.count, py_scc.count);
    assert_eq!(rust_scc.count, 2);
}

#[test]
fn strongly_connected_components_cycle_with_chain_matches_python_igraph() {
    // 0 -> 1 -> 2 -> 0 forms a cycle; 2 -> 3 -> 4 are dangling singletons.
    let mut g = Graph::new(5, true).expect("new directed");
    for (u, v) in [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4)] {
        g.add_edge(u, v).expect("add edge");
    }
    let rust_scc = strongly_connected_components(&g).expect("rust scc");

    let py_scc: PyComponents = serde_json::from_value(run_ok(
        "strongly_connected_components",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode python scc");
    assert_eq!(rust_scc.membership, py_scc.membership);
    assert_eq!(rust_scc.count, py_scc.count);
    assert_eq!(rust_scc.count, 3);
}

#[test]
fn distances_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust = distances(&g, 0).expect("rust distances");
    let py: Vec<Option<u32>> =
        serde_json::from_value(run_ok("distances", &g, serde_json::json!({"source": 0})))
            .expect("decode python distances");
    assert_eq!(rust, py);
}

#[test]
fn distances_directed_chain_matches_python_igraph() {
    // Directed 0 -> 1 -> 2 -> 3; from 1 distances [inf, 0, 1, 2].
    let mut g = Graph::new(4, true).expect("new directed");
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = distances(&g, 1).expect("rust distances");
    let py: Vec<Option<u32>> =
        serde_json::from_value(run_ok("distances", &g, serde_json::json!({"source": 1})))
            .expect("decode python distances");
    assert_eq!(rust, py);
    assert_eq!(rust, vec![None, Some(0), Some(1), Some(2)]);
}

#[test]
fn articulation_points_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let mut rust = articulation_points(&g).expect("rust articulation");
    rust.sort_unstable();
    let py: Vec<u32> =
        serde_json::from_value(run_ok("articulation_points", &g, serde_json::json!({})))
            .expect("decode python articulation");
    assert_eq!(rust, py);
}

#[test]
fn articulation_points_cycle_with_pendant_matches_python_igraph() {
    // Cycle 0-1-2-0 plus pendant 2-3-4: expected articulation = {2, 3}.
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(3, 4).unwrap();
    let mut rust = articulation_points(&g).expect("rust articulation");
    rust.sort_unstable();
    let py: Vec<u32> =
        serde_json::from_value(run_ok("articulation_points", &g, serde_json::json!({})))
            .expect("decode python articulation");
    assert_eq!(rust, py);
    assert_eq!(rust, vec![2, 3]);
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
