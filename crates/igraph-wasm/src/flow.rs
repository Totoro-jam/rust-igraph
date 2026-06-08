use super::*;

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(js_name = "maxFlow")]
    pub fn max_flow(&self, source: u32, target: u32) -> Result<String, JsError> {
        let value = max_flow_value(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

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

    #[wasm_bindgen(js_name = "maximumCut")]
    pub fn maximum_cut(&self) -> Result<String, JsError> {
        let mc = maximum_cut(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxCutOutput {
            partition: mc.partition,
            cut_value: mc.cut_value,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "mincutValue")]
    pub fn mincut_value(&self) -> Result<String, JsError> {
        let v = mincut_value(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowResult { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "vertexDisjointPaths")]
    pub fn vertex_disjoint_paths(&self, source: u32, target: u32) -> Result<String, JsError> {
        let v = vertex_disjoint_paths(&self.inner, source, target)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "edgeDisjointPaths")]
    pub fn edge_disjoint_paths(&self, source: u32, target: u32) -> Result<String, JsError> {
        let v = edge_disjoint_paths(&self.inner, source, target)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value: v };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cohesiveBlocks")]
    pub fn cohesive_blocks(&self) -> Result<String, JsError> {
        let cb = cohesive_blocks(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = cb.blocks.len();
        let result = CohesiveBlocksResult {
            blocks: cb.blocks,
            cohesion: cb.cohesion,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "allMinimalStSeparators")]
    pub fn all_minimal_st_separators(&self) -> Result<String, JsError> {
        let seps =
            all_minimal_st_separators(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let count = seps.len();
        let result = SeparatorsResult {
            separators: seps,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "adhesion")]
    pub fn adhesion_wasm(&self) -> Result<String, JsError> {
        let value = adhesion(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cohesion")]
    pub fn cohesion_wasm(&self) -> Result<String, JsError> {
        let value = cohesion(&self.inner, true).map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "cutValue")]
    pub fn cut_value_wasm(&self, partition: &[u8]) -> Result<String, JsError> {
        let bools: Vec<bool> = partition.iter().map(|&b| b != 0).collect();
        let value = cut_value(&self.inner, &bools).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CutValueResult { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maxFlowDetailed")]
    pub fn max_flow_detailed(&self, source: u32, target: u32) -> Result<String, JsError> {
        let mf = max_flow(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = MaxFlowDetailResult {
            value: mf.value,
            flow: mf.flow,
            cut: mf.cut,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "allStCuts")]
    pub fn all_st_cuts_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let st =
            all_st_cuts(&self.inner, source, target).map_err(|e| JsError::new(&e.to_string()))?;
        let count = st.cuts.len();
        let result = StCutsResult {
            cuts: st.cuts,
            partition1s: st.partition1s,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stMincutValue")]
    pub fn st_mincut_value_wasm(&self, source: u32, target: u32) -> Result<f64, JsError> {
        st_mincut_value(&self.inner, source, target, None).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stVertexConnectivity")]
    pub fn st_vertex_connectivity_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let value = st_vertex_connectivity(&self.inner, source, target, VconnNei::NumberOfNodes)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = ScalarI64Result { value };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "mincut")]
    pub fn mincut_wasm(&self) -> Result<String, JsError> {
        let result = mincut(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct MincutOut {
            value: f64,
            cut: Vec<u32>,
            partition: Vec<u32>,
        }
        let out = MincutOut {
            value: result.value,
            cut: result.cut,
            partition: result.partition,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stMincut")]
    pub fn st_mincut_wasm(&self, source: u32, target: u32) -> Result<String, JsError> {
        let result = st_mincut(&self.inner, source, target, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct StMincutOut {
            value: f64,
            cut: Vec<u32>,
            partition: Vec<u32>,
        }
        let out = StMincutOut {
            value: result.value,
            cut: result.cut,
            partition: result.partition,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "evenTarjanReduction")]
    pub fn even_tarjan_reduction_wasm(&self) -> Result<String, JsError> {
        let result =
            even_tarjan_reduction(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        let edges = result
            .graph
            .get_edgelist()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let vcount = result.graph.vcount();
        #[derive(Serialize)]
        struct EtOut {
            edges: Vec<(u32, u32)>,
            vcount: u32,
            capacity: Vec<f64>,
        }
        let out = EtOut {
            vcount,
            edges,
            capacity: result.capacity,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "maxFlowValue")]
    pub fn max_flow_value_wasm(&self, source: u32, target: u32) -> Result<f64, JsError> {
        max_flow_value(&self.inner, source, target, None).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "stEdgeConnectivity")]
    pub fn st_edge_connectivity_wasm(&self, source: u32, target: u32) -> Result<i64, JsError> {
        st_edge_connectivity(&self.inner, source, target).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "solveLsap")]
    pub fn solve_lsap_wasm(costs: &[f64], n: u32) -> Result<Vec<u32>, JsError> {
        solve_lsap(costs, n as usize).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "allStMincuts")]
    pub fn all_st_mincuts_wasm(
        &self,
        source: u32,
        target: u32,
        capacity: Option<Vec<f64>>,
    ) -> Result<String, JsError> {
        #[derive(Serialize)]
        struct Out {
            value: f64,
            cuts: Vec<Vec<u32>>,
            partition1s: Vec<Vec<u32>>,
        }
        let cap = capacity.as_deref();
        let result = all_st_mincuts(&self.inner, source, target, cap)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let out = Out {
            value: result.value,
            cuts: result.cuts,
            partition1s: result.partition1s,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "feedbackVertexSet")]
    pub fn feedback_vertex_set_wasm(&self) -> Result<String, JsError> {
        let vertices = feedback_vertex_set(&self.inner, None, FvsAlgorithm::Greedy)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let count = vertices.len();
        let result = FeedbackVertexSetResult { vertices, count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "gomoryHuTree")]
    #[allow(clippy::cast_possible_truncation)]
    pub fn gomory_hu_tree(&self) -> Result<String, JsError> {
        let ght = gomory_hu_tree(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let tree_edges: Vec<[u32; 2]> = (0..ght.tree.ecount())
            .map(|eid| {
                let (u, v) = ght.tree.edge(eid as u32).unwrap_or((0, 0));
                [u, v]
            })
            .collect();
        let result = GomoryHuResult {
            tree_edges,
            flows: ght.flows,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }
}
