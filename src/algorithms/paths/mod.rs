//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence),
//! ALGO-CC-041 (Eulerian path/cycle construction, undirected), ALGO-SP-020
//! (eccentricity / radius / diameter).

// `pub(crate)` so the inner module names (`distances`, `eulerian`)
// don't double-list with the function re-exports in rustdoc.
pub(crate) mod astar;
pub(crate) mod dijkstra;
pub(crate) mod distances;
pub(crate) mod eulerian;
pub(crate) mod eulerian_construct;
pub(crate) mod floyd_warshall;
pub(crate) mod radii;
pub(crate) mod random_walk;

pub use astar::a_star_path;
pub use dijkstra::{
    DijkstraAllPaths, DijkstraMode, DijkstraPaths, dijkstra_all_shortest_paths, dijkstra_distances,
    dijkstra_distances_cutoff, dijkstra_distances_cutoff_with_mode, dijkstra_distances_multi,
    dijkstra_distances_multi_with_mode, dijkstra_distances_with_mode, dijkstra_path_to,
    dijkstra_path_to_with_mode, dijkstra_paths, dijkstra_paths_with_mode,
};
pub use distances::distances;
pub use eulerian::{EulerianClassification, is_eulerian};
pub use eulerian_construct::eulerian_path;
pub use floyd_warshall::floyd_warshall_distances;
pub use radii::{
    diameter, diameter_weighted, diameter_weighted_with_mode, eccentricity, eccentricity_weighted,
    eccentricity_weighted_with_mode, radius, radius_weighted, radius_weighted_with_mode,
};
pub use random_walk::random_walk;
