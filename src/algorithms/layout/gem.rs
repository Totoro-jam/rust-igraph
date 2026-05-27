//! GEM force-directed layout (ALGO-LO-007).
//!
//! Graph Embedder algorithm by Arne Frick, Andreas Ludwig, Heiko Mehldau,
//! "A Fast Adaptive Layout Algorithm for Undirected Graphs",
//! Proc. Graph Drawing 1994, LNCS 894, pp. 388-403, 1995.
//!
//! Key features: per-vertex adaptive temperature, oscillation/rotation
//! detection via impulse history, gravitational barycenter pull.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Parameters for the GEM layout algorithm.
#[derive(Debug, Clone)]
pub struct GemParams {
    /// Maximum number of single-vertex iterations.
    /// A reasonable default is `40 * n * n`.
    pub maxiter: u32,
    /// Maximum allowed local temperature. Default: `n` (vertex count).
    pub temp_max: f64,
    /// Global temperature threshold for termination. Default: `0.1`.
    pub temp_min: f64,
    /// Initial local temperature per vertex. Default: `sqrt(n)`.
    pub temp_init: f64,
}

impl GemParams {
    /// Create parameters with defaults scaled to the given vertex count.
    pub fn for_graph(n: u32) -> Self {
        let nf = f64::from(n);
        Self {
            maxiter: 40u32.saturating_mul(n).saturating_mul(n),
            temp_max: nf,
            temp_min: 0.1,
            temp_init: nf.sqrt(),
        }
    }
}

/// Compute the GEM force-directed layout.
///
/// Places vertices using adaptive per-vertex temperatures with
/// gravitational pull toward the barycenter, repulsive forces between
/// all vertex pairs, and attractive spring forces along edges.
///
/// Edge directions are ignored (treated as undirected).
///
/// # Arguments
///
/// * `graph` — input graph.
/// * `seed` — optional initial positions. If `None`, random positions
///   are generated.
/// * `params` — algorithm parameters (use `GemParams::for_graph(n)`
///   for sensible defaults).
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if temperatures violate
/// `temp_min <= temp_init <= temp_max`, any temperature is non-positive,
/// or `seed` length doesn't match vertex count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_gem, GemParams};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 0).unwrap();
///
/// let params = GemParams::for_graph(6);
/// let pos = layout_gem(&g, None, &params).unwrap();
/// assert_eq!(pos.len(), 6);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_gem(
    graph: &Graph,
    seed: Option<&[[f64; 2]]>,
    params: &GemParams,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    if params.temp_max <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "temp_max must be positive".into(),
        ));
    }
    if params.temp_min <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "temp_min must be positive".into(),
        ));
    }
    if params.temp_init <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "temp_init must be positive".into(),
        ));
    }
    if params.temp_max < params.temp_init || params.temp_init < params.temp_min {
        return Err(IgraphError::InvalidArgument(
            "requires temp_min <= temp_init <= temp_max".into(),
        ));
    }

    if let Some(s) = seed {
        if s.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "seed length ({}) must equal vertex count ({})",
                s.len(),
                n
            )));
        }
    }

    // Constants from the paper
    let elen_des2: f64 = 128.0 * 128.0;
    let gamma: f64 = 1.0 / 16.0;
    let alpha_o: f64 = std::f64::consts::PI;
    let alpha_r: f64 = std::f64::consts::PI / 3.0;
    let sigma_o: f64 = 1.0 / 3.0;
    let sigma_r: f64 = 1.0 / (2.0 * n as f64);

    // Compute vertex "mass" (degree * (degree/2 + 1))
    let mut phi = vec![0.0_f64; n];
    for v in 0..n {
        let deg = graph.degree(v as VertexId).unwrap_or(0) as f64;
        phi[v] = deg * (deg / 2.0 + 1.0);
    }

    // Initialize positions
    let mut pos = if let Some(s) = seed {
        s.to_vec()
    } else {
        let width_half = n as f64 * 100.0;
        let mut rng = SplitMix64::new(42);
        (0..n)
            .map(|_| {
                [
                    rng.next_uniform() * 2.0 * width_half - width_half,
                    rng.next_uniform() * 2.0 * width_half - width_half,
                ]
            })
            .collect()
    };

    // Barycenter
    let mut barycenter_x: f64 = pos.iter().map(|p| p[0]).sum();
    let mut barycenter_y: f64 = pos.iter().map(|p| p[1]).sum();

    // Per-vertex state
    let mut impulse_x = vec![0.0_f64; n];
    let mut impulse_y = vec![0.0_f64; n];
    let mut temp = vec![params.temp_init; n];
    let mut skew_gauge = vec![0.0_f64; n];

    // Permutation for random vertex selection
    let mut perm: Vec<usize> = (0..n).collect();
    let mut perm_pointer: usize = 0;
    let mut rng = SplitMix64::new(123);

    // Build adjacency list (undirected)
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let ecount = graph.ecount();
    for eid in 0..ecount as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            if src != tgt {
                adj[src as usize].push(tgt);
                adj[tgt as usize].push(src);
            }
        }
    }

    let mut temp_global = params.temp_init * n as f64;
    let mut maxiter = params.maxiter;

    while temp_global > params.temp_min * n as f64 && maxiter > 0 {
        // Choose vertex v
        if perm_pointer == 0 {
            fisher_yates_shuffle(&mut perm, &mut rng);
            perm_pointer = n;
        }
        perm_pointer -= 1;
        let v = perm[perm_pointer];

        // Gravitational pull toward barycenter
        let nf = n as f64;
        let mut px = (barycenter_x / nf - pos[v][0]) * gamma * phi[v];
        let mut py = (barycenter_y / nf - pos[v][1]) * gamma * phi[v];

        // Random jitter
        px += rng.next_uniform() * 64.0 - 32.0;
        py += rng.next_uniform() * 64.0 - 32.0;

        // Repulsive forces from all other vertices
        for u in 0..n {
            if u == v {
                continue;
            }
            let dx = pos[v][0] - pos[u][0];
            let dy = pos[v][1] - pos[u][1];
            let dist2 = dx * dx + dy * dy;
            if dist2 != 0.0 {
                px += dx * elen_des2 / dist2;
                py += dy * elen_des2 / dist2;
            }
        }

        // Attractive forces from neighbors
        for &u in &adj[v] {
            let ui = u as usize;
            let dx = pos[v][0] - pos[ui][0];
            let dy = pos[v][1] - pos[ui][1];
            let dist2 = dx * dx + dy * dy;
            if phi[v] != 0.0 {
                px -= dx * dist2 / (elen_des2 * phi[v]);
                py -= dy * dist2 / (elen_des2 * phi[v]);
            }
        }

        // Update position
        if px != 0.0 || py != 0.0 {
            let plen = (px * px + py * py).sqrt();
            px *= temp[v] / plen;
            py *= temp[v] / plen;
            pos[v][0] += px;
            pos[v][1] += py;
            barycenter_x += px;
            barycenter_y += py;
        }

        // Temperature adjustment via oscillation/rotation detection
        let pvx = impulse_x[v];
        let pvy = impulse_y[v];
        if pvx != 0.0 || pvy != 0.0 {
            let beta = (pvy - py).atan2(pvx - px);
            let sin_beta = beta.sin();
            let sign_sin_beta = if sin_beta > 0.0 {
                1.0
            } else if sin_beta < 0.0 {
                -1.0
            } else {
                0.0
            };
            let cos_beta = beta.cos();
            let abs_cos_beta = cos_beta.abs();
            let old_temp = temp[v];

            if sin_beta >= (std::f64::consts::FRAC_PI_2 + alpha_r / 2.0).sin() {
                skew_gauge[v] += sigma_r * sign_sin_beta;
            }
            if abs_cos_beta >= (alpha_o / 2.0).cos() {
                temp[v] *= sigma_o * cos_beta;
            }
            temp[v] *= 1.0 - skew_gauge[v].abs();
            if temp[v] > params.temp_max {
                temp[v] = params.temp_max;
            }
            if temp[v] < 0.0 {
                temp[v] = 0.0;
            }
            impulse_x[v] = px;
            impulse_y[v] = py;
            temp_global += temp[v] - old_temp;
        }

        maxiter -= 1;
    }

    Ok(pos)
}

// ═══════════════════════════════════════════════════════════════════
// Internal RNG (deterministic, no external deps)
// ═══════════════════════════════════════════════════════════════════

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

fn fisher_yates_shuffle(perm: &mut [usize], rng: &mut SplitMix64) {
    let n = perm.len();
    for i in (1..n).rev() {
        let j = (rng.next_u64() as usize) % (i + 1);
        perm.swap(i, j);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gem_empty() {
        let g = Graph::with_vertices(0);
        let params = GemParams::for_graph(0);
        let pos = layout_gem(&g, None, &params).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_gem_single_vertex() {
        let g = Graph::with_vertices(1);
        let params = GemParams::for_graph(1);
        let pos = layout_gem(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].is_finite());
        assert!(pos[0][1].is_finite());
    }

    #[test]
    fn test_gem_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let params = GemParams::for_graph(3);
        let pos = layout_gem(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_gem_path() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = GemParams::for_graph(5);
        let pos = layout_gem(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_gem_no_overlap() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = GemParams::for_graph(4);
        let pos = layout_gem(&g, None, &params).unwrap();
        // Vertices shouldn't all collapse to the same point
        let mut all_same = true;
        for i in 1..4 {
            if (pos[i][0] - pos[0][0]).abs() > 1e-6 || (pos[i][1] - pos[0][1]).abs() > 1e-6 {
                all_same = false;
                break;
            }
        }
        assert!(!all_same, "all vertices collapsed to the same point");
    }

    #[test]
    fn test_gem_with_seed() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seed = vec![[0.0, 0.0], [100.0, 0.0], [50.0, 86.6]];
        let params = GemParams::for_graph(3);
        let pos = layout_gem(&g, Some(&seed), &params).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_gem_seed_wrong_length() {
        let g = Graph::with_vertices(3);
        let seed = vec![[0.0, 0.0], [1.0, 0.0]];
        let params = GemParams::for_graph(3);
        let result = layout_gem(&g, Some(&seed), &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_gem_invalid_temp() {
        let g = Graph::with_vertices(3);
        let params = GemParams {
            maxiter: 100,
            temp_max: -1.0,
            temp_min: 0.1,
            temp_init: 1.0,
        };
        assert!(layout_gem(&g, None, &params).is_err());

        let params2 = GemParams {
            maxiter: 100,
            temp_max: 10.0,
            temp_min: 5.0,
            temp_init: 2.0,
        };
        assert!(layout_gem(&g, None, &params2).is_err());
    }

    #[test]
    fn test_gem_deterministic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let params = GemParams::for_graph(4);
        let pos1 = layout_gem(&g, None, &params).unwrap();
        let pos2 = layout_gem(&g, None, &params).unwrap();
        for i in 0..4 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-10);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_gem_disconnected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let params = GemParams::for_graph(4);
        let pos = layout_gem(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_gem_star() {
        let mut g = Graph::with_vertices(6);
        for i in 1..6 {
            g.add_edge(0, i).unwrap();
        }
        let params = GemParams {
            maxiter: 1000,
            temp_max: 6.0,
            temp_min: 0.1,
            temp_init: 2.4,
        };
        let pos = layout_gem(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 6);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }
}
