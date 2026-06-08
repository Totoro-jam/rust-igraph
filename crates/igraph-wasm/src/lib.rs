#![allow(clippy::needless_pass_by_value, clippy::items_after_statements)]

use rust_igraph::{
    AdjacencyType, BfsMode, BipartiteMode, ChungLuVariant, ConnectednessMode, DegreeMode, DhParams,
    DijkstraMode, DominatorMode, DrlOptions, EccMode, FasAlgorithm, FrParams, FvsAlgorithm,
    GemParams, Graph, GraphoptParams, GreedyColoringHeuristic, KkParams, LaplacianNormalization,
    LglParams, LoopHandling, MstAlgorithm, RtMode, SimpleCycleMode, SimplePathMode, SortOrder,
    StarMode, SubcomponentMode, SugiyamaParams, ToDirectedMode, ToUndirectedMode, TreeMode,
    VconnNei, VertexId, WheelMode, adhesion, all_minimal_st_separators, all_simple_paths,
    all_st_cuts, are_adjacent, articulation_points, assortativity_degree, atlas,
    automorphism_group, average_local_efficiency, avg_nearest_neighbor_degree, barabasi_game_bag,
    bellman_ford_distances, betweenness, betweenness_weighted, bfs, bfs_simple, bfs_tree,
    bibcoupling, biconnected_components, bipartite_game_gnm, bipartite_game_gnp,
    bipartite_projection_size, bond_percolation, bridges, canonical_permutation, cartesian_product,
    chromatic_number_upper_bound, chung_lu_game, circulant, clique_number, clique_size_hist,
    cliques, closeness, closeness_weighted, cocitation, cohesion, cohesive_blocks,
    community_voronoi, complementer, compose, connect_neighborhood, connected_components,
    constraint, contract_vertices, convergence_degree, convex_hull_2d, coreness, correlated_game,
    correlated_pair_game, count_adjacent_triangles, count_automorphisms, count_loops, count_mutual,
    count_reachable, count_triangles, cut_value, cycle_graph, de_bruijn, decompose, degeneracy,
    degree_correlation_vector, degree_distribution, degree_sequence, delaunay_graph, density, dfs,
    diameter, dijkstra_distances, disjoint_union, distances, dominator_tree, eccentricity,
    edge_betweenness, edge_betweenness_community, edge_connectivity, edge_disjoint_paths,
    eigenvector_centrality, erdos_renyi_gnm, erdos_renyi_gnp, eulerian_cycle, eulerian_path,
    even_tarjan_reduction, famous, famous_names, fast_greedy_modularity, feedback_arc_set,
    feedback_vertex_set, find_cycle, floyd_warshall_distances, fluid_communities, from_prufer,
    full_bipartite, full_citation, full_graph, fundamental_cycles, generalized_petersen,
    get_adjacency, get_edgelist, get_laplacian, get_shortest_path, girth, global_efficiency,
    gomory_hu_tree, graph_center, graph_power, grg_game, harmonic_centrality, has_mutual,
    hrg_consensus, hrg_create, hrg_fit, hrg_game, hrg_predict, hrg_sample, hsbm_game,
    hub_and_authority_scores, hypercube, independence_number, induced_subgraph, infomap,
    intersection, invert_permutation, is_acyclic, is_biconnected, is_bipartite, is_clique,
    is_complete, is_connected, is_cubic, is_cycle, is_dag, is_dominating_set, is_edge_cover,
    is_eulerian, is_forest, is_independent_vertex_set, is_k_degenerate, is_mutual, is_outerplanar,
    is_path, is_perfect, is_planar, is_regular, is_simple, is_star, is_tournament, is_tree,
    is_triangle_free, is_vertex_cover, is_wheel, isomorphic, isomorphic_bliss, k_shortest_paths,
    kary_tree, katz_centrality, label_propagation, layout_bipartite, layout_circle,
    layout_davidson_harel, layout_drl, layout_fruchterman_reingold, layout_gem, layout_graphopt,
    layout_grid, layout_kamada_kawai, layout_lgl, layout_mds, layout_random,
    layout_reingold_tilford, layout_star, layout_sugiyama, leading_eigenvector, leiden, line_graph,
    list_triangles, local_efficiency, louvain, max_degree, max_flow, max_flow_value,
    maximal_cliques, maximum_cut, mean_degree, mean_distance, min_degree, mincut, mincut_value,
    minimum_cycle_basis, minimum_dominating_set, minimum_edge_cover, minimum_spanning_tree,
    minimum_vertex_cover, modularity, mycielskian, neighborhood, pagerank, path_graph,
    path_length_hist, permute_vertices, personalized_pagerank, power_law_fit, preference_game,
    radius, random_spanning_tree, random_walk, reciprocity, regularity, reverse, reverse_edges,
    rich_club_sequence, ring_graph, running_mean, similarity_dice, similarity_jaccard,
    simple_cycles, simplify, sir, sort_vertices_by_degree, spanner, spinglass, st_mincut,
    st_mincut_value, st_vertex_connectivity, star_graph, static_fitness_game,
    static_power_law_game, strength, strongly_connected_components, subcomponent, subisomorphic,
    to_directed, to_prufer, to_undirected, topological_sorting, transitive_closure,
    transitivity_undirected, tree_game_lerw, tree_game_prufer, triad_census, trussness,
    unfold_tree, union, vertex_coloring_greedy, vertex_connectivity, vertex_disjoint_paths,
    walktrap, watts_strogatz_game, wheel_graph, write_dot, write_gml, write_graphml,
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

    #[wasm_bindgen(js_name = "addEdge")]
    pub fn add_edge(&mut self, u: u32, v: u32) -> Result<(), JsError> {
        self.inner
            .add_edge(u, v)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn vcount(&self) -> u32 {
        self.inner.vcount()
    }

    pub fn ecount(&self) -> u32 {
        u32::try_from(self.inner.ecount()).unwrap_or(u32::MAX)
    }

    #[wasm_bindgen(js_name = "isDirected")]
    pub fn is_directed(&self) -> bool {
        self.inner.is_directed()
    }

    #[wasm_bindgen(js_name = "getEdges")]
    pub fn get_edges(&self) -> Vec<u32> {
        let mut result = Vec::with_capacity(self.inner.ecount().saturating_mul(2));
        for (u, v) in self.inner.edges() {
            result.push(u);
            result.push(v);
        }
        result
    }

    // --- Algorithms (return JSON strings) ---

    pub fn bfs(&self, root: u32) -> Result<String, JsError> {
        let order = bfs(&self.inner, root).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BfsResult { order };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn dijkstra(&self, source: u32, weights: &[f64]) -> Result<String, JsError> {
        let raw = dijkstra_distances(&self.inner, source, weights)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let distances: Vec<f64> = raw
            .into_iter()
            .map(|d| d.unwrap_or(f64::INFINITY))
            .collect();
        let result = DijkstraResult { distances };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn pagerank(&self) -> Result<String, JsError> {
        let scores = pagerank(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn louvain(&self) -> Result<String, JsError> {
        let lr = louvain(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LouvainOutput {
            membership: lr.membership,
            modularity: lr.modularity,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn betweenness(&self) -> Result<String, JsError> {
        let scores = betweenness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BetweennessResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "connectedComponents")]
    pub fn connected_components(&self) -> Result<String, JsError> {
        let cc = connected_components(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ComponentsResult {
            membership: cc.membership,
            count: cc.count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn infomap(&self) -> Result<String, JsError> {
        let r = infomap(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = InfomapOutput {
            membership: r.membership,
            codelength: r.codelength,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn spinglass(&self) -> Result<String, JsError> {
        let r = spinglass(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = SpinglassOutput {
            membership: r.membership,
            modularity: r.modularity,
            nb_clusters: r.nb_clusters,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn closeness(&self) -> Result<String, JsError> {
        let raw = closeness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let scores: Vec<f64> = raw.into_iter().map(|v| v.unwrap_or(0.0)).collect();
        let result = ClosenessResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "eigenvectorCentrality")]
    pub fn eigenvector_centrality(&self) -> Result<String, JsError> {
        let scores =
            eigenvector_centrality(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn dfs(&self, root: u32) -> Result<String, JsError> {
        let order = dfs(&self.inner, root).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DfsResult { order };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "labelPropagation")]
    pub fn label_propagation(&self) -> Result<String, JsError> {
        let r = label_propagation(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LabelPropOutput {
            membership: r.membership,
            nb_clusters: r.nb_clusters,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn walktrap(&self) -> Result<String, JsError> {
        let r = walktrap(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let best_mod = r.modularity.last().copied().unwrap_or(0.0);
        let result = WalktrapOutput {
            membership: r.membership,
            nb_clusters: r.nb_clusters,
            modularity: best_mod,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn leiden(&self) -> Result<String, JsError> {
        let r = leiden(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LeidenOutput {
            membership: r.membership,
            quality: r.quality,
            nb_clusters: r.nb_clusters,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "fastGreedy")]
    pub fn fast_greedy(&self) -> Result<String, JsError> {
        let r = fast_greedy_modularity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let best_mod = r.modularity.last().copied().unwrap_or(0.0);
        let result = FastGreedyOutput {
            membership: r.membership,
            nb_clusters: r.nb_clusters,
            modularity: best_mod,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "leadingEigenvector")]
    pub fn leading_eigenvector(&self) -> Result<String, JsError> {
        let r = leading_eigenvector(&self.inner, None, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LeadingEigenvectorOutput {
            membership: r.membership,
            modularity: r.modularity,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeBetweennessCommunity")]
    pub fn edge_betweenness_community(&self) -> Result<String, JsError> {
        let r =
            edge_betweenness_community(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EdgeBetweennessOutput {
            membership: r.membership,
            nb_clusters: r.nb_clusters,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "fluidCommunities")]
    pub fn fluid_communities(&self, k: u32) -> Result<String, JsError> {
        let r = fluid_communities(&self.inner, k).map_err(|e| JsError::new(&e.to_string()))?;
        let result = FluidOutput {
            membership: r.membership,
            nb_clusters: r.nb_clusters,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "harmonicCentrality")]
    pub fn harmonic_centrality(&self) -> Result<String, JsError> {
        let scores = harmonic_centrality(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hubAndAuthorityScores")]
    pub fn hub_and_authority_scores(&self) -> Result<String, JsError> {
        let r = hub_and_authority_scores(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = HitsOutput {
            hub: r.hub,
            authority: r.authority,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "katzCentrality")]
    pub fn katz_centrality(&self) -> Result<String, JsError> {
        let scores = katz_centrality(&self.inner, 0.01, 1.0, None, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "graphStats")]
    pub fn graph_stats(&self) -> Result<String, JsError> {
        let connected = is_connected(&self.inner, ConnectednessMode::Weak)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let diam = self
            .inner
            .diameter()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let g = girth(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let tri = count_triangles(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let bip = is_bipartite(&self.inner)
            .map_err(|e| JsError::new(&e.to_string()))?
            .is_bipartite;
        let result = GraphStatsResult {
            vcount: self.inner.vcount(),
            ecount: u32::try_from(self.inner.ecount()).unwrap_or(u32::MAX),
            is_directed: self.inner.is_directed(),
            is_connected: connected,
            diameter: diam,
            girth: g,
            triangles: tri,
            is_bipartite: bip,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maxFlow")]
    pub fn max_flow(&self, source: u32, target: u32) -> Result<String, JsError> {
        let value = max_flow_value(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "articulationPoints")]
    pub fn articulation_points(&self) -> Result<String, JsError> {
        let vertices =
            articulation_points(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ArticulationResult { vertices };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "degreeSequence")]
    pub fn degree_sequence(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let degrees = degree_sequence(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DegreeSequenceResult { degrees };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutFr")]
    pub fn layout_fr(&self, niter: u32) -> Result<String, JsError> {
        let params = FrParams {
            niter,
            ..FrParams::default()
        };
        let coords = layout_fruchterman_reingold(&self.inner, &params)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult {
            coords: coords.into_iter().map(|(x, y)| [x, y]).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stronglyConnectedComponents")]
    pub fn strongly_connected_components(&self) -> Result<String, JsError> {
        let cc =
            strongly_connected_components(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = SccResult {
            membership: cc.membership,
            count: cc.count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn bridges(&self) -> Result<String, JsError> {
        let edge_ids = bridges(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = edge_ids
            .iter()
            .map(|&eid| {
                let (s, t) = self.inner.edge(eid).unwrap_or((0, 0));
                [s, t]
            })
            .collect();
        let count = u32::try_from(edges.len()).unwrap_or(u32::MAX);
        let result = BridgesResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "vertexColoring")]
    pub fn vertex_coloring(&self) -> Result<String, JsError> {
        let colors = vertex_coloring_greedy(&self.inner, GreedyColoringHeuristic::DSatur)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let chromatic = colors.iter().copied().max().map_or(0, |c| c + 1);
        let result = ColoringResult { colors, chromatic };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "topologicalSort")]
    pub fn topological_sort(&self) -> Result<String, JsError> {
        let order = topological_sorting(&self.inner, DijkstraMode::Out)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = TopoSortResult { order };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn transitivity(&self) -> Result<String, JsError> {
        let value = transitivity_undirected(&self.inner)
            .map_err(|e| JsError::new(&e.to_string()))?
            .unwrap_or(0.0);
        let result = TransitivityResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeBetweenness")]
    pub fn edge_betweenness(&self) -> Result<String, JsError> {
        let scores = edge_betweenness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EdgeBetweennessResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "triadCensus")]
    pub fn triad_census(&self) -> Result<String, JsError> {
        let tc = triad_census(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = TriadCensusResult {
            counts: tc.counts.to_vec(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "canonicalPermutation")]
    pub fn canonical_permutation(&self) -> Result<String, JsError> {
        let perm =
            canonical_permutation(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CanonicalResult { permutation: perm };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countAutomorphisms")]
    pub fn count_automorphisms(&self) -> Result<String, JsError> {
        let count =
            count_automorphisms(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = AutomorphismResult { count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isomorphicBliss")]
    pub fn isomorphic_bliss(&self, other: &WasmGraph) -> Result<String, JsError> {
        let r = isomorphic_bliss(&self.inner, &other.inner, None, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = IsomorphismResult {
            isomorphic: r.iso,
            mapping: r.map12,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "diameter")]
    pub fn diameter(&self) -> Result<String, JsError> {
        let d = diameter(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DiameterResult { diameter: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "randomWalk")]
    pub fn random_walk(&self, start: u32, steps: u32, seed: u64) -> Result<String, JsError> {
        let (vertices, _edges) =
            random_walk(&self.inner, None, start, DijkstraMode::Out, steps, seed)
                .map_err(|e| JsError::new(&e.to_string()))?;
        let result = RandomWalkResult { vertices };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "shortestPath")]
    pub fn shortest_path(&self, source: u32, target: u32) -> Result<String, JsError> {
        let sp = self
            .inner
            .shortest_path_to(source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ShortestPathResult { path: sp.vertices };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "erdosRenyi")]
    pub fn erdos_renyi(n: u32, p: f64, seed: u64) -> Result<WasmGraph, JsError> {
        let g =
            erdos_renyi_gnp(n, p, false, false, seed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "fullGraph")]
    pub fn full_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = full_graph(n, false, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "cycleGraph")]
    pub fn cycle_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = cycle_graph(n, false, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "ringGraph")]
    pub fn ring_graph(n: u32, circular: bool) -> Result<WasmGraph, JsError> {
        let g = ring_graph(n, false, false, circular).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "wattsStrogatz")]
    pub fn watts_strogatz(n: u32, k: u32, p: f64, seed: u64) -> Result<WasmGraph, JsError> {
        let g = watts_strogatz_game(n, k, p, false, false, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "barabasiAlbert")]
    pub fn barabasi_albert(n: u32, m: u32, seed: u64) -> Result<WasmGraph, JsError> {
        let g = barabasi_game_bag(n, m, false, false, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    // --- Layout algorithms ---

    #[wasm_bindgen(js_name = "layoutCircle")]
    pub fn layout_circle(&self) -> Result<String, JsError> {
        let coords = layout_circle(&self.inner, None);
        let result = LayoutResult {
            coords: coords.into_iter().map(|(x, y)| [x, y]).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutRandom")]
    pub fn layout_random(&self, seed: u64) -> Result<String, JsError> {
        let coords = layout_random(&self.inner, seed);
        let result = LayoutResult {
            coords: coords.into_iter().map(|(x, y)| [x, y]).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutGrid")]
    pub fn layout_grid(&self, width: i32) -> Result<String, JsError> {
        let coords = layout_grid(&self.inner, width);
        let result = LayoutResult {
            coords: coords.into_iter().map(|(x, y)| [x, y]).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutStar")]
    pub fn layout_star(&self, center: u32) -> Result<String, JsError> {
        let coords =
            layout_star(&self.inner, center, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult {
            coords: coords.into_iter().map(|(x, y)| [x, y]).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutKamadaKawai")]
    pub fn layout_kamada_kawai(&self) -> Result<String, JsError> {
        let n = self.inner.vcount() as usize;
        let params = KkParams::default_for(n);
        let coords = layout_kamada_kawai(&self.inner, None, &params, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Graph metrics ---

    pub fn coreness(&self) -> Result<String, JsError> {
        let cores = coreness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CorenessResult { cores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn eccentricity(&self) -> Result<String, JsError> {
        let values = eccentricity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EccentricityResult { values };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn density(&self) -> Result<String, JsError> {
        let d = density(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DensityResult { density: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn radius(&self) -> Result<String, JsError> {
        let r = radius(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = RadiusResult { radius: r };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "meanDistance")]
    pub fn mean_distance(&self) -> Result<String, JsError> {
        let md = mean_distance(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MeanDistanceResult { mean_distance: md };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "meanDegree")]
    pub fn mean_degree(&self) -> Result<String, JsError> {
        let md = mean_degree(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MeanDegreeResult { mean_degree: md };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "assortativityDegree")]
    pub fn assortativity_degree(&self) -> Result<String, JsError> {
        let a = assortativity_degree(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = AssortativityResult { assortativity: a };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn constraint(&self) -> Result<String, JsError> {
        let scores = constraint(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ConstraintResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn reciprocity(&self) -> Result<String, JsError> {
        let r = reciprocity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ReciprocityResult { reciprocity: r };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Property queries ---

    #[wasm_bindgen(js_name = "isTree")]
    pub fn is_tree(&self) -> Result<String, JsError> {
        let r =
            is_tree(&self.inner, DijkstraMode::Out).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: r.is_some() };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isForest")]
    pub fn is_forest(&self) -> Result<String, JsError> {
        let r =
            is_forest(&self.inner, DijkstraMode::Out).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: r.is_some() };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isDag")]
    pub fn is_dag(&self) -> Result<String, JsError> {
        let result = BoolResult {
            value: is_dag(&self.inner),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isAcyclic")]
    pub fn is_acyclic(&self) -> Result<String, JsError> {
        let result = BoolResult {
            value: is_acyclic(&self.inner),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isComplete")]
    pub fn is_complete(&self) -> Result<String, JsError> {
        let v = is_complete(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBiconnected")]
    pub fn is_biconnected(&self) -> Result<String, JsError> {
        let v = is_biconnected(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isTournament")]
    pub fn is_tournament(&self) -> Result<String, JsError> {
        let v = is_tournament(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCubic")]
    pub fn is_cubic(&self) -> Result<String, JsError> {
        let v = is_cubic(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCycle")]
    pub fn is_cycle(&self) -> Result<String, JsError> {
        let v = is_cycle(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPath")]
    pub fn is_path(&self) -> Result<String, JsError> {
        let v = is_path(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isStar")]
    pub fn is_star(&self) -> Result<String, JsError> {
        let v = is_star(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isWheel")]
    pub fn is_wheel(&self) -> Result<String, JsError> {
        let v = is_wheel(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPerfect")]
    pub fn is_perfect(&self) -> Result<String, JsError> {
        let v = is_perfect(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isTriangleFree")]
    pub fn is_triangle_free(&self) -> Result<String, JsError> {
        let v = is_triangle_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isOuterplanar")]
    pub fn is_outerplanar(&self) -> Result<String, JsError> {
        let v = is_outerplanar(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPlanar")]
    pub fn is_planar(&self) -> Result<String, JsError> {
        let v = is_planar(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "graphProperties")]
    pub fn graph_properties(&self) -> Result<String, JsError> {
        let result = GraphPropertiesResult {
            is_tree: is_tree(&self.inner, DijkstraMode::Out)
                .map(|r| r.is_some())
                .unwrap_or(false),
            is_forest: is_forest(&self.inner, DijkstraMode::Out)
                .map(|r| r.is_some())
                .unwrap_or(false),
            is_dag: is_dag(&self.inner),
            is_acyclic: is_acyclic(&self.inner),
            is_complete: is_complete(&self.inner).unwrap_or(false),
            is_biconnected: is_biconnected(&self.inner).unwrap_or(false),
            is_bipartite: is_bipartite(&self.inner)
                .map(|r| r.is_bipartite)
                .unwrap_or(false),
            is_connected: is_connected(&self.inner, ConnectednessMode::Weak).unwrap_or(false),
            is_tournament: is_tournament(&self.inner).unwrap_or(false),
            is_cubic: is_cubic(&self.inner).unwrap_or(false),
            is_cycle: is_cycle(&self.inner).unwrap_or(false),
            is_path: is_path(&self.inner).unwrap_or(false),
            is_star: is_star(&self.inner).unwrap_or(false),
            is_wheel: is_wheel(&self.inner).unwrap_or(false),
            is_perfect: is_perfect(&self.inner).unwrap_or(false),
            is_triangle_free: is_triangle_free(&self.inner).unwrap_or(false),
            is_outerplanar: is_outerplanar(&self.inner).unwrap_or(false),
            is_planar: is_planar(&self.inner).unwrap_or(false),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Additional algorithms ---

    #[wasm_bindgen(js_name = "automorphismGroup")]
    pub fn automorphism_group(&self) -> Result<String, JsError> {
        let gens =
            automorphism_group(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let count = gens.len();
        let result = AutomorphismGroupResult {
            generators: gens,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "girth")]
    pub fn girth(&self) -> Result<String, JsError> {
        let g = girth(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DiameterResult { diameter: g };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "distances")]
    pub fn distances(&self, source: u32) -> Result<String, JsError> {
        let dists = distances(&self.inner, source).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DistancesResult { distances: dists };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "floydWarshallDistances")]
    pub fn floyd_warshall_distances(&self) -> Result<String, JsError> {
        let mat = floyd_warshall_distances(&self.inner, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let matrix: Vec<Vec<f64>> = mat
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|d| d.unwrap_or(f64::INFINITY))
                    .collect()
            })
            .collect();
        let result = FloydWarshallResult { matrix };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "fundamentalCycles")]
    pub fn fundamental_cycles(&self) -> Result<String, JsError> {
        let raw = fundamental_cycles(&self.inner, None, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let cycles: Vec<Vec<u32>> = raw.into_iter().collect();
        let count = cycles.len();
        let result = CyclesResult { cycles, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "minimumCycleBasis")]
    pub fn minimum_cycle_basis(&self) -> Result<String, JsError> {
        let raw = minimum_cycle_basis(&self.inner, None, true)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let cycles: Vec<Vec<u32>> = raw.into_iter().collect();
        let count = cycles.len();
        let result = CyclesResult { cycles, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn trussness(&self) -> Result<String, JsError> {
        let t = trussness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = TrussnessResult { trussness: t };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "listTriangles")]
    pub fn list_triangles(&self) -> Result<String, JsError> {
        let tris = list_triangles(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let triangles: Vec<[u32; 3]> = tris.into_iter().map(|(a, b, c)| [a, b, c]).collect();
        let count = triangles.len();
        let result = TriangleListResult { triangles, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Graph operators ---

    #[wasm_bindgen(js_name = "simplify")]
    pub fn simplify(&self) -> Result<WasmGraph, JsError> {
        let g = simplify(&self.inner, true, true).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "lineGraph")]
    pub fn line_graph(&self) -> Result<WasmGraph, JsError> {
        let g = line_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "complement")]
    pub fn complement(&self) -> Result<WasmGraph, JsError> {
        let g = complementer(&self.inner, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // --- Cliques & independence ---

    #[wasm_bindgen(js_name = "cliqueNumber")]
    pub fn clique_number(&self) -> Result<String, JsError> {
        let v = clique_number(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maximalCliques")]
    pub fn maximal_cliques(&self) -> Result<String, JsError> {
        let c = maximal_cliques(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = c.len();
        let result = CliquesResult { cliques: c, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "independenceNumber")]
    pub fn independence_number(&self) -> Result<String, JsError> {
        let v = independence_number(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Connectivity ---

    #[wasm_bindgen(js_name = "vertexConnectivity")]
    pub fn vertex_connectivity(&self) -> Result<String, JsError> {
        let v = vertex_connectivity(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeConnectivity")]
    pub fn edge_connectivity(&self) -> Result<String, JsError> {
        let v = edge_connectivity(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Spanning tree ---

    #[wasm_bindgen(js_name = "minimumSpanningTree")]
    pub fn minimum_spanning_tree(&self, weights: Option<Vec<f64>>) -> Result<String, JsError> {
        let w = weights.as_deref();
        let edges = minimum_spanning_tree(&self.inner, w, MstAlgorithm::Prim)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = MstResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Weighted degree (strength) ---

    pub fn strength(&self, weights: Vec<f64>) -> Result<String, JsError> {
        let s = strength(&self.inner, &weights).map_err(|e| JsError::new(&e.to_string()))?;
        let result = StrengthResult { scores: s };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Feedback arc set ---

    #[wasm_bindgen(js_name = "feedbackArcSet")]
    pub fn feedback_arc_set(&self, weights: Option<Vec<f64>>) -> Result<String, JsError> {
        let w = weights.as_deref();
        let edges = feedback_arc_set(&self.inner, w, FasAlgorithm::EadesLinSmyth)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = FeedbackArcSetResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Weighted shortest paths ---

    #[wasm_bindgen(js_name = "bellmanFordDistances")]
    pub fn bellman_ford_distances(
        &self,
        source: u32,
        weights: Vec<f64>,
    ) -> Result<String, JsError> {
        let d = bellman_ford_distances(&self.inner, source, &weights)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = WeightedDistancesResult { distances: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Weighted centrality ---

    #[wasm_bindgen(js_name = "closenessWeighted")]
    pub fn closeness_weighted(&self, weights: Vec<f64>) -> Result<String, JsError> {
        let c =
            closeness_weighted(&self.inner, &weights).map_err(|e| JsError::new(&e.to_string()))?;
        let scores: Vec<f64> = c.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
        let result = ClosenessResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "betweennessWeighted")]
    pub fn betweenness_weighted(&self, weights: Vec<f64>) -> Result<String, JsError> {
        let b = betweenness_weighted(&self.inner, &weights)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = BetweennessResult { scores: b };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Graph I/O export ---

    #[wasm_bindgen(js_name = "writeGml")]
    pub fn write_gml(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_gml(&self.inner, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeDot")]
    pub fn write_dot(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_dot(&self.inner, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeGraphml")]
    pub fn write_graphml(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_graphml(&self.inner, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Additional graph generators ---

    #[wasm_bindgen(js_name = "pathGraph")]
    pub fn path_graph(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = path_graph(n, directed, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "starGraph")]
    pub fn star_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = star_graph(n, StarMode::Undirected, 0).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "famousGraph")]
    pub fn famous_graph(name: &str) -> Result<WasmGraph, JsError> {
        let g = famous(name).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // --- Decompose & modularity ---

    pub fn decompose(&self) -> Result<Vec<WasmGraph>, JsError> {
        let graphs = decompose(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(graphs.into_iter().map(|g| WasmGraph { inner: g }).collect())
    }

    pub fn modularity(&self, membership: Vec<u32>) -> Result<String, JsError> {
        let m =
            modularity(&self.inner, &membership, 1.0).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DensityResult { density: m };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "degreeDistribution")]
    pub fn degree_distribution(&self) -> Result<String, JsError> {
        let d = degree_distribution(&self.inner, rust_igraph::DegreeMode::All)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = DegreeResult { degrees: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Biconnected components ---

    #[wasm_bindgen(js_name = "biconnectedComponents")]
    pub fn biconnected_components(&self) -> Result<String, JsError> {
        let bc = biconnected_components(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BiconnectedResult {
            count: bc.count,
            components: bc.components,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Bipartite check ---

    #[wasm_bindgen(js_name = "isBipartiteDetailed")]
    pub fn is_bipartite_detailed(&self) -> Result<String, JsError> {
        let bp = is_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BipartiteCheckResult {
            is_bipartite: bp.is_bipartite,
            types: bp.types.iter().map(|&b| u32::from(b)).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Eulerian ---

    #[wasm_bindgen(js_name = "isEulerian")]
    pub fn is_eulerian(&self) -> Result<String, JsError> {
        let e = is_eulerian(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EulerianCheckResult {
            has_path: e.has_path,
            has_cycle: e.has_cycle,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "eulerianPath")]
    pub fn eulerian_path(&self) -> Result<String, JsError> {
        let ep = eulerian_path(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EulerianPathResult {
            exists: ep.is_some(),
            edges: ep.unwrap_or_default(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "eulerianCycle")]
    pub fn eulerian_cycle(&self) -> Result<String, JsError> {
        let result = if let Ok(edges) = eulerian_cycle(&self.inner) {
            EulerianPathResult {
                exists: true,
                edges,
            }
        } else {
            EulerianPathResult {
                exists: false,
                edges: vec![],
            }
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Maximum cut ---

    #[wasm_bindgen(js_name = "maximumCut")]
    pub fn maximum_cut(&self) -> Result<String, JsError> {
        let mc = maximum_cut(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxCutOutput {
            partition: mc.partition,
            cut_value: mc.cut_value,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Mincut value ---

    #[wasm_bindgen(js_name = "mincutValue")]
    pub fn mincut_value(&self) -> Result<String, JsError> {
        let v = mincut_value(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Disjoint paths ---

    #[wasm_bindgen(js_name = "vertexDisjointPaths")]
    pub fn vertex_disjoint_paths(&self, source: u32, target: u32) -> Result<String, JsError> {
        let v = vertex_disjoint_paths(&self.inner, source, target)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeDisjointPaths")]
    pub fn edge_disjoint_paths(&self, source: u32, target: u32) -> Result<String, JsError> {
        let v = edge_disjoint_paths(&self.inner, source, target)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Efficiency ---

    #[wasm_bindgen(js_name = "globalEfficiency")]
    pub fn global_efficiency(&self) -> Result<String, JsError> {
        let v = global_efficiency(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EfficiencyResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "localEfficiency")]
    pub fn local_efficiency(&self) -> Result<String, JsError> {
        let v = local_efficiency(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LocalEfficiencyResult { scores: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Degeneracy ---

    pub fn degeneracy(&self) -> Result<String, JsError> {
        let v = degeneracy(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Find cycle ---

    #[wasm_bindgen(js_name = "findCycle")]
    pub fn find_cycle(&self) -> Result<String, JsError> {
        let c = find_cycle(&self.inner, rust_igraph::CycleMode::All)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = FindCycleResult {
            found: !c.vertices.is_empty(),
            vertices: c.vertices,
            edges: c.edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- All simple paths ---

    #[wasm_bindgen(js_name = "allSimplePaths")]
    pub fn all_simple_paths(&self, source: u32, target: u32) -> Result<String, JsError> {
        let targets = [target];
        let paths = all_simple_paths(
            &self.inner,
            source,
            Some(&targets),
            SimplePathMode::Out,
            0,
            -1,
            1000,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        let count = paths.len();
        let result = SimplePathsResult { paths, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Cohesive blocks ---

    #[wasm_bindgen(js_name = "cohesiveBlocks")]
    pub fn cohesive_blocks(&self) -> Result<String, JsError> {
        let cb = cohesive_blocks(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = cb.blocks.len();
        let result = CohesiveBlocksResult {
            blocks: cb.blocks,
            cohesion: cb.cohesion,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Average nearest-neighbor degree ---

    #[wasm_bindgen(js_name = "avgNearestNeighborDegree")]
    pub fn avg_nearest_neighbor_degree(&self) -> Result<String, JsError> {
        let knn =
            avg_nearest_neighbor_degree(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = KnnResult { scores: knn };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Chromatic number upper bound ---

    #[wasm_bindgen(js_name = "chromaticNumberUpperBound")]
    pub fn chromatic_number_upper_bound(&self) -> Result<String, JsError> {
        let val =
            chromatic_number_upper_bound(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: val };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Convergence degree ---

    #[wasm_bindgen(js_name = "convergenceDegree")]
    pub fn convergence_degree(&self) -> Result<String, JsError> {
        let scores = convergence_degree(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ConvergenceDegreeResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Similarity Jaccard ---

    #[wasm_bindgen(js_name = "similarityJaccard")]
    pub fn similarity_jaccard(&self) -> Result<String, JsError> {
        let flat = similarity_jaccard(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let n = self.inner.vcount() as usize;
        let matrix: Vec<Vec<f64>> = (0..n).map(|i| flat[i * n..(i + 1) * n].to_vec()).collect();
        let result = SimilarityMatrixResult { matrix, size: n };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Similarity Dice ---

    #[wasm_bindgen(js_name = "similarityDice")]
    pub fn similarity_dice(&self) -> Result<String, JsError> {
        let flat = similarity_dice(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let n = self.inner.vcount() as usize;
        let matrix: Vec<Vec<f64>> = (0..n).map(|i| flat[i * n..(i + 1) * n].to_vec()).collect();
        let result = SimilarityMatrixResult { matrix, size: n };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Community Voronoi ---

    #[wasm_bindgen(js_name = "communityVoronoi")]
    pub fn community_voronoi(&self) -> Result<String, JsError> {
        let cv = community_voronoi(&self.inner, None, None, DijkstraMode::Out, 1.0)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = VoronoiCommunityResult {
            membership: cv.membership,
            generators: cv.generators,
            modularity: cv.modularity,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Graph center ---

    #[wasm_bindgen(js_name = "graphCenter")]
    pub fn graph_center(&self) -> Result<String, JsError> {
        let mode = EccMode::All;
        let vertices = graph_center(&self.inner, mode).map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = GraphCenterResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Neighborhood ---

    #[wasm_bindgen(js_name = "neighborhood")]
    pub fn neighborhood(&self, order: i32) -> Result<String, JsError> {
        let neighborhoods =
            neighborhood(&self.inner, order).map_err(|e| JsError::new(&e.to_string()))?;
        let result = NeighborhoodResult { neighborhoods };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- K-shortest paths ---

    #[wasm_bindgen(js_name = "kShortestPaths")]
    pub fn k_shortest_paths(&self, source: u32, target: u32, k: usize) -> Result<String, JsError> {
        let m = self.inner.ecount();
        let weights = vec![1.0_f64; m];
        let kpaths = k_shortest_paths(&self.inner, source, target, &weights, k, DijkstraMode::Out)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = kpaths.len();
        let paths: Vec<KPathEntry> = kpaths
            .into_iter()
            .map(|p| KPathEntry {
                vertices: p.vertices,
                weight: p.weight,
            })
            .collect();
        let result = KShortestPathsResult { paths, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- All minimal S-T separators ---

    #[wasm_bindgen(js_name = "allMinimalStSeparators")]
    pub fn all_minimal_st_separators(&self) -> Result<String, JsError> {
        let seps =
            all_minimal_st_separators(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = seps.len();
        let result = SeparatorsResult {
            separators: seps,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Clustering coefficients ---

    #[wasm_bindgen(js_name = "clusteringCoefficients")]
    pub fn clustering_coefficients(&self) -> Result<String, JsError> {
        let scores = self
            .inner
            .clustering_coefficients()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ClusteringCoeffResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Average path length ---

    #[wasm_bindgen(js_name = "averagePathLength")]
    pub fn average_path_length(&self) -> Result<String, JsError> {
        let value = self
            .inner
            .average_path_length()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = AveragePathLengthResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Laplacian matrix ---

    #[wasm_bindgen(js_name = "getLaplacian")]
    pub fn get_laplacian(&self, normalization: &str) -> Result<String, JsError> {
        let norm = match normalization {
            "symmetric" => LaplacianNormalization::Symmetric,
            "left" => LaplacianNormalization::Left,
            "right" => LaplacianNormalization::Right,
            _ => LaplacianNormalization::Unnormalized,
        };
        let mode = if self.inner.is_directed() {
            DegreeMode::Out
        } else {
            DegreeMode::All
        };
        let matrix = get_laplacian(&self.inner, mode, norm, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let size = matrix.len();
        let result = LaplacianResult { matrix, size };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- HRG (Hierarchical Random Graph) ---

    #[wasm_bindgen(js_name = "hrgFit")]
    pub fn hrg_fit(&self, steps: u64, seed: u64) -> Result<String, JsError> {
        let hrg =
            hrg_fit(&self.inner, None, steps, seed).map_err(|e| JsError::new(&e.to_string()))?;
        let result = HrgTreeResult {
            size: hrg.size(),
            left: hrg.left.clone(),
            right: hrg.right.clone(),
            prob: hrg.prob.clone(),
            vertices: hrg.vertices.clone(),
            edges: hrg.edges.clone(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hrgCreate")]
    pub fn hrg_create(&self, probs_flat: &[f64]) -> Result<String, JsError> {
        let hrg = hrg_create(&self.inner, probs_flat).map_err(|e| JsError::new(&e.to_string()))?;
        let result = HrgTreeResult {
            size: hrg.size(),
            left: hrg.left.clone(),
            right: hrg.right.clone(),
            prob: hrg.prob.clone(),
            vertices: hrg.vertices.clone(),
            edges: hrg.edges.clone(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hrgSample")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hrg_sample(&self, steps: u64, seed: u64) -> Result<String, JsError> {
        let hrg =
            hrg_fit(&self.inner, None, steps, seed).map_err(|e| JsError::new(&e.to_string()))?;
        let sampled = hrg_sample(&hrg, seed).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..sampled.ecount())
            .map(|eid| {
                let (u, v) = sampled.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: sampled.vcount(),
            ecount: sampled.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hrgGame")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hrg_game(&self, steps: u64, seed: u64) -> Result<String, JsError> {
        let hrg =
            hrg_fit(&self.inner, None, steps, seed).map_err(|e| JsError::new(&e.to_string()))?;
        let sampled =
            hrg_game(&hrg, seed.wrapping_add(1)).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..sampled.ecount())
            .map(|eid| {
                let (u, v) = sampled.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: sampled.vcount(),
            ecount: sampled.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hrgPredict")]
    pub fn hrg_predict(&self, num_samples: u64, seed: u64) -> Result<String, JsError> {
        let preds = hrg_predict(&self.inner, None, num_samples, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let predictions: Vec<HrgPrediction> = preds
            .iter()
            .map(|&(from, to, probability)| HrgPrediction {
                from,
                to,
                probability,
            })
            .collect();
        let count = predictions.len();
        let result = HrgPredictResult { predictions, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hrgConsensus")]
    pub fn hrg_consensus(&self, num_samples: u64, seed: u64) -> Result<String, JsError> {
        let (parents, weights) = hrg_consensus(&self.inner, None, num_samples, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = HrgConsensusResult { parents, weights };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Layout engines ---

    #[wasm_bindgen(js_name = "layoutDrl")]
    pub fn layout_drl(&self) -> Result<String, JsError> {
        let coords = layout_drl(&self.inner, None, &DrlOptions::default(), None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutGem")]
    pub fn layout_gem(&self) -> Result<String, JsError> {
        let coords = layout_gem(
            &self.inner,
            None,
            &GemParams::for_graph(self.inner.vcount()),
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutGraphopt")]
    pub fn layout_graphopt(&self) -> Result<String, JsError> {
        let coords = layout_graphopt(&self.inner, None, &GraphoptParams::default())
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutLgl")]
    pub fn layout_lgl(&self) -> Result<String, JsError> {
        let coords = layout_lgl(&self.inner, &LglParams::default())
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutMds")]
    pub fn layout_mds(&self) -> Result<String, JsError> {
        let coords = layout_mds(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutDavidsonHarel")]
    pub fn layout_davidson_harel(&self) -> Result<String, JsError> {
        let coords = layout_davidson_harel(&self.inner, None, &DhParams::for_graph(&self.inner))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutSugiyama")]
    pub fn layout_sugiyama(&self) -> Result<String, JsError> {
        let sug = layout_sugiyama(&self.inner, None, &SugiyamaParams::default())
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = SugiyamaLayoutResult {
            coords: sug.positions,
            dummy_coords: sug.dummy_positions,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutBipartite")]
    pub fn layout_bipartite(&self, types_flat: &[u8]) -> Result<String, JsError> {
        let types: Vec<bool> = types_flat.iter().map(|&t| t != 0).collect();
        let coords = layout_bipartite(&self.inner, &types, 1.0, 1.0, 100)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutReingoldTilford")]
    pub fn layout_reingold_tilford(&self, root: i32) -> Result<String, JsError> {
        let root_opt = if root < 0 {
            None
        } else {
            u32::try_from(root).ok()
        };
        let coords = layout_reingold_tilford(&self.inner, root_opt, RtMode::Out)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = LayoutResult { coords };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Graph transforms ---

    #[wasm_bindgen(js_name = "toDirected")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_directed(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "mutual" => ToDirectedMode::Mutual,
            _ => ToDirectedMode::Arbitrary,
        };
        let g = to_directed(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "toUndirected")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_undirected(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "collapse" => ToUndirectedMode::Collapse,
            "mutual" => ToUndirectedMode::Mutual,
            _ => ToUndirectedMode::Each,
        };
        let g = to_undirected(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "reverseGraph")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn reverse_graph(&self) -> Result<String, JsError> {
        let g = reverse(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "contractVertices")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn contract_vertices(&self, mapping: &[u32]) -> Result<String, JsError> {
        let g =
            contract_vertices(&self.inner, mapping).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "inducedSubgraph")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn induced_subgraph(&self, vids: &[u32]) -> Result<String, JsError> {
        let sub = induced_subgraph(&self.inner, vids).map_err(|e| JsError::new(&e.to_string()))?;
        let g = &sub.graph;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "permuteVertices")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn permute_vertices(&self, perm: &[u32]) -> Result<String, JsError> {
        let g = permute_vertices(&self.inner, perm).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|eid| {
                let (u, v) = g.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GraphResult {
            vcount: g.vcount(),
            ecount: g.ecount(),
            edges,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // --- Analytics ---

    #[wasm_bindgen(js_name = "personalizedPagerank")]
    pub fn personalized_pagerank(&self, reset: &[f64], damping: f64) -> Result<String, JsError> {
        let scores = personalized_pagerank(&self.inner, reset, damping)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maxDegree")]
    pub fn max_degree(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let value = max_degree(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "minDegree")]
    pub fn min_degree(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let value = min_degree(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSimple")]
    pub fn is_simple(&self) -> Result<String, JsError> {
        let value = is_simple(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isRegular")]
    pub fn is_regular(&self) -> Result<String, JsError> {
        let value = is_regular(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BoolResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countMutual")]
    pub fn count_mutual(&self, loops: bool) -> Result<String, JsError> {
        let count = count_mutual(&self.inner, loops).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CountResult { count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getAdjacency")]
    pub fn get_adjacency(&self, adj_type: &str) -> Result<String, JsError> {
        let t = match adj_type {
            "upper" => AdjacencyType::Upper,
            "lower" => AdjacencyType::Lower,
            _ => AdjacencyType::Both,
        };
        let matrix = get_adjacency(&self.inner, t, None, LoopHandling::NoLoops)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let size = matrix.len();
        let result = AdjacencyMatrixResult { matrix, size };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getEdgelist")]
    pub fn get_edgelist(&self) -> Result<String, JsError> {
        let el = get_edgelist(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = el.iter().map(|&(u, v)| [u, v]).collect();
        let count = edges.len();
        let result = EdgelistResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "subcomponent")]
    pub fn subcomponent(&self, source: u32, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "in" => SubcomponentMode::In,
            "out" => SubcomponentMode::Out,
            _ => SubcomponentMode::All,
        };
        let vertices =
            subcomponent(&self.inner, source, m).map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = SubcomponentResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "gomoryHuTree")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn gomory_hu_tree(&self) -> Result<String, JsError> {
        let ght = gomory_hu_tree(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let tree_edges: Vec<[u32; 2]> = (0..ght.tree.ecount())
            .map(|eid| {
                let (u, v) = ght.tree.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GomoryHuResult {
            tree_edges,
            flows: ght.flows,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "randomSpanningTree")]
    pub fn random_spanning_tree(&self, seed: u64) -> Result<String, JsError> {
        let edges = random_spanning_tree(&self.inner, None, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = MstResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "richClubSequence")]
    pub fn rich_club_sequence(&self) -> Result<String, JsError> {
        let n = self.inner.vcount();
        let order: Vec<VertexId> = (0..n).collect();
        let coefficients = rich_club_sequence(&self.inner, None, &order, false, false, false)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = RichClubResult { coefficients };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "spanner")]
    pub fn spanner(&self, stretch: f64) -> Result<String, JsError> {
        let edges =
            spanner(&self.inner, stretch, None).map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = SpannerResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Graph generators ────────────────────────────────────────

    #[wasm_bindgen(js_name = "atlasGraph")]
    pub fn atlas_graph(number: u32) -> Result<WasmGraph, JsError> {
        let g = atlas(number).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "chungLuGame")]
    pub fn chung_lu_game_wasm(
        out_weights: &[f64],
        variant: &str,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        let v = match variant {
            "maxEntropy" => ChungLuVariant::Maxent,
            _ => ChungLuVariant::Original,
        };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = chung_lu_game(out_weights, None, loops, v, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "erdosRenyiGnm")]
    pub fn erdos_renyi_gnm_wasm(
        n: u32,
        m: f64,
        directed: bool,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = erdos_renyi_gnm(n, m as u64, directed, loops, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fromPrufer")]
    pub fn from_prufer_wasm(prufer: &[u32]) -> Result<WasmGraph, JsError> {
        let g = from_prufer(prufer).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fullBipartite")]
    pub fn full_bipartite_wasm(
        n1: u32,
        n2: u32,
        directed: bool,
        mode: &str,
    ) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => BipartiteMode::In,
            "all" => BipartiteMode::All,
            _ => BipartiteMode::Out,
        };
        let fb = full_bipartite(n1, n2, directed, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: fb.graph })
    }

    #[wasm_bindgen(js_name = "grgGame")]
    pub fn grg_game_wasm(
        n: u32,
        radius: f64,
        torus: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g =
            grg_game(n, radius, torus, seed as u64).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hypercubeGraph")]
    pub fn hypercube_graph(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = hypercube(n, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "karyTree")]
    pub fn kary_tree_wasm(n: u32, children: u32, mode: &str) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => TreeMode::In,
            "undirected" => TreeMode::Undirected,
            _ => TreeMode::Out,
        };
        let g = kary_tree(n, children, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // ── Graph operations ────────────────────────────────────────

    #[wasm_bindgen(js_name = "disjointUnion")]
    pub fn disjoint_union_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g =
            disjoint_union(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "graphPower")]
    pub fn graph_power_wasm(&self, order: u32) -> Result<WasmGraph, JsError> {
        let g = graph_power(&self.inner, order).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // ── Analytics ───────────────────────────────────────────────

    #[wasm_bindgen(js_name = "adhesion")]
    pub fn adhesion_wasm(&self) -> Result<String, JsError> {
        let value = adhesion(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cohesion")]
    pub fn cohesion_wasm(&self) -> Result<String, JsError> {
        let value = cohesion(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "coreness")]
    pub fn coreness_wasm(&self) -> Result<String, JsError> {
        let cores = coreness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CorenessResult { cores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "degeneracy")]
    pub fn degeneracy_wasm(&self) -> Result<String, JsError> {
        let value = degeneracy(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "density")]
    pub fn density_wasm(&self) -> Result<String, JsError> {
        let d = density(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DensityResult { density: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cutValue")]
    pub fn cut_value_wasm(&self, partition: &[u8]) -> Result<String, JsError> {
        let bools: Vec<bool> = partition.iter().map(|&b| b != 0).collect();
        let value = cut_value(&self.inner, &bools).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CutValueResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maxFlowDetailed")]
    pub fn max_flow_detailed(&self, source: u32, target: u32) -> Result<String, JsError> {
        let mf = max_flow(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowDetailResult {
            value: mf.value,
            flow: mf.flow,
            cut: mf.cut,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "allStCuts")]
    pub fn all_st_cuts_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let st =
            all_st_cuts(&self.inner, source, target).map_err(|e| JsError::new(&e.to_string()))?;
        let count = st.cuts.len();
        let result = StCutsResult {
            cuts: st.cuts,
            partition1s: st.partition1s,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "feedbackVertexSet")]
    pub fn feedback_vertex_set_wasm(&self) -> Result<String, JsError> {
        let vertices = feedback_vertex_set(&self.inner, None, FvsAlgorithm::Greedy)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = FeedbackVertexSetResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Constructors (batch 3) ──────────────────────────────────

    #[wasm_bindgen(js_name = "wheelGraph")]
    pub fn wheel_graph_wasm(n: u32, mode: &str, center: u32) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => WheelMode::In,
            "undirected" => WheelMode::Undirected,
            _ => WheelMode::Out,
        };
        let g = wheel_graph(n, m, center).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "mycielskian")]
    pub fn mycielskian_wasm(&self, k: u32) -> Result<WasmGraph, JsError> {
        let g = mycielskian(&self.inner, k).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "deBruijn")]
    pub fn de_bruijn_wasm(m: u32, n: u32) -> Result<WasmGraph, JsError> {
        let g = de_bruijn(m, n).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "generalizedPetersen")]
    pub fn generalized_petersen_wasm(n: u32, k: u32) -> Result<WasmGraph, JsError> {
        let g = generalized_petersen(n, k).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fullCitation")]
    pub fn full_citation_wasm(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = full_citation(n, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "circulant")]
    pub fn circulant_wasm(n: u32, shifts: &[i32], directed: bool) -> Result<WasmGraph, JsError> {
        let shifts64: Vec<i64> = shifts.iter().map(|&s| i64::from(s)).collect();
        let g = circulant(n, &shifts64, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "staticFitnessGame")]
    pub fn static_fitness_game_wasm(
        no_of_edges: u32,
        fitness_out: &[f64],
        loops: bool,
        multiple: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = static_fitness_game(no_of_edges, fitness_out, None, loops, multiple, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "staticPowerLawGame")]
    pub fn static_power_law_game_wasm(
        no_of_nodes: u32,
        no_of_edges: u32,
        exponent_out: f64,
        loops: bool,
        multiple: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = static_power_law_game(
            no_of_nodes,
            no_of_edges,
            exponent_out,
            None,
            loops,
            multiple,
            false,
            seed as u64,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // ── Graph operations (batch 3) ──────────────────────────────

    #[wasm_bindgen(js_name = "connectNeighborhood")]
    pub fn connect_neighborhood_wasm(&self, order: u32) -> Result<WasmGraph, JsError> {
        let g =
            connect_neighborhood(&self.inner, order).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "intersection")]
    pub fn intersection_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g =
            intersection(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "union")]
    pub fn union_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g = union(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "reverseEdges")]
    pub fn reverse_edges_wasm(&self, eids: &[u32]) -> Result<WasmGraph, JsError> {
        let g = reverse_edges(&self.inner, eids).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "transitiveClosure")]
    pub fn transitive_closure_wasm(&self) -> Result<WasmGraph, JsError> {
        let g = transitive_closure(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // ── Predicates (batch 3) ────────────────────────────────────

    #[wasm_bindgen(js_name = "isClique")]
    pub fn is_clique_wasm(&self, vertices: &[u32], directed: bool) -> Result<bool, JsError> {
        is_clique(&self.inner, vertices, directed).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isMutual")]
    pub fn is_mutual_wasm(&self, loops: bool) -> Result<String, JsError> {
        let flags = is_mutual(&self.inner, loops).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&flags).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isKDegenerate")]
    pub fn is_k_degenerate_wasm(&self, k: u32) -> Result<bool, JsError> {
        is_k_degenerate(&self.inner, k).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isIndependentVertexSet")]
    pub fn is_independent_vertex_set_wasm(&self, vertices: &[u32]) -> Result<bool, JsError> {
        is_independent_vertex_set(&self.inner, vertices).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isVertexCover")]
    pub fn is_vertex_cover_wasm(&self, cover: &[u32]) -> bool {
        is_vertex_cover(&self.inner, cover)
    }

    #[wasm_bindgen(js_name = "isEdgeCover")]
    pub fn is_edge_cover_wasm(&self, cover: &[u32]) -> bool {
        is_edge_cover(&self.inner, cover)
    }

    #[wasm_bindgen(js_name = "isDominatingSet")]
    pub fn is_dominating_set_wasm(&self, dom_set: &[u32]) -> bool {
        is_dominating_set(&self.inner, dom_set)
    }

    // ── Analytics (batch 3) ─────────────────────────────────────

    #[wasm_bindgen(js_name = "minimumEdgeCover")]
    pub fn minimum_edge_cover_wasm(&self) -> Result<String, JsError> {
        let edges = minimum_edge_cover(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = SpannerResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "minimumVertexCover")]
    pub fn minimum_vertex_cover_wasm(&self) -> Result<String, JsError> {
        let vertices =
            minimum_vertex_cover(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = FeedbackVertexSetResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "minimumDominatingSet")]
    pub fn minimum_dominating_set_wasm(&self) -> Result<String, JsError> {
        let vertices =
            minimum_dominating_set(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = FeedbackVertexSetResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stMincutValue")]
    pub fn st_mincut_value_wasm(&self, source: u32, target: u32) -> Result<f64, JsError> {
        st_mincut_value(&self.inner, source, target, None).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stVertexConnectivity")]
    pub fn st_vertex_connectivity_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let value = st_vertex_connectivity(&self.inner, source, target, VconnNei::NumberOfNodes)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "regularity")]
    pub fn regularity_wasm(&self) -> Result<String, JsError> {
        let value = regularity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&value).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countReachable")]
    pub fn count_reachable_wasm(&self) -> Result<String, JsError> {
        let counts = count_reachable(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&counts).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "sortVerticesByDegree")]
    pub fn sort_vertices_by_degree_wasm(
        &self,
        mode: &str,
        ascending: bool,
    ) -> Result<String, JsError> {
        let dm = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let order = if ascending {
            SortOrder::Ascending
        } else {
            SortOrder::Descending
        };
        let sorted = sort_vertices_by_degree(&self.inner, dm, order)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&sorted).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getShortestPath")]
    pub fn get_shortest_path_wasm(
        &self,
        from: u32,
        to: u32,
        mode: &str,
    ) -> Result<String, JsError> {
        let dm = match mode {
            "in" => DijkstraMode::In,
            "out" => DijkstraMode::Out,
            _ => DijkstraMode::All,
        };
        let sp = get_shortest_path(&self.inner, from, to, None, dm)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&sp.vertices).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "dominatorTree")]
    pub fn dominator_tree_wasm(&self, root: u32, mode: &str) -> Result<String, JsError> {
        let dm = match mode {
            "in" => DominatorMode::In,
            _ => DominatorMode::Out,
        };
        let dt = dominator_tree(&self.inner, root, dm).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&dt.idom).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Isomorphism (batch 4) ───────────────────────────────────────

    #[wasm_bindgen(js_name = "isomorphic")]
    pub fn isomorphic_wasm(&self, other: &WasmGraph) -> Result<bool, JsError> {
        isomorphic(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "subisomorphic")]
    pub fn subisomorphic_wasm(&self, other: &WasmGraph) -> Result<bool, JsError> {
        subisomorphic(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Analysis (batch 4) ──────────────────────────────────────────

    #[wasm_bindgen(js_name = "simpleCycles")]
    pub fn simple_cycles_wasm(
        &self,
        mode: &str,
        min_length: u32,
        max_length: i32,
        max_results: i32,
    ) -> Result<String, JsError> {
        let m = match mode {
            "in" => SimpleCycleMode::In,
            "out" => SimpleCycleMode::Out,
            _ => SimpleCycleMode::All,
        };
        #[allow(clippy::cast_sign_loss)]
        let max_len = if max_length > 0 {
            Some(max_length as u32)
        } else {
            None
        };
        #[allow(clippy::cast_sign_loss)]
        let max_res = if max_results > 0 {
            Some(max_results as usize)
        } else {
            None
        };
        let cycles = simple_cycles(&self.inner, m, min_length, max_len, max_res)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result: Vec<Vec<u32>> = cycles.into_iter().map(|c| c.vertices).collect();
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "sir")]
    pub fn sir_wasm(
        &self,
        beta: f64,
        gamma: f64,
        no_sim: u32,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let results = sir(&self.inner, beta, gamma, no_sim as usize, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct SirOut {
            times: Vec<f64>,
            no_s: Vec<usize>,
            no_i: Vec<usize>,
            no_r: Vec<usize>,
        }
        let out: Vec<SirOut> = results
            .into_iter()
            .map(|r| SirOut {
                times: r.times,
                no_s: r.no_s,
                no_i: r.no_i,
                no_r: r.no_r,
            })
            .collect();
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "unfoldTree")]
    pub fn unfold_tree_wasm(&self, roots: &[u32], mode: &str) -> Result<String, JsError> {
        let dm = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let result =
            unfold_tree(&self.inner, roots, dm).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct UnfoldOut {
            edges: Vec<(u32, u32)>,
            vertex_index: Vec<u32>,
        }
        let edges = result
            .tree
            .get_edgelist()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let out = UnfoldOut {
            edges,
            vertex_index: result.vertex_index,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "mincut")]
    pub fn mincut_wasm(&self) -> Result<String, JsError> {
        let result = mincut(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct MincutOut {
            value: f64,
            cut: Vec<u32>,
            partition: Vec<u32>,
        }
        let out = MincutOut {
            value: result.value,
            cut: result.cut,
            partition: result.partition,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stMincut")]
    pub fn st_mincut_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let result = st_mincut(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct StMincutOut {
            value: f64,
            cut: Vec<u32>,
            partition: Vec<u32>,
        }
        let out = StMincutOut {
            value: result.value,
            cut: result.cut,
            partition: result.partition,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "pathLengthHist")]
    pub fn path_length_hist_wasm(&self, directed: bool) -> Result<String, JsError> {
        let result =
            path_length_hist(&self.inner, directed).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct HistOut {
            hist: Vec<f64>,
            unconnected: f64,
        }
        let out = HistOut {
            hist: result.hist,
            unconnected: result.unconnected,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "evenTarjanReduction")]
    pub fn even_tarjan_reduction_wasm(&self) -> Result<String, JsError> {
        let result =
            even_tarjan_reduction(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let edges = result
            .graph
            .get_edgelist()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let vcount = result.graph.vcount();
        #[derive(Serialize)]
        struct EtOut {
            edges: Vec<(u32, u32)>,
            vcount: u32,
            capacity: Vec<f64>,
        }
        let out = EtOut {
            vcount,
            edges,
            capacity: result.capacity,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hasMutual")]
    pub fn has_mutual_wasm(&self, loops: bool) -> Result<bool, JsError> {
        has_mutual(&self.inner, loops).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "toPrufer")]
    pub fn to_prufer_wasm(&self) -> Result<String, JsError> {
        let seq = to_prufer(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&seq).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Generators (batch 4) ────────────────────────────────────────

    #[wasm_bindgen(js_name = "treeGameLerw")]
    pub fn tree_game_lerw_wasm(n: u32, directed: bool, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = tree_game_lerw(n, directed, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "treeGamePrufer")]
    pub fn tree_game_prufer_wasm(n: u32, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = tree_game_prufer(n, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "preferenceGame")]
    pub fn preference_game_wasm(
        nodes: u32,
        types: u32,
        pref_matrix_flat: &[f64],
        directed: bool,
        loops: bool,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let k = types as usize;
        let pref_matrix: Vec<Vec<f64>> = pref_matrix_flat.chunks(k).map(<[f64]>::to_vec).collect();
        let (g, node_types) =
            preference_game(nodes, types, None, false, &pref_matrix, directed, loops, s)
                .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PrefOut {
            edges: Vec<(u32, u32)>,
            vcount: u32,
            node_types: Vec<u32>,
        }
        let out = PrefOut {
            edges: g.get_edgelist().map_err(|e| JsError::new(&e.to_string()))?,
            vcount: g.vcount(),
            node_types,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hsbmGame")]
    #[allow(clippy::many_single_char_names)]
    pub fn hsbm_game_wasm(
        n: u32,
        m: u32,
        rho_flat: &[f64],
        c_flat: &[f64],
        p: f64,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let k = rho_flat.len();
        let c: Vec<Vec<f64>> = c_flat.chunks(k).map(<[f64]>::to_vec).collect();
        let g = hsbm_game(n, m, rho_flat, &c, p, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "correlatedGame")]
    pub fn correlated_game_wasm(&self, corr: f64, p: f64, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = correlated_game(&self.inner, corr, p, None, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "correlatedPairGame")]
    pub fn correlated_pair_game_wasm(
        n: u32,
        corr: f64,
        p: f64,
        directed: bool,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let (g1, g2) = correlated_pair_game(n, corr, p, directed, None, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PairOut {
            graph1_edges: Vec<(u32, u32)>,
            graph2_edges: Vec<(u32, u32)>,
            vcount: u32,
        }
        let out = PairOut {
            graph1_edges: g1
                .get_edgelist()
                .map_err(|e| JsError::new(&e.to_string()))?,
            graph2_edges: g2
                .get_edgelist()
                .map_err(|e| JsError::new(&e.to_string()))?,
            vcount: n,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Utilities (batch 4) ─────────────────────────────────────────

    #[wasm_bindgen(js_name = "famousNames")]
    pub fn famous_names_wasm() -> String {
        let names = famous_names();
        serde_json::to_string(names).unwrap_or_else(|_| "[]".to_string())
    }

    #[wasm_bindgen(js_name = "invertPermutation")]
    pub fn invert_permutation_wasm(perm: &[u32]) -> Result<String, JsError> {
        let result = invert_permutation(perm).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Statistics (batch 4) ────────────────────────────────────────

    #[wasm_bindgen(js_name = "runningMean")]
    pub fn running_mean_wasm(data: &[f64], binwidth: u32) -> Result<String, JsError> {
        let result =
            running_mean(data, binwidth as usize).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "powerLawFit")]
    pub fn power_law_fit_wasm(data: &[f64], xmin: f64) -> Result<String, JsError> {
        let result = power_law_fit(data, xmin, false).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PlFitOut {
            continuous: bool,
            alpha: f64,
            xmin: f64,
            log_likelihood: f64,
            ks_statistic: f64,
        }
        let out = PlFitOut {
            continuous: result.continuous,
            alpha: result.alpha,
            xmin: result.xmin,
            log_likelihood: result.log_likelihood,
            ks_statistic: result.ks_statistic,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "convexHull2d")]
    pub fn convex_hull_2d_wasm(points_flat: &[f64]) -> Result<String, JsError> {
        let points: Vec<(f64, f64)> = points_flat.chunks(2).map(|c| (c[0], c[1])).collect();
        let result = convex_hull_2d(&points).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct HullOut {
            hull_vertices: Vec<usize>,
            hull_coords: Vec<(f64, f64)>,
        }
        let out = HullOut {
            hull_vertices: result.hull_vertices,
            hull_coords: result.hull_coords,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Graph queries (batch 5) ─────────────────────────────────────

    #[wasm_bindgen(js_name = "areAdjacent")]
    pub fn are_adjacent_wasm(&self, v1: u32, v2: u32) -> Result<bool, JsError> {
        are_adjacent(&self.inner, v1, v2).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countLoops")]
    pub fn count_loops_wasm(&self) -> Result<String, JsError> {
        let n = count_loops(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&n).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countAdjacentTriangles")]
    pub fn count_adjacent_triangles_wasm(&self) -> Result<String, JsError> {
        let counts =
            count_adjacent_triangles(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&counts).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cocitation")]
    pub fn cocitation_wasm(&self) -> Result<String, JsError> {
        let result = cocitation(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "bibcoupling")]
    pub fn bibcoupling_wasm(&self) -> Result<String, JsError> {
        let result = bibcoupling(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "averageLocalEfficiency")]
    pub fn average_local_efficiency_wasm(&self) -> Result<f64, JsError> {
        average_local_efficiency(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "degreeCorrelationVector")]
    pub fn degree_correlation_vector_wasm(&self, mode: &str) -> Result<String, JsError> {
        let dm = match mode {
            "in" => DegreeMode::In,
            "out" => DegreeMode::Out,
            _ => DegreeMode::All,
        };
        let result = degree_correlation_vector(&self.inner, dm, dm, true, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Cliques (batch 5) ───────────────────────────────────────────

    #[wasm_bindgen(js_name = "cliques")]
    pub fn cliques_wasm(&self, min_size: u32, max_size: u32) -> Result<String, JsError> {
        let result = cliques(&self.inner, min_size, max_size, Some(10000))
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cliqueSizeHist")]
    pub fn clique_size_hist_wasm(&self) -> Result<String, JsError> {
        let result = clique_size_hist(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Traversal (batch 5) ─────────────────────────────────────────

    #[wasm_bindgen(js_name = "bfsTree")]
    pub fn bfs_tree_wasm(&self, root: u32) -> Result<String, JsError> {
        let result = bfs_tree(&self.inner, root).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct BfsTreeOut {
            order: Vec<u32>,
            distances: Vec<Option<u32>>,
        }
        let out = BfsTreeOut {
            order: result.order,
            distances: result.distances,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "bfsSimple")]
    pub fn bfs_simple_wasm(&self, root: u32, mode: &str) -> Result<String, JsError> {
        let bm = match mode {
            "in" => BfsMode::In,
            "out" => BfsMode::Out,
            _ => BfsMode::All,
        };
        let result = bfs_simple(&self.inner, root, bm).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct BfsSimpleOut {
            order: Vec<u32>,
            layers: Vec<usize>,
        }
        let out = BfsSimpleOut {
            order: result.order,
            layers: result.layers,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Operators (batch 5) ─────────────────────────────────────────

    #[wasm_bindgen(js_name = "cartesianProduct")]
    pub fn cartesian_product_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g = cartesian_product(&self.inner, &other.inner)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "compose")]
    pub fn compose_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g = compose(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    // ── Percolation (batch 5) ───────────────────────────────────────

    #[wasm_bindgen(js_name = "bondPercolation")]
    pub fn bond_percolation_wasm(&self, edge_order: &[u32]) -> Result<String, JsError> {
        let result =
            bond_percolation(&self.inner, edge_order).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PercolationOut {
            giant_size: Vec<u32>,
            vertex_count: Vec<u32>,
        }
        let out = PercolationOut {
            giant_size: result.giant_size,
            vertex_count: result.vertex_count,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Bipartite (batch 5) ─────────────────────────────────────────

    #[wasm_bindgen(js_name = "bipartiteProjectionSize")]
    pub fn bipartite_projection_size_wasm(&self, types: &[u8]) -> Result<String, JsError> {
        let type_vec: Vec<bool> = types.iter().map(|&b| b != 0).collect();
        let result = bipartite_projection_size(&self.inner, &type_vec)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct BiprojSize {
            vcount1: u32,
            ecount1: u32,
            vcount2: u32,
            ecount2: u32,
        }
        let out = BiprojSize {
            vcount1: result.vcount1,
            ecount1: result.ecount1,
            vcount2: result.vcount2,
            ecount2: result.ecount2,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Generators (batch 5) ────────────────────────────────────────

    #[wasm_bindgen(js_name = "bipartiteGameGnp")]
    pub fn bipartite_game_gnp_wasm(
        n1: u32,
        n2: u32,
        p: f64,
        directed: bool,
        mode: &str,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let bm = match mode {
            "in" => BipartiteMode::In,
            "out" => BipartiteMode::Out,
            _ => BipartiteMode::All,
        };
        let bg = bipartite_game_gnp(n1, n2, p, directed, bm, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: bg.graph })
    }

    #[wasm_bindgen(js_name = "bipartiteGameGnm")]
    pub fn bipartite_game_gnm_wasm(
        n1: u32,
        n2: u32,
        m: u32,
        directed: bool,
        mode: &str,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let bm = match mode {
            "in" => BipartiteMode::In,
            "out" => BipartiteMode::Out,
            _ => BipartiteMode::All,
        };
        let bg = bipartite_game_gnm(n1, n2, u64::from(m), directed, bm, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: bg.graph })
    }

    #[wasm_bindgen(js_name = "delaunayGraph")]
    pub fn delaunay_graph_wasm(points_flat: &[f64], dim: u32) -> Result<WasmGraph, JsError> {
        let d = dim as usize;
        let points: Vec<Vec<f64>> = points_flat.chunks(d).map(<[f64]>::to_vec).collect();
        let g = delaunay_graph(&points).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }
}
