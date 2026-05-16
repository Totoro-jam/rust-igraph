//! Graph properties — invariants and metrics. Phase 1 entries:
//! ALGO-PR-001 (`girth`), ALGO-PR-002 (triangles + global/local
//! transitivity), ALGO-PR-003 (density + mean distance), ALGO-PR-004
//! (reciprocity), ALGO-PR-005 (avg nearest-neighbour degree).

pub mod basic;
pub mod girth;
pub mod knn;
pub mod reciprocity;
pub mod triangles;

pub use basic::{density, mean_distance};
pub use girth::girth;
pub use knn::avg_nearest_neighbor_degree;
pub use reciprocity::reciprocity;
pub use triangles::{count_triangles, transitivity_local_undirected, transitivity_undirected};
