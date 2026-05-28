//! Spatial / geometric algorithms (ALGO-GEO-*).

pub(crate) mod convex_hull;
pub(crate) mod edge_lengths;

pub use convex_hull::{ConvexHullResult, convex_hull_2d};
pub use edge_lengths::{DistanceMetric, spatial_edge_lengths};
