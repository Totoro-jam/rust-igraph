use super::*;

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(js_name = "canonicalPermutation")]
    pub fn canonical_permutation(&self) -> Result<String, JsError> {
        let perm =
            canonical_permutation(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = CanonicalResult { permutation: perm };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "countAutomorphisms")]
    pub fn count_automorphisms(&self) -> Result<String, JsError> {
        let count =
            count_automorphisms(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let result = AutomorphismResult { count };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isomorphicBliss")]
    pub fn isomorphic_bliss(&self, other: &WasmGraph) -> Result<String, JsError> {
        let r = isomorphic_bliss(&self.inner, &other.inner, None, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let result = IsomorphismResult {
            isomorphic: r.iso,
            mapping: r.map12,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "automorphismGroup")]
    pub fn automorphism_group(&self) -> Result<String, JsError> {
        let gens =
            automorphism_group(&self.inner, None).map_err(|e| JsError::new(&e.to_string()))?;
        let count = gens.len();
        let result = AutomorphismGroupResult {
            generators: gens,
            count,
        };
        serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isomorphic")]
    pub fn isomorphic_wasm(&self, other: &WasmGraph) -> Result<bool, JsError> {
        isomorphic(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "subisomorphic")]
    pub fn subisomorphic_wasm(&self, other: &WasmGraph) -> Result<bool, JsError> {
        subisomorphic(&self.inner, &other.inner).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "isSameGraph")]
    pub fn is_same_graph_wasm(&self, other: &WasmGraph) -> bool {
        is_same_graph(&self.inner, &other.inner)
    }
}
