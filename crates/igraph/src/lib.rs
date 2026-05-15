//! rust-igraph: pure-Rust port of the igraph network analysis library.
//!
//! Phase 0 walking-skeleton facade: re-exports core types and a couple of
//! algorithms so users (and the `bfs_karate` example) can `use igraph::*`.
//! The rich high-level API (`VertexClustering`, `Layout`, `Cut`, ...) lands
//! in Phase 10.

pub use igraph_algorithms::io::read_edgelist;
pub use igraph_algorithms::traversal::bfs;
pub use igraph_core::{Graph, IgraphError, IgraphResult, VertexId};
