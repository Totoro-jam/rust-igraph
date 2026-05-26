//! Network-flow algorithms (ALGO-FL-*).
//!
//! First entry: [`max_flow_value`] (`ALGO-FL-002`) — scalar
//! maximum-flow value via Dinic's algorithm (BFS level graph + DFS
//! blocking flow). Mirrors igraph C `igraph_maxflow_value` from
//! `references/igraph/src/flow/flow.c`, which delegates to the
//! Goldberg-Tarjan push-relabel `igraph_maxflow`. The scalar max-flow
//! value is unique (max-flow / min-cut theorem) regardless of which
//! algorithm produced it, so the two backends agree bit-for-bit on
//! unit-capacity fixtures and within numerical tolerance on weighted
//! ones.

pub(crate) mod edge_disjoint_paths;
pub(crate) mod max_flow;
pub(crate) mod st_edge_connectivity;
pub(crate) mod st_mincut;
pub(crate) mod st_vertex_connectivity;

pub use edge_disjoint_paths::edge_disjoint_paths;
pub use max_flow::max_flow_value;
pub use st_edge_connectivity::st_edge_connectivity;
pub use st_mincut::st_mincut_value;
pub use st_vertex_connectivity::{VconnNei, st_vertex_connectivity};
