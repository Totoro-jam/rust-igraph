use rust_igraph::{
    FrParams, Graph, VertexId, betweenness, bfs, closeness, connected_components, dfs,
    dijkstra_distances, edge_betweenness_community, eigenvector_centrality, fast_greedy_modularity,
    fluid_communities, infomap, label_propagation, layout_fruchterman_reingold,
    leading_eigenvector, leiden, louvain, pagerank, spinglass, walktrap,
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
struct LayoutResult {
    coords: Vec<[f64; 2]>,
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
}
