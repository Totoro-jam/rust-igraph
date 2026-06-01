//! Graph properties — invariants and metrics. Phase 1 entries:
//! ALGO-PR-001 (`girth`), ALGO-PR-002 (triangles + global/local
//! transitivity), ALGO-PR-003 (density + mean distance), ALGO-PR-004
//! (reciprocity), ALGO-PR-005 (avg nearest-neighbour degree),
//! ALGO-PR-006 (degree assortativity).

// `pub(crate)` instead of `pub` so the inner module names (often
// identical to the function they expose, e.g. `is_simple`,
// `pagerank`) don't collide with the function re-exports in the
// rendered rustdoc — see https://github.com/rust-lang/rust/issues/...
pub(crate) mod adjacency;
pub(crate) mod are_adjacent;
pub(crate) mod assortativity;
pub(crate) mod assortativity_nominal;
pub(crate) mod assortativity_values;
pub(crate) mod assortativity_weighted;
pub(crate) mod basic;
pub(crate) mod betweenness;
pub(crate) mod betweenness_cutoff;
pub(crate) mod betweenness_subset;
pub(crate) mod betweenness_weighted;
pub(crate) mod centralization;
pub(crate) mod closeness;
pub(crate) mod closeness_cutoff;
pub(crate) mod closeness_weighted;
pub(crate) mod constraint;
pub(crate) mod convergence_degree;
pub(crate) mod coreness;
pub(crate) mod degree;
pub(crate) mod degree_correlation;
pub(crate) mod degree_distribution;
pub(crate) mod ecc;
pub(crate) mod edge_betweenness;
pub(crate) mod edge_betweenness_cutoff;
pub(crate) mod edge_betweenness_subset;
pub(crate) mod edge_betweenness_weighted;
pub(crate) mod edgelist;
pub(crate) mod efficiency;
pub(crate) mod eigenvector;
pub(crate) mod get_biadjacency;
pub(crate) mod get_biadjacency_weighted;
pub(crate) mod get_eids;
pub(crate) mod girth;
pub(crate) mod graphicality;
pub(crate) mod harmonic;
pub(crate) mod harmonic_cutoff;
pub(crate) mod harmonic_weighted;
pub(crate) mod hits;
pub(crate) mod is_acyclic;
pub(crate) mod is_biclique;
pub(crate) mod is_bipartite;
pub(crate) mod is_block;
pub(crate) mod is_bowtie_free;
pub(crate) mod is_bull_free;
pub(crate) mod is_c4_free;
pub(crate) mod is_c5_free;
pub(crate) mod is_cactus;
pub(crate) mod is_caterpillar;
pub(crate) mod is_claw_free;
pub(crate) mod is_clique;
pub(crate) mod is_cluster;
pub(crate) mod is_cograph;
pub(crate) mod is_complete;
pub(crate) mod is_complete_multipartite;
pub(crate) mod is_cycle;
pub(crate) mod is_dag;
pub(crate) mod is_diamond_free;
pub(crate) mod is_distance_hereditary;
pub(crate) mod is_forest;
pub(crate) mod is_gem_free;
pub(crate) mod is_geodetic;
pub(crate) mod is_lobster;
pub(crate) mod is_path;
pub(crate) mod is_paw_free;
pub(crate) mod is_pseudo_forest;
pub(crate) mod is_ptolemaic;
pub(crate) mod is_regular;
pub(crate) mod is_self_complementary;
pub(crate) mod is_semicomplete;
pub(crate) mod is_simple;
pub(crate) mod is_split;
pub(crate) mod is_star;
pub(crate) mod is_strongly_regular;
pub(crate) mod is_threshold;
pub(crate) mod is_tournament;
pub(crate) mod is_tree;
pub(crate) mod is_triangle_free;
pub(crate) mod is_trivially_perfect;
pub(crate) mod is_unicyclic;
pub(crate) mod is_wheel;
pub(crate) mod is_windmill;
pub(crate) mod joint_degree_distribution;
pub(crate) mod joint_degree_matrix;
pub(crate) mod joint_type_distribution;
pub(crate) mod knn;
pub(crate) mod laplacian;
pub(crate) mod list_triangles;
pub(crate) mod local_scan;
pub(crate) mod local_scan_k;
pub(crate) mod mean_distance_weighted;
pub(crate) mod multiplicity;
pub(crate) mod mutual;
pub(crate) mod neighborhood;
pub(crate) mod pagerank;
pub(crate) mod pagerank_linsys;
pub(crate) mod pagerank_weighted;
pub(crate) mod perfect;
pub(crate) mod personalized_pagerank;
pub(crate) mod power_law_fit;
pub(crate) mod reciprocity;
pub(crate) mod rich_club;
pub(crate) mod running_mean;
pub(crate) mod similarity;
pub(crate) mod sort_by_degree;
pub(crate) mod stochastic;
pub(crate) mod strength;
pub(crate) mod topological_sorting;
pub(crate) mod triangles;
pub(crate) mod trussness;
pub(crate) mod unfold_tree;

pub use adjacency::{AdjacencyType, LoopHandling, get_adjacency};
pub use are_adjacent::are_adjacent;
pub use assortativity::{assortativity_degree, assortativity_degree_directed};
pub use assortativity_nominal::assortativity_nominal;
pub use assortativity_values::assortativity;
pub use assortativity_weighted::{
    assortativity_degree_directed_weighted, assortativity_degree_weighted,
};
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
pub use closeness::closeness;
pub use closeness_cutoff::{ClosenessCutoffResult, closeness_cutoff};
pub use closeness_weighted::closeness_weighted;
pub use constraint::constraint;
pub use convergence_degree::{convergence_degree, convergence_degree_full};
pub use coreness::{CorenessMode, coreness, coreness_with_mode};
pub use degree::{DegreeMode, degree_sequence, max_degree, max_degree_vertex, min_degree};
pub use degree_correlation::degree_correlation_vector;
pub use degree_distribution::degree_distribution;
pub use ecc::ecc;
pub use edge_betweenness::edge_betweenness;
pub use edge_betweenness_cutoff::edge_betweenness_cutoff;
pub use edge_betweenness_subset::edge_betweenness_subset;
pub use edge_betweenness_weighted::edge_betweenness_weighted;
pub use edgelist::get_edgelist;
pub use efficiency::{average_local_efficiency, global_efficiency, local_efficiency};
pub use eigenvector::{
    EigenvectorMode, EigenvectorScores, eigenvector_centrality, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_full,
    eigenvector_centrality_weighted,
};
pub use get_biadjacency::{GetBiadjacencyResult, get_biadjacency_matrix};
pub use get_biadjacency_weighted::{GetBiadjacencyWeightedResult, get_biadjacency_weighted};
pub use get_eids::get_eids;
pub use girth::girth;
pub use graphicality::{EdgeTypeFilter, is_bigraphical, is_graphical};
pub use harmonic::harmonic_centrality;
pub use harmonic_cutoff::harmonic_centrality_cutoff;
pub use harmonic_weighted::harmonic_centrality_weighted;
pub use hits::{HitsScores, hub_and_authority_scores};
pub use is_acyclic::is_acyclic;
pub use is_biclique::is_biclique;
pub use is_bipartite::{BipartiteResult, is_bipartite};
pub use is_block::is_block_graph;
pub use is_bowtie_free::is_bowtie_free;
pub use is_bull_free::is_bull_free;
pub use is_c4_free::is_c4_free;
pub use is_c5_free::is_c5_free;
pub use is_cactus::is_cactus_graph;
pub use is_caterpillar::is_caterpillar;
pub use is_claw_free::is_claw_free;
pub use is_clique::{is_clique, is_independent_vertex_set};
pub use is_cluster::is_cluster_graph;
pub use is_cograph::is_cograph;
pub use is_complete::is_complete;
pub use is_complete_multipartite::is_complete_multipartite;
pub use is_cycle::is_cycle;
pub use is_dag::is_dag;
pub use is_diamond_free::is_diamond_free;
pub use is_distance_hereditary::is_distance_hereditary;
pub use is_forest::is_forest;
pub use is_gem_free::is_gem_free;
pub use is_geodetic::is_geodetic;
pub use is_lobster::is_lobster;
pub use is_path::is_path;
pub use is_paw_free::is_paw_free;
pub use is_pseudo_forest::is_pseudo_forest;
pub use is_ptolemaic::is_ptolemaic;
pub use is_regular::{is_regular, regularity};
pub use is_self_complementary::is_self_complementary;
pub use is_semicomplete::is_semicomplete;
pub use is_simple::{SimpleMode, is_simple, is_simple_with_mode};
pub use is_split::is_split_graph;
pub use is_star::is_star;
pub use is_strongly_regular::{StronglyRegularParams, is_strongly_regular};
pub use is_threshold::is_threshold_graph;
pub use is_tournament::is_tournament;
pub use is_tree::is_tree;
pub use is_triangle_free::is_triangle_free;
pub use is_trivially_perfect::is_trivially_perfect;
pub use is_unicyclic::is_unicyclic;
pub use is_wheel::is_wheel;
pub use is_windmill::is_windmill;
pub use joint_degree_distribution::joint_degree_distribution;
pub use joint_degree_matrix::joint_degree_matrix;
pub use joint_type_distribution::joint_type_distribution;
pub use knn::avg_nearest_neighbor_degree;
pub use laplacian::{LaplacianNormalization, get_laplacian};
pub use list_triangles::list_triangles;
pub use local_scan::{
    local_scan_0, local_scan_0_them, local_scan_1, local_scan_1_ecount, local_scan_1_ecount_them,
    local_scan_subset_ecount,
};
pub use local_scan_k::{local_scan_k, local_scan_k_ecount, local_scan_k_ecount_them};
pub use mean_distance_weighted::mean_distance_weighted;
pub use multiplicity::{
    count_loops, count_multiple, count_multiple_1, has_loop, has_multiple, is_loop, is_multiple,
};
pub use mutual::{count_mutual, has_mutual, is_mutual};
pub use neighborhood::{
    NeighborhoodMode, neighborhood, neighborhood_graphs, neighborhood_graphs_with_mode,
    neighborhood_size, neighborhood_size_with_mode, neighborhood_with_mode,
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
pub use rich_club::rich_club_sequence;
pub use running_mean::{expand_path_to_pairs, running_mean};
pub use similarity::{
    bibcoupling, cocitation, similarity_dice, similarity_dice_es, similarity_dice_pairs,
    similarity_inverse_log_weighted, similarity_inverse_log_weighted_pairs, similarity_jaccard,
    similarity_jaccard_es, similarity_jaccard_pairs,
};
pub use sort_by_degree::{SortOrder, sort_vertices_by_degree};
pub use stochastic::get_stochastic;
pub use strength::{StrengthMode, diversity, strength, strength_with_mode};
pub use topological_sorting::topological_sorting;
pub use triangles::{
    TransitivityMode, count_adjacent_triangles, count_triangles, transitivity_avglocal_undirected,
    transitivity_local_undirected, transitivity_undirected,
};
pub use trussness::trussness;
pub use unfold_tree::{UnfoldTreeResult, unfold_tree};
