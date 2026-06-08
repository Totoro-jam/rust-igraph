use super::*;

#[wasm_bindgen]
impl WasmGraph {
    pub fn pagerank(&self) -> Result<String, JsError> {
        let scores = pagerank(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn betweenness(&self) -> Result<String, JsError> {
        let scores = betweenness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BetweennessResult { scores };
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

    pub fn constraint(&self) -> Result<String, JsError> {
        let scores = constraint(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ConstraintResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn coreness(&self) -> Result<String, JsError> {
        let cores = coreness(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CorenessResult { cores };
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

    pub fn strength(&self, weights: Vec<f64>) -> Result<String, JsError> {
        let s = strength(&self.inner, &weights).map_err(|e| JsError::new(&e.to_string()))?;
        let result = StrengthResult { scores: s };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "personalizedPagerank")]
    pub fn personalized_pagerank(&self, reset: &[f64], damping: f64) -> Result<String, JsError> {
        let scores = personalized_pagerank(&self.inner, reset, damping)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = PageRankResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn degeneracy(&self) -> Result<String, JsError> {
        let v = degeneracy(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "convergenceDegree")]
    pub fn convergence_degree(&self) -> Result<String, JsError> {
        let scores = convergence_degree(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ConvergenceDegreeResult { scores };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "similarityJaccard")]
    pub fn similarity_jaccard(&self) -> Result<String, JsError> {
        let flat = similarity_jaccard(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let n = self.inner.vcount() as usize;
        let matrix: Vec<Vec<f64>> = (0..n).map(|i| flat[i * n..(i + 1) * n].to_vec()).collect();
        let result = SimilarityMatrixResult { matrix, size: n };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "similarityDice")]
    pub fn similarity_dice(&self) -> Result<String, JsError> {
        let flat = similarity_dice(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let n = self.inner.vcount() as usize;
        let matrix: Vec<Vec<f64>> = (0..n).map(|i| flat[i * n..(i + 1) * n].to_vec()).collect();
        let result = SimilarityMatrixResult { matrix, size: n };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "clusteringCoefficients")]
    pub fn clustering_coefficients(&self) -> Result<String, JsError> {
        let scores = self
            .inner
            .clustering_coefficients()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ClusteringCoeffResult { scores };
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

    #[wasm_bindgen(js_name = "betweennessCutoff")]
    pub fn betweenness_cutoff_wasm(&self, cutoff: u32) -> Result<Vec<f64>, JsError> {
        betweenness_cutoff(&self.inner, cutoff).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeBetweennessWeighted")]
    pub fn edge_betweenness_weighted_wasm(&self, weights: &[f64]) -> Result<Vec<f64>, JsError> {
        edge_betweenness_weighted(&self.inner, weights).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "harmonicCentralityWeighted")]
    pub fn harmonic_centrality_weighted_wasm(&self, weights: &[f64]) -> Result<Vec<f64>, JsError> {
        harmonic_centrality_weighted(&self.inner, weights).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "pagerankWeighted")]
    pub fn pagerank_weighted_wasm(&self, weights: &[f64]) -> Result<Vec<f64>, JsError> {
        pagerank_weighted(&self.inner, weights).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeBetweennessCutoff")]
    pub fn edge_betweenness_cutoff_wasm(&self, cutoff: u32) -> Result<Vec<f64>, JsError> {
        edge_betweenness_cutoff(&self.inner, cutoff).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "harmonicCentralityCutoff")]
    pub fn harmonic_centrality_cutoff_wasm(
        &self,
        cutoff: u32,
        normalized: bool,
    ) -> Result<Vec<f64>, JsError> {
        harmonic_centrality_cutoff(&self.inner, cutoff, normalized)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "corenessWithMode")]
    pub fn coreness_with_mode_wasm(&self, mode: &str) -> Result<String, JsError> {
        let m = match mode {
            "in" => CorenessMode::In,
            "out" => CorenessMode::Out,
            _ => CorenessMode::All,
        };
        let result =
            coreness_with_mode(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }
}
