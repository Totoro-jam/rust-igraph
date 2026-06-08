use super::*;

#[wasm_bindgen]
impl WasmGraph {
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

    #[wasm_bindgen(js_name = "difference")]
    pub fn difference_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g = difference(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "graphJoin")]
    pub fn join_wasm(&self, other: &WasmGraph) -> Result<WasmGraph, JsError> {
        let g = join(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "rewire")]
    pub fn rewire_wasm(
        &self,
        num_trials: u32,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = rewire(&self.inner, num_trials as usize, loops, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "linegraph")]
    pub fn linegraph_wasm(&self) -> Result<WasmGraph, JsError> {
        let g = linegraph(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "reverse")]
    pub fn reverse_wasm(&self) -> Result<WasmGraph, JsError> {
        let g = reverse(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "disjointUnionMany")]
    pub fn disjoint_union_many_wasm(graphs: Vec<WasmGraph>) -> Result<WasmGraph, JsError> {
        let refs: Vec<&Graph> = graphs.iter().map(|g| &g.inner).collect();
        let g = disjoint_union_many(&refs).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "unionMany")]
    pub fn union_many_wasm(graphs: Vec<WasmGraph>) -> Result<WasmGraph, JsError> {
        let refs: Vec<&Graph> = graphs.iter().map(|g| &g.inner).collect();
        let g = union_many(&refs).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "intersectionMany")]
    pub fn intersection_many_wasm(graphs: Vec<WasmGraph>) -> Result<WasmGraph, JsError> {
        let refs: Vec<&Graph> = graphs.iter().map(|g| &g.inner).collect();
        let g = intersection_many(&refs).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }
}
