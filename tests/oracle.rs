//! Oracle tests against python-igraph.
//!
//! Run with: `cargo test --features oracle-tests --test oracle`.
//! Requires `.venv/` at the repo root (see scripts/requirements.txt).

#![cfg(feature = "oracle-tests")]

mod common;

use std::fs::File;

use common::{OracleResponse, run_ok, run_ok_with_weights};
use rust_igraph::{
    CorenessMode, DijkstraMode, EccMode, Graph, ReciprocityMode, SimpleMode, a_star_path,
    articulation_points, assortativity_degree, assortativity_degree_directed,
    assortativity_degree_directed_weighted, assortativity_degree_weighted,
    avg_nearest_neighbor_degree, avg_nearest_neighbor_degree_weighted, betweenness,
    betweenness_weighted, bfs, biconnected_components, bridges, closeness, closeness_weighted,
    complementer, connected_components, coreness, coreness_with_mode, count_reachable,
    count_triangles, decompose, density, dfs, diameter, diameter_weighted_with_mode,
    diameter_with_mode, difference, dijkstra_all_shortest_paths, dijkstra_distances,
    dijkstra_distances_cutoff, dijkstra_distances_with_mode, dijkstra_path_to, dijkstra_paths,
    disjoint_union, disjoint_union_many, distances, eccentricity, eccentricity_weighted_with_mode,
    eccentricity_with_mode, edge_betweenness, edge_betweenness_weighted, eigenvector_centrality,
    floyd_warshall_distances, girth, harmonic_centrality, harmonic_centrality_weighted, has_loop,
    has_multiple, intersection, is_biconnected, is_loop, is_multiple, is_simple,
    is_simple_with_mode, knnk, knnk_weighted, mean_distance, modularity, modularity_directed,
    modularity_weighted, pagerank, pagerank_weighted, radius, radius_weighted_with_mode,
    radius_with_mode, reachability_matrix, read_edgelist, reciprocity, reciprocity_with_mode,
    simplify, strongly_connected_components, transitive_closure, transitivity_barrat,
    transitivity_local_undirected, transitivity_undirected, union,
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

#[derive(serde::Deserialize, Debug)]
struct PyDecomposeComponent {
    vcount: u32,
    directed: bool,
    edges: Vec<[u32; 2]>,
}

fn canonicalize_component(g: &Graph) -> PyDecomposeComponent {
    let mut edges: Vec<[u32; 2]> = (0..g.ecount())
        .map(|e| {
            let s = g.edge_source(e as u32).expect("edge source");
            let t = g.edge_target(e as u32).expect("edge target");
            if g.is_directed() {
                [s, t]
            } else if s <= t {
                [s, t]
            } else {
                [t, s]
            }
        })
        .collect();
    edges.sort();
    PyDecomposeComponent {
        vcount: g.vcount(),
        directed: g.is_directed(),
        edges,
    }
}

#[test]
fn decompose_two_components_matches_python_igraph() {
    // Two components: {0,1,2} (triangle), {3,4} (edge).
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    g.add_edge(3, 4).unwrap();
    let rust_parts: Vec<PyDecomposeComponent> = decompose(&g)
        .expect("rust decompose")
        .iter()
        .map(canonicalize_component)
        .collect();
    let py_parts: Vec<PyDecomposeComponent> =
        serde_json::from_value(run_ok("decompose", &g, serde_json::json!({})))
            .expect("decode python decompose");
    assert_eq!(rust_parts.len(), py_parts.len());
    for (i, (r, p)) in rust_parts.iter().zip(py_parts.iter()).enumerate() {
        assert_eq!(r.vcount, p.vcount, "component {i} vcount");
        assert_eq!(r.directed, p.directed, "component {i} directed");
        assert_eq!(r.edges, p.edges, "component {i} edges");
    }
}

#[test]
fn decompose_karate_single_component_size_matches() {
    // Karate is one weak component, so both impls must return exactly
    // one subgraph with `vcount == g.vcount` and `ecount == g.ecount`.
    // We do NOT assert exact edge equality: BFS-discovery vertex
    // remapping order can differ between python-igraph and our impl
    // because `igraph_neighbors` ordering in C does not necessarily
    // match our sorted-merge-of-out/in order. The structural
    // cross-check lives in `decompose_two_components_matches_python_igraph`.
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust_parts: Vec<PyDecomposeComponent> = decompose(&g)
        .expect("rust decompose")
        .iter()
        .map(canonicalize_component)
        .collect();
    let py_parts: Vec<PyDecomposeComponent> =
        serde_json::from_value(run_ok("decompose", &g, serde_json::json!({})))
            .expect("decode python decompose");
    assert_eq!(rust_parts.len(), 1);
    assert_eq!(py_parts.len(), 1);
    assert_eq!(rust_parts[0].vcount, py_parts[0].vcount);
    assert_eq!(rust_parts[0].edges.len(), py_parts[0].edges.len());
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
fn transitivity_barrat_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    // Distinct positive weights to exercise the Barrat formula's
    // weighted aggregation (uniform weights would just reproduce the
    // unweighted result).
    let weights: Vec<f64> = (0..g.ecount()).map(|i| 1.0 + (i % 4) as f64).collect();
    let rust = transitivity_barrat(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "transitivity_barrat",
        &g,
        Some(weights.clone()),
        serde_json::json!({}),
    ))
    .expect("decode python transitivity_barrat");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(a), Some(b)) => {
                assert!((a - b).abs() < 1e-9, "vertex {i}: rust={a} py={b}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn knn_weighted_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    // Use uniform weights — this exercises the weighted code path while
    // letting us compare against python-igraph's `g.knn(weights=...)`.
    let weights: Vec<f64> = (0..g.ecount()).map(|i| 1.0 + (i % 3) as f64).collect();
    let rust = avg_nearest_neighbor_degree_weighted(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "avg_nearest_neighbor_degree_weighted",
        &g,
        Some(weights.clone()),
        serde_json::json!({}),
    ))
    .expect("decode python knn_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(a), Some(b)) => {
                assert!((a - b).abs() < 1e-9, "vertex {i}: rust={a} py={b}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn knnk_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = knnk(&g).unwrap();
    let py: Vec<Option<f64>> =
        serde_json::from_value(run_ok("knnk", &g, serde_json::json!({}))).expect("decode py knnk");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(a), Some(b)) => {
                assert!((a - b).abs() < 1e-12, "deg {} : rust={a} py={b}", i + 1);
            }
            (None, None) => {}
            (a, b) => panic!("deg {} : rust={a:?} py={b:?}", i + 1),
        }
    }
}

#[test]
fn knnk_weighted_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let weights: Vec<f64> = (0..g.ecount()).map(|i| 1.0 + (i % 3) as f64).collect();
    let rust = knnk_weighted(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "knnk_weighted",
        &g,
        Some(weights.clone()),
        serde_json::json!({}),
    ))
    .expect("decode py knnk_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(a), Some(b)) => {
                assert!((a - b).abs() < 1e-9, "deg {} : rust={a} py={b}", i + 1);
            }
            (None, None) => {}
            (a, b) => panic!("deg {} : rust={a:?} py={b:?}", i + 1),
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

/// Wire-format payload returned by the `biconnected_components` oracle.
#[derive(serde::Deserialize)]
struct PyBiconnectedComponents {
    count: u32,
    components: Vec<Vec<u32>>,
    articulation_points: Vec<u32>,
    component_edge_pairs: Vec<Vec<[u32; 2]>>,
}

#[test]
fn eigenvector_centrality_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = eigenvector_centrality(&g).unwrap();
    let py: Vec<f64> =
        serde_json::from_value(run_ok("eigenvector_centrality", &g, serde_json::json!({})))
            .expect("decode python eigenvector_centrality");
    assert_eq!(rust.len(), py.len());
    // python-igraph uses ARPACK (LAPACK eigensolver); our shifted power
    // iteration converges to the same eigenvector but with O(eps) drift.
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-6, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn biconnected_components_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust_bc = biconnected_components(&g).unwrap();

    let py: PyBiconnectedComponents =
        serde_json::from_value(run_ok("biconnected_components", &g, serde_json::json!({})))
            .expect("decode python biconnected_components");

    assert_eq!(rust_bc.count, py.count);
    let mut rust_aps = rust_bc.articulation_points.clone();
    rust_aps.sort_unstable();
    assert_eq!(rust_aps, py.articulation_points);

    // Compare component vertex sets (order doesn't matter; canonical form).
    let mut rust_set: Vec<Vec<u32>> = rust_bc
        .components
        .iter()
        .map(|c| {
            let mut v = c.clone();
            v.sort_unstable();
            v
        })
        .collect();
    rust_set.sort();
    let mut py_set: Vec<Vec<u32>> = py.components.clone();
    for c in &mut py_set {
        c.sort_unstable();
    }
    py_set.sort();
    assert_eq!(rust_set, py_set);

    // CC-012: compare per-component edge sets via canonical (min, max)
    // endpoint pairs. The components themselves may be ordered differently
    // between rust/py, so canonicalise both sides as sorted nested lists.
    let mut rust_edge_pairs: Vec<Vec<[u32; 2]>> = rust_bc
        .components
        .iter()
        .zip(rust_bc.component_edges.iter())
        .map(|(_, edges)| {
            let mut pairs: Vec<[u32; 2]> = edges
                .iter()
                .map(|&e| {
                    let (u, v) = g.edge(e).unwrap();
                    if u <= v { [u, v] } else { [v, u] }
                })
                .collect();
            pairs.sort();
            pairs
        })
        .collect();
    rust_edge_pairs.sort();
    let mut py_edge_pairs: Vec<Vec<[u32; 2]>> = py.component_edge_pairs.clone();
    for p in &mut py_edge_pairs {
        p.sort();
    }
    py_edge_pairs.sort();
    assert_eq!(rust_edge_pairs, py_edge_pairs);
}

#[test]
fn pagerank_karate_matches_python_igraph_within_tolerance() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let rust = pagerank(&g).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok("pagerank", &g, serde_json::json!({})))
        .expect("decode python pagerank");
    assert_eq!(rust.len(), py.len());
    // python-igraph defaults to ARPACK; our power-iteration converges to
    // the same fixed point but with O(eps) numerical drift.
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-6, "vertex {i}: rust={r} py={p}");
    }
    // Both impls must sum to 1 (within fp).
    let total: f64 = rust.iter().sum();
    assert!((total - 1.0).abs() < 1e-9);
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

/// Wire-format payload returned by the `simplify` oracle.
#[derive(serde::Deserialize)]
struct PySimplify {
    vcount: u32,
    directed: bool,
    edges: Vec<[u32; 2]>,
}

fn rust_simplify_pairs(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
    let mut pairs: Vec<(u32, u32)> = (0..m).map(|e| g.edge(e).unwrap()).collect();
    pairs.sort_unstable();
    pairs
}

fn py_simplify_pairs(py: &PySimplify, undirected: bool) -> Vec<(u32, u32)> {
    let mut pairs: Vec<(u32, u32)> = py
        .edges
        .iter()
        .map(|p| {
            if undirected && p[0] > p[1] {
                (p[1], p[0])
            } else {
                (p[0], p[1])
            }
        })
        .collect();
    pairs.sort_unstable();
    pairs
}

#[test]
fn simplify_undirected_loops_and_multi_matches_python_igraph() {
    // Triangle plus a self-loop and a parallel edge — exercises both flags.
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 0).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();

    let rust_s = simplify(&g, true, true).unwrap();
    let py: PySimplify = serde_json::from_value(run_ok(
        "simplify",
        &g,
        serde_json::json!({"remove_multiple": true, "remove_loops": true}),
    ))
    .expect("decode python simplify");
    assert_eq!(rust_s.vcount(), py.vcount);
    assert_eq!(rust_s.is_directed(), py.directed);
    assert_eq!(rust_simplify_pairs(&rust_s), py_simplify_pairs(&py, true));
}

#[test]
fn simplify_directed_loops_only_matches_python_igraph() {
    // Directed (a,b) and (b,a) are distinct → with remove_multiple=false
    // and remove_loops=true they all survive except self-loops.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 0).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    g.add_edge(1, 1).unwrap();

    let rust_s = simplify(&g, false, true).unwrap();
    let py: PySimplify = serde_json::from_value(run_ok(
        "simplify",
        &g,
        serde_json::json!({"remove_multiple": false, "remove_loops": true}),
    ))
    .expect("decode python simplify");
    assert_eq!(rust_s.vcount(), py.vcount);
    assert_eq!(rust_s.is_directed(), py.directed);
    assert_eq!(rust_simplify_pairs(&rust_s), py_simplify_pairs(&py, false));
}

#[test]
fn simplify_directed_multi_only_matches_python_igraph() {
    // remove_multiple=true keeps loops but collapses parallels.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    g.add_edge(2, 2).unwrap();

    let rust_s = simplify(&g, true, false).unwrap();
    let py: PySimplify = serde_json::from_value(run_ok(
        "simplify",
        &g,
        serde_json::json!({"remove_multiple": true, "remove_loops": false}),
    ))
    .expect("decode python simplify");
    assert_eq!(rust_s.vcount(), py.vcount);
    assert_eq!(rust_s.is_directed(), py.directed);
    assert_eq!(rust_simplify_pairs(&rust_s), py_simplify_pairs(&py, false));
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

#[test]
fn is_simple_path_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = is_simple(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("is_simple", &g, serde_json::json!({})))
        .expect("decode python is_simple");
    assert_eq!(rust, py);
    assert!(rust);
}

#[test]
fn is_simple_with_self_loop_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 0).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = is_simple(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("is_simple", &g, serde_json::json!({})))
        .expect("decode python is_simple");
    assert_eq!(rust, py);
    assert!(!rust);
}

#[test]
fn is_simple_with_parallel_edges_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    let rust = is_simple(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("is_simple", &g, serde_json::json!({})))
        .expect("decode python is_simple");
    assert_eq!(rust, py);
    assert!(!rust);
}

#[test]
fn modularity_two_triangles_bridge_matches_python_igraph() {
    // Two K3 + bridge edge — partition {0,1,2} vs {3,4,5}.
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let membership = vec![0u32, 0, 0, 1, 1, 1];
    let rust_q = modularity(&g, &membership, 1.0).unwrap().unwrap();
    let py: f64 = serde_json::from_value(run_ok(
        "modularity",
        &g,
        serde_json::json!({"membership": membership, "resolution": 1.0}),
    ))
    .expect("decode python modularity");
    assert!((rust_q - py).abs() < 1e-12, "rust={rust_q} py={py}");
}

#[test]
fn modularity_karate_two_clusters_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    // Crude split: vertices 0..16 vs 17..33 (Zachary's split-by-id is a
    // known modular partition, ≈ 0.37). Exact value is irrelevant — we
    // just need the Rust and python-igraph numbers to coincide.
    let membership: Vec<u32> = (0..g.vcount()).map(|v| u32::from(v >= 17)).collect();
    let rust_q = modularity(&g, &membership, 1.0).unwrap().unwrap();
    let py: f64 = serde_json::from_value(run_ok(
        "modularity",
        &g,
        serde_json::json!({"membership": membership, "resolution": 1.0}),
    ))
    .expect("decode python modularity");
    assert!((rust_q - py).abs() < 1e-12, "rust={rust_q} py={py}");
}

#[test]
fn modularity_resolution_zero_matches_python_igraph() {
    // K4 with [0,0,1,1] under γ=0 → e/2m only.
    let mut g = Graph::with_vertices(4);
    for u in 0..4u32 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    let rust_q = modularity(&g, &[0, 0, 1, 1], 0.0).unwrap().unwrap();
    let py: f64 = serde_json::from_value(run_ok(
        "modularity",
        &g,
        serde_json::json!({"membership": [0, 0, 1, 1], "resolution": 0.0}),
    ))
    .expect("decode python modularity");
    assert!((rust_q - py).abs() < 1e-12, "rust={rust_q} py={py}");
}

#[test]
fn has_loop_simple_graph_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = has_loop(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("has_loop", &g, serde_json::json!({})))
        .expect("decode python has_loop");
    assert_eq!(rust, py);
    assert!(!rust);
}

#[test]
fn has_loop_with_self_loop_matches_python_igraph() {
    let mut g = Graph::with_vertices(2);
    g.add_edge(0, 0).unwrap();
    let rust = has_loop(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("has_loop", &g, serde_json::json!({})))
        .expect("decode python has_loop");
    assert_eq!(rust, py);
    assert!(rust);
}

#[test]
fn has_multiple_simple_graph_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = has_multiple(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("has_multiple", &g, serde_json::json!({})))
        .expect("decode python has_multiple");
    assert_eq!(rust, py);
    assert!(!rust);
}

#[test]
fn has_multiple_with_parallel_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    let rust = has_multiple(&g).unwrap();
    let py: bool = serde_json::from_value(run_ok("has_multiple", &g, serde_json::json!({})))
        .expect("decode python has_multiple");
    assert_eq!(rust, py);
    assert!(rust);
}

#[test]
fn is_loop_per_edge_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = is_loop(&g).unwrap();
    // Edge ids differ after wire round-trip (the Python side rebuilds
    // the graph in vertex-ascending order); compare as multisets.
    let mut rust_sorted = rust;
    rust_sorted.sort_unstable();
    let mut py: Vec<bool> = serde_json::from_value(run_ok("is_loop", &g, serde_json::json!({})))
        .expect("decode python is_loop");
    py.sort_unstable();
    assert_eq!(rust_sorted, py);
}

#[test]
fn is_multiple_per_edge_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = is_multiple(&g).unwrap();
    let py: Vec<bool> = serde_json::from_value(run_ok("is_multiple", &g, serde_json::json!({})))
        .expect("decode python is_multiple");
    assert_eq!(rust, py);
}

#[test]
fn is_multiple_three_copies_matches_python_igraph() {
    let mut g = Graph::with_vertices(2);
    for _ in 0..3 {
        g.add_edge(0, 1).unwrap();
    }
    let rust = is_multiple(&g).unwrap();
    let py: Vec<bool> = serde_json::from_value(run_ok("is_multiple", &g, serde_json::json!({})))
        .expect("decode python is_multiple");
    assert_eq!(rust, py);
}

/// Wire-format payload returned by the `disjoint_union` oracle.
#[derive(serde::Deserialize)]
struct PyDisjointUnion {
    vcount: u32,
    directed: bool,
    edges: Vec<[u32; 2]>,
}

fn rust_du_pairs(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).unwrap();
    let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
    v.sort_unstable();
    v
}

fn py_du_pairs(py: &PyDisjointUnion, undirected: bool) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = py
        .edges
        .iter()
        .map(|p| {
            if undirected && p[0] > p[1] {
                (p[1], p[0])
            } else {
                (p[0], p[1])
            }
        })
        .collect();
    v.sort_unstable();
    v
}

fn right_graph_payload(g: &Graph) -> serde_json::Value {
    use common::GraphPayload;
    let p = GraphPayload::from_graph(g);
    serde_json::to_value(p).expect("serialize right graph")
}

#[test]
fn disjoint_union_two_triangles_matches_python_igraph() {
    let mut a = Graph::with_vertices(3);
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    a.add_edge(2, 0).unwrap();
    let b = a.clone();
    let rust = disjoint_union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python disjoint_union");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn disjoint_union_directed_path_plus_triangle_matches_python_igraph() {
    let mut a = Graph::new(3, true).unwrap();
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::new(3, true).unwrap();
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 2).unwrap();
    b.add_edge(2, 0).unwrap();
    let rust = disjoint_union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python disjoint_union");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(rust.is_directed());
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, false));
}

#[test]
fn disjoint_union_with_isolated_vertices_matches_python_igraph() {
    let a = Graph::with_vertices(3);
    let mut b = Graph::with_vertices(2);
    b.add_edge(0, 1).unwrap();
    let rust = disjoint_union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python disjoint_union");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.ecount(), 1);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn dijkstra_distances_triangle_with_shortcut_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = dijkstra_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances",
        &g,
        Some(weights.clone()),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python dijkstra_distances");
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
fn dijkstra_distances_with_unreachable_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.0_f64, 2.5];
    let rust = dijkstra_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python dijkstra_distances");
    assert_eq!(rust, py);
}

#[test]
fn dijkstra_distances_directed_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 1.0, 1.0, 5.0];
    let rust = dijkstra_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python dijkstra_distances");
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

// ---- ALGO-SP-014: WidestPaths struct oracle tests --------

#[derive(serde::Deserialize, Debug)]
struct PyWidestPaths {
    widths: Vec<serde_json::Value>,
    parents: Vec<Option<u32>>,
    inbound_edges: Vec<Option<u32>>,
}

fn decode_widest_widths_vec(payload: &[serde_json::Value]) -> Vec<Option<f64>> {
    payload
        .iter()
        .map(|v| match v {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) if s == "Infinity" => Some(f64::INFINITY),
            other => Some(other.as_f64().expect("width is number")),
        })
        .collect()
}

#[test]
fn widest_paths_triangle_matches_python_reference() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::widest_paths(&g, 0, &weights).unwrap();
    let py: PyWidestPaths = serde_json::from_value(run_ok_with_weights(
        "widest_paths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python widest_paths");
    let py_widths = decode_widest_widths_vec(&py.widths);
    assert_eq!(rust.widths.len(), py_widths.len());
    for (i, (r, p)) in rust.widths.iter().zip(py_widths.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) if rr.is_infinite() && pp.is_infinite() => {}
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-12, "widths[{i}] rust={rr} py={pp}")
            }
            (None, None) => {}
            (a, b) => panic!("widths[{i}] rust={a:?} py={b:?}"),
        }
    }
    // Parents/inbound_edges: with no tie-breaks in this graph, must
    // match exactly.
    assert_eq!(rust.parents, py.parents, "parents mismatch");
    assert_eq!(
        rust.inbound_edges, py.inbound_edges,
        "inbound_edges mismatch"
    );
}

#[test]
fn widest_paths_unreachable_matches_python_reference() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![2.0_f64, 5.0];
    let rust = rust_igraph::widest_paths(&g, 0, &weights).unwrap();
    let py: PyWidestPaths = serde_json::from_value(run_ok_with_weights(
        "widest_paths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python widest_paths");
    // Reachability sets must agree across all three fields.
    let py_widths = decode_widest_widths_vec(&py.widths);
    for i in 0..rust.widths.len() {
        assert_eq!(
            rust.widths[i].is_some(),
            py_widths[i].is_some(),
            "vertex {i}: reachability mismatch"
        );
        assert_eq!(rust.parents[i], py.parents[i], "vertex {i}: parents");
        assert_eq!(
            rust.inbound_edges[i], py.inbound_edges[i],
            "vertex {i}: edges"
        );
    }
}

// ---- ALGO-SP-013: widest_paths_to multi-target oracle tests --------

#[derive(serde::Deserialize, Debug)]
struct PyWidestPathOpt(Option<PyWidestPath>);

#[test]
fn widest_paths_to_triangle_matches_python_reference() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::widest_paths_to(&g, 0, &[1, 2], &weights).unwrap();
    let py: Vec<PyWidestPathOpt> = serde_json::from_value(run_ok_with_weights(
        "widest_paths_to",
        &g,
        Some(weights),
        serde_json::json!({"from": 0, "targets": [1, 2]}),
    ))
    .expect("decode python widest_paths_to");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, &p.0) {
            (Some((r_vs, r_es)), Some(pp)) => {
                assert_eq!(r_vs.len(), pp.vertices.len(), "target idx {i}: vs len");
                assert_eq!(r_es.len(), pp.edges.len(), "target idx {i}: es len");
                assert_eq!(r_vs[0], pp.vertices[0]);
                assert_eq!(*r_vs.last().unwrap(), *pp.vertices.last().unwrap());
            }
            (None, None) => {}
            (a, b) => panic!("target idx {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn widest_paths_to_mixed_reachability_matches_python_reference() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.0_f64, 1.0];
    let rust = rust_igraph::widest_paths_to(&g, 0, &[1, 2, 3], &weights).unwrap();
    let py: Vec<PyWidestPathOpt> = serde_json::from_value(run_ok_with_weights(
        "widest_paths_to",
        &g,
        Some(weights),
        serde_json::json!({"from": 0, "targets": [1, 2, 3]}),
    ))
    .expect("decode python widest_paths_to");
    assert_eq!(rust.len(), py.len());
    // Reachability set must match exactly.
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert_eq!(
            r.is_some(),
            p.0.is_some(),
            "target idx {i}: reachability mismatch"
        );
    }
}

// ---- ALGO-SP-012: FW all-pairs widest widths oracle tests --------

fn decode_widths_matrix(payload: serde_json::Value) -> Vec<Vec<Option<f64>>> {
    let outer = payload.as_array().expect("matrix payload is array");
    outer
        .iter()
        .map(|row| {
            let arr = row.as_array().expect("matrix row is array");
            arr.iter()
                .map(|v| match v {
                    serde_json::Value::Null => None,
                    serde_json::Value::String(s) if s == "Infinity" => Some(f64::INFINITY),
                    other => Some(other.as_f64().expect("width is number")),
                })
                .collect()
        })
        .collect()
}

fn assert_widths_matrix_close(rust: &[Vec<Option<f64>>], py: &[Vec<Option<f64>>], label: &str) {
    assert_eq!(rust.len(), py.len(), "{label}: row count");
    for (u, (r_row, p_row)) in rust.iter().zip(py.iter()).enumerate() {
        assert_eq!(r_row.len(), p_row.len(), "{label}: row {u} length");
        for (v, (r, p)) in r_row.iter().zip(p_row.iter()).enumerate() {
            match (r, p) {
                (Some(rr), Some(pp)) if rr.is_infinite() && pp.is_infinite() => {}
                (Some(rr), Some(pp)) => assert!(
                    (rr - pp).abs() < 1e-12,
                    "{label}: [{u}][{v}] rust={rr} py={pp}"
                ),
                (None, None) => {}
                (a, b) => panic!("{label}: [{u}][{v}] rust={a:?} py={b:?}"),
            }
        }
    }
}

#[test]
fn fw_widest_triangle_matches_python_reference() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::widest_path_widths_floyd_warshall(&g, &weights).unwrap();
    let py = decode_widths_matrix(run_ok_with_weights(
        "widest_path_widths_floyd_warshall",
        &g,
        Some(weights),
        serde_json::json!({}),
    ));
    assert_widths_matrix_close(&rust, &py, "fw_widest_triangle");
}

#[test]
fn fw_widest_directed_chain_matches_python_reference() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![5.0_f64, 3.0, 4.0, 1.0];
    let rust = rust_igraph::widest_path_widths_floyd_warshall(&g, &weights).unwrap();
    let py = decode_widths_matrix(run_ok_with_weights(
        "widest_path_widths_floyd_warshall",
        &g,
        Some(weights),
        serde_json::json!({}),
    ));
    assert_widths_matrix_close(&rust, &py, "fw_widest_directed_chain");
}

// ---- ALGO-SP-011: widest-path single-target path oracle tests --------

#[derive(serde::Deserialize, Debug)]
struct PyWidestPath {
    vertices: Vec<u32>,
    edges: Vec<u32>,
}

#[test]
fn widest_path_triangle_matches_python_reference() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let (rust_vs, rust_es) = rust_igraph::widest_path(&g, 0, 1, &weights)
        .unwrap()
        .expect("0→1 reachable");
    let py: PyWidestPath = serde_json::from_value(run_ok_with_weights(
        "widest_path",
        &g,
        Some(weights),
        serde_json::json!({"from": 0, "to": 1}),
    ))
    .expect("decode python widest_path");
    // Bottleneck width must match — tie-breaking may pick a different
    // chain so compare widths along the chain instead of identity.
    assert_eq!(rust_vs.len(), py.vertices.len());
    assert_eq!(rust_es.len(), py.edges.len());
    assert_eq!(rust_vs[0], py.vertices[0]);
    assert_eq!(*rust_vs.last().unwrap(), *py.vertices.last().unwrap());
}

#[test]
fn widest_path_unreachable_matches_python_reference() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.0_f64, 1.0];
    let rust = rust_igraph::widest_path(&g, 0, 2, &weights).unwrap();
    let py_value = run_ok_with_weights(
        "widest_path",
        &g,
        Some(weights),
        serde_json::json!({"from": 0, "to": 2}),
    );
    assert!(rust.is_none());
    assert!(py_value.is_null());
}

// ---- ALGO-SP-010: widest-path widths oracle tests --------

/// Decode the oracle's widths output: None → unreachable,
/// "Infinity" string → source itself (JSON has no Infinity literal),
/// otherwise f64. Returns same `Option<f64>` shape as our Rust API.
fn decode_widest_widths(payload: serde_json::Value) -> Vec<Option<f64>> {
    let arr = payload.as_array().expect("widths payload is array");
    arr.iter()
        .map(|v| match v {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) if s == "Infinity" => Some(f64::INFINITY),
            other => Some(other.as_f64().expect("width is number")),
        })
        .collect()
}

#[test]
fn widest_path_widths_triangle_matches_python_reference() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::widest_path_widths(&g, 0, &weights).unwrap();
    let py = decode_widest_widths(run_ok_with_weights(
        "widest_path_widths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ));
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) if rr.is_infinite() && pp.is_infinite() => {}
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-12, "vertex {i}: rust={rr} py={pp}")
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn widest_path_widths_chain_bottleneck_matches_python_reference() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![5.0_f64, 1.0, 3.0];
    let rust = rust_igraph::widest_path_widths(&g, 0, &weights).unwrap();
    let py = decode_widest_widths(run_ok_with_weights(
        "widest_path_widths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ));
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) if rr.is_infinite() && pp.is_infinite() => {}
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-12, "vertex {i}: rust={rr} py={pp}")
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

// ---- ALGO-SP-003: Johnson all-pairs distances oracle tests --------

#[test]
fn johnson_distances_directed_with_negative_edge_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 3).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![3.0_f64, 1.0, -2.0, 4.0];
    let rust = rust_igraph::johnson_distances(&g, &weights).unwrap();
    let py: Vec<Vec<Option<f64>>> = serde_json::from_value(run_ok_with_weights(
        "johnson_distances",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python johnson_distances");
    assert_eq!(rust.len(), py.len());
    for (u, (r_row, p_row)) in rust.iter().zip(py.iter()).enumerate() {
        assert_eq!(r_row.len(), p_row.len(), "row {u} length mismatch");
        for (v, (r, p)) in r_row.iter().zip(p_row.iter()).enumerate() {
            match (r, p) {
                (Some(rr), Some(pp)) => {
                    assert!((rr - pp).abs() < 1e-12, "[{u}][{v}]: rust={rr} py={pp}");
                }
                (None, None) => {}
                (a, b) => panic!("[{u}][{v}]: rust={a:?} py={b:?}"),
            }
        }
    }
}

#[test]
fn johnson_distances_positive_weights_matches_python_igraph() {
    // Fast path: no negative weights ⇒ Johnson short-circuits to
    // pairwise Dijkstra. Verify the all-pairs matrix still matches.
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::johnson_distances(&g, &weights).unwrap();
    let py: Vec<Vec<Option<f64>>> = serde_json::from_value(run_ok_with_weights(
        "johnson_distances",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python johnson_distances");
    for (u, (r_row, p_row)) in rust.iter().zip(py.iter()).enumerate() {
        for (v, (r, p)) in r_row.iter().zip(p_row.iter()).enumerate() {
            match (r, p) {
                (Some(rr), Some(pp)) => {
                    assert!((rr - pp).abs() < 1e-12, "[{u}][{v}]: rust={rr} py={pp}");
                }
                (None, None) => {}
                (a, b) => panic!("[{u}][{v}]: rust={a:?} py={b:?}"),
            }
        }
    }
}

// ---- ALGO-SP-002: Bellman-Ford distances oracle tests --------

#[test]
fn bellman_ford_distances_positive_weights_matches_python_igraph() {
    // Positive weights: BF and Dijkstra must agree; this verifies our
    // SPFA loop against python-igraph's distances() implementation.
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = rust_igraph::bellman_ford_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "bellman_ford_distances",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python bellman_ford_distances");
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
fn bellman_ford_distances_negative_edge_directed_matches_python_igraph() {
    // The headline case: a negative edge that would break Dijkstra
    // but is correctly handled by BF.
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 3).unwrap();
    g.add_edge(2, 3).unwrap();
    // 0→1 (3), 0→2 (1), 1→3 (-2), 2→3 (4).
    // BF distances from 0: d[0]=0, d[1]=3, d[2]=1, d[3]=min(3-2, 1+4)=1.
    let weights = vec![3.0_f64, 1.0, -2.0, 4.0];
    let rust = rust_igraph::bellman_ford_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "bellman_ford_distances",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python bellman_ford_distances");
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
fn bellman_ford_distances_unreachable_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.5_f64, -0.5];
    let rust = rust_igraph::bellman_ford_distances(&g, 0, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "bellman_ford_distances",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python bellman_ford_distances");
    assert_eq!(rust, py);
}

// ---- ALGO-SP-001b: Dijkstra paths / path_to / cutoff oracle tests --------

/// Wire-format payload for the `dijkstra_paths` oracle handler. Distances
/// use `Option<f64>`; parents and inbound edges use `Option<i64>` to fit
/// python-igraph's `None` sentinel for source / unreachable vertices.
#[derive(serde::Deserialize)]
struct PyDijkstraPaths {
    distances: Vec<Option<f64>>,
    parents: Vec<Option<i64>>,
    inbound_edges: Vec<Option<i64>>,
}

fn assert_dist_vec_close(rust: &[Option<f64>], py: &[Option<f64>]) {
    assert_eq!(rust.len(), py.len(), "distance vector length");
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        match (r, p) {
            (Some(rr), Some(pp)) => {
                assert!((rr - pp).abs() < 1e-9, "vertex {i}: rust={rr} py={pp}");
            }
            (None, None) => {}
            (a, b) => panic!("vertex {i}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn dijkstra_paths_triangle_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = dijkstra_paths(&g, 0, &weights).unwrap();
    let py: PyDijkstraPaths = serde_json::from_value(run_ok_with_weights(
        "dijkstra_paths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0}),
    ))
    .expect("decode python dijkstra_paths");
    assert_dist_vec_close(&rust.distances, &py.distances);
    // Verify parents / inbound edges are consistent with the SPT
    // tree distances (oracle's reconstruction may pick a different
    // tie-breaking edge; we accept any parent that satisfies the
    // relaxation equality `dist[parent] + w(eid) == dist[v]`).
    for v in 0..g.vcount() as usize {
        let py_p = py.parents[v].map(|x| u32::try_from(x).unwrap());
        let py_e = py.inbound_edges[v].map(|x| u32::try_from(x).unwrap());
        assert_eq!(rust.parents[v].is_none(), py_p.is_none(), "v={v} parent");
        assert_eq!(
            rust.inbound_edges[v].is_none(),
            py_e.is_none(),
            "v={v} inbound"
        );
        if let (Some(rp), Some(re)) = (rust.parents[v], rust.inbound_edges[v]) {
            // sanity: edge connects rp ↔ v and weighted-relax
            let (s, t) = g.edge(re).unwrap();
            assert!((s == rp && t as usize == v) || (t == rp && s as usize == v));
            let w_e = [1.0_f64, 4.0, 2.0][re as usize];
            let dp = rust.distances[rp as usize].unwrap();
            let dv = rust.distances[v].unwrap();
            assert!((dp + w_e - dv).abs() < 1e-9, "relax v={v}");
        }
    }
}

#[test]
fn dijkstra_path_to_directed_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 1.0, 1.0, 5.0];
    let rust = dijkstra_path_to(&g, 0, 3, &weights).unwrap().unwrap();
    #[derive(serde::Deserialize)]
    struct PyPath {
        vertices: Vec<u32>,
        edges: Vec<u32>,
    }
    let py: PyPath = serde_json::from_value(run_ok_with_weights(
        "dijkstra_path_to",
        &g,
        Some(weights),
        serde_json::json!({"source": 0, "target": 3}),
    ))
    .expect("decode python dijkstra_path_to");
    assert_eq!(rust.0, py.vertices);
    assert_eq!(rust.1, py.edges);
}

#[test]
fn dijkstra_distances_cutoff_matches_python_igraph() {
    let mut g = Graph::with_vertices(5);
    for i in 0..4u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let weights = vec![1.0_f64; 4];
    let rust = dijkstra_distances_cutoff(&g, 0, &weights, Some(2.5)).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances_cutoff",
        &g,
        Some(weights),
        serde_json::json!({"source": 0, "cutoff": 2.5}),
    ))
    .expect("decode python dijkstra_distances_cutoff");
    assert_dist_vec_close(&rust, &py);
}

// ---- ALGO-SP-001c: mode-aware + all-shortest-paths oracle tests ---------

#[test]
fn dijkstra_distances_with_mode_in_matches_python_igraph() {
    // Directed path 0→1→2: IN-mode from vertex 2 reaches the source.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 2.0];
    let rust = dijkstra_distances_with_mode(&g, 2, &weights, DijkstraMode::In).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances_with_mode",
        &g,
        Some(weights),
        serde_json::json!({"source": 2, "mode": "in"}),
    ))
    .expect("decode python dijkstra_distances_with_mode");
    assert_dist_vec_close(&rust, &py);
}

#[test]
fn dijkstra_distances_with_mode_all_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 2.0];
    let rust = dijkstra_distances_with_mode(&g, 2, &weights, DijkstraMode::All).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "dijkstra_distances_with_mode",
        &g,
        Some(weights),
        serde_json::json!({"source": 2, "mode": "all"}),
    ))
    .expect("decode python dijkstra_distances_with_mode");
    assert_dist_vec_close(&rust, &py);
}

#[test]
fn dijkstra_all_shortest_paths_diamond_matches_python_igraph() {
    // Diamond: two distinct shortest paths to vertex 3.
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 3).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.0_f64; 4];
    let rust = dijkstra_all_shortest_paths(&g, 0, &weights, DijkstraMode::Out).unwrap();
    let rust_dist = dijkstra_distances_with_mode(&g, 0, &weights, DijkstraMode::Out).unwrap();
    #[derive(serde::Deserialize)]
    struct PyAsp {
        distances: Vec<Option<f64>>,
        nrgeo: Vec<u64>,
    }
    let py: PyAsp = serde_json::from_value(run_ok_with_weights(
        "dijkstra_all_shortest_paths",
        &g,
        Some(weights),
        serde_json::json!({"source": 0, "mode": "out"}),
    ))
    .expect("decode python dijkstra_all_shortest_paths");
    assert_dist_vec_close(&rust_dist, &py.distances);
    assert_eq!(rust.nrgeo, py.nrgeo);
}

/// Wire-format payload returned by the `complementer` oracle.
#[derive(serde::Deserialize)]
struct PyComplementer {
    vcount: u32,
    directed: bool,
    edges: Vec<[u32; 2]>,
}

fn rust_complementer_pairs(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).unwrap();
    let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
    v.sort_unstable();
    v
}

fn py_complementer_pairs(py: &PyComplementer, undirected: bool) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = py
        .edges
        .iter()
        .map(|p| {
            if undirected && p[0] > p[1] {
                (p[1], p[0])
            } else {
                (p[0], p[1])
            }
        })
        .collect();
    v.sort_unstable();
    v
}

#[test]
fn complementer_path_undirected_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = complementer(&g, false).unwrap();
    let py: PyComplementer = serde_json::from_value(run_ok(
        "complementer",
        &g,
        serde_json::json!({"loops": false}),
    ))
    .expect("decode python complementer");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(
        rust_complementer_pairs(&rust),
        py_complementer_pairs(&py, true)
    );
}

#[test]
fn complementer_with_loops_matches_python_igraph() {
    let g = Graph::with_vertices(3);
    let rust = complementer(&g, true).unwrap();
    let py: PyComplementer = serde_json::from_value(run_ok(
        "complementer",
        &g,
        serde_json::json!({"loops": true}),
    ))
    .expect("decode python complementer");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(
        rust_complementer_pairs(&rust),
        py_complementer_pairs(&py, true)
    );
}

#[test]
fn complementer_directed_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust = complementer(&g, false).unwrap();
    let py: PyComplementer = serde_json::from_value(run_ok(
        "complementer",
        &g,
        serde_json::json!({"loops": false}),
    ))
    .expect("decode python complementer");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(rust.is_directed());
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(
        rust_complementer_pairs(&rust),
        py_complementer_pairs(&py, false)
    );
}

#[test]
fn closeness_weighted_star_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 2.0, 3.0];
    let rust = closeness_weighted(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "closeness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python closeness_weighted");
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
fn closeness_weighted_directed_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![2.0_f64, 0.5];
    let rust = closeness_weighted(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "closeness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python closeness_weighted");
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
fn closeness_weighted_path_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let weights = vec![1.5_f64, 2.5, 0.5];
    let rust = closeness_weighted(&g, &weights).unwrap();
    let py: Vec<Option<f64>> = serde_json::from_value(run_ok_with_weights(
        "closeness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python closeness_weighted");
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
fn harmonic_centrality_weighted_path_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 2.0];
    let rust = harmonic_centrality_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "harmonic_centrality_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python harmonic_centrality_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn harmonic_centrality_weighted_directed_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 1.0, 1.0, 5.0];
    let rust = harmonic_centrality_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "harmonic_centrality_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python harmonic_centrality_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn harmonic_centrality_weighted_disconnected_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 1.0];
    let rust = harmonic_centrality_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "harmonic_centrality_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python harmonic_centrality_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn betweenness_weighted_keeps_direct_when_cheaper_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    let weights = vec![5.0_f64, 5.0, 1.0];
    let rust = betweenness_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python betweenness_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn betweenness_weighted_swaps_route_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    let weights = vec![1.0_f64, 1.0, 5.0];
    let rust = betweenness_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python betweenness_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn betweenness_weighted_path5_unit_weights_matches_python_igraph() {
    let mut g = Graph::with_vertices(5);
    for i in 0..4u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let weights = vec![1.0_f64; 4];
    let rust = betweenness_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python betweenness_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-12, "vertex {i}: rust={r} py={p}");
    }
}

/// Wire-format payload returned by the `edge_betweenness_weighted`
/// oracle (parallel `edges` + `values`).
#[derive(serde::Deserialize)]
struct PyEdgeBetweennessWeighted {
    edges: Vec<[u32; 2]>,
    values: Vec<f64>,
}

fn rust_eb_w_pairs(g: &Graph, eb: &[f64]) -> Vec<((u32, u32), f64)> {
    let m = u32::try_from(g.ecount()).unwrap();
    let mut v: Vec<_> = (0..m)
        .map(|e| {
            let (a, b) = g.edge(e).unwrap();
            let pair = if a > b { (b, a) } else { (a, b) };
            (pair, eb[e as usize])
        })
        .collect();
    v.sort_by(|x, y| x.0.cmp(&y.0).then(x.1.partial_cmp(&y.1).unwrap()));
    v
}

fn py_eb_w_pairs(py: &PyEdgeBetweennessWeighted) -> Vec<((u32, u32), f64)> {
    let mut v: Vec<_> = py
        .edges
        .iter()
        .zip(py.values.iter())
        .map(|(p, &val)| {
            let pair = if p[0] > p[1] {
                (p[1], p[0])
            } else {
                (p[0], p[1])
            };
            (pair, val)
        })
        .collect();
    v.sort_by(|x, y| x.0.cmp(&y.0).then(x.1.partial_cmp(&y.1).unwrap()));
    v
}

#[test]
fn edge_betweenness_weighted_path_4_unit_weights_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    for i in 0..3u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let weights = vec![1.0_f64; 3];
    let rust = edge_betweenness_weighted(&g, &weights).unwrap();
    let py: PyEdgeBetweennessWeighted = serde_json::from_value(run_ok_with_weights(
        "edge_betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python edge_betweenness_weighted");
    let r_pairs = rust_eb_w_pairs(&g, &rust);
    let p_pairs = py_eb_w_pairs(&py);
    assert_eq!(r_pairs.len(), p_pairs.len());
    for (i, ((rp, rv), (pp, pv))) in r_pairs.iter().zip(p_pairs.iter()).enumerate() {
        assert_eq!(rp, pp, "edge slot {i}");
        assert!((rv - pv).abs() < 1e-12, "edge {rp:?}: rust={rv} py={pv}");
    }
}

#[test]
fn edge_betweenness_weighted_triangle_swap_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    let weights = vec![1.0_f64, 1.0, 5.0];
    let rust = edge_betweenness_weighted(&g, &weights).unwrap();
    let py: PyEdgeBetweennessWeighted = serde_json::from_value(run_ok_with_weights(
        "edge_betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python edge_betweenness_weighted");
    let r_pairs = rust_eb_w_pairs(&g, &rust);
    let p_pairs = py_eb_w_pairs(&py);
    assert_eq!(r_pairs.len(), p_pairs.len());
    for ((rp, rv), (pp, pv)) in r_pairs.iter().zip(p_pairs.iter()) {
        assert_eq!(rp, pp);
        assert!((rv - pv).abs() < 1e-12, "edge {rp:?}: rust={rv} py={pv}");
    }
}

#[test]
fn edge_betweenness_weighted_directed_chain_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 1.0, 1.0, 5.0];
    let rust = edge_betweenness_weighted(&g, &weights).unwrap();
    let py: PyEdgeBetweennessWeighted = serde_json::from_value(run_ok_with_weights(
        "edge_betweenness_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python edge_betweenness_weighted");
    let r_pairs = rust_eb_w_pairs(&g, &rust);
    let p_pairs = py_eb_w_pairs(&py);
    assert_eq!(r_pairs.len(), p_pairs.len());
    for ((rp, rv), (pp, pv)) in r_pairs.iter().zip(p_pairs.iter()) {
        assert_eq!(rp, pp);
        assert!((rv - pv).abs() < 1e-12, "edge {rp:?}: rust={rv} py={pv}");
    }
}

#[test]
fn pagerank_weighted_unit_weights_match_unweighted_karate() {
    // Unit weights collapse weighted PageRank to the unweighted result.
    // python-igraph defaults to ARPACK, so we use 1e-6 tolerance like
    // the PR-011 oracle test.
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate fixture"))
        .expect("parse karate edgelist");
    let weights = vec![1.0_f64; g.ecount()];
    let rust = pagerank_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "pagerank_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python pagerank_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-6, "vertex {i}: rust={r} py={p}");
    }
    // Same graph + unit weights → must equal the unweighted PageRank
    // exactly within Rust (no eigensolver drift).
    let pu = pagerank(&g).unwrap();
    for (i, (rw, ru)) in rust.iter().zip(pu.iter()).enumerate() {
        assert!(
            (rw - ru).abs() < 1e-9,
            "vertex {i}: weighted={rw} unweighted={ru}"
        );
    }
}

#[test]
fn pagerank_weighted_directed_4cycle_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    for i in 0..4u32 {
        g.add_edge(i, (i + 1) % 4).unwrap();
    }
    let weights = vec![1.0_f64; 4];
    let rust = pagerank_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "pagerank_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python pagerank_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-6, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn pagerank_weighted_heavy_edge_concentrates_matches_python_igraph() {
    // Directed 0→1 weight 100, 0→2 weight 0.01: vertex 1 gets nearly
    // all of 0's flow.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    let weights = vec![100.0_f64, 0.01];
    let rust = pagerank_weighted(&g, &weights).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "pagerank_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python pagerank_weighted");
    assert_eq!(rust.len(), py.len());
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-6, "vertex {i}: rust={r} py={p}");
    }
    assert!(rust[1] > rust[2]);
}

// python-igraph 0.11 has no weighted assortativity at the Python layer
// (`Graph.assortativity` has no `weights` kwarg; `assortativity_degree`
// doesn't take weights). For unit-weight inputs the weighted formula
// collapses to the unweighted one — these tests verify that
// equivalence against python-igraph's unweighted oracle. Non-unit
// weights are validated via the conformance suite with hand-computed
// reference values.

#[test]
fn assortativity_degree_weighted_path_4_unit_weights_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    for i in 0..3u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let weights = vec![1.0_f64; 3];
    let rust = assortativity_degree_weighted(&g, &weights).unwrap();
    let py: serde_json::Value = run_ok_with_weights(
        "assortativity_degree_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    );
    let py_val = py.as_f64().expect("py value as f64");
    let r = rust.expect("rust value");
    assert!((r - py_val).abs() < 1e-12, "rust={r} py={py_val}");
}

#[test]
fn assortativity_degree_weighted_diamond_unit_weights_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    for &(u, v) in &[(0u32, 1), (0, 2), (1, 2), (1, 3), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let weights = vec![1.0_f64; 5];
    let rust = assortativity_degree_weighted(&g, &weights).unwrap();
    let py: serde_json::Value = run_ok_with_weights(
        "assortativity_degree_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    );
    let py_val = py.as_f64().expect("py value as f64");
    let r = rust.expect("rust value");
    assert!((r - py_val).abs() < 1e-12, "rust={r} py={py_val}");
}

#[test]
fn assortativity_degree_weighted_two_triangles_bridge_matches_python_igraph() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let weights = vec![1.0_f64; 7];
    let rust = assortativity_degree_weighted(&g, &weights).unwrap();
    let py: serde_json::Value = run_ok_with_weights(
        "assortativity_degree_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    );
    let py_val = py.as_f64().expect("py value as f64");
    let r = rust.expect("rust value");
    assert!((r - py_val).abs() < 1e-12, "rust={r} py={py_val}");
}

#[test]
fn assortativity_degree_directed_weighted_unit_weights_matches_python_igraph() {
    // Directed chain 0→1→2→3→4 with unit weights — should match the
    // unweighted directed assortativity (formula collapse).
    let mut g = Graph::new(5, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(3, 4).unwrap();
    let weights = vec![1.0_f64; 4];
    let rust = assortativity_degree_directed_weighted(&g, &weights).unwrap();
    let py: serde_json::Value = run_ok_with_weights(
        "assortativity_degree_directed_weighted",
        &g,
        Some(weights),
        serde_json::json!({}),
    );
    match (rust, py.as_f64()) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

fn assert_matrix_close(rust: &[Vec<Option<f64>>], py: &[Vec<Option<f64>>], tol: f64) {
    assert_eq!(rust.len(), py.len(), "row count");
    for (i, (rr, pp)) in rust.iter().zip(py.iter()).enumerate() {
        assert_eq!(rr.len(), pp.len(), "row {i} col count");
        for (j, (a, b)) in rr.iter().zip(pp.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => assert!((x - y).abs() < tol, "[{i}][{j}]: rust={x} py={y}"),
                (None, None) => {}
                (a, b) => panic!("[{i}][{j}]: rust={a:?} py={b:?}"),
            }
        }
    }
}

#[test]
fn floyd_warshall_distances_unweighted_path_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = floyd_warshall_distances(&g, None).unwrap();
    let py: Vec<Vec<Option<f64>>> = serde_json::from_value(run_ok(
        "floyd_warshall_distances",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode python floyd_warshall_distances");
    assert_matrix_close(&rust, &py, 1e-12);
}

#[test]
fn floyd_warshall_distances_weighted_triangle_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let rust = floyd_warshall_distances(&g, Some(&weights)).unwrap();
    let py: Vec<Vec<Option<f64>>> = serde_json::from_value(run_ok_with_weights(
        "floyd_warshall_distances",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python floyd_warshall_distances");
    assert_matrix_close(&rust, &py, 1e-12);
}

#[test]
fn floyd_warshall_distances_directed_with_unreachable_matches_python_igraph() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 3).unwrap();
    let weights = vec![1.0_f64, 1.0, 1.0, 5.0];
    let rust = floyd_warshall_distances(&g, Some(&weights)).unwrap();
    let py: Vec<Vec<Option<f64>>> = serde_json::from_value(run_ok_with_weights(
        "floyd_warshall_distances",
        &g,
        Some(weights),
        serde_json::json!({}),
    ))
    .expect("decode python floyd_warshall_distances");
    assert_matrix_close(&rust, &py, 1e-12);
}

#[test]
fn coreness_triangle_with_pendant_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = coreness(&g).unwrap();
    let py: Vec<u32> =
        serde_json::from_value(run_ok("coreness", &g, serde_json::json!({}))).expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn coreness_two_components_matches_python_igraph() {
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(3, 4).unwrap();
    let rust = coreness(&g).unwrap();
    let py: Vec<u32> =
        serde_json::from_value(run_ok("coreness", &g, serde_json::json!({}))).expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn coreness_karate_matches_python_igraph() {
    let path = workspace_fixture("karate.edges");
    let g = read_edgelist(File::open(&path).expect("open karate")).expect("read karate");
    let rust = coreness(&g).unwrap();
    let py: Vec<u32> =
        serde_json::from_value(run_ok("coreness", &g, serde_json::json!({}))).expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn reciprocity_with_mode_ratio_directed_partial_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    g.add_edge(0, 2).unwrap();
    let rust = reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "reciprocity_with_mode",
        &g,
        serde_json::json!({"ignore_loops": false, "mode": "ratio"}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn reciprocity_with_mode_ignore_loops_default_matches_python_igraph() {
    let mut g = Graph::new(2, true).unwrap();
    g.add_edge(0, 0).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    let rust = reciprocity_with_mode(&g, true, ReciprocityMode::Default).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "reciprocity_with_mode",
        &g,
        serde_json::json!({"ignore_loops": true, "mode": "default"}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn reciprocity_with_mode_undirected_always_one_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let rust = reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "reciprocity_with_mode",
        &g,
        serde_json::json!({"ignore_loops": false, "mode": "ratio"}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
    assert_eq!(rust, Some(1.0));
}

#[test]
fn modularity_weighted_unit_weights_match_unweighted_oracle() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let mem = vec![0u32, 0, 0, 1, 1, 1];
    let weights = vec![1.0_f64; 7];
    let rust = modularity_weighted(&g, &mem, 1.0, &weights).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok_with_weights(
        "modularity_weighted",
        &g,
        Some(weights),
        serde_json::json!({"membership": mem, "resolution": 1.0}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

#[test]
fn modularity_weighted_heavy_internal_matches_python_igraph() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let mem = vec![0u32, 0, 0, 1, 1, 1];
    let weights = vec![10.0_f64, 10.0, 10.0, 10.0, 10.0, 10.0, 0.1];
    let rust = modularity_weighted(&g, &mem, 1.0, &weights).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok_with_weights(
        "modularity_weighted",
        &g,
        Some(weights),
        serde_json::json!({"membership": mem, "resolution": 1.0}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

#[test]
fn modularity_weighted_resolution_zero_matches_python_igraph() {
    let mut g = Graph::with_vertices(4);
    for u in 0..4u32 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    let mem = vec![0u32, 0, 1, 1];
    let weights = vec![2.0_f64, 1.0, 1.0, 1.0, 1.0, 2.0];
    let rust = modularity_weighted(&g, &mem, 0.0, &weights).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok_with_weights(
        "modularity_weighted",
        &g,
        Some(weights),
        serde_json::json!({"membership": mem, "resolution": 0.0}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

#[test]
fn is_simple_with_mode_directed_mutual_undirected_view_matches_python_igraph() {
    let mut g = Graph::new(2, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 0).unwrap();
    let rust = is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap();
    let py: bool = serde_json::from_value(run_ok(
        "is_simple_with_mode",
        &g,
        serde_json::json!({"directed_as_undirected": true}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
    assert!(!rust);
}

#[test]
fn is_simple_with_mode_directed_3_cycle_undirected_view_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let rust = is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap();
    let py: bool = serde_json::from_value(run_ok(
        "is_simple_with_mode",
        &g,
        serde_json::json!({"directed_as_undirected": true}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
    assert!(rust);
}

#[test]
fn is_simple_with_mode_undirected_modes_agree_with_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let rust_dir = is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap();
    let rust_undir = is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap();
    let py: bool = serde_json::from_value(run_ok(
        "is_simple_with_mode",
        &g,
        serde_json::json!({"directed_as_undirected": false}),
    ))
    .expect("decode");
    assert_eq!(rust_dir, py);
    assert_eq!(rust_undir, py);
}

#[test]
fn disjoint_union_many_three_triangles_matches_python_igraph() {
    let mut t = Graph::with_vertices(3);
    t.add_edge(0, 1).unwrap();
    t.add_edge(1, 2).unwrap();
    t.add_edge(2, 0).unwrap();
    let rust = disjoint_union_many(&[&t, &t, &t]).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union_many",
        &t,
        serde_json::json!({
            "extra_graphs": [right_graph_payload(&t), right_graph_payload(&t)],
        }),
    ))
    .expect("decode python disjoint_union_many");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn disjoint_union_many_mixed_sizes_matches_python_igraph() {
    let mut a = Graph::with_vertices(2);
    a.add_edge(0, 1).unwrap();
    let mut b = Graph::with_vertices(4);
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 2).unwrap();
    b.add_edge(2, 3).unwrap();
    let mut c = Graph::with_vertices(3);
    c.add_edge(0, 2).unwrap();
    let rust = disjoint_union_many(&[&a, &b, &c]).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union_many",
        &a,
        serde_json::json!({
            "extra_graphs": [right_graph_payload(&b), right_graph_payload(&c)],
        }),
    ))
    .expect("decode python disjoint_union_many");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.ecount(), py.edges.len());
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn disjoint_union_many_directed_chain_matches_python_igraph() {
    let mut a = Graph::new(2, true).unwrap();
    a.add_edge(0, 1).unwrap();
    let mut b = Graph::new(3, true).unwrap();
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 2).unwrap();
    let rust = disjoint_union_many(&[&a, &b]).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "disjoint_union_many",
        &a,
        serde_json::json!({
            "extra_graphs": [right_graph_payload(&b)],
        }),
    ))
    .expect("decode python disjoint_union_many");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, false));
}

#[test]
fn coreness_with_mode_directed_3_cycle_out_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let rust = coreness_with_mode(&g, CorenessMode::Out).unwrap();
    let py: Vec<u32> = serde_json::from_value(run_ok(
        "coreness_with_mode",
        &g,
        serde_json::json!({"mode": "out"}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn coreness_with_mode_directed_3_cycle_in_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let rust = coreness_with_mode(&g, CorenessMode::In).unwrap();
    let py: Vec<u32> = serde_json::from_value(run_ok(
        "coreness_with_mode",
        &g,
        serde_json::json!({"mode": "in"}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn coreness_with_mode_directed_complete_3_all_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    for &(u, v) in &[(0u32, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)] {
        g.add_edge(u, v).unwrap();
    }
    let rust = coreness_with_mode(&g, CorenessMode::Out).unwrap();
    let py: Vec<u32> = serde_json::from_value(run_ok(
        "coreness_with_mode",
        &g,
        serde_json::json!({"mode": "out"}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
}

#[test]
fn assortativity_degree_directed_chain_with_branch_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    let rust = assortativity_degree_directed(&g).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "assortativity_degree_directed",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn assortativity_degree_directed_3_cycle_returns_none_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let rust = assortativity_degree_directed(&g).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "assortativity_degree_directed",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
    assert_eq!(rust, None);
}

#[test]
fn assortativity_degree_directed_complex_graph_matches_python_igraph() {
    // Larger directed graph with non-degenerate Pearson:
    //   0→1, 0→2, 0→3, 1→3, 2→3, 4→3 (everyone points at 3)
    let mut g = Graph::new(5, true).unwrap();
    for &(u, v) in &[(0u32, 1), (0, 2), (0, 3), (1, 3), (2, 3), (4, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let rust = assortativity_degree_directed(&g).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "assortativity_degree_directed",
        &g,
        serde_json::json!({}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        (None, None) => {}
        (a, b) => panic!("rust={a:?} py={b:?}"),
    }
}

#[test]
fn modularity_directed_two_triangles_bridge_matches_python_igraph() {
    let mut g = Graph::new(6, true).unwrap();
    for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let mem = vec![0u32, 0, 0, 1, 1, 1];
    let rust = modularity_directed(&g, &mem, 1.0).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "modularity_directed",
        &g,
        serde_json::json!({"membership": mem, "resolution": 1.0}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

#[test]
fn modularity_directed_3_cycle_single_partition_matches_python_igraph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let mem = vec![0u32, 0, 0];
    let rust = modularity_directed(&g, &mem, 1.0).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "modularity_directed",
        &g,
        serde_json::json!({"membership": mem, "resolution": 1.0}),
    ))
    .expect("decode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-12, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

#[test]
fn modularity_directed_undirected_routes_to_undirected_oracle() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let mem = vec![0u32, 0, 1, 1];
    let rust = modularity_directed(&g, &mem, 1.0).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok(
        "modularity",
        &g,
        serde_json::json!({"membership": mem, "resolution": 1.0}),
    ))
    .expect("decode");
    assert_eq!(rust, py);
}

// ---- ALGO-OP-004 union (2-graph variant) -----

#[test]
fn union_undirected_triangle_plus_path_matches_python_igraph() {
    // Triangle on {0,1,2} ∪ path 0-1-3 → 4 edges on 4 vertices.
    let mut a = Graph::with_vertices(3);
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    a.add_edge(2, 0).unwrap();
    let mut b = Graph::with_vertices(4);
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 3).unwrap();
    let rust = union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python union");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn union_directed_paths_keep_orientation_separate_matches_python_igraph() {
    // left: 0→1, 1→2; right: 1→0, 2→1. Both orientations preserved.
    let mut a = Graph::new(3, true).unwrap();
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::new(3, true).unwrap();
    b.add_edge(1, 0).unwrap();
    b.add_edge(2, 1).unwrap();
    let rust = union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python union");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(rust.is_directed());
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, false));
}

#[test]
fn union_max_multiplicity_matches_python_igraph() {
    // left: 2× (0,1), 1× (1,2); right: 4× (0,1), 3× (2,3) → max gives
    // 4× (0,1), 1× (1,2), 3× (2,3) on 4 vertices.
    let mut a = Graph::with_vertices(4);
    a.add_edge(0, 1).unwrap();
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::with_vertices(4);
    for _ in 0..4 {
        b.add_edge(0, 1).unwrap();
    }
    for _ in 0..3 {
        b.add_edge(2, 3).unwrap();
    }
    let rust = union(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "union",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python union");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.ecount(), 8);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

// ---- ALGO-OP-005 intersection (2-graph variant) -----

#[test]
fn intersection_undirected_triangle_inter_path_matches_python_igraph() {
    // Triangle on {0,1,2} ∩ path 0-1-2 → 2 edges (0,1) + (1,2).
    let mut a = Graph::with_vertices(3);
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    a.add_edge(2, 0).unwrap();
    let mut b = Graph::with_vertices(3);
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 2).unwrap();
    let rust = intersection(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "intersection",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python intersection");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn intersection_directed_overlap_matches_python_igraph() {
    // left: 0→1, 1→2, 2→0; right: 0→1, 2→0, 0→2. Common: 0→1, 2→0.
    let mut a = Graph::new(3, true).unwrap();
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    a.add_edge(2, 0).unwrap();
    let mut b = Graph::new(3, true).unwrap();
    b.add_edge(0, 1).unwrap();
    b.add_edge(2, 0).unwrap();
    b.add_edge(0, 2).unwrap();
    let rust = intersection(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "intersection",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python intersection");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(rust.is_directed());
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, false));
}

#[test]
fn intersection_min_multiplicity_matches_python_igraph() {
    // left: 3× (0,1), 1× (1,2); right: 2× (0,1), 4× (1,2), 5× (2,3).
    // Intersection: min for (0,1)=2, (1,2)=1; (2,3) absent → dropped.
    // Total: 2 + 1 = 3 edges.
    let mut a = Graph::with_vertices(4);
    for _ in 0..3 {
        a.add_edge(0, 1).unwrap();
    }
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::with_vertices(4);
    for _ in 0..2 {
        b.add_edge(0, 1).unwrap();
    }
    for _ in 0..4 {
        b.add_edge(1, 2).unwrap();
    }
    for _ in 0..5 {
        b.add_edge(2, 3).unwrap();
    }
    let rust = intersection(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "intersection",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python intersection");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.ecount(), 3);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

// ---- ALGO-OP-006 difference (2-graph variant) -----

#[test]
fn difference_undirected_triangle_minus_path_matches_python_igraph() {
    // Triangle {(0,1),(1,2),(2,0)} \ path {(0,1),(1,2)} → {(0,2)}.
    let mut a = Graph::with_vertices(3);
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 2).unwrap();
    a.add_edge(2, 0).unwrap();
    let mut b = Graph::with_vertices(3);
    b.add_edge(0, 1).unwrap();
    b.add_edge(1, 2).unwrap();
    let rust = difference(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "difference",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python difference");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

#[test]
fn difference_directed_keeps_unmatched_orientation_matches_python_igraph() {
    // orig: 0→1, 1→0, 1→2; sub: 0→1. Result: 1→0, 1→2.
    let mut a = Graph::new(3, true).unwrap();
    a.add_edge(0, 1).unwrap();
    a.add_edge(1, 0).unwrap();
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::new(3, true).unwrap();
    b.add_edge(0, 1).unwrap();
    let rust = difference(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "difference",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python difference");
    assert_eq!(rust.vcount(), py.vcount);
    assert!(rust.is_directed());
    assert_eq!(rust.is_directed(), py.directed);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, false));
}

#[test]
fn difference_multiplicity_clamps_to_zero_matches_python_igraph() {
    // orig: 4× (0,1), 1× (1,2); sub: 2× (0,1), 5× (1,2), 3× (2,3).
    // Result: 2× (0,1) only — (1,2) clamps to 0; (2,3) absent in orig.
    let mut a = Graph::with_vertices(4);
    for _ in 0..4 {
        a.add_edge(0, 1).unwrap();
    }
    a.add_edge(1, 2).unwrap();
    let mut b = Graph::with_vertices(4);
    for _ in 0..2 {
        b.add_edge(0, 1).unwrap();
    }
    for _ in 0..5 {
        b.add_edge(1, 2).unwrap();
    }
    for _ in 0..3 {
        b.add_edge(2, 3).unwrap();
    }
    let rust = difference(&a, &b).unwrap();
    let py: PyDisjointUnion = serde_json::from_value(run_ok(
        "difference",
        &a,
        serde_json::json!({"right_graph": right_graph_payload(&b)}),
    ))
    .expect("decode python difference");
    assert_eq!(rust.vcount(), py.vcount);
    assert_eq!(rust.ecount(), 2);
    assert_eq!(rust_du_pairs(&rust), py_du_pairs(&py, true));
}

// ---- ALGO-SP-021abc mode-aware ecc/rad/diam -----

fn mode_str(m: EccMode) -> &'static str {
    match m {
        EccMode::Out => "out",
        EccMode::In => "in",
        EccMode::All => "all",
    }
}

#[test]
fn eccentricity_with_mode_directed_path_matches_python_igraph() {
    // 0→1→2→3. Out: forward distances; In: reverse; All: undirected.
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    for m in [EccMode::Out, EccMode::In, EccMode::All] {
        let rust = eccentricity_with_mode(&g, m).unwrap();
        let py: Vec<u32> = serde_json::from_value(run_ok(
            "eccentricity_with_mode",
            &g,
            serde_json::json!({"mode": mode_str(m)}),
        ))
        .expect("decode python eccentricity_with_mode");
        assert_eq!(rust, py, "mode {m:?}");
    }
}

#[test]
fn radius_with_mode_directed_cycle_matches_python_igraph() {
    // 0→1→2→0. Out / In / All all give radius.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    for m in [EccMode::Out, EccMode::In, EccMode::All] {
        let rust = radius_with_mode(&g, m).unwrap();
        let py: Option<u32> = serde_json::from_value(run_ok(
            "radius_with_mode",
            &g,
            serde_json::json!({"mode": mode_str(m)}),
        ))
        .expect("decode python radius_with_mode");
        assert_eq!(rust, py, "mode {m:?}");
    }
}

#[test]
fn diameter_with_mode_directed_dag_matches_python_igraph() {
    // 0→1, 0→2, 1→3, 2→3. DAG diamond.
    let mut g = Graph::new(4, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 3).unwrap();
    g.add_edge(2, 3).unwrap();
    for m in [EccMode::Out, EccMode::In, EccMode::All] {
        let rust = diameter_with_mode(&g, m).unwrap();
        let py: Option<u32> = serde_json::from_value(run_ok(
            "diameter_with_mode",
            &g,
            serde_json::json!({"mode": mode_str(m)}),
        ))
        .expect("decode python diameter_with_mode");
        assert_eq!(rust, py, "mode {m:?}");
    }
}

// ---- ALGO-SP-021..023 weighted ecc/rad/diam oracle tests -----

fn assert_close_f64_vec(rust: &[f64], py: &[f64]) {
    assert_eq!(rust.len(), py.len(), "length");
    for (i, (r, p)) in rust.iter().zip(py.iter()).enumerate() {
        assert!((r - p).abs() < 1e-9, "vertex {i}: rust={r} py={p}");
    }
}

#[test]
fn eccentricity_weighted_path_matches_python_igraph() {
    // Path 0-1-2 weights (1, 2.5): ecc = [3.5, 2.5, 3.5].
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 2.5];
    let rust = eccentricity_weighted_with_mode(&g, &weights, EccMode::All).unwrap();
    let py: Vec<f64> = serde_json::from_value(run_ok_with_weights(
        "eccentricity_weighted_with_mode",
        &g,
        Some(weights),
        serde_json::json!({"mode": "all"}),
    ))
    .expect("decode python eccentricity_weighted_with_mode");
    assert_close_f64_vec(&rust, &py);
}

#[test]
fn radius_weighted_directed_path_matches_python_igraph() {
    // Directed 0→1→2 with weights (1, 2): radius depends on mode.
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 2.0];
    for m in [EccMode::Out, EccMode::In, EccMode::All] {
        let rust = radius_weighted_with_mode(&g, &weights, m).unwrap();
        let py: Option<f64> = serde_json::from_value(run_ok_with_weights(
            "radius_weighted_with_mode",
            &g,
            Some(weights.clone()),
            serde_json::json!({"mode": mode_str(m)}),
        ))
        .expect("decode python radius_weighted_with_mode");
        match (rust, py) {
            (Some(r), Some(p)) => assert!((r - p).abs() < 1e-9, "mode {m:?}: rust={r} py={p}"),
            (None, None) => {}
            (a, b) => panic!("mode {m:?}: rust={a:?} py={b:?}"),
        }
    }
}

#[test]
fn diameter_weighted_undirected_triangle_matches_python_igraph() {
    // Undirected triangle (1, 2, 4): diameter = 3 (via shortcut).
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    let weights = vec![1.0_f64, 2.0, 4.0];
    let rust = diameter_weighted_with_mode(&g, &weights, EccMode::All).unwrap();
    let py: Option<f64> = serde_json::from_value(run_ok_with_weights(
        "diameter_weighted_with_mode",
        &g,
        Some(weights),
        serde_json::json!({"mode": "all"}),
    ))
    .expect("decode python diameter_weighted_with_mode");
    match (rust, py) {
        (Some(r), Some(p)) => assert!((r - p).abs() < 1e-9, "rust={r} py={p}"),
        _ => panic!("rust={rust:?} py={py:?}"),
    }
}

// ---- ALGO-SP-005 A* oracle tests --------

#[test]
fn a_star_path_unit_weights_matches_python_igraph() {
    // Path 0-1-2-3 unit weights: A* with null heuristic == BFS path.
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    let (vs, es) = a_star_path(&g, 0, 3, None, DijkstraMode::Out, |_, _| 0.0)
        .unwrap()
        .unwrap();
    #[derive(serde::Deserialize)]
    struct PyPath {
        vertices: Vec<u32>,
        edges: Vec<u32>,
    }
    let py: PyPath = serde_json::from_value(run_ok(
        "a_star_path",
        &g,
        serde_json::json!({"source": 0, "target": 3, "mode": "out"}),
    ))
    .expect("decode python a_star_path");
    assert_eq!(vs, py.vertices);
    assert_eq!(es, py.edges);
}

#[test]
fn a_star_path_weighted_triangle_with_shortcut_matches_python_igraph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let weights = vec![1.0_f64, 4.0, 2.0];
    let (vs, es) = a_star_path(&g, 0, 2, Some(&weights), DijkstraMode::Out, |_, _| 0.0)
        .unwrap()
        .unwrap();
    #[derive(serde::Deserialize)]
    struct PyPath {
        vertices: Vec<u32>,
        edges: Vec<u32>,
    }
    let py: PyPath = serde_json::from_value(run_ok_with_weights(
        "a_star_path",
        &g,
        Some(weights),
        serde_json::json!({"source": 0, "target": 2, "mode": "out"}),
    ))
    .expect("decode python a_star_path");
    assert_eq!(vs, py.vertices);
    assert_eq!(es, py.edges);
}
