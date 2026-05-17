//! Shared helpers for integration tests.
//!
//! `oracle::run` calls `scripts/oracle.py` via the project-local Python venv
//! (`.venv/bin/python`) and parses the JSON response. `oracle::ok` panics if
//! the call fails so tests can stay focused on equality assertions.

#![allow(dead_code)] // helpers are referenced by individual test binaries

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Canonical wire-format graph payload understood by `scripts/oracle.py`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPayload {
    pub n: u32,
    pub edges: Vec<(u32, u32)>,
    pub directed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights: Option<Vec<f64>>,
}

impl GraphPayload {
    pub fn from_graph(g: &rust_igraph::Graph) -> Self {
        // Reconstruct the edge list by walking neighbors. Directed graphs
        // emit each (u, v) edge once via out-neighbours; undirected graphs
        // emit each pair once via the canonical `u < v` rule. Self-loops
        // on undirected graphs are reported twice by `neighbors()` after
        // ALGO-CORE-001a's indexed-edgelist backend (LOOPS_TWICE default);
        // divide that count to recover the edge multiplicity.
        let directed = g.is_directed();
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for u in 0..g.vcount() {
            if directed {
                // Out-neighbours only; each directed edge appears exactly
                // once across the whole loop.
                for v in g.neighbors(u).expect("vertex in range") {
                    edges.push((u, v));
                }
            } else {
                let mut self_loops = 0;
                for v in g.neighbors(u).expect("vertex in range") {
                    if u == v {
                        self_loops += 1;
                    } else if u < v {
                        edges.push((u, v));
                    }
                }
                for _ in 0..(self_loops / 2) {
                    edges.push((u, u));
                }
            }
        }
        Self {
            n: g.vcount(),
            edges,
            directed,
            weights: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OracleRequest<'a, P: Serialize> {
    pub graph: GraphPayload,
    pub algo: &'a str,
    pub params: P,
}

#[derive(Debug, Deserialize)]
pub struct OracleResponse {
    pub ok: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    #[allow(dead_code)]
    pub version: String,
}

/// Resolve the path to the Python interpreter inside `.venv/`.
///
/// `CARGO_MANIFEST_DIR` is the repo root, so `.venv/bin/python` sits right
/// next to it. Tests panic with an actionable message if the venv is missing.
pub fn venv_python() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".venv/bin/python");
        assert!(
            p.exists(),
            "Python venv not found at {}. Run:\n  python3 -m venv .venv\n  .venv/bin/pip install -r scripts/requirements.txt",
            p.display()
        );
        p
    })
    .as_path()
}

/// Resolve the path to `scripts/oracle.py`.
pub fn oracle_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/oracle.py")
}

/// Call the oracle and return the parsed response. Panics if the subprocess
/// exits abnormally or returns malformed JSON; tests should use [`run_ok`]
/// for the common success path.
pub fn run<P: Serialize>(algo: &str, graph: &rust_igraph::Graph, params: P) -> OracleResponse {
    run_with_weights(algo, graph, None, params)
}

/// Variant that lets tests attach a per-edge `weights` vector to the
/// request. Required for Dijkstra-style algorithms where the graph
/// payload alone doesn't carry weights — `python-igraph` reads them
/// from `graph["weight"]` (we set the attribute on the rebuilt graph
/// via the wire format's `weights` field). When `weights` is `Some`,
/// the `GraphPayload` is built in **stored edge order** (`g.edge(0..m)`)
/// rather than the default by-vertex traversal, so that `weights[e]`
/// stays paired with the right edge across the wire.
pub fn run_with_weights<P: Serialize>(
    algo: &str,
    graph: &rust_igraph::Graph,
    weights: Option<Vec<f64>>,
    params: P,
) -> OracleResponse {
    let payload = if weights.is_some() {
        let m = u32::try_from(graph.ecount()).expect("ecount fits in u32");
        let edges: Vec<(u32, u32)> = (0..m).map(|e| graph.edge(e).expect("edge ok")).collect();
        GraphPayload {
            n: graph.vcount(),
            edges,
            directed: graph.is_directed(),
            weights,
        }
    } else {
        GraphPayload::from_graph(graph)
    };
    let req = OracleRequest {
        graph: payload,
        algo,
        params,
    };
    let stdin_payload = serde_json::to_vec(&req).expect("serialize request");

    let mut child = Command::new(venv_python())
        .arg(oracle_script())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn oracle.py");
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("oracle stdin")
            .write_all(&stdin_payload)
            .expect("write request");
    }
    let out = child.wait_with_output().expect("wait for oracle");
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "oracle returned malformed JSON: {e}\n--- stdout ---\n{stdout}\n--- stderr ---\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// Convenience wrapper: panic if the oracle reports failure, else return the
/// JSON value of `result`. A `null` result is a valid value (returned as
/// `serde_json::Value::Null`) — algorithms like `girth` use it to encode
/// "no cycle" / "infinity".
pub fn run_ok<P: Serialize>(
    algo: &str,
    graph: &rust_igraph::Graph,
    params: P,
) -> serde_json::Value {
    run_ok_with_weights(algo, graph, None, params)
}

/// Weight-aware variant of [`run_ok`]. See [`run_with_weights`].
pub fn run_ok_with_weights<P: Serialize>(
    algo: &str,
    graph: &rust_igraph::Graph,
    weights: Option<Vec<f64>>,
    params: P,
) -> serde_json::Value {
    let resp = run_with_weights(algo, graph, weights, params);
    assert!(
        resp.ok,
        "oracle reported failure: {:?}",
        resp.error.unwrap_or_default()
    );
    // serde_json deserialises `result: null` as `Option<Value>::None`;
    // a missing `result` key would also be `None`. Treat both as JSON Null
    // so call sites can decode `Option<T>` directly.
    resp.result.unwrap_or(serde_json::Value::Null)
}
