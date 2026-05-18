//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence),
//! ALGO-CC-041 (Eulerian path/cycle construction, undirected), ALGO-SP-020
//! (eccentricity / radius / diameter).

// `pub(crate)` so the inner module names (`distances`, `eulerian`)
// don't double-list with the function re-exports in rustdoc.
pub(crate) mod dijkstra;
pub(crate) mod distances;
pub(crate) mod eulerian;
pub(crate) mod eulerian_construct;
pub(crate) mod floyd_warshall;
pub(crate) mod radii;

pub use dijkstra::dijkstra_distances;
pub use distances::distances;
pub use eulerian::{EulerianClassification, is_eulerian};
pub use eulerian_construct::eulerian_path;
pub use floyd_warshall::floyd_warshall_distances;
pub use radii::{diameter, eccentricity, radius};
