#![allow(
    clippy::needless_pass_by_value,
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::wildcard_imports
)]

mod centrality;
mod community;
mod core;
mod flow;
mod generators;
mod io;
mod isomorphism;
mod layout;
mod operators;
mod paths;
mod predicates;

use rust_igraph::{
    // batch 11
    AdjacencyMode,
    AdjacencyType,
    BfsMode,
    BipartiteMode,
    ChungLuVariant,
    ConnectednessMode,
    CorenessMode,
    DegreeMode,
    DhParams,
    DijkstraMode,
    DominatorMode,
    DrlOptions,
    EccMode,
    FasAlgorithm,
    FrParams,
    FvsAlgorithm,
    GemParams,
    Graph,
    GraphoptParams,
    GreedyColoringHeuristic,
    KkParams,
    LaplacianNormalization,
    LglParams,
    LoopHandling,
    LoopsMode,
    MstAlgorithm,
    RtMode,
    SimpleCycleMode,
    SimpleMode,
    SimplePathMode,
    SortOrder,
    StarMode,
    SubcomponentMode,
    SugiyamaParams,
    ToDirectedMode,
    ToUndirectedMode,
    TreeMode,
    VconnNei,
    VertexId,
    VoronoiTiebreaker,
    WheelMode,
    adhesion,
    adjacency,
    all_minimal_st_separators,
    all_simple_paths,
    all_st_cuts,
    all_st_mincuts,
    are_adjacent,
    articulation_points,
    // batch 8
    assortativity,
    assortativity_degree,
    assortativity_nominal,
    asymmetric_preference_game,
    atlas,
    automorphism_group,
    average_local_efficiency,
    avg_nearest_neighbor_degree,
    barabasi_game_bag,
    bellman_ford_distances,
    betweenness,
    betweenness_cutoff,
    betweenness_weighted,
    bfs,
    bfs_simple,
    bfs_tree,
    biadjacency,
    bibcoupling,
    biconnected_components,
    bipartite_game_gnm,
    bipartite_game_gnp,
    bipartite_projection_size,
    bond_percolation,
    bridges,
    canonical_permutation,
    cartesian_product,
    chromatic_number_upper_bound,
    chung_lu_game,
    circulant,
    clique_number,
    clique_size_hist,
    cliques,
    closeness,
    closeness_weighted,
    cocitation,
    cohesion,
    cohesive_blocks,
    community_voronoi,
    complementer,
    compose,
    connect_neighborhood,
    connected_components,
    constraint,
    contract_vertices,
    convergence_degree,
    convex_hull_2d,
    coreness,
    coreness_with_mode,
    correlated_game,
    correlated_pair_game,
    count_adjacent_triangles,
    count_automorphisms,
    count_loops,
    count_mutual,
    count_reachable,
    count_triangles,
    cut_value,
    cycle_graph,
    de_bruijn,
    decompose,
    degeneracy,
    degree_correlation_vector,
    degree_distribution,
    degree_sequence,
    // batch 9
    degree_sequence_game_configuration,
    delaunay_graph,
    density,
    dfs,
    diameter,
    // batch 7
    difference,
    dijkstra_distances,
    disjoint_union,
    // batch 10
    disjoint_union_many,
    distances,
    dominator_tree,
    eccentricity,
    edge_betweenness,
    edge_betweenness_community,
    edge_betweenness_community_weighted,
    edge_betweenness_cutoff,
    edge_betweenness_weighted,
    edge_connectivity,
    edge_disjoint_paths,
    eigenvector_centrality,
    erdos_renyi_gnm,
    erdos_renyi_gnp,
    eulerian_cycle,
    eulerian_path,
    even_tarjan_reduction,
    expand_path_to_pairs,
    extended_chordal_ring,
    famous,
    famous_names,
    fast_greedy_modularity,
    feedback_arc_set,
    feedback_vertex_set,
    find_cycle,
    floyd_warshall_distances,
    fluid_communities,
    forest_fire_game,
    from_prufer,
    full_bipartite,
    full_citation,
    full_graph,
    fundamental_cycles,
    gabriel_graph,
    generalized_petersen,
    get_adjacency,
    get_edgelist,
    get_laplacian,
    get_shortest_path,
    get_stochastic,
    girth,
    global_efficiency,
    gomory_hu_tree,
    graph_center,
    graph_power,
    grg_game,
    grg_game_with_coords,
    growing_random_game,
    hamming,
    harmonic_centrality,
    harmonic_centrality_cutoff,
    harmonic_centrality_weighted,
    has_mutual,
    hexagonal_lattice,
    hrg_consensus,
    hrg_create,
    hrg_fit,
    hrg_game,
    hrg_predict,
    hrg_sample,
    hsbm_game,
    hsbm_list_game,
    hub_and_authority_scores,
    hypercube,
    independence_number,
    induced_subgraph,
    infomap,
    intersection,
    intersection_many,
    invert_permutation,
    is_acyclic,
    // batch 6
    is_apex_forest,
    is_apex_tree,
    is_banner_free,
    is_biconnected,
    is_bipartite,
    is_biregular,
    is_block_graph,
    is_bowtie_free,
    is_bull_free,
    is_c4_free,
    is_c5_free,
    is_cactus_graph,
    is_caterpillar,
    is_chain_graph,
    is_chordal_bipartite,
    is_claw_free,
    is_clique,
    is_cluster_graph,
    is_co_bipartite,
    is_co_chordal,
    is_cograph,
    is_complete,
    is_complete_bipartite,
    is_complete_multipartite,
    is_connected,
    is_cricket_free,
    is_cubic,
    is_cycle,
    is_dag,
    is_dart_free,
    is_diamond_free,
    is_distance_hereditary,
    is_dominating_set,
    is_edge_cover,
    is_eulerian,
    is_forest,
    is_fork_free,
    is_gem_free,
    is_geodetic,
    is_house_free,
    is_independent_vertex_set,
    is_k_degenerate,
    is_lobster,
    is_mutual,
    is_net_free,
    is_outerplanar,
    is_p5_free,
    is_path,
    is_paw_free,
    is_perfect,
    is_planar,
    is_proper_interval,
    is_pseudo_forest,
    is_ptolemaic,
    is_regular,
    is_same_graph,
    is_self_complementary,
    is_semicomplete,
    is_series_parallel,
    is_simple,
    is_simple_with_mode,
    is_spider,
    is_split_graph,
    is_star,
    is_strongly_chordal,
    is_threshold_graph,
    is_tournament,
    is_tree,
    is_triangle_free,
    is_unicyclic,
    is_vertex_cover,
    is_weakly_chordal,
    is_well_covered,
    is_wheel,
    is_windmill,
    isomorphic,
    isomorphic_bliss,
    johnson_distances,
    join,
    joint_degree_distribution,
    k_regular_game,
    k_shortest_paths,
    kary_tree,
    katz_centrality,
    kautz,
    label_propagation,
    layout_bipartite,
    layout_circle,
    layout_davidson_harel,
    layout_drl,
    layout_drl_3d,
    layout_fruchterman_reingold,
    layout_gem,
    layout_graphopt,
    layout_grid,
    layout_kamada_kawai,
    layout_lgl,
    layout_mds,
    layout_random,
    layout_reingold_tilford,
    layout_star,
    layout_sugiyama,
    lcf,
    leading_eigenvector,
    leiden,
    line_graph,
    linegraph,
    list_triangles,
    local_efficiency,
    louvain,
    max_degree,
    max_flow,
    max_flow_value,
    maximal_cliques,
    maximum_cut,
    maximum_independent_set,
    mean_degree,
    mean_distance,
    mean_distance_weighted,
    min_degree,
    mincut,
    mincut_value,
    minimum_cycle_basis,
    minimum_dominating_set,
    minimum_edge_cover,
    minimum_spanning_tree,
    minimum_vertex_cover,
    modularity,
    modularity_matrix,
    mycielski_graph,
    mycielskian,
    nearest_neighbor_graph,
    neighborhood,
    pagerank,
    pagerank_weighted,
    path_graph,
    path_length_hist,
    permute_vertices,
    personalized_pagerank,
    power_law_fit,
    preference_game,
    radius,
    random_spanning_tree,
    random_walk,
    reachability_matrix,
    read_dl,
    read_dot,
    read_edgelist,
    read_gml,
    read_graphml,
    read_leda,
    read_lgl,
    read_ncol,
    read_pajek,
    realize_directed_degree_sequence,
    recent_degree_game,
    reciprocity,
    regular_tree,
    regularity,
    relative_neighborhood_graph,
    reverse,
    reverse_edges,
    rewire,
    rich_club_sequence,
    ring_graph,
    running_mean,
    satisfies_dirac,
    satisfies_ore,
    sbm_game,
    similarity_dice,
    similarity_jaccard,
    simple_cycles,
    simple_interconnected_islands_game,
    simplify,
    sir,
    solve_lsap,
    sort_vertices_by_degree,
    spanner,
    spatial_edge_lengths,
    spinglass,
    square_lattice,
    st_edge_connectivity,
    st_mincut,
    st_mincut_value,
    st_vertex_connectivity,
    star_graph,
    static_fitness_game,
    static_power_law_game,
    strength,
    strongly_connected_components,
    subcomponent,
    subisomorphic,
    symmetric_tree,
    to_directed,
    to_prufer,
    to_undirected,
    topological_sorting,
    transitive_closure,
    transitivity_undirected,
    tree_from_parent_vector,
    tree_game_lerw,
    tree_game_prufer,
    triad_census,
    triangular_lattice,
    trussness,
    turan,
    unfold_tree,
    union,
    union_many,
    vertex_coloring_greedy,
    vertex_connectivity,
    vertex_disjoint_paths,
    voronoi,
    walktrap,
    watts_strogatz_game,
    wheel_graph,
    write_dl,
    write_dot,
    write_edgelist,
    write_gml,
    write_graphml,
    write_leda,
    write_lgl,
    write_ncol,
    write_pajek,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct BfsResult {
    order: Vec<u32>,
}

#[derive(Serialize)]
struct DijkstraResult {
    distances: Vec<f64>,
}

#[derive(Serialize)]
struct PageRankResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct LouvainOutput {
    membership: Vec<u32>,
    modularity: f64,
}

#[derive(Serialize)]
struct BetweennessResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct ComponentsResult {
    membership: Vec<u32>,
    count: u32,
}

#[derive(Serialize)]
struct InfomapOutput {
    membership: Vec<u32>,
    codelength: f64,
}

#[derive(Serialize)]
struct SpinglassOutput {
    membership: Vec<u32>,
    modularity: f64,
    nb_clusters: u32,
}

#[derive(Serialize)]
struct ClosenessResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct DfsResult {
    order: Vec<u32>,
}

#[derive(Serialize)]
struct LabelPropOutput {
    membership: Vec<u32>,
    nb_clusters: u32,
}

#[derive(Serialize)]
struct WalktrapOutput {
    membership: Vec<u32>,
    nb_clusters: u32,
    modularity: f64,
}

#[derive(Serialize)]
struct LeidenOutput {
    membership: Vec<u32>,
    quality: f64,
    nb_clusters: u32,
}

#[derive(Serialize)]
struct FastGreedyOutput {
    membership: Vec<u32>,
    nb_clusters: u32,
    modularity: f64,
}

#[derive(Serialize)]
struct LeadingEigenvectorOutput {
    membership: Vec<u32>,
    modularity: f64,
}

#[derive(Serialize)]
struct EdgeBetweennessOutput {
    membership: Vec<u32>,
    nb_clusters: u32,
}

#[derive(Serialize)]
struct FluidOutput {
    membership: Vec<u32>,
    nb_clusters: u32,
}

#[derive(Serialize)]
struct HitsOutput {
    hub: Vec<f64>,
    authority: Vec<f64>,
}

#[derive(Serialize)]
struct LayoutResult {
    coords: Vec<[f64; 2]>,
}

#[derive(Serialize)]
struct GraphStatsResult {
    vcount: u32,
    ecount: u32,
    is_directed: bool,
    is_connected: bool,
    diameter: Option<u32>,
    girth: Option<u32>,
    triangles: u64,
    is_bipartite: bool,
}

#[derive(Serialize)]
struct MaxFlowResult {
    value: f64,
}

#[derive(Serialize)]
struct ArticulationResult {
    vertices: Vec<u32>,
}

#[derive(Serialize)]
struct DegreeResult {
    degrees: Vec<u32>,
}

#[derive(Serialize)]
struct SccResult {
    membership: Vec<u32>,
    count: u32,
}

#[derive(Serialize)]
struct BridgesResult {
    edges: Vec<[u32; 2]>,
    count: u32,
}

#[derive(Serialize)]
struct ColoringResult {
    colors: Vec<u32>,
    chromatic: u32,
}

#[derive(Serialize)]
struct TopoSortResult {
    order: Vec<u32>,
}

#[derive(Serialize)]
struct TransitivityResult {
    value: f64,
}

#[derive(Serialize)]
struct EdgeBetweennessResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct TriadCensusResult {
    counts: Vec<f64>,
}

#[derive(Serialize)]
struct CanonicalResult {
    permutation: Vec<u32>,
}

#[derive(Serialize)]
struct AutomorphismResult {
    count: f64,
}

#[derive(Serialize)]
struct IsomorphismResult {
    isomorphic: bool,
    mapping: Vec<u32>,
}

#[derive(Serialize)]
struct DiameterResult {
    diameter: Option<u32>,
}

#[derive(Serialize)]
struct RandomWalkResult {
    vertices: Vec<u32>,
}

#[derive(Serialize)]
struct ShortestPathResult {
    path: Vec<u32>,
}

#[derive(Serialize)]
struct CorenessResult {
    cores: Vec<u32>,
}

#[derive(Serialize)]
struct EccentricityResult {
    values: Vec<u32>,
}

#[derive(Serialize)]
struct DensityResult {
    density: Option<f64>,
}

#[derive(Serialize)]
struct RadiusResult {
    radius: Option<u32>,
}

#[derive(Serialize)]
struct MeanDistanceResult {
    mean_distance: Option<f64>,
}

#[derive(Serialize)]
struct MeanDegreeResult {
    mean_degree: Option<f64>,
}

#[derive(Serialize)]
struct AssortativityResult {
    assortativity: Option<f64>,
}

#[derive(Serialize)]
struct ConstraintResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct ReciprocityResult {
    reciprocity: Option<f64>,
}

#[derive(Serialize)]
struct BoolResult {
    value: bool,
}

#[derive(Serialize)]
#[allow(clippy::struct_excessive_bools)]
struct GraphPropertiesResult {
    is_tree: bool,
    is_forest: bool,
    is_dag: bool,
    is_acyclic: bool,
    is_complete: bool,
    is_biconnected: bool,
    is_bipartite: bool,
    is_connected: bool,
    is_tournament: bool,
    is_cubic: bool,
    is_cycle: bool,
    is_path: bool,
    is_star: bool,
    is_wheel: bool,
    is_perfect: bool,
    is_triangle_free: bool,
    is_outerplanar: bool,
    is_planar: bool,
}

#[derive(Serialize)]
struct AutomorphismGroupResult {
    generators: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct DistancesResult {
    distances: Vec<Option<u32>>,
}

#[derive(Serialize)]
struct FloydWarshallResult {
    matrix: Vec<Vec<f64>>,
}

#[derive(Serialize)]
struct CyclesResult {
    cycles: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct TrussnessResult {
    trussness: Vec<u32>,
}

#[derive(Serialize)]
struct TriangleListResult {
    triangles: Vec<[u32; 3]>,
    count: usize,
}

#[derive(Serialize)]
struct CliquesResult {
    cliques: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct ScalarU32Result {
    value: u32,
}

#[derive(Serialize)]
struct ScalarI64Result {
    value: i64,
}

#[derive(Serialize)]
struct MstResult {
    edges: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct StrengthResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct FeedbackArcSetResult {
    edges: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct WeightedDistancesResult {
    distances: Vec<Option<f64>>,
}

#[derive(Serialize)]
struct BiconnectedResult {
    count: u32,
    components: Vec<Vec<u32>>,
}

#[derive(Serialize)]
struct BipartiteCheckResult {
    is_bipartite: bool,
    types: Vec<u32>,
}

#[derive(Serialize)]
struct EulerianCheckResult {
    has_path: bool,
    has_cycle: bool,
}

#[derive(Serialize)]
struct EulerianPathResult {
    edges: Vec<u32>,
    exists: bool,
}

#[derive(Serialize)]
struct MaxCutOutput {
    partition: Vec<bool>,
    cut_value: usize,
}

#[derive(Serialize)]
struct EfficiencyResult {
    value: Option<f64>,
}

#[derive(Serialize)]
struct LocalEfficiencyResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct CohesiveBlocksResult {
    blocks: Vec<Vec<u32>>,
    cohesion: Vec<i64>,
    count: usize,
}

#[derive(Serialize)]
struct SimplePathsResult {
    paths: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct FindCycleResult {
    vertices: Vec<u32>,
    edges: Vec<u32>,
    found: bool,
}

#[derive(Serialize)]
struct KnnResult {
    scores: Vec<Option<f64>>,
}

#[derive(Serialize)]
struct ConvergenceDegreeResult {
    scores: Vec<f64>,
}

#[derive(Serialize)]
struct SimilarityMatrixResult {
    matrix: Vec<Vec<f64>>,
    size: usize,
}

#[derive(Serialize)]
struct VoronoiCommunityResult {
    membership: Vec<u32>,
    generators: Vec<u32>,
    modularity: Option<f64>,
}

#[derive(Serialize)]
struct GraphCenterResult {
    vertices: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct NeighborhoodResult {
    neighborhoods: Vec<Vec<u32>>,
}

#[derive(Serialize)]
struct KShortestPathsResult {
    paths: Vec<KPathEntry>,
    count: usize,
}

#[derive(Serialize)]
struct KPathEntry {
    vertices: Vec<u32>,
    weight: f64,
}

#[derive(Serialize)]
struct SeparatorsResult {
    separators: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct ClusteringCoeffResult {
    scores: Vec<Option<f64>>,
}

#[derive(Serialize)]
struct AveragePathLengthResult {
    value: Option<f64>,
}

#[derive(Serialize)]
struct LaplacianResult {
    matrix: Vec<Vec<f64>>,
    size: usize,
}

#[derive(Serialize)]
struct HrgTreeResult {
    size: u32,
    left: Vec<i32>,
    right: Vec<i32>,
    prob: Vec<f64>,
    vertices: Vec<i32>,
    edges: Vec<i32>,
}

#[derive(Serialize)]
struct HrgPrediction {
    from: u32,
    to: u32,
    probability: f64,
}

#[derive(Serialize)]
struct HrgPredictResult {
    predictions: Vec<HrgPrediction>,
    count: usize,
}

#[derive(Serialize)]
struct HrgConsensusResult {
    parents: Vec<i32>,
    weights: Vec<f64>,
}

#[derive(Serialize)]
struct GraphResult {
    vcount: u32,
    ecount: usize,
    edges: Vec<[u32; 2]>,
}

#[derive(Serialize)]
struct DegreeSequenceResult {
    degrees: Vec<u32>,
}

#[derive(Serialize)]
struct AdjacencyMatrixResult {
    matrix: Vec<Vec<f64>>,
    size: usize,
}

#[derive(Serialize)]
struct EdgelistResult {
    edges: Vec<[u32; 2]>,
    count: usize,
}

#[derive(Serialize)]
struct SubcomponentResult {
    vertices: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct GomoryHuResult {
    tree_edges: Vec<[u32; 2]>,
    flows: Vec<f64>,
}

#[derive(Serialize)]
struct RichClubResult {
    coefficients: Vec<f64>,
}

#[derive(Serialize)]
struct SpannerResult {
    edges: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct SugiyamaLayoutResult {
    coords: Vec<[f64; 2]>,
    dummy_coords: Vec<[f64; 2]>,
}

#[derive(Serialize)]
struct CountResult {
    count: usize,
}

#[derive(Serialize)]
struct MaxFlowDetailResult {
    value: f64,
    flow: Vec<f64>,
    cut: Vec<u32>,
}

#[derive(Serialize)]
struct StCutsResult {
    cuts: Vec<Vec<u32>>,
    partition1s: Vec<Vec<u32>>,
    count: usize,
}

#[derive(Serialize)]
struct FeedbackVertexSetResult {
    vertices: Vec<u32>,
    count: usize,
}

#[derive(Serialize)]
struct CutValueResult {
    value: usize,
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
