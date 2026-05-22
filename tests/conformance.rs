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
fn knnk_three_source_conformance() {
    run_conformance("knnk", |g, _params| {
        let v = rust_igraph::knnk(g).expect("knnk");
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
fn avg_nearest_neighbor_degree_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("avg_nearest_neighbor_degree_weighted");
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
            let v = rust_igraph::avg_nearest_neighbor_degree_weighted(&g, &weights)
                .expect("knn_weighted");
            let arr: serde_json::Value = v
                .into_iter()
                .map(|o| match o {
                    Some(x) => serde_json::json!(x),
                    None => serde_json::Value::Null,
                })
                .collect();
            assert!(
                json_approx_eq(&arr, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                arr,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "avg_nearest_neighbor_degree_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn knnk_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("knnk_weighted");
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
            let v = rust_igraph::knnk_weighted(&g, &weights).expect("knnk_weighted");
            let arr: serde_json::Value = v
                .into_iter()
                .map(|o| match o {
                    Some(x) => serde_json::json!(x),
                    None => serde_json::Value::Null,
                })
                .collect();
            assert!(
                json_approx_eq(&arr, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                arr,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "knnk_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn decompose_three_source_conformance() {
    run_conformance("decompose", |g, _params| {
        let parts = rust_igraph::decompose(g).expect("decompose");
        let comps: Vec<serde_json::Value> = parts
            .iter()
            .map(|sub| {
                let mut edges: Vec<[u32; 2]> = (0..sub.ecount())
                    .map(|e| {
                        let eid = u32::try_from(e).expect("edge id fits u32");
                        let s = sub.edge_source(eid).expect("edge source");
                        let t = sub.edge_target(eid).expect("edge target");
                        if sub.is_directed() || s <= t {
                            [s, t]
                        } else {
                            [t, s]
                        }
                    })
                    .collect();
                edges.sort_unstable();
                serde_json::json!({
                    "vcount": sub.vcount(),
                    "directed": sub.is_directed(),
                    "edges": edges,
                })
            })
            .collect();
        serde_json::Value::Array(comps)
    });
}

#[test]
fn transitivity_barrat_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("transitivity_barrat");
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
            let v = rust_igraph::transitivity_barrat(&g, &weights).expect("transitivity_barrat");
            let arr: serde_json::Value = v
                .into_iter()
                .map(|o| match o {
                    Some(x) => serde_json::json!(x),
                    None => serde_json::Value::Null,
                })
                .collect();
            assert!(
                json_approx_eq(&arr, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                arr,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "transitivity_barrat");
            let _ = case.origin;
        }
    }
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
fn coreness_three_source_conformance() {
    run_conformance("coreness", |g, _params| {
        let cores = rust_igraph::coreness(g).expect("coreness");
        serde_json::json!(cores)
    });
}

#[test]
fn assortativity_degree_directed_three_source_conformance() {
    run_conformance("assortativity_degree_directed", |g, _params| {
        let r =
            rust_igraph::assortativity_degree_directed(g).expect("assortativity_degree_directed");
        match r {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn coreness_with_mode_three_source_conformance() {
    use rust_igraph::CorenessMode;
    run_conformance("coreness_with_mode", |g, params| {
        let mode_str = params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("all");
        let mode = match mode_str {
            "in" => CorenessMode::In,
            "out" => CorenessMode::Out,
            _ => CorenessMode::All,
        };
        let cores = rust_igraph::coreness_with_mode(g, mode).expect("coreness_with_mode");
        serde_json::json!(cores)
    });
}

#[test]
fn is_simple_with_mode_three_source_conformance() {
    use rust_igraph::SimpleMode;
    run_conformance("is_simple_with_mode", |g, params| {
        let undirected = params
            .get("directed_as_undirected")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let mode = if undirected {
            SimpleMode::DirectedAsUndirected
        } else {
            SimpleMode::DirectedAsDirected
        };
        let r = rust_igraph::is_simple_with_mode(g, mode).expect("is_simple_with_mode");
        serde_json::json!(r)
    });
}

#[test]
fn modularity_directed_three_source_conformance() {
    run_conformance("modularity_directed", |g, params| {
        let mem: Vec<u32> = params
            .get("membership")
            .and_then(serde_json::Value::as_array)
            .expect("membership param missing")
            .iter()
            .map(|v| u32::try_from(v.as_u64().expect("u32 label")).expect("fits u32"))
            .collect();
        let resolution = params
            .get("resolution")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);
        let r = rust_igraph::modularity_directed(g, &mem, resolution).expect("modularity_directed");
        match r {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn modularity_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner because the standard
    // `run_conformance` signature only forwards `(graph, params)`,
    // but modularity_weighted needs `case.graph.weights` too.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("modularity_weighted");
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
            let mem: Vec<u32> = case
                .params
                .get("membership")
                .and_then(serde_json::Value::as_array)
                .expect("membership param missing")
                .iter()
                .map(|v| u32::try_from(v.as_u64().expect("u32 label")).expect("fits u32"))
                .collect();
            let resolution = case
                .params
                .get("resolution")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);
            let r = rust_igraph::modularity_weighted(&g, &mem, resolution, &weights)
                .expect("modularity_weighted");
            let rust_json = match r {
                Some(v) => serde_json::json!(v),
                None => serde_json::Value::Null,
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "modularity_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn reciprocity_with_mode_three_source_conformance() {
    use rust_igraph::ReciprocityMode;
    run_conformance("reciprocity_with_mode", |g, params| {
        let ignore_loops = params
            .get("ignore_loops")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let mode_str = params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("default");
        let mode = match mode_str {
            "ratio" => ReciprocityMode::Ratio,
            _ => ReciprocityMode::Default,
        };
        let r = rust_igraph::reciprocity_with_mode(g, ignore_loops, mode)
            .expect("reciprocity_with_mode");
        match r {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
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
fn eigenvector_centrality_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    fn chop(v: &[f64]) -> Vec<f64> {
        v.iter()
            .map(|&x| if x.abs() < 1e-9 { 0.0 } else { x })
            .collect()
    }
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("eigenvector_centrality_weighted");
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
            let s = rust_igraph::eigenvector_centrality_weighted(&g, &weights)
                .expect("eigenvector_centrality_weighted");
            let actual = serde_json::json!({
                "vector": chop(&s.vector),
                "eigenvalue": s.eigenvalue,
            });
            assert!(
                json_approx_eq(&actual, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                actual,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "eigenvector_centrality_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn eigenvector_centrality_directed_three_source_conformance() {
    fn chop(v: &[f64]) -> Vec<f64> {
        v.iter()
            .map(|&x| if x.abs() < 1e-9 { 0.0 } else { x })
            .collect()
    }
    run_conformance("eigenvector_centrality_directed", |g, params| {
        let mode = match params.get("mode").and_then(|v| v.as_str()).unwrap_or("out") {
            "in" => rust_igraph::EigenvectorMode::In,
            "all" => rust_igraph::EigenvectorMode::All,
            _ => rust_igraph::EigenvectorMode::Out,
        };
        let s = rust_igraph::eigenvector_centrality_directed(g, mode)
            .expect("eigenvector_centrality_directed");
        serde_json::json!({
            "vector": chop(&s.vector),
            "eigenvalue": s.eigenvalue,
        })
    });
}

#[test]
fn hub_and_authority_scores_three_source_conformance() {
    // Floor sub-1e-9 magnitudes to 0 to mirror upstream's vector_chop
    // pre-print step in tests/unit/hub_and_authority.c — without it,
    // 1e-15-scale numerical drift on "exact zero" entries fails the
    // 1e-12*scale json-approx-eq check.
    fn chop(v: &[f64]) -> Vec<f64> {
        v.iter()
            .map(|&x| if x.abs() < 1e-9 { 0.0 } else { x })
            .collect()
    }
    run_conformance("hub_and_authority_scores", |g, _params| {
        let s = rust_igraph::hub_and_authority_scores(g).expect("hub_and_authority_scores");
        serde_json::json!({
            "hub": chop(&s.hub),
            "authority": chop(&s.authority),
            "eigenvalue": s.eigenvalue,
        })
    });
}

#[test]
fn hub_and_authority_scores_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    fn chop(v: &[f64]) -> Vec<f64> {
        v.iter()
            .map(|&x| if x.abs() < 1e-9 { 0.0 } else { x })
            .collect()
    }
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("hub_and_authority_scores_weighted");
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
            let s = rust_igraph::hub_and_authority_scores_weighted(&g, &weights)
                .expect("hub_and_authority_scores_weighted");
            let actual = serde_json::json!({
                "hub": chop(&s.hub),
                "authority": chop(&s.authority),
                "eigenvalue": s.eigenvalue,
            });
            assert!(
                json_approx_eq(&actual, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                actual,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "hub_and_authority_scores_weighted");
            let _ = case.origin;
        }
    }
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
fn biconnected_component_edges_three_source_conformance() {
    run_conformance("biconnected_component_edges", |g, _params| {
        let bc = rust_igraph::biconnected_components(g).expect("biconnected_components");
        // Canonicalise to sorted per-component endpoint pairs (min, max),
        // outer list lexicographically sorted. Matches the manifest's
        // expected-payload encoding.
        let mut canon: Vec<Vec<[u32; 2]>> = bc
            .component_edges
            .iter()
            .map(|edges| {
                let mut pairs: Vec<[u32; 2]> = edges
                    .iter()
                    .map(|&e| {
                        let (u, v) = g.edge(e).expect("edge endpoints");
                        if u <= v { [u, v] } else { [v, u] }
                    })
                    .collect();
                pairs.sort_unstable();
                pairs
            })
            .collect();
        canon.sort();
        serde_json::json!(canon)
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
fn louvain_three_source_conformance() {
    // Louvain partitions vary with shuffle order across
    // implementations, so the conformance harness asserts on (a) the
    // modularity-score window upstream attains and (b) the community
    // count window. Exact-membership equality would be too brittle:
    // even a different SplitMix64 seed flips it.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("louvain");
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
                serde_json::from_slice(&bytes).expect("parse louvain fixture JSON");
            let g = build_graph(&case.graph);
            let resolution = case
                .params
                .get("resolution")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);
            let r = match case.graph.weights.as_ref() {
                Some(w) => rust_igraph::louvain_with_options(&g, Some(w), resolution, 0)
                    .expect("louvain_with_options"),
                None => rust_igraph::louvain_with_options(&g, None, resolution, 0)
                    .expect("louvain_with_options"),
            };
            let exp = case
                .expected
                .as_object()
                .expect("louvain `expected` must be an object");
            let q_min = exp
                .get("modularity_min")
                .and_then(serde_json::Value::as_f64)
                .expect("modularity_min");
            let q_max = exp
                .get("modularity_max")
                .and_then(serde_json::Value::as_f64)
                .expect("modularity_max");
            let k_min = u32::try_from(
                exp.get("k_min")
                    .and_then(serde_json::Value::as_u64)
                    .expect("k_min"),
            )
            .expect("k_min fits u32");
            let k_max = u32::try_from(
                exp.get("k_max")
                    .and_then(serde_json::Value::as_u64)
                    .expect("k_max"),
            )
            .expect("k_max fits u32");
            let k = r.membership.iter().copied().max().map_or(0, |m| m + 1);
            assert!(
                r.modularity >= q_min - 1e-9 && r.modularity <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                r.modularity,
                q_min,
                q_max,
                case.origin,
            );
            assert!(
                (k_min..=k_max).contains(&k),
                "{}: k = {} outside [{}, {}] (origin: {})",
                path.display(),
                k,
                k_min,
                k_max,
                case.origin,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "louvain");
            seen_sources.insert(match src {
                "c" => "c",
                "py" => "py",
                "r" => "r",
                _ => unreachable!(),
            });
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no louvain fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + objective enum + dual range
fn leiden_three_source_conformance() {
    // Leiden partitions vary with shuffle order across implementations
    // (Traag-Waltman-van Eck 2019 uses both a queue-based fast-move
    // and an exp(diff/β) randomized refinement), so the conformance
    // harness asserts on (a) the modularity-score window upstream
    // attains and (b) the community count window. Exact-membership
    // equality would be too brittle even within a single seed family.
    use rust_igraph::{LeidenObjective, LeidenOptions, leiden_with_options};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("leiden");
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
                serde_json::from_slice(&bytes).expect("parse leiden fixture JSON");
            let g = build_graph(&case.graph);
            let objective = match case
                .params
                .get("objective")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("modularity")
            {
                "modularity" => LeidenObjective::Modularity,
                "cpm" | "CPM" => LeidenObjective::Cpm,
                "er" | "ER" => LeidenObjective::Er,
                other => panic!("unknown leiden objective: {other}"),
            };
            let resolution = case
                .params
                .get("resolution")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);
            let opts = LeidenOptions {
                objective,
                resolution,
                ..LeidenOptions::default()
            };
            let r = leiden_with_options(&g, case.graph.weights.as_deref(), &opts)
                .expect("leiden_with_options");
            let exp = case
                .expected
                .as_object()
                .expect("leiden `expected` must be an object");
            let q_min = exp
                .get("modularity_min")
                .and_then(serde_json::Value::as_f64)
                .expect("modularity_min");
            let q_max = exp
                .get("modularity_max")
                .and_then(serde_json::Value::as_f64)
                .expect("modularity_max");
            let k_min = u32::try_from(
                exp.get("k_min")
                    .and_then(serde_json::Value::as_u64)
                    .expect("k_min"),
            )
            .expect("k_min fits u32");
            let k_max = u32::try_from(
                exp.get("k_max")
                    .and_then(serde_json::Value::as_u64)
                    .expect("k_max"),
            )
            .expect("k_max fits u32");
            // Leiden's internal Q uses IGRAPH_LOOPS; on loop-free graphs
            // this equals standalone modularity. All Leiden fixtures are
            // loop-free, so r.quality is directly comparable.
            assert!(
                r.quality >= q_min - 1e-9 && r.quality <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                r.quality,
                q_min,
                q_max,
                case.origin,
            );
            assert!(
                (k_min..=k_max).contains(&r.nb_clusters),
                "{}: k = {} outside [{}, {}] (origin: {})",
                path.display(),
                r.nb_clusters,
                k_min,
                k_max,
                case.origin,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "leiden");
            seen_sources.insert(match src {
                "c" => "c",
                "py" => "py",
                "r" => "r",
                _ => unreachable!(),
            });
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no leiden fixtures from source {src}"
        );
    }
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
fn is_tree_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_tree");
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
            let mode_str = case
                .params
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("out");
            let mode = match mode_str {
                "out" => rust_igraph::DijkstraMode::Out,
                "in" => rust_igraph::DijkstraMode::In,
                "all" => rust_igraph::DijkstraMode::All,
                other => panic!("unexpected mode {other} in {}", path.display()),
            };
            let rust = rust_igraph::is_tree(&g, mode).expect("is_tree");
            let rust_json = serde_json::Value::Bool(rust.is_some());
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_tree");
            let _ = case.origin;
        }
    }
}

#[test]
fn is_forest_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_forest");
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
            let mode_str = case
                .params
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("out");
            let mode = match mode_str {
                "out" => rust_igraph::DijkstraMode::Out,
                "in" => rust_igraph::DijkstraMode::In,
                "all" => rust_igraph::DijkstraMode::All,
                other => panic!("unexpected mode {other} in {}", path.display()),
            };
            let rust = rust_igraph::is_forest(&g, mode).expect("is_forest");
            let rust_json = match &rust {
                Some(roots) => serde_json::json!({
                    "is_forest": true,
                    "roots": roots,
                }),
                None => serde_json::json!({
                    "is_forest": false,
                    "roots": [],
                }),
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_forest");
            let _ = case.origin;
        }
    }
}

#[test]
fn is_complete_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_complete");
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
            let rust = rust_igraph::is_complete(&g).expect("is_complete");
            let rust_json = serde_json::Value::Bool(rust);
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_complete");
            let _ = case.origin;
        }
    }
}

#[test]
fn neighborhood_size_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("neighborhood_size");
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
            let order = case
                .params
                .get("order")
                .and_then(serde_json::Value::as_i64)
                .and_then(|x| i32::try_from(x).ok())
                .unwrap_or(1);
            let mode = match case
                .params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("all")
            {
                "out" => rust_igraph::NeighborhoodMode::Out,
                "in" => rust_igraph::NeighborhoodMode::In,
                _ => rust_igraph::NeighborhoodMode::All,
            };
            let mindist = case
                .params
                .get("mindist")
                .and_then(serde_json::Value::as_i64)
                .and_then(|x| i32::try_from(x).ok())
                .unwrap_or(0);
            let rust = rust_igraph::neighborhood_size_with_mode(&g, order, mode, mindist)
                .expect("neighborhood_size");
            let rust_json =
                serde_json::Value::Array(rust.iter().map(|&x| serde_json::json!(x)).collect());
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "neighborhood_size");
            let _ = case.origin;
        }
    }
}

#[test]
fn neighborhood_three_source_conformance() {
    // PR-027b: neighborhood vertex lists. Fixtures store sorted lists
    // because the BFS visitation order differs between implementations,
    // so we sort the Rust output before comparing.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("neighborhood");
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
            let order = case
                .params
                .get("order")
                .and_then(serde_json::Value::as_i64)
                .and_then(|x| i32::try_from(x).ok())
                .unwrap_or(1);
            let mode = match case
                .params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("all")
            {
                "out" => rust_igraph::NeighborhoodMode::Out,
                "in" => rust_igraph::NeighborhoodMode::In,
                _ => rust_igraph::NeighborhoodMode::All,
            };
            let mindist = case
                .params
                .get("mindist")
                .and_then(serde_json::Value::as_i64)
                .and_then(|x| i32::try_from(x).ok())
                .unwrap_or(0);
            let mut rust = rust_igraph::neighborhood_with_mode(&g, order, mode, mindist)
                .expect("neighborhood");
            for inner in &mut rust {
                inner.sort_unstable();
            }
            let rust_json = serde_json::Value::Array(
                rust.into_iter()
                    .map(|v| {
                        serde_json::Value::Array(
                            v.into_iter().map(serde_json::Value::from).collect(),
                        )
                    })
                    .collect(),
            );
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "neighborhood");
            let _ = case.origin;
        }
    }
}

#[test]
fn is_acyclic_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_acyclic");
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
            let rust = rust_igraph::is_acyclic(&g);
            let rust_json = serde_json::Value::Bool(rust);
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_acyclic");
            let _ = case.origin;
        }
    }
}

#[test]
fn topological_sorting_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("topological_sorting");
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
            let mode_str = case
                .params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("out");
            let mode = match mode_str {
                "in" => rust_igraph::DijkstraMode::In,
                "all" => rust_igraph::DijkstraMode::All,
                _ => rust_igraph::DijkstraMode::Out,
            };
            let order = rust_igraph::topological_sorting(&g, mode).expect("topological_sorting");
            let rust_json: serde_json::Value =
                order.into_iter().map(serde_json::Value::from).collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "topological_sorting");
            let _ = case.origin;
        }
    }
}

#[test]
fn is_dag_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_dag");
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
            let rust = rust_igraph::is_dag(&g);
            let rust_json = serde_json::Value::Bool(rust);
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_dag");
            let _ = case.origin;
        }
    }
}

#[test]
fn is_same_graph_three_source_conformance() {
    // Reads the "other" graph from params.other, builds a Rust
    // Graph for each side, and compares via `is_same_graph`.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("is_same_graph");
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
            let g1 = build_graph(&case.graph);
            // Decode the "other" graph payload into the same shape
            // GraphPayload uses, then build it.
            let other_value = case
                .params
                .get("other")
                .expect("other graph payload missing");
            let other_payload: GraphPayload =
                serde_json::from_value(other_value.clone()).expect("decode other graph");
            let g2 = build_graph(&other_payload);
            let rust = rust_igraph::is_same_graph(&g1, &g2);
            let rust_json = serde_json::Value::Bool(rust);
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "is_same_graph");
            let _ = case.origin;
        }
    }
}

#[test]
fn site_percolation_three_source_conformance() {
    // Reads vertex_order from params; runs site_percolation against
    // the build_graph-constructed Rust graph.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("site_percolation");
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
            let vertex_order: Vec<u32> = case
                .params
                .get("vertex_order")
                .and_then(serde_json::Value::as_array)
                .expect("vertex_order param missing")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("vertex id is integer"))
                        .expect("vertex id fits in u32")
                })
                .collect();
            let p = rust_igraph::site_percolation(&g, &vertex_order).expect("site_percolation");
            let rust_json = serde_json::json!({
                "giant_size": p.giant_size,
                "edge_count": p.edge_count,
            });
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "site_percolation");
            let _ = case.origin;
        }
    }
}

#[test]
fn bond_percolation_three_source_conformance() {
    // Reads edge_order from params, resolves through Rust's
    // graph.edge() (via bond_percolation itself), and compares to
    // the hand-computed expected curves.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("bond_percolation");
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
            let edge_order: Vec<u32> = case
                .params
                .get("edge_order")
                .and_then(serde_json::Value::as_array)
                .expect("edge_order param missing")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("edge id is integer"))
                        .expect("edge id fits in u32")
                })
                .collect();
            let p = rust_igraph::bond_percolation(&g, &edge_order).expect("bond_percolation");
            let rust_json = serde_json::json!({
                "giant_size": p.giant_size,
                "vertex_count": p.vertex_count,
            });
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "bond_percolation");
            let _ = case.origin;
        }
    }
}

#[test]
fn edgelist_percolation_three_source_conformance() {
    // Percolation is order-sensitive — read edges directly from
    // `case.graph.edges` (preserves the JSON insertion order; we
    // don't go through `build_graph` because internal index rebuilds
    // would lose ordering for our purposes).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("edgelist_percolation");
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
            let edges: Vec<(u32, u32)> = case.graph.edges.clone();
            let p = rust_igraph::edgelist_percolation(&edges).expect("edgelist_percolation");
            let rust_json = serde_json::json!({
                "giant_size": p.giant_size,
                "vertex_count": p.vertex_count,
            });
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "edgelist_percolation");
            let _ = case.origin;
        }
    }
}

#[test]
fn widest_paths_three_source_conformance() {
    // SPT struct: widths + parents + inbound_edges. Source's width
    // is +∞ by convention and encoded as null in fixtures (JSON has
    // no Infinity literal); the runner converts our Rust Some(inf)
    // to null for source position.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("widest_paths");
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
            let sp = rust_igraph::widest_paths(&g, source, &weights).expect("widest_paths");
            let widths_json: serde_json::Value = sp
                .widths
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if i == source as usize {
                        return serde_json::Value::Null;
                    }
                    match x {
                        Some(v) if v.is_finite() => serde_json::json!(v),
                        _ => serde_json::Value::Null,
                    }
                })
                .collect();
            let parents_json: serde_json::Value = sp
                .parents
                .iter()
                .map(|p| match p {
                    Some(v) => serde_json::json!(v),
                    None => serde_json::Value::Null,
                })
                .collect();
            let edges_json: serde_json::Value = sp
                .inbound_edges
                .iter()
                .map(|e| match e {
                    Some(v) => serde_json::json!(v),
                    None => serde_json::Value::Null,
                })
                .collect();
            let rust_json = serde_json::json!({
                "widths": widths_json,
                "parents": parents_json,
                "inbound_edges": edges_json,
            });
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "widest_paths");
            let _ = case.origin;
        }
    }
}

#[test]
fn widest_paths_to_three_source_conformance() {
    // Multi-target: expected is a list, each entry either null or
    // `{vertices, edges}`. Targets come from params.targets.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("widest_paths_to");
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
            let from = u32::try_from(
                case.params
                    .get("from")
                    .and_then(serde_json::Value::as_u64)
                    .expect("from param missing"),
            )
            .expect("from fits in u32");
            let targets: Vec<u32> = case
                .params
                .get("targets")
                .and_then(serde_json::Value::as_array)
                .expect("targets param missing")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("target is integer"))
                        .expect("target fits in u32")
                })
                .collect();
            let result = rust_igraph::widest_paths_to(&g, from, &targets, &weights)
                .expect("widest_paths_to");
            let rust_json: serde_json::Value = result
                .into_iter()
                .map(|p| match p {
                    None => serde_json::Value::Null,
                    Some((vs, es)) => serde_json::json!({
                        "vertices": vs,
                        "edges": es,
                    }),
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "widest_paths_to");
            let _ = case.origin;
        }
    }
}

#[test]
fn widest_path_widths_floyd_warshall_three_source_conformance() {
    // All-pairs matrix. Diagonal entries are +∞ by convention and
    // encoded as null in fixtures; this runner converts our Rust
    // Some(inf) to null to match the fixture format.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("widest_path_widths_floyd_warshall");
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
            let matrix = rust_igraph::widest_path_widths_floyd_warshall(&g, &weights)
                .expect("widest_path_widths_floyd_warshall");
            let rust_json: serde_json::Value = matrix
                .into_iter()
                .map(|row| {
                    let row_json: serde_json::Value = row
                        .into_iter()
                        .map(|x| match x {
                            Some(v) if v.is_finite() => serde_json::json!(v),
                            // Source-to-self (+∞) and unreachable (None
                            // here would already match) both serialise
                            // to null to align with the fixture format.
                            _ => serde_json::Value::Null,
                        })
                        .collect();
                    row_json
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "widest_path_widths_floyd_warshall");
            let _ = case.origin;
        }
    }
}

#[test]
fn widest_path_three_source_conformance() {
    // Single source-to-target. JSON null means unreachable;
    // `{vertices, edges}` means the path.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("widest_path");
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
            let from = u32::try_from(
                case.params
                    .get("from")
                    .and_then(serde_json::Value::as_u64)
                    .expect("from param missing"),
            )
            .expect("from fits in u32");
            let to = u32::try_from(
                case.params
                    .get("to")
                    .and_then(serde_json::Value::as_u64)
                    .expect("to param missing"),
            )
            .expect("to fits in u32");
            let result = rust_igraph::widest_path(&g, from, to, &weights).expect("widest_path");
            let rust_json = match result {
                None => serde_json::Value::Null,
                Some((vs, es)) => serde_json::json!({
                    "vertices": vs,
                    "edges": es,
                }),
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "widest_path");
            let _ = case.origin;
        }
    }
}

#[test]
fn widest_path_widths_three_source_conformance() {
    // Widths convention: source's own width is `+inf`; JSON has no
    // infinity literal so fixtures encode source position as `null`
    // and this runner replaces our `Some(inf)` with `null` before
    // comparing.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("widest_path_widths");
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
            let widths =
                rust_igraph::widest_path_widths(&g, source, &weights).expect("widest_path_widths");
            let rust_json: serde_json::Value = widths
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    // Source entry: infinite by convention — encode as null
                    // to match fixture format.
                    if i == source as usize {
                        return serde_json::Value::Null;
                    }
                    match x {
                        Some(v) if v.is_finite() => serde_json::json!(v),
                        _ => serde_json::Value::Null,
                    }
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "widest_path_widths");
            let _ = case.origin;
        }
    }
}

#[test]
fn johnson_distances_three_source_conformance() {
    // Matrix-shaped: each fixture's `expected` is Vec<Vec<Option<f64>>>.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("johnson_distances");
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
            let matrix = rust_igraph::johnson_distances(&g, &weights).expect("johnson_distances");
            let rust_json: serde_json::Value = matrix
                .into_iter()
                .map(|row| {
                    let row_json: serde_json::Value = row
                        .into_iter()
                        .map(|x| match x {
                            Some(v) => serde_json::json!(v),
                            None => serde_json::Value::Null,
                        })
                        .collect();
                    row_json
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "johnson_distances");
            let _ = case.origin;
        }
    }
}

#[test]
fn bellman_ford_distances_three_source_conformance() {
    // Parallels dijkstra_distances_three_source_conformance: same
    // wire shape (per-vertex Option<f64>), same need for per-fixture
    // weights from case.graph.weights.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("bellman_ford_distances");
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
            let d = rust_igraph::bellman_ford_distances(&g, source, &weights)
                .expect("bellman_ford_distances");
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
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "bellman_ford_distances");
            let _ = case.origin;
        }
    }
}

#[test]
fn dijkstra_paths_three_source_conformance() {
    // Compares only `distances` since parents/inbound_edges depend on
    // tie-breaking which differs between igraph C / R / py / Rust.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_paths");
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
            let p = rust_igraph::dijkstra_paths(&g, source, &weights).expect("dijkstra_paths");
            let rust_json = serde_json::json!({
                "distances": p
                    .distances
                    .into_iter()
                    .map(|x| match x {
                        Some(v) => serde_json::json!(v),
                        None => serde_json::Value::Null,
                    })
                    .collect::<Vec<_>>(),
            });
            // Fixture's expected payload may include parents/edges; we
            // only check distances. Build a cropped expected value.
            let expected_dist = case
                .expected
                .get("distances")
                .cloned()
                .unwrap_or_else(|| case.expected.clone());
            let cropped_expected = serde_json::json!({"distances": expected_dist});
            assert!(
                json_approx_eq(&rust_json, &cropped_expected),
                "{}: expected {} got {}",
                path.display(),
                cropped_expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_paths");
            let _ = case.origin;
        }
    }
}

#[test]
fn dijkstra_path_to_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_path_to");
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
            let target = u32::try_from(
                case.params
                    .get("target")
                    .and_then(serde_json::Value::as_u64)
                    .expect("target param missing"),
            )
            .expect("target fits in u32");
            let r = rust_igraph::dijkstra_path_to(&g, source, target, &weights)
                .expect("dijkstra_path_to");
            let rust_json = match r {
                None => serde_json::Value::Null,
                Some((vs, es)) => serde_json::json!({
                    "vertices": vs,
                    "edges": es,
                }),
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_path_to");
            let _ = case.origin;
        }
    }
}

#[test]
fn a_star_path_three_source_conformance() {
    // SP-005 A* with null heuristic (h ≡ 0) ≡ Dijkstra single-source
    // single-target. Path enumeration order is heap-dependent and not
    // checked across impls — we compare vertex chain + edge chain
    // directly, picking the same canonical chain igraph C / py / R
    // would.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("a_star_path");
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
            let weights = case.graph.weights.clone();
            let source = u32::try_from(
                case.params
                    .get("source")
                    .and_then(serde_json::Value::as_u64)
                    .expect("source param missing"),
            )
            .expect("source fits in u32");
            let target = u32::try_from(
                case.params
                    .get("target")
                    .and_then(serde_json::Value::as_u64)
                    .expect("target param missing"),
            )
            .expect("target fits in u32");
            let mode = match case
                .params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("out")
            {
                "in" => rust_igraph::DijkstraMode::In,
                "all" => rust_igraph::DijkstraMode::All,
                _ => rust_igraph::DijkstraMode::Out,
            };
            let r =
                rust_igraph::a_star_path(&g, source, target, weights.as_deref(), mode, |_, _| 0.0)
                    .expect("a_star_path");
            let rust_json = match r {
                None => serde_json::Value::Null,
                Some((vs, es)) => serde_json::json!({
                    "vertices": vs,
                    "edges": es,
                }),
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "a_star_path");
            let _ = case.origin;
        }
    }
}

#[test]
fn dijkstra_distances_cutoff_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_distances_cutoff");
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
            let cutoff = case
                .params
                .get("cutoff")
                .and_then(serde_json::Value::as_f64);
            let d = rust_igraph::dijkstra_distances_cutoff(&g, source, &weights, cutoff)
                .expect("dijkstra_distances_cutoff");
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
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_distances_cutoff");
            let _ = case.origin;
        }
    }
}

fn dijkstra_mode_from_params(params: &serde_json::Value) -> rust_igraph::DijkstraMode {
    let s = params
        .get("mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("out");
    match s {
        "in" => rust_igraph::DijkstraMode::In,
        "all" => rust_igraph::DijkstraMode::All,
        _ => rust_igraph::DijkstraMode::Out,
    }
}

#[test]
fn dijkstra_distances_with_mode_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_distances_with_mode");
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
            let mode = dijkstra_mode_from_params(&case.params);
            let d = rust_igraph::dijkstra_distances_with_mode(&g, source, &weights, mode)
                .expect("dijkstra_distances_with_mode");
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
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_distances_with_mode");
            let _ = case.origin;
        }
    }
}

#[test]
fn dijkstra_all_shortest_paths_three_source_conformance() {
    // We compare only `distances` and `nrgeo` (path enumeration order
    // may differ between igraph C / py / R / Rust).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dijkstra_all_shortest_paths");
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
            let mode = dijkstra_mode_from_params(&case.params);
            let r = rust_igraph::dijkstra_all_shortest_paths(&g, source, &weights, mode)
                .expect("dijkstra_all_shortest_paths");
            // Distances independently via dijkstra_distances_with_mode.
            let d = rust_igraph::dijkstra_distances_with_mode(&g, source, &weights, mode)
                .expect("dijkstra_distances_with_mode");
            let rust_json = serde_json::json!({
                "distances": d
                    .into_iter()
                    .map(|x| match x {
                        Some(v) => serde_json::json!(v),
                        None => serde_json::Value::Null,
                    })
                    .collect::<Vec<_>>(),
                "nrgeo": r.nrgeo,
            });
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "dijkstra_all_shortest_paths");
            let _ = case.origin;
        }
    }
}

#[test]
fn assortativity_degree_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    // Fixtures use hand-computed reference values for non-unit weights
    // because python-igraph 0.11 has no Python-level weighted
    // assortativity API.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("assortativity_degree_weighted");
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
            let r = rust_igraph::assortativity_degree_weighted(&g, &weights)
                .expect("assortativity_degree_weighted");
            let rust_json = match r {
                Some(v) => serde_json::json!(v),
                None => serde_json::Value::Null,
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "assortativity_degree_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn assortativity_degree_directed_weighted_three_source_conformance() {
    // PR-006d. Bespoke fixture-walking runner (needs case.graph.weights);
    // fixtures use hand-computed reference values for non-unit weights
    // because python-igraph 0.11 has no Python-level weighted
    // assortativity API (same convention as the undirected variant).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("assortativity_degree_directed_weighted");
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
            let r = rust_igraph::assortativity_degree_directed_weighted(&g, &weights)
                .expect("assortativity_degree_directed_weighted");
            let rust_json = match r {
                Some(v) => serde_json::json!(v),
                None => serde_json::Value::Null,
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "assortativity_degree_directed_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn floyd_warshall_distances_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights when the
    // fixture provides them; falls back to unweighted otherwise).
    // Output is a Vec<Vec<Option<f64>>> 2D matrix, so we encode each row
    // separately and compare via the same json_approx_eq the rest of
    // the suite uses (handles the f64-roundtrip drift).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("floyd_warshall_distances");
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
            let weights = case.graph.weights.clone();
            let matrix = rust_igraph::floyd_warshall_distances(&g, weights.as_deref())
                .expect("floyd_warshall_distances");
            let rust_json: serde_json::Value = matrix
                .into_iter()
                .map(|row| -> serde_json::Value {
                    row.into_iter()
                        .map(|x| match x {
                            Some(v) => serde_json::json!(v),
                            None => serde_json::Value::Null,
                        })
                        .collect()
                })
                .collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "floyd_warshall_distances");
            let _ = case.origin;
        }
    }
}

#[test]
fn pagerank_weighted_three_source_conformance() {
    // Bespoke fixture-walking runner (needs case.graph.weights).
    // PageRank power iteration vs python-igraph ARPACK → 1e-6 tolerance
    // (matches PR-011's pagerank conformance test).
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("pagerank_weighted");
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
            let pr = rust_igraph::pagerank_weighted(&g, &weights).expect("pagerank_weighted");
            let exp = case
                .expected
                .as_array()
                .expect("expected is JSON array of numbers");
            assert_eq!(pr.len(), exp.len(), "{}: length mismatch", path.display());
            for (i, (rust, exp_v)) in pr.iter().zip(exp.iter()).enumerate() {
                let py = exp_v.as_f64().expect("expected entry as f64");
                assert!(
                    (rust - py).abs() < 1e-6,
                    "{}: vertex {i}: rust={rust} expected={py}",
                    path.display()
                );
            }
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "pagerank_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn edge_betweenness_weighted_three_source_conformance() {
    // Bespoke runner: needs both `case.graph.weights` and the
    // parallel `(edges, values)` expected shape. Canonicalise edge
    // endpoint pairs before comparison so Rust storage order vs
    // python-igraph's by-vertex rebuild order doesn't matter.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("edge_betweenness_weighted");
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
            let directed = case.graph.directed;
            let eb = rust_igraph::edge_betweenness_weighted(&g, &weights)
                .expect("edge_betweenness_weighted");

            // Build canonical Vec<((min, max), value)> for both sides
            // and compare element-wise after sorting by canonical pair.
            let canonicalise =
                |u: u32, v: u32| -> (u32, u32) { if directed || u <= v { (u, v) } else { (v, u) } };

            let m_u = u32::try_from(g.ecount()).expect("ecount fits in u32");
            let mut rust_pairs: Vec<((u32, u32), f64)> = (0..m_u)
                .map(|e| {
                    let (u, v) = g.edge(e).unwrap();
                    (canonicalise(u, v), eb[e as usize])
                })
                .collect();
            rust_pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.partial_cmp(&b.1).unwrap()));

            let expected_obj = case
                .expected
                .as_object()
                .expect("expected is JSON object {edges, values}");
            let exp_edges = expected_obj
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .expect("expected.edges array");
            let exp_values = expected_obj
                .get("values")
                .and_then(serde_json::Value::as_array)
                .expect("expected.values array");
            let mut exp_pairs: Vec<((u32, u32), f64)> = exp_edges
                .iter()
                .zip(exp_values.iter())
                .map(|(eptr, vptr)| {
                    let arr = eptr.as_array().expect("edge as array");
                    let u = u32::try_from(arr[0].as_u64().expect("u32")).unwrap();
                    let v = u32::try_from(arr[1].as_u64().expect("u32")).unwrap();
                    let val = vptr.as_f64().expect("value as f64");
                    (canonicalise(u, v), val)
                })
                .collect();
            exp_pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.partial_cmp(&b.1).unwrap()));

            assert_eq!(
                rust_pairs.len(),
                exp_pairs.len(),
                "{}: edge count mismatch",
                path.display()
            );
            for ((rp, rv), (ep, ev)) in rust_pairs.iter().zip(exp_pairs.iter()) {
                assert_eq!(rp, ep, "{}: edge endpoint mismatch", path.display());
                assert!(
                    (rv - ev).abs() < 1e-9 * ev.abs().max(1.0),
                    "{}: edge {rp:?}: rust={rv} expected={ev}",
                    path.display()
                );
            }
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "edge_betweenness_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn betweenness_weighted_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("betweenness_weighted");
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
            let b = rust_igraph::betweenness_weighted(&g, &weights).expect("betweenness_weighted");
            let rust_json: serde_json::Value =
                b.into_iter().map(|v| serde_json::json!(v)).collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "betweenness_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn harmonic_centrality_weighted_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("harmonic_centrality_weighted");
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
            let h = rust_igraph::harmonic_centrality_weighted(&g, &weights)
                .expect("harmonic_centrality_weighted");
            let rust_json: serde_json::Value =
                h.into_iter().map(|v| serde_json::json!(v)).collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "harmonic_centrality_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn closeness_weighted_three_source_conformance() {
    // Bespoke runner — `run_conformance` only forwards `(graph, params)`
    // but we need access to `case.graph.weights`. Same shape as the
    // dijkstra_distances fn above.
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("closeness_weighted");
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
            let c = rust_igraph::closeness_weighted(&g, &weights).expect("closeness_weighted");
            let rust_json: serde_json::Value = c
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
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "closeness_weighted");
            let _ = case.origin;
        }
    }
}

#[test]
fn complementer_three_source_conformance() {
    run_conformance("complementer", |g, params| {
        let loops = params
            .get("loops")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let c = rust_igraph::complementer(g, loops).expect("complementer");
        let m = u32::try_from(c.ecount()).expect("ecount fits in u32");
        let mut edges: Vec<[u32; 2]> = (0..m)
            .map(|e| {
                let (a, b) = c.edge(e).unwrap();
                [a, b]
            })
            .collect();
        edges.sort_unstable();
        serde_json::json!({
            "vcount": c.vcount(),
            "directed": c.is_directed(),
            "edges": edges,
        })
    });
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
fn union_three_source_conformance() {
    run_conformance("union", |g, params| {
        let right_payload: GraphPayload = serde_json::from_value(
            params
                .get("right_graph")
                .expect("right_graph param missing")
                .clone(),
        )
        .expect("decode right_graph payload");
        let right = build_graph(&right_payload);
        let u = rust_igraph::union(g, &right).expect("union");
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
fn intersection_three_source_conformance() {
    run_conformance("intersection", |g, params| {
        let right_payload: GraphPayload = serde_json::from_value(
            params
                .get("right_graph")
                .expect("right_graph param missing")
                .clone(),
        )
        .expect("decode right_graph payload");
        let right = build_graph(&right_payload);
        let u = rust_igraph::intersection(g, &right).expect("intersection");
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
fn difference_three_source_conformance() {
    run_conformance("difference", |g, params| {
        let right_payload: GraphPayload = serde_json::from_value(
            params
                .get("right_graph")
                .expect("right_graph param missing")
                .clone(),
        )
        .expect("decode right_graph payload");
        let right = build_graph(&right_payload);
        let u = rust_igraph::difference(g, &right).expect("difference");
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

fn ecc_mode_from_params(params: &serde_json::Value) -> rust_igraph::EccMode {
    let s = params.get("mode").and_then(|v| v.as_str()).unwrap_or("out");
    match s {
        "in" => rust_igraph::EccMode::In,
        "all" => rust_igraph::EccMode::All,
        _ => rust_igraph::EccMode::Out,
    }
}

#[test]
fn eccentricity_with_mode_three_source_conformance() {
    run_conformance("eccentricity_with_mode", |g, params| {
        let mode = ecc_mode_from_params(params);
        let ecc = rust_igraph::eccentricity_with_mode(g, mode).expect("eccentricity_with_mode");
        serde_json::json!(ecc)
    });
}

#[test]
fn radius_with_mode_three_source_conformance() {
    run_conformance("radius_with_mode", |g, params| {
        let mode = ecc_mode_from_params(params);
        let r = rust_igraph::radius_with_mode(g, mode).expect("radius_with_mode");
        match r {
            Some(n) => serde_json::json!(n),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn diameter_with_mode_three_source_conformance() {
    run_conformance("diameter_with_mode", |g, params| {
        let mode = ecc_mode_from_params(params);
        let d = rust_igraph::diameter_with_mode(g, mode).expect("diameter_with_mode");
        match d {
            Some(n) => serde_json::json!(n),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn eccentricity_weighted_with_mode_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("eccentricity_weighted_with_mode");
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
            let mode = ecc_mode_from_params(&case.params);
            let r = rust_igraph::eccentricity_weighted_with_mode(&g, &weights, mode)
                .expect("eccentricity_weighted_with_mode");
            let rust_json: serde_json::Value = r.into_iter().map(serde_json::Value::from).collect();
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "eccentricity_weighted_with_mode");
            let _ = case.origin;
        }
    }
}

#[test]
fn radius_weighted_with_mode_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("radius_weighted_with_mode");
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
            let mode = ecc_mode_from_params(&case.params);
            let r = rust_igraph::radius_weighted_with_mode(&g, &weights, mode)
                .expect("radius_weighted_with_mode");
            let rust_json = match r {
                Some(v) => serde_json::json!(v),
                None => serde_json::Value::Null,
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "radius_weighted_with_mode");
            let _ = case.origin;
        }
    }
}

#[test]
fn diameter_weighted_with_mode_three_source_conformance() {
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("diameter_weighted_with_mode");
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
            let mode = ecc_mode_from_params(&case.params);
            let d = rust_igraph::diameter_weighted_with_mode(&g, &weights, mode)
                .expect("diameter_weighted_with_mode");
            let rust_json = match d {
                Some(v) => serde_json::json!(v),
                None => serde_json::Value::Null,
            };
            assert!(
                json_approx_eq(&rust_json, &case.expected),
                "{}: expected {} got {}",
                path.display(),
                case.expected,
                rust_json,
            );
            assert_eq!(case.source, src);
            assert_eq!(case.algo, "diameter_weighted_with_mode");
            let _ = case.origin;
        }
    }
}

#[test]
fn disjoint_union_many_three_source_conformance() {
    run_conformance("disjoint_union_many", |g, params| {
        let extras: Vec<GraphPayload> = serde_json::from_value(
            params
                .get("extra_graphs")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![])),
        )
        .expect("decode extra_graphs payload list");
        let extra_graphs: Vec<rust_igraph::Graph> = extras.iter().map(build_graph).collect();
        let mut refs: Vec<&rust_igraph::Graph> = Vec::with_capacity(1 + extra_graphs.len());
        refs.push(g);
        for eg in &extra_graphs {
            refs.push(eg);
        }
        let u = rust_igraph::disjoint_union_many(&refs).expect("disjoint_union_many");
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
fn global_efficiency_three_source_conformance() {
    run_conformance("global_efficiency", |g, _params| {
        let d = rust_igraph::global_efficiency(g).expect("global_efficiency");
        match d {
            Some(v) => serde_json::json!(v),
            None => serde_json::Value::Null,
        }
    });
}

#[test]
fn local_efficiency_two_source_conformance() {
    // python-igraph 0.11 does not expose `local_efficiency`, so the
    // py source genuinely cannot contribute fixtures.
    run_conformance_with_skip("local_efficiency", &["py"], |g, _params| {
        let v = rust_igraph::local_efficiency(g).expect("local_efficiency");
        serde_json::json!(v)
    });
}

#[test]
fn average_local_efficiency_two_source_conformance() {
    run_conformance_with_skip("average_local_efficiency", &["py"], |g, _params| {
        let v = rust_igraph::average_local_efficiency(g).expect("average_local_efficiency");
        serde_json::json!(v)
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

#[test]
fn convergence_degree_three_source_conformance() {
    // Per-edge convergence values; NaN encodes as JSON `null` to match
    // the upstream Python/R wire format (closeness uses the same trick).
    run_conformance("convergence_degree", |g, _params| {
        let r = rust_igraph::convergence_degree(g).expect("convergence_degree");
        let arr: Vec<serde_json::Value> = r
            .into_iter()
            .map(|x| {
                if x.is_nan() {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(x)
                }
            })
            .collect();
        serde_json::Value::Array(arr)
    });
}

#[test]
fn count_loops_three_source_conformance() {
    // Scalar count, no edge-id alignment issues.
    run_conformance("count_loops", |g, _params| {
        let n = rust_igraph::count_loops(g).expect("count_loops");
        serde_json::json!(n)
    });
}

#[test]
fn count_multiple_three_source_conformance() {
    // Per-edge multiplicity. Edge ids permute through the wire so we
    // compare as a sorted multiset — the multi-set of multiplicities is
    // an invariant of the underlying multigraph.
    run_conformance("count_multiple", |g, _params| {
        let mut v = rust_igraph::count_multiple(g).expect("count_multiple");
        v.sort_unstable();
        serde_json::json!(v)
    });
}

#[test]
fn count_adjacent_triangles_three_source_conformance() {
    // Per-vertex adjacent-triangle count. Vertex ids are stable through
    // the wire, so direct equality works without sorting.
    run_conformance("count_adjacent_triangles", |g, _params| {
        let v = rust_igraph::count_adjacent_triangles(g).expect("count_adjacent_triangles");
        serde_json::json!(v)
    });
}
