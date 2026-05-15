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
    pub fn from_graph(g: &igraph_core::Graph) -> Self {
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for u in 0..g.vcount() {
            for &v in g.neighbors(u).expect("vertex in range") {
                if u <= v {
                    edges.push((u, v));
                }
            }
        }
        Self {
            n: g.vcount(),
            edges,
            directed: false,
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
/// Looks one level up from `igraph` crate's manifest dir to the workspace
/// root, then `.venv/bin/python`. Tests panic with an actionable message if
/// the venv is missing.
pub fn venv_python() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("workspace root above crates/igraph")
            .to_path_buf();
        let p = workspace_root.join(".venv/bin/python");
        if !p.exists() {
            panic!(
                "Python venv not found at {}. Run:\n  python3 -m venv .venv\n  .venv/bin/pip install -r scripts/requirements.txt",
                p.display()
            );
        }
        p
    })
    .as_path()
}

/// Resolve the path to `scripts/oracle.py`.
pub fn oracle_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root above crates/igraph")
        .join("scripts/oracle.py")
}

/// Call the oracle and return the parsed response. Panics if the subprocess
/// exits abnormally or returns malformed JSON; tests should use [`run_ok`]
/// for the common success path.
pub fn run<P: Serialize>(algo: &str, graph: &igraph_core::Graph, params: P) -> OracleResponse {
    let req = OracleRequest {
        graph: GraphPayload::from_graph(graph),
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
/// JSON value of `result`.
pub fn run_ok<P: Serialize>(
    algo: &str,
    graph: &igraph_core::Graph,
    params: P,
) -> serde_json::Value {
    let resp = run(algo, graph, params);
    assert!(
        resp.ok,
        "oracle reported failure: {:?}",
        resp.error.unwrap_or_default()
    );
    resp.result.expect("oracle ok response missing result")
}
