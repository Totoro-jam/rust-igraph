#![allow(clippy::needless_pass_by_value)]

use rust_igraph::{
    ConnectednessMode, DijkstraMode, FasAlgorithm, FrParams, Graph, GreedyColoringHeuristic,
    KkParams, MstAlgorithm, StarMode, VertexId, articulation_points, assortativity_degree,
    automorphism_group, barabasi_game_bag, bellman_ford_distances, betweenness,
    betweenness_weighted, bfs, bridges, canonical_permutation, clique_number, closeness,
    closeness_weighted, complementer, connected_components, constraint, coreness,
    count_automorphisms, count_triangles, cycle_graph, decompose, degree_distribution, density,
    dfs, diameter, dijkstra_distances, distances, eccentricity, edge_betweenness,
    edge_betweenness_community, edge_connectivity, eigenvector_centrality, erdos_renyi_gnp, famous,
    fast_greedy_modularity, feedback_arc_set, floyd_warshall_distances, fluid_communities,
    full_graph, fundamental_cycles, girth, harmonic_centrality, hub_and_authority_scores,
    independence_number, infomap, is_acyclic, is_biconnected, is_bipartite, is_complete,
    is_connected, is_cubic, is_cycle, is_dag, is_forest, is_outerplanar, is_path, is_perfect,
    is_star, is_tournament, is_tree, is_triangle_free, is_wheel, isomorphic_bliss, katz_centrality,
    label_propagation, layout_circle, layout_fruchterman_reingold, layout_grid,
    layout_kamada_kawai, layout_random, layout_star, leading_eigenvector, leiden, line_graph,
    list_triangles, louvain, max_flow_value, maximal_cliques, mean_degree, mean_distance,
    minimum_cycle_basis, minimum_spanning_tree, modularity, pagerank, path_graph, radius,
    random_walk, reciprocity, ring_graph, simplify, spinglass, star_graph, strength,
    strongly_connected_components, topological_sorting, transitivity_undirected, triad_census,
    trussness, vertex_coloring_greedy, vertex_connectivity, walktrap, watts_strogatz_game,
    write_dot, write_gml, write_graphml,
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
    pub fn degree_sequence(&self) -> Result<String, JsError> {
        let degrees = self
            .inner
            .degree_sequence()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = DegreeResult { degrees };
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
}
