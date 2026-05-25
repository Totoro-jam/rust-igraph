//! # rust-igraph
//!
//! Pure-Rust port of the [igraph](https://igraph.org) network analysis
//! library. Targets full API parity with igraph C v1.0.x (~850 public
//! functions), validated continuously against the three official
//! implementations (igraph C, python-igraph, R-igraph).
//!
//! > **Status**: alpha — only the Phase 0 walking-skeleton API is shipped
//! > (`Graph`, `read_edgelist`, `bfs`). The catalog grows
//! > algorithm-by-algorithm through Phase 1-10. See the project's
//! > [master plan](https://github.com/Totoro-jam/rust-igraph/blob/main/docs/plans/MASTER_PLAN.md).
//!
//! ## Quick start
//!
//! ```
//! use rust_igraph::{Graph, bfs};
//!
//! let mut g = Graph::with_vertices(4);
//! g.add_edge(0, 1).unwrap();
//! g.add_edge(0, 2).unwrap();
//! g.add_edge(1, 3).unwrap();
//!
//! let order = bfs(&g, 0).unwrap();
//! assert_eq!(order, vec![0, 1, 2, 3]);
//! ```
//!
//! ## License
//!
//! GPL-2.0-or-later, matching upstream igraph.

pub mod algorithms;
pub mod core;

// Top-level re-exports for the common case.
// IMPORTANT: when a function name collides with the file/module it
// lives in (e.g. `is_simple` in `is_simple.rs`), `pub use
// crate::algorithms::properties::is_simple` resolves ambiguously to
// **both** the module and the function, and rustdoc renders the
// re-export twice on the crate-root page. To suppress that, every
// re-export below points at the inner module path explicitly so the
// resolution is unambiguously the function. The submodules
// themselves are `pub(crate)` (see each algorithm-group `mod.rs`),
// so the deep paths stay crate-internal at the type-checker level
// while rustdoc only renders the function entry.
pub use crate::algorithms::community::community_to_membership::{
    CommunityToMembershipResult, community_to_membership,
};
pub use crate::algorithms::community::community_voronoi::{
    CommunityVoronoiResult, community_voronoi,
};
pub use crate::algorithms::community::compare_communities::{
    CommunityComparison, compare_communities,
};
pub use crate::algorithms::community::edge_betweenness_community::{
    EdgeBetweennessResult, edge_betweenness_community,
};
pub use crate::algorithms::community::edge_betweenness_community_weighted::edge_betweenness_community_weighted;
pub use crate::algorithms::community::fast_greedy_modularity::{
    FastGreedyResult, fast_greedy_modularity, fast_greedy_modularity_weighted,
};
pub use crate::algorithms::community::fluid_communities::{
    FLUID_DEFAULT_MAX_ITERATIONS, FluidOptions, FluidResult, fluid_communities,
    fluid_communities_with_options,
};
pub use crate::algorithms::community::label_propagation::{
    LpaOptions, LpaResult, LpaVariant, label_propagation, label_propagation_weighted,
    label_propagation_with_options,
};
pub use crate::algorithms::community::leiden::{
    LEIDEN_DEFAULT_BETA, LEIDEN_DEFAULT_ITERATIONS, LeidenObjective, LeidenOptions, LeidenResult,
    leiden, leiden_weighted, leiden_with_options,
};
pub use crate::algorithms::community::louvain::{
    LouvainResult, louvain, louvain_weighted, louvain_with_options,
};
pub use crate::algorithms::community::modularity::{
    modularity, modularity_directed, modularity_weighted, modularity_weighted_directed,
};
pub use crate::algorithms::community::reindex_membership::{
    ReindexMembershipResult, reindex_membership,
};
pub use crate::algorithms::community::split_join_distance::{
    SplitJoinDistance, split_join_distance,
};
pub use crate::algorithms::community::walktrap::{
    WALKTRAP_DEFAULT_STEPS, WalktrapOptions, WalktrapResult, walktrap, walktrap_weighted,
    walktrap_with_options,
};
pub use crate::algorithms::connectivity::articulation::articulation_points;
pub use crate::algorithms::connectivity::biconnected::{
    BiconnectedComponents, biconnected_components,
};
pub use crate::algorithms::connectivity::bridges::bridges;
pub use crate::algorithms::connectivity::components::{ConnectedComponents, connected_components};
pub use crate::algorithms::connectivity::decompose::decompose;
pub use crate::algorithms::connectivity::is_biconnected::is_biconnected;
pub use crate::algorithms::connectivity::percolation::{
    EdgelistPercolation, SitePercolation, bond_percolation, edgelist_percolation, site_percolation,
};
pub use crate::algorithms::connectivity::reachability::count_reachable;
pub use crate::algorithms::connectivity::reachability_matrix::reachability_matrix;
pub use crate::algorithms::connectivity::strong::strongly_connected_components;
pub use crate::algorithms::connectivity::transitive_closure::transitive_closure;
pub use crate::algorithms::constructors::atlas::{ATLAS_SIZE, atlas};
pub use crate::algorithms::constructors::circulant::circulant;
pub use crate::algorithms::constructors::create::create;
pub use crate::algorithms::constructors::de_bruijn::de_bruijn;
pub use crate::algorithms::constructors::famous::{famous, famous_names};
pub use crate::algorithms::constructors::full::full_graph;
pub use crate::algorithms::constructors::full_citation::full_citation;
pub use crate::algorithms::constructors::full_multipartite::{
    FullMultipartite, MultipartiteMode, full_multipartite,
};
pub use crate::algorithms::constructors::generalized_petersen::generalized_petersen;
pub use crate::algorithms::constructors::hamming::hamming;
pub use crate::algorithms::constructors::hexagonal_lattice::hexagonal_lattice;
pub use crate::algorithms::constructors::hypercube::{MAX_HYPERCUBE_DIMENSION, hypercube};
pub use crate::algorithms::constructors::kary_tree::{TreeMode, kary_tree};
pub use crate::algorithms::constructors::kautz::kautz;
pub use crate::algorithms::constructors::lcf::lcf;
pub use crate::algorithms::constructors::linegraph::linegraph;
pub use crate::algorithms::constructors::mycielskian::{mycielski_graph, mycielskian};
pub use crate::algorithms::constructors::prufer::from_prufer;
pub use crate::algorithms::constructors::regular_tree::regular_tree;
pub use crate::algorithms::constructors::ring::{cycle_graph, path_graph, ring_graph};
pub use crate::algorithms::constructors::square_lattice::square_lattice;
pub use crate::algorithms::constructors::star::{StarMode, star_graph};
pub use crate::algorithms::constructors::symmetric_tree::symmetric_tree;
pub use crate::algorithms::constructors::tree_from_parent_vector::tree_from_parent_vector;
pub use crate::algorithms::constructors::triangular_lattice::triangular_lattice;
pub use crate::algorithms::constructors::turan::turan;
pub use crate::algorithms::constructors::wheel::{WheelMode, wheel_graph};
pub use crate::algorithms::games::barabasi::barabasi_game_bag;
pub use crate::algorithms::games::barabasi_aging::barabasi_aging_game;
pub use crate::algorithms::games::barabasi_psumtree::{
    barabasi_game_psumtree, barabasi_game_psumtree_multiple,
};
pub use crate::algorithms::games::callaway_traits::callaway_traits_game;
pub use crate::algorithms::games::chung_lu::{ChungLuVariant, chung_lu_game};
pub use crate::algorithms::games::cited_type::cited_type_game;
pub use crate::algorithms::games::correlated::{correlated_game, correlated_pair_game};
pub use crate::algorithms::games::degree_sequence::degree_sequence_game_configuration;
pub use crate::algorithms::games::degree_sequence_configuration_simple::degree_sequence_game_configuration_simple;
pub use crate::algorithms::games::degree_sequence_fast_heur::degree_sequence_game_fast_heur_simple;
pub use crate::algorithms::games::degree_sequence_vl::degree_sequence_game_vl;
pub use crate::algorithms::games::dotproduct::{
    DotProductWarnings, dot_product_game, dot_product_game_with_warnings,
};
pub use crate::algorithms::games::edge_switching_simple::degree_sequence_game_edge_switching_simple;
pub use crate::algorithms::games::erdos_renyi::{erdos_renyi_gnm, erdos_renyi_gnp};
pub use crate::algorithms::games::establishment::establishment_game;
pub use crate::algorithms::games::forestfire::forest_fire_game;
pub use crate::algorithms::games::grg::{grg_game, grg_game_with_coords};
pub use crate::algorithms::games::growing_random::growing_random_game;
pub use crate::algorithms::games::hsbm::{hsbm_game, hsbm_list_game};
pub use crate::algorithms::games::islands::simple_interconnected_islands_game;
pub use crate::algorithms::games::k_regular::k_regular_game;
pub use crate::algorithms::games::lastcit::lastcit_game;
pub use crate::algorithms::games::preference::{asymmetric_preference_game, preference_game};
pub use crate::algorithms::games::recent_degree::recent_degree_game;
pub use crate::algorithms::games::sbm::sbm_game;
pub use crate::algorithms::games::static_fitness::{static_fitness_game, static_power_law_game};
pub use crate::algorithms::games::tree::tree_game_lerw;
pub use crate::algorithms::games::watts::watts_strogatz_game;
pub use crate::algorithms::io::edgelist::read_edgelist;
pub use crate::algorithms::operators::complementer::complementer;
pub use crate::algorithms::operators::difference::difference;
pub use crate::algorithms::operators::disjoint_union::{disjoint_union, disjoint_union_many};
pub use crate::algorithms::operators::intersection::intersection;
pub use crate::algorithms::operators::is_same_graph::is_same_graph;
pub use crate::algorithms::operators::simplify::simplify;
pub use crate::algorithms::operators::union::union;
pub use crate::algorithms::paths::astar::a_star_path;
pub use crate::algorithms::paths::bellman_ford::{
    bellman_ford_distances, bellman_ford_distances_with_mode,
};
pub use crate::algorithms::paths::dijkstra::{
    DijkstraAllPaths, DijkstraMode, DijkstraPaths, dijkstra_all_shortest_paths, dijkstra_distances,
    dijkstra_distances_cutoff, dijkstra_distances_cutoff_with_mode, dijkstra_distances_multi,
    dijkstra_distances_multi_with_mode, dijkstra_distances_with_mode, dijkstra_path_to,
    dijkstra_path_to_with_mode, dijkstra_paths, dijkstra_paths_with_mode,
};
pub use crate::algorithms::paths::distances::distances;
pub use crate::algorithms::paths::eulerian::{EulerianClassification, is_eulerian};
pub use crate::algorithms::paths::eulerian_construct::eulerian_path;
pub use crate::algorithms::paths::floyd_warshall::floyd_warshall_distances;
pub use crate::algorithms::paths::johnson::johnson_distances;
pub use crate::algorithms::paths::radii::{
    EccMode, diameter, diameter_weighted, diameter_weighted_with_mode, diameter_with_mode,
    eccentricity, eccentricity_weighted, eccentricity_weighted_with_mode, eccentricity_with_mode,
    radius, radius_weighted, radius_weighted_with_mode, radius_with_mode,
};
pub use crate::algorithms::paths::random_walk::random_walk;
pub use crate::algorithms::paths::voronoi::{VoronoiPartition, VoronoiTiebreaker, voronoi};
pub use crate::algorithms::paths::widest_path::{
    WidestPathResult, WidestPaths, widest_path, widest_path_widths,
    widest_path_widths_floyd_warshall, widest_path_widths_floyd_warshall_with_mode,
    widest_path_widths_with_mode, widest_path_with_mode, widest_paths, widest_paths_to,
    widest_paths_to_with_mode, widest_paths_with_mode,
};
pub use crate::algorithms::properties::assortativity::{
    assortativity_degree, assortativity_degree_directed,
};
pub use crate::algorithms::properties::assortativity_weighted::{
    assortativity_degree_directed_weighted, assortativity_degree_weighted,
};
pub use crate::algorithms::properties::basic::{density, mean_distance};
pub use crate::algorithms::properties::betweenness::betweenness;
pub use crate::algorithms::properties::betweenness_weighted::betweenness_weighted;
pub use crate::algorithms::properties::closeness::closeness;
pub use crate::algorithms::properties::closeness_weighted::closeness_weighted;
pub use crate::algorithms::properties::convergence_degree::{
    convergence_degree, convergence_degree_full,
};
pub use crate::algorithms::properties::coreness::{CorenessMode, coreness, coreness_with_mode};
pub use crate::algorithms::properties::ecc::ecc;
pub use crate::algorithms::properties::edge_betweenness::edge_betweenness;
pub use crate::algorithms::properties::edge_betweenness_weighted::edge_betweenness_weighted;
pub use crate::algorithms::properties::efficiency::{
    average_local_efficiency, global_efficiency, local_efficiency,
};
pub use crate::algorithms::properties::eigenvector::{
    EigenvectorMode, EigenvectorScores, eigenvector_centrality, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_full,
    eigenvector_centrality_weighted,
};
pub use crate::algorithms::properties::girth::girth;
pub use crate::algorithms::properties::harmonic::harmonic_centrality;
pub use crate::algorithms::properties::harmonic_weighted::harmonic_centrality_weighted;
pub use crate::algorithms::properties::hits::{
    HitsScores, hub_and_authority_scores, hub_and_authority_scores_weighted,
};
pub use crate::algorithms::properties::is_acyclic::is_acyclic;
pub use crate::algorithms::properties::is_complete::is_complete;
pub use crate::algorithms::properties::is_dag::is_dag;
pub use crate::algorithms::properties::is_forest::is_forest;
pub use crate::algorithms::properties::is_simple::{SimpleMode, is_simple, is_simple_with_mode};
pub use crate::algorithms::properties::is_tree::is_tree;
pub use crate::algorithms::properties::knn::{
    avg_nearest_neighbor_degree, avg_nearest_neighbor_degree_weighted, knnk, knnk_weighted,
};
pub use crate::algorithms::properties::multiplicity::{
    count_loops, count_multiple, has_loop, has_multiple, is_loop, is_multiple,
};
pub use crate::algorithms::properties::neighborhood::{
    NeighborhoodMode, neighborhood, neighborhood_size, neighborhood_size_with_mode,
    neighborhood_with_mode,
};
pub use crate::algorithms::properties::pagerank::pagerank;
pub use crate::algorithms::properties::pagerank_weighted::pagerank_weighted;
pub use crate::algorithms::properties::reciprocity::{
    ReciprocityMode, reciprocity, reciprocity_with_mode,
};
pub use crate::algorithms::properties::topological_sorting::topological_sorting;
pub use crate::algorithms::properties::triangles::{
    count_adjacent_triangles, count_triangles, transitivity_barrat, transitivity_local_undirected,
    transitivity_undirected,
};
pub use crate::algorithms::spanning::mst::{MstAlgorithm, minimum_spanning_tree};
pub use crate::algorithms::traversal::bfs::{BfsTree, bfs, bfs_tree};
pub use crate::algorithms::traversal::dfs::dfs;
pub use crate::core::error::{IgraphError, IgraphResult};
pub use crate::core::graph::{Graph, VertexId};
