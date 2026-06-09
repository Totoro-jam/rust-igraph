//! Graph properties — invariants and metrics. Phase 1 entries:
//! ALGO-PR-001 (`girth`), ALGO-PR-002 (triangles + global/local
//! transitivity), ALGO-PR-003 (density + mean distance), ALGO-PR-004
//! (reciprocity), ALGO-PR-005 (avg nearest-neighbour degree),
//! ALGO-PR-006 (degree assortativity).

// `pub(crate)` instead of `pub` so the inner module names (often
// identical to the function they expose, e.g. `is_simple`,
// `pagerank`) don't collide with the function re-exports in the
// rendered rustdoc — see https://github.com/rust-lang/rust/issues/...
pub(crate) mod abc_variants;
pub(crate) mod adjacency;
pub(crate) mod algebraic_connectivity;
pub(crate) mod are_adjacent;
pub(crate) mod assortativity;
pub(crate) mod assortativity_nominal;
pub(crate) mod assortativity_values;
pub(crate) mod assortativity_weighted;
pub(crate) mod augmented_zagreb;
pub(crate) mod balaban_index;
pub(crate) mod basic;
pub(crate) mod betweenness;
pub(crate) mod betweenness_cutoff;
pub(crate) mod betweenness_subset;
pub(crate) mod betweenness_weighted;
pub(crate) mod centralization;
pub(crate) mod clique_cover;
pub(crate) mod closeness;
pub(crate) mod closeness_cutoff;
pub(crate) mod closeness_weighted;
pub(crate) mod constraint;
pub(crate) mod convergence_degree;
pub(crate) mod coreness;
pub(crate) mod cut_metrics;
pub(crate) mod degree;
pub(crate) mod degree_correlation;
pub(crate) mod degree_distribution;
pub(crate) mod degree_eccentricity;
pub(crate) mod diffusion;
pub(crate) mod distance_spectrum;
pub(crate) mod ecc;
pub(crate) mod eccentric_connectivity;
pub(crate) mod edge_betweenness;
pub(crate) mod edge_betweenness_cutoff;
pub(crate) mod edge_betweenness_subset;
pub(crate) mod edge_betweenness_weighted;
pub(crate) mod edge_degree_indices;
pub(crate) mod edgelist;
pub(crate) mod efficiency;
pub(crate) mod eigenvector;
pub(crate) mod forgotten_zagreb;
pub(crate) mod general_randic;
pub(crate) mod get_biadjacency;
pub(crate) mod get_biadjacency_weighted;
pub(crate) mod get_eids;
pub(crate) mod girth;
pub(crate) mod gourava_index;
pub(crate) mod graph_bandwidth;
pub(crate) mod graph_coloring;
pub(crate) mod graph_curvature;
pub(crate) mod graph_entropy;
pub(crate) mod graph_periphery;
pub(crate) mod graphicality;
pub(crate) mod hamiltonian;
pub(crate) mod harmonic;
pub(crate) mod harmonic_cutoff;
pub(crate) mod harmonic_weighted;
pub(crate) mod hits;
pub(crate) mod homophily;
pub(crate) mod hosoya_index;
pub(crate) mod hyper_wiener;
pub(crate) mod hyper_zagreb;
pub(crate) mod hyperbolicity;
pub(crate) mod independent_set;
pub(crate) mod inverse_degree;
pub(crate) mod irregularity;
pub(crate) mod is_acyclic;
pub(crate) mod is_apex_forest;
pub(crate) mod is_apex_tree;
pub(crate) mod is_banner_free;
pub(crate) mod is_biclique;
pub(crate) mod is_bipartite;
pub(crate) mod is_biregular;
pub(crate) mod is_block;
pub(crate) mod is_bowtie_free;
pub(crate) mod is_bull_free;
pub(crate) mod is_c4_free;
pub(crate) mod is_c5_free;
pub(crate) mod is_cactus;
pub(crate) mod is_caterpillar;
pub(crate) mod is_chain_graph;
pub(crate) mod is_chordal_bipartite;
pub(crate) mod is_claw_free;
pub(crate) mod is_clique;
pub(crate) mod is_cluster;
pub(crate) mod is_co_bipartite;
pub(crate) mod is_co_chordal;
pub(crate) mod is_cograph;
pub(crate) mod is_complete;
pub(crate) mod is_complete_bipartite;
pub(crate) mod is_complete_multipartite;
pub(crate) mod is_cricket_free;
pub(crate) mod is_cubic;
pub(crate) mod is_cycle;
pub(crate) mod is_dag;
pub(crate) mod is_dart_free;
pub(crate) mod is_diamond_free;
pub(crate) mod is_distance_hereditary;
pub(crate) mod is_forest;
pub(crate) mod is_fork_free;
pub(crate) mod is_gem_free;
pub(crate) mod is_geodetic;
pub(crate) mod is_house_free;
pub(crate) mod is_k_degenerate;
pub(crate) mod is_lobster;
pub(crate) mod is_net_free;
pub(crate) mod is_outerplanar;
pub(crate) mod is_p5_free;
pub(crate) mod is_path;
pub(crate) mod is_paw_free;
pub(crate) mod is_planar;
pub(crate) mod is_proper_interval;
pub(crate) mod is_pseudo_forest;
pub(crate) mod is_ptolemaic;
pub(crate) mod is_regular;
pub(crate) mod is_self_complementary;
pub(crate) mod is_semicomplete;
pub(crate) mod is_series_parallel;
pub(crate) mod is_simple;
pub(crate) mod is_spider;
pub(crate) mod is_split;
pub(crate) mod is_star;
pub(crate) mod is_strongly_chordal;
pub(crate) mod is_strongly_regular;
pub(crate) mod is_threshold;
pub(crate) mod is_tournament;
pub(crate) mod is_tree;
pub(crate) mod is_triangle_free;
pub(crate) mod is_trivially_perfect;
pub(crate) mod is_unicyclic;
pub(crate) mod is_weakly_chordal;
pub(crate) mod is_well_covered;
pub(crate) mod is_wheel;
pub(crate) mod is_windmill;
pub(crate) mod joint_degree_distribution;
pub(crate) mod joint_degree_matrix;
pub(crate) mod joint_type_distribution;
pub(crate) mod katz_centrality;
pub(crate) mod knn;
pub(crate) mod label_spread;
pub(crate) mod laplacian;
pub(crate) mod link_prediction;
pub(crate) mod list_triangles;
pub(crate) mod local_scan;
pub(crate) mod local_scan_k;
pub(crate) mod matching;
pub(crate) mod mean_distance_weighted;
pub(crate) mod merrifield_simmons;
pub(crate) mod mostar_index;
pub(crate) mod multiplicity;
pub(crate) mod mutual;
pub(crate) mod narumi_katayama;
pub(crate) mod neighbor_agg;
pub(crate) mod neighborhood;
pub(crate) mod neighborhood_zagreb;
pub(crate) mod normalized_laplacian;
pub(crate) mod pagerank;
pub(crate) mod pagerank_linsys;
pub(crate) mod pagerank_weighted;
pub(crate) mod perfect;
pub(crate) mod personalized_pagerank;
pub(crate) mod power_law_fit;
pub(crate) mod reciprocity;
pub(crate) mod reformulated_zagreb;
pub(crate) mod resistance;
pub(crate) mod rich_club;
pub(crate) mod robustness;
pub(crate) mod running_mean;
pub(crate) mod rwpe;
pub(crate) mod satisfies_dirac;
pub(crate) mod satisfies_ore;
pub(crate) mod schultz_index;
pub(crate) mod signal_smoothness;
pub(crate) mod signless_laplacian;
pub(crate) mod similarity;
pub(crate) mod sombor_index;
pub(crate) mod sort_by_degree;
pub(crate) mod spectral_metrics;
pub(crate) mod stochastic;
pub(crate) mod strength;
pub(crate) mod structural_features;
pub(crate) mod sum_connectivity;
pub(crate) mod summary;
pub(crate) mod szeged_edge;
pub(crate) mod szeged_index;
pub(crate) mod topological_indices;
pub(crate) mod topological_sorting;
pub(crate) mod treewidth;
pub(crate) mod triangles;
pub(crate) mod trussness;
pub(crate) mod unfold_tree;
pub(crate) mod wiener_polarity_index;
pub(crate) mod zagreb_connection;

pub use abc_variants::{degree_sum_index, fifth_ga_index, fourth_abc_index};
pub use adjacency::{AdjacencyType, LoopHandling, get_adjacency};
pub use algebraic_connectivity::{
    algebraic_connectivity, fiedler_vector, laplacian_spectrum, spanning_tree_count,
    spectral_bisection,
};
pub use are_adjacent::are_adjacent;
pub use assortativity::{assortativity_degree, assortativity_degree_directed};
pub use assortativity_nominal::assortativity_nominal;
pub use assortativity_values::assortativity;
pub use assortativity_weighted::{
    assortativity_degree_directed_weighted, assortativity_degree_weighted,
};
pub use augmented_zagreb::{
    atom_bond_sum_connectivity, augmented_zagreb_index, geometric_arithmetic_index,
};
pub use balaban_index::balaban_j_index;
pub use basic::{density, mean_degree, mean_distance};
pub use betweenness::betweenness;
pub use betweenness_cutoff::betweenness_cutoff;
pub use betweenness_subset::betweenness_subset;
pub use betweenness_weighted::betweenness_weighted;
pub use centralization::{
    CentralizationMode, CentralizationResult, LoopMode, centralization,
    centralization_betweenness_tmax, centralization_betweenness_wrapper,
    centralization_closeness_tmax, centralization_closeness_wrapper, centralization_degree_tmax,
    centralization_degree_wrapper, centralization_eigenvector_tmax,
    centralization_eigenvector_wrapper,
};
pub use clique_cover::{clique_cover_number, greedy_clique_cover, is_clique_cover};
pub use closeness::closeness;
pub use closeness_cutoff::{ClosenessCutoffResult, closeness_cutoff};
pub use closeness_weighted::closeness_weighted;
pub use constraint::constraint;
pub use convergence_degree::{convergence_degree, convergence_degree_full};
pub use coreness::{CorenessMode, coreness, coreness_with_mode};
pub use cut_metrics::{conductance, cut_size, expansion, normalized_cut, ratio_cut};
pub use degree::{DegreeMode, degree_sequence, max_degree, max_degree_vertex, min_degree};
pub use degree_correlation::degree_correlation_vector;
pub use degree_distribution::degree_distribution;
pub use degree_eccentricity::{degree_eccentricity_index, eccentric_distance_sum, lanzhou_index};
pub use diffusion::{heat_kernel_diffuse, ppr_diffuse, symmetric_diffuse};
pub use distance_spectrum::{
    distance_energy, distance_estrada_index, distance_spectral_radius, distance_spectrum,
    wiener_index,
};
pub use ecc::ecc;
pub use eccentric_connectivity::{
    connective_eccentricity_index, eccentric_connectivity_index, total_eccentricity,
};
pub use edge_betweenness::edge_betweenness;
pub use edge_betweenness_cutoff::edge_betweenness_cutoff;
pub use edge_betweenness_subset::edge_betweenness_subset;
pub use edge_betweenness_weighted::edge_betweenness_weighted;
pub use edge_degree_indices::{bertz_complexity_index, gordon_scantlebury_index, platt_index};
pub use edgelist::get_edgelist;
pub use efficiency::{average_local_efficiency, global_efficiency, local_efficiency};
pub use eigenvector::{
    EigenvectorMode, EigenvectorScores, eigenvector_centrality, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_full,
    eigenvector_centrality_weighted,
};
pub use forgotten_zagreb::{forgotten_index, modified_first_zagreb, reduced_second_zagreb};
pub use general_randic::{
    general_randic_index, general_sum_connectivity_index, reciprocal_randic_index,
};
pub use get_biadjacency::{GetBiadjacencyResult, get_biadjacency_matrix};
pub use get_biadjacency_weighted::{GetBiadjacencyWeightedResult, get_biadjacency_weighted};
pub use get_eids::get_eids;
pub use girth::girth;
pub use gourava_index::{first_gourava_index, first_hyper_gourava_index, second_gourava_index};
pub use graph_bandwidth::{bandwidth, bandwidth_lower_bound, bandwidth_of_labeling};
pub use graph_coloring::{
    chromatic_number_greedy, greedy_clique_number, greedy_coloring, greedy_coloring_largest_first,
    greedy_coloring_with_order, is_proper_coloring,
};
pub use graph_curvature::{
    augmented_forman_ricci_curvature, forman_ricci_curvature, mean_forman_ricci,
    ollivier_ricci_curvature,
};
pub use graph_entropy::{
    degree_entropy, degree_structural_info, edge_entropy, von_neumann_entropy,
};
pub use graph_periphery::{EccentricityClasses, eccentricity_classes, graph_periphery};
pub use graphicality::{EdgeTypeFilter, is_bigraphical, is_graphical};
pub use hamiltonian::{
    hamiltonian_cycle, hamiltonian_path, has_hamiltonian_cycle, has_hamiltonian_path,
    is_hamiltonian_cycle, is_hamiltonian_path,
};
pub use harmonic::harmonic_centrality;
pub use harmonic_cutoff::harmonic_centrality_cutoff;
pub use harmonic_weighted::harmonic_centrality_weighted;
pub use hits::{HitsScores, hub_and_authority_scores};
pub use homophily::{class_homophily, edge_heterophily, edge_homophily, node_homophily};
pub use hosoya_index::{hosoya_index, matching_count_sequence};
pub use hyper_wiener::{harary_index, hyper_wiener_index};
pub use hyper_zagreb::{first_hyper_zagreb, first_redefined_zagreb, second_hyper_zagreb};
pub use hyperbolicity::{hyperbolicity, hyperbolicity_twice};
pub use independent_set::{greedy_independent_set, independence_ratio};
pub use inverse_degree::{first_zagreb_coindex, inverse_degree_index, second_zagreb_coindex};
pub use irregularity::{albertson_index, degree_variance, sigma_index, total_irregularity};
pub use is_acyclic::is_acyclic;
pub use is_apex_forest::is_apex_forest;
pub use is_apex_tree::is_apex_tree;
pub use is_banner_free::is_banner_free;
pub use is_biclique::is_biclique;
pub use is_bipartite::{BipartiteResult, is_bipartite};
pub use is_biregular::is_biregular;
pub use is_block::is_block_graph;
pub use is_bowtie_free::is_bowtie_free;
pub use is_bull_free::is_bull_free;
pub use is_c4_free::is_c4_free;
pub use is_c5_free::is_c5_free;
pub use is_cactus::is_cactus_graph;
pub use is_caterpillar::is_caterpillar;
pub use is_chain_graph::is_chain_graph;
pub use is_chordal_bipartite::is_chordal_bipartite;
pub use is_claw_free::is_claw_free;
pub use is_clique::{is_clique, is_independent_vertex_set};
pub use is_cluster::is_cluster_graph;
pub use is_co_bipartite::is_co_bipartite;
pub use is_co_chordal::is_co_chordal;
pub use is_cograph::is_cograph;
pub use is_complete::is_complete;
pub use is_complete_bipartite::is_complete_bipartite;
pub use is_complete_multipartite::is_complete_multipartite;
pub use is_cricket_free::is_cricket_free;
pub use is_cubic::is_cubic;
pub use is_cycle::is_cycle;
pub use is_dag::is_dag;
pub use is_dart_free::is_dart_free;
pub use is_diamond_free::is_diamond_free;
pub use is_distance_hereditary::is_distance_hereditary;
pub use is_forest::is_forest;
pub use is_fork_free::is_fork_free;
pub use is_gem_free::is_gem_free;
pub use is_geodetic::is_geodetic;
pub use is_house_free::is_house_free;
pub use is_k_degenerate::{degeneracy, is_k_degenerate};
pub use is_lobster::is_lobster;
pub use is_net_free::is_net_free;
pub use is_outerplanar::is_outerplanar;
pub use is_p5_free::is_p5_free;
pub use is_path::is_path;
pub use is_paw_free::is_paw_free;
pub use is_planar::is_planar;
pub use is_proper_interval::is_proper_interval;
pub use is_pseudo_forest::is_pseudo_forest;
pub use is_ptolemaic::is_ptolemaic;
pub use is_regular::{is_regular, regularity};
pub use is_self_complementary::is_self_complementary;
pub use is_semicomplete::is_semicomplete;
pub use is_series_parallel::is_series_parallel;
pub use is_simple::{SimpleMode, is_simple, is_simple_with_mode};
pub use is_spider::is_spider;
pub use is_split::is_split_graph;
pub use is_star::is_star;
pub use is_strongly_chordal::is_strongly_chordal;
pub use is_strongly_regular::{StronglyRegularParams, is_strongly_regular};
pub use is_threshold::is_threshold_graph;
pub use is_tournament::is_tournament;
pub use is_tree::is_tree;
pub use is_triangle_free::is_triangle_free;
pub use is_trivially_perfect::is_trivially_perfect;
pub use is_unicyclic::is_unicyclic;
pub use is_weakly_chordal::is_weakly_chordal;
pub use is_well_covered::is_well_covered;
pub use is_wheel::is_wheel;
pub use is_windmill::is_windmill;
pub use joint_degree_distribution::joint_degree_distribution;
pub use joint_degree_matrix::joint_degree_matrix;
pub use joint_type_distribution::joint_type_distribution;
pub use knn::avg_nearest_neighbor_degree;
pub use label_spread::{LabelSpreadResult, label_propagate_predict, label_spread};
pub use laplacian::{LaplacianNormalization, get_laplacian};
pub use link_prediction::{
    link_pred_adamic_adar, link_pred_common_neighbors, link_pred_jaccard,
    link_pred_preferential_attachment, link_pred_resource_allocation,
};
pub use list_triangles::list_triangles;
pub use local_scan::{
    local_scan_0, local_scan_0_them, local_scan_1, local_scan_1_ecount, local_scan_1_ecount_them,
    local_scan_subset_ecount,
};
pub use local_scan_k::{local_scan_k, local_scan_k_ecount, local_scan_k_ecount_them};
pub use matching::{
    greedy_matching, is_perfect_matching, is_valid_matching, matching_number, maximum_matching,
};
pub use mean_distance_weighted::mean_distance_weighted;
pub use merrifield_simmons::{independent_set_count_sequence, merrifield_simmons_index};
pub use mostar_index::{degree_distance, gutman_index, mostar_index};
pub use multiplicity::{
    count_loops, count_multiple, count_multiple_1, has_loop, has_multiple, is_loop, is_multiple,
};
pub use mutual::{count_mutual, has_mutual, is_mutual};
pub use narumi_katayama::{
    first_multiplicative_zagreb, narumi_katayama_index, second_multiplicative_zagreb,
};
pub use neighbor_agg::{AggMode, attention_aggregate, neighbor_aggregate};
pub use neighborhood::{
    NeighborhoodMode, neighborhood, neighborhood_graphs, neighborhood_graphs_with_mode,
    neighborhood_size, neighborhood_size_with_mode, neighborhood_with_mode,
};
pub use neighborhood_zagreb::{
    first_neighborhood_zagreb, neighborhood_forgotten_index, second_neighborhood_zagreb,
};
pub use normalized_laplacian::{
    bipartiteness_ratio, cheeger_bounds, normalized_algebraic_connectivity,
    normalized_laplacian_spectrum, spectral_gap_ratio,
};
pub use pagerank::pagerank;
pub use pagerank_linsys::pagerank_linsys;
pub use pagerank_weighted::pagerank_weighted;
pub use perfect::is_perfect;
pub use personalized_pagerank::{
    personalized_pagerank, personalized_pagerank_default, personalized_pagerank_vs,
};
pub use power_law_fit::{PowerLawFitResult, power_law_fit};
pub use reciprocity::{ReciprocityMode, reciprocity, reciprocity_with_mode};
pub use reformulated_zagreb::{
    first_reformulated_zagreb, second_reformulated_zagreb, third_zagreb_index,
};
pub use resistance::{
    effective_resistance, effective_resistance_matrix, kirchhoff_index, resistance_centrality,
};
pub use rich_club::rich_club_sequence;
pub use robustness::{edge_resilience, graph_integrity, graph_toughness, vertex_resilience};
pub use running_mean::{expand_path_to_pairs, running_mean};
pub use rwpe::{rwpe, rwpe_vertices};
pub use satisfies_dirac::satisfies_dirac;
pub use satisfies_ore::satisfies_ore;
pub use schultz_index::schultz_index;
pub use signal_smoothness::{
    dirichlet_energy, normalized_dirichlet_energy, smoothness_ratio, total_variation,
};
pub use signless_laplacian::{
    signless_laplacian_energy, signless_laplacian_smallest, signless_laplacian_spectral_radius,
    signless_laplacian_spectrum,
};
pub use similarity::{
    bibcoupling, cocitation, similarity_dice, similarity_dice_es, similarity_dice_pairs,
    similarity_inverse_log_weighted, similarity_inverse_log_weighted_pairs, similarity_jaccard,
    similarity_jaccard_es, similarity_jaccard_pairs,
};
pub use sombor_index::{average_sombor_index, reduced_sombor_index, sombor_index};
pub use sort_by_degree::{SortOrder, sort_vertices_by_degree};
pub use spectral_metrics::{
    communicability_matrix, estrada_index, graph_energy, natural_connectivity, spectral_gap,
    spectral_radius, subgraph_centrality,
};
pub use stochastic::get_stochastic;
pub use strength::{StrengthMode, diversity, strength, strength_with_mode};
pub use structural_features::{StructuralFeatures, degree_profile, structural_feature_vectors};
pub use sum_connectivity::{
    inverse_sum_indeg_index, sum_connectivity_index, symmetric_division_deg_index,
};
pub use summary::{GraphSummary, graph_summary, graph_summary_string};
pub use szeged_edge::{edge_pi_index, edge_szeged_index, graovac_ghorbani_index};
pub use szeged_index::{pi_index, revised_szeged_index, szeged_index};
pub use topological_indices::{
    abc_index, first_zagreb_index, harmonic_graph_index, randic_index, second_zagreb_index,
};
pub use topological_sorting::topological_sorting;
pub use treewidth::{elimination_ordering, treewidth_min_fill, treewidth_upper_bound};
pub use triangles::{
    TransitivityMode, count_adjacent_triangles, count_triangles, transitivity_avglocal_undirected,
    transitivity_local_undirected, transitivity_undirected,
};
pub use trussness::trussness;
pub use unfold_tree::{UnfoldTreeResult, unfold_tree};
pub use wiener_polarity_index::{count_pairs_at_distance, wiener_polarity_index};
pub use zagreb_connection::{
    first_zagreb_connection, modified_first_zagreb_connection, second_zagreb_connection,
};
