use rust_igraph::{
    ConnectednessMode, DijkstraMode, FrParams, Graph, GreedyColoringHeuristic, VertexId,
    articulation_points, betweenness, bfs, bridges, closeness, connected_components,
    count_triangles, dfs, dijkstra_distances, edge_betweenness, edge_betweenness_community,
    eigenvector_centrality, fast_greedy_modularity, fluid_communities, girth, harmonic_centrality,
    hub_and_authority_scores, infomap, is_bipartite, is_connected, katz_centrality,
    label_propagation, layout_fruchterman_reingold, leading_eigenvector, leiden, louvain,
    max_flow_value, pagerank, spinglass, strongly_connected_components, topological_sorting,
    transitivity_undirected, vertex_coloring_greedy, walktrap,
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
        let count = edges.len() as u32;
        let result = BridgesResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "vertexColoring")]
    pub fn vertex_coloring(&self) -> Result<String, JsError> {
        let colors = vertex_coloring_greedy(&self.inner, GreedyColoringHeuristic::DSatur)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let chromatic = colors.iter().copied().max().map(|c| c + 1).unwrap_or(0);
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
}
