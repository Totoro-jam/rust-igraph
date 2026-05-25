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
#[allow(clippy::too_many_lines)] // three-source dispatch + variant enum + dual range
fn label_propagation_three_source_conformance() {
    // LPA partitions vary with shuffle order across implementations
    // (Raghavan–Albert–Kumara 2007, plus the Traag–Šubelj 2023 fast
    // variant), so the conformance harness asserts on (a) the
    // modularity-score window upstream attains and (b) the community
    // count window. `LpaResult` carries no quality field, so we compute
    // Q from the partition via `modularity_weighted` (which collapses
    // to the unweighted case under unit weights).
    use rust_igraph::{
        LpaOptions, LpaVariant, label_propagation_with_options, modularity, modularity_weighted,
    };

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("label_propagation");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse lpa fixture JSON");
            let g = build_graph(&case.graph);
            let variant = match case
                .params
                .get("variant")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("fast")
            {
                "fast" => LpaVariant::Fast,
                "dominance" => LpaVariant::Dominance,
                "retention" => LpaVariant::Retention,
                other => panic!("unknown lpa variant: {other}"),
            };
            let seed = case
                .params
                .get("seed")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let opts = LpaOptions {
                variant,
                seed,
                ..LpaOptions::default()
            };
            let r = label_propagation_with_options(&g, case.graph.weights.as_deref(), &opts)
                .expect("label_propagation_with_options");
            let q = match case.graph.weights.as_deref() {
                Some(w) => modularity_weighted(&g, &r.membership, 1.0, w)
                    .expect("modularity_weighted")
                    .unwrap_or(0.0),
                None => modularity(&g, &r.membership, 1.0)
                    .expect("modularity")
                    .unwrap_or(0.0),
            };
            let exp = case
                .expected
                .as_object()
                .expect("lpa `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
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
            assert_eq!(case.algo, "label_propagation");
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
            "no label_propagation fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + Q range + k range
fn edge_betweenness_community_three_source_conformance() {
    // Newman-Girvan edge-betweenness community detection is fully
    // deterministic (no PRNG), but the *exact* membership differs across
    // implementations whenever there is a ties amongst the highest-
    // betweenness edges. Both the C and the python/R bindings break ties
    // by smallest index; our port does the same, but graph ordering can
    // still differ, so we settle on the same Q/k envelope used for the
    // other community detectors.
    // `modularity_directed` falls through to `modularity` on undirected
    // graphs, so a single dispatch covers both fixture orientations.
    use rust_igraph::{edge_betweenness_community, modularity_directed};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("edge_betweenness_community");
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
                serde_json::from_slice(&bytes).expect("parse EB community fixture JSON");
            let g = build_graph(&case.graph);
            let r = edge_betweenness_community(&g).expect("edge_betweenness_community");
            // Best-Q membership is what the upstream community detectors
            // return; we recompute Q here so the test does not depend on
            // any drift between the dendrogram modularity and the
            // standalone `modularity()` implementation.
            let q = modularity_directed(&g, &r.membership, 1.0)
                .expect("modularity_directed")
                .unwrap_or(0.0);
            let exp = case
                .expected
                .as_object()
                .expect("EB community `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
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
            assert_eq!(case.algo, "edge_betweenness_community");
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
            "no edge_betweenness_community fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + Q range + k range
fn fast_greedy_modularity_three_source_conformance() {
    // Clauset-Newman-Moore (2004) fast greedy modularity. The dendrogram
    // construction is deterministic given a tie-break rule, but the rule
    // varies across ports (C uses original-id ordering, R sometimes uses
    // arbitrary heap order), so we settle on a Q/k envelope around the
    // upstream values reported in the C unit test.
    use rust_igraph::{fast_greedy_modularity, modularity};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("fast_greedy_modularity");
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
                serde_json::from_slice(&bytes).expect("parse fast_greedy_modularity fixture JSON");
            let g = build_graph(&case.graph);
            let r = fast_greedy_modularity(&g).expect("fast_greedy_modularity");
            // Best-Q membership is what the upstream community detectors
            // return; we recompute Q via the standalone modularity() to
            // remove any in-port drift between the dendrogram trajectory
            // and our standalone Q implementation.
            let q = modularity(&g, &r.membership, 1.0)
                .expect("modularity")
                .unwrap_or(0.0);
            let exp = case
                .expected
                .as_object()
                .expect("fast_greedy_modularity `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
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
            assert_eq!(case.algo, "fast_greedy_modularity");
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
            "no fast_greedy_modularity fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + weighted branch + Q range + k range
fn walktrap_three_source_conformance() {
    // Pons-Latapy (2005) random-walk community detection. The merge
    // trajectory depends on heap tie-breaks, so we keep a Q envelope
    // (recomputed via the standalone modularity) and a k range. The
    // weighted fixture (`algo == "walktrap_weighted"`) carries
    // `graph.weights` and recomputes Q with `modularity_weighted`.
    use rust_igraph::{modularity, modularity_weighted, walktrap, walktrap_weighted};
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("walktrap");
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
                serde_json::from_slice(&bytes).expect("parse walktrap fixture JSON");
            let g = build_graph(&case.graph);
            let (membership, nb_clusters, q) = match case.algo.as_str() {
                "walktrap" => {
                    let r = walktrap(&g).expect("walktrap");
                    let q = modularity(&g, &r.membership, 1.0)
                        .expect("modularity")
                        .unwrap_or(0.0);
                    (r.membership, r.nb_clusters, q)
                }
                "walktrap_weighted" => {
                    let weights = case
                        .graph
                        .weights
                        .clone()
                        .expect("weighted walktrap fixture must carry graph.weights");
                    let r = walktrap_weighted(&g, &weights).expect("walktrap_weighted");
                    let q = modularity_weighted(&g, &r.membership, 1.0, &weights)
                        .expect("modularity_weighted")
                        .unwrap_or(0.0);
                    (r.membership, r.nb_clusters, q)
                }
                other => panic!("unexpected walktrap fixture algo: {other}"),
            };
            let _ = membership;
            let exp = case
                .expected
                .as_object()
                .expect("walktrap `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
                q_min,
                q_max,
                case.origin,
            );
            assert!(
                (k_min..=k_max).contains(&nb_clusters),
                "{}: k = {} outside [{}, {}] (origin: {})",
                path.display(),
                nb_clusters,
                k_min,
                k_max,
                case.origin,
            );
            assert_eq!(case.source, src);
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
            "no walktrap fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + params decode + canonicalisation
fn community_to_membership_three_source_conformance() {
    // Pure-function helper that cuts a binary dendrogram. Fixtures
    // embed the dendrogram (merges + nodes + steps) in `params` and
    // the expected partition in `expected.membership/csize`. Cluster
    // labels can differ between implementations (python-igraph labels
    // by first-leaf-encounter order; our Rust impl labels by top-down
    // merge traversal), so the comparison is partition-equivalent:
    // both labelings are canonicalised to first-occurrence order over
    // vertices 0..n, and csize is compared as a multiset.
    use rust_igraph::community_to_membership;
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("community_to_membership");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "community_to_membership");

            let params = case.params.as_object().expect("params must be an object");
            let steps = u32::try_from(
                params
                    .get("steps")
                    .and_then(serde_json::Value::as_u64)
                    .expect("steps"),
            )
            .expect("steps fits u32");
            let merges_arr = params
                .get("merges")
                .and_then(serde_json::Value::as_array)
                .expect("merges array");
            let merges: Vec<[u32; 2]> = merges_arr
                .iter()
                .map(|row| {
                    let r = row.as_array().expect("merge row array");
                    let c1 =
                        u32::try_from(r.first().and_then(serde_json::Value::as_u64).expect("c1"))
                            .expect("c1 fits u32");
                    let c2 =
                        u32::try_from(r.get(1).and_then(serde_json::Value::as_u64).expect("c2"))
                            .expect("c2 fits u32");
                    [c1, c2]
                })
                .collect();
            let nodes = case.graph.n;

            let got =
                community_to_membership(&merges, nodes, steps).expect("community_to_membership");

            let exp = case
                .expected
                .as_object()
                .expect("expected must be an object");
            let exp_membership: Vec<u32> = exp
                .get("membership")
                .and_then(serde_json::Value::as_array)
                .expect("expected.membership")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("membership entry"))
                        .expect("membership entry fits u32")
                })
                .collect();
            let exp_csize: Vec<u32> = exp
                .get("csize")
                .and_then(serde_json::Value::as_array)
                .expect("expected.csize")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("csize entry")).expect("csize entry fits u32")
                })
                .collect();

            let got_canon = canonicalize_partition(&got.membership);
            let exp_canon = canonicalize_partition(&exp_membership);
            assert_eq!(
                got_canon,
                exp_canon,
                "{}: partition mismatch — got {:?}, expected {:?} (origin: {})",
                path.display(),
                got.membership,
                exp_membership,
                case.origin,
            );

            let mut got_sizes = got.csize.clone();
            got_sizes.sort_unstable();
            let mut exp_sizes = exp_csize.clone();
            exp_sizes.sort_unstable();
            assert_eq!(
                got_sizes,
                exp_sizes,
                "{}: csize multiset mismatch — got {:?}, expected {:?} (origin: {})",
                path.display(),
                got.csize,
                exp_csize,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no community_to_membership fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + params decode + canonicalisation
fn reindex_membership_three_source_conformance() {
    // Pure helper — densifies a membership vector to 0..k-1 using
    // first-occurrence ordering. Cluster *labels* across impls may
    // diverge (the C large-id fallback sorts ascending; R's wrapper
    // sometimes preserves sort order), so the comparison is
    // partition-equivalent. We also check that `new_to_old.len() ==
    // nb_clusters` matches between sources, and that the per-vertex
    // partition is identical to the input.
    use rust_igraph::reindex_membership;
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("reindex_membership");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "reindex_membership");

            let params = case.params.as_object().expect("params must be an object");
            let input: Vec<u32> = params
                .get("membership")
                .and_then(serde_json::Value::as_array)
                .expect("params.membership")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("membership entry"))
                        .expect("membership entry fits u32")
                })
                .collect();

            let got = reindex_membership(&input).expect("reindex_membership");

            let exp = case
                .expected
                .as_object()
                .expect("expected must be an object");
            let exp_membership: Vec<u32> = exp
                .get("membership")
                .and_then(serde_json::Value::as_array)
                .expect("expected.membership")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("membership entry"))
                        .expect("membership entry fits u32")
                })
                .collect();
            let exp_new_to_old: Vec<u32> = exp
                .get("new_to_old")
                .and_then(serde_json::Value::as_array)
                .expect("expected.new_to_old")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("new_to_old entry"))
                        .expect("new_to_old entry fits u32")
                })
                .collect();

            // Same number of clusters across impls.
            assert_eq!(
                got.nb_clusters() as usize,
                exp_new_to_old.len(),
                "{}: nb_clusters mismatch — got {}, expected {} (origin: {})",
                path.display(),
                got.nb_clusters(),
                exp_new_to_old.len(),
                case.origin,
            );

            // Partition equivalence: canonical labels under the
            // first-occurrence relabel match exactly.
            let got_canon = canonicalize_partition(&got.membership);
            let exp_canon = canonicalize_partition(&exp_membership);
            assert_eq!(
                got_canon,
                exp_canon,
                "{}: partition mismatch — got {:?}, expected {:?} (origin: {})",
                path.display(),
                got.membership,
                exp_membership,
                case.origin,
            );

            // The output also preserves the original partition over
            // the input.
            for i in 0..input.len() {
                for j in (i + 1)..input.len() {
                    assert_eq!(
                        input[i] == input[j],
                        got.membership[i] == got.membership[j],
                        "{}: partition diverged between input and output at ({i}, {j})",
                        path.display(),
                    );
                }
            }

            // Round-trip: new_to_old[got.membership[i]] == input[i].
            for (i, &old) in input.iter().enumerate() {
                let new = got.membership[i] as usize;
                assert_eq!(
                    got.new_to_old[new],
                    old,
                    "{}: new_to_old round-trip failed at index {i}",
                    path.display(),
                );
            }

            assert_eq!(case.source, src);
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
            "no reindex_membership fixtures from source {src}"
        );
    }
}

/// Relabel cluster ids in order of first occurrence over vertices
/// `0..n`, so two equivalent partitions with different cluster id
/// assignments compare equal. Used by
/// [`community_to_membership_three_source_conformance`] and
/// [`reindex_membership_three_source_conformance`].
fn canonicalize_partition(membership: &[u32]) -> Vec<u32> {
    let mut remap: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
    let mut next_id: u32 = 0;
    let mut out = Vec::with_capacity(membership.len());
    for &c in membership {
        let canon = *remap.entry(c).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        out.push(canon);
    }
    out
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + Q range + k range
fn edge_betweenness_community_weighted_three_source_conformance() {
    // Weighted Girvan-Newman dendrogram. Best-Q partition is sensitive
    // to tie-breaks across ports, so we accept a Q envelope (recomputed
    // via standalone modularity_weighted) and a k range — same shape as
    // the unweighted CO-006 oracle.
    // `modularity_weighted_directed` falls through to
    // `modularity_weighted` on undirected graphs, so one dispatch
    // covers both fixture orientations.
    use rust_igraph::{edge_betweenness_community_weighted, modularity_weighted_directed};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("edge_betweenness_community_weighted");
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
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse edge_betweenness_community_weighted fixture JSON");
            let g = build_graph(&case.graph);
            let weights = case
                .graph
                .weights
                .clone()
                .expect("weighted fixture must carry graph.weights");
            let r = edge_betweenness_community_weighted(&g, &weights)
                .expect("edge_betweenness_community_weighted");
            let q = modularity_weighted_directed(&g, &r.membership, 1.0, &weights)
                .expect("modularity_weighted_directed")
                .unwrap_or(0.0);
            let exp = case
                .expected
                .as_object()
                .expect("edge_betweenness_community_weighted `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: weighted Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
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
            assert_eq!(case.algo, "edge_betweenness_community_weighted");
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
            "no edge_betweenness_community_weighted fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + k pin + Q range
fn fluid_communities_three_source_conformance() {
    // Fluid Communities (Parés et al. 2017) is stochastic across
    // implementations; we pin `k` exactly (since k is a user input)
    // and accept a modularity-score window upstream attains.
    // `FluidResult` carries no quality field, so we compute Q from the
    // partition via `modularity`.
    use rust_igraph::{FluidOptions, fluid_communities_with_options, modularity};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("fluid_communities");
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
                serde_json::from_slice(&bytes).expect("parse fluid fixture JSON");
            let g = build_graph(&case.graph);
            let k = u32::try_from(
                case.params
                    .get("k")
                    .and_then(serde_json::Value::as_u64)
                    .expect("fluid params.k"),
            )
            .expect("k fits u32");
            let seed = case
                .params
                .get("seed")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let opts = FluidOptions {
                seed,
                ..FluidOptions::default()
            };
            let r = fluid_communities_with_options(&g, k, &opts)
                .expect("fluid_communities_with_options");
            let q = modularity(&g, &r.membership, 1.0)
                .expect("modularity")
                .unwrap_or(0.0);
            let exp = case
                .expected
                .as_object()
                .expect("fluid `expected` must be an object");
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
            assert!(
                q >= q_min - 1e-9 && q <= q_max + 1e-9,
                "{}: Q = {} outside [{}, {}] (origin: {})",
                path.display(),
                q,
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
            assert_eq!(case.algo, "fluid_communities");
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
            "no fluid_communities fixtures from source {src}"
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
fn ecc_three_source_conformance() {
    // ALGO-PR-031: edge clustering coefficient (Radicchi 2004).
    // python-igraph 0.11 and R-igraph both lack a user-facing
    // `ecc()` (R has the internal `ecc_impl()`); the corresponding
    // manifests are hand-derived parity fixtures.
    use rust_igraph::ecc;
    run_conformance("ecc", |g, params| {
        let k = u32::try_from(
            params
                .get("k")
                .and_then(serde_json::Value::as_u64)
                .expect("`k` param required"),
        )
        .expect("k fits in u32");
        let offset = params
            .get("offset")
            .and_then(serde_json::Value::as_bool)
            .expect("`offset` param required");
        let normalize = params
            .get("normalize")
            .and_then(serde_json::Value::as_bool)
            .expect("`normalize` param required");
        let eids: Option<Vec<u32>> = params.get("eids").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .map(|e| {
                        u32::try_from(
                            e.as_u64()
                                .expect("eids entries must be non-negative integers"),
                        )
                        .expect("eid fits in u32")
                    })
                    .collect()
            })
        });
        let values = ecc(g, eids.as_deref(), k, offset, normalize).expect("ecc");
        // Encode NaN as JSON null — the fixtures use null for NaN
        // since `serde_json` rejects NaN in floats.
        serde_json::Value::Array(
            values
                .into_iter()
                .map(|v| {
                    if v.is_nan() {
                        serde_json::Value::Null
                    } else {
                        serde_json::json!(v)
                    }
                })
                .collect(),
        )
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

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 5 method decode
fn compare_communities_three_source_conformance() {
    // Partition-distance helper. Each fixture carries two membership
    // vectors and the method name; the expected value is a scalar f64
    // that we compare within 1e-9 tolerance (Meilă/Danon/Hubert-Arabie
    // formulas are closed-form rationals, so the only float drift comes
    // from log2 rounding).
    use rust_igraph::{CommunityComparison, compare_communities};
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("compare_communities");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "compare_communities");

            let params = case.params.as_object().expect("params must be an object");
            let comm1: Vec<u32> = params
                .get("comm1")
                .and_then(serde_json::Value::as_array)
                .expect("params.comm1")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("comm1 entry")).expect("comm1 entry fits u32")
                })
                .collect();
            let comm2: Vec<u32> = params
                .get("comm2")
                .and_then(serde_json::Value::as_array)
                .expect("params.comm2")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("comm2 entry")).expect("comm2 entry fits u32")
                })
                .collect();
            let method_str = params
                .get("method")
                .and_then(serde_json::Value::as_str)
                .expect("params.method");
            // Accept both Rust CamelCase (the R-igraph manifest, for
            // consistency with the public enum) and the upstream
            // snake_case spelling used in igraph C / python-igraph.
            let method = match method_str {
                "VariationOfInformation" | "vi" | "variation_of_information" => {
                    CommunityComparison::VariationOfInformation
                }
                "NormalizedMutualInformation" | "nmi" | "normalized_mutual_information" => {
                    CommunityComparison::NormalizedMutualInformation
                }
                "SplitJoin" | "split_join" => CommunityComparison::SplitJoin,
                "Rand" | "rand" => CommunityComparison::Rand,
                "AdjustedRand" | "adjusted_rand" => CommunityComparison::AdjustedRand,
                other => panic!("unknown method {other} in {}", path.display()),
            };

            let got = compare_communities(&comm1, &comm2, method).expect("compare_communities");
            // Accept either a bare scalar (R manifest) or `{"value": f64}`
            // (C and py manifests).
            let expected = case
                .expected
                .as_f64()
                .or_else(|| {
                    case.expected
                        .as_object()
                        .and_then(|m| m.get("value"))
                        .and_then(serde_json::Value::as_f64)
                })
                .expect("expected must be f64 or {value: f64}");

            assert!(
                (got - expected).abs() < 1e-9,
                "{}: {method_str} mismatch — got {got}, expected {expected} (origin: {})",
                path.display(),
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no compare_communities fixtures from source {src}"
        );
    }
}

#[test]
fn split_join_distance_three_source_conformance() {
    // `split_join_distance` returns the asymmetric pair (d12, d21);
    // every source emits `expected = {"d12": u64, "d21": u64}`.
    use rust_igraph::split_join_distance;
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("split_join_distance");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "split_join_distance");

            let params = case.params.as_object().expect("params must be an object");
            let comm1: Vec<u32> = params
                .get("comm1")
                .and_then(serde_json::Value::as_array)
                .expect("params.comm1")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("comm1 entry")).expect("comm1 entry fits u32")
                })
                .collect();
            let comm2: Vec<u32> = params
                .get("comm2")
                .and_then(serde_json::Value::as_array)
                .expect("params.comm2")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("comm2 entry")).expect("comm2 entry fits u32")
                })
                .collect();

            let got = split_join_distance(&comm1, &comm2).expect("split_join_distance");

            let exp_obj = case
                .expected
                .as_object()
                .expect("expected must be {d12, d21}");
            let exp_d12 = exp_obj
                .get("d12")
                .and_then(serde_json::Value::as_u64)
                .expect("expected.d12");
            let exp_d21 = exp_obj
                .get("d21")
                .and_then(serde_json::Value::as_u64)
                .expect("expected.d21");

            assert_eq!(
                got.d12,
                exp_d12,
                "{}: d12 mismatch — got {}, expected {} (origin: {})",
                path.display(),
                got.d12,
                exp_d12,
                case.origin,
            );
            assert_eq!(
                got.d21,
                exp_d21,
                "{}: d21 mismatch — got {}, expected {} (origin: {})",
                path.display(),
                got.d21,
                exp_d21,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no split_join_distance fixtures from source {src}"
        );
    }
}

#[test]
fn voronoi_three_source_conformance() {
    // `voronoi` (ALGO-SP-007) needs four params from each fixture:
    // - generators: array of u32 vertex ids
    // - mode: "out" | "in" | "all" (DijkstraMode)
    // - tiebreaker: "first" | "last" | "random"
    // - weights (optional): pulled from graph.weights
    //
    // RANDOM tiebreaker fixtures are intentionally not extracted — the C
    // reference uses Mersenne Twister seeded with 42 while our SplitMix64
    // produces different tie selections at the same dilation. The runner
    // still supports tiebreaker="random" so a fixture could opt in later.
    use rust_igraph::{DijkstraMode, VoronoiTiebreaker, voronoi};
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("voronoi");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "voronoi");

            let g = build_graph(&case.graph);
            let weights = case.graph.weights.clone();

            let params = case.params.as_object().expect("params must be an object");
            let generators: Vec<u32> = params
                .get("generators")
                .and_then(serde_json::Value::as_array)
                .expect("params.generators")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("generator id")).expect("generator id fits u32")
                })
                .collect();
            let mode = match params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .expect("params.mode")
            {
                "out" => DijkstraMode::Out,
                "in" => DijkstraMode::In,
                "all" => DijkstraMode::All,
                other => panic!("{}: unknown mode {other}", path.display()),
            };
            let tiebreaker = match params
                .get("tiebreaker")
                .and_then(serde_json::Value::as_str)
                .expect("params.tiebreaker")
            {
                "first" => VoronoiTiebreaker::First,
                "last" => VoronoiTiebreaker::Last,
                "random" => VoronoiTiebreaker::Random,
                other => panic!("{}: unknown tiebreaker {other}", path.display()),
            };

            let got = voronoi(&g, weights.as_deref(), mode, &generators, tiebreaker, 42)
                .expect("voronoi");

            // Encode actual result the way fixtures encode it: Inf → null,
            // unreachable membership → null. Then compare via json_approx_eq
            // so the 1-ULP scale tolerance applies to weighted-distance
            // cases (none today, but a future fixture set might add some).
            let mem: Vec<serde_json::Value> = got
                .membership
                .iter()
                .map(|o| match o {
                    Some(i) => serde_json::json!(i),
                    None => serde_json::Value::Null,
                })
                .collect();
            let dist: Vec<serde_json::Value> = got
                .distances
                .iter()
                .map(|d| {
                    if d.is_infinite() {
                        serde_json::Value::Null
                    } else {
                        serde_json::json!(d)
                    }
                })
                .collect();
            let actual = serde_json::json!({
                "membership": mem,
                "distances": dist,
            });

            assert!(
                json_approx_eq(&actual, &case.expected),
                "{}: voronoi mismatch\n  origin:   {}\n  actual:   {}\n  expected: {}",
                path.display(),
                case.origin,
                actual,
                case.expected,
            );

            assert_eq!(case.source, src);
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
            "no voronoi fixtures from source {src}"
        );
    }
}

#[test]
fn community_voronoi_three_source_conformance() {
    // `community_voronoi` (ALGO-CO-009) — assertions are on `generators`
    // (a deterministic ordered list driven by greedy LRD descent) and
    // `community_count` (distinct values in the membership vector).
    //
    // Raw membership labels are NOT compared: the inner `voronoi` call
    // uses a RANDOM tiebreaker seeded with 42 (matching the C
    // reference), but our SplitMix64 does not produce identical tie
    // selections to C's Mersenne Twister, so labels can drift for
    // vertices that are equidistant to multiple generators. The
    // generator list is unaffected by the tiebreaker — it is selected
    // by LRD ordering before voronoi is called — and the number of
    // communities equals `generators.len()` for the non-degenerate
    // cases, so both invariants are reproducible.
    use rust_igraph::{DijkstraMode, community_voronoi};
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("community_voronoi");
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
            let case: Conformance = serde_json::from_slice(&bytes).expect("parse fixture JSON");
            assert_eq!(case.algo, "community_voronoi");

            let g = build_graph(&case.graph);

            let params = case.params.as_object().expect("params must be an object");
            let mode = match params
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .expect("params.mode")
            {
                "out" => DijkstraMode::Out,
                "in" => DijkstraMode::In,
                "all" => DijkstraMode::All,
                other => panic!("{}: unknown mode {other}", path.display()),
            };
            let r = params
                .get("r")
                .and_then(serde_json::Value::as_f64)
                .expect("params.r");

            let got = community_voronoi(&g, None, None, mode, r).expect("community_voronoi");

            let expected_gens: Vec<u32> = case
                .expected
                .get("generators")
                .and_then(serde_json::Value::as_array)
                .expect("expected.generators")
                .iter()
                .map(|v| {
                    u32::try_from(v.as_u64().expect("generator id")).expect("generator id fits u32")
                })
                .collect();
            assert_eq!(
                got.generators,
                expected_gens,
                "{}: generator list mismatch\n  origin:   {}\n  actual:   {:?}\n  expected: {:?}",
                path.display(),
                case.origin,
                got.generators,
                expected_gens,
            );

            let expected_count = usize::try_from(
                case.expected
                    .get("community_count")
                    .and_then(serde_json::Value::as_u64)
                    .expect("expected.community_count"),
            )
            .expect("expected.community_count fits usize");
            let distinct: std::collections::BTreeSet<u32> =
                got.membership.iter().copied().collect();
            assert_eq!(
                distinct.len(),
                expected_count,
                "{}: distinct community count mismatch\n  origin:   {}\n  actual:   {} ({:?})\n  expected: {}",
                path.display(),
                case.origin,
                distinct.len(),
                distinct,
                expected_count,
            );

            assert_eq!(case.source, src);
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
            "no community_voronoi fixtures from source {src}"
        );
    }
}

#[test]
fn minimum_spanning_tree_three_source_conformance() {
    // MST needs the per-fixture weights vector (lives on graph payload,
    // not in params) and the `method` selector. We compare on the
    // matroid invariant — total weight + edge count — instead of exact
    // edge IDs, so multiple equally-light spanning trees don't trip
    // the harness.
    use rust_igraph::{MstAlgorithm, minimum_spanning_tree};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("minimum_spanning_tree");
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
            assert_eq!(case.algo, "minimum_spanning_tree");
            let g = build_graph(&case.graph);
            let weights_owned = case.graph.weights.clone();
            let weights_opt: Option<&[f64]> = weights_owned.as_deref();
            let method_str = case
                .params
                .get("method")
                .and_then(|v| v.as_str())
                .expect("method param missing");
            let method = match method_str {
                "automatic" => MstAlgorithm::Automatic,
                "unweighted" => MstAlgorithm::Unweighted,
                "prim" => MstAlgorithm::Prim,
                "kruskal" => MstAlgorithm::Kruskal,
                other => panic!("unknown MST method '{other}' in {}", path.display()),
            };
            let edges = minimum_spanning_tree(&g, weights_opt, method)
                .expect("minimum_spanning_tree should succeed on conformance graphs");
            // Total weight: sum of weights[e] if weights provided, else
            // unit-weight (matches our fixtures' total_weight = edge_count
            // for unweighted cases).
            let total_weight: f64 = match weights_opt {
                Some(w) => edges.iter().map(|&e| w[e as usize]).sum(),
                #[allow(clippy::cast_precision_loss)]
                None => edges.len() as f64,
            };
            let actual = serde_json::json!({
                "total_weight": total_weight,
                "edge_count": edges.len(),
            });
            assert!(
                json_approx_eq(&actual, &case.expected),
                "minimum_spanning_tree conformance failure\n  fixture: {}\n  source:  {}\n  origin:  {}\n  actual:   {}\n  expected: {}",
                path.display(),
                case.source,
                case.origin,
                actual,
                case.expected,
            );
            assert_eq!(case.source, src);
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
            "no minimum_spanning_tree fixtures from source {src}"
        );
    }
}

/// Helper: extract a typed param from the conformance JSON. Panics
/// with a fixture path on missing/wrong-type so failures point at the
/// offending fixture rather than producing an opaque `serde` error.
fn er_param_u32(case: &Conformance, key: &str, path: &std::path::Path) -> u32 {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: param `{}` missing or not u32",
                path.display(),
                key
            )
        })
}

fn er_param_u64(case: &Conformance, key: &str, path: &std::path::Path) -> u64 {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: param `{}` missing or not u64",
                path.display(),
                key
            )
        })
}

fn er_param_f64(case: &Conformance, key: &str, path: &std::path::Path) -> f64 {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: param `{}` missing or not f64",
                path.display(),
                key
            )
        })
}

fn er_param_bool(case: &Conformance, key: &str, path: &std::path::Path) -> bool {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: param `{}` missing or not bool",
                path.display(),
                key
            )
        })
}

fn er_expected_u32(case: &Conformance, key: &str, path: &std::path::Path) -> u32 {
    let v = case
        .expected
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: expected.{} missing or not u64",
                path.display(),
                key
            )
        });
    u32::try_from(v).unwrap_or_else(|_| {
        panic!(
            "ER fixture {}: expected.{} = {} doesn't fit in u32",
            path.display(),
            key,
            v
        )
    })
}

fn er_expected_u64(case: &Conformance, key: &str, path: &std::path::Path) -> u64 {
    case.expected
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: expected.{} missing or not u64",
                path.display(),
                key
            )
        })
}

fn er_expected_bool(case: &Conformance, key: &str, path: &std::path::Path) -> bool {
    case.expected
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_else(|| {
            panic!(
                "ER fixture {}: expected.{} missing or not bool",
                path.display(),
                key
            )
        })
}

#[test]
fn erdos_renyi_gnp_three_source_conformance() {
    // ER is a *generator* — no input graph, no portable RNG seed across
    // implementations. Each fixture's `params` carries (n, p, directed,
    // loops, seed) and `expected` carries the structural invariants we
    // can check independently of the RNG: vcount exact, ecount inside
    // a ±6σ Binomial band, directed flag exact.
    use rust_igraph::erdos_renyi_gnp;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("erdos_renyi_gnp");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "erdos_renyi_gnp");

            let n = er_param_u32(&case, "n", &path);
            let p = er_param_f64(&case, "p", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = erdos_renyi_gnp(n, p, directed, loops, seed)
                .expect("erdos_renyi_gnp should succeed on conformance fixtures");

            let got_vertices = graph.vcount();
            let got_edges = graph.ecount() as u64;

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let edges_low = er_expected_u64(&case, "ecount_min", &path);
            let edges_high = er_expected_u64(&case, "ecount_max", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                got_vertices,
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert!(
                got_edges >= edges_low && got_edges <= edges_high,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                got_edges,
                edges_low,
                edges_high,
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no erdos_renyi_gnp fixtures from source {src}"
        );
    }
}

#[test]
fn erdos_renyi_gnm_three_source_conformance() {
    // G(n, m) samples without replacement → ecount is a *sharp*
    // constraint (equals m), not a band. vcount and directed are also
    // exact.
    use rust_igraph::erdos_renyi_gnm;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("erdos_renyi_gnm");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "erdos_renyi_gnm");

            let n = er_param_u32(&case, "n", &path);
            let m = er_param_u64(&case, "m", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = erdos_renyi_gnm(n, m, directed, loops, seed)
                .expect("erdos_renyi_gnm should succeed on conformance fixtures");

            let got_vertices = graph.vcount();
            let got_edges = graph.ecount() as u64;

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                got_vertices,
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                got_edges,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no erdos_renyi_gnm fixtures from source {src}"
        );
    }
}

#[test]
fn barabasi_game_bag_three_source_conformance() {
    // BA-BAG is a *generator* like ER. Cross-implementation seed
    // portability is impossible (each library uses its own RNG), so the
    // expected block carries structural invariants only:
    //   * vcount: exact match with params.n
    //   * ecount: exact match with (n - 1) * m — BAG is deterministic
    //     in edge count when m is a scalar (barabasi.c:113-117)
    //   * directed: exact boolean
    //   * ba_temporal_order: every edge (src, dst) satisfies dst < src
    //     (BA edges always point from new vertex to an earlier one,
    //     barabasi.c:158-170)
    use rust_igraph::barabasi_game_bag;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("barabasi_game_bag");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "barabasi_game_bag");

            let n = er_param_u32(&case, "n", &path);
            let m = er_param_u32(&case, "m", &path);
            let outpref = er_param_bool(&case, "outpref", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = barabasi_game_bag(n, m, outpref, directed, seed)
                .expect("barabasi_game_bag should succeed on conformance fixtures");

            let got_vertices = graph.vcount();
            let got_edges = graph.ecount() as u64;

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_temporal = er_expected_bool(&case, "ba_temporal_order", &path);

            assert_eq!(
                got_vertices,
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                got_edges,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if want_temporal {
                // Directed: BA edges go from new vertex to older
                // (`dst < src` strictly). Undirected: the storage layer
                // canonicalises to `(min, max)`, so the temporal order
                // is lost — assert the no-self-loop invariant that
                // BAG's sample-before-push order guarantees instead.
                let n_edges =
                    u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
                for eid in 0..n_edges {
                    let (src_v, dst_v) = graph.edge(eid).expect("conformance edge id in bounds");
                    if graph.is_directed() {
                        assert!(
                            dst_v < src_v,
                            "BA temporal-order violation in {}: edge ({src_v} -> {dst_v})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    } else {
                        assert_ne!(
                            src_v,
                            dst_v,
                            "BA-BAG must not produce self-loops in {}: edge ({src_v}, {dst_v})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                }
            }

            assert_eq!(case.source, src);
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
            "no barabasi_game_bag fixtures from source {src}"
        );
    }
}

#[test]
fn growing_random_game_three_source_conformance() {
    // Growing-random is a *generator*; expected block is structural only:
    //   * vcount: exact match with params.n
    //   * ecount: exact (n - 1) * m
    //   * directed: exact boolean
    //   * ba_temporal_order: only set when citation=true. Same canonical-
    //     storage caveat applies for undirected: the temporal order is
    //     lost, but the citation kernel never picks (i, i), so the
    //     "no self-loops" invariant survives canonicalisation.
    use rust_igraph::growing_random_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("growing_random_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "growing_random_game");

            let n = er_param_u32(&case, "n", &path);
            let m = er_param_u32(&case, "m", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let citation = er_param_bool(&case, "citation", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = growing_random_game(n, m, directed, citation, seed)
                .expect("growing_random_game should succeed on conformance fixtures");

            let got_vertices = graph.vcount();
            let got_edges = graph.ecount() as u64;

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_temporal = er_expected_bool(&case, "ba_temporal_order", &path);

            assert_eq!(
                got_vertices,
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                got_edges,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if want_temporal {
                let n_edges =
                    u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
                for eid in 0..n_edges {
                    let (src_v, dst_v) = graph.edge(eid).expect("conformance edge id in bounds");
                    if graph.is_directed() {
                        assert!(
                            dst_v < src_v,
                            "citation temporal-order violation in {}: edge ({src_v} -> {dst_v})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    } else {
                        assert_ne!(
                            src_v,
                            dst_v,
                            "citation mode must not produce self-loops in {}: edge ({src_v}, {dst_v})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                }
            }

            assert_eq!(case.source, src);
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
            "no growing_random_game fixtures from source {src}"
        );
    }
}

struct TreeUnionFind {
    parent: Vec<u32>,
}

impl TreeUnionFind {
    fn new(n: u32) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }
    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let p = self.parent[x as usize];
            self.parent[x as usize] = self.parent[p as usize];
            x = self.parent[x as usize];
        }
        x
    }
    fn union(&mut self, a: u32, b: u32) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            false
        } else {
            self.parent[ra as usize] = rb;
            true
        }
    }
}

fn assert_spanning_tree(
    graph: &rust_igraph::Graph,
    path: &std::path::Path,
    source: &str,
    origin: &str,
) {
    let n_vertices = graph.vcount();
    let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
    let mut uf = TreeUnionFind::new(n_vertices);
    for eid in 0..n_edges {
        let (a, b) = graph.edge(eid).expect("conformance edge id in bounds");
        assert_ne!(
            a,
            b,
            "tree must not contain self-loop in {}\n  source: {source}\n  origin: {origin}",
            path.display(),
        );
        assert!(
            uf.union(a, b),
            "tree must not contain a cycle in {}: edge ({a}, {b}) closed one\n  source: {source}\n  origin: {origin}",
            path.display(),
        );
    }
    if n_vertices > 0 {
        let root = uf.find(0);
        for v in 1..n_vertices {
            assert_eq!(
                uf.find(v),
                root,
                "tree must be connected in {}: vertex {v} is in a different component\n  source: {source}\n  origin: {origin}",
                path.display(),
            );
        }
    }
}

#[test]
fn tree_game_lerw_three_source_conformance() {
    // Wilson LERW spanning-tree generator. Structural invariants only —
    // RNG state is not portable across implementations:
    //   * vcount = params.n (exact)
    //   * ecount = max(0, n - 1) (exact spanning-tree edge count)
    //   * directed flag exact
    //   * is_tree: the edge set is acyclic AND connected when projected
    //     onto the undirected graph (union-find check).
    use rust_igraph::tree_game_lerw;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("tree_game_lerw");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "tree_game_lerw");

            let n = er_param_u32(&case, "n", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = tree_game_lerw(n, directed, seed)
                .expect("tree_game_lerw should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_tree = er_expected_bool(&case, "is_tree", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_tree {
                assert_spanning_tree(&graph, &path, &case.source, &case.origin);
            }

            assert_eq!(case.source, src);
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
            "no tree_game_lerw fixtures from source {src}"
        );
    }
}

#[test]
fn grg_game_three_source_conformance() {
    // Geometric random graph generator. Structural invariants only —
    // RNG state is not portable across implementations:
    //   * vcount = params.n (exact)
    //   * directed flag exact (always false in upstream)
    //   * is_simple: no self-loops and no multi-edges (HashSet size)
    //   * ecount_min <= ecount <= ecount_max (loose RNG-tolerant band
    //     centred on E[edges] = n(n-1)/2 · π·r² for the plane interior)
    use rust_igraph::grg_game;
    use std::collections::HashSet;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("grg_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "grg_game");

            let n = er_param_u32(&case, "n", &path);
            let radius = er_param_f64(&case, "radius", &path);
            let torus = er_param_bool(&case, "torus", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = grg_game(n, radius, torus, seed)
                .expect("grg_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_simple {
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for grg fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no grg_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn forest_fire_game_three_source_conformance() {
    // Leskovec–Kleinberg–Faloutsos forest-fire model. RNG state is not
    // portable across implementations, so we check structural
    // invariants only:
    //   * vcount = params.n (exact)
    //   * directed flag exact
    //   * is_simple: every src → dst burn is stamped, so no self-loops
    //     and no parallel edges (HashSet canonical-pair check)
    //   * ecount_min <= ecount <= ecount_max (loose RNG-tolerant band;
    //     lower bound = max(n-1, 0) when ambs > 0, since each new node
    //     contributes at least one fresh citation)
    use rust_igraph::forest_fire_game;
    use std::collections::HashSet;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("forest_fire_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "forest_fire_game");

            let n = er_param_u32(&case, "n", &path);
            let fw_prob = er_param_f64(&case, "fw_prob", &path);
            let bw_factor = er_param_f64(&case, "bw_factor", &path);
            let ambs = er_param_u32(&case, "ambs", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = forest_fire_game(n, fw_prob, bw_factor, ambs, directed, seed)
                .expect("forest_fire_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_simple {
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for forest_fire fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no forest_fire_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn simple_interconnected_islands_game_three_source_conformance() {
    // Inter-connected Erdős–Rényi islands. RNG state is not portable
    // across implementations, so we check structural invariants only:
    //   * vcount = islands_n * islands_size (exact)
    //   * directed = false (model is always undirected)
    //   * is_simple: intra slice is strictly upper-triangular, inter
    //     slice samples a disjoint bipartite cell — no self-loops, no
    //     parallel edges (HashSet canonical-pair check)
    //   * ecount_min <= ecount <= ecount_max where the band is built
    //     from E[intra] = islands_n · C(size, 2) · pin and exact_inter
    //     = C(islands_n, 2) · n_inter
    use rust_igraph::simple_interconnected_islands_game;
    use std::collections::HashSet;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("simple_interconnected_islands_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "simple_interconnected_islands_game");

            let islands_n = er_param_u32(&case, "islands_n", &path);
            let islands_size = er_param_u32(&case, "islands_size", &path);
            let islands_pin = er_param_f64(&case, "islands_pin", &path);
            let n_inter = er_param_u32(&case, "n_inter", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = simple_interconnected_islands_game(
                islands_n,
                islands_size,
                islands_pin,
                n_inter,
                seed,
            )
            .expect("simple_interconnected_islands_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_simple {
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for islands fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no simple_interconnected_islands_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn k_regular_game_three_source_conformance() {
    // K-regular sampler. RNG state is not portable across
    // implementations, so we check structural invariants only:
    //   * vcount = n (exact)
    //   * directed matches the param flag
    //   * ecount = n*k/2 (undirected) or n*k (directed) — recorded as
    //     an exact ecount band per fixture
    //   * every_degree (undirected) or every_out_degree/every_in_degree
    //     (directed) — every vertex hits exactly k
    //   * is_simple: when multiple=false, no self-loops and no parallel
    //     edges (HashSet canonical-pair check)
    use rust_igraph::k_regular_game;
    use std::collections::HashSet;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("k_regular_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "k_regular_game");

            let n = er_param_u32(&case, "n", &path);
            let k = er_param_u32(&case, "k", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let multiple = er_param_bool(&case, "multiple", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = k_regular_game(n, k, directed, multiple, seed)
                .expect("k_regular_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let vcount_usize = graph.vcount() as usize;
            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");

            if want_is_simple {
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for k_regular fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if directed || a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            if directed {
                let want_out = case
                    .expected
                    .get("every_out_degree")
                    .and_then(serde_json::Value::as_u64);
                let want_in = case
                    .expected
                    .get("every_in_degree")
                    .and_then(serde_json::Value::as_u64);
                if let (Some(out_k), Some(in_k)) = (want_out, want_in) {
                    let mut out_deg = vec![0u64; vcount_usize];
                    let mut in_deg = vec![0u64; vcount_usize];
                    for eid in 0..n_edges {
                        let (u, v) = graph
                            .edge(eid)
                            .expect("edge id within bounds for k_regular fixture");
                        out_deg[u as usize] += 1;
                        in_deg[v as usize] += 1;
                    }
                    for (v, d) in out_deg.iter().enumerate() {
                        assert_eq!(
                            *d,
                            out_k,
                            "out-degree of vertex {v} = {d}, expected {out_k} in {}\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                    for (v, d) in in_deg.iter().enumerate() {
                        assert_eq!(
                            *d,
                            in_k,
                            "in-degree of vertex {v} = {d}, expected {in_k} in {}\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                }
            } else if let Some(every_deg) = case
                .expected
                .get("every_degree")
                .and_then(serde_json::Value::as_u64)
            {
                let mut deg = vec![0u64; vcount_usize];
                for eid in 0..n_edges {
                    let (u, v) = graph
                        .edge(eid)
                        .expect("edge id within bounds for k_regular fixture");
                    deg[u as usize] += 1;
                    // Self-loop contributes 2 to undirected degree; both
                    // increments coincide when u == v.
                    deg[v as usize] += 1;
                }
                for (v, d) in deg.iter().enumerate() {
                    assert_eq!(
                        *d,
                        every_deg,
                        "degree of vertex {v} = {d}, expected {every_deg} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no k_regular_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn watts_strogatz_game_three_source_conformance() {
    // Watts-Strogatz 1-D small-world sampler. RNG state is not portable
    // across implementations, so only structural invariants are checked:
    //   * vcount = size (exact)
    //   * directed = false (model is always undirected)
    //   * ecount = size*nei (rewire preserves edge count, never adds /
    //     drops edges)
    //   * every_degree (when present) — undirected degree per vertex
    //   * is_simple: when multiple=false and loops=false, no self-loops
    //     and no parallel edges (HashSet canonical-pair check)
    use rust_igraph::watts_strogatz_game;
    use std::collections::HashSet;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("watts_strogatz_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "watts_strogatz_game");

            let size = er_param_u32(&case, "size", &path);
            let nei = er_param_u32(&case, "nei", &path);
            let p = er_param_f64(&case, "p", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let multiple = er_param_bool(&case, "multiple", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = watts_strogatz_game(size, nei, p, loops, multiple, seed)
                .expect("watts_strogatz_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let vcount_usize = graph.vcount() as usize;
            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");

            if want_is_simple {
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for watts fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            if let Some(every_deg) = case
                .expected
                .get("every_degree")
                .and_then(serde_json::Value::as_u64)
            {
                let mut deg = vec![0u64; vcount_usize];
                for eid in 0..n_edges {
                    let (u, v) = graph
                        .edge(eid)
                        .expect("edge id within bounds for watts fixture");
                    deg[u as usize] += 1;
                    deg[v as usize] += 1;
                }
                for (v, d) in deg.iter().enumerate() {
                    assert_eq!(
                        *d,
                        every_deg,
                        "degree of vertex {v} = {d}, expected {every_deg} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no watts_strogatz_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn sbm_game_three_source_conformance() {
    // Stochastic Block Model sampler. RNG state is not portable across
    // implementations, so each fixture pins parameters and bands the
    // structural invariants:
    //   * vcount = sum(block_sizes) (exact);
    //   * directed matches the flag;
    //   * ecount lies in a generous band [ecount_min, ecount_max];
    //   * when expected.is_simple = true, no self-loops and no
    //     parallel edges;
    //   * when expected.diagonal_only_pref = true, every edge stays
    //     inside a single block (only meaningful when the manifest's
    //     pref matrix is block-diagonal).
    use rust_igraph::sbm_game;
    use std::collections::HashSet;

    fn parse_pref_matrix(case: &Conformance, path: &std::path::Path) -> Vec<Vec<f64>> {
        let rows = case
            .params
            .get("pref_matrix")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "SBM fixture {}: param `pref_matrix` missing or not array",
                    path.display()
                )
            });
        rows.iter()
            .map(|row| {
                row.as_array()
                    .unwrap_or_else(|| {
                        panic!("SBM fixture {}: pref_matrix row not array", path.display())
                    })
                    .iter()
                    .map(|cell| {
                        cell.as_f64().unwrap_or_else(|| {
                            panic!("SBM fixture {}: pref_matrix cell not f64", path.display())
                        })
                    })
                    .collect()
            })
            .collect()
    }

    fn parse_block_sizes(case: &Conformance, path: &std::path::Path) -> Vec<u32> {
        case.params
            .get("block_sizes")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "SBM fixture {}: param `block_sizes` missing or not array",
                    path.display()
                )
            })
            .iter()
            .map(|cell| {
                let v = cell.as_u64().unwrap_or_else(|| {
                    panic!("SBM fixture {}: block_sizes cell not u64", path.display())
                });
                u32::try_from(v).unwrap_or_else(|_| {
                    panic!(
                        "SBM fixture {}: block_sizes cell {} does not fit in u32",
                        path.display(),
                        v
                    )
                })
            })
            .collect()
    }

    fn block_of(v: u32, offsets: &[u32]) -> usize {
        // `offsets` has `k + 1` entries: [0, s0, s0+s1, ..., n].
        // Returns the largest i with offsets[i] <= v.
        for (i, &boundary) in offsets.iter().enumerate().skip(1) {
            if v < boundary {
                return i - 1;
            }
        }
        offsets.len().saturating_sub(2)
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("sbm_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "sbm_game");

            let pref_matrix = parse_pref_matrix(&case, &path);
            let block_sizes = parse_block_sizes(&case, &path);
            let directed = er_param_bool(&case, "directed", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let multiple = er_param_bool(&case, "multiple", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = sbm_game(&pref_matrix, &block_sizes, directed, loops, multiple, seed)
                .expect("sbm_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");

            if want_is_simple {
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for sbm fixture");
                    assert_ne!(
                        a,
                        b,
                        "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                    let pair = if directed || a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            if let Some(true) = case
                .expected
                .get("diagonal_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                let mut offsets: Vec<u32> = Vec::with_capacity(block_sizes.len() + 1);
                offsets.push(0);
                let mut acc: u32 = 0;
                for &s in &block_sizes {
                    acc = acc.checked_add(s).expect("block-size sum fits in u32");
                    offsets.push(acc);
                }
                for eid in 0..n_edges {
                    let (u, v) = graph
                        .edge(eid)
                        .expect("edge id within bounds for sbm fixture");
                    let bu = block_of(u, &offsets);
                    let bv = block_of(v, &offsets);
                    assert_eq!(
                        bu,
                        bv,
                        "edge {u}-{v} crosses blocks ({bu} vs {bv}) in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no sbm_game fixtures from source {src}"
        );
    }
}

/// Helper: parse a JSON array-of-numbers param into Vec<f64>.
fn er_param_f64_vec(case: &Conformance, key: &str, path: &std::path::Path) -> Vec<f64> {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "HSBM fixture {}: param `{}` missing or not array",
                path.display(),
                key
            )
        })
        .iter()
        .map(|cell| {
            cell.as_f64().unwrap_or_else(|| {
                panic!(
                    "HSBM fixture {}: param `{}` cell is not f64",
                    path.display(),
                    key
                )
            })
        })
        .collect()
}

/// Helper: parse a JSON array-of-arrays-of-numbers param into Vec<Vec<f64>>.
fn er_param_f64_matrix(case: &Conformance, key: &str, path: &std::path::Path) -> Vec<Vec<f64>> {
    case.params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "HSBM fixture {}: param `{}` missing or not array-of-arrays",
                path.display(),
                key
            )
        })
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap_or_else(|| {
                    panic!(
                        "HSBM fixture {}: param `{}` row is not array",
                        path.display(),
                        key
                    )
                })
                .iter()
                .map(|cell| {
                    cell.as_f64().unwrap_or_else(|| {
                        panic!(
                            "HSBM fixture {}: param `{}` cell is not f64",
                            path.display(),
                            key
                        )
                    })
                })
                .collect()
        })
        .collect()
}

fn assert_no_self_loops_no_multi_edges(
    graph: &rust_igraph::Graph,
    path: &std::path::Path,
    source: &str,
    origin: &str,
) {
    use std::collections::HashSet;
    let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
    let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
    for eid in 0..n_edges {
        let (a, b) = graph.edge(eid).expect("edge id within bounds");
        assert_ne!(
            a,
            b,
            "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
            path.display(),
            source,
            origin,
        );
        let pair = if a <= b { (a, b) } else { (b, a) };
        assert!(
            canonical.insert(pair),
            "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
            path.display(),
            source,
            origin,
        );
    }
}

#[test]
fn hsbm_game_three_source_conformance() {
    // Hierarchical Stochastic Block Model (uniform-per-macro variant).
    // RNG state is not portable across implementations, so each fixture
    // pins parameters and checks the structural invariants:
    //   * vcount = n (exact);
    //   * directed = false (HSBM is always undirected);
    //   * ecount lies in [ecount_min, ecount_max] (band — exact when the
    //     fixture uses p∈{0, 1} and pins the C entries);
    //   * no self-loops, no parallel edges (HSBM produces simple graphs).
    use rust_igraph::hsbm_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("hsbm_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "hsbm_game");

            let n = er_param_u32(&case, "n", &path);
            let m = er_param_u32(&case, "m", &path);
            let rho = er_param_f64_vec(&case, "rho", &path);
            let c_matrix = er_param_f64_matrix(&case, "c", &path);
            let p = er_param_f64(&case, "p", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = hsbm_game(n, m, &rho, &c_matrix, p, seed)
                .expect("hsbm_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_simple {
                assert_no_self_loops_no_multi_edges(&graph, &path, &case.source, &case.origin);
            }

            assert_eq!(case.source, src);
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
            "no hsbm_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn hsbm_list_game_three_source_conformance() {
    // Hierarchical SBM with per-macro `m_list`, `rho_list`, `c_list`.
    // Same invariants as `hsbm_game_three_source_conformance`, with the
    // added constraint that vcount = sum(m_list).
    use rust_igraph::hsbm_list_game;

    fn parse_m_list(case: &Conformance, path: &std::path::Path) -> Vec<u32> {
        case.params
            .get("m_list")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "HSBM-list fixture {}: param `m_list` missing or not array",
                    path.display()
                )
            })
            .iter()
            .map(|cell| {
                let v = cell.as_u64().unwrap_or_else(|| {
                    panic!(
                        "HSBM-list fixture {}: m_list cell is not u64",
                        path.display()
                    )
                });
                u32::try_from(v).unwrap_or_else(|_| {
                    panic!(
                        "HSBM-list fixture {}: m_list cell {} does not fit in u32",
                        path.display(),
                        v
                    )
                })
            })
            .collect()
    }

    fn parse_rho_list(case: &Conformance, path: &std::path::Path) -> Vec<Vec<f64>> {
        case.params
            .get("rho_list")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "HSBM-list fixture {}: param `rho_list` missing or not array",
                    path.display()
                )
            })
            .iter()
            .map(|row| {
                row.as_array()
                    .unwrap_or_else(|| {
                        panic!(
                            "HSBM-list fixture {}: rho_list row is not array",
                            path.display()
                        )
                    })
                    .iter()
                    .map(|cell| {
                        cell.as_f64().unwrap_or_else(|| {
                            panic!(
                                "HSBM-list fixture {}: rho_list cell is not f64",
                                path.display()
                            )
                        })
                    })
                    .collect()
            })
            .collect()
    }

    fn parse_c_list(case: &Conformance, path: &std::path::Path) -> Vec<Vec<Vec<f64>>> {
        case.params
            .get("c_list")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "HSBM-list fixture {}: param `c_list` missing or not array",
                    path.display()
                )
            })
            .iter()
            .map(|matrix| {
                matrix
                    .as_array()
                    .unwrap_or_else(|| {
                        panic!(
                            "HSBM-list fixture {}: c_list matrix is not array",
                            path.display()
                        )
                    })
                    .iter()
                    .map(|row| {
                        row.as_array()
                            .unwrap_or_else(|| {
                                panic!(
                                    "HSBM-list fixture {}: c_list row is not array",
                                    path.display()
                                )
                            })
                            .iter()
                            .map(|cell| {
                                cell.as_f64().unwrap_or_else(|| {
                                    panic!(
                                        "HSBM-list fixture {}: c_list cell is not f64",
                                        path.display()
                                    )
                                })
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("hsbm_list_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "hsbm_list_game");

            let n = er_param_u32(&case, "n", &path);
            let m_list = parse_m_list(&case, &path);
            let rho_list = parse_rho_list(&case, &path);
            let c_list = parse_c_list(&case, &path);
            let p = er_param_f64(&case, "p", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = hsbm_list_game(n, &m_list, &rho_list, &c_list, p, seed)
                .expect("hsbm_list_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Extra invariant for the list variant: vcount must equal
            // sum(m_list). Catches manifest-typo bugs early.
            let m_sum: u64 = m_list.iter().copied().map(u64::from).sum();
            assert_eq!(
                u64::from(want_vertices),
                m_sum,
                "manifest mismatch in {}: vcount={} but sum(m_list)={}",
                path.display(),
                want_vertices,
                m_sum,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            if want_is_simple {
                assert_no_self_loops_no_multi_edges(&graph, &path, &case.source, &case.origin);
            }

            assert_eq!(case.source, src);
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
            "no hsbm_list_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn chung_lu_game_three_source_conformance() {
    // Chung–Lu expected-degree sampler (Miller–Hagberg). RNG state is not
    // portable across implementations, so each fixture pins:
    //   * vcount = len(out_weights) (exact);
    //   * directed = (in_weights is not None) (exact);
    //   * ecount within [ecount_min, ecount_max] (band — exact when the
    //     weights are all-zero or the variant degenerates);
    //   * is_simple when set true → no self-loops + no parallel edges;
    //   * no_multi_edges (when set true, allows loops but never parallel).
    use rust_igraph::{ChungLuVariant, chung_lu_game};
    use std::collections::HashSet;

    fn parse_variant(case: &Conformance, path: &std::path::Path) -> ChungLuVariant {
        let s = case
            .params
            .get("variant")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "Chung-Lu fixture {}: param `variant` missing or not string",
                    path.display()
                )
            });
        match s.to_ascii_lowercase().as_str() {
            "original" => ChungLuVariant::Original,
            "maxent" => ChungLuVariant::Maxent,
            "nr" => ChungLuVariant::Nr,
            other => panic!(
                "Chung-Lu fixture {}: unknown variant `{}` (want original|maxent|nr)",
                path.display(),
                other,
            ),
        }
    }

    fn parse_optional_f64_vec(
        case: &Conformance,
        key: &str,
        path: &std::path::Path,
    ) -> Option<Vec<f64>> {
        let v = case.params.get(key).unwrap_or_else(|| {
            panic!(
                "Chung-Lu fixture {}: param `{}` missing (must be null or array)",
                path.display(),
                key
            )
        });
        if v.is_null() {
            return None;
        }
        let arr = v.as_array().unwrap_or_else(|| {
            panic!(
                "Chung-Lu fixture {}: param `{}` must be null or array of numbers",
                path.display(),
                key
            )
        });
        Some(
            arr.iter()
                .map(|cell| {
                    cell.as_f64().unwrap_or_else(|| {
                        panic!(
                            "Chung-Lu fixture {}: param `{}` cell is not f64",
                            path.display(),
                            key
                        )
                    })
                })
                .collect(),
        )
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("chung_lu_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "chung_lu_game");

            let out_weights = er_param_f64_vec(&case, "out_weights", &path);
            let in_weights = parse_optional_f64_vec(&case, "in_weights", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let variant = parse_variant(&case, &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = chung_lu_game(&out_weights, in_weights.as_deref(), loops, variant, seed)
                .expect("chung_lu_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // vcount must equal len(out_weights); the directed flag must
            // equal in_weights.is_some(). Both are deterministic regardless
            // of seed — catches manifest-typo bugs early.
            assert_eq!(
                want_vertices as usize,
                out_weights.len(),
                "manifest mismatch in {}: vcount={} but len(out_weights)={}",
                path.display(),
                want_vertices,
                out_weights.len(),
            );
            assert_eq!(
                want_directed,
                in_weights.is_some(),
                "manifest mismatch in {}: directed={} but in_weights.is_some()={}",
                path.display(),
                want_directed,
                in_weights.is_some(),
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let want_is_simple = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let want_no_multi = case
                .expected
                .get("no_multi_edges")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            if want_is_simple || want_no_multi {
                // is_simple → no self-loops + no parallel edges.
                // no_multi_edges → loops allowed, parallel edges forbidden.
                // For directed graphs, (a, b) and (b, a) are distinct
                // edges and must NOT be canonicalized together.
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph.edge(eid).expect("edge id within bounds");
                    if want_is_simple {
                        assert_ne!(
                            a,
                            b,
                            "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                    let pair = if want_directed || a <= b {
                        (a, b)
                    } else {
                        (b, a)
                    };
                    assert!(
                        canonical.insert(pair),
                        "parallel edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            if !loops {
                // Sanity: with loops=false, the sampler must never emit a
                // self-loop. This holds even when is_simple is not set.
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                for eid in 0..n_edges {
                    let (a, b) = graph.edge(eid).expect("edge id within bounds");
                    assert_ne!(
                        a,
                        b,
                        "self-loop with loops=false in {} (edge {eid})\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no chung_lu_game fixtures from source {src}"
        );
    }
}

fn parse_optional_f64_vec_named(
    case: &Conformance,
    key: &str,
    label: &str,
    path: &std::path::Path,
) -> Option<Vec<f64>> {
    let v = case.params.get(key).unwrap_or_else(|| {
        panic!(
            "{label} fixture {}: param `{}` missing (must be null or array)",
            path.display(),
            key
        )
    });
    if v.is_null() {
        return None;
    }
    let arr = v.as_array().unwrap_or_else(|| {
        panic!(
            "{label} fixture {}: param `{}` must be null or array of numbers",
            path.display(),
            key
        )
    });
    Some(
        arr.iter()
            .map(|cell| {
                cell.as_f64().unwrap_or_else(|| {
                    panic!(
                        "{label} fixture {}: param `{}` cell is not f64",
                        path.display(),
                        key
                    )
                })
            })
            .collect(),
    )
}

fn assert_static_invariants(
    graph: &rust_igraph::Graph,
    case: &Conformance,
    path: &std::path::Path,
    want_directed: bool,
    loops: bool,
) {
    use std::collections::HashSet;

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_ecount_min = er_expected_u64(case, "ecount_min", path);
    let want_ecount_max = er_expected_u64(case, "ecount_max", path);

    assert_eq!(
        graph.vcount(),
        want_vertices,
        "vcount mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
    assert_eq!(
        graph.is_directed(),
        want_directed,
        "directed mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );

    let ecount = graph.ecount() as u64;
    assert!(
        ecount >= want_ecount_min && ecount <= want_ecount_max,
        "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
        ecount,
        want_ecount_min,
        want_ecount_max,
        path.display(),
        case.source,
        case.origin,
    );

    let want_is_simple = case
        .expected
        .get("is_simple")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let want_no_multi = case
        .expected
        .get("no_multi_edges")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if want_is_simple || want_no_multi {
        let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
        let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
        for eid in 0..n_edges {
            let (a, b) = graph.edge(eid).expect("edge id within bounds");
            if want_is_simple {
                assert_ne!(
                    a,
                    b,
                    "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }
            let pair = if want_directed || a <= b {
                (a, b)
            } else {
                (b, a)
            };
            assert!(
                canonical.insert(pair),
                "parallel edge {pair:?} in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
        }
    }

    if !loops {
        let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
        for eid in 0..n_edges {
            let (a, b) = graph.edge(eid).expect("edge id within bounds");
            assert_ne!(
                a,
                b,
                "self-loop with loops=false in {} (edge {eid})\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn static_fitness_game_three_source_conformance() {
    // Goh-Kahng-Kim cumulative-fitness sampler. RNG state is not portable
    // across implementations, so each fixture pins:
    //   * vcount = len(fitness_out) (exact);
    //   * directed = (fitness_in is not None) (exact);
    //   * ecount in [ecount_min, ecount_max] — pinned tight to m exactly
    //     for non-empty cases since the sampler always reaches the
    //     requested edge count when capacity > m;
    //   * is_simple ⇔ no self-loops and no parallel edges;
    //   * no_multi_edges ⇔ parallel edges forbidden but loops permitted.
    use rust_igraph::static_fitness_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("static_fitness_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "static_fitness_game");

            let no_of_edges = er_param_u32(&case, "no_of_edges", &path);
            let fitness_out = er_param_f64_vec(&case, "fitness_out", &path);
            let fitness_in =
                parse_optional_f64_vec_named(&case, "fitness_in", "static_fitness", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let multiple = er_param_bool(&case, "multiple", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = static_fitness_game(
                no_of_edges,
                &fitness_out,
                fitness_in.as_deref(),
                loops,
                multiple,
                seed,
            )
            .expect("static_fitness_game should succeed on conformance fixtures");

            let want_directed = er_expected_bool(&case, "directed", &path);

            // vcount ≡ len(fitness_out); directed ≡ fitness_in.is_some().
            // Catches manifest-typo bugs early.
            assert_eq!(
                want_directed,
                fitness_in.is_some(),
                "manifest mismatch in {}: directed={} but fitness_in.is_some()={}",
                path.display(),
                want_directed,
                fitness_in.is_some(),
            );
            let want_vertices = er_expected_u32(&case, "vcount", &path);
            assert_eq!(
                want_vertices as usize,
                fitness_out.len(),
                "manifest mismatch in {}: vcount={} but len(fitness_out)={}",
                path.display(),
                want_vertices,
                fitness_out.len(),
            );

            assert_static_invariants(&graph, &case, &path, want_directed, loops);

            assert_eq!(case.source, src);
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
            "no static_fitness_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn static_power_law_game_three_source_conformance() {
    // Goh et al. (2001) power-law wrapper around static_fitness_game with
    // the Cho et al. (2009) finite-size correction. RNG state is not
    // portable; each fixture pins vcount, directedness, ecount band, and
    // is_simple / no_multi_edges where applicable.
    use rust_igraph::static_power_law_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("static_power_law_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "static_power_law_game");

            let no_of_nodes = er_param_u32(&case, "no_of_nodes", &path);
            let no_of_edges = er_param_u32(&case, "no_of_edges", &path);
            let exponent_out = er_param_f64(&case, "exponent_out", &path);
            let exponent_in: Option<f64> = case
                .params
                .get("exponent_in")
                .and_then(|v| if v.is_null() { None } else { v.as_f64() });
            let loops = er_param_bool(&case, "loops", &path);
            let multiple = er_param_bool(&case, "multiple", &path);
            let finite_size_correction = er_param_bool(&case, "finite_size_correction", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = static_power_law_game(
                no_of_nodes,
                no_of_edges,
                exponent_out,
                exponent_in,
                loops,
                multiple,
                finite_size_correction,
                seed,
            )
            .expect("static_power_law_game should succeed on conformance fixtures");

            let want_directed = er_expected_bool(&case, "directed", &path);
            assert_eq!(
                want_directed,
                exponent_in.is_some(),
                "manifest mismatch in {}: directed={} but exponent_in.is_some()={}",
                path.display(),
                want_directed,
                exponent_in.is_some(),
            );

            assert_static_invariants(&graph, &case, &path, want_directed, loops);

            assert_eq!(case.source, src);
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
            "no static_power_law_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn preference_game_three_source_conformance() {
    // Symmetric preference-game sampler. RNG state is not portable across
    // implementations, so each fixture pins parameters and bands the
    // structural invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a generous band [ecount_min, ecount_max];
    //   * when expected.is_simple = true, no parallel edges and no
    //     self-loops (unless `loops=true`, in which case self-loops
    //     are tolerated but still tracked uniquely);
    //   * every assigned type stays in [0, max_type];
    //   * when expected.diagonal_only_pref = true, every edge connects
    //     two vertices of the same type (the manifest's pref matrix
    //     is block-diagonal).
    use rust_igraph::preference_game;
    use std::collections::HashSet;

    fn parse_type_dist(case: &Conformance) -> Option<Vec<f64>> {
        let raw = case.params.get("type_dist")?;
        if raw.is_null() {
            return None;
        }
        raw.as_array().map(|arr| {
            arr.iter()
                .map(|cell| {
                    cell.as_f64()
                        .expect("preference fixture: type_dist cell not f64")
                })
                .collect()
        })
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("preference_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "preference_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let types = er_param_u32(&case, "types", &path);
            let type_dist_owned = parse_type_dist(&case);
            let fixed_sizes = er_param_bool(&case, "fixed_sizes", &path);
            let pref_matrix = er_param_f64_matrix(&case, "pref_matrix", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let (graph, node_types) = preference_game(
                nodes,
                types,
                type_dist_owned.as_deref(),
                fixed_sizes,
                &pref_matrix,
                directed,
                loops,
                seed,
            )
            .expect("preference_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);
            let want_max_type = er_expected_u32(&case, "max_type", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(
                node_types.len(),
                nodes as usize,
                "node_types length mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            for (v, &t) in node_types.iter().enumerate() {
                assert!(
                    t <= want_max_type,
                    "vertex {v} has type {t} > max_type {want_max_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");

            if want_is_simple {
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for preference fixture");
                    if !loops {
                        assert_ne!(
                            a,
                            b,
                            "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                    let pair = if directed || a <= b { (a, b) } else { (b, a) };
                    assert!(
                        canonical.insert(pair),
                        "multi-edge {pair:?} in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            if let Some(true) = case
                .expected
                .get("diagonal_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph
                        .edge(eid)
                        .expect("edge id within bounds for preference fixture");
                    let tu = node_types[u as usize];
                    let tv = node_types[v as usize];
                    assert_eq!(
                        tu,
                        tv,
                        "edge {u}-{v} crosses types ({tu} vs {tv}) in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no preference_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn asymmetric_preference_game_three_source_conformance() {
    // Asymmetric preference-game sampler. Always directed. RNG state is
    // not portable across implementations, so each fixture pins
    // parameters and bands the structural invariants:
    //   * vcount = nodes (exact);
    //   * directed = true (model is always directed);
    //   * ecount lies in a generous band [ecount_min, ecount_max];
    //   * when expected.is_simple = true, no parallel edges and no
    //     self-loops (unless `loops=true`);
    //   * every out-type stays in [0, max_out_type];
    //   * every in-type stays in [0, max_in_type].
    use rust_igraph::asymmetric_preference_game;
    use std::collections::HashSet;

    fn parse_type_dist_matrix(case: &Conformance) -> Option<Vec<Vec<f64>>> {
        let raw = case.params.get("type_dist_matrix")?;
        if raw.is_null() {
            return None;
        }
        raw.as_array().map(|outer| {
            outer
                .iter()
                .map(|row| {
                    row.as_array()
                        .expect("asymmetric_preference fixture: type_dist_matrix row not array")
                        .iter()
                        .map(|cell| {
                            cell.as_f64().expect(
                                "asymmetric_preference fixture: type_dist_matrix cell not f64",
                            )
                        })
                        .collect()
                })
                .collect()
        })
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("asymmetric_preference_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "asymmetric_preference_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let no_out_types = er_param_u32(&case, "no_out_types", &path);
            let no_in_types = er_param_u32(&case, "no_in_types", &path);
            let type_dist_matrix_owned = parse_type_dist_matrix(&case);
            let pref_matrix = er_param_f64_matrix(&case, "pref_matrix", &path);
            let loops = er_param_bool(&case, "loops", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let (graph, node_out, node_in) = asymmetric_preference_game(
                nodes,
                no_out_types,
                no_in_types,
                type_dist_matrix_owned.as_deref(),
                &pref_matrix,
                loops,
                seed,
            )
            .expect("asymmetric_preference_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_is_simple = er_expected_bool(&case, "is_simple", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);
            let want_max_out_type = er_expected_u32(&case, "max_out_type", &path);
            let want_max_in_type = er_expected_u32(&case, "max_in_type", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(
                node_out.len(),
                nodes as usize,
                "node_out length mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                node_in.len(),
                nodes as usize,
                "node_in length mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            for (v, &t) in node_out.iter().enumerate() {
                assert!(
                    t <= want_max_out_type,
                    "vertex {v} has out_type {t} > max_out_type {want_max_out_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }
            for (v, &t) in node_in.iter().enumerate() {
                assert!(
                    t <= want_max_in_type,
                    "vertex {v} has in_type {t} > max_in_type {want_max_in_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            if want_is_simple {
                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
                for eid in 0..n_edges {
                    let (a, b) = graph
                        .edge(eid)
                        .expect("edge id within bounds for asymmetric preference fixture");
                    if !loops {
                        assert_ne!(
                            a,
                            b,
                            "self-loop in {} (edge {eid})\n  source: {}\n  origin: {}",
                            path.display(),
                            case.source,
                            case.origin,
                        );
                    }
                    assert!(
                        canonical.insert((a, b)),
                        "multi-edge ({a},{b}) in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no asymmetric_preference_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn establishment_game_three_source_conformance() {
    // Establishment / sample-traits sampler. RNG state is not portable
    // across implementations, so each fixture pins parameters and bands
    // the structural invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a generous band [ecount_min, ecount_max];
    //   * graph is simple by construction (Floyd distinct picks, growth
    //     direction always backward in time) — verified always;
    //   * every assigned type stays in [0, max_type];
    //   * when expected.diagonal_only_pref = true, every edge connects
    //     two same-type vertices (the manifest pref is block-diagonal);
    //   * when expected.cross_only_pref = true, every edge connects two
    //     different-type vertices (off-diagonal pref only).
    use rust_igraph::establishment_game;
    use std::collections::HashSet;

    fn parse_type_dist(case: &Conformance) -> Option<Vec<f64>> {
        let raw = case.params.get("type_dist")?;
        if raw.is_null() {
            return None;
        }
        raw.as_array().map(|arr| {
            arr.iter()
                .map(|cell| {
                    cell.as_f64()
                        .expect("establishment fixture: type_dist cell not f64")
                })
                .collect()
        })
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("establishment_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "establishment_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let types = er_param_u32(&case, "types", &path);
            let k = er_param_u32(&case, "k", &path);
            let type_dist_owned = parse_type_dist(&case);
            let pref_matrix = er_param_f64_matrix(&case, "pref_matrix", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let (graph, node_types) = establishment_game(
                nodes,
                types,
                k,
                type_dist_owned.as_deref(),
                &pref_matrix,
                directed,
                seed,
            )
            .expect("establishment_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);
            let want_max_type = er_expected_u32(&case, "max_type", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(
                node_types.len(),
                nodes as usize,
                "node_types length mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            for (v, &t) in node_types.iter().enumerate() {
                assert!(
                    t <= want_max_type,
                    "vertex {v} has type {t} > max_type {want_max_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            // Establishment is always simple by construction: Floyd picks
            // are distinct and edges always go from a later vertex to an
            // earlier one, so no parallels and no self-loops.
            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(n_edges as usize);
            for eid in 0..n_edges {
                let (a, b) = graph
                    .edge(eid)
                    .expect("edge id within bounds for establishment fixture");
                assert_ne!(a, b, "self-loop in {} (edge {eid})", path.display());
                let pair = if directed || a <= b { (a, b) } else { (b, a) };
                assert!(
                    canonical.insert(pair),
                    "multi-edge {pair:?} in {}",
                    path.display()
                );
            }

            if let Some(true) = case
                .expected
                .get("diagonal_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    let tu = node_types[u as usize];
                    let tv = node_types[v as usize];
                    assert_eq!(
                        tu,
                        tv,
                        "edge {u}-{v} crosses types ({tu} vs {tv}) in {}",
                        path.display()
                    );
                }
            }
            if let Some(true) = case
                .expected
                .get("cross_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    let tu = node_types[u as usize];
                    let tv = node_types[v as usize];
                    assert_ne!(
                        tu,
                        tv,
                        "edge {u}-{v} same-type ({tu}) violates cross-only in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no establishment_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn callaway_traits_game_three_source_conformance() {
    // Callaway-traits / sample-traits-callaway sampler. RNG state is
    // not portable across implementations, so each fixture pins
    // parameters and bands the structural invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a generous band [ecount_min, ecount_max];
    //   * NOTE: unlike establishment_game, callaway picks BOTH
    //     endpoints uniformly from [0, i] inclusive, so self-loops and
    //     multi-edges ARE allowed by construction — we do not assert
    //     simplicity here.
    //   * every assigned type stays in [0, max_type];
    //   * when expected.diagonal_only_pref = true, every edge connects
    //     two same-type vertices (the manifest pref is block-diagonal);
    //   * when expected.cross_only_pref = true, every edge connects two
    //     different-type vertices (off-diagonal pref only).
    use rust_igraph::callaway_traits_game;

    fn parse_type_dist(case: &Conformance) -> Option<Vec<f64>> {
        let raw = case.params.get("type_dist")?;
        if raw.is_null() {
            return None;
        }
        raw.as_array().map(|arr| {
            arr.iter()
                .map(|cell| {
                    cell.as_f64()
                        .expect("callaway fixture: type_dist cell not f64")
                })
                .collect()
        })
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("callaway_traits_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "callaway_traits_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let types = er_param_u32(&case, "types", &path);
            let edges_per_step = er_param_u32(&case, "edges_per_step", &path);
            let type_dist_owned = parse_type_dist(&case);
            let pref_matrix = er_param_f64_matrix(&case, "pref_matrix", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let (graph, node_types) = callaway_traits_game(
                nodes,
                types,
                edges_per_step,
                type_dist_owned.as_deref(),
                &pref_matrix,
                directed,
                seed,
            )
            .expect("callaway_traits_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);
            let want_max_type = er_expected_u32(&case, "max_type", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(
                node_types.len(),
                nodes as usize,
                "node_types length mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            for (v, &t) in node_types.iter().enumerate() {
                assert!(
                    t <= want_max_type,
                    "vertex {v} has type {t} > max_type {want_max_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("diagonal_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    let tu = node_types[u as usize];
                    let tv = node_types[v as usize];
                    assert_eq!(
                        tu,
                        tv,
                        "edge {u}-{v} crosses types ({tu} vs {tv}) in {}",
                        path.display()
                    );
                }
            }
            if let Some(true) = case
                .expected
                .get("cross_only_pref")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    let tu = node_types[u as usize];
                    let tv = node_types[v as usize];
                    assert_ne!(
                        tu,
                        tv,
                        "edge {u}-{v} same-type ({tu}) violates cross-only in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no callaway_traits_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + invariant checks
fn cited_type_game_three_source_conformance() {
    // Cited-type citation game (ALGO-GN-017). RNG state is not
    // portable; each fixture pins parameters and asserts structural
    // invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a band [ecount_min, ecount_max] — for
    //     positive-pref runs both bounds equal (n-1)*eps;
    //   * NOTE: cited_type allows MULTI-edges when eps≥2 (multiple
    //     citations at one step may select the same target). Simplicity
    //     is therefore NOT asserted here.
    //   * when expected.no_self_loops = true: assert s != d for every
    //     edge (true whenever the cumulative pref sum is positive);
    //   * when expected.all_self_loops = true: assert s == d for every
    //     edge (true under the sum=0 fallback path).
    use rust_igraph::cited_type_game;

    fn parse_u32_array(case: &Conformance, key: &str, path: &std::path::Path) -> Vec<u32> {
        case.params
            .get(key)
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "cited_type fixture {}: param `{}` missing or not array",
                    path.display(),
                    key
                )
            })
            .iter()
            .map(|cell| {
                let n = cell.as_u64().unwrap_or_else(|| {
                    panic!(
                        "cited_type fixture {}: param `{}` cell is not u64",
                        path.display(),
                        key
                    )
                });
                u32::try_from(n).unwrap_or_else(|_| {
                    panic!(
                        "cited_type fixture {}: param `{}` cell {} does not fit u32",
                        path.display(),
                        key,
                        n
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("cited_type_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "cited_type_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let types = parse_u32_array(&case, "types", &path);
            let pref = er_param_f64_vec(&case, "pref", &path);
            let edges_per_step = er_param_u32(&case, "edges_per_step", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = cited_type_game(nodes, &types, &pref, edges_per_step, directed, seed)
                .expect("cited_type_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);
            let want_max_type = er_expected_u32(&case, "max_type", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            for (v, &t) in types.iter().enumerate() {
                assert!(
                    t <= want_max_type,
                    "vertex {v} has type {t} > max_type {want_max_type} in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }
            if let Some(true) = case
                .expected
                .get("all_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_eq!(
                        u,
                        v,
                        "edge {eid}: ({u}-{v}) is not a self-loop but all_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no cited_type_game fixtures from source {src}"
        );
    }
}

#[test]
fn lastcit_game_three_source_conformance() {
    // Last-citation citation game (ALGO-GN-018). RNG state is not
    // portable; each fixture pins parameters and asserts structural
    // invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a band [ecount_min, ecount_max] — for typical
    //     runs both bounds equal (nodes-1)*edges_per_node;
    //   * lastcit NEVER self-loops (psumtree only ranges over [0, i)
    //     before vertex i is inserted), so no_self_loops is always true
    //     and we assert it whenever the fixture flags it;
    //   * lastcit MAY produce multi-edges when edges_per_node ≥ 2;
    //     simplicity is therefore NOT asserted here.
    use rust_igraph::lastcit_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("lastcit_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "lastcit_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let edges_per_node = er_param_u32(&case, "edges_per_node", &path);
            let agebins = er_param_u32(&case, "agebins", &path);
            let preference = er_param_f64_vec(&case, "preference", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = lastcit_game(nodes, edges_per_node, agebins, &preference, directed, seed)
                .expect("lastcit_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no lastcit_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 8 param helpers + 4 expected assertions
fn recent_degree_game_three_source_conformance() {
    // Recent-degree-window preferential attachment (ALGO-GN-019). RNG
    // state is not portable; each fixture pins SplitMix64 output and
    // asserts structural invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a band [ecount_min, ecount_max] — for constant-m
    //     runs both bounds equal (nodes-1)*m;
    //   * recent_degree NEVER self-loops (psumtree ranges over [0, i)
    //     before vertex i is inserted), so no_self_loops is always true
    //     and we assert it whenever the fixture flags it;
    //   * recent_degree MAY produce multi-edges when m ≥ 2; simplicity
    //     is therefore NOT asserted here.
    use rust_igraph::recent_degree_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("recent_degree_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "recent_degree_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let power = er_param_f64(&case, "power", &path);
            let time_window = er_param_u32(&case, "time_window", &path);
            let m = er_param_u32(&case, "m", &path);
            let outpref = er_param_bool(&case, "outpref", &path);
            let zero_appeal = er_param_f64(&case, "zero_appeal", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = recent_degree_game(
                nodes,
                power,
                time_window,
                m,
                None,
                outpref,
                zero_appeal,
                directed,
                seed,
            )
            .expect("recent_degree_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no recent_degree_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + variant selector + 5 expected assertions
fn barabasi_psumtree_three_source_conformance() {
    // BA-PSUMTREE / BA-PSUMTREE-MULTIPLE preferential attachment
    // (ALGO-GN-020). RNG state is not portable; each fixture pins
    // SplitMix64 output and asserts structural invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount lies in a band [ecount_min, ecount_max] — for simple
    //     variant both bounds equal (nodes-1)*m; for multiple variant
    //     both bounds equal (nodes-1)*m - m*(m-1)/2 when n > m;
    //   * BA-PSUMTREE NEVER self-loops (the bounded prefix-sum search
    //     ranges over [0, i) before vertex i is inserted), so the
    //     no_self_loops flag is asserted whenever the fixture sets it.
    //   * The simple variant is also free of within-step multi-edges,
    //     but cross-step multi-edges can occur in either variant when
    //     two new vertices independently cite the same hub. We do NOT
    //     assert simplicity here.
    //
    // The `params.variant` field selects between the simple and
    // multiple entry points.
    use rust_igraph::{barabasi_game_psumtree, barabasi_game_psumtree_multiple};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("barabasi_game_psumtree");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "barabasi_game_psumtree");

            let nodes = er_param_u32(&case, "nodes", &path);
            let power = er_param_f64(&case, "power", &path);
            let m = er_param_u32(&case, "m", &path);
            let outpref = er_param_bool(&case, "outpref", &path);
            let a = er_param_f64(&case, "a", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);
            let variant = case
                .params
                .get("variant")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("missing `variant` param in {}", path.display()));

            let graph = match variant {
                "psumtree" => {
                    barabasi_game_psumtree(nodes, power, m, None, outpref, a, directed, seed)
                }
                "psumtree_multiple" => barabasi_game_psumtree_multiple(
                    nodes, power, m, None, outpref, a, directed, seed,
                ),
                other => panic!(
                    "unknown variant `{other}` in {} — expected `psumtree` or `psumtree_multiple`",
                    path.display()
                ),
            }
            .expect("barabasi_game_psumtree* should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no barabasi_game_psumtree fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 13-param call + 5 expected assertions
fn barabasi_aging_three_source_conformance() {
    // BA-with-vertex-aging (ALGO-GN-021). RNG state is not portable;
    // each fixture pins SplitMix64 output and asserts structural
    // invariants:
    //   * vcount = nodes (exact);
    //   * directed flag exact;
    //   * ecount = (nodes - 1) * m exactly when `outseq` is None — no
    //     saturation branch fires because the C kernel writes one edge
    //     per attempted draw regardless of within-step collisions;
    //   * NEVER self-loops by construction (the BIT search is bounded
    //     to [0, i) before vertex i is inserted).
    use rust_igraph::barabasi_aging_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("barabasi_aging_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "barabasi_aging_game");

            let nodes = er_param_u32(&case, "nodes", &path);
            let m = er_param_u32(&case, "m", &path);
            let outpref = er_param_bool(&case, "outpref", &path);
            let pa_exp = er_param_f64(&case, "pa_exp", &path);
            let aging_exp = er_param_f64(&case, "aging_exp", &path);
            let aging_bins = er_param_u32(&case, "aging_bins", &path);
            let zero_deg_appeal = er_param_f64(&case, "zero_deg_appeal", &path);
            let zero_age_appeal = er_param_f64(&case, "zero_age_appeal", &path);
            let deg_coef = er_param_f64(&case, "deg_coef", &path);
            let age_coef = er_param_f64(&case, "age_coef", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = barabasi_aging_game(
                nodes,
                m,
                None,
                outpref,
                pa_exp,
                aging_exp,
                aging_bins,
                zero_deg_appeal,
                zero_age_appeal,
                deg_coef,
                age_coef,
                directed,
                seed,
            )
            .expect("barabasi_aging_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no barabasi_aging_game fixtures from source {src}"
        );
    }
}

#[test]
fn dot_product_three_source_conformance() {
    // Random dot-product graph (ALGO-GN-022). RNG state is not portable
    // to upstream's glibc-style RNG, so each fixture pins latent
    // position vectors `vecs[i]` that collapse every pair to a
    // deterministic regime (`prob == 1` always-edge, `prob == 0`
    // never-edge, `prob > 1` always-edge short-circuit, `prob < 0`
    // always-skip), giving an *exact* or tightly-banded `ecount` under
    // any RNG. Per-fixture invariants asserted:
    //   * vcount = len(vecs);
    //   * directed flag exact;
    //   * ecount falls in [ecount_min, ecount_max];
    //   * NEVER self-loops by construction.
    use rust_igraph::dot_product_game;

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("dot_product_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "dot_product_game");

            let vecs = er_param_f64_matrix(&case, "vecs", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);

            let graph = dot_product_game(&vecs, directed, seed)
                .expect("dot_product_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no dot_product_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn correlated_game_three_source_conformance() {
    // Correlated Erdős–Rényi (ALGO-GN-023). RNG state is not portable to
    // upstream's MT/glibc RNGs, so fixtures pin corr = 1.0 cases (which
    // produce an exact copy of `old_graph` with p_del = p_add = 0, no
    // RNG draws needed) and read the old graph from `case.graph`.
    // Per-fixture invariants asserted:
    //   * vcount = old.vcount;
    //   * directed flag = old.is_directed (correlated_game preserves it);
    //   * ecount inside [ecount_min, ecount_max] band;
    //   * no self-loops by construction;
    //   * is_simple = true (no parallels either).
    use rust_igraph::{correlated_game, is_simple};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("correlated_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "correlated_game");

            // The old graph is encoded in the top-level `graph` field.
            let old = build_graph(&case.graph);
            let corr = er_param_f64(&case, "corr", &path);
            let p = er_param_f64(&case, "p", &path);
            let seed = er_param_u64(&case, "seed", &path);
            let permutation_value = case
                .params
                .get("permutation")
                .unwrap_or(&serde_json::Value::Null);
            let permutation: Option<Vec<u32>> = match permutation_value {
                serde_json::Value::Null => None,
                serde_json::Value::Array(items) => Some(
                    items
                        .iter()
                        .map(|v| {
                            u32::try_from(v.as_u64().expect("permutation entry must be u64"))
                                .expect("permutation entry fits u32")
                        })
                        .collect(),
                ),
                other => panic!(
                    "correlated_game fixture {}: `permutation` must be null or array of u32 (got {other:?})",
                    path.display()
                ),
            };

            let graph = correlated_game(&old, corr, p, permutation.as_deref(), seed)
                .expect("correlated_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let ecount = graph.ecount() as u64;
            assert!(
                ecount >= want_ecount_min && ecount <= want_ecount_max,
                "ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                ecount,
                want_ecount_min,
                want_ecount_max,
                path.display(),
                case.source,
                case.origin,
            );

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            if let Some(true) = case
                .expected
                .get("no_self_loops")
                .and_then(serde_json::Value::as_bool)
            {
                for eid in 0..n_edges {
                    let (u, v) = graph.edge(eid).expect("edge id in bounds");
                    assert_ne!(
                        u,
                        v,
                        "edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                        path.display()
                    );
                }
            }

            if let Some(true) = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
            {
                assert!(
                    is_simple(&graph).expect("is_simple ok"),
                    "graph not simple but is_simple=true in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no correlated_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn correlated_pair_game_three_source_conformance() {
    // Correlated pair (ALGO-GN-023). Each fixture's `params` carries
    // (n, corr, p, directed, permutation, seed); `expected` carries
    // vcount + directed + 6σ Binomial bands on ecount (applied to
    // BOTH returned graphs, since marginals match ER(n, p)).
    use rust_igraph::{correlated_pair_game, is_simple};

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("correlated_pair_game");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "correlated_pair_game");

            let n = er_param_u32(&case, "n", &path);
            let corr = er_param_f64(&case, "corr", &path);
            let p = er_param_f64(&case, "p", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let seed = er_param_u64(&case, "seed", &path);
            let permutation_value = case
                .params
                .get("permutation")
                .unwrap_or(&serde_json::Value::Null);
            let permutation: Option<Vec<u32>> = match permutation_value {
                serde_json::Value::Null => None,
                serde_json::Value::Array(items) => Some(
                    items
                        .iter()
                        .map(|v| {
                            u32::try_from(v.as_u64().expect("permutation entry must be u64"))
                                .expect("permutation entry fits u32")
                        })
                        .collect(),
                ),
                other => panic!(
                    "correlated_pair_game fixture {}: `permutation` must be null or array of u32 (got {other:?})",
                    path.display()
                ),
            };

            let (g1, g2) = correlated_pair_game(n, corr, p, directed, permutation.as_deref(), seed)
                .expect("correlated_pair_game should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount_min = er_expected_u64(&case, "ecount_min", &path);
            let want_ecount_max = er_expected_u64(&case, "ecount_max", &path);

            for (label, graph) in [("g1", &g1), ("g2", &g2)] {
                assert_eq!(
                    graph.vcount(),
                    want_vertices,
                    "{label} vcount mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
                assert_eq!(
                    graph.is_directed(),
                    want_directed,
                    "{label} directed mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );

                let ecount = graph.ecount() as u64;
                assert!(
                    ecount >= want_ecount_min && ecount <= want_ecount_max,
                    "{label} ecount {} outside band [{}, {}] in {}\n  source: {}\n  origin: {}",
                    ecount,
                    want_ecount_min,
                    want_ecount_max,
                    path.display(),
                    case.source,
                    case.origin,
                );

                let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32");
                if let Some(true) = case
                    .expected
                    .get("no_self_loops")
                    .and_then(serde_json::Value::as_bool)
                {
                    for eid in 0..n_edges {
                        let (u, v) = graph.edge(eid).expect("edge id in bounds");
                        assert_ne!(
                            u,
                            v,
                            "{label} edge {eid}: self-loop ({u}-{v}) but no_self_loops=true in {}",
                            path.display()
                        );
                    }
                }

                if let Some(true) = case
                    .expected
                    .get("is_simple")
                    .and_then(serde_json::Value::as_bool)
                {
                    assert!(
                        is_simple(graph).expect("is_simple ok"),
                        "{label} not simple but is_simple=true in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            assert_eq!(case.source, src);
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
            "no correlated_pair_game fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn degree_sequence_game_configuration_three_source_conformance() {
    // Configuration-model degree-sequence generator (ALGO-GN-024). The
    // configuration variant is degree-preserving by construction, so
    // every fixture pins the exact (out_degrees, in_degrees) vector that
    // the resulting graph must realise — no RNG-state portability is
    // needed and no bands are used.
    use rust_igraph::degree_sequence_game_configuration;

    fn json_u32_vec(value: &serde_json::Value, path: &std::path::Path, field: &str) -> Vec<u32> {
        let array = value.as_array().unwrap_or_else(|| {
            panic!(
                "degree-sequence fixture {}: `{}` must be a JSON array",
                path.display(),
                field
            )
        });
        array
            .iter()
            .map(|item| {
                let raw = item.as_u64().unwrap_or_else(|| {
                    panic!(
                        "degree-sequence fixture {}: `{}` entry must be u64",
                        path.display(),
                        field
                    )
                });
                u32::try_from(raw).unwrap_or_else(|_| {
                    panic!(
                        "degree-sequence fixture {}: `{}` entry {} doesn't fit in u32",
                        path.display(),
                        field,
                        raw
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("degree_sequence_game_configuration");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse conformance fixture JSON");
            assert_eq!(case.algo, "degree_sequence_game_configuration");

            let out_value = case.params.get("out_degrees").unwrap_or_else(|| {
                panic!(
                    "degree-sequence fixture {}: param `out_degrees` missing",
                    path.display()
                )
            });
            let out_degrees = json_u32_vec(out_value, &path, "params.out_degrees");
            let in_value = case
                .params
                .get("in_degrees")
                .unwrap_or(&serde_json::Value::Null);
            let in_degrees: Option<Vec<u32>> = match in_value {
                serde_json::Value::Null => None,
                serde_json::Value::Array(_) => {
                    Some(json_u32_vec(in_value, &path, "params.in_degrees"))
                }
                other => panic!(
                    "degree-sequence fixture {}: `in_degrees` must be null or array (got {other:?})",
                    path.display()
                ),
            };
            let seed = er_param_u64(&case, "seed", &path);

            let graph =
                degree_sequence_game_configuration(&out_degrees, in_degrees.as_deref(), seed)
                    .expect(
                        "degree_sequence_game_configuration should succeed on conformance fixtures",
                    );

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_ecount = er_expected_u64(&case, "ecount", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_ecount,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Re-derive the observed degree sequence(s) and compare to
            // the pinned expected vectors. Configuration is
            // degree-preserving, so equality must be exact.
            let vcount = graph.vcount() as usize;
            let mut observed_out = vec![0u32; vcount];
            let mut observed_in = vec![0u32; vcount];
            let ecount = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            for eid in 0..ecount {
                let (src_vid, dst_vid) = graph.edge(eid).expect("edge id in bounds");
                if graph.is_directed() {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_in[dst_vid as usize] = observed_in[dst_vid as usize].saturating_add(1);
                } else {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_out[dst_vid as usize] =
                        observed_out[dst_vid as usize].saturating_add(1);
                }
            }
            let want_out = json_u32_vec(
                case.expected.get("out_degrees").unwrap_or_else(|| {
                    panic!(
                        "degree-sequence fixture {}: expected.out_degrees missing",
                        path.display()
                    )
                }),
                &path,
                "expected.out_degrees",
            );
            assert_eq!(
                observed_out,
                want_out,
                "out-degree sequence mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_in_value = case
                .expected
                .get("in_degrees")
                .unwrap_or(&serde_json::Value::Null);
            match want_in_value {
                serde_json::Value::Null => {
                    assert!(
                        !graph.is_directed(),
                        "fixture {} expects in_degrees=None but graph is directed",
                        path.display()
                    );
                }
                serde_json::Value::Array(_) => {
                    assert!(
                        graph.is_directed(),
                        "fixture {} expects in_degrees vector but graph is undirected",
                        path.display()
                    );
                    let want_in = json_u32_vec(want_in_value, &path, "expected.in_degrees");
                    assert_eq!(
                        observed_in,
                        want_in,
                        "in-degree sequence mismatch in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
                other => panic!(
                    "degree-sequence fixture {}: expected.in_degrees must be null or array (got {other:?})",
                    path.display()
                ),
            }

            assert_eq!(case.source, src);
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
            "no degree_sequence_game_configuration fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn degree_sequence_game_vl_three_source_conformance() {
    // Viger-Latapy degree-sequence generator (ALGO-GN-025). VL samples a
    // *connected, simple* undirected graph realising the given degree
    // sequence exactly. Fixtures pin: vcount, ecount=Σd/2, exact degree
    // match, simplicity, and weak connectivity (across non-isolated
    // vertices). RNG state is not shared with the upstream
    // implementations, so we assert structural invariants only.
    use rust_igraph::{
        SimpleMode, connected_components, degree_sequence_game_vl, is_simple_with_mode,
    };

    fn json_u32_vec(value: &serde_json::Value, path: &std::path::Path, field: &str) -> Vec<u32> {
        let array = value.as_array().unwrap_or_else(|| {
            panic!(
                "VL fixture {}: `{}` must be a JSON array",
                path.display(),
                field
            )
        });
        array
            .iter()
            .map(|item| {
                let raw = item.as_u64().unwrap_or_else(|| {
                    panic!(
                        "VL fixture {}: `{}` entry must be u64",
                        path.display(),
                        field
                    )
                });
                u32::try_from(raw).unwrap_or_else(|_| {
                    panic!(
                        "VL fixture {}: `{}` entry {} doesn't fit in u32",
                        path.display(),
                        field,
                        raw
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("degree_sequence_game_vl");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read VL fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read VL fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse VL conformance fixture JSON");
            assert_eq!(case.algo, "degree_sequence_game_vl");

            let degrees_value = case.params.get("degrees").unwrap_or_else(|| {
                panic!("VL fixture {}: param `degrees` missing", path.display())
            });
            let degrees = json_u32_vec(degrees_value, &path, "params.degrees");
            let seed = er_param_u64(&case, "seed", &path);

            let graph = degree_sequence_game_vl(&degrees, seed)
                .expect("degree_sequence_game_vl should succeed on conformance fixtures");

            let want_vcount = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);

            assert_eq!(
                graph.vcount(),
                want_vcount,
                "VL vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert!(
                !graph.is_directed(),
                "VL fixture {}: graph must be undirected",
                path.display()
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "VL directed mismatch in {}",
                path.display()
            );
            assert_eq!(
                u64::try_from(graph.ecount()).expect("ecount fits in u64"),
                want_edges,
                "VL ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Observed degree sequence must match the input *exactly*.
            let vcount = graph.vcount() as usize;
            let mut observed = vec![0u32; vcount];
            let ecount = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            for eid in 0..ecount {
                let (src_vid, dst_vid) = graph.edge(eid).expect("edge id in bounds");
                observed[src_vid as usize] = observed[src_vid as usize].saturating_add(1);
                observed[dst_vid as usize] = observed[dst_vid as usize].saturating_add(1);
            }
            let want_deg = json_u32_vec(
                case.expected.get("degrees").unwrap_or_else(|| {
                    panic!("VL fixture {}: expected.degrees missing", path.display())
                }),
                &path,
                "expected.degrees",
            );
            assert_eq!(
                observed,
                want_deg,
                "VL degree sequence mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Simplicity invariant (no self-loops, no multi-edges).
            let want_simple = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let observed_simple = is_simple_with_mode(&graph, SimpleMode::DirectedAsDirected)
                .expect("is_simple should succeed on VL output");
            assert_eq!(
                observed_simple,
                want_simple,
                "VL simplicity mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Weak connectivity over the active subgraph: VL guarantees
            // every vertex with positive degree is in a single component.
            // For inputs where all degrees are zero, the graph is
            // vacuously connected.
            let want_connected = case
                .expected
                .get("is_connected")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let active_present = degrees.iter().any(|&d| d > 0);
            let observed_connected = if active_present {
                let cc = connected_components(&graph).expect("connected_components should succeed");
                // count distinct component ids ignoring trivially-isolated singletons.
                let mut seen_cid = std::collections::BTreeSet::<u32>::new();
                for (v, &deg) in observed.iter().enumerate().take(vcount) {
                    if deg > 0 {
                        seen_cid.insert(cc.membership[v]);
                    }
                }
                seen_cid.len() <= 1
            } else {
                true
            };
            assert_eq!(
                observed_connected,
                want_connected,
                "VL connectivity mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no degree_sequence_game_vl fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn degree_sequence_game_fast_heur_simple_three_source_conformance() {
    // Fast-heuristic-simple degree-sequence generator (ALGO-GN-026). The
    // method samples a *simple* (no self-loops, no multi-edges) undirected
    // or directed graph realising the input degree sequence exactly. RNG
    // state is not portable to upstream igraph C / NumPy / R, so the
    // fixtures pin structural invariants only: vcount, ecount, exact
    // (out/in-)degree match, simplicity. Connectivity is NOT guaranteed
    // by this method.
    use rust_igraph::{SimpleMode, degree_sequence_game_fast_heur_simple, is_simple_with_mode};

    fn json_u32_vec(value: &serde_json::Value, path: &std::path::Path, field: &str) -> Vec<u32> {
        let array = value.as_array().unwrap_or_else(|| {
            panic!(
                "FAST_HEUR fixture {}: `{}` must be a JSON array",
                path.display(),
                field
            )
        });
        array
            .iter()
            .map(|item| {
                let raw = item.as_u64().unwrap_or_else(|| {
                    panic!(
                        "FAST_HEUR fixture {}: `{}` entry must be u64",
                        path.display(),
                        field
                    )
                });
                u32::try_from(raw).unwrap_or_else(|_| {
                    panic!(
                        "FAST_HEUR fixture {}: `{}` entry {} doesn't fit in u32",
                        path.display(),
                        field,
                        raw
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("degree_sequence_game_fast_heur_simple");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read FAST_HEUR fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read FAST_HEUR fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse FAST_HEUR conformance fixture JSON");
            assert_eq!(case.algo, "degree_sequence_game_fast_heur_simple");

            let out_value = case.params.get("out_degrees").unwrap_or_else(|| {
                panic!(
                    "FAST_HEUR fixture {}: param `out_degrees` missing",
                    path.display()
                )
            });
            let out_degrees = json_u32_vec(out_value, &path, "params.out_degrees");
            let in_degrees: Option<Vec<u32>> = case.params.get("in_degrees").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    Some(json_u32_vec(v, &path, "params.in_degrees"))
                }
            });
            let seed = er_param_u64(&case, "seed", &path);

            let graph = degree_sequence_game_fast_heur_simple(
                &out_degrees,
                in_degrees.as_deref(),
                seed,
            )
            .expect("degree_sequence_game_fast_heur_simple should succeed on conformance fixtures");

            let want_vcount = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);

            assert_eq!(
                graph.vcount(),
                want_vcount,
                "FAST_HEUR vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "FAST_HEUR directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                u64::try_from(graph.ecount()).expect("ecount fits in u64"),
                want_edges,
                "FAST_HEUR ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Observed (out- or all-)degree sequence must match input exactly.
            let vcount = graph.vcount() as usize;
            let mut observed_out = vec![0u32; vcount];
            let mut observed_in = vec![0u32; vcount];
            let ecount = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            for eid in 0..ecount {
                let (src_vid, dst_vid) = graph.edge(eid).expect("edge id in bounds");
                if graph.is_directed() {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_in[dst_vid as usize] = observed_in[dst_vid as usize].saturating_add(1);
                } else {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_out[dst_vid as usize] =
                        observed_out[dst_vid as usize].saturating_add(1);
                }
            }
            let want_out = json_u32_vec(
                case.expected.get("out_degrees").unwrap_or_else(|| {
                    panic!(
                        "FAST_HEUR fixture {}: expected.out_degrees missing",
                        path.display()
                    )
                }),
                &path,
                "expected.out_degrees",
            );
            assert_eq!(
                observed_out,
                want_out,
                "FAST_HEUR out/all-degree mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if graph.is_directed() {
                let want_in_val = case.expected.get("in_degrees").unwrap_or_else(|| {
                    panic!(
                        "FAST_HEUR fixture {}: expected.in_degrees missing (directed)",
                        path.display()
                    )
                });
                if !want_in_val.is_null() {
                    let want_in = json_u32_vec(want_in_val, &path, "expected.in_degrees");
                    assert_eq!(
                        observed_in,
                        want_in,
                        "FAST_HEUR in-degree mismatch in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            // Simplicity invariant (no self-loops, no multi-edges).
            let want_simple = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let observed_simple = is_simple_with_mode(&graph, SimpleMode::DirectedAsDirected)
                .expect("is_simple should succeed on FAST_HEUR output");
            assert_eq!(
                observed_simple,
                want_simple,
                "FAST_HEUR simplicity mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no degree_sequence_game_fast_heur_simple fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn degree_sequence_game_configuration_simple_three_source_conformance() {
    // Configuration-model simple-graph degree-sequence generator
    // (ALGO-GN-027). Two-swap-per-edge incremental Fisher-Yates over the
    // flat stub bag with restart-on-collision (self-loop or multi-edge),
    // returning a *simple* (no self-loops, no multi-edges) undirected or
    // directed graph realising the input sequence exactly. RNG state is
    // not portable to upstream igraph C / NumPy / R, so the fixtures pin
    // structural invariants only: vcount, ecount, exact (out/in-)degree
    // match, simplicity. Connectivity is NOT guaranteed by this method.
    use rust_igraph::{SimpleMode, degree_sequence_game_configuration_simple, is_simple_with_mode};

    fn json_u32_vec(value: &serde_json::Value, path: &std::path::Path, field: &str) -> Vec<u32> {
        let array = value.as_array().unwrap_or_else(|| {
            panic!(
                "CONFIG_SIMPLE fixture {}: `{}` must be a JSON array",
                path.display(),
                field
            )
        });
        array
            .iter()
            .map(|item| {
                let raw = item.as_u64().unwrap_or_else(|| {
                    panic!(
                        "CONFIG_SIMPLE fixture {}: `{}` entry must be u64",
                        path.display(),
                        field
                    )
                });
                u32::try_from(raw).unwrap_or_else(|_| {
                    panic!(
                        "CONFIG_SIMPLE fixture {}: `{}` entry {} doesn't fit in u32",
                        path.display(),
                        field,
                        raw
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("degree_sequence_game_configuration_simple");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read CONFIG_SIMPLE fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read CONFIG_SIMPLE fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse CONFIG_SIMPLE conformance fixture JSON");
            assert_eq!(case.algo, "degree_sequence_game_configuration_simple");

            let out_value = case.params.get("out_degrees").unwrap_or_else(|| {
                panic!(
                    "CONFIG_SIMPLE fixture {}: param `out_degrees` missing",
                    path.display()
                )
            });
            let out_degrees = json_u32_vec(out_value, &path, "params.out_degrees");
            let in_degrees: Option<Vec<u32>> = case.params.get("in_degrees").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    Some(json_u32_vec(v, &path, "params.in_degrees"))
                }
            });
            let seed = er_param_u64(&case, "seed", &path);

            let graph = degree_sequence_game_configuration_simple(
                &out_degrees,
                in_degrees.as_deref(),
                seed,
            )
            .expect(
                "degree_sequence_game_configuration_simple should succeed on conformance fixtures",
            );

            let want_vcount = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);

            assert_eq!(
                graph.vcount(),
                want_vcount,
                "CONFIG_SIMPLE vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "CONFIG_SIMPLE directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                u64::try_from(graph.ecount()).expect("ecount fits in u64"),
                want_edges,
                "CONFIG_SIMPLE ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Observed (out- or all-)degree sequence must match input exactly.
            let vcount = graph.vcount() as usize;
            let mut observed_out = vec![0u32; vcount];
            let mut observed_in = vec![0u32; vcount];
            let ecount = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            for eid in 0..ecount {
                let (src_vid, dst_vid) = graph.edge(eid).expect("edge id in bounds");
                if graph.is_directed() {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_in[dst_vid as usize] = observed_in[dst_vid as usize].saturating_add(1);
                } else {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_out[dst_vid as usize] =
                        observed_out[dst_vid as usize].saturating_add(1);
                }
            }
            let want_out = json_u32_vec(
                case.expected.get("out_degrees").unwrap_or_else(|| {
                    panic!(
                        "CONFIG_SIMPLE fixture {}: expected.out_degrees missing",
                        path.display()
                    )
                }),
                &path,
                "expected.out_degrees",
            );
            assert_eq!(
                observed_out,
                want_out,
                "CONFIG_SIMPLE out/all-degree mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if graph.is_directed() {
                let want_in_val = case.expected.get("in_degrees").unwrap_or_else(|| {
                    panic!(
                        "CONFIG_SIMPLE fixture {}: expected.in_degrees missing (directed)",
                        path.display()
                    )
                });
                if !want_in_val.is_null() {
                    let want_in = json_u32_vec(want_in_val, &path, "expected.in_degrees");
                    assert_eq!(
                        observed_in,
                        want_in,
                        "CONFIG_SIMPLE in-degree mismatch in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            let want_simple = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let observed_simple = is_simple_with_mode(&graph, SimpleMode::DirectedAsDirected)
                .expect("is_simple should succeed on CONFIG_SIMPLE output");
            assert_eq!(
                observed_simple,
                want_simple,
                "CONFIG_SIMPLE simplicity mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no degree_sequence_game_configuration_simple fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn degree_sequence_game_edge_switching_simple_three_source_conformance() {
    // Edge-switching MCMC simple-graph degree-sequence generator
    // (ALGO-GN-028). Two-phase: deterministic Havel-Hakimi / Kleitman-Wang
    // INDEX seed, then 10·|E| degree-preserving edge-switching MCMC trials.
    // Cost is linear in |E| regardless of density, so dense / skewed
    // sequences that exceed CONFIGURATION_SIMPLE's restart budget run
    // reliably here. RNG state is not portable to upstream igraph C /
    // NumPy / R, so fixtures pin structural invariants only: vcount,
    // ecount, exact (out/in-)degree match, simplicity. Connectivity is
    // NOT guaranteed (use ALGO-GN-025 VL for that).
    use rust_igraph::{
        SimpleMode, degree_sequence_game_edge_switching_simple, is_simple_with_mode,
    };

    fn json_u32_vec(value: &serde_json::Value, path: &std::path::Path, field: &str) -> Vec<u32> {
        let array = value.as_array().unwrap_or_else(|| {
            panic!(
                "EDGE_SWITCHING_SIMPLE fixture {}: `{}` must be a JSON array",
                path.display(),
                field
            )
        });
        array
            .iter()
            .map(|item| {
                let raw = item.as_u64().unwrap_or_else(|| {
                    panic!(
                        "EDGE_SWITCHING_SIMPLE fixture {}: `{}` entry must be u64",
                        path.display(),
                        field
                    )
                });
                u32::try_from(raw).unwrap_or_else(|_| {
                    panic!(
                        "EDGE_SWITCHING_SIMPLE fixture {}: `{}` entry {} doesn't fit in u32",
                        path.display(),
                        field,
                        raw
                    )
                })
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("degree_sequence_game_edge_switching_simple");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read EDGE_SWITCHING_SIMPLE fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read EDGE_SWITCHING_SIMPLE fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse EDGE_SWITCHING_SIMPLE conformance fixture JSON");
            assert_eq!(case.algo, "degree_sequence_game_edge_switching_simple");

            let out_value = case.params.get("out_degrees").unwrap_or_else(|| {
                panic!(
                    "EDGE_SWITCHING_SIMPLE fixture {}: param `out_degrees` missing",
                    path.display()
                )
            });
            let out_degrees = json_u32_vec(out_value, &path, "params.out_degrees");
            let in_degrees: Option<Vec<u32>> = case.params.get("in_degrees").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    Some(json_u32_vec(v, &path, "params.in_degrees"))
                }
            });
            let seed = er_param_u64(&case, "seed", &path);

            let graph = degree_sequence_game_edge_switching_simple(
                &out_degrees,
                in_degrees.as_deref(),
                seed,
            )
            .expect(
                "degree_sequence_game_edge_switching_simple should succeed on conformance fixtures",
            );

            let want_vcount = er_expected_u32(&case, "vcount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);

            assert_eq!(
                graph.vcount(),
                want_vcount,
                "EDGE_SWITCHING_SIMPLE vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "EDGE_SWITCHING_SIMPLE directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                u64::try_from(graph.ecount()).expect("ecount fits in u64"),
                want_edges,
                "EDGE_SWITCHING_SIMPLE ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            // Observed (out- or all-)degree sequence must match input exactly.
            let vcount = graph.vcount() as usize;
            let mut observed_out = vec![0u32; vcount];
            let mut observed_in = vec![0u32; vcount];
            let ecount = u32::try_from(graph.ecount()).expect("ecount fits in u32");
            for eid in 0..ecount {
                let (src_vid, dst_vid) = graph.edge(eid).expect("edge id in bounds");
                if graph.is_directed() {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_in[dst_vid as usize] = observed_in[dst_vid as usize].saturating_add(1);
                } else {
                    observed_out[src_vid as usize] =
                        observed_out[src_vid as usize].saturating_add(1);
                    observed_out[dst_vid as usize] =
                        observed_out[dst_vid as usize].saturating_add(1);
                }
            }
            let want_out = json_u32_vec(
                case.expected.get("out_degrees").unwrap_or_else(|| {
                    panic!(
                        "EDGE_SWITCHING_SIMPLE fixture {}: expected.out_degrees missing",
                        path.display()
                    )
                }),
                &path,
                "expected.out_degrees",
            );
            assert_eq!(
                observed_out,
                want_out,
                "EDGE_SWITCHING_SIMPLE out/all-degree mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            if graph.is_directed() {
                let want_in_val = case.expected.get("in_degrees").unwrap_or_else(|| {
                    panic!(
                        "EDGE_SWITCHING_SIMPLE fixture {}: expected.in_degrees missing (directed)",
                        path.display()
                    )
                });
                if !want_in_val.is_null() {
                    let want_in = json_u32_vec(want_in_val, &path, "expected.in_degrees");
                    assert_eq!(
                        observed_in,
                        want_in,
                        "EDGE_SWITCHING_SIMPLE in-degree mismatch in {}\n  source: {}\n  origin: {}",
                        path.display(),
                        case.source,
                        case.origin,
                    );
                }
            }

            let want_simple = case
                .expected
                .get("is_simple")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let observed_simple = is_simple_with_mode(&graph, SimpleMode::DirectedAsDirected)
                .expect("is_simple should succeed on EDGE_SWITCHING_SIMPLE output");
            assert_eq!(
                observed_simple,
                want_simple,
                "EDGE_SWITCHING_SIMPLE simplicity mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no degree_sequence_game_edge_switching_simple fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + edge-list compare (directed vs canonical-multiset)
fn ring_graph_three_source_conformance() {
    // Ring is a *deterministic constructor*: no RNG, the expected block
    // carries an exact edge list in upstream raw order. Rust storage
    // canonicalises undirected edges (min endpoint first) and igraph C
    // does not, so we compare via multisets of canonicalised tuples for
    // undirected graphs and exact ordered vectors for directed graphs.
    use rust_igraph::ring_graph;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("ring_graph");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read ring fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse ring conformance fixture JSON");
            assert_eq!(case.algo, "ring_graph");

            let n = er_param_u32(&case, "n", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let mutual = er_param_bool(&case, "mutual", &path);
            let circular = er_param_bool(&case, "circular", &path);

            let graph = ring_graph(n, directed, mutual, circular)
                .expect("ring_graph should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "ring fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "ring fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| graph.edge(eid).expect("conformance ring edge id in bounds"))
                .collect();

            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed ring edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected ring edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no ring_graph fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 4-mode parse + directed/canon compare
fn star_graph_three_source_conformance() {
    // Star is a *deterministic constructor*: no RNG, the expected block
    // carries an exact edge list in upstream raw order. Rust storage
    // canonicalises undirected edges (min endpoint first) and the
    // three reference implementations do not, so we compare via
    // multisets of canonicalised tuples for undirected graphs and
    // exact ordered vectors for directed graphs (matches the upstream
    // C loop's leaf-iteration order: [0, center) then (center, n)).
    use rust_igraph::{StarMode, star_graph};

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_mode(case: &Conformance, path: &std::path::Path) -> StarMode {
        let raw = case
            .params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "star fixture {}: params.mode missing or not string",
                    path.display()
                )
            });
        match raw {
            "Out" => StarMode::Out,
            "In" => StarMode::In,
            "Mutual" => StarMode::Mutual,
            "Undirected" => StarMode::Undirected,
            other => panic!("star fixture {}: unknown mode {other:?}", path.display()),
        }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("star_graph");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read star fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse star conformance fixture JSON");
            assert_eq!(case.algo, "star_graph");

            let n = er_param_u32(&case, "n", &path);
            let center = er_param_u32(&case, "center", &path);
            let mode = parse_mode(&case, &path);

            let graph = star_graph(n, mode, center)
                .expect("star_graph should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "star fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "star fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| graph.edge(eid).expect("conformance star edge id in bounds"))
                .collect();

            let directed = matches!(mode, StarMode::Out | StarMode::In | StarMode::Mutual);
            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed star edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected star edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no star_graph fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 4-mode parse + directed/canon compare
fn wheel_graph_three_source_conformance() {
    // Wheel = star ∪ rim cycle. Edge sequence is deterministic and
    // documented in `regular.c:igraph_wheel`: the star block is emitted
    // first (per StarMode), then the rim sweep `e_0..e_{n-2}`, then —
    // for Mutual mode only — the reverse of every rim arc in
    // reverse-discovery order. Rust storage canonicalises undirected
    // edges, so we compare canonical multisets for `Undirected` and
    // exact ordered vectors for the directed modes.
    use rust_igraph::{WheelMode, wheel_graph};

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_mode(case: &Conformance, path: &std::path::Path) -> WheelMode {
        let raw = case
            .params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "wheel fixture {}: params.mode missing or not string",
                    path.display()
                )
            });
        match raw {
            "Out" => WheelMode::Out,
            "In" => WheelMode::In,
            "Mutual" => WheelMode::Mutual,
            "Undirected" => WheelMode::Undirected,
            other => panic!("wheel fixture {}: unknown mode {other:?}", path.display()),
        }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("wheel_graph");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read wheel fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse wheel conformance fixture JSON");
            assert_eq!(case.algo, "wheel_graph");

            let n = er_param_u32(&case, "n", &path);
            let center = er_param_u32(&case, "center", &path);
            let mode = parse_mode(&case, &path);

            let graph = wheel_graph(n, mode, center)
                .expect("wheel_graph should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "wheel fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "wheel fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance wheel edge id in bounds")
                })
                .collect();

            let directed = matches!(mode, WheelMode::Out | WheelMode::In | WheelMode::Mutual);
            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed wheel edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected wheel edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no wheel_graph fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 3-mode parse + directed/canon compare
fn kary_tree_three_source_conformance() {
    // k-ary tree builds a BFS-ordered rooted tree: parent 0 → children
    // 1..=children, then parent 1 → next children, etc. Edge sequence is
    // deterministic and documented in `regular.c:igraph_kary_tree`. For
    // OUT/UNDIRECTED the raw arc is `(parent, child)`; for IN it is
    // `(child, parent)`. Rust storage canonicalises undirected edges,
    // so we compare canonical multisets for Undirected and exact
    // ordered vectors for the directed modes.
    use rust_igraph::{TreeMode, kary_tree};

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_mode(case: &Conformance, path: &std::path::Path) -> TreeMode {
        let raw = case
            .params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "kary_tree fixture {}: params.mode missing or not string",
                    path.display()
                )
            });
        match raw {
            "Out" => TreeMode::Out,
            "In" => TreeMode::In,
            "Undirected" => TreeMode::Undirected,
            other => panic!(
                "kary_tree fixture {}: unknown mode {other:?}",
                path.display()
            ),
        }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("kary_tree");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read kary_tree fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse kary_tree conformance fixture JSON");
            assert_eq!(case.algo, "kary_tree");

            let n = er_param_u32(&case, "n", &path);
            let children = er_param_u32(&case, "children", &path);
            let mode = parse_mode(&case, &path);

            let graph = kary_tree(n, children, mode)
                .expect("kary_tree should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "kary_tree fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "kary_tree fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance kary_tree edge id in bounds")
                })
                .collect();

            let directed = matches!(mode, TreeMode::Out | TreeMode::In);
            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed kary_tree edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected kary_tree edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no kary_tree fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 3-mode parse + directed/canon compare
fn symmetric_tree_three_source_conformance() {
    // Symmetric tree: per-level branching `branches[d]` children at
    // depth d. BFS layout, same parent→child ordering as kary_tree.
    // Documented in `regular.c:igraph_symmetric_tree`.
    use rust_igraph::{TreeMode, symmetric_tree};

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_mode(case: &Conformance, path: &std::path::Path) -> TreeMode {
        let raw = case
            .params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "symmetric_tree fixture {}: params.mode missing or not string",
                    path.display()
                )
            });
        match raw {
            "Out" => TreeMode::Out,
            "In" => TreeMode::In,
            "Undirected" => TreeMode::Undirected,
            other => panic!(
                "symmetric_tree fixture {}: unknown mode {other:?}",
                path.display()
            ),
        }
    }

    fn parse_branches(case: &Conformance, path: &std::path::Path) -> Vec<u32> {
        let raw = case
            .params
            .get("branches")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "symmetric_tree fixture {}: params.branches missing or not array",
                    path.display()
                )
            });
        raw.iter()
            .map(|v| {
                u32::try_from(v.as_u64().unwrap_or_else(|| {
                    panic!(
                        "symmetric_tree fixture {}: branches entry not u64",
                        path.display()
                    )
                }))
                .expect("branches entry fits in u32")
            })
            .collect()
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("symmetric_tree");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read symmetric_tree fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse symmetric_tree conformance fixture JSON");
            assert_eq!(case.algo, "symmetric_tree");

            let branches = parse_branches(&case, &path);
            let mode = parse_mode(&case, &path);

            let graph = symmetric_tree(&branches, mode)
                .expect("symmetric_tree should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "symmetric_tree fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "symmetric_tree fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance symmetric_tree edge id in bounds")
                })
                .collect();

            let directed = matches!(mode, TreeMode::Out | TreeMode::In);
            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed symmetric_tree edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected symmetric_tree edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no symmetric_tree fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + 3-mode parse + directed/canon compare
fn regular_tree_three_source_conformance() {
    // Regular tree (Bethe lattice): every non-leaf vertex has total
    // degree exactly k. Implemented in C as a thin wrapper over
    // `igraph_symmetric_tree(branches=[k, k-1, ..., k-1], len=h)`.
    // Documented in `regular.c:igraph_regular_tree`.
    use rust_igraph::{TreeMode, regular_tree};

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_mode(case: &Conformance, path: &std::path::Path) -> TreeMode {
        let raw = case
            .params
            .get("mode")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "regular_tree fixture {}: params.mode missing or not string",
                    path.display()
                )
            });
        match raw {
            "Out" => TreeMode::Out,
            "In" => TreeMode::In,
            "Undirected" => TreeMode::Undirected,
            other => panic!(
                "regular_tree fixture {}: unknown mode {other:?}",
                path.display()
            ),
        }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("regular_tree");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read regular_tree fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse regular_tree conformance fixture JSON");
            assert_eq!(case.algo, "regular_tree");

            let h = er_param_u32(&case, "h", &path);
            let k = er_param_u32(&case, "k", &path);
            let mode = parse_mode(&case, &path);

            let graph = regular_tree(h, k, mode)
                .expect("regular_tree should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "regular_tree fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "regular_tree fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance regular_tree edge id in bounds")
                })
                .collect();

            let directed = matches!(mode, TreeMode::Out | TreeMode::In);
            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed regular_tree edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected regular_tree edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no regular_tree fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + directed/canon compare
fn hypercube_three_source_conformance() {
    // n-dimensional hypercube Q_n: 2^n vertices, edge iff IDs differ
    // in exactly one bit. Documented in `regular.c:igraph_hypercube`.
    use rust_igraph::hypercube;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("hypercube");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read hypercube fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse hypercube conformance fixture JSON");
            assert_eq!(case.algo, "hypercube");

            let n = er_param_u32(&case, "n", &path);
            let directed = er_param_bool(&case, "directed", &path);

            let graph =
                hypercube(n, directed).expect("hypercube should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "hypercube fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "hypercube fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance hypercube edge id in bounds")
                })
                .collect();

            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed hypercube edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected hypercube edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no hypercube fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + directed/canon compare
fn hamming_three_source_conformance() {
    // d-dimensional Hamming graph H(n, q): q^n vertices indexed by
    // base-q digit strings, edge iff strings differ in exactly one
    // position. Documented in `regular.c:igraph_hamming`.
    use rust_igraph::hamming;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("hamming");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read hamming fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse hamming conformance fixture JSON");
            assert_eq!(case.algo, "hamming");

            let n = er_param_u32(&case, "n", &path);
            let q = er_param_u32(&case, "q", &path);
            let directed = er_param_bool(&case, "directed", &path);

            let graph =
                hamming(n, q, directed).expect("hamming should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "hamming fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "hamming fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance hamming edge id in bounds")
                })
                .collect();

            if directed {
                assert_eq!(
                    got_pairs,
                    want_pairs,
                    "directed hamming edge sequence mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "undirected hamming edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no hamming fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + directed/canon compare + array params
fn square_lattice_three_source_conformance() {
    // Multi-dimensional square lattice with optional per-axis torus
    // wrap and directed/mutual flags. Documented in
    // `regular.c:igraph_square_lattice`. Only `nei in {0, 1}` is
    // supported by this port; fixtures all set `nei = 1`.
    use rust_igraph::square_lattice;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn parse_u32_array(case: &Conformance, key: &str, path: &std::path::Path) -> Vec<u32> {
        let Some(arr) = case.params.get(key).and_then(serde_json::Value::as_array) else {
            panic!(
                "square_lattice fixture {}: param `{}` missing or not array",
                path.display(),
                key
            );
        };
        arr.iter()
            .map(|v| {
                u32::try_from(v.as_u64().unwrap_or_else(|| {
                    panic!(
                        "square_lattice fixture {}: param `{}` element not u64",
                        path.display(),
                        key
                    )
                }))
                .expect("dim element fits u32")
            })
            .collect()
    }

    fn parse_optional_bool_array(
        case: &Conformance,
        key: &str,
        path: &std::path::Path,
    ) -> Option<Vec<bool>> {
        match case.params.get(key) {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::Array(arr)) => Some(
                arr.iter()
                    .map(|v| {
                        v.as_bool().unwrap_or_else(|| {
                            panic!(
                                "square_lattice fixture {}: param `{}` element not bool",
                                path.display(),
                                key
                            )
                        })
                    })
                    .collect(),
            ),
            _ => panic!(
                "square_lattice fixture {}: param `{}` neither null nor array",
                path.display(),
                key
            ),
        }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("square_lattice");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read square_lattice fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse square_lattice conformance fixture JSON");
            assert_eq!(case.algo, "square_lattice");

            let dims = parse_u32_array(&case, "dim", &path);
            let nei = er_param_u32(&case, "nei", &path);
            let directed = er_param_bool(&case, "directed", &path);
            let mutual = er_param_bool(&case, "mutual", &path);
            let periodic = parse_optional_bool_array(&case, "periodic", &path);

            let graph = square_lattice(&dims, nei, directed, mutual, periodic.as_deref())
                .expect("square_lattice should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "square_lattice fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "square_lattice fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance square_lattice edge id in bounds")
                })
                .collect();

            // Always compare as a canonicalized multiset: both endpoints
            // are by convention `min(u,v) <= max(u,v)` after Graph::add_edges,
            // but the emission *order* in the C reference is a function of
            // the lattice walk and is not portable across the three sources
            // (python and R cite different vertex orderings even for the
            // same shape). Multiset compare gives a clean cross-impl check.
            let mut got_canon: Vec<(u32, u32)> =
                got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
            let mut want_canon: Vec<(u32, u32)> =
                want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
            got_canon.sort_unstable();
            want_canon.sort_unstable();
            assert_eq!(
                got_canon,
                want_canon,
                "square_lattice edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no square_lattice fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + canonicalised edge multiset compare
fn generalized_petersen_three_source_conformance() {
    // Generalized Petersen graph G(n, k). Constraints: n >= 3 and
    // 0 < k < n/2. Always undirected; |V| = 2n, |E| = 3n.
    //
    // Edge emission order differs between sources:
    //   * C reference (this port) emits (outer, rung, inner) per i.
    //   * python-igraph / R-igraph's Famous-database `Petersen` and
    //     `Dodecahedron` graphs lay out their edges in a different
    //     order entirely.
    // We therefore compare the canonicalised edge multiset, mirroring
    // the strategy used by square_lattice above.
    use rust_igraph::generalized_petersen;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("generalized_petersen");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read generalized_petersen fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance = serde_json::from_slice(&bytes)
                .expect("parse generalized_petersen conformance fixture JSON");
            assert_eq!(case.algo, "generalized_petersen");

            let n = er_param_u32(&case, "n", &path);
            let k = er_param_u32(&case, "k", &path);

            let graph = generalized_petersen(n, k)
                .expect("generalized_petersen should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "generalized_petersen fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "generalized_petersen fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance generalized_petersen edge id in bounds")
                })
                .collect();

            let mut got_canon: Vec<(u32, u32)> =
                got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
            let mut want_canon: Vec<(u32, u32)> =
                want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
            got_canon.sort_unstable();
            want_canon.sort_unstable();
            assert_eq!(
                got_canon,
                want_canon,
                "generalized_petersen edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            assert_eq!(case.source, src);
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
            "no generalized_petersen fixtures from source {src}"
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // three-source dispatch + canonicalised edge multiset compare
fn circulant_three_source_conformance() {
    // Circulant graph G(n, shifts). Edge emission order is driven by
    // the shift list and is not portable across upstream sources, so we
    // compare the canonicalised edge multiset (same strategy as
    // square_lattice / generalized_petersen).
    use rust_igraph::circulant;

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("circulant");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read circulant fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse circulant conformance fixture JSON");
            assert_eq!(case.algo, "circulant");

            let n = er_param_u32(&case, "n", &path);
            let directed = er_param_bool(&case, "directed", &path);

            let shifts_raw = case
                .params
                .get("shifts")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "circulant fixture {}: params.shifts missing or not array",
                        path.display()
                    )
                });
            let shifts: Vec<i64> = shifts_raw
                .iter()
                .map(|v| {
                    v.as_i64().unwrap_or_else(|| {
                        panic!(
                            "circulant fixture {}: params.shifts entry not i64",
                            path.display()
                        )
                    })
                })
                .collect();

            let graph = circulant(n, &shifts, directed)
                .expect("circulant should succeed on conformance fixtures");

            let want_vertices = er_expected_u32(&case, "vcount", &path);
            let want_edges = er_expected_u64(&case, "ecount", &path);
            let want_directed = er_expected_bool(&case, "directed", &path);

            assert_eq!(
                graph.vcount(),
                want_vertices,
                "vcount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.is_directed(),
                want_directed,
                "directed mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );
            assert_eq!(
                graph.ecount() as u64,
                want_edges,
                "ecount mismatch in {}\n  source: {}\n  origin: {}",
                path.display(),
                case.source,
                case.origin,
            );

            let want_edges_raw = case
                .expected
                .get("edges")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "circulant fixture {}: expected.edges missing or not array",
                        path.display()
                    )
                });
            let mut want_pairs: Vec<(u32, u32)> = Vec::with_capacity(want_edges_raw.len());
            for v in want_edges_raw {
                let pair = v.as_array().unwrap_or_else(|| {
                    panic!(
                        "circulant fixture {}: expected.edges entry not array",
                        path.display()
                    )
                });
                let u = u32::try_from(pair[0].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                let w = u32::try_from(pair[1].as_u64().expect("edge endpoint u64"))
                    .expect("edge endpoint fits in u32");
                want_pairs.push((u, w));
            }

            let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
            let got_pairs: Vec<(u32, u32)> = (0..n_edges)
                .map(|eid| {
                    graph
                        .edge(eid)
                        .expect("conformance circulant edge id in bounds")
                })
                .collect();

            // For directed circulants the edge orientation matters; for
            // undirected ones we canonicalise (lo, hi). This matches how
            // we compare undirected edges everywhere else in the suite.
            if directed {
                let mut got_sorted = got_pairs.clone();
                let mut want_sorted = want_pairs.clone();
                got_sorted.sort_unstable();
                want_sorted.sort_unstable();
                assert_eq!(
                    got_sorted,
                    want_sorted,
                    "circulant (directed) edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            } else {
                let mut got_canon: Vec<(u32, u32)> =
                    got_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                let mut want_canon: Vec<(u32, u32)> =
                    want_pairs.iter().map(|&(u, v)| canon(u, v)).collect();
                got_canon.sort_unstable();
                want_canon.sort_unstable();
                assert_eq!(
                    got_canon,
                    want_canon,
                    "circulant edge multiset mismatch in {}\n  source: {}\n  origin: {}",
                    path.display(),
                    case.source,
                    case.origin,
                );
            }

            assert_eq!(case.source, src);
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
            "no circulant fixtures from source {src}"
        );
    }
}

fn check_de_bruijn_fixture(case: &Conformance, path: &std::path::Path) {
    use rust_igraph::de_bruijn;

    let m = er_param_u32(case, "m", path);
    let n = er_param_u32(case, "n", path);

    let graph = de_bruijn(m, n).expect("de_bruijn should succeed on conformance fixtures");

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_edges = er_expected_u64(case, "ecount", path);
    let want_directed = er_expected_bool(case, "directed", path);

    assert_eq!(
        graph.vcount(),
        want_vertices,
        "vcount mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.is_directed(),
        want_directed,
        "directed mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.ecount() as u64,
        want_edges,
        "ecount mismatch in {}",
        path.display()
    );

    let want_edges_raw = case
        .expected
        .get("edges")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "de_bruijn fixture {}: expected.edges missing",
                path.display()
            )
        });
    let want_pairs: Vec<(u32, u32)> = want_edges_raw
        .iter()
        .map(|v| {
            let pair = v
                .as_array()
                .unwrap_or_else(|| panic!("de_bruijn fixture {}: edge not array", path.display()));
            let u = u32::try_from(pair[0].as_u64().expect("u64")).expect("u32");
            let w = u32::try_from(pair[1].as_u64().expect("u64")).expect("u32");
            (u, w)
        })
        .collect();

    let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
    let got_pairs: Vec<(u32, u32)> = (0..n_edges)
        .map(|eid| {
            graph
                .edge(eid)
                .expect("conformance de_bruijn edge id in bounds")
        })
        .collect();

    assert_eq!(
        got_pairs,
        want_pairs,
        "de_bruijn arc list mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
}

#[test]
fn de_bruijn_three_source_conformance() {
    // De Bruijn B(m, n) is always directed and its arc emission order
    // is deterministic and identical across upstream C, python-igraph
    // and rigraph (they all dispatch to `igraph_de_bruijn`). We can
    // therefore compare the raw ordered arc list rather than the
    // canonicalised multiset — drift in emission order would be a real
    // regression.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("de_bruijn");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read de_bruijn fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse de_bruijn conformance fixture JSON");
            assert_eq!(case.algo, "de_bruijn");
            assert_eq!(case.source, src);
            check_de_bruijn_fixture(&case, &path);
            seen_sources.insert(src);
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no de_bruijn fixtures from source {src}"
        );
    }
}

fn check_kautz_fixture(case: &Conformance, path: &std::path::Path) {
    use rust_igraph::kautz;

    let m = er_param_u32(case, "m", path);
    let n = er_param_u32(case, "n", path);

    let graph = kautz(m, n).expect("kautz should succeed on conformance fixtures");

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_edges = er_expected_u64(case, "ecount", path);
    let want_directed = er_expected_bool(case, "directed", path);

    assert_eq!(
        graph.vcount(),
        want_vertices,
        "vcount mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.is_directed(),
        want_directed,
        "directed mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.ecount() as u64,
        want_edges,
        "ecount mismatch in {}",
        path.display()
    );

    let want_edges_raw = case
        .expected
        .get("edges")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("kautz fixture {}: expected.edges missing", path.display()));
    let want_pairs: Vec<(u32, u32)> = want_edges_raw
        .iter()
        .map(|v| {
            let pair = v
                .as_array()
                .unwrap_or_else(|| panic!("kautz fixture {}: edge not array", path.display()));
            let u = u32::try_from(pair[0].as_u64().expect("u64")).expect("u32");
            let w = u32::try_from(pair[1].as_u64().expect("u64")).expect("u32");
            (u, w)
        })
        .collect();

    let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
    let got_pairs: Vec<(u32, u32)> = (0..n_edges)
        .map(|eid| {
            graph
                .edge(eid)
                .expect("conformance kautz edge id in bounds")
        })
        .collect();

    assert_eq!(
        got_pairs,
        want_pairs,
        "kautz arc list mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
}

#[test]
fn kautz_three_source_conformance() {
    // K(m, n) is always directed and the C source emits arcs in a fixed
    // source-major, target-ascending order. python-igraph (`Graph.Kautz`)
    // and rigraph (`make_kautz_graph`) both dispatch to `igraph_kautz`,
    // so the ordered arc list is identical across the three oracles.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("kautz");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read kautz fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse kautz conformance fixture JSON");
            assert_eq!(case.algo, "kautz");
            assert_eq!(case.source, src);
            check_kautz_fixture(&case, &path);
            seen_sources.insert(src);
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no kautz fixtures from source {src}"
        );
    }
}

fn check_full_graph_fixture(case: &Conformance, path: &std::path::Path) {
    use rust_igraph::full_graph;

    let n = er_param_u32(case, "n", path);
    let directed = er_param_bool(case, "directed", path);
    let loops = er_param_bool(case, "loops", path);

    let graph =
        full_graph(n, directed, loops).expect("full_graph should succeed on conformance fixtures");

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_edges = er_expected_u64(case, "ecount", path);
    let want_directed = er_expected_bool(case, "directed", path);

    assert_eq!(
        graph.vcount(),
        want_vertices,
        "vcount mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.is_directed(),
        want_directed,
        "directed mismatch in {}",
        path.display()
    );
    assert_eq!(
        graph.ecount() as u64,
        want_edges,
        "ecount mismatch in {}",
        path.display()
    );

    let want_edges_raw = case
        .expected
        .get("edges")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "full_graph fixture {}: expected.edges missing",
                path.display()
            )
        });
    let want_pairs: Vec<(u32, u32)> = want_edges_raw
        .iter()
        .map(|v| {
            let pair = v
                .as_array()
                .unwrap_or_else(|| panic!("full_graph fixture {}: edge not array", path.display()));
            let u = u32::try_from(pair[0].as_u64().expect("u64")).expect("u32");
            let w = u32::try_from(pair[1].as_u64().expect("u64")).expect("u32");
            (u, w)
        })
        .collect();

    let n_edges = u32::try_from(graph.ecount()).expect("ecount fits in u32 in conformance");
    let got_pairs: Vec<(u32, u32)> = (0..n_edges)
        .map(|eid| {
            graph
                .edge(eid)
                .expect("conformance full_graph edge id in bounds")
        })
        .collect();

    assert_eq!(
        got_pairs,
        want_pairs,
        "full_graph edge list mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
}

#[test]
fn full_graph_three_source_conformance() {
    // `igraph_full(n, directed, loops)` emits edges in a fixed,
    // source-major, target-ascending order across all four (directed,
    // loops) combinations. python-igraph (`Graph.Full`) and rigraph
    // (`make_full_graph`) both dispatch to `igraph_full`, so the ordered
    // edge list is identical across the three oracles.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("full_graph");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read full_graph fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse full_graph conformance fixture JSON");
            assert_eq!(case.algo, "full_graph");
            assert_eq!(case.source, src);
            check_full_graph_fixture(&case, &path);
            seen_sources.insert(src);
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no full_graph fixtures from source {src}"
        );
    }
}

fn check_linegraph_fixture(case: &Conformance, path: &std::path::Path) {
    use rust_igraph::linegraph;
    use std::collections::BTreeMap;

    let g = build_graph(&case.graph);
    let l = linegraph(&g).expect("linegraph should succeed on conformance fixtures");

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_edges = er_expected_u64(case, "ecount", path);
    let want_directed = er_expected_bool(case, "directed", path);

    assert_eq!(
        l.vcount(),
        want_vertices,
        "linegraph vcount mismatch in {}",
        path.display()
    );
    assert_eq!(
        l.is_directed(),
        want_directed,
        "linegraph directed mismatch in {}",
        path.display()
    );
    assert_eq!(
        l.ecount() as u64,
        want_edges,
        "linegraph ecount mismatch in {}",
        path.display()
    );

    let want_edges_raw = case
        .expected
        .get("edges")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "linegraph fixture {}: expected.edges missing",
                path.display()
            )
        });

    // Canonicalise both sides as a multiset of unordered pairs (for
    // undirected) or ordered pairs (for directed). `igraph_linegraph` and
    // our Rust port emit edges in the same raw order, but `Graph::edge`
    // returns undirected endpoints as (min, max) by construction, while
    // upstream python-igraph keeps the (smaller, larger) canonical form
    // too — yet differing internal sort orders across rigraph versions
    // make raw-ordered comparison brittle. Comparing as multisets is the
    // semantically correct check that matches what `igraph_is_same_graph`
    // (the upstream C unit-test assertion) does.
    let mut want_ms: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for v in want_edges_raw {
        let pair = v
            .as_array()
            .unwrap_or_else(|| panic!("linegraph fixture {}: edge not array", path.display()));
        let u = u32::try_from(pair[0].as_u64().expect("u64")).expect("u32");
        let w = u32::try_from(pair[1].as_u64().expect("u64")).expect("u32");
        let key = if want_directed || u <= w {
            (u, w)
        } else {
            (w, u)
        };
        *want_ms.entry(key).or_insert(0) += 1;
    }

    let n_edges = u32::try_from(l.ecount()).expect("ecount fits u32 in conformance");
    let mut got_ms: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for eid in 0..n_edges {
        let (u, w) = l.edge(eid).expect("linegraph edge id in range");
        let key = if want_directed || u <= w {
            (u, w)
        } else {
            (w, u)
        };
        *got_ms.entry(key).or_insert(0) += 1;
    }

    assert_eq!(
        got_ms,
        want_ms,
        "linegraph edge multiset mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
}

#[test]
fn linegraph_three_source_conformance() {
    // `igraph_linegraph` is reached identically by python-igraph
    // (`Graph.linegraph`) and rigraph (`make_line_graph`). The upstream
    // C unit test asserts `igraph_is_same_graph` — i.e. multiset
    // equality of edges — so this test compares multisets of (canonical)
    // endpoint pairs rather than raw ordered lists.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("linegraph");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read linegraph fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse linegraph conformance fixture JSON");
            assert_eq!(case.algo, "linegraph");
            assert_eq!(case.source, src);
            check_linegraph_fixture(&case, &path);
            seen_sources.insert(src);
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no linegraph fixtures from source {src}"
        );
    }
}

fn check_from_prufer_fixture(case: &Conformance, path: &std::path::Path) {
    use rust_igraph::from_prufer;
    use std::collections::BTreeMap;

    let prufer_vals: Vec<u32> = case
        .params
        .get("prufer")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "from_prufer fixture {}: params.prufer missing or not array",
                path.display()
            )
        })
        .iter()
        .map(|v| {
            u32::try_from(v.as_u64().unwrap_or_else(|| {
                panic!(
                    "from_prufer fixture {}: prufer entry not u64",
                    path.display()
                )
            }))
            .expect("prufer entry fits in u32")
        })
        .collect();

    let tree =
        from_prufer(&prufer_vals).expect("from_prufer should succeed on conformance fixtures");

    let want_vertices = er_expected_u32(case, "vcount", path);
    let want_edges = er_expected_u64(case, "ecount", path);
    let want_directed = er_expected_bool(case, "directed", path);

    assert_eq!(
        tree.vcount(),
        want_vertices,
        "from_prufer vcount mismatch in {}",
        path.display()
    );
    assert_eq!(
        tree.is_directed(),
        want_directed,
        "from_prufer directed mismatch in {}",
        path.display()
    );
    assert_eq!(
        tree.ecount() as u64,
        want_edges,
        "from_prufer ecount mismatch in {}",
        path.display()
    );

    let want_edges_raw = case
        .expected
        .get("edges")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "from_prufer fixture {}: expected.edges missing",
                path.display()
            )
        });

    // Always undirected — compare canonical (min, max) multisets so the
    // three sources can keep distinct internal edge orderings.
    let mut want_ms: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for v in want_edges_raw {
        let pair = v
            .as_array()
            .unwrap_or_else(|| panic!("from_prufer fixture {}: edge not array", path.display()));
        let u = u32::try_from(pair[0].as_u64().expect("u64")).expect("u32");
        let w = u32::try_from(pair[1].as_u64().expect("u64")).expect("u32");
        let key = if u <= w { (u, w) } else { (w, u) };
        *want_ms.entry(key).or_insert(0) += 1;
    }

    let n_edges = u32::try_from(tree.ecount()).expect("ecount fits u32 in conformance");
    let mut got_ms: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for eid in 0..n_edges {
        let (u, w) = tree.edge(eid).expect("from_prufer edge id in range");
        let key = if u <= w { (u, w) } else { (w, u) };
        *got_ms.entry(key).or_insert(0) += 1;
    }

    assert_eq!(
        got_ms,
        want_ms,
        "from_prufer edge multiset mismatch in {}\n  source: {}\n  origin: {}",
        path.display(),
        case.source,
        case.origin,
    );
}

#[test]
fn from_prufer_three_source_conformance() {
    // `igraph_from_prufer` is reached identically by python-igraph
    // (`Graph.Prufer`) and rigraph (`make_from_prufer`). The C unit test
    // asserts an exact edge sequence; we compare canonical edge
    // multisets so cross-source orderings stay compatible.
    let mut seen_sources = std::collections::BTreeSet::<&'static str>::new();
    for src in ["c", "py", "r"] {
        let dir = workspace_root()
            .join("tests/conformance")
            .join(src)
            .join("from_prufer");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).expect("read from_prufer fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path).expect("read fixture file");
            let case: Conformance =
                serde_json::from_slice(&bytes).expect("parse from_prufer conformance fixture JSON");
            assert_eq!(case.algo, "from_prufer");
            assert_eq!(case.source, src);
            check_from_prufer_fixture(&case, &path);
            seen_sources.insert(src);
        }
    }
    for src in ["c", "py", "r"] {
        assert!(
            seen_sources.contains(src),
            "no from_prufer fixtures from source {src}"
        );
    }
}
