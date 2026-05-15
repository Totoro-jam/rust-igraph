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
    let mut g = rust_igraph::Graph::with_vertices(payload.n);
    for &(u, v) in &payload.edges {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

#[test]
fn bfs_three_source_conformance() {
    let cases = load_all("bfs");
    assert!(
        !cases.is_empty(),
        "no BFS conformance fixtures found — did you run \
         `.venv/bin/python -m scripts.test_extract.from_c --algo bfs` (and from_py / from_r)?"
    );

    let mut counts = std::collections::HashMap::<&'static str, usize>::new();
    for (path, case) in cases {
        assert_eq!(case.algo, "bfs");
        let root = u32::try_from(
            case.params
                .get("root")
                .and_then(serde_json::Value::as_u64)
                .expect("bfs param `root`"),
        )
        .expect("root fits in u32");
        let g = build_graph(&case.graph);
        let actual = rust_igraph::bfs(&g, root)
            .unwrap_or_else(|e| panic!("bfs failed for {}: {e:?}", path.display()));
        let expected: Vec<u32> = serde_json::from_value(case.expected.clone())
            .unwrap_or_else(|e| panic!("expected vec<u32> in {}: {e}", path.display()));
        assert_eq!(
            actual,
            expected,
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
    // Phase 0 guarantee: at least one fixture from each of the three sources.
    for source in ["c", "py", "r"] {
        assert!(
            counts.get(source).copied().unwrap_or(0) > 0,
            "no fixtures from source {source}"
        );
    }
}
