#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::doc_markdown,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::cast_lossless
)]

//! Spinglass community detection (Reichardt-Bornholdt 2006).
//!
//! Implements the community structure detection algorithm based on the
//! Potts model with simulated annealing. Vertices are assigned spin states
//! (1..q) and the Hamiltonian is minimized via heat-bath updates at
//! decreasing temperatures.
//!
//! Two null models:
//! - **Simple** (G(n,p)): baseline is a random graph with edge probability p
//! - **Config** (configuration model): baseline respects the degree sequence
//!
//! Two update modes:
//! - **Sequential**: randomly pick one node per step, update immediately
//! - **Parallel**: compute optimal spins for all nodes, then update all at once
//!
//! Reference: Reichardt & Bornholdt, "Statistical Mechanics of Community
//! Detection", Phys. Rev. E 74, 016110 (2006).
//!
//! # Example
//!
//! ```
//! use rust_igraph::{Graph, spinglass};
//!
//! // Two triangles connected by a single edge
//! let g = Graph::from_edges(&[(0,1),(1,2),(2,0),(3,4),(4,5),(5,3),(2,3)], false, None).unwrap();
//! let result = spinglass(&g, None).unwrap();
//! assert!(result.nb_clusters >= 2);
//! ```

use crate::core::{Graph, IgraphError, IgraphResult};

/// Update rule for the spinglass null model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinglassUpdateRule {
    /// G(n,p) random graph null model.
    Simple,
    /// Configuration model (degree-preserving) null model.
    Config,
}

/// Result of spinglass community detection.
#[derive(Debug, Clone)]
pub struct SpinglassResult {
    /// Community membership for each vertex (0-indexed).
    pub membership: Vec<u32>,
    /// Generalized modularity of the partition.
    pub modularity: f64,
    /// Final temperature of the simulated annealing.
    pub temperature: f64,
    /// Number of communities found.
    pub nb_clusters: u32,
    /// Size of each community.
    pub csize: Vec<u32>,
}

/// Options for spinglass community detection.
#[derive(Debug, Clone)]
pub struct SpinglassOptions {
    /// Number of spins (maximum number of communities). Default: 25.
    pub spins: u32,
    /// Whether to use parallel update mode. Default: false.
    pub parallel_update: bool,
    /// Starting temperature. Default: 1.0.
    pub start_temp: f64,
    /// Stopping temperature. Default: 0.01.
    pub stop_temp: f64,
    /// Cooling factor for simulated annealing. Default: 0.99.
    pub cool_fact: f64,
    /// Update rule (null model). Default: Config.
    pub update_rule: SpinglassUpdateRule,
    /// Resolution parameter gamma. Default: 1.0.
    pub gamma: f64,
    /// Random seed for reproducibility. Default: 42.
    pub seed: u64,
}

impl Default for SpinglassOptions {
    fn default() -> Self {
        Self {
            spins: 25,
            parallel_update: false,
            start_temp: 1.0,
            stop_temp: 0.01,
            cool_fact: 0.99,
            update_rule: SpinglassUpdateRule::Config,
            gamma: 1.0,
            seed: 42,
        }
    }
}

/// Default number of spins for spinglass.
pub const SPINGLASS_DEFAULT_SPINS: u32 = 25;

// ── Internal PRNG ──────────────────────────────────────────────────
// Same SplitMix64 used in other community algorithms.

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn gen_range(&mut self, lo: u64, hi: u64) -> u64 {
        if lo >= hi {
            return lo;
        }
        let range = hi - lo;
        lo + self.next_u64() % range
    }

    fn gen_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    fn gen_f64_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.gen_f64() * (hi - lo)
    }
}

// ── Internal Potts Model ───────────────────────────────────────────

struct PottsModel {
    /// Adjacency list: for each vertex, list of (neighbor, weight).
    adj: Vec<Vec<(usize, f64)>>,
    /// Number of vertices.
    n: usize,
    /// Number of spins (q).
    q: usize,
    /// Operation mode: 0 = Simple, 1 = Config.
    operation_mode: usize,
    /// Spin assignment per vertex (1..q).
    spin: Vec<u32>,
    /// Per-spin count/weight ("color_field"):
    /// mode 0: count of vertices per spin
    /// mode 1: sum of vertex degrees per spin
    color_field: Vec<f64>,
    /// Total degree sum.
    total_degree_sum: f64,
    /// Vertex weights (sum of incident edge weights).
    vertex_weight: Vec<f64>,
    /// Qmatrix[i][j] = sum of edge weights between spin-i and spin-j vertices.
    q_matrix: Vec<Vec<f64>>,
    /// Qa[i] = sum over j of Qmatrix[i][j].
    qa: Vec<f64>,
    /// Sum of all edge weights (each edge counted once).
    sum_weights: f64,
    /// Acceptance ratio.
    acceptance: f64,
    /// PRNG.
    rng: SplitMix64,
}

impl PottsModel {
    fn new(
        graph: &Graph,
        weights: Option<&[f64]>,
        q: usize,
        operation_mode: usize,
        seed: u64,
    ) -> IgraphResult<Self> {
        let n = graph.vcount() as usize;
        let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        let mut sum_weights = 0.0;

        for eid in 0..graph.ecount() {
            let (u, v) = graph.edge(eid as u32)?;
            let w = weights.map_or(1.0, |ws| ws[eid]);
            adj[u as usize].push((v as usize, w));
            if u != v {
                adj[v as usize].push((u as usize, w));
            }
            sum_weights += w;
        }

        let mut vertex_weight = vec![0.0; n];
        for v in 0..n {
            for &(_, w) in &adj[v] {
                vertex_weight[v] += w;
            }
        }

        Ok(Self {
            adj,
            n,
            q,
            operation_mode,
            spin: vec![0; n],
            color_field: vec![0.0; q + 1],
            total_degree_sum: 0.0,
            vertex_weight,
            q_matrix: vec![vec![0.0; q + 1]; q + 1],
            qa: vec![0.0; q + 1],
            sum_weights,
            acceptance: 0.0,
            rng: SplitMix64(seed),
        })
    }

    fn assign_initial_conf(&mut self) {
        for i in 0..=self.q {
            self.color_field[i] = 0.0;
        }
        self.total_degree_sum = 0.0;
        for v in 0..self.n {
            let s = (self.rng.gen_range(1, self.q as u64 + 1)) as u32;
            self.spin[v] = s;
            let s_idx = s as usize;
            let deg = self.vertex_weight[v];
            if self.operation_mode == 0 {
                self.color_field[s_idx] += 1.0;
            } else {
                self.color_field[s_idx] += deg;
            }
            self.total_degree_sum += deg;
        }
    }

    fn initialize_q_matrix(&mut self) -> f64 {
        for i in 0..=self.q {
            self.qa[i] = 0.0;
            for j in 0..=self.q {
                self.q_matrix[i][j] = 0.0;
            }
        }
        // Walk all edges once via the adjacency representation
        // To avoid double-counting, we walk graph edges directly
        // adj has both directions, so we need to be careful
        // Actually we stored adj symmetrically, so each edge appears twice.
        // We accumulate from adj but divide by 2... no, the C code also
        // counts each edge twice in Qmatrix (both [i][j] and [j][i]).
        // Let's replicate: for each directed (u->v) entry in adj, add to Qmatrix.
        // Since adj is symmetric, each undirected edge gives two entries,
        // which fills both Qmatrix[si][sj] and Qmatrix[sj][si].
        for v in 0..self.n {
            let sv = self.spin[v] as usize;
            for &(u, w) in &self.adj[v] {
                let su = self.spin[u] as usize;
                self.q_matrix[sv][su] += w;
            }
        }
        // But wait - in the C code, each edge contributes to BOTH [i][j] and [j][i].
        // Our adj is already symmetric (each edge stored twice), so each undirected
        // edge contributes w to q_matrix[si][sj] from v's perspective and
        // w to q_matrix[sj][si] from u's perspective. This matches the C behavior.

        for i in 0..=self.q {
            for j in 0..=self.q {
                self.qa[i] += self.q_matrix[i][j];
            }
        }
        self.calculate_q()
    }

    fn calculate_q(&self) -> f64 {
        let two_m = 2.0 * self.sum_weights;
        if two_m == 0.0 {
            return 0.0;
        }
        let mut q_val = 0.0;
        for i in 0..=self.q {
            q_val += self.q_matrix[i][i] - self.qa[i] * self.qa[i] / two_m;
        }
        q_val / two_m
    }

    fn find_start_temp(&mut self, gamma: f64, prob: f64, ts: f64) -> f64 {
        let mut kt = ts;
        self.assign_initial_conf();
        self.initialize_q_matrix();
        let threshold = (1.0 - 1.0 / self.q as f64) * 0.95;
        while self.acceptance < threshold {
            kt *= 1.1;
            self.heat_bath_parallel_lookup(gamma, prob, kt, 50);
        }
        kt * 1.1
    }

    /// Sequential (random node) heat-bath update at temperature kT.
    fn heat_bath_lookup(&mut self, gamma: f64, prob_base: f64, kt: f64, max_sweeps: u32) -> f64 {
        let mut changes: u64 = 0;
        let mut neighbours = vec![0.0; self.q + 1];
        let mut weights_arr = vec![0.0; self.q + 1];

        for _sweep in 0..max_sweeps {
            for _step in 0..self.n {
                let rn = self.rng.gen_range(0, self.n as u64) as usize;
                // Count neighbours per spin
                for i in 0..=self.q {
                    neighbours[i] = 0.0;
                    weights_arr[i] = 0.0;
                }
                let degree = self.vertex_weight[rn];
                for &(nbr, w) in &self.adj[rn] {
                    neighbours[self.spin[nbr] as usize] += w;
                }

                let old_spin = self.spin[rn] as usize;
                let (prob, delta) = match self.operation_mode {
                    0 => (prob_base, 1.0),
                    _ => (degree / self.total_degree_sum, degree),
                };

                let beta = 1.0 / kt;
                let mut minweight = 0.0_f64;
                weights_arr[old_spin] = 0.0;
                for spin in 1..=self.q {
                    if spin != old_spin {
                        let h = self.color_field[spin] - (self.color_field[old_spin] - delta);
                        weights_arr[spin] =
                            neighbours[old_spin] - neighbours[spin] + gamma * prob * h;
                        if weights_arr[spin] < minweight {
                            minweight = weights_arr[spin];
                        }
                    }
                }
                let mut norm = 0.0;
                for spin in 1..=self.q {
                    weights_arr[spin] -= minweight;
                    weights_arr[spin] = (-beta * weights_arr[spin]).exp();
                    norm += weights_arr[spin];
                }

                let mut r = self.rng.gen_f64_range(0.0, norm);
                let mut spin_opt = old_spin;
                for spin in 1..=self.q {
                    if r <= weights_arr[spin] {
                        spin_opt = spin;
                        break;
                    }
                    r -= weights_arr[spin];
                }

                let new_spin = spin_opt;
                if new_spin != old_spin {
                    changes += 1;
                    self.spin[rn] = new_spin as u32;
                    self.color_field[old_spin] -= delta;
                    self.color_field[new_spin] += delta;
                    // Qmatrix update
                    for &(nbr, w) in &self.adj[rn] {
                        let sn = self.spin[nbr] as usize;
                        self.q_matrix[old_spin][sn] -= w;
                        self.q_matrix[new_spin][sn] += w;
                        self.q_matrix[sn][old_spin] -= w;
                        self.q_matrix[sn][new_spin] += w;
                        self.qa[old_spin] -= w;
                        self.qa[new_spin] += w;
                    }
                }
            }
        }
        self.acceptance = changes as f64 / self.n as f64 / max_sweeps as f64;
        self.acceptance
    }

    /// Parallel heat-bath update at temperature kT.
    fn heat_bath_parallel_lookup(
        &mut self,
        gamma: f64,
        prob_base: f64,
        kt: f64,
        max_sweeps: u32,
    ) -> i64 {
        let mut changes: i64 = 1;
        let mut neighbours = vec![0.0; self.q + 1];
        let mut weights_arr = vec![0.0; self.q + 1];
        let mut new_spins = vec![0u32; self.n];
        let mut prev_spins = vec![0u32; self.n];

        let mut sweep = 0u32;
        while sweep < max_sweeps && changes > 0 {
            let mut cyclic = true;
            sweep += 1;
            changes = 0;

            // Phase 1: compute new spins for all nodes
            for v in 0..self.n {
                for i in 0..=self.q {
                    neighbours[i] = 0.0;
                    weights_arr[i] = 0.0;
                }
                let degree = self.vertex_weight[v];
                for &(nbr, w) in &self.adj[v] {
                    neighbours[self.spin[nbr] as usize] += w;
                }

                let old_spin = self.spin[v] as usize;
                let (prob, delta) = match self.operation_mode {
                    0 => (prob_base, 1.0),
                    _ => (degree / self.total_degree_sum, degree),
                };

                let beta = 1.0 / kt;
                let mut minweight = 0.0_f64;
                weights_arr[old_spin] = 0.0;
                for spin in 1..=self.q {
                    if spin != old_spin {
                        let h = self.color_field[spin] + delta - self.color_field[old_spin];
                        weights_arr[spin] =
                            neighbours[old_spin] - neighbours[spin] + gamma * prob * h;
                        if weights_arr[spin] < minweight {
                            minweight = weights_arr[spin];
                        }
                    }
                }
                let mut norm = 0.0;
                for spin in 1..=self.q {
                    weights_arr[spin] -= minweight;
                    weights_arr[spin] = (-beta * weights_arr[spin]).exp();
                    norm += weights_arr[spin];
                }

                let mut r = self.rng.gen_f64_range(0.0, norm);
                let mut spin_opt = old_spin;
                for spin in 1..=self.q {
                    if r <= weights_arr[spin] {
                        spin_opt = spin;
                        break;
                    }
                    r -= weights_arr[spin];
                }
                new_spins[v] = spin_opt as u32;
            }

            // Phase 2: apply all spin changes
            for v in 0..self.n {
                let old_spin = self.spin[v] as usize;
                let new_spin = new_spins[v] as usize;
                if new_spin != old_spin {
                    changes += 1;
                    self.spin[v] = new_spin as u32;
                    if new_spins[v] != prev_spins[v] {
                        cyclic = false;
                    }
                    prev_spins[v] = old_spin as u32;
                    let delta = if self.operation_mode == 0 {
                        1.0
                    } else {
                        self.vertex_weight[v]
                    };
                    self.color_field[old_spin] -= delta;
                    self.color_field[new_spin] += delta;
                    for &(nbr, w) in &self.adj[v] {
                        let sn = self.spin[nbr] as usize;
                        self.q_matrix[old_spin][sn] -= w;
                        self.q_matrix[new_spin][sn] += w;
                        self.q_matrix[sn][old_spin] -= w;
                        self.q_matrix[sn][new_spin] += w;
                        self.qa[old_spin] -= w;
                        self.qa[new_spin] += w;
                    }
                }
            }

            if cyclic {
                self.acceptance = 0.0;
                return 0;
            }
        }
        self.acceptance = changes as f64 / self.n as f64;
        changes
    }

    /// Sequential (random node) heat-bath update at zero temperature (greedy).
    fn heat_bath_lookup_zero_temp(&mut self, gamma: f64, prob_base: f64, max_sweeps: u32) -> f64 {
        let mut changes: u64 = 0;
        let mut neighbours = vec![0.0; self.q + 1];

        for _sweep in 0..max_sweeps {
            for _step in 0..self.n {
                let rn = self.rng.gen_range(0, self.n as u64) as usize;
                for i in 0..=self.q {
                    neighbours[i] = 0.0;
                }
                let degree = self.vertex_weight[rn];
                for &(nbr, w) in &self.adj[rn] {
                    neighbours[self.spin[nbr] as usize] += w;
                }

                let old_spin = self.spin[rn] as usize;
                let (prob, delta) = match self.operation_mode {
                    0 => (prob_base, 1.0),
                    _ => (degree / self.total_degree_sum, degree),
                };

                let mut spin_opt = old_spin;
                let mut delta_e_min = 0.0_f64;
                for spin in 1..=self.q {
                    if spin != old_spin {
                        let h = self.color_field[spin] + delta - self.color_field[old_spin];
                        let delta_e = neighbours[old_spin] - neighbours[spin] + gamma * prob * h;
                        if delta_e < delta_e_min {
                            spin_opt = spin;
                            delta_e_min = delta_e;
                        }
                    }
                }

                let new_spin = spin_opt;
                if new_spin != old_spin {
                    changes += 1;
                    self.spin[rn] = new_spin as u32;
                    self.color_field[old_spin] -= delta;
                    self.color_field[new_spin] += delta;
                    for &(nbr, w) in &self.adj[rn] {
                        let sn = self.spin[nbr] as usize;
                        self.q_matrix[old_spin][sn] -= w;
                        self.q_matrix[new_spin][sn] += w;
                        self.q_matrix[sn][old_spin] -= w;
                        self.q_matrix[sn][new_spin] += w;
                        self.qa[old_spin] -= w;
                        self.qa[new_spin] += w;
                    }
                }
            }
        }
        self.acceptance = changes as f64 / self.n as f64 / max_sweeps as f64;
        self.acceptance
    }

    /// Parallel heat-bath update at zero temperature.
    fn heat_bath_parallel_lookup_zero_temp(
        &mut self,
        gamma: f64,
        prob_base: f64,
        max_sweeps: u32,
    ) -> i64 {
        let mut changes: i64 = 1;
        let mut neighbours = vec![0.0; self.q + 1];
        let mut new_spins = vec![0u32; self.n];
        let mut prev_spins = vec![0u32; self.n];

        let mut sweep = 0u32;
        while sweep < max_sweeps && changes > 0 {
            let mut cyclic = true;
            sweep += 1;
            changes = 0;

            for v in 0..self.n {
                for i in 0..=self.q {
                    neighbours[i] = 0.0;
                }
                let degree = self.vertex_weight[v];
                for &(nbr, w) in &self.adj[v] {
                    neighbours[self.spin[nbr] as usize] += w;
                }

                let old_spin = self.spin[v] as usize;
                let (prob, delta) = match self.operation_mode {
                    0 => (prob_base, 1.0),
                    _ => (degree / self.total_degree_sum, degree),
                };

                let mut spin_opt = old_spin;
                let mut delta_e_min = 0.0_f64;
                for spin in 1..=self.q {
                    if spin != old_spin {
                        let h = self.color_field[spin] + delta - self.color_field[old_spin];
                        let delta_e = neighbours[old_spin] - neighbours[spin] + gamma * prob * h;
                        if delta_e < delta_e_min {
                            spin_opt = spin;
                            delta_e_min = delta_e;
                        }
                    }
                }
                new_spins[v] = spin_opt as u32;
            }

            for v in 0..self.n {
                let old_spin = self.spin[v] as usize;
                let new_spin = new_spins[v] as usize;
                if new_spin != old_spin {
                    changes += 1;
                    self.spin[v] = new_spin as u32;
                    if new_spins[v] != prev_spins[v] {
                        cyclic = false;
                    }
                    prev_spins[v] = old_spin as u32;
                    self.color_field[old_spin] -= 1.0;
                    self.color_field[new_spin] += 1.0;
                    for &(nbr, w) in &self.adj[v] {
                        let sn = self.spin[nbr] as usize;
                        self.q_matrix[old_spin][sn] -= w;
                        self.q_matrix[new_spin][sn] += w;
                        self.q_matrix[sn][old_spin] -= w;
                        self.q_matrix[sn][new_spin] += w;
                        self.qa[old_spin] -= w;
                        self.qa[new_spin] += w;
                    }
                }
            }

            if cyclic {
                self.acceptance = 0.0;
                return 0;
            }
        }
        self.acceptance = changes as f64 / self.n as f64;
        changes
    }

    fn write_clusters(&self, gamma: f64) -> SpinglassResult {
        // Count nodes per spin and build densified mapping
        let mut nodes_per_spin = vec![0u32; self.q + 1];
        for v in 0..self.n {
            nodes_per_spin[self.spin[v] as usize] += 1;
        }

        // Build mapping from spin -> dense community id
        let mut spin_to_community = vec![0u32; self.q + 1];
        let mut nb_clusters = 0u32;
        let mut csize = Vec::new();
        for spin in 1..=self.q {
            if nodes_per_spin[spin] > 0 {
                spin_to_community[spin] = nb_clusters;
                nb_clusters += 1;
                csize.push(nodes_per_spin[spin]);
            }
        }

        let mut membership = vec![0u32; self.n];
        for v in 0..self.n {
            membership[v] = spin_to_community[self.spin[v] as usize];
        }

        // Compute generalized modularity
        let modularity = if self.sum_weights > 0.0 {
            let mut q_val = 0.0;
            for spin in 1..=self.q {
                if nodes_per_spin[spin] > 0 {
                    // Count inner and outer links for this spin
                    let mut inner = 0.0;
                    let mut total = 0.0;
                    for v in 0..self.n {
                        if self.spin[v] as usize == spin {
                            for &(nbr, w) in &self.adj[v] {
                                if self.spin[nbr] as usize == spin {
                                    inner += w;
                                }
                                total += w;
                            }
                        }
                    }
                    let t1 = inner / (2.0 * self.sum_weights);
                    let t2 = total / (2.0 * self.sum_weights);
                    q_val += t1;
                    q_val -= gamma * t2 * t2;
                }
            }
            q_val
        } else {
            0.0
        };

        SpinglassResult {
            membership,
            modularity,
            temperature: 0.0, // set by caller
            nb_clusters,
            csize,
        }
    }
}

// ── Public API ─────────────────────────────────────────────────────

/// Spinglass community detection with default options.
///
/// The graph must be connected. Direction of edges is ignored.
///
/// # Errors
///
/// Returns an error if the graph is disconnected, has fewer than 2 vertices,
/// or if weights contain negative values.
pub fn spinglass(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<SpinglassResult> {
    spinglass_with_options(graph, weights, &SpinglassOptions::default())
}

/// Spinglass community detection with edge weights.
///
/// # Errors
///
/// Returns an error if the graph is disconnected, has fewer than 2 vertices,
/// or if weights contain non-positive values.
pub fn spinglass_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<SpinglassResult> {
    spinglass_with_options(graph, Some(weights), &SpinglassOptions::default())
}

/// Spinglass community detection with full options.
///
/// # Errors
///
/// Returns an error if the graph is disconnected, has fewer than 2 vertices,
/// or if invalid options are provided.
pub fn spinglass_with_options(
    graph: &Graph,
    weights: Option<&[f64]>,
    options: &SpinglassOptions,
) -> IgraphResult<SpinglassResult> {
    let n = graph.vcount();

    // Validation
    if options.spins < 2 {
        return Err(IgraphError::InvalidArgument(
            "Number of spins must be at least 2".into(),
        ));
    }
    if let Some(ws) = weights {
        if ws.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(
                "Weight vector length must equal edge count".into(),
            ));
        }
        if ws.iter().any(|&w| w < 0.0) {
            return Err(IgraphError::InvalidArgument(
                "Edge weights must be non-negative".into(),
            ));
        }
    }
    if options.cool_fact <= 0.0 || options.cool_fact >= 1.0 {
        return Err(IgraphError::InvalidArgument(
            "Cooling factor must be in (0, 1)".into(),
        ));
    }
    if options.gamma < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "Gamma must be non-negative".into(),
        ));
    }

    let zero_t = options.start_temp == 0.0 && options.stop_temp == 0.0;
    if !zero_t {
        if options.start_temp <= 0.0 || options.stop_temp <= 0.0 {
            return Err(IgraphError::InvalidArgument(
                "Starting and stopping temperatures must be both positive or both zero".into(),
            ));
        }
        if options.start_temp <= options.stop_temp {
            return Err(IgraphError::InvalidArgument(
                "Starting temperature must be larger than stopping temperature".into(),
            ));
        }
    }

    // Trivial cases
    if n < 2 {
        let membership = (0..n).map(|_| 0u32).collect::<Vec<_>>();
        let nb_clusters = u32::from(n > 0);
        let csize = if n == 0 { vec![] } else { vec![1] };
        return Ok(SpinglassResult {
            membership,
            modularity: 0.0,
            temperature: options.stop_temp,
            nb_clusters,
            csize,
        });
    }

    // Check connectivity
    if !is_weakly_connected(graph) {
        return Err(IgraphError::InvalidArgument(
            "Spinglass community detection requires a connected graph".into(),
        ));
    }

    let operation_mode = match options.update_rule {
        SpinglassUpdateRule::Simple => 0,
        SpinglassUpdateRule::Config => 1,
    };

    let mut pm = PottsModel::new(
        graph,
        weights,
        options.spins as usize,
        operation_mode,
        options.seed,
    )?;

    let prob = if pm.n > 1 {
        2.0 * pm.sum_weights / pm.n as f64 / (pm.n as f64 - 1.0)
    } else {
        0.0
    };

    let mut kt = if zero_t {
        options.stop_temp
    } else {
        pm.find_start_temp(options.gamma, prob, options.start_temp)
    };

    // Re-assign initial configuration
    pm.assign_initial_conf();
    pm.initialize_q_matrix();

    let mut changes: i64 = 1;
    let mut runs = 0u64;
    let spins_f = options.spins as f64;
    let threshold = (1.0 - 1.0 / spins_f) * 0.01;

    while changes > 0 && (kt / options.stop_temp > 1.0 || (zero_t && runs < 150)) {
        runs += 1;
        if zero_t {
            if options.parallel_update {
                changes = pm.heat_bath_parallel_lookup_zero_temp(options.gamma, prob, 50);
            } else {
                let acc = pm.heat_bath_lookup_zero_temp(options.gamma, prob, 50);
                changes = i64::from(acc >= threshold);
            }
        } else {
            kt *= options.cool_fact;
            if options.parallel_update {
                changes = pm.heat_bath_parallel_lookup(options.gamma, prob, kt, 50);
            } else {
                let acc = pm.heat_bath_lookup(options.gamma, prob, kt, 50);
                changes = i64::from(acc >= threshold);
            }
        }
    }

    let mut result = pm.write_clusters(options.gamma);
    result.temperature = kt;
    Ok(result)
}

/// Check weak connectivity (simplified BFS).
fn is_weakly_connected(graph: &Graph) -> bool {
    let n = graph.vcount() as usize;
    if n <= 1 {
        return true;
    }
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    visited[0] = true;
    queue.push_back(0u32);
    let mut count = 1usize;
    while let Some(v) = queue.pop_front() {
        if let Ok(nbrs) = graph.neighbors(v) {
            for &nbr in &nbrs {
                let ni = nbr as usize;
                if !visited[ni] {
                    visited[ni] = true;
                    count += 1;
                    queue.push_back(nbr);
                }
            }
        }
    }
    count == n
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_two_communities() {
        // Two K4 cliques connected by a single bridge edge
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (4, 5),
                (4, 6),
                (4, 7),
                (5, 6),
                (5, 7),
                (6, 7),
                (3, 4), // bridge
            ],
            false,
            None,
        )
        .unwrap();

        let result = spinglass(&g, None).unwrap();
        assert_eq!(result.membership.len(), 8);
        assert!(
            result.nb_clusters >= 2,
            "Expected at least 2 clusters, got {}",
            result.nb_clusters
        );
        assert!(
            result.modularity > 0.0,
            "Expected positive modularity, got {}",
            result.modularity
        );
        // Cluster sizes should sum to 8
        let total: u32 = result.csize.iter().sum();
        assert_eq!(total, 8);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::from_edges(&[] as &[(u32, u32)], false, Some(1)).unwrap();
        let result = spinglass(&g, None).unwrap();
        assert_eq!(result.membership, vec![0]);
        assert_eq!(result.nb_clusters, 1);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::from_edges(&[] as &[(u32, u32)], false, Some(0)).unwrap();
        let result = spinglass(&g, None).unwrap();
        assert!(result.membership.is_empty());
        assert_eq!(result.nb_clusters, 0);
    }

    #[test]
    fn disconnected_graph_error() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let result = spinglass(&g, None);
        assert!(result.is_err());
    }

    #[test]
    fn deterministic_with_seed() {
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (4, 5),
                (4, 6),
                (4, 7),
                (5, 6),
                (5, 7),
                (6, 7),
                (3, 4),
            ],
            false,
            None,
        )
        .unwrap();

        let opts = SpinglassOptions {
            seed: 12345,
            ..SpinglassOptions::default()
        };
        let r1 = spinglass_with_options(&g, None, &opts).unwrap();
        let r2 = spinglass_with_options(&g, None, &opts).unwrap();
        assert_eq!(r1.membership, r2.membership);
    }

    #[test]
    fn weighted_edges() {
        // Two communities with strong internal edges and weak bridge
        let g = Graph::from_edges(
            &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)],
            false,
            None,
        )
        .unwrap();
        let weights = vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 0.1];
        let result = spinglass_weighted(&g, &weights).unwrap();
        assert_eq!(result.membership.len(), 6);
        assert!(result.nb_clusters >= 2);
    }

    #[test]
    fn invalid_spins() {
        let g = Graph::from_edges(&[(0, 1)], false, None).unwrap();
        let opts = SpinglassOptions {
            spins: 1,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_cool_fact() {
        let g = Graph::from_edges(&[(0, 1)], false, None).unwrap();
        let opts = SpinglassOptions {
            cool_fact: 1.5,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_gamma() {
        let g = Graph::from_edges(&[(0, 1)], false, None).unwrap();
        let opts = SpinglassOptions {
            gamma: -1.0,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn negative_weights_error() {
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
        let result = spinglass(&g, Some(&[1.0, -1.0, 1.0]));
        assert!(result.is_err());
    }

    #[test]
    fn simple_update_rule() {
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (4, 5),
                (4, 6),
                (4, 7),
                (5, 6),
                (5, 7),
                (6, 7),
                (3, 4),
            ],
            false,
            None,
        )
        .unwrap();

        let opts = SpinglassOptions {
            update_rule: SpinglassUpdateRule::Simple,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts).unwrap();
        assert!(result.nb_clusters >= 2);
    }

    #[test]
    fn parallel_update() {
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (4, 5),
                (4, 6),
                (4, 7),
                (5, 6),
                (5, 7),
                (6, 7),
                (3, 4),
            ],
            false,
            None,
        )
        .unwrap();

        let opts = SpinglassOptions {
            parallel_update: true,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts).unwrap();
        assert!(result.nb_clusters >= 1);
    }

    #[test]
    fn zero_temperature() {
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (4, 5),
                (4, 6),
                (4, 7),
                (5, 6),
                (5, 7),
                (6, 7),
                (3, 4),
            ],
            false,
            None,
        )
        .unwrap();

        let opts = SpinglassOptions {
            start_temp: 0.0,
            stop_temp: 0.0,
            ..SpinglassOptions::default()
        };
        let result = spinglass_with_options(&g, None, &opts).unwrap();
        assert!(result.nb_clusters >= 1);
    }

    #[test]
    fn karate_club() {
        let edges: &[(u32, u32)] = &[
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
            (0, 17),
            (0, 19),
            (0, 21),
            (0, 31),
            (1, 2),
            (1, 3),
            (1, 7),
            (1, 13),
            (1, 17),
            (1, 19),
            (1, 21),
            (1, 30),
            (2, 3),
            (2, 7),
            (2, 8),
            (2, 9),
            (2, 13),
            (2, 27),
            (2, 28),
            (2, 32),
            (3, 7),
            (3, 12),
            (3, 13),
            (4, 6),
            (4, 10),
            (5, 6),
            (5, 10),
            (5, 16),
            (6, 16),
            (8, 30),
            (8, 32),
            (8, 33),
            (9, 33),
            (13, 33),
            (14, 32),
            (14, 33),
            (15, 32),
            (15, 33),
            (18, 32),
            (18, 33),
            (19, 33),
            (20, 32),
            (20, 33),
            (22, 32),
            (22, 33),
            (23, 25),
            (23, 27),
            (23, 29),
            (23, 32),
            (23, 33),
            (24, 25),
            (24, 27),
            (24, 31),
            (25, 31),
            (26, 29),
            (26, 33),
            (27, 33),
            (28, 31),
            (28, 33),
            (29, 32),
            (29, 33),
            (30, 32),
            (30, 33),
            (31, 32),
            (31, 33),
            (32, 33),
        ];
        let g = Graph::from_edges(edges, false, None).unwrap();
        let result = spinglass(&g, None).unwrap();
        assert_eq!(result.membership.len(), 34);
        // Karate club typically yields 3-5 communities
        assert!(
            result.nb_clusters >= 2 && result.nb_clusters <= 10,
            "Unexpected number of clusters: {}",
            result.nb_clusters
        );
        // Modularity should be positive
        assert!(
            result.modularity > 0.2,
            "Modularity too low: {}",
            result.modularity
        );
    }
}
