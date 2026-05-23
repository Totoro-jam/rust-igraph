//! Core data structures for rust-igraph.
//!
//! Phase 0 walking-skeleton scope: just enough types so `algorithms::bfs`,
//! `algorithms::io::read_edgelist`, and the oracle test can land. The full
//! `igraph_t`-equivalent structure replaces this during Phase 1
//! (see `docs/plans/MASTER_PLAN.md`).

pub mod cache;
pub mod error;
pub mod graph;
pub mod rng;

pub use cache::CachedProperty;
pub use error::{IgraphError, IgraphResult};
pub use graph::{Graph, VertexId};
