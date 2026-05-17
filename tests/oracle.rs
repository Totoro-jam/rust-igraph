//! Oracle tests against python-igraph.
//!
//! Run with: `cargo test --features oracle-tests --test oracle`.
//! Requires `.venv/` at the repo root (see scripts/requirements.txt).

#![cfg(feature = "oracle-tests")]

mod common;

use std::fs::File;

use common::{OracleResponse, run_ok};
use rust_igraph::{
    Graph, articulation_points, assortativity_degree, avg_nearest_neighbor_degree, betweenness,
    bfs, bridges, closeness, connected_components, count_reachable, count_triangles, density, dfs,
    diameter, distances, eccentricity, edge_betweenness, girth, harmonic_centrality,
    is_biconnected, mean_distance, radius, reachability_matrix, read_edgelist, reciprocity,
    strongly_connected_components, transitive_closure, transitivity_local_undirected,
    transitivity_undirected,
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

/// Resolve a graph's bridges to canonicalised endpoint pairs `(min, max)`.
/// Edge ids aren't stable across the python wire format
/// (`GraphPayload::from_graph` rebuilds the edge list via
/// `neighbors()` iteration), so we compare endpoint sets — the two
/// impls' bridge *edge ids* may differ but the underlying edges (as
/// vertex pairs) must agree.
fn rust_bridge_pairs(g: &rust_igraph::Graph) -> Vec<(u32, u32)> {
    let mut pairs: Vec<(u32, u32)> = bridges(g)
        .expect("rust bridges")
        .into_iter()
        .map(|e| {
            let (u, v) = g.edge(e).expect("edge id valid");
            if u <= v { (u, v) } else { (v, u) }
        })
        .collect();
    pairs.sort_unstable();
    pairs
}

fn py_bridge_pairs(g: &rust_igraph::Graph) -> Vec<(u32, u32)> {
    let py_ids: Vec<u32> = serde_json::from_value(run_ok("bridges", g, serde_json::json!({})))
        .expect("decode python bridges");
    // Reconstruct the same edge list python sees on the wire, then
    // resolve ids → pairs through it.
    let payload = common::GraphPayload::from_graph(g);
    let mut pairs: Vec<(u32, u32)> = py_ids
        .iter()
        .map(|&e| {
            let (u, v) = payload.edges[e as usize];
            if u <= v { (u, v) } else { (v, u) }
        })
        .collect();
    pairs.sort_unstable();
    pairs
}

#[test]
fn bridges_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    assert_eq!(rust_bridge_pairs(&g), py_bridge_pairs(&g));
}

#[test]
fn bridges_two_triangles_via_bridge_matches_python_igraph() {
    // Triangles {0,1,2}, {3,4,5} joined by edge 2-3.
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let rust = rust_bridge_pairs(&g);
    assert_eq!(rust, py_bridge_pairs(&g));
    assert_eq!(rust, vec![(2, 3)]);
}

#[test]
fn is_biconnected_matches_python_igraph_on_several_graphs() {
    // Triangle (true), path-3 (false), 4-cycle (true), star (false).
    let cases: Vec<(rust_igraph::Graph, bool)> = vec![
        (
            {
                let mut g = Graph::with_vertices(3);
                g.add_edge(0, 1).unwrap();
                g.add_edge(1, 2).unwrap();
                g.add_edge(2, 0).unwrap();
                g
            },
            true,
        ),
        (
            {
                let mut g = Graph::with_vertices(3);
                g.add_edge(0, 1).unwrap();
                g.add_edge(1, 2).unwrap();
                g
            },
            false,
        ),
        (
            {
                let mut g = Graph::with_vertices(4);
                for i in 0..4u32 {
                    g.add_edge(i, (i + 1) % 4).unwrap();
                }
                g
            },
            true,
        ),
        (
            {
                let mut g = Graph::with_vertices(4);
                for v in 1..4u32 {
                    g.add_edge(0, v).unwrap();
                }
                g
            },
            false,
        ),
    ];
    for (g, expected) in cases {
        let rust = is_biconnected(&g).unwrap();
        let py: bool = serde_json::from_value(run_ok("is_biconnected", &g, serde_json::json!({})))
            .expect("decode python is_biconnected");
        assert_eq!(rust, py, "rust vs python disagreed on graph");
        assert_eq!(rust, expected, "rust mismatch with expected");
    }
}

#[test]
fn girth_matches_python_igraph_on_several_graphs() {
    // Triangle (3), 4-cycle (4), 5-cycle (5), tree (None).
    let cases: Vec<(rust_igraph::Graph, Option<u32>)> = vec![
        (
            {
                let mut g = Graph::with_vertices(3);
                g.add_edge(0, 1).unwrap();
                g.add_edge(1, 2).unwrap();
                g.add_edge(2, 0).unwrap();
                g
            },
            Some(3),
        ),
        (
            {
                let mut g = Graph::with_vertices(4);
                for i in 0..4u32 {
                    g.add_edge(i, (i + 1) % 4).unwrap();
                }
                g
            },
            Some(4),
        ),
        (
            {
                let mut g = Graph::with_vertices(5);
                for i in 0..5u32 {
                    g.add_edge(i, (i + 1) % 5).unwrap();
                }
                g
            },
            Some(5),
        ),
        (
            {
                let mut g = Graph::with_vertices(4);
                g.add_edge(0, 1).unwrap();
                g.add_edge(1, 2).unwrap();
                g.add_edge(2, 3).unwrap();
                g
            },
            None,
        ),
    ];
    for (g, expected) in cases {
        let rust = girth(&g).unwrap();
        let py: Option<u32> = serde_json::from_value(run_ok("girth", &g, serde_json::json!({})))
            .expect("decode python girth");
        assert_eq!(rust, py, "rust vs python disagreed on a girth case");
        assert_eq!(rust, expected, "rust mismatch with expected");
    }
}

#[test]
fn girth_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = girth(&g).unwrap();
    let py: Option<u32> = serde_json::from_value(run_ok("girth", &g, serde_json::json!({})))
        .expect("decode python girth");
    assert_eq!(rust, py);
    // Karate has many triangles → girth 3.
    assert_eq!(rust, Some(3));
}

#[test]
fn eccentricity_radius_diameter_karate_match_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust_ecc = eccentricity(&g).unwrap();
    let py_ecc: Vec<u32> =
        serde_json::from_value(run_ok("eccentricity", &g, serde_json::json!({})))
            .expect("decode python eccentricity");
    assert_eq!(rust_ecc, py_ecc);

    let rust_r = radius(&g).unwrap();
    let py_r: Option<u32> = serde_json::from_value(run_ok("radius", &g, serde_json::json!({})))
        .expect("decode python radius");
    assert_eq!(rust_r, py_r);

    let rust_d = diameter(&g).unwrap();
    let py_d: Option<u32> = serde_json::from_value(run_ok("diameter", &g, serde_json::json!({})))
        .expect("decode python diameter");
    assert_eq!(rust_d, py_d);
}

#[test]
fn count_triangles_and_transitivity_karate_match_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust_n = count_triangles(&g).unwrap();
    let py_n: u64 = serde_json::from_value(run_ok("count_triangles", &g, serde_json::json!({})))
        .expect("decode python count_triangles");
    assert_eq!(rust_n, py_n);

    let rust_t = transitivity_undirected(&g).unwrap();
    let py_t: Option<f64> =
        serde_json::from_value(run_ok("transitivity_undirected", &g, serde_json::json!({})))
            .expect("decode python transitivity");
    // Both impls compute 3 * triangles / triples on the same graph; the
    // operands are integers, so the f64 result is exactly representable.
    assert_eq!(rust_t, py_t);
}

#[test]
fn transitivity_local_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust = transitivity_local_undirected(&g).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok(
        "transitivity_local_undirected",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode python local transitivity");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-12, "vertex {i}: rust={rr} python={pp}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} python={b:?}"),
        }
    }
}

#[test]
fn density_and_mean_distance_karate_match_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    let rust_density = density(&g).unwrap();
    let py_density: Option<f64> =
        serde_json::from_value(run_ok("density", &g, serde_json::json!({})))
            .expect("decode python density");
    match (rust_density, py_density) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "density rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("density rust={a:?} py={b:?}"),
    }

    let rust_mean = mean_distance(&g).unwrap();
    let py_mean: Option<f64> =
        serde_json::from_value(run_ok("mean_distance", &g, serde_json::json!({})))
            .expect("decode python mean_distance");
    match (rust_mean, py_mean) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "mean_distance rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("mean_distance rust={a:?} py={b:?}"),
    }
}

#[test]
fn count_reachable_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = count_reachable(&g).unwrap();
    let py: Vec<u32> = serde_json::from_value(run_ok("count_reachable", &g, serde_json::json!({})))
        .expect("decode python count_reachable");
    assert_eq!(rust, py);
    // Karate is fully connected → every vertex reaches all 34.
    assert_eq!(rust, vec![34; 34]);
}

#[test]
fn count_reachable_directed_chain_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = count_reachable(&g).unwrap();
    let py: Vec<u32> = serde_json::from_value(run_ok("count_reachable", &g, serde_json::json!({})))
        .expect("decode python count_reachable");
    assert_eq!(rust, py);
    assert_eq!(rust, vec![4, 3, 2, 1]);
}

#[test]
fn reciprocity_directed_chain_with_partial_back_matches_python_igraph() {
    // 0 -> 1, 1 -> 0, 0 -> 2: 3 edges, 2 reciprocal → 2/3.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    g.add_edge(0, 2).unwrap();
    let rust_r = reciprocity(&g).unwrap();
    let py_r: Option<f64> =
        serde_json::from_value(run_ok("reciprocity", &g, serde_json::json!({})))
            .expect("decode python reciprocity");
    match (rust_r, py_r) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn knn_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = avg_nearest_neighbor_degree(&g).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok(
        "avg_nearest_neighbor_degree",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode python knn");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(a), Some(b)) => {
                assert!((a - b).abs() < 1e-12, "vertex {i}: rust={a} py={b}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn assortativity_degree_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = assortativity_degree(&g).unwrap();
    let py: Option<f64> =
        serde_json::from_value(run_ok("assortativity_degree", &g, serde_json::json!({})))
            .expect("decode python assortativity");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn reachability_matrix_directed_3cycle_matches_python_igraph() {
    // Directed 3-cycle: every vertex reaches every other.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let rust = reachability_matrix(&g).unwrap();
    let py: Vec<Vec<bool>> =
        serde_json::from_value(run_ok("reachability_matrix", &g, serde_json::json!({})))
            .expect("decode python reachability_matrix");
    assert_eq!(rust, py);
}

#[test]
fn reachability_matrix_disconnected_undirected_matches_python_igraph() {
    // {0-1-2} and {3-4}: cross-component pairs unreachable.
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(3, 4).unwrap();
    let rust = reachability_matrix(&g).unwrap();
    let py: Vec<Vec<bool>> =
        serde_json::from_value(run_ok("reachability_matrix", &g, serde_json::json!({})))
            .expect("decode python reachability_matrix");
    assert_eq!(rust, py);
}

/// Wire-format payload returned by the `transitive_closure` oracle.
#[derive(serde::Deserialize)]
struct PyTransitiveClosure {
    vcount: u32,
    directed: bool,
    edges: Vec<[u32; 2]>,
}

fn rust_tc_pairs(tc: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(tc.ecount()).expect("ecount fits in u32");
    let mut pairs: Vec<(u32, u32)> = (0..m).map(|e| tc.edge(e).unwrap()).collect();
    pairs.sort_unstable();
    pairs
}

fn py_tc_pairs(py: &PyTransitiveClosure) -> Vec<(u32, u32)> {
    let mut pairs: Vec<(u32, u32)> = py.edges.iter().map(|p| (p[0], p[1])).collect();
    pairs.sort_unstable();
    pairs
}

fn transitive_closure_oracle_pair(g: &Graph) -> (PyTransitiveClosure, Graph) {
    let tc = transitive_closure(g).expect("rust transitive_closure");
    let py: PyTransitiveClosure =
        serde_json::from_value(run_ok("transitive_closure", g, serde_json::json!({})))
            .expect("decode python transitive_closure");
    (py, tc)
}

#[test]
fn transitive_closure_directed_path_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let (py, tc) = transitive_closure_oracle_pair(&g);
    assert_eq!(tc.vcount(), py.vcount);
    assert_eq!(tc.is_directed(), py.directed);
    assert_eq!(rust_tc_pairs(&tc), py_tc_pairs(&py));
}

#[test]
fn transitive_closure_undirected_two_components_matches_python_igraph() {
    // {0-1-2} and {3-4}: closure has within-component edges only.
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(3, 4).unwrap();
    let (py, tc) = transitive_closure_oracle_pair(&g);
    assert_eq!(tc.vcount(), py.vcount);
    assert_eq!(tc.is_directed(), py.directed);
    assert_eq!(rust_tc_pairs(&tc), py_tc_pairs(&py));
}

/// Wire-format payload returned by the `edge_betweenness` oracle.
#[derive(serde::Deserialize)]
struct PyEdgeBetweenness {
    edges: Vec<[u32; 2]>,
    values: Vec<f64>,
}

#[test]
fn edge_betweenness_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");

    // Rust: build (canonical pair, score) map.
    let rust_eb = edge_betweenness(&g).unwrap();
    let m = u32::try_from(g.ecount()).expect("ecount fits");
    let mut rust_map: std::collections::BTreeMap<(u32, u32), f64> =
        std::collections::BTreeMap::new();
    for e in 0..m {
        let (u, v) = g.edge(e).unwrap();
        let key = if u <= v { (u, v) } else { (v, u) };
        // Same pair could appear twice in a multigraph; sum the scores.
        *rust_map.entry(key).or_insert(0.0) += rust_eb[e as usize];
    }

    // Python: pull `(edges, values)` and build the same map.
    let py: PyEdgeBetweenness =
        serde_json::from_value(run_ok("edge_betweenness", &g, serde_json::json!({})))
            .expect("decode python edge_betweenness");
    let mut py_map: std::collections::BTreeMap<(u32, u32), f64> = std::collections::BTreeMap::new();
    for (pair, &val) in py.edges.iter().zip(py.values.iter()) {
        let (u, v) = (pair[0], pair[1]);
        let key = if u <= v { (u, v) } else { (v, u) };
        *py_map.entry(key).or_insert(0.0) += val;
    }

    assert_eq!(rust_map.len(), py_map.len(), "edge count mismatch");
    for ((rk, rv), (pk, pv)) in rust_map.iter().zip(py_map.iter()) {
        assert_eq!(rk, pk);
        assert!((rv - pv).abs() < 1e-9, "edge {rk:?}: rust={rv} py={pv}");
    }
}

#[test]
fn betweenness_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = betweenness(&g).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok("betweenness", &g, serde_json::json!({})))
        .expect("decode python betweenness");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-10, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn harmonic_centrality_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = harmonic_centrality(&g).unwrap();
    let py: Vec<f64> =
        serde_json::from_value(run_ok("harmonic_centrality", &g, serde_json::json!({})))
            .expect("decode python harmonic");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn closeness_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = closeness(&g).unwrap();
    let py: Vec<Option<f64>> =
        serde_json::from_value(run_ok("closeness", &g, serde_json::json!({})))
            .expect("decode python closeness");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-12, "vertex {i}: rust={rr} py={pp}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
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
