//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence),
//! ALGO-CC-041 (Eulerian path/cycle construction, undirected), ALGO-SP-020
//! (eccentricity / radius / diameter).

pub mod dijkstra;
pub mod distances;
pub mod eulerian;
pub mod eulerian_construct;
pub mod radii;

pub use dijkstra::dijkstra_distances;
pub use distances::distances;
pub use eulerian::{EulerianClassification, is_eulerian};
pub use eulerian_construct::eulerian_path;
pub use radii::{diameter, eccentricity, radius};
