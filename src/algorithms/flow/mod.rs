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

pub(crate) mod max_flow;

pub use max_flow::max_flow_value;
