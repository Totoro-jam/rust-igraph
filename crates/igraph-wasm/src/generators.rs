use super::*;

#[wasm_bindgen]
impl WasmGraph {
    #[wasm_bindgen(js_name = "erdosRenyi")]
    pub fn erdos_renyi(n: u32, p: f64, seed: u64) -> Result<WasmGraph, JsError> {
        let g =
            erdos_renyi_gnp(n, p, false, false, seed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "fullGraph")]
    pub fn full_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = full_graph(n, false, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "cycleGraph")]
    pub fn cycle_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = cycle_graph(n, false, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "ringGraph")]
    pub fn ring_graph(n: u32, circular: bool) -> Result<WasmGraph, JsError> {
        let g = ring_graph(n, false, false, circular).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "wattsStrogatz")]
    pub fn watts_strogatz(n: u32, k: u32, p: f64, seed: u64) -> Result<WasmGraph, JsError> {
        let g = watts_strogatz_game(n, k, p, false, false, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "barabasiAlbert")]
    pub fn barabasi_albert(n: u32, m: u32, seed: u64) -> Result<WasmGraph, JsError> {
        let g = barabasi_game_bag(n, m, false, false, seed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner: g })
    }

    #[wasm_bindgen(js_name = "pathGraph")]
    pub fn path_graph(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = path_graph(n, directed, false).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "starGraph")]
    pub fn star_graph(n: u32) -> Result<WasmGraph, JsError> {
        let g = star_graph(n, StarMode::Undirected, 0).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "famousGraph")]
    pub fn famous_graph(name: &str) -> Result<WasmGraph, JsError> {
        let g = famous(name).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "atlasGraph")]
    pub fn atlas_graph(number: u32) -> Result<WasmGraph, JsError> {
        let g = atlas(number).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "chungLuGame")]
    pub fn chung_lu_game_wasm(
        out_weights: &[f64],
        variant: &str,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        let v = match variant {
            "maxEntropy" => ChungLuVariant::Maxent,
            _ => ChungLuVariant::Original,
        };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = chung_lu_game(out_weights, None, loops, v, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "erdosRenyiGnm")]
    pub fn erdos_renyi_gnm_wasm(
        n: u32,
        m: f64,
        directed: bool,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = erdos_renyi_gnm(n, m as u64, directed, loops, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fromPrufer")]
    pub fn from_prufer_wasm(prufer: &[u32]) -> Result<WasmGraph, JsError> {
        let g = from_prufer(prufer).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fullBipartite")]
    pub fn full_bipartite_wasm(
        n1: u32,
        n2: u32,
        directed: bool,
        mode: &str,
    ) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => BipartiteMode::In,
            "all" => BipartiteMode::All,
            _ => BipartiteMode::Out,
        };
        let fb = full_bipartite(n1, n2, directed, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: fb.graph })
    }

    #[wasm_bindgen(js_name = "grgGame")]
    pub fn grg_game_wasm(
        n: u32,
        radius: f64,
        torus: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g =
            grg_game(n, radius, torus, seed as u64).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hypercubeGraph")]
    pub fn hypercube_graph(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = hypercube(n, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "karyTree")]
    pub fn kary_tree_wasm(n: u32, children: u32, mode: &str) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => TreeMode::In,
            "undirected" => TreeMode::Undirected,
            _ => TreeMode::Out,
        };
        let g = kary_tree(n, children, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "wheelGraph")]
    pub fn wheel_graph_wasm(n: u32, mode: &str, center: u32) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => WheelMode::In,
            "undirected" => WheelMode::Undirected,
            _ => WheelMode::Out,
        };
        let g = wheel_graph(n, m, center).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "mycielskian")]
    pub fn mycielskian_wasm(&self, k: u32) -> Result<WasmGraph, JsError> {
        let g = mycielskian(&self.inner, k).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "deBruijn")]
    pub fn de_bruijn_wasm(m: u32, n: u32) -> Result<WasmGraph, JsError> {
        let g = de_bruijn(m, n).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "generalizedPetersen")]
    pub fn generalized_petersen_wasm(n: u32, k: u32) -> Result<WasmGraph, JsError> {
        let g = generalized_petersen(n, k).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "fullCitation")]
    pub fn full_citation_wasm(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = full_citation(n, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "circulant")]
    pub fn circulant_wasm(n: u32, shifts: &[i32], directed: bool) -> Result<WasmGraph, JsError> {
        let shifts64: Vec<i64> = shifts.iter().map(|&s| i64::from(s)).collect();
        let g = circulant(n, &shifts64, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "staticFitnessGame")]
    pub fn static_fitness_game_wasm(
        no_of_edges: u32,
        fitness_out: &[f64],
        loops: bool,
        multiple: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = static_fitness_game(no_of_edges, fitness_out, None, loops, multiple, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "staticPowerLawGame")]
    pub fn static_power_law_game_wasm(
        no_of_nodes: u32,
        no_of_edges: u32,
        exponent_out: f64,
        loops: bool,
        multiple: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let g = static_power_law_game(
            no_of_nodes,
            no_of_edges,
            exponent_out,
            None,
            loops,
            multiple,
            false,
            seed as u64,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "treeGameLerw")]
    pub fn tree_game_lerw_wasm(n: u32, directed: bool, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = tree_game_lerw(n, directed, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "treeGamePrufer")]
    pub fn tree_game_prufer_wasm(n: u32, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = tree_game_prufer(n, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "preferenceGame")]
    pub fn preference_game_wasm(
        nodes: u32,
        types: u32,
        pref_matrix_flat: &[f64],
        directed: bool,
        loops: bool,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let k = types as usize;
        let pref_matrix: Vec<Vec<f64>> = pref_matrix_flat.chunks(k).map(<[f64]>::to_vec).collect();
        let (g, node_types) =
            preference_game(nodes, types, None, false, &pref_matrix, directed, loops, s)
                .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PrefOut {
            edges: Vec<(u32, u32)>,
            vcount: u32,
            node_types: Vec<u32>,
        }
        let out = PrefOut {
            edges: g.get_edgelist().map_err(|e| JsError::new(&e.to_string()))?,
            vcount: g.vcount(),
            node_types,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "hsbmGame")]
    #[allow(clippy::many_single_char_names)]
    pub fn hsbm_game_wasm(
        n: u32,
        m: u32,
        rho_flat: &[f64],
        c_flat: &[f64],
        p: f64,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let k = rho_flat.len();
        let c: Vec<Vec<f64>> = c_flat.chunks(k).map(<[f64]>::to_vec).collect();
        let g = hsbm_game(n, m, rho_flat, &c, p, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "correlatedGame")]
    pub fn correlated_game_wasm(&self, corr: f64, p: f64, seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = correlated_game(&self.inner, corr, p, None, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "correlatedPairGame")]
    pub fn correlated_pair_game_wasm(
        n: u32,
        corr: f64,
        p: f64,
        directed: bool,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let (g1, g2) = correlated_pair_game(n, corr, p, directed, None, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PairOut {
            graph1_edges: Vec<(u32, u32)>,
            graph2_edges: Vec<(u32, u32)>,
            vcount: u32,
        }
        let out = PairOut {
            graph1_edges: g1
                .get_edgelist()
                .map_err(|e| JsError::new(&e.to_string()))?,
            graph2_edges: g2
                .get_edgelist()
                .map_err(|e| JsError::new(&e.to_string()))?,
            vcount: n,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "famousNames")]
    pub fn famous_names_wasm() -> String {
        let names = famous_names();
        serde_json::to_string(names).unwrap_or_else(|_| "[]".to_string())
    }

    #[wasm_bindgen(js_name = "bipartiteGameGnp")]
    pub fn bipartite_game_gnp_wasm(
        n1: u32,
        n2: u32,
        p: f64,
        directed: bool,
        mode: &str,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let bm = match mode {
            "in" => BipartiteMode::In,
            "out" => BipartiteMode::Out,
            _ => BipartiteMode::All,
        };
        let bg = bipartite_game_gnp(n1, n2, p, directed, bm, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: bg.graph })
    }

    #[wasm_bindgen(js_name = "bipartiteGameGnm")]
    pub fn bipartite_game_gnm_wasm(
        n1: u32,
        n2: u32,
        m: u32,
        directed: bool,
        mode: &str,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let bm = match mode {
            "in" => BipartiteMode::In,
            "out" => BipartiteMode::Out,
            _ => BipartiteMode::All,
        };
        let bg = bipartite_game_gnm(n1, n2, u64::from(m), directed, bm, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: bg.graph })
    }

    #[wasm_bindgen(js_name = "delaunayGraph")]
    pub fn delaunay_graph_wasm(points_flat: &[f64], dim: u32) -> Result<WasmGraph, JsError> {
        let d = dim as usize;
        let points: Vec<Vec<f64>> = points_flat.chunks(d).map(<[f64]>::to_vec).collect();
        let g = delaunay_graph(&points).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "squareLattice")]
    pub fn square_lattice_wasm(
        dims: &[u32],
        nei: u32,
        directed: bool,
        mutual: bool,
    ) -> Result<WasmGraph, JsError> {
        let g = square_lattice(dims, nei, directed, mutual, None)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "triangularLattice")]
    pub fn triangular_lattice_wasm(
        dims: &[u32],
        directed: bool,
        mutual: bool,
    ) -> Result<WasmGraph, JsError> {
        let g =
            triangular_lattice(dims, directed, mutual).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hexagonalLattice")]
    pub fn hexagonal_lattice_wasm(
        dims: &[u32],
        directed: bool,
        mutual: bool,
    ) -> Result<WasmGraph, JsError> {
        let g =
            hexagonal_lattice(dims, directed, mutual).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "kautzGraph")]
    pub fn kautz_wasm(m: u32, n: u32) -> Result<WasmGraph, JsError> {
        let g = kautz(m, n).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "lcfGraph")]
    pub fn lcf_wasm(n: u32, shifts: &[i32], repeats: u32) -> Result<WasmGraph, JsError> {
        let shifts64: Vec<i64> = shifts.iter().map(|&s| i64::from(s)).collect();
        let g = lcf(n, &shifts64, repeats).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "turanGraph")]
    pub fn turan_wasm(n: u32, r: u32) -> Result<WasmGraph, JsError> {
        let result = turan(n, r).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph {
            inner: result.graph,
        })
    }

    #[wasm_bindgen(js_name = "regularTree")]
    pub fn regular_tree_wasm(h: u32, k: u32, mode: &str) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => TreeMode::In,
            "out" => TreeMode::Out,
            _ => TreeMode::Undirected,
        };
        let g = regular_tree(h, k, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "symmetricTree")]
    pub fn symmetric_tree_wasm(branches: &[u32], mode: &str) -> Result<WasmGraph, JsError> {
        let m = match mode {
            "in" => TreeMode::In,
            "out" => TreeMode::Out,
            _ => TreeMode::Undirected,
        };
        let g = symmetric_tree(branches, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hammingGraph")]
    pub fn hamming_wasm(n: u32, q: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = hamming(n, q, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "gabrielGraph")]
    pub fn gabriel_graph_wasm(points_flat: &[f64], dim: u32) -> Result<WasmGraph, JsError> {
        let d = dim as usize;
        let points: Vec<Vec<f64>> = points_flat.chunks(d).map(<[f64]>::to_vec).collect();
        let g = gabriel_graph(&points).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "relativeNeighborhoodGraph")]
    pub fn relative_neighborhood_graph_wasm(
        points_flat: &[f64],
        dim: u32,
    ) -> Result<WasmGraph, JsError> {
        let d = dim as usize;
        let points: Vec<Vec<f64>> = points_flat.chunks(d).map(<[f64]>::to_vec).collect();
        let g = relative_neighborhood_graph(&points).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "nearestNeighborGraph")]
    pub fn nearest_neighbor_graph_wasm(
        points_flat: &[f64],
        dim: u32,
        k: u32,
    ) -> Result<WasmGraph, JsError> {
        use rust_igraph::DistanceMetric;
        let d = dim as usize;
        let points: Vec<Vec<f64>> = points_flat.chunks(d).map(<[f64]>::to_vec).collect();
        let g = nearest_neighbor_graph(
            &points,
            DistanceMetric::Euclidean,
            i64::from(k),
            f64::INFINITY,
            false,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "erdosRenyiGnp")]
    pub fn erdos_renyi_gnp_wasm(
        n: u32,
        p: f64,
        directed: bool,
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g =
            erdos_renyi_gnp(n, p, directed, loops, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "kRegularGame")]
    pub fn k_regular_game_wasm(
        n: u32,
        k: u32,
        directed: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g =
            k_regular_game(n, k, directed, false, s).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "wattsStrogatzGame")]
    pub fn watts_strogatz_game_wasm(
        size: u32,
        nei: u32,
        p: f64,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = watts_strogatz_game(size, nei, p, false, false, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "forestFireGame")]
    pub fn forest_fire_game_wasm(
        n: u32,
        fw_prob: f64,
        bw_factor: f64,
        directed: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = forest_fire_game(n, fw_prob, bw_factor, 1, directed, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "growingRandomGame")]
    pub fn growing_random_game_wasm(
        n: u32,
        m: u32,
        directed: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = growing_random_game(n, m, directed, false, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hypercube")]
    pub fn hypercube_wasm(n: u32, directed: bool) -> Result<WasmGraph, JsError> {
        let g = hypercube(n, directed).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "mycielskiGraph")]
    pub fn mycielski_graph_wasm(k: u32) -> Result<WasmGraph, JsError> {
        let g = mycielski_graph(k).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "famous")]
    pub fn famous_wasm(name: &str) -> Result<WasmGraph, JsError> {
        let g = famous(name).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "simpleInterconnectedIslandsGame")]
    pub fn simple_interconnected_islands_game_wasm(
        islands_n: u32,
        islands_size: u32,
        islands_pin: f64,
        n_inter: u32,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g =
            simple_interconnected_islands_game(islands_n, islands_size, islands_pin, n_inter, s)
                .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "recentDegreeGame")]
    pub fn recent_degree_game_wasm(
        nodes: u32,
        power: f64,
        time_window: u32,
        m: u32,
        directed: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = recent_degree_game(nodes, power, time_window, m, None, false, 1.0, directed, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "extendedChordalRing")]
    pub fn extended_chordal_ring_wasm(
        nodes: u32,
        w_flat: &[i32],
        cols: u32,
        directed: bool,
    ) -> Result<WasmGraph, JsError> {
        let c = cols as usize;
        let w_i64: Vec<Vec<i64>> = w_flat
            .chunks(c)
            .map(|row| row.iter().map(|&v| i64::from(v)).collect())
            .collect();
        let w_refs: Vec<&[i64]> = w_i64.iter().map(Vec::as_slice).collect();
        let g = extended_chordal_ring(nodes, &w_refs, directed)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "realizeDirectedDegreeSequence")]
    pub fn realize_directed_degree_sequence_wasm(
        outdeg: &[u32],
        indeg: &[u32],
    ) -> Result<WasmGraph, JsError> {
        use rust_igraph::RealizeDegseqMethod;
        let g = realize_directed_degree_sequence(outdeg, indeg, RealizeDegseqMethod::Smallest)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "degreeSequenceGame")]
    pub fn degree_sequence_game_wasm(out_degrees: &[u32], seed: f64) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let g = degree_sequence_game_configuration(out_degrees, None, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "sbmGame")]
    pub fn sbm_game_wasm(
        pref_matrix_flat: &[f64],
        n_blocks: u32,
        block_sizes: &[u32],
        directed: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let nb = n_blocks as usize;
        let pref_matrix: Vec<Vec<f64>> = pref_matrix_flat.chunks(nb).map(<[f64]>::to_vec).collect();
        let g = sbm_game(&pref_matrix, block_sizes, directed, false, false, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "treeFromParentVector")]
    pub fn tree_from_parent_vector_wasm(parents: &[i32], mode: &str) -> Result<WasmGraph, JsError> {
        use rust_igraph::TreeMode;
        let m = match mode {
            "in" => TreeMode::In,
            _ => TreeMode::Out,
        };
        let parents_i64: Vec<i64> = parents.iter().map(|&p| i64::from(p)).collect();
        let g =
            tree_from_parent_vector(&parents_i64, m).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "adjacencyMatrix")]
    pub fn adjacency_matrix_wasm(matrix: &[i64], n: u32, mode: &str) -> Result<WasmGraph, JsError> {
        let n_us = n as usize;
        let rows: Vec<Vec<i64>> = matrix.chunks(n_us).map(<[i64]>::to_vec).collect();
        let refs: Vec<&[i64]> = rows.iter().map(Vec::as_slice).collect();
        let adj_mode = match mode {
            "undirected" => AdjacencyMode::Undirected,
            "upper" => AdjacencyMode::Upper,
            "lower" => AdjacencyMode::Lower,
            "min" => AdjacencyMode::Min,
            "max" => AdjacencyMode::Max,
            "plus" => AdjacencyMode::Plus,
            _ => AdjacencyMode::Directed,
        };
        let g = adjacency(&refs, adj_mode, LoopsMode::Once)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "biadjacencyMatrix")]
    pub fn biadjacency_matrix_wasm(
        matrix: &[f64],
        nrow: u32,
        ncol: u32,
        directed: bool,
    ) -> Result<String, JsError> {
        #[derive(Serialize)]
        struct Out {
            edges: Vec<[u32; 2]>,
            vcount: u32,
            types: Vec<bool>,
        }
        let nr = nrow as usize;
        let nc = ncol as usize;
        let rows: Vec<Vec<f64>> = matrix.chunks(nc).take(nr).map(<[f64]>::to_vec).collect();
        let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
        let result = biadjacency(&refs, directed, BipartiteMode::All, false)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..result.graph.ecount())
            .map(|e| {
                let (s, t) = result.graph.edge(e as u32).unwrap_or((0, 0));
                [s, t]
            })
            .collect();
        let out = Out {
            edges,
            vcount: result.graph.vcount(),
            types: result.types,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "grgGameWithCoords")]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn grg_game_with_coords_wasm(
        n: u32,
        radius: f64,
        torus: bool,
        seed: f64,
    ) -> Result<String, JsError> {
        #[derive(Serialize)]
        struct Out {
            edges: Vec<[u32; 2]>,
            vcount: u32,
            x: Vec<f64>,
            y: Vec<f64>,
        }
        let (g, xs, ys) = grg_game_with_coords(n, radius, torus, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let edges: Vec<[u32; 2]> = (0..g.ecount())
            .map(|e| {
                let (s, t) = g.edge(e as u32).unwrap_or((0, 0));
                [s, t]
            })
            .collect();
        let out = Out {
            edges,
            vcount: g.vcount(),
            x: xs,
            y: ys,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "asymmetricPreferenceGame")]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn asymmetric_preference_game_wasm(
        nodes: u32,
        no_out_types: u32,
        no_in_types: u32,
        pref_matrix_flat: &[f64],
        loops: bool,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        let cols = no_in_types as usize;
        let pref: Vec<Vec<f64>> = pref_matrix_flat.chunks(cols).map(<[f64]>::to_vec).collect();
        let (g, _, _) = asymmetric_preference_game(
            nodes,
            no_out_types,
            no_in_types,
            None,
            &pref,
            loops,
            seed as u64,
        )
        .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: g })
    }

    #[wasm_bindgen(js_name = "hsbmListGame")]
    #[allow(clippy::cast_sign_loss)]
    pub fn hsbm_list_game_wasm(
        nodes: u32,
        m_list: &[u32],
        prob: f64,
        seed: f64,
    ) -> Result<WasmGraph, JsError> {
        let levels = m_list.len();
        let rho: Vec<Vec<f64>> = (0..levels)
            .map(|idx| {
                let sz = m_list[idx] as usize;
                let weight = 1.0 / sz as f64;
                vec![weight; sz]
            })
            .collect();
        let conn: Vec<Vec<Vec<f64>>> = (0..levels).map(|_| vec![vec![1.0]]).collect();
        let graph = hsbm_list_game(nodes, m_list, &rho, &conn, prob, seed as u64)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmGraph { inner: graph })
    }

    #[wasm_bindgen(js_name = "sir")]
    pub fn sir_wasm(
        &self,
        beta: f64,
        gamma: f64,
        no_sim: u32,
        seed: f64,
    ) -> Result<String, JsError> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let s = seed as u64;
        let results = sir(&self.inner, beta, gamma, no_sim as usize, s)
            .map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct SirOut {
            times: Vec<f64>,
            no_s: Vec<usize>,
            no_i: Vec<usize>,
            no_r: Vec<usize>,
        }
        let out: Vec<SirOut> = results
            .into_iter()
            .map(|r| SirOut {
                times: r.times,
                no_s: r.no_s,
                no_i: r.no_i,
                no_r: r.no_r,
            })
            .collect();
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "bondPercolation")]
    pub fn bond_percolation_wasm(&self, edge_order: &[u32]) -> Result<String, JsError> {
        let result =
            bond_percolation(&self.inner, edge_order).map_err(|e| JsError::new(&e.to_string()))?;
        #[derive(Serialize)]
        struct PercolationOut {
            giant_size: Vec<u32>,
            vertex_count: Vec<u32>,
        }
        let out = PercolationOut {
            giant_size: result.giant_size,
            vertex_count: result.vertex_count,
        };
        serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
    }
}
