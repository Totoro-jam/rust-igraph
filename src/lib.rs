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
pub use crate::algorithms::community::modularity;
pub use crate::algorithms::connectivity::{
    BiconnectedComponents, ConnectedComponents, articulation_points, biconnected_components,
    bridges, connected_components, count_reachable, is_biconnected, reachability_matrix,
    strongly_connected_components, transitive_closure,
};
pub use crate::algorithms::io::read_edgelist;
pub use crate::algorithms::operators::{disjoint_union, simplify};
pub use crate::algorithms::paths::{
    EulerianClassification, diameter, dijkstra_distances, distances, eccentricity, eulerian_path,
    is_eulerian, radius,
};
pub use crate::algorithms::properties::{
    assortativity_degree, avg_nearest_neighbor_degree, betweenness, closeness, count_triangles,
    density, edge_betweenness, eigenvector_centrality, girth, harmonic_centrality, has_loop,
    has_multiple, is_loop, is_multiple, is_simple, mean_distance, pagerank, reciprocity,
    transitivity_local_undirected, transitivity_undirected,
};
pub use crate::algorithms::traversal::{BfsTree, bfs, bfs_tree, dfs};
pub use crate::core::error::{IgraphError, IgraphResult};
pub use crate::core::graph::{Graph, VertexId};
