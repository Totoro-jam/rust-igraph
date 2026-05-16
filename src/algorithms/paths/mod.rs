//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence),
//! ALGO-SP-020 (eccentricity / radius / diameter).

pub mod distances;
pub mod eulerian;
pub mod radii;

pub use distances::distances;
pub use eulerian::{EulerianClassification, is_eulerian};
pub use radii::{diameter, eccentricity, radius};
