//! Three-source conformance tests.
//!
//! Replays JSON fixtures harvested by `scripts/test_extract/from_{c,py,r}.py`
//! through the Rust implementation and asserts equality with the upstream
//! expected output. Failures here indicate divergence from one of the three
//! official igraph implementations (C / python-igraph / R-igraph).
//!
//! Always-on (no feature gate) so plain `cargo test` exercises it; the only
//! external dependency is the JSON fixtures, which live in-tree.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GraphPayload {
    n: u32,
    edges: Vec<(u32, u32)>,
    #[allow(dead_code)]
    directed: bool,
    weights: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
struct Conformance {
    source: String,
    origin: String,
    graph: GraphPayload,
    algo: String,
    params: serde_json::Value,
    expected: serde_json::Value,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Approximate JSON equality. Integers and bools must match exactly;
/// floats compare with a relative+absolute tolerance to absorb the
/// 1-ULP differences that appear when the same f64 round-trips through
/// Python's `json.dumps` and Rust's `serde_json` with different
/// shortest-repr digit counts. Recurses into arrays and objects.
fn json_approx_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value as V;
    match (a, b) {
        (V::Null, V::Null) => true,
        (V::Bool(x), V::Bool(y)) => x == y,
        (V::String(x), V::String(y)) => x == y,
        (V::Number(x), V::Number(y)) => {
            if let (Some(xi), Some(yi)) = (x.as_i64(), y.as_i64()) {
                return xi == yi;
            }
            if let (Some(xu), Some(yu)) = (x.as_u64(), y.as_u64()) {
                return xu == yu;
            }
            match (x.as_f64(), y.as_f64()) {
                (Some(xf), Some(yf)) => {
                    if xf.is_nan() && yf.is_nan() {
                        return true;
                    }
                    let abs = (xf - yf).abs();
                    let scale = xf.abs().max(yf.abs()).max(1.0);
                    abs <= 1e-12_f64 * scale
                }
                _ => false,
            }
        }
        (V::Array(xs), V::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| json_approx_eq(x, y))
        }
        (V::Object(xs), V::Object(ys)) => {
            xs.len() == ys.len()
                && xs
                    .iter()
                    .all(|(k, v)| ys.get(k).is_some_and(|w| json_approx_eq(v, w)))
        }
        _ => false,
    }
}

fn load_all(algo: &str) -> Vec<(PathBuf, Conformance)> {
    let mut out = Vec::new();
    for source in &["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(source)
            .join(algo);
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read conformance dir") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            out.push((path, case));
        }
    }
    out
}

fn build_graph(payload: &GraphPayload) -> rust_igraph::Graph {
    let mut g = rust_igraph::Graph::new(payload.n, payload.directed).expect("graph init");
    for &(u, v) in &payload.edges {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

/// Run every fixture under `tests/conformance/{c,py,r}/<algo>/` through
/// `runner` and assert equality with the upstream expected JSON value.
/// Also asserts the per-AWU invariant that all three sources contribute
/// at least one fixture (Phase-0 guarantee).
///
/// `runner` receives the full graph and the fixture's `params` JSON
/// object; it returns a JSON value to compare against `expected`.
fn run_conformance(
    algo: &str,
    runner: impl Fn(&rust_igraph::Graph, &serde_json::Value) -> serde_json::Value,
) {
    run_conformance_with_skip(algo, &[], runner);
}

/// Same as [`run_conformance`] but allows naming sources that genuinely
/// don't expose the algorithm (e.g. python-igraph 0.11 lacks
/// `is_eulerian`). `skip_sources` entries must be one of
/// `"c"`, `"py"`, `"r"`. Document the skip in CONFORMANCE.md.
fn run_conformance_with_skip(
    algo: &str,
    skip_sources: &[&str],
    runner: impl Fn(&rust_igraph::Graph, &serde_json::Value) -> serde_json::Value,
) {
    let cases = load_all(algo);
    assert!(
        !cases.is_empty(),
        "no {algo} conformance fixtures found — did you run \
         `.venv/bin/python -m scripts.test_extract.from_c --algo {algo}` (and from_py / from_r)?"
    );

    let mut counts = std::collections::HashMap::<&'static str, usize>::new();
    for (path, case) in cases {
        assert_eq!(case.algo, algo);
        let g = build_graph(&case.graph);
        let actual = runner(&g, &case.params);
        assert!(
            json_approx_eq(&actual, &case.expected),
            "conformance failure\n  fixture: {}\n  source:  {}\n  origin:  {}\n  actual:   {}\n  expected: {}",
            path.display(),
            case.source,
            case.origin,
            actual,
            case.expected,
        );
        let key: &'static str = match case.source.as_str() {
            "c" => "c",
            "py" => "py",
            "r" => "r",
            _ => panic!("unknown source {} in {}", case.source, path.display()),
        };
        *counts.entry(key).or_default() += 1;
    }
    for source in ["c", "py", "r"] {
        if skip_sources.contains(&source) {
            continue;
        }
        assert!(
            counts.get(source).copied().unwrap_or(0) > 0,
            "no {algo} fixtures from source {source}"
        );
    }
}

fn root_param(params: &serde_json::Value) -> u32 {
    u32::try_from(
        params
            .get("root")
            .and_then(serde_json::Value::as_u64)
            .expect("`root` param required"),
    )
    .expect("root fits in u32")
}

#[test]
fn bfs_three_source_conformance() {
    run_conformance("bfs", |g, params| {
        let order = rust_igraph::bfs(g, root_param(params)).expect("bfs");
        serde_json::json!(order)
    });
}

#[test]
fn dfs_three_source_conformance() {
    run_conformance("dfs", |g, params| {
        let order = rust_igraph::dfs(g, root_param(params)).expect("dfs");
        serde_json::json!(order)
    });
}

#[test]
fn connected_components_three_source_conformance() {
    run_conformance("connected_components", |g, _params| {
        let cc = rust_igraph::connected_components(g).expect("cc");
        serde_json::json!({
            "membership": cc.membership,
            "count": cc.count,
        })
    });
}

#[test]
fn strongly_connected_components_three_source_conformance() {
    run_conformance("strongly_connected_components", |g, _params| {
        let scc = rust_igraph::strongly_connected_components(g).expect("scc");
        serde_json::json!({
            "membership": scc.membership,
            "count": scc.count,
        })
    });
}

#[test]
fn eccentricity_three_source_conformance() {
    run_conformance("eccentricity", |g, _params| {
        let ecc = rust_igraph::eccentricity(g).expect("eccentricity");
        serde_json::json!(ecc)
    });
}

#[test]
fn radius_three_source_conformance() {
    run_conformance("radius", |g, _params| {
        let r = rust_igraph::radius(g).expect("radius");
        match r {
            Some(n) => serde_json::json!(n),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn transitivity_local_undirected_three_source_conformance() {
    run_conformance("transitivity_local_undirected", |g, _params| {
        let v = rust_igraph::transitivity_local_undirected(g).expect("local transitivity");
        let arr: Vec<serde_json::Value> = v
            .into_iter()
            .map(|o| match o {
                Some(x) => serde_json::json!(x),
                None => serde_json::Value::Null,
            })
            .collect();
        serde_json::Value::Array(arr)
    });
}

#[test]
fn avg_nearest_neighbor_degree_three_source_conformance() {
    run_conformance("avg_nearest_neighbor_degree", |g, _params| {
        let v = rust_igraph::avg_nearest_neighbor_degree(g).expect("knn");
        let arr: Vec<serde_json::Value> = v
            .into_iter()
            .map(|o| match o {
                Some(x) => serde_json::json!(x),
                None => serde_json::Value::Null,
            })
            .collect();
        serde_json::Value::Array(arr)
    });
}

#[test]
fn reciprocity_three_source_conformance() {
    run_conformance("reciprocity", |g, _params| {
        let r = rust_igraph::reciprocity(g).expect("reciprocity");
        match r {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn count_reachable_three_source_conformance() {
    run_conformance("count_reachable", |g, _params| {
        let r = rust_igraph::count_reachable(g).expect("count_reachable");
        serde_json::json!(r)
    });
}

#[test]
fn reachability_matrix_three_source_conformance() {
    run_conformance("reachability_matrix", |g, _params| {
        let m = rust_igraph::reachability_matrix(g).expect("reachability_matrix");
        serde_json::json!(m)
    });
}

#[test]
fn eigenvector_centrality_three_source_conformance() {
    run_conformance("eigenvector_centrality", |g, _params| {
        let ec = rust_igraph::eigenvector_centrality(g).expect("eigenvector_centrality");
        serde_json::json!(ec)
    });
}

#[test]
fn biconnected_components_three_source_conformance() {
    run_conformance("biconnected_components", |g, _params| {
        let bc = rust_igraph::biconnected_components(g).expect("biconnected_components");
        // Canonicalise: each component sorted, list of components sorted.
        let mut comps: Vec<Vec<u32>> = bc
            .components
            .iter()
            .map(|c| {
                let mut v = c.clone();
                v.sort_unstable();
                v
            })
            .collect();
        comps.sort();
        let mut aps = bc.articulation_points.clone();
        aps.sort_unstable();
        serde_json::json!({
            "count": bc.count,
            "components": comps,
            "articulation_points": aps,
        })
    });
}

#[test]
fn pagerank_three_source_conformance() {
    run_conformance("pagerank", |g, _params| {
        let pr = rust_igraph::pagerank(g).expect("pagerank");
        serde_json::json!(pr)
    });
}

#[test]
fn edge_betweenness_three_source_conformance() {
    run_conformance("edge_betweenness", |g, _params| {
        let eb = rust_igraph::edge_betweenness(g).expect("edge_betweenness");
        serde_json::json!(eb)
    });
}

#[test]
fn betweenness_three_source_conformance() {
    run_conformance("betweenness", |g, _params| {
        let b = rust_igraph::betweenness(g).expect("betweenness");
        serde_json::json!(b)
    });
}

#[test]
fn harmonic_centrality_three_source_conformance() {
    run_conformance("harmonic_centrality", |g, _params| {
        let h = rust_igraph::harmonic_centrality(g).expect("harmonic_centrality");
        serde_json::json!(h)
    });
}

#[test]
fn closeness_three_source_conformance() {
    run_conformance("closeness", |g, _params| {
        let c = rust_igraph::closeness(g).expect("closeness");
        let arr: Vec<serde_json::Value> = c
            .into_iter()
            .map(|o| match o {
                Some(x) => serde_json::json!(x),
                None => serde_json::Value::Null,
            })
            .collect();
        serde_json::Value::Array(arr)
    });
}

#[test]
fn transitive_closure_three_source_conformance() {
    run_conformance("transitive_closure", |g, _params| {
        let tc = rust_igraph::transitive_closure(g).expect("transitive_closure");
        let m = u32::try_from(tc.ecount()).expect("ecount fits in u32");
        let mut edges: Vec<[u32; 2]> = (0..m)
            .map(|e| {
                let (u, v) = tc.edge(e).unwrap();
                [u, v]
            })
            .collect();
        edges.sort_unstable();
        serde_json::json!({
            "vcount": tc.vcount(),
            "directed": tc.is_directed(),
            "edges": edges,
        })
    });
}

#[test]
fn is_simple_three_source_conformance() {
    run_conformance("is_simple", |g, _params| {
        let s = rust_igraph::is_simple(g).expect("is_simple");
        serde_json::json!(s)
    });
}

#[test]
fn has_loop_three_source_conformance() {
    run_conformance("has_loop", |g, _params| {
        serde_json::json!(rust_igraph::has_loop(g).expect("has_loop"))
    });
}

#[test]
fn has_multiple_three_source_conformance() {
    run_conformance("has_multiple", |g, _params| {
        serde_json::json!(rust_igraph::has_multiple(g).expect("has_multiple"))
    });
}

#[test]
fn is_loop_three_source_conformance() {
    // Edge ids change after wire round-trip, so compare as a sorted
    // multiset (count of true and false matters; per-edge alignment
    // does not).
    run_conformance("is_loop", |g, _params| {
        let mut v = rust_igraph::is_loop(g).expect("is_loop");
        v.sort_unstable();
        serde_json::json!(v)
    });
}

#[test]
fn is_multiple_three_source_conformance() {
    run_conformance("is_multiple", |g, _params| {
        let mut v = rust_igraph::is_multiple(g).expect("is_multiple");
        v.sort_unstable();
        serde_json::json!(v)
    });
}

#[test]
fn modularity_three_source_conformance() {
    run_conformance("modularity", |g, params| {
        let membership: Vec<u32> = params
            .get("membership")
            .and_then(serde_json::Value::as_array)
            .expect("membership param missing")
            .iter()
            .map(|v| u32::try_from(v.as_u64().expect("non-negative int label")).unwrap())
            .collect();
        let resolution = params
            .get("resolution")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);
        let q = rust_igraph::modularity(g, &membership, resolution).expect("modularity");
        match q {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn dijkstra_distances_three_source_conformance() {
    // We need the per-fixture weights vector, which lives on the graph
    // payload — not threaded through `run_conformance`'s `(graph,
    // params)` runner. Iterate fixtures by hand to access `case.graph.weights`.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_distances");
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = std::fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            let g = build_graph(&case.graph);
            let weights = case.graph.weights.clone().unwrap_or_default();
            let source = u32::try_from(
                case.params
                    .get("source")
                    .and_then(serde_json::Value::as_u64)
                    .expect("source param missing"),
            )
            .expect("source fits in u32");
            let d =
                rust_igraph::dijkstra_distances(&g, source, &weights).expect("dijkstra_distances");
            let rust_json: serde_json::Value = d
                .into_iter()
                .map(|x| match x {
                    Some(v) => serde_json::json!(v),
                    None => serde_json::Value::Null,
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            // Also enforce that the fixture's source label matches the
            // directory layout to keep manifests honest.
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_distances");
            // Origin is informational only.
            let _ = case.origin;
        }
    }
}

#[test]
fn disjoint_union_three_source_conformance() {
    run_conformance("disjoint_union", |g, params| {
        let right_payload: GraphPayload = serde_json::from_value(
            params
                .get("right_graph")
                .expect("right_graph param missing")
                .clone(),
        )
        .expect("decode right_graph payload");
        let right = build_graph(&right_payload);
        let u = rust_igraph::disjoint_union(g, &right).expect("disjoint_union");
        let m = u32::try_from(u.ecount()).expect("ecount fits in u32");
        let mut edges: Vec<[u32; 2]> = (0..m)
            .map(|e| {
                let (a, b) = u.edge(e).unwrap();
                [a, b]
            })
            .collect();
        edges.sort_unstable();
        serde_json::json!({
            "vcount": u.vcount(),
            "directed": u.is_directed(),
            "edges": edges,
        })
    });
}

#[test]
fn simplify_three_source_conformance() {
    run_conformance("simplify", |g, params| {
        let remove_multiple = params
            .get("remove_multiple")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let remove_loops = params
            .get("remove_loops")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let s = rust_igraph::simplify(g, remove_multiple, remove_loops).expect("simplify");
        let m = u32::try_from(s.ecount()).expect("ecount fits in u32");
        let mut edges: Vec<[u32; 2]> = (0..m)
            .map(|e| {
                let (u, v) = s.edge(e).unwrap();
                [u, v]
            })
            .collect();
        edges.sort_unstable();
        serde_json::json!({
            "vcount": s.vcount(),
            "directed": s.is_directed(),
            "edges": edges,
        })
    });
}

#[test]
fn assortativity_degree_three_source_conformance() {
    run_conformance("assortativity_degree", |g, _params| {
        let r = rust_igraph::assortativity_degree(g).expect("assortativity");
        match r {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn density_three_source_conformance() {
    run_conformance("density", |g, _params| {
        let d = rust_igraph::density(g).expect("density");
        match d {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn mean_distance_three_source_conformance() {
    run_conformance("mean_distance", |g, _params| {
        let d = rust_igraph::mean_distance(g).expect("mean_distance");
        match d {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn count_triangles_three_source_conformance() {
    run_conformance("count_triangles", |g, _params| {
        let n = rust_igraph::count_triangles(g).expect("count_triangles");
        serde_json::json!(n)
    });
}

#[test]
fn transitivity_undirected_three_source_conformance() {
    run_conformance("transitivity_undirected", |g, _params| {
        let t = rust_igraph::transitivity_undirected(g).expect("transitivity");
        match t {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn diameter_three_source_conformance() {
    run_conformance("diameter", |g, _params| {
        let d = rust_igraph::diameter(g).expect("diameter");
        match d {
            Some(n) => serde_json::json!(n),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn girth_three_source_conformance() {
    run_conformance("girth", |g, _params| {
        let g_val = rust_igraph::girth(g).expect("girth");
        // Map `Option<u32>` to JSON `null` / integer for stable comparison
        // with the fixtures (which encode `inf` as JSON null).
        match g_val {
            Some(n) => serde_json::json!(n),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn is_biconnected_three_source_conformance() {
    run_conformance("is_biconnected", |g, _params| {
        let r = rust_igraph::is_biconnected(g).expect("is_biconnected");
        serde_json::json!(r)
    });
}

#[test]
fn bridges_three_source_conformance() {
    run_conformance("bridges", |g, _params| {
        let mut br = rust_igraph::bridges(g).expect("bridges");
        // Sort to canonicalise (DFS-discovery order is impl-dependent).
        // Edge ids match the fixture's `edges` order — `build_graph` adds
        // them in that order so ids are stable.
        br.sort_unstable();
        serde_json::json!(br)
    });
}

#[test]
fn articulation_points_three_source_conformance() {
    run_conformance("articulation_points", |g, _params| {
        let mut ap = rust_igraph::articulation_points(g).expect("articulation");
        // Sort to match the fixture's canonicalised representation
        // (DFS-discovery order is implementation-dependent across the
        // three reference impls).
        ap.sort_unstable();
        serde_json::json!(ap)
    });
}

#[test]
fn eulerian_path_three_source_conformance() {
    // python-igraph 0.11 has no Eulerian API at all; py-skip per CC-040 lineage.
    // Each fixture's `expected` is the walk length (multiple valid walks
    // exist — we don't pin a specific edge sequence). The runner compares
    // `len(walk)` directly.
    run_conformance_with_skip("eulerian_path", &["py"], |g, _params| {
        let walk = rust_igraph::eulerian_path(g)
            .expect("eulerian_path")
            .expect("walk should exist for these fixtures");
        serde_json::json!(walk.len())
    });
}

#[test]
fn is_eulerian_three_source_conformance() {
    // python-igraph 0.11.x exposes no Eulerian API (verified — no
    // Graph.is_eulerian / has_eulerian_path); skip the "py" source
    // requirement until a future python-igraph release adds it.
    run_conformance_with_skip("is_eulerian", &["py"], |g, _params| {
        let r = rust_igraph::is_eulerian(g).expect("is_eulerian");
        serde_json::json!({
            "has_path": r.has_path,
            "has_cycle": r.has_cycle,
        })
    });
}

#[test]
fn distances_three_source_conformance() {
    run_conformance("distances", |g, params| {
        let source = u32::try_from(
            params
                .get("source")
                .and_then(serde_json::Value::as_u64)
                .expect("`source` param required"),
        )
        .expect("source fits in u32");
        let d = rust_igraph::distances(g, source).expect("distances");
        serde_json::json!(d)
    });
}
