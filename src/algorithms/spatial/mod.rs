//! Spatial / geometric algorithms (ALGO-GEO-*).

pub(crate) mod circle_beta_skeleton;
pub(crate) mod convex_hull;
pub(crate) mod edge_lengths;
pub(crate) mod gabriel_graph;
pub(crate) mod lune_beta_skeleton;
pub(crate) mod nearest_neighbor_graph;
pub(crate) mod relative_neighborhood_graph;

pub use circle_beta_skeleton::circle_beta_skeleton;
pub use convex_hull::{ConvexHullResult, convex_hull_2d};
pub use edge_lengths::{DistanceMetric, spatial_edge_lengths};
pub use gabriel_graph::gabriel_graph;
pub use lune_beta_skeleton::lune_beta_skeleton;
pub use nearest_neighbor_graph::nearest_neighbor_graph;
pub use relative_neighborhood_graph::relative_neighborhood_graph;
