//! Minimal import set for common use cases.
//!
//! ```
//! use rust_igraph::prelude::*;
//! ```
//!
//! This re-exports only the essential types and the most frequently used
//! algorithms. For the full API, import directly from the crate root:
//!
//! ```rust,no_run
//! use rust_igraph::{Graph, louvain, pagerank, betweenness};
//! ```

pub use crate::core::builder::GraphBuilder;
pub use crate::core::error::{IgraphError, IgraphResult};
pub use crate::core::graph::{EdgeIter, Graph, VertexId};

// Traversal
pub use crate::algorithms::traversal::bfs::{bfs, bfs_simple, bfs_tree};
pub use crate::algorithms::traversal::dfs::{dfs, dfs_tree};

// Shortest paths
pub use crate::algorithms::paths::dijkstra::{dijkstra_distances, dijkstra_paths};
pub use crate::algorithms::paths::radii::{diameter, eccentricity, radius};
pub use crate::algorithms::paths::shortest_paths::get_shortest_paths;

// Centrality
pub use crate::algorithms::properties::betweenness::betweenness;
pub use crate::algorithms::properties::closeness::closeness;
pub use crate::algorithms::properties::pagerank::pagerank;

// Community detection
pub use crate::algorithms::community::leiden::leiden;
pub use crate::algorithms::community::louvain::louvain;

// Connectivity
pub use crate::algorithms::connectivity::articulation::articulation_points;
pub use crate::algorithms::connectivity::bridges::bridges;
pub use crate::algorithms::connectivity::components::connected_components;
pub use crate::algorithms::connectivity::is_connected::{ConnectednessMode, is_connected};

// Properties
pub use crate::algorithms::properties::basic::density;
pub use crate::algorithms::properties::degree::{DegreeMode, degree_sequence};
pub use crate::algorithms::properties::is_bipartite::{BipartiteResult, is_bipartite};
pub use crate::algorithms::properties::is_simple::is_simple;
pub use crate::algorithms::properties::summary::{GraphSummary, graph_summary};
pub use crate::algorithms::properties::triangles::transitivity_undirected;

// Cliques
pub use crate::algorithms::cliques::clique_number;

// Generators
pub use crate::algorithms::constructors::famous::famous;
pub use crate::algorithms::constructors::full::full_graph;
