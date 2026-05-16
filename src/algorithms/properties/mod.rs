//! Graph properties — invariants and metrics. Phase 1 entries:
//! ALGO-PR-001 (`girth`), ALGO-PR-002 (triangles + global/local
//! transitivity), ALGO-PR-003 (density + mean distance), ALGO-PR-004
//! (reciprocity).

pub mod basic;
pub mod girth;
pub mod reciprocity;
pub mod triangles;

pub use basic::{density, mean_distance};
pub use girth::girth;
pub use reciprocity::reciprocity;
pub use triangles::{count_triangles, transitivity_local_undirected, transitivity_undirected};
