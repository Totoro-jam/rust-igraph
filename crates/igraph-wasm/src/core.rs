use super::*;

#[wasm_bindgen]
impl WasmGraph {
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

    #[wasm_bindgen(js_name = "connectedComponents")]
    pub fn connected_components(&self) -> Result<String, JsError> {
        let cc = connected_components(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ComponentsResult {
            membership: cc.membership,
            count: cc.count,
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

    pub fn density(&self) -> Result<String, JsError> {
        let d = density(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DensityResult { density: d };
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

    pub fn reciprocity(&self) -> Result<String, JsError> {
        let r = reciprocity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ReciprocityResult { reciprocity: r };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "degreeDistribution")]
    pub fn degree_distribution(&self) -> Result<String, JsError> {
        let d = degree_distribution(&self.inner, rust_igraph::DegreeMode::All)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = DegreeResult { degrees: d };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn decompose(&self) -> Result<Vec<WasmGraph>, JsError> {
        let graphs = decompose(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(graphs.into_iter().map(|g| WasmGraph { inner: g }).collect())
    }

    #[wasm_bindgen(js_name = "biconnectedComponents")]
    pub fn biconnected_components(&self) -> Result<String, JsError> {
        let bc = biconnected_components(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = BiconnectedResult {
            count: bc.count,
            components: bc.components,
        };
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

    #[wasm_bindgen(js_name = "areAdjacent")]
    pub fn are_adjacent_wasm(&self, v1: u32, v2: u32) -> Result<bool, JsError> {
        are_adjacent(&self.inner, v1, v2).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countLoops")]
    pub fn count_loops_wasm(&self) -> Result<String, JsError> {
        let n = count_loops(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&n).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "avgNearestNeighborDegree")]
    pub fn avg_nearest_neighbor_degree(&self) -> Result<String, JsError> {
        let knn =
            avg_nearest_neighbor_degree(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = KnnResult { scores: knn };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
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

    #[wasm_bindgen(js_name = "neighborhood")]
    pub fn neighborhood(&self, order: i32) -> Result<String, JsError> {
        let neighborhoods =
            neighborhood(&self.inner, order).map_err(|e| JsError::new(&e.to_string()))?;
        let result = NeighborhoodResult { neighborhoods };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "getStochastic")]
    pub fn get_stochastic_wasm(&self, column_wise: bool) -> Result<String, JsError> {
        let mat = get_stochastic(&self.inner, column_wise, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&mat).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "assortativity")]
    pub fn assortativity_wasm(&self, values: &[f64], directed: bool) -> Result<f64, JsError> {
        let r = assortativity(&self.inner, values, None, None, directed, true)
            .map_err(|e| JsError::new(&e.to_string()))?;
        r.ok_or_else(|| JsError::new("assortativity undefined for this graph"))
    }

    #[wasm_bindgen(js_name = "assortativityNominal")]
    pub fn assortativity_nominal_wasm(
        &self,
        types: &[u32],
        directed: bool,
    ) -> Result<f64, JsError> {
        assortativity_nominal(&self.inner, types, directed, true)
            .map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "jointDegreeDistribution")]
    pub fn joint_degree_distribution_wasm(&self, normalized: bool) -> Result<String, JsError> {
        let mat = joint_degree_distribution(
            &self.inner,
            DegreeMode::All,
            DegreeMode::All,
            false,
            normalized,
            None,
            None,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&mat).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "toPrufer")]
    pub fn to_prufer_wasm(&self) -> Result<String, JsError> {
        let seq = to_prufer(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&seq).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "invertPermutation")]
    pub fn invert_permutation_wasm(perm: &[u32]) -> Result<String, JsError> {
        let result = invert_permutation(perm).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "expandPathToPairs")]
    pub fn expand_path_to_pairs_wasm(path: &[u32]) -> Result<String, JsError> {
        let pairs = expand_path_to_pairs(path);
        serde_json::to_string(&pairs).map_err(|e| JsError::new(&e.to_string()))
    }
}
