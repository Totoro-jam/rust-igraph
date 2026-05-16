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
    #[allow(dead_code)]
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
        assert_eq!(
            actual,
            case.expected,
            "conformance failure\n  fixture: {}\n  source:  {}\n  origin:  {}",
            path.display(),
            case.source,
            case.origin
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
