//! TEMPLATE: Step-3 skeleton for one Algorithm Work Unit (AWU).
//!
//! Copy this file into `src/algorithms/<group>/<name>.rs`,
//! replace every `{{...}}` placeholder, and leave the body as
//! `unimplemented!()`. The `awu-translate` skill takes over from here.
//!
//! Placeholders:
//!   {{ALGO_ID}}         e.g. ALGO-CT-002
//!   {{ALGO_TITLE}}      e.g. "Betweenness centrality"
//!   {{C_PATH}}           e.g. centrality_betweenness.c
//!   {{C_LINES}}          e.g. 1404
//!   {{FN_NAME}}         e.g. betweenness
//!   {{RETURN_TYPE}}     e.g. Vec<f64>
//!   {{REFERENCE}}       optional paper / docs link
//!
//! Delete this header block once the file is real (do not commit the template
//! comment as live documentation).

use rust_igraph::{Graph, IgraphError, IgraphResult};

/// {{ALGO_TITLE}}.
///
/// Counterpart of `igraph_{{FN_NAME}}` — see
/// `references/igraph/src/{{C_PATH}}` ({{C_LINES}} lines).
///
/// # Arguments
/// * `graph` - input graph
/// // TODO({{ALGO_ID}}): document each parameter (units, ranges, defaults)
///
/// # Returns
/// // TODO({{ALGO_ID}}): describe shape and ordering guarantees
///
/// # Errors
/// // TODO({{ALGO_ID}}): list every IgraphError variant returned and the
/// // condition that triggers it
///
/// # Examples
/// ```
/// // TODO({{ALGO_ID}}): minimal runnable doctest using public API only
/// ```
///
/// # References
/// // TODO({{ALGO_ID}}): paper / igraph docs / {{REFERENCE}}
pub fn {{FN_NAME}}(graph: &Graph /* TODO({{ALGO_ID}}): params */) -> IgraphResult<{{RETURN_TYPE}}> {
    let _ = graph;
    unimplemented!("{{ALGO_ID}} not yet translated; see references/igraph/src/{{C_PATH}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO({{ALGO_ID}}): unit tests follow the awu-tester contract:
    //   empty / single / complete K5 / directed-vs-undirected / error path
    // The awu-tester skill drops them here.

    #[test]
    fn skeleton_compiles() {
        // Trivial smoke — replaced once the real implementation lands.
        let g = Graph::with_vertices(0);
        let _ = std::panic::catch_unwind(|| {
            let _ = {{FN_NAME}}(&g /* TODO({{ALGO_ID}}): params */);
        });
    }
}
