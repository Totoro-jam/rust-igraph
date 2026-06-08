use super::*;

#[wasm_bindgen]
impl WasmGraph {
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

    pub fn dfs(&self, root: u32) -> Result<String, JsError> {
        let order = dfs(&self.inner, root).map_err(|e| JsError::new(&e.to_string()))?;
        let result = DfsResult { order };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "topologicalSort")]
    pub fn topological_sort(&self) -> Result<String, JsError> {
        let order = topological_sorting(&self.inner, DijkstraMode::Out)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = TopoSortResult { order };
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

    pub fn eccentricity(&self) -> Result<String, JsError> {
        let values = eccentricity(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = EccentricityResult { values };
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

    #[wasm_bindgen(js_name = "minimumSpanningTree")]
    pub fn minimum_spanning_tree(&self, weights: Option<Vec<f64>>) -> Result<String, JsError> {
        let w = weights.as_deref();
        let edges = minimum_spanning_tree(&self.inner, w, MstAlgorithm::Prim)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = MstResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "feedbackArcSet")]
    pub fn feedback_arc_set(&self, weights: Option<Vec<f64>>) -> Result<String, JsError> {
        let w = weights.as_deref();
        let edges = feedback_arc_set(&self.inner, w, FasAlgorithm::EadesLinSmyth)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = FeedbackArcSetResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "graphCenter")]
    pub fn graph_center(&self) -> Result<String, JsError> {
        let mode = EccMode::All;
        let vertices = graph_center(&self.inner, mode).map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = GraphCenterResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "averagePathLength")]
    pub fn average_path_length(&self) -> Result<String, JsError> {
        let value = self
            .inner
            .average_path_length()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = AveragePathLengthResult { value };
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

    #[wasm_bindgen(js_name = "spanner")]
    pub fn spanner(&self, stretch: f64) -> Result<String, JsError> {
        let edges =
            spanner(&self.inner, stretch, None).map_err(|e| JsError::new(&e.to_string()))?;
        let count = edges.len();
        let result = SpannerResult { edges, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
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

    #[wasm_bindgen(js_name = "averageLocalEfficiency")]
    pub fn average_local_efficiency_wasm(&self) -> Result<f64, JsError> {
        average_local_efficiency(&self.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "johnsonDistances")]
    pub fn johnson_distances_wasm(&self, weights: &[f64]) -> Result<String, JsError> {
        let dists =
            johnson_distances(&self.inner, weights).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&dists).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "topologicalSorting")]
    pub fn topological_sorting_wasm(&self, mode: &str) -> Result<Vec<u32>, JsError> {
        let m = match mode {
            "in" => DijkstraMode::In,
            _ => DijkstraMode::Out,
        };
        topological_sorting(&self.inner, m).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "meanDistanceWeighted")]
    pub fn mean_distance_weighted_wasm(
        &self,
        weights: &[f64],
        directed: bool,
    ) -> Result<f64, JsError> {
        let r = mean_distance_weighted(&self.inner, weights, directed, true)
            .map_err(|e| JsError::new(&e.to_string()))?;
        r.ok_or_else(|| JsError::new("mean distance undefined (disconnected graph)"))
    }

    #[wasm_bindgen(js_name = "reachabilityMatrix")]
    pub fn reachability_matrix_wasm(&self) -> Result<String, JsError> {
        let mat = reachability_matrix(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&mat).map_err(|e| JsError::new(&e.to_string()))
    }
}
