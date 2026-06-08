use super::*;

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(js_name = "writeGml")]
    pub fn write_gml(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_gml(&self.inner, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeDot")]
    pub fn write_dot(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_dot(&self.inner, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeGraphml")]
    pub fn write_graphml(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_graphml(&self.inner, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "readEdgelist")]
    pub fn read_edgelist_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let g = read_edgelist(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "readGml")]
    pub fn read_gml_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let g = read_gml(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "writeEdgelist")]
    pub fn write_edgelist_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_edgelist(&self.inner, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writePajek")]
    pub fn write_pajek_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_pajek(&self.inner, None, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeNcol")]
    pub fn write_ncol_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_ncol(&self.inner, None, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "readGraphml")]
    pub fn read_graphml_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_graphml(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "readPajek")]
    pub fn read_pajek_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_pajek(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "readNcol")]
    pub fn read_ncol_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_ncol(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "readLgl")]
    pub fn read_lgl_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_lgl(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "writeLgl")]
    pub fn write_lgl_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_lgl(&self.inner, None, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "readDl")]
    pub fn read_dl_wasm(text: &str, directed: bool) -> Result<WasmGraph, JsError> {
        let result =
            read_dl(text.as_bytes(), directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "readDot")]
    pub fn read_dot_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_dot(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "readLeda")]
    pub fn read_leda_wasm(text: &str) -> Result<WasmGraph, JsError> {
        let result = read_leda(text.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "writeDl")]
    pub fn write_dl_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_dl(&self.inner, None, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "writeLeda")]
    pub fn write_leda_wasm(&self) -> Result<String, JsError> {
        let mut buf = Vec::new();
        write_leda(&self.inner, None, None, &mut buf).map_err(|e| JsError::new(&e.to_string()))?;
        String::from_utf8(buf).map_err(|e| JsError::new(&e.to_string()))
    }
}
