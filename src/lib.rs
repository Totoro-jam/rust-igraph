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
pub use crate::algorithms::chordality::{
    ChordalResult, McsResult, is_chordal, maximum_cardinality_search,
};
pub use crate::algorithms::cliques::{
    clique_number, clique_size_hist, cliques, independence_number, independent_vertex_sets,
    largest_cliques, largest_independent_vertex_sets, largest_weighted_cliques, maximal_cliques,
    maximal_cliques_count, maximal_cliques_hist, maximal_independent_vertex_sets,
    weighted_clique_number, weighted_cliques,
};
pub use crate::algorithms::coloring::{
    BipartiteColoringResult, BipartiteEdgeDirection, GreedyColoringHeuristic,
    is_bipartite_coloring, is_edge_coloring, is_vertex_coloring, vertex_coloring_greedy,
};
pub use crate::algorithms::community::community_to_membership::{
    CommunityToMembershipResult, community_to_membership, le_community_to_membership,
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
pub use crate::algorithms::community::modularity_matrix::modularity_matrix;
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
pub use crate::algorithms::connectivity::cohesive_blocks::{CohesiveBlocks, cohesive_blocks};
pub use crate::algorithms::connectivity::components::{ConnectedComponents, connected_components};
pub use crate::algorithms::connectivity::decompose::decompose;
pub use crate::algorithms::connectivity::is_biconnected::is_biconnected;
pub use crate::algorithms::connectivity::is_connected::{ConnectednessMode, is_connected};
pub use crate::algorithms::connectivity::percolation::{
    EdgelistPercolation, SitePercolation, bond_percolation, edgelist_percolation, site_percolation,
};
pub use crate::algorithms::connectivity::reachability::count_reachable;
pub use crate::algorithms::connectivity::reachability_matrix::reachability_matrix;
pub use crate::algorithms::connectivity::separators::{
    all_minimal_st_separators, is_minimal_separator, is_separator, minimum_size_separators,
};
pub use crate::algorithms::connectivity::strong::strongly_connected_components;
pub use crate::algorithms::connectivity::subcomponent::{SubcomponentMode, subcomponent};
pub use crate::algorithms::connectivity::transitive_closure::transitive_closure;
pub use crate::algorithms::constructors::adjacency::{AdjacencyMode, LoopsMode, adjacency};
pub use crate::algorithms::constructors::atlas::{ATLAS_SIZE, atlas};
pub use crate::algorithms::constructors::biadjacency::{BiadjacencyResult, biadjacency};
pub use crate::algorithms::constructors::circulant::circulant;
pub use crate::algorithms::constructors::create::create;
pub use crate::algorithms::constructors::create_bipartite::create_bipartite;
pub use crate::algorithms::constructors::de_bruijn::de_bruijn;
pub use crate::algorithms::constructors::extended_chordal_ring::extended_chordal_ring;
pub use crate::algorithms::constructors::famous::{famous, famous_names};
pub use crate::algorithms::constructors::full::full_graph;
pub use crate::algorithms::constructors::full_bipartite::{FullBipartite, full_bipartite};
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
pub use crate::algorithms::constructors::prufer::{from_prufer, to_prufer};
pub use crate::algorithms::constructors::realize_bipartite_degree_sequence::realize_bipartite_degree_sequence;
pub use crate::algorithms::constructors::realize_degree_sequence::{
    RealizeDegseqMethod, realize_degree_sequence,
};
pub use crate::algorithms::constructors::regular_tree::regular_tree;
pub use crate::algorithms::constructors::ring::{cycle_graph, path_graph, ring_graph};
pub use crate::algorithms::constructors::square_lattice::square_lattice;
pub use crate::algorithms::constructors::star::{StarMode, star_graph};
pub use crate::algorithms::constructors::symmetric_tree::symmetric_tree;
pub use crate::algorithms::constructors::tree_from_parent_vector::tree_from_parent_vector;
pub use crate::algorithms::constructors::triangular_lattice::triangular_lattice;
pub use crate::algorithms::constructors::turan::turan;
pub use crate::algorithms::constructors::weighted_adjacency::weighted_adjacency;
pub use crate::algorithms::constructors::weighted_biadjacency::{
    WeightedBiadjacencyResult, weighted_biadjacency,
};
pub use crate::algorithms::constructors::wheel::{WheelMode, wheel_graph};
pub use crate::algorithms::cycles::{CycleMode, CycleResult, find_cycle};
pub use crate::algorithms::embedding::dim_select::dim_select;
pub use crate::algorithms::epidemics::{Sir, sir};
pub use crate::algorithms::feedback_arc_set::{FasAlgorithm, feedback_arc_set};
pub use crate::algorithms::flow::all_st_cuts::{StCuts, all_st_cuts};
pub use crate::algorithms::flow::all_st_mincuts::{StMinCuts, all_st_mincuts};
pub use crate::algorithms::flow::dominator_tree::{DominatorMode, DominatorTree, dominator_tree};
pub use crate::algorithms::flow::edge_connectivity::{adhesion, edge_connectivity};
pub use crate::algorithms::flow::edge_disjoint_paths::edge_disjoint_paths;
pub use crate::algorithms::flow::gomory_hu_tree::{GomoryHuTree, gomory_hu_tree};
pub use crate::algorithms::flow::max_flow::{MaxFlow, max_flow, max_flow_value};
pub use crate::algorithms::flow::mincut::{Mincut, mincut};
pub use crate::algorithms::flow::mincut_value::mincut_value;
pub use crate::algorithms::flow::st_edge_connectivity::st_edge_connectivity;
pub use crate::algorithms::flow::st_mincut::{StMincut, st_mincut, st_mincut_value};
pub use crate::algorithms::flow::st_vertex_connectivity::{VconnNei, st_vertex_connectivity};
pub use crate::algorithms::flow::vertex_connectivity::{cohesion, vertex_connectivity};
pub use crate::algorithms::flow::vertex_disjoint_paths::vertex_disjoint_paths;
pub use crate::algorithms::fundamental_cycles::fundamental_cycles;
pub use crate::algorithms::games::barabasi::barabasi_game_bag;
pub use crate::algorithms::games::barabasi_aging::barabasi_aging_game;
pub use crate::algorithms::games::barabasi_psumtree::{
    barabasi_game_psumtree, barabasi_game_psumtree_multiple,
};
pub use crate::algorithms::games::bipartite::{
    BipartiteGraph, BipartiteMode, bipartite_game_gnm, bipartite_game_gnp, bipartite_iea_game,
};
pub use crate::algorithms::games::callaway_traits::callaway_traits_game;
pub use crate::algorithms::games::chung_lu::{ChungLuVariant, chung_lu_game};
pub use crate::algorithms::games::cited_type::cited_type_game;
pub use crate::algorithms::games::citing_cited_type::citing_cited_type_game;
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
pub use crate::algorithms::games::iea_game::iea_game;
pub use crate::algorithms::games::islands::simple_interconnected_islands_game;
pub use crate::algorithms::games::k_regular::k_regular_game;
pub use crate::algorithms::games::lastcit::lastcit_game;
pub use crate::algorithms::games::preference::{asymmetric_preference_game, preference_game};
pub use crate::algorithms::games::recent_degree::recent_degree_game;
pub use crate::algorithms::games::recent_degree_aging::recent_degree_aging_game;
pub use crate::algorithms::games::sbm::sbm_game;
pub use crate::algorithms::games::static_fitness::{static_fitness_game, static_power_law_game};
pub use crate::algorithms::games::tree::{tree_game_lerw, tree_game_prufer};
pub use crate::algorithms::games::watts::watts_strogatz_game;
pub use crate::algorithms::io::dimacs::{
    DimacsGraph, DimacsProblem, read_dimacs, write_dimacs_flow,
};
pub use crate::algorithms::io::dl::{DlGraph, read_dl};
pub use crate::algorithms::io::dot::write_dot;
pub use crate::algorithms::io::edgelist::{read_edgelist, write_edgelist};
pub use crate::algorithms::io::gml::{read_gml, write_gml};
pub use crate::algorithms::io::graphdb::read_graphdb;
pub use crate::algorithms::io::graphml::write_graphml;
pub use crate::algorithms::io::leda::write_leda;
pub use crate::algorithms::io::lgl::{LglGraph, read_lgl, write_lgl};
pub use crate::algorithms::io::ncol::{NcolGraph, read_ncol, write_ncol};
pub use crate::algorithms::io::pajek::{PajekGraph, read_pajek, write_pajek};
pub use crate::algorithms::isomorphism::canonical::automorphism_group::automorphism_group;
pub use crate::algorithms::isomorphism::canonical::canonical_permutation::canonical_permutation;
pub use crate::algorithms::isomorphism::canonical::count_automorphisms::count_automorphisms;
pub use crate::algorithms::isomorphism::canonical::isomorphic_bliss::isomorphic_bliss;
pub use crate::algorithms::isomorphism::lad::{
    LadSubisomorphism, get_subisomorphisms_lad, subisomorphic_lad,
};
pub use crate::algorithms::isomorphism::queries::{isomorphic, subisomorphic};
pub use crate::algorithms::isomorphism::simplify_and_colorize::{
    SimplifyAndColorize, simplify_and_colorize,
};
pub use crate::algorithms::isomorphism::subiso::{
    Vf2Subisomorphism, count_subisomorphisms_vf2, get_subisomorphisms_vf2, subisomorphic_vf2,
};
pub use crate::algorithms::isomorphism::vf2::{
    Vf2Isomorphism, count_isomorphisms_vf2, get_isomorphisms_vf2, isomorphic_vf2,
};
pub use crate::algorithms::layout::bipartite::layout_bipartite;
pub use crate::algorithms::layout::davidson_harel::{DhParams, layout_davidson_harel};
pub use crate::algorithms::layout::drl::{DrlOptions, DrlTemplate, layout_drl};
pub use crate::algorithms::layout::fruchterman_reingold::{
    FrBounds, FrBounds3d, FrGrid, FrParams, FrParams3d, layout_fruchterman_reingold,
    layout_fruchterman_reingold_3d,
};
pub use crate::algorithms::layout::gem::{GemParams, layout_gem};
pub use crate::algorithms::layout::graphopt::{GraphoptParams, layout_graphopt};
pub use crate::algorithms::layout::kamada_kawai::{
    KkBounds, KkBounds3d, KkParams, KkParams3d, layout_kamada_kawai, layout_kamada_kawai_3d,
};
pub use crate::algorithms::layout::lgl::{LglParams, layout_lgl};
pub use crate::algorithms::layout::mds::layout_mds;
pub use crate::algorithms::layout::merge_dla::layout_merge_dla;
pub use crate::algorithms::layout::reingold_tilford::{
    RootChoice, RtMode, layout_reingold_tilford, layout_reingold_tilford_circular,
    roots_for_tree_layout,
};
pub use crate::algorithms::layout::simple::{
    layout_circle, layout_grid, layout_grid_3d, layout_random, layout_random_3d, layout_sphere,
    layout_star,
};
pub use crate::algorithms::layout::sugiyama::{SugiyamaParams, SugiyamaResult, layout_sugiyama};
pub use crate::algorithms::layout::umap::{
    UmapParams, layout_umap, layout_umap_3d, umap_compute_weights,
};
pub use crate::algorithms::matching::{
    MatchingResult, is_matching, is_maximal_matching, maximum_bipartite_matching,
    maximum_bipartite_matching_weighted,
};
pub use crate::algorithms::matching_lsap::solve_lsap;
pub use crate::algorithms::minimum_cycle_basis::minimum_cycle_basis;
pub use crate::algorithms::motifs::{
    DyadCensus, TriadCensus, TriadType, dyad_census, graph_count, isoclass, isoclass_create,
    isoclass_subgraph, motifs_randesu, motifs_randesu_callback, motifs_randesu_estimate,
    motifs_randesu_no, triad_census,
};
pub use crate::algorithms::operators::bipartite_projection::{
    BipartiteProjection, bipartite_projection,
};
pub use crate::algorithms::operators::bipartite_projection_size::{
    BipartiteProjectionSize, bipartite_projection_size,
};
pub use crate::algorithms::operators::complementer::complementer;
pub use crate::algorithms::operators::compose::compose;
pub use crate::algorithms::operators::connect_neighborhood::{connect_neighborhood, graph_power};
pub use crate::algorithms::operators::contract_vertices::contract_vertices;
pub use crate::algorithms::operators::difference::difference;
pub use crate::algorithms::operators::disjoint_union::{disjoint_union, disjoint_union_many};
pub use crate::algorithms::operators::even_tarjan::{EvenTarjanResult, even_tarjan_reduction};
pub use crate::algorithms::operators::induced_subgraph::{InducedSubgraphResult, induced_subgraph};
pub use crate::algorithms::operators::induced_subgraph_edges::induced_subgraph_edges;
pub use crate::algorithms::operators::intersection::{intersection, intersection_many};
pub use crate::algorithms::operators::is_same_graph::is_same_graph;
pub use crate::algorithms::operators::join::join;
pub use crate::algorithms::operators::permute_vertices::{invert_permutation, permute_vertices};
pub use crate::algorithms::operators::products::{
    GraphProductType, cartesian_product, graph_product, lexicographic_product, modular_product,
    rooted_product, strong_product, tensor_product,
};
pub use crate::algorithms::operators::residual_graph::{
    ResidualGraphResult, residual_graph, reverse_residual_graph,
};
pub use crate::algorithms::operators::reverse::{reverse, reverse_edges};
pub use crate::algorithms::operators::rewire::rewire;
pub use crate::algorithms::operators::rewire_edges::{
    RewireDirectedMode, rewire_directed_edges, rewire_edges,
};
pub use crate::algorithms::operators::simplify::simplify;
pub use crate::algorithms::operators::subgraph_from_edges::{
    SubgraphFromEdgesResult, subgraph_from_edges,
};
pub use crate::algorithms::operators::to_directed::{ToDirectedMode, to_directed};
pub use crate::algorithms::operators::to_undirected::{ToUndirectedMode, to_undirected};
pub use crate::algorithms::operators::union::{union, union_many};
pub use crate::algorithms::paths::all_shortest_paths::{
    AllShortestPaths, AllShortestPathsMode, get_all_shortest_paths,
    get_all_shortest_paths_with_mode,
};
pub use crate::algorithms::paths::astar::a_star_path;
pub use crate::algorithms::paths::bellman_ford::{
    BellmanFordPathEntry, bellman_ford_distances, bellman_ford_distances_with_mode,
    bellman_ford_path_to, bellman_ford_path_to_with_mode, bellman_ford_paths,
    bellman_ford_paths_with_mode,
};
pub use crate::algorithms::paths::dijkstra::{
    DijkstraAllPaths, DijkstraMode, DijkstraPaths, dijkstra_all_shortest_paths, dijkstra_distances,
    dijkstra_distances_cutoff, dijkstra_distances_cutoff_with_mode, dijkstra_distances_multi,
    dijkstra_distances_multi_with_mode, dijkstra_distances_with_mode, dijkstra_path_to,
    dijkstra_path_to_with_mode, dijkstra_paths, dijkstra_paths_with_mode,
};
pub use crate::algorithms::paths::distances::distances;
pub use crate::algorithms::paths::distances_all::{
    DistancesMode, distances_all, distances_all_with_mode,
};
pub use crate::algorithms::paths::distances_cutoff::{
    DistancesCutoffMode, distances_cutoff, distances_cutoff_multi, distances_cutoff_with_mode,
};
pub use crate::algorithms::paths::distances_from::{
    DistancesFromMode, distances_from, distances_from_with_mode,
};
pub use crate::algorithms::paths::edge_path_to_vertex_path::{
    WalkMode, vertex_path_from_edge_path,
};
pub use crate::algorithms::paths::eulerian::{EulerianClassification, is_eulerian};
pub use crate::algorithms::paths::eulerian_construct::{eulerian_cycle, eulerian_path};
pub use crate::algorithms::paths::floyd_warshall::floyd_warshall_distances;
pub use crate::algorithms::paths::get_shortest_path::{ShortestPath, get_shortest_path};
pub use crate::algorithms::paths::graph_center::{
    PseudoDiameterResult, graph_center, pseudo_diameter,
};
pub use crate::algorithms::paths::histogram::{PathLengthHistResult, path_length_hist};
pub use crate::algorithms::paths::johnson::johnson_distances;
pub use crate::algorithms::paths::k_shortest_paths::{KShortestPath, k_shortest_paths};
pub use crate::algorithms::paths::radii::{
    EccMode, diameter, diameter_weighted, diameter_weighted_with_mode, diameter_with_mode,
    eccentricity, eccentricity_weighted, eccentricity_weighted_with_mode, eccentricity_with_mode,
    radius, radius_weighted, radius_weighted_with_mode, radius_with_mode,
};
pub use crate::algorithms::paths::random_walk::random_walk;
pub use crate::algorithms::paths::shortest_paths::{
    ShortestPathMode, get_shortest_paths, get_shortest_paths_with_mode,
};
pub use crate::algorithms::paths::simple_paths::{SimplePathMode, all_simple_paths};
pub use crate::algorithms::paths::spanner::spanner;
pub use crate::algorithms::paths::voronoi::{VoronoiPartition, VoronoiTiebreaker, voronoi};
pub use crate::algorithms::paths::widest_path::{
    WidestPathResult, WidestPaths, widest_path, widest_path_widths,
    widest_path_widths_floyd_warshall, widest_path_widths_floyd_warshall_with_mode,
    widest_path_widths_with_mode, widest_path_with_mode, widest_paths, widest_paths_to,
    widest_paths_to_with_mode, widest_paths_with_mode,
};
pub use crate::algorithms::properties::adjacency::{AdjacencyType, LoopHandling, get_adjacency};
pub use crate::algorithms::properties::are_adjacent::are_adjacent;
pub use crate::algorithms::properties::assortativity::{
    assortativity_degree, assortativity_degree_directed,
};
pub use crate::algorithms::properties::assortativity_nominal::assortativity_nominal;
pub use crate::algorithms::properties::assortativity_values::assortativity;
pub use crate::algorithms::properties::assortativity_weighted::{
    assortativity_degree_directed_weighted, assortativity_degree_weighted,
};
pub use crate::algorithms::properties::basic::{density, mean_degree, mean_distance};
pub use crate::algorithms::properties::betweenness::betweenness;
pub use crate::algorithms::properties::betweenness_cutoff::betweenness_cutoff;
pub use crate::algorithms::properties::betweenness_subset::betweenness_subset;
pub use crate::algorithms::properties::betweenness_weighted::betweenness_weighted;
pub use crate::algorithms::properties::centralization::{
    CentralizationMode, CentralizationResult, LoopMode, centralization,
    centralization_betweenness_tmax, centralization_betweenness_wrapper,
    centralization_closeness_tmax, centralization_closeness_wrapper, centralization_degree_tmax,
    centralization_degree_wrapper, centralization_eigenvector_tmax,
    centralization_eigenvector_wrapper,
};
pub use crate::algorithms::properties::closeness::closeness;
pub use crate::algorithms::properties::closeness_cutoff::{
    ClosenessCutoffResult, closeness_cutoff,
};
pub use crate::algorithms::properties::closeness_weighted::closeness_weighted;
pub use crate::algorithms::properties::constraint::constraint;
pub use crate::algorithms::properties::convergence_degree::{
    convergence_degree, convergence_degree_full,
};
pub use crate::algorithms::properties::coreness::{CorenessMode, coreness, coreness_with_mode};
pub use crate::algorithms::properties::degree::{
    DegreeMode, degree_sequence, max_degree, max_degree_vertex, min_degree,
};
pub use crate::algorithms::properties::degree_correlation::degree_correlation_vector;
pub use crate::algorithms::properties::degree_distribution::degree_distribution;
pub use crate::algorithms::properties::ecc::ecc;
pub use crate::algorithms::properties::edge_betweenness::edge_betweenness;
pub use crate::algorithms::properties::edge_betweenness_cutoff::edge_betweenness_cutoff;
pub use crate::algorithms::properties::edge_betweenness_subset::edge_betweenness_subset;
pub use crate::algorithms::properties::edge_betweenness_weighted::edge_betweenness_weighted;
pub use crate::algorithms::properties::edgelist::get_edgelist;
pub use crate::algorithms::properties::efficiency::{
    average_local_efficiency, global_efficiency, local_efficiency,
};
pub use crate::algorithms::properties::eigenvector::{
    EigenvectorMode, EigenvectorScores, eigenvector_centrality, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_full,
    eigenvector_centrality_weighted,
};
pub use crate::algorithms::properties::get_biadjacency::{
    GetBiadjacencyResult, get_biadjacency_matrix,
};
pub use crate::algorithms::properties::get_biadjacency_weighted::{
    GetBiadjacencyWeightedResult, get_biadjacency_weighted,
};
pub use crate::algorithms::properties::get_eids::get_eids;
pub use crate::algorithms::properties::girth::girth;
pub use crate::algorithms::properties::graphicality::{
    EdgeTypeFilter, is_bigraphical, is_graphical,
};
pub use crate::algorithms::properties::harmonic::harmonic_centrality;
pub use crate::algorithms::properties::harmonic_cutoff::harmonic_centrality_cutoff;
pub use crate::algorithms::properties::harmonic_weighted::harmonic_centrality_weighted;
pub use crate::algorithms::properties::hits::{
    HitsScores, hub_and_authority_scores, hub_and_authority_scores_weighted,
};
pub use crate::algorithms::properties::is_acyclic::is_acyclic;
pub use crate::algorithms::properties::is_bipartite::{BipartiteResult, is_bipartite};
pub use crate::algorithms::properties::is_clique::{is_clique, is_independent_vertex_set};
pub use crate::algorithms::properties::is_complete::is_complete;
pub use crate::algorithms::properties::is_dag::is_dag;
pub use crate::algorithms::properties::is_forest::is_forest;
pub use crate::algorithms::properties::is_simple::{SimpleMode, is_simple, is_simple_with_mode};
pub use crate::algorithms::properties::is_tree::is_tree;
pub use crate::algorithms::properties::is_triangle_free::is_triangle_free;
pub use crate::algorithms::properties::joint_degree_distribution::joint_degree_distribution;
pub use crate::algorithms::properties::joint_degree_matrix::joint_degree_matrix;
pub use crate::algorithms::properties::joint_type_distribution::joint_type_distribution;
pub use crate::algorithms::properties::knn::{
    avg_nearest_neighbor_degree, avg_nearest_neighbor_degree_weighted, knnk, knnk_weighted,
};
pub use crate::algorithms::properties::laplacian::{LaplacianNormalization, get_laplacian};
pub use crate::algorithms::properties::list_triangles::list_triangles;
pub use crate::algorithms::properties::local_scan::{
    local_scan_0, local_scan_0_them, local_scan_1, local_scan_1_ecount, local_scan_1_ecount_them,
    local_scan_subset_ecount,
};
pub use crate::algorithms::properties::local_scan_k::{
    local_scan_k, local_scan_k_ecount, local_scan_k_ecount_them,
};
pub use crate::algorithms::properties::mean_distance_weighted::mean_distance_weighted;
pub use crate::algorithms::properties::multiplicity::{
    count_loops, count_multiple, count_multiple_1, has_loop, has_multiple, is_loop, is_multiple,
};
pub use crate::algorithms::properties::mutual::{count_mutual, has_mutual, is_mutual};
pub use crate::algorithms::properties::neighborhood::{
    NeighborhoodMode, neighborhood, neighborhood_graphs, neighborhood_graphs_with_mode,
    neighborhood_size, neighborhood_size_with_mode, neighborhood_with_mode,
};
pub use crate::algorithms::properties::pagerank::pagerank;
pub use crate::algorithms::properties::pagerank_linsys::pagerank_linsys;
pub use crate::algorithms::properties::pagerank_weighted::pagerank_weighted;
pub use crate::algorithms::properties::perfect::is_perfect;
pub use crate::algorithms::properties::personalized_pagerank::{
    personalized_pagerank, personalized_pagerank_default, personalized_pagerank_vs,
};
pub use crate::algorithms::properties::power_law_fit::{PowerLawFitResult, power_law_fit};
pub use crate::algorithms::properties::reciprocity::{
    ReciprocityMode, reciprocity, reciprocity_with_mode,
};
pub use crate::algorithms::properties::rich_club::rich_club_sequence;
pub use crate::algorithms::properties::running_mean::{expand_path_to_pairs, running_mean};
pub use crate::algorithms::properties::similarity::{
    bibcoupling, cocitation, similarity_dice, similarity_dice_es, similarity_dice_pairs,
    similarity_inverse_log_weighted, similarity_inverse_log_weighted_pairs, similarity_jaccard,
    similarity_jaccard_es, similarity_jaccard_pairs,
};
pub use crate::algorithms::properties::sort_by_degree::{SortOrder, sort_vertices_by_degree};
pub use crate::algorithms::properties::stochastic::get_stochastic;
pub use crate::algorithms::properties::strength::{
    StrengthMode, diversity, strength, strength_with_mode,
};
pub use crate::algorithms::properties::topological_sorting::topological_sorting;
pub use crate::algorithms::properties::triangles::{
    TransitivityMode, count_adjacent_triangles, count_triangles, transitivity_avglocal_undirected,
    transitivity_barrat, transitivity_local_undirected, transitivity_undirected,
};
pub use crate::algorithms::properties::trussness::trussness;
pub use crate::algorithms::properties::unfold_tree::{UnfoldTreeResult, unfold_tree};
pub use crate::algorithms::simple_cycles::{SimpleCycle, SimpleCycleMode, simple_cycles};
pub use crate::algorithms::spanning::mst::{MstAlgorithm, minimum_spanning_tree};
pub use crate::algorithms::spanning::random_spanning_tree::random_spanning_tree;
pub use crate::algorithms::spatial::beta_weighted_gabriel_graph::{
    BetaWeightedGabriel, beta_weighted_gabriel_graph,
};
pub use crate::algorithms::spatial::circle_beta_skeleton::circle_beta_skeleton;
pub use crate::algorithms::spatial::convex_hull::{ConvexHullResult, convex_hull_2d};
pub use crate::algorithms::spatial::edge_lengths::{DistanceMetric, spatial_edge_lengths};
pub use crate::algorithms::spatial::gabriel_graph::gabriel_graph;
pub use crate::algorithms::spatial::lune_beta_skeleton::lune_beta_skeleton;
pub use crate::algorithms::spatial::nearest_neighbor_graph::nearest_neighbor_graph;
pub use crate::algorithms::spatial::relative_neighborhood_graph::relative_neighborhood_graph;
pub use crate::algorithms::traversal::bfs::{
    BfsMode, BfsSimple, BfsTree, bfs, bfs_simple, bfs_tree,
};
pub use crate::algorithms::traversal::dfs::{
    DfsMode, DfsSimple, DfsTree, dfs, dfs_simple, dfs_tree,
};
pub use crate::core::error::{IgraphError, IgraphResult};
pub use crate::core::graph::{Graph, VertexId};
