//! # rust-igraph
//!
//! Pure-Rust port of the [igraph](https://igraph.org) network analysis
//! library. Targets full API parity with igraph C v1.0.x (~850 public
//! functions), validated continuously against the three official
//! implementations (igraph C, python-igraph, R-igraph).
//!
//! > **Status**: alpha — only the Phase 0 walking-skeleton API is shipped
//! > (`Graph`, `read_edgelist`, `bfs`). The catalog grows
//! > algorithm-by-algorithm through Phase 1-10. See the project's
//! > [master plan](https://github.com/Totoro-jam/rust-igraph/blob/main/docs/plans/MASTER_PLAN.md).
//!
//! ## Quick start
//!
//! ```
//! use rust_igraph::{Graph, bfs};
//!
//! let mut g = Graph::with_vertices(4);
//! g.add_edge(0, 1).unwrap();
//! g.add_edge(0, 2).unwrap();
//! g.add_edge(1, 3).unwrap();
//!
//! let order = bfs(&g, 0).unwrap();
//! assert_eq!(order, vec![0, 1, 2, 3]);
//! ```
//!
//! ## License
//!
//! GPL-2.0-or-later, matching upstream igraph.

pub mod algorithms;
pub mod core;

// Top-level re-exports for the common case.
pub use crate::algorithms::connectivity::{
    ConnectedComponents, connected_components, strongly_connected_components,
};
pub use crate::algorithms::io::read_edgelist;
pub use crate::algorithms::paths::distances;
pub use crate::algorithms::traversal::{bfs, dfs};
pub use crate::core::error::{IgraphError, IgraphResult};
pub use crate::core::graph::{Graph, VertexId};
