//! Path-related algorithms. Phase 1 entries: ALGO-SP-006 (unweighted
//! single-source distances), ALGO-CC-040 (Eulerian path / cycle existence),
//! ALGO-CC-041 (Eulerian path/cycle construction, undirected), ALGO-SP-020
//! (eccentricity / radius / diameter).

// `pub(crate)` so the inner module names (`distances`, `eulerian`)
// don't double-list with the function re-exports in rustdoc.
pub(crate) mod all_shortest_paths;
pub(crate) mod astar;
pub(crate) mod bellman_ford;
pub(crate) mod dijkstra;
pub(crate) mod distances;
pub(crate) mod distances_all;
pub(crate) mod distances_cutoff;
pub(crate) mod distances_from;
pub(crate) mod edge_path_to_vertex_path;
pub(crate) mod eulerian;
pub(crate) mod eulerian_construct;
pub(crate) mod floyd_warshall;
pub(crate) mod get_shortest_path;
pub(crate) mod get_shortest_path_astar;
pub(crate) mod graph_center;
pub(crate) mod histogram;
pub(crate) mod johnson;
pub(crate) mod k_shortest_paths;
pub(crate) mod radii;
pub(crate) mod random_walk;
pub(crate) mod shortest_paths;
pub(crate) mod simple_paths;
pub(crate) mod spanner;
pub(crate) mod voronoi;
pub(crate) mod widest_path;

pub use all_shortest_paths::{
    AllShortestPaths, AllShortestPathsMode, get_all_shortest_paths,
    get_all_shortest_paths_with_mode,
};
pub use astar::a_star_path;
pub use dijkstra::{
    DijkstraAllPaths, DijkstraMode, DijkstraPaths, dijkstra_all_shortest_paths, dijkstra_distances,
    dijkstra_distances_cutoff, dijkstra_distances_cutoff_with_mode, dijkstra_distances_multi,
    dijkstra_distances_multi_with_mode, dijkstra_distances_with_mode, dijkstra_path_to,
    dijkstra_path_to_with_mode, dijkstra_paths, dijkstra_paths_with_mode,
};
pub use distances::distances;
pub use distances_all::{DistancesMode, distances_all, distances_all_with_mode};
pub use distances_cutoff::{
    DistancesCutoffMode, distances_cutoff, distances_cutoff_multi, distances_cutoff_with_mode,
};
pub use distances_from::{DistancesFromMode, distances_from, distances_from_with_mode};
pub use edge_path_to_vertex_path::{WalkMode, vertex_path_from_edge_path};
pub use eulerian::{EulerianClassification, is_eulerian};
pub use eulerian_construct::{eulerian_cycle, eulerian_path};
pub use floyd_warshall::floyd_warshall_distances;
pub use get_shortest_path::{ShortestPath, get_shortest_path};
pub use get_shortest_path_astar::{AstarHeuristic, get_shortest_path_astar};
pub use graph_center::{PseudoDiameterResult, graph_center, pseudo_diameter};
pub use histogram::{PathLengthHistResult, path_length_hist};
pub use k_shortest_paths::{KShortestPath, k_shortest_paths};
pub use radii::{
    diameter, diameter_weighted, diameter_weighted_with_mode, eccentricity, eccentricity_weighted,
    eccentricity_weighted_with_mode, radius, radius_weighted, radius_weighted_with_mode,
};
pub use random_walk::random_walk;
pub use shortest_paths::{ShortestPathMode, get_shortest_paths, get_shortest_paths_with_mode};
pub use simple_paths::{SimplePathMode, all_simple_paths};
pub use spanner::spanner;
pub use voronoi::{VoronoiPartition, VoronoiTiebreaker, voronoi};
