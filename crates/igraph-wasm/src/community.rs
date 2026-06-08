use super::*;

#[wasm_bindgen]
impl WasmGraph {
    pub fn louvain(&self) -> Result<String, JsError> {
        let lr = louvain(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = LouvainOutput {
            membership: lr.membership,
            modularity: lr.modularity,
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

    pub fn modularity(&self, membership: Vec<u32>) -> Result<String, JsError> {
        let m =
            modularity(&self.inner, &membership, 1.0).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DensityResult { density: m };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "modularityMatrix")]
    pub fn modularity_matrix_wasm(&self) -> Result<String, JsError> {
        let mat = modularity_matrix(&self.inner, None, 1.0, self.inner.is_directed())
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&mat).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "edgeBetweennessCommunityWeighted")]
    pub fn edge_betweenness_community_weighted_wasm(
        &self,
        weights: &[f64],
    ) -> Result<String, JsError> {
        #[derive(Serialize)]
        struct Out {
            membership: Vec<u32>,
            nb_clusters: u32,
            modularity: Vec<f64>,
        }
        let result = edge_betweenness_community_weighted(&self.inner, weights)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let out = Out {
            membership: result.membership,
            nb_clusters: result.nb_clusters,
            modularity: result.modularity,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }
}
