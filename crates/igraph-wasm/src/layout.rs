use super::*;

#[wasm_bindgen]
impl WasmGraph {
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

    #[wasm_bindgen(js_name = "spatialEdgeLengths")]
    pub fn spatial_edge_lengths_wasm(
        &self,
        coords_flat: &[f64],
        dim: u32,
    ) -> Result<Vec<f64>, JsError> {
        use rust_igraph::DistanceMetric;
        let d = dim as usize;
        let coords: Vec<Vec<f64>> = coords_flat.chunks(d).map(<[f64]>::to_vec).collect();
        spatial_edge_lengths(&self.inner, &coords, DistanceMetric::Euclidean)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "voronoi")]
    pub fn voronoi_wasm(
        &self,
        generators: &[u32],
        mode: &str,
        tiebreaker: &str,
    ) -> Result<String, JsError> {
        let m = match mode {
            "out" => DijkstraMode::Out,
            "in" => DijkstraMode::In,
            _ => DijkstraMode::All,
        };
        let tb = match tiebreaker {
            "random" => VoronoiTiebreaker::Random,
            _ => VoronoiTiebreaker::First,
        };
        let result = voronoi(&self.inner, None, m, generators, tb, 42)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct VoronoiOut {
            membership: Vec<Option<u32>>,
            distances: Vec<f64>,
        }
        let out = VoronoiOut {
            membership: result.membership,
            distances: result.distances,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "layoutDrl3d")]
    pub fn layout_drl_3d_wasm(&self) -> Result<String, JsError> {
        let opts = DrlOptions::default();
        let coords = layout_drl_3d(&self.inner, None, &opts, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        serde_json::to_string(&coords).map_err(|e| JsError::new(&e.to_string()))
    }
}
