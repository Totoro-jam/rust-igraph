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

pub(crate) mod all_st_cuts;
pub(crate) mod all_st_mincuts;
pub(crate) mod dominator_tree;
pub(crate) mod edge_connectivity;
pub(crate) mod edge_disjoint_paths;
pub(crate) mod gomory_hu_tree;
pub(crate) mod max_flow;
pub(crate) mod mincut;
pub(crate) mod mincut_value;
pub(crate) mod provan_shier;
pub(crate) mod st_edge_connectivity;
pub(crate) mod st_mincut;
pub(crate) mod st_vertex_connectivity;
pub(crate) mod vertex_connectivity;
pub(crate) mod vertex_disjoint_paths;

pub use all_st_cuts::{StCuts, all_st_cuts};
pub use all_st_mincuts::{StMinCuts, all_st_mincuts};
pub use dominator_tree::{DominatorMode, DominatorTree, dominator_tree};
pub use edge_connectivity::{adhesion, edge_connectivity};
pub use edge_disjoint_paths::edge_disjoint_paths;
pub use gomory_hu_tree::{GomoryHuTree, gomory_hu_tree};
pub use max_flow::{MaxFlow, max_flow, max_flow_value};
pub use mincut::{Mincut, mincut};
pub use mincut_value::mincut_value;
pub use st_edge_connectivity::st_edge_connectivity;
pub use st_mincut::{StMincut, st_mincut, st_mincut_value};
pub use st_vertex_connectivity::{VconnNei, st_vertex_connectivity};
pub use vertex_connectivity::{cohesion, vertex_connectivity};
pub use vertex_disjoint_paths::vertex_disjoint_paths;
