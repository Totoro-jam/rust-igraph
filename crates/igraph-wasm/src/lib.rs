#![allow(
    clippy::needless_pass_by_value,
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::wildcard_imports
)]

mod centrality;
mod community;
mod flow;
mod generators;
mod io;
mod isomorphism;
mod layout;
mod operators;
mod paths;
mod predicates;
#[path = "core.rs"]
mod wasm_core;

pub(crate) use rust_igraph::{
    AdjacencyMode, AdjacencyType, BfsMode, BipartiteMode, ChungLuVariant, ConnectednessMode,
    CorenessMode, DegreeMode, DhParams, DijkstraMode, DominatorMode, DrlOptions, EccMode,
    FasAlgorithm, FrParams, FvsAlgorithm, GemParams, Graph, GraphoptParams,
    GreedyColoringHeuristic, KkParams, LaplacianNormalization, LglParams, LoopHandling, LoopsMode,
    MstAlgorithm, RtMode, SimpleCycleMode, SimpleMode, SimplePathMode, SortOrder, StarMode,
    SubcomponentMode, SugiyamaParams, ToDirectedMode, ToUndirectedMode, TreeMode, VconnNei,
    VertexId, VoronoiTiebreaker, WheelMode, adhesion, adjacency, all_minimal_st_separators,
    all_simple_paths, all_st_cuts, all_st_mincuts, are_adjacent, articulation_points,
    assortativity, assortativity_degree, assortativity_nominal, asymmetric_preference_game, atlas,
    automorphism_group, average_local_efficiency, avg_nearest_neighbor_degree, barabasi_game_bag,
    bellman_ford_distances, betweenness, betweenness_cutoff, betweenness_weighted, bfs, bfs_simple,
    bfs_tree, biadjacency, bibcoupling, biconnected_components, bipartite_game_gnm,
    bipartite_game_gnp, bipartite_projection_size, bond_percolation, bridges,
    canonical_permutation, cartesian_product, chromatic_number_upper_bound, chung_lu_game,
    circulant, clique_number, clique_size_hist, cliques, closeness, closeness_weighted, cocitation,
    cohesion, cohesive_blocks, community_voronoi, complementer, compose, connect_neighborhood,
    connected_components, constraint, contract_vertices, convergence_degree, convex_hull_2d,
    coreness, coreness_with_mode, correlated_game, correlated_pair_game, count_adjacent_triangles,
    count_automorphisms, count_loops, count_mutual, count_reachable, count_triangles, cut_value,
    cycle_graph, de_bruijn, decompose, degeneracy, degree_correlation_vector, degree_distribution,
    degree_sequence, degree_sequence_game_configuration, delaunay_graph, density, dfs, diameter,
    difference, dijkstra_distances, disjoint_union, disjoint_union_many, distances, dominator_tree,
    eccentricity, edge_betweenness, edge_betweenness_community,
    edge_betweenness_community_weighted, edge_betweenness_cutoff, edge_betweenness_weighted,
    edge_connectivity, edge_disjoint_paths, eigenvector_centrality, erdos_renyi_gnm,
    erdos_renyi_gnp, eulerian_cycle, eulerian_path, even_tarjan_reduction, expand_path_to_pairs,
    extended_chordal_ring, famous, famous_names, fast_greedy_modularity, feedback_arc_set,
    feedback_vertex_set, find_cycle, floyd_warshall_distances, fluid_communities, forest_fire_game,
    from_prufer, full_bipartite, full_citation, full_graph, fundamental_cycles, gabriel_graph,
    generalized_petersen, get_adjacency, get_edgelist, get_laplacian, get_shortest_path,
    get_stochastic, girth, global_efficiency, gomory_hu_tree, graph_center, graph_power, grg_game,
    grg_game_with_coords, growing_random_game, hamming, harmonic_centrality,
    harmonic_centrality_cutoff, harmonic_centrality_weighted, has_mutual, hexagonal_lattice,
    hrg_consensus, hrg_create, hrg_fit, hrg_game, hrg_predict, hrg_sample, hsbm_game,
    hsbm_list_game, hub_and_authority_scores, hypercube, independence_number, induced_subgraph,
    infomap, intersection, intersection_many, invert_permutation, is_acyclic, is_apex_forest,
    is_apex_tree, is_banner_free, is_biconnected, is_bipartite, is_biregular, is_block_graph,
    is_bowtie_free, is_bull_free, is_c4_free, is_c5_free, is_cactus_graph, is_caterpillar,
    is_chain_graph, is_chordal_bipartite, is_claw_free, is_clique, is_cluster_graph,
    is_co_bipartite, is_co_chordal, is_cograph, is_complete, is_complete_bipartite,
    is_complete_multipartite, is_connected, is_cricket_free, is_cubic, is_cycle, is_dag,
    is_dart_free, is_diamond_free, is_distance_hereditary, is_dominating_set, is_edge_cover,
    is_eulerian, is_forest, is_fork_free, is_gem_free, is_geodetic, is_house_free,
    is_independent_vertex_set, is_k_degenerate, is_lobster, is_mutual, is_net_free, is_outerplanar,
    is_p5_free, is_path, is_paw_free, is_perfect, is_planar, is_proper_interval, is_pseudo_forest,
    is_ptolemaic, is_regular, is_same_graph, is_self_complementary, is_semicomplete,
    is_series_parallel, is_simple, is_simple_with_mode, is_spider, is_split_graph, is_star,
    is_strongly_chordal, is_threshold_graph, is_tournament, is_tree, is_triangle_free,
    is_unicyclic, is_vertex_cover, is_weakly_chordal, is_well_covered, is_wheel, is_windmill,
    isomorphic, isomorphic_bliss, johnson_distances, join, joint_degree_distribution,
    k_regular_game, k_shortest_paths, kary_tree, katz_centrality, kautz, label_propagation,
    layout_bipartite, layout_circle, layout_davidson_harel, layout_drl, layout_drl_3d,
    layout_fruchterman_reingold, layout_gem, layout_graphopt, layout_grid, layout_kamada_kawai,
    layout_lgl, layout_mds, layout_random, layout_reingold_tilford, layout_star, layout_sugiyama,
    lcf, leading_eigenvector, leiden, line_graph, linegraph, list_triangles, local_efficiency,
    louvain, max_degree, max_flow, max_flow_value, maximal_cliques, maximum_cut,
    maximum_independent_set, mean_degree, mean_distance, mean_distance_weighted, min_degree,
    mincut, mincut_value, minimum_cycle_basis, minimum_dominating_set, minimum_edge_cover,
    minimum_spanning_tree, minimum_vertex_cover, modularity, modularity_matrix, mycielski_graph,
    mycielskian, nearest_neighbor_graph, neighborhood, pagerank, pagerank_weighted, path_graph,
    path_length_hist, permute_vertices, personalized_pagerank, power_law_fit, preference_game,
    radius, random_spanning_tree, random_walk, reachability_matrix, read_dl, read_dot,
    read_edgelist, read_gml, read_graphml, read_leda, read_lgl, read_ncol, read_pajek,
    realize_directed_degree_sequence, recent_degree_game, reciprocity, regular_tree, regularity,
    relative_neighborhood_graph, reverse, reverse_edges, rewire, rich_club_sequence, ring_graph,
    running_mean, satisfies_dirac, satisfies_ore, sbm_game, similarity_dice, similarity_jaccard,
    simple_cycles, simple_interconnected_islands_game, simplify, sir, solve_lsap,
    sort_vertices_by_degree, spanner, spatial_edge_lengths, spinglass, square_lattice,
    st_edge_connectivity, st_mincut, st_mincut_value, st_vertex_connectivity, star_graph,
    static_fitness_game, static_power_law_game, strength, strongly_connected_components,
    subcomponent, subisomorphic, symmetric_tree, to_directed, to_prufer, to_undirected,
    topological_sorting, transitive_closure, transitivity_undirected, tree_from_parent_vector,
    tree_game_lerw, tree_game_prufer, triad_census, triangular_lattice, trussness, turan,
    unfold_tree, union, union_many, vertex_coloring_greedy, vertex_connectivity,
    vertex_disjoint_paths, voronoi, walktrap, watts_strogatz_game, wheel_graph, write_dl,
    write_dot, write_edgelist, write_gml, write_graphml, write_leda, write_lgl, write_ncol,
    write_pajek,
};
pub(crate) use serde::Serialize;
pub(crate) use wasm_bindgen::prelude::*;

// --- Serialization structs used across child modules ---

#[derive(Serialize)]
pub(crate) struct BfsResult {
    pub(crate) order: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct DijkstraResult {
    pub(crate) distances: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct PageRankResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct LouvainOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) modularity: f64,
}

#[derive(Serialize)]
pub(crate) struct BetweennessResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct ComponentsResult {
    pub(crate) membership: Vec<u32>,
    pub(crate) count: u32,
}

#[derive(Serialize)]
pub(crate) struct InfomapOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) codelength: f64,
}

#[derive(Serialize)]
pub(crate) struct SpinglassOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) modularity: f64,
    pub(crate) nb_clusters: u32,
}

#[derive(Serialize)]
pub(crate) struct ClosenessResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct DfsResult {
    pub(crate) order: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct LabelPropOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) nb_clusters: u32,
}

#[derive(Serialize)]
pub(crate) struct WalktrapOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) nb_clusters: u32,
    pub(crate) modularity: f64,
}

#[derive(Serialize)]
pub(crate) struct LeidenOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) quality: f64,
    pub(crate) nb_clusters: u32,
}

#[derive(Serialize)]
pub(crate) struct FastGreedyOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) nb_clusters: u32,
    pub(crate) modularity: f64,
}

#[derive(Serialize)]
pub(crate) struct LeadingEigenvectorOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) modularity: f64,
}

#[derive(Serialize)]
pub(crate) struct EdgeBetweennessOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) nb_clusters: u32,
}

#[derive(Serialize)]
pub(crate) struct FluidOutput {
    pub(crate) membership: Vec<u32>,
    pub(crate) nb_clusters: u32,
}

#[derive(Serialize)]
pub(crate) struct HitsOutput {
    pub(crate) hub: Vec<f64>,
    pub(crate) authority: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct LayoutResult {
    pub(crate) coords: Vec<[f64; 2]>,
}

#[derive(Serialize)]
pub(crate) struct GraphStatsResult {
    pub(crate) vcount: u32,
    pub(crate) ecount: u32,
    pub(crate) is_directed: bool,
    pub(crate) is_connected: bool,
    pub(crate) diameter: Option<u32>,
    pub(crate) girth: Option<u32>,
    pub(crate) triangles: u64,
    pub(crate) is_bipartite: bool,
}

#[derive(Serialize)]
pub(crate) struct MaxFlowResult {
    pub(crate) value: f64,
}

#[derive(Serialize)]
pub(crate) struct ArticulationResult {
    pub(crate) vertices: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct DegreeResult {
    pub(crate) degrees: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct SccResult {
    pub(crate) membership: Vec<u32>,
    pub(crate) count: u32,
}

#[derive(Serialize)]
pub(crate) struct BridgesResult {
    pub(crate) edges: Vec<[u32; 2]>,
    pub(crate) count: u32,
}

#[derive(Serialize)]
pub(crate) struct ColoringResult {
    pub(crate) colors: Vec<u32>,
    pub(crate) chromatic: u32,
}

#[derive(Serialize)]
pub(crate) struct TopoSortResult {
    pub(crate) order: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct TransitivityResult {
    pub(crate) value: f64,
}

#[derive(Serialize)]
pub(crate) struct EdgeBetweennessResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct TriadCensusResult {
    pub(crate) counts: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct CanonicalResult {
    pub(crate) permutation: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct AutomorphismResult {
    pub(crate) count: f64,
}

#[derive(Serialize)]
pub(crate) struct IsomorphismResult {
    pub(crate) isomorphic: bool,
    pub(crate) mapping: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct DiameterResult {
    pub(crate) diameter: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct RandomWalkResult {
    pub(crate) vertices: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct ShortestPathResult {
    pub(crate) path: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct CorenessResult {
    pub(crate) cores: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct EccentricityResult {
    pub(crate) values: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct DensityResult {
    pub(crate) density: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct RadiusResult {
    pub(crate) radius: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct MeanDistanceResult {
    pub(crate) mean_distance: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct MeanDegreeResult {
    pub(crate) mean_degree: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct AssortativityResult {
    pub(crate) assortativity: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct ConstraintResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct ReciprocityResult {
    pub(crate) reciprocity: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct BoolResult {
    pub(crate) value: bool,
}

#[derive(Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct GraphPropertiesResult {
    pub(crate) is_tree: bool,
    pub(crate) is_forest: bool,
    pub(crate) is_dag: bool,
    pub(crate) is_acyclic: bool,
    pub(crate) is_complete: bool,
    pub(crate) is_biconnected: bool,
    pub(crate) is_bipartite: bool,
    pub(crate) is_connected: bool,
    pub(crate) is_tournament: bool,
    pub(crate) is_cubic: bool,
    pub(crate) is_cycle: bool,
    pub(crate) is_path: bool,
    pub(crate) is_star: bool,
    pub(crate) is_wheel: bool,
    pub(crate) is_perfect: bool,
    pub(crate) is_triangle_free: bool,
    pub(crate) is_outerplanar: bool,
    pub(crate) is_planar: bool,
}

#[derive(Serialize)]
pub(crate) struct AutomorphismGroupResult {
    pub(crate) generators: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct DistancesResult {
    pub(crate) distances: Vec<Option<u32>>,
}

#[derive(Serialize)]
pub(crate) struct FloydWarshallResult {
    pub(crate) matrix: Vec<Vec<f64>>,
}

#[derive(Serialize)]
pub(crate) struct CyclesResult {
    pub(crate) cycles: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct TrussnessResult {
    pub(crate) trussness: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct TriangleListResult {
    pub(crate) triangles: Vec<[u32; 3]>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct CliquesResult {
    pub(crate) cliques: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct ScalarU32Result {
    pub(crate) value: u32,
}

#[derive(Serialize)]
pub(crate) struct ScalarI64Result {
    pub(crate) value: i64,
}

#[derive(Serialize)]
pub(crate) struct MstResult {
    pub(crate) edges: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct StrengthResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct FeedbackArcSetResult {
    pub(crate) edges: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct WeightedDistancesResult {
    pub(crate) distances: Vec<Option<f64>>,
}

#[derive(Serialize)]
pub(crate) struct BiconnectedResult {
    pub(crate) count: u32,
    pub(crate) components: Vec<Vec<u32>>,
}

#[derive(Serialize)]
pub(crate) struct BipartiteCheckResult {
    pub(crate) is_bipartite: bool,
    pub(crate) types: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct EulerianCheckResult {
    pub(crate) has_path: bool,
    pub(crate) has_cycle: bool,
}

#[derive(Serialize)]
pub(crate) struct EulerianPathResult {
    pub(crate) edges: Vec<u32>,
    pub(crate) exists: bool,
}

#[derive(Serialize)]
pub(crate) struct MaxCutOutput {
    pub(crate) partition: Vec<bool>,
    pub(crate) cut_value: usize,
}

#[derive(Serialize)]
pub(crate) struct EfficiencyResult {
    pub(crate) value: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct LocalEfficiencyResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct CohesiveBlocksResult {
    pub(crate) blocks: Vec<Vec<u32>>,
    pub(crate) cohesion: Vec<i64>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct SimplePathsResult {
    pub(crate) paths: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct FindCycleResult {
    pub(crate) vertices: Vec<u32>,
    pub(crate) edges: Vec<u32>,
    pub(crate) found: bool,
}

#[derive(Serialize)]
pub(crate) struct KnnResult {
    pub(crate) scores: Vec<Option<f64>>,
}

#[derive(Serialize)]
pub(crate) struct ConvergenceDegreeResult {
    pub(crate) scores: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct SimilarityMatrixResult {
    pub(crate) matrix: Vec<Vec<f64>>,
    pub(crate) size: usize,
}

#[derive(Serialize)]
pub(crate) struct VoronoiCommunityResult {
    pub(crate) membership: Vec<u32>,
    pub(crate) generators: Vec<u32>,
    pub(crate) modularity: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct GraphCenterResult {
    pub(crate) vertices: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct NeighborhoodResult {
    pub(crate) neighborhoods: Vec<Vec<u32>>,
}

#[derive(Serialize)]
pub(crate) struct KShortestPathsResult {
    pub(crate) paths: Vec<KPathEntry>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct KPathEntry {
    pub(crate) vertices: Vec<u32>,
    pub(crate) weight: f64,
}

#[derive(Serialize)]
pub(crate) struct SeparatorsResult {
    pub(crate) separators: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct ClusteringCoeffResult {
    pub(crate) scores: Vec<Option<f64>>,
}

#[derive(Serialize)]
pub(crate) struct AveragePathLengthResult {
    pub(crate) value: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct LaplacianResult {
    pub(crate) matrix: Vec<Vec<f64>>,
    pub(crate) size: usize,
}

#[derive(Serialize)]
pub(crate) struct HrgTreeResult {
    pub(crate) size: u32,
    pub(crate) left: Vec<i32>,
    pub(crate) right: Vec<i32>,
    pub(crate) prob: Vec<f64>,
    pub(crate) vertices: Vec<i32>,
    pub(crate) edges: Vec<i32>,
}

#[derive(Serialize)]
pub(crate) struct HrgPrediction {
    pub(crate) from: u32,
    pub(crate) to: u32,
    pub(crate) probability: f64,
}

#[derive(Serialize)]
pub(crate) struct HrgPredictResult {
    pub(crate) predictions: Vec<HrgPrediction>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct HrgConsensusResult {
    pub(crate) parents: Vec<i32>,
    pub(crate) weights: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct GraphResult {
    pub(crate) vcount: u32,
    pub(crate) ecount: usize,
    pub(crate) edges: Vec<[u32; 2]>,
}

#[derive(Serialize)]
pub(crate) struct DegreeSequenceResult {
    pub(crate) degrees: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct AdjacencyMatrixResult {
    pub(crate) matrix: Vec<Vec<f64>>,
    pub(crate) size: usize,
}

#[derive(Serialize)]
pub(crate) struct EdgelistResult {
    pub(crate) edges: Vec<[u32; 2]>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct SubcomponentResult {
    pub(crate) vertices: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct GomoryHuResult {
    pub(crate) tree_edges: Vec<[u32; 2]>,
    pub(crate) flows: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct RichClubResult {
    pub(crate) coefficients: Vec<f64>,
}

#[derive(Serialize)]
pub(crate) struct SpannerResult {
    pub(crate) edges: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct SugiyamaLayoutResult {
    pub(crate) coords: Vec<[f64; 2]>,
    pub(crate) dummy_coords: Vec<[f64; 2]>,
}

#[derive(Serialize)]
pub(crate) struct CountResult {
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct MaxFlowDetailResult {
    pub(crate) value: f64,
    pub(crate) flow: Vec<f64>,
    pub(crate) cut: Vec<u32>,
}

#[derive(Serialize)]
pub(crate) struct StCutsResult {
    pub(crate) cuts: Vec<Vec<u32>>,
    pub(crate) partition1s: Vec<Vec<u32>>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct FeedbackVertexSetResult {
    pub(crate) vertices: Vec<u32>,
    pub(crate) count: usize,
}

#[derive(Serialize)]
pub(crate) struct CutValueResult {
    pub(crate) value: usize,
}

#[wasm_bindgen]
pub struct WasmGraph {
    inner: Graph,
}

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(directed: bool) -> Result<WasmGraph, JsError> {
        let g = Graph::new(0, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    /// Create from a flat array of edge pairs: [u0, v0, u1, v1, ...].
    #[wasm_bindgen(js_name = "fromEdges")]
    pub fn from_edges(edges_flat: &[u32], directed: bool) -> Result<WasmGraph, JsError> {
        let pairs: Vec<(VertexId, VertexId)> = edges_flat
            .chunks_exact(2)
            .map(|pair| (pair[0], pair[1]))
            .collect();
        let g =
            Graph::from_edges(&pairs, directed, None).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }
}
