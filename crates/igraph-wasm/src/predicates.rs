use super::*;

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(js_name = "vertexColoring")]
    pub fn vertex_coloring(&self) -> Result<String, JsError> {
        let colors = vertex_coloring_greedy(&self.inner, GreedyColoringHeuristic::ColoredNeighbors)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let chromatic = colors.iter().copied().max().unwrap_or(0) + 1;
        let result = ColoringResult { colors, chromatic };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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
            is_tree: is_tree(&self.inner, DijkstraMode::Out).is_ok_and(|r| r.is_some()),
            is_forest: is_forest(&self.inner, DijkstraMode::Out).is_ok_and(|r| r.is_some()),
            is_dag: is_dag(&self.inner),
            is_acyclic: is_acyclic(&self.inner),
            is_complete: is_complete(&self.inner).unwrap_or(false),
            is_biconnected: is_biconnected(&self.inner).unwrap_or(false),
            is_bipartite: is_bipartite(&self.inner).is_ok_and(|r| r.is_bipartite),
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

    #[wasm_bindgen(js_name = "isBipartiteDetailed")]
    pub fn is_bipartite_detailed(&self) -> Result<String, JsError> {
        let bp = is_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BipartiteCheckResult {
            is_bipartite: bp.is_bipartite,
            types: bp.types.iter().map(|&b| u32::from(b)).collect(),
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isEulerian")]
    pub fn is_eulerian(&self) -> Result<String, JsError> {
        let e = is_eulerian(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EulerianCheckResult {
            has_path: e.has_path,
            has_cycle: e.has_cycle,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "chromaticNumberUpperBound")]
    pub fn chromatic_number_upper_bound(&self) -> Result<String, JsError> {
        let val =
            chromatic_number_upper_bound(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarU32Result { value: val };
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

    #[wasm_bindgen(js_name = "hasMutual")]
    pub fn has_mutual_wasm(&self, loops: bool) -> Result<bool, JsError> {
        has_mutual(&self.inner, loops).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "satisfiesDirac")]
    pub fn satisfies_dirac_wasm(&self) -> Result<bool, JsError> {
        satisfies_dirac(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "satisfiesOre")]
    pub fn satisfies_ore_wasm(&self) -> Result<bool, JsError> {
        satisfies_ore(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSimpleWithMode")]
    pub fn is_simple_with_mode_wasm(&self, mode: &str) -> Result<bool, JsError> {
        let m = match mode {
            "undirected" => SimpleMode::DirectedAsUndirected,
            _ => SimpleMode::DirectedAsDirected,
        };
        is_simple_with_mode(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maximumIndependentSet")]
    pub fn maximum_independent_set_wasm(&self) -> Result<Vec<u32>, JsError> {
        maximum_independent_set(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "isBipartite")]
    pub fn is_bipartite_wasm(&self) -> Result<String, JsError> {
        let result = is_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct BipartiteOut {
            is_bipartite: bool,
            types: Vec<bool>,
        }
        let out = BipartiteOut {
            is_bipartite: result.is_bipartite,
            types: result.types,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isConnected")]
    pub fn is_connected_wasm(&self, mode: &str) -> Result<bool, JsError> {
        let m = match mode {
            "strong" => ConnectednessMode::Strong,
            _ => ConnectednessMode::Weak,
        };
        is_connected(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isApexForest")]
    pub fn is_apex_forest_wasm(&self) -> Result<bool, JsError> {
        is_apex_forest(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isApexTree")]
    pub fn is_apex_tree_wasm(&self) -> Result<bool, JsError> {
        is_apex_tree(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBannerFree")]
    pub fn is_banner_free_wasm(&self) -> Result<bool, JsError> {
        is_banner_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBiregular")]
    pub fn is_biregular_wasm(&self) -> Result<bool, JsError> {
        is_biregular(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBlockGraph")]
    pub fn is_block_graph_wasm(&self) -> Result<bool, JsError> {
        is_block_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBowtieFree")]
    pub fn is_bowtie_free_wasm(&self) -> Result<bool, JsError> {
        is_bowtie_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isBullFree")]
    pub fn is_bull_free_wasm(&self) -> Result<bool, JsError> {
        is_bull_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isC4Free")]
    pub fn is_c4_free_wasm(&self) -> Result<bool, JsError> {
        is_c4_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isC5Free")]
    pub fn is_c5_free_wasm(&self) -> Result<bool, JsError> {
        is_c5_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCactusGraph")]
    pub fn is_cactus_graph_wasm(&self) -> Result<bool, JsError> {
        is_cactus_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCaterpillar")]
    pub fn is_caterpillar_wasm(&self) -> Result<bool, JsError> {
        is_caterpillar(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isChainGraph")]
    pub fn is_chain_graph_wasm(&self) -> Result<bool, JsError> {
        is_chain_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isChordalBipartite")]
    pub fn is_chordal_bipartite_wasm(&self) -> Result<bool, JsError> {
        is_chordal_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isClawFree")]
    pub fn is_claw_free_wasm(&self) -> Result<bool, JsError> {
        is_claw_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isClusterGraph")]
    pub fn is_cluster_graph_wasm(&self) -> Result<bool, JsError> {
        is_cluster_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCoBipartite")]
    pub fn is_co_bipartite_wasm(&self) -> Result<bool, JsError> {
        is_co_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCoChordal")]
    pub fn is_co_chordal_wasm(&self) -> Result<bool, JsError> {
        is_co_chordal(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCograph")]
    pub fn is_cograph_wasm(&self) -> Result<bool, JsError> {
        is_cograph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCompleteBipartite")]
    pub fn is_complete_bipartite_wasm(&self) -> Result<bool, JsError> {
        is_complete_bipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isCompleteMultipartite")]
    pub fn is_complete_multipartite_wasm(&self) -> Result<bool, JsError> {
        let r = is_complete_multipartite(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(r.is_some())
    }

    #[wasm_bindgen(js_name = "isCricketFree")]
    pub fn is_cricket_free_wasm(&self) -> Result<bool, JsError> {
        is_cricket_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isDartFree")]
    pub fn is_dart_free_wasm(&self) -> Result<bool, JsError> {
        is_dart_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isDiamondFree")]
    pub fn is_diamond_free_wasm(&self) -> Result<bool, JsError> {
        is_diamond_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isDistanceHereditary")]
    pub fn is_distance_hereditary_wasm(&self) -> Result<bool, JsError> {
        is_distance_hereditary(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isForkFree")]
    pub fn is_fork_free_wasm(&self) -> Result<bool, JsError> {
        is_fork_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isGemFree")]
    pub fn is_gem_free_wasm(&self) -> Result<bool, JsError> {
        is_gem_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isGeodetic")]
    pub fn is_geodetic_wasm(&self) -> Result<bool, JsError> {
        is_geodetic(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isHouseFree")]
    pub fn is_house_free_wasm(&self) -> Result<bool, JsError> {
        is_house_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isLobster")]
    pub fn is_lobster_wasm(&self) -> Result<bool, JsError> {
        is_lobster(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isNetFree")]
    pub fn is_net_free_wasm(&self) -> Result<bool, JsError> {
        is_net_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isP5Free")]
    pub fn is_p5_free_wasm(&self) -> Result<bool, JsError> {
        is_p5_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPawFree")]
    pub fn is_paw_free_wasm(&self) -> Result<bool, JsError> {
        is_paw_free(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isProperInterval")]
    pub fn is_proper_interval_wasm(&self) -> Result<bool, JsError> {
        is_proper_interval(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPseudoForest")]
    pub fn is_pseudo_forest_wasm(&self) -> Result<bool, JsError> {
        is_pseudo_forest(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isPtolemaic")]
    pub fn is_ptolemaic_wasm(&self) -> Result<bool, JsError> {
        is_ptolemaic(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSelfComplementary")]
    pub fn is_self_complementary_wasm(&self) -> Result<bool, JsError> {
        is_self_complementary(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSemicomplete")]
    pub fn is_semicomplete_wasm(&self) -> Result<bool, JsError> {
        is_semicomplete(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSeriesParallel")]
    pub fn is_series_parallel_wasm(&self) -> Result<bool, JsError> {
        is_series_parallel(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSpider")]
    pub fn is_spider_wasm(&self) -> Result<bool, JsError> {
        is_spider(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSplitGraph")]
    pub fn is_split_graph_wasm(&self) -> Result<bool, JsError> {
        is_split_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isStronglyChordal")]
    pub fn is_strongly_chordal_wasm(&self) -> Result<bool, JsError> {
        is_strongly_chordal(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isThresholdGraph")]
    pub fn is_threshold_graph_wasm(&self) -> Result<bool, JsError> {
        is_threshold_graph(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isUnicyclic")]
    pub fn is_unicyclic_wasm(&self) -> Result<bool, JsError> {
        is_unicyclic(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isWeaklyChordal")]
    pub fn is_weakly_chordal_wasm(&self) -> Result<bool, JsError> {
        is_weakly_chordal(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isWellCovered")]
    pub fn is_well_covered_wasm(&self) -> Result<bool, JsError> {
        is_well_covered(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isWindmill")]
    pub fn is_windmill_wasm(&self) -> Result<bool, JsError> {
        let r = is_windmill(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(r.is_some())
    }
}
