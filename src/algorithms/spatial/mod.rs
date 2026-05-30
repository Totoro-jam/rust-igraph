//! Spatial / geometric algorithms (ALGO-GEO-*).

pub(crate) mod convex_hull;
pub(crate) mod edge_lengths;
pub(crate) mod gabriel_graph;
pub(crate) mod nearest_neighbor_graph;
pub(crate) mod relative_neighborhood_graph;

pub use convex_hull::{ConvexHullResult, convex_hull_2d};
pub use edge_lengths::{DistanceMetric, spatial_edge_lengths};
pub use gabriel_graph::gabriel_graph;
pub use nearest_neighbor_graph::nearest_neighbor_graph;
pub use relative_neighborhood_graph::relative_neighborhood_graph;
