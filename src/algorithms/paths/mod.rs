//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence).

pub mod distances;
pub mod eulerian;

pub use distances::distances;
pub use eulerian::{EulerianClassification, is_eulerian};
