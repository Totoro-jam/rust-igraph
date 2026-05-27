//! UMAP (Uniform Manifold Approximation and Projection) layout (ALGO-LO-013).
//!
//! Stochastic gradient descent layout that minimizes cross-entropy between
//! high-dimensional and low-dimensional probability distributions. Supports
//! distance-to-weight conversion, negative sampling, and both 2D and 3D.
//!
//! Reference: McInnes, Healy, Melville — UMAP: Uniform Manifold Approximation
//! and Projection for Dimension Reduction (2020). arXiv:1802.03426.
//! igraph C: `src/layout/umap.c`.

use crate::core::{Graph, IgraphError, IgraphResult};

const FORCE_LIMIT: f64 = 4.0;
const MIN_DISTANCE_ATTRACTION: f64 = 0.0001;
const CORRECT_DISTANCE_REPULSION: f64 = 0.01;

/// Parameters for the UMAP layout algorithm.
#[derive(Debug, Clone)]
pub struct UmapParams {
    /// Minimum distance parameter controlling embedding tightness.
    /// Typical values: 0.0 to 1.0. Default: 0.01.
    pub min_dist: f64,
    /// Number of SGD epochs. Typical values: 30 to 500. Default: 500.
    pub epochs: u32,
    /// Whether `distances` are pre-computed weights (skip weight computation).
    /// Default: false.
    pub distances_are_weights: bool,
}

impl Default for UmapParams {
    fn default() -> Self {
        Self {
            min_dist: 0.01,
            epochs: 500,
            distances_are_weights: false,
        }
    }
}

/// Compute UMAP weights from edge distances.
///
/// For each vertex, finds rho (minimum distance) and sigma (scale factor),
/// then converts distances to exponentially decaying weights. Symmetrizes
/// via fuzzy union: W = W1 + W2 - W1 * W2.
///
/// # Arguments
///
/// * `graph` — input graph (may be directed for kNN graphs).
/// * `distances` — per-edge distances. If `None`, all edges get weight 1.
///
/// Returns a weight vector of length `ecount`.
pub fn umap_compute_weights(graph: &Graph, distances: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    if let Some(d) = distances {
        if d.len() != m {
            return Err(IgraphError::InvalidArgument(
                "distances length must equal edge count".into(),
            ));
        }
        if m > 0 {
            let min_d = d.iter().copied().fold(f64::INFINITY, f64::min);
            if min_d < 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "distances must not be negative".into(),
                ));
            }
            if min_d.is_nan() {
                return Err(IgraphError::InvalidArgument(
                    "distances must not be NaN".into(),
                ));
            }
        }
    }

    // Build adjacency: for each vertex, list of (neighbor, edge_id)
    let mut out_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for eid in 0..m {
        let (src, tgt) = graph.edge(eid as u32)?;
        out_edges[src as usize].push((tgt as usize, eid));
    }

    // Per-vertex: compute weights and store in neighbors_seen / weights_seen
    let mut neighbors_seen: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut weights_seen: Vec<Vec<f64>> = vec![Vec::new(); n];

    for i in 0..n {
        let neis = &out_edges[i];
        if neis.is_empty() {
            continue;
        }

        // Find rho and dist_max
        let (rho, dist_max) = if let Some(dists) = distances {
            let mut rho = f64::INFINITY;
            let mut dmax = f64::NEG_INFINITY;
            for &(_nb, eid) in neis {
                let d = dists[eid];
                if d < rho {
                    rho = d;
                }
                if d > dmax {
                    dmax = d;
                }
            }
            (rho, dmax)
        } else {
            (0.0, 0.0)
        };

        // Find sigma
        let sigma = if dist_max == rho {
            -1.0 // flag: all neighbors equidistant
        } else {
            let target = (neis.len() as f64).log2();
            find_sigma(distances.unwrap(), neis, rho, target)
        };

        // Convert to weights
        for &(nb, eid) in neis {
            let weight = if sigma < 0.0 {
                1.0
            } else {
                let d = distances.unwrap()[eid];
                (-(d - rho) / sigma).exp()
            };

            // Check for self-loops
            if nb == i {
                return Err(IgraphError::InvalidArgument(
                    "input graph must contain no self-loops".into(),
                ));
            }

            neighbors_seen[i].push(nb);
            weights_seen[i].push(weight);
        }
    }

    // Symmetrize via fuzzy union
    let mut result = vec![0.0_f64; m];

    for eid in 0..m {
        let (src, tgt) = graph.edge(eid as u32)?;
        let i = src as usize;
        let k = tgt as usize;

        // Direct weight from i→k
        let mut weight = 0.0_f64;
        let mut found_idx = None;
        for (l, &nb) in neighbors_seen[i].iter().enumerate() {
            if nb == k {
                weight = weights_seen[i][l];
                found_idx = Some(l);
                break;
            }
        }

        // If tagged (-1), this edge was already unioned from the other direction
        if weight < 0.0 {
            result[eid] = 0.0;
            continue;
        }

        // Tag so opposite won't double-count
        if let Some(idx) = found_idx {
            weights_seen[i][idx] = -1.0;
        }

        // Opposite weight from k→i
        let mut weight_inv = 0.0_f64;
        let mut found_inv_idx = None;
        for (l, &nb) in neighbors_seen[k].iter().enumerate() {
            if nb == i {
                weight_inv = weights_seen[k][l];
                found_inv_idx = Some(l);
                break;
            }
        }

        if weight_inv < 0.0 {
            result[eid] = 0.0;
            continue;
        }

        if let Some(idx) = found_inv_idx {
            weights_seen[k][idx] = -1.0;
        }

        // Fuzzy union
        result[eid] = weight + weight_inv - weight * weight_inv;
    }

    Ok(result)
}

/// Find sigma for a vertex by binary search.
fn find_sigma(distances: &[f64], neis: &[(usize, usize)], rho: f64, target: f64) -> f64 {
    let mut sigma = 1.0_f64;
    let tol = 0.01;
    let maxiter = 100;
    let mut step = sigma;
    let mut seen_max = false;

    for iter in 0..maxiter {
        let sum: f64 = neis
            .iter()
            .map(|&(_nb, eid)| (-(distances[eid] - rho) / sigma).exp())
            .sum();

        if sum < target {
            if seen_max {
                step /= 2.0;
            } else if iter > 0 {
                step *= 2.0;
            }
            sigma += step;
        } else {
            seen_max = true;
            step /= 2.0;
            sigma -= step;
        }

        if (sum - target).abs() < tol {
            break;
        }
    }

    sigma
}

/// Fit the a and b parameters using Gauss-Newton with line search.
///
/// These control the smooth probability function Q(d) = (1 + a*d^(2b))^-1
/// in the embedding space.
fn fit_ab(min_dist: f64) -> (f64, f64) {
    let nr_points = 300;
    let end_point = 3.0_f64;
    let mut a = 1.8_f64;
    let mut b = 0.8_f64;
    let tol = 0.001;
    let maxiter = 100;

    // Distance lattice
    let x: Vec<f64> = (0..nr_points)
        .map(|i| (end_point / nr_points as f64) * i as f64 + 0.001)
        .collect();

    let mut residuals = vec![0.0_f64; nr_points];
    let mut powb = vec![0.0_f64; nr_points];
    let mut squared_sum_res;
    let mut squared_sum_res_old = f64::INFINITY;

    for iter in 0..maxiter {
        // Compute residuals
        squared_sum_res = 0.0;
        for i in 0..nr_points {
            powb[i] = x[i].powf(2.0 * b);
            let q = 1.0 / (1.0 + a * powb[i]);
            let p = if x[i] <= min_dist {
                1.0
            } else {
                (-(x[i] - min_dist)).exp()
            };
            residuals[i] = q - p;
            squared_sum_res += residuals[i] * residuals[i];
        }

        if squared_sum_res < tol * tol {
            break;
        }
        if iter > 0 && (squared_sum_res_old.sqrt() - squared_sum_res.sqrt()).abs() < tol {
            break;
        }

        // Jacobian
        let mut j_a = vec![0.0_f64; nr_points];
        let mut j_b = vec![0.0_f64; nr_points];
        for i in 0..nr_points {
            let tmp = 1.0 + a * powb[i];
            j_a[i] = -2.0 * powb[i] / (tmp * tmp);
            j_b[i] = j_a[i] * a * x[i].ln() * 2.0;
        }

        // J^T @ J (2x2) and J^T @ r (2x1)
        let mut jtj = [[0.0_f64; 2]; 2];
        let mut jtr = [0.0_f64; 2];
        for i in 0..nr_points {
            let jrow = [j_a[i], j_b[i]];
            for j1 in 0..2 {
                for j2 in 0..2 {
                    jtj[j1][j2] += jrow[j1] * jrow[j2];
                }
                jtr[j1] += jrow[j1] * residuals[i];
            }
        }

        // Solve 2x2 system via Cramer's rule
        let det = jtj[0][0] * jtj[1][1] - jtj[0][1] * jtj[1][0];
        if det.abs() < 1e-30 {
            break;
        }
        let da = -(jtj[1][1] * jtr[0] - jtj[0][1] * jtr[1]) / det;
        let db = -(jtj[0][0] * jtr[1] - jtj[1][0] * jtr[0]) / det;

        // Line search
        let mut da_step = da;
        let mut db_step = db;
        squared_sum_res_old = squared_sum_res;

        let mut best_ssr = compute_ssr(&x, a + da_step, b + db_step, min_dist);

        for _k in 0..30 {
            da_step /= 2.0;
            db_step /= 2.0;
            let new_ssr = compute_ssr(&x, a + da_step, b + db_step, min_dist);
            if new_ssr > best_ssr - tol {
                da_step *= 2.0;
                db_step *= 2.0;
                break;
            }
            best_ssr = new_ssr;
        }

        a += da_step;
        b += db_step;
    }

    (a, b)
}

fn compute_ssr(x: &[f64], a: f64, b: f64, min_dist: f64) -> f64 {
    let mut ssr = 0.0;
    for &xi in x {
        let q = 1.0 / (1.0 + a * xi.powf(2.0 * b));
        let p = if xi <= min_dist {
            1.0
        } else {
            (-(xi - min_dist)).exp()
        };
        let r = q - p;
        ssr += r * r;
    }
    ssr
}

fn clip_force(force: f64) -> f64 {
    force.clamp(-FORCE_LIMIT, FORCE_LIMIT)
}

fn attract(dsq: f64, a: f64, b: f64) -> f64 {
    -(2.0 * a * b * dsq.powf(b - 1.0)) / (1.0 + a * dsq.powf(b))
}

fn repel(dsq: f64, a: f64, b: f64) -> f64 {
    let dsq_min = CORRECT_DISTANCE_REPULSION * CORRECT_DISTANCE_REPULSION;
    (2.0 * b) / (dsq_min + dsq) / (1.0 + a * dsq.powf(b))
}

/// Compute the 2D UMAP layout.
///
/// Places vertices using stochastic gradient descent that minimizes
/// cross-entropy between high-dimensional edge probabilities and
/// low-dimensional distances.
///
/// # Arguments
///
/// * `graph` — input graph (treated as directed for kNN, undirected for general).
/// * `seed` — optional initial positions `[x, y]` per vertex.
/// * `distances` — optional per-edge distances. If `None`, all edges have weight 1.
/// * `params` — algorithm parameters.
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns error if distances length doesn't match edge count, distances are
/// negative/NaN, or seed dimensions are wrong.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_umap, UmapParams};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
///
/// let params = UmapParams { epochs: 50, ..UmapParams::default() };
/// let pos = layout_umap(&g, None, None, &params).unwrap();
/// assert_eq!(pos.len(), 6);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_umap(
    graph: &Graph,
    seed: Option<&[[f64; 2]]>,
    distances: Option<&[f64]>,
    params: &UmapParams,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;

    if params.min_dist < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "min_dist must not be negative".into(),
        ));
    }

    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        if let Some(s) = seed {
            if s.len() != 1 {
                return Err(IgraphError::InvalidArgument(
                    "seed must have exactly vcount positions".into(),
                ));
            }
            return Ok(s.to_vec());
        }
        return Ok(vec![[0.0, 0.0]]);
    }

    // Compute or use weights
    let weights = if params.distances_are_weights {
        distances
            .ok_or_else(|| {
                IgraphError::InvalidArgument(
                    "distances_are_weights=true but no distances provided".into(),
                )
            })?
            .to_vec()
    } else {
        umap_compute_weights(graph, distances)?
    };

    // Initial positions
    let mut pos: Vec<[f64; 2]> = if let Some(s) = seed {
        if s.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "seed length {} != vcount {}",
                s.len(),
                n
            )));
        }
        s.to_vec()
    } else {
        let mut rng = SplitMix64::new(42);
        (0..n)
            .map(|_| [rng.next_uniform(), rng.next_uniform()])
            .collect()
    };

    // Fit a, b
    let (a, b) = fit_ab(params.min_dist);

    // Build edge list
    let m = graph.ecount();
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(m);
    for eid in 0..m as u32 {
        let (s, t) = graph.edge(eid)?;
        edges.push((s as usize, t as usize));
    }

    // Build adjacency for neighbor avoidance (small graphs)
    let avoid_neighbor_repulsion = n < 100;
    let adj: Vec<Vec<usize>> = if avoid_neighbor_repulsion {
        let mut a = vec![Vec::new(); n];
        for &(s, t) in &edges {
            a[s].push(t);
            if s != t {
                a[t].push(s);
            }
        }
        a
    } else {
        Vec::new()
    };

    let negative_sampling_rate = 5usize.min(n - 1);

    // Next epoch sample tracking
    let mut next_epoch: Vec<f64> = vec![0.0; m];

    let mut rng = SplitMix64::new(123);

    // SGD epochs
    for epoch in 0..params.epochs {
        let learning_rate = 1.0 - (epoch as f64 + 1.0) / params.epochs as f64;

        for eid in 0..m {
            if weights[eid] <= 0.0 {
                continue;
            }
            if next_epoch[eid] - epoch as f64 >= 1.0 {
                continue;
            }

            next_epoch[eid] += 1.0 / weights[eid];

            let (from_v, to_v) = edges[eid];

            // Process both directions (swap from/to)
            for swap in 0..2u8 {
                let (from, to) = if swap == 0 {
                    (from_v, to_v)
                } else {
                    (to_v, from_v)
                };

                // Attraction
                let dx = pos[from][0] - pos[to][0];
                let dy = pos[from][1] - pos[to][1];
                let dsq = dx * dx + dy * dy;

                if dsq >= MIN_DISTANCE_ATTRACTION * MIN_DISTANCE_ATTRACTION {
                    let force = attract(dsq, a, b);
                    let fx = clip_force(force * dx);
                    let fy = clip_force(force * dy);
                    pos[from][0] += learning_rate * fx;
                    pos[from][1] += learning_rate * fy;
                }

                // Negative sampling (repulsion)
                for _j in 0..negative_sampling_rate {
                    let mut neg = rng.next_usize(n - 1);
                    if neg >= from {
                        neg += 1;
                    }

                    // Skip actual neighbors for small graphs
                    if avoid_neighbor_repulsion && adj[from].contains(&neg) {
                        continue;
                    }

                    let dx = pos[from][0] - pos[neg][0];
                    let dy = pos[from][1] - pos[neg][1];
                    let dsq = dx * dx + dy * dy;

                    let force = repel(dsq, a, b);
                    let fx = clip_force(force * dx);
                    let fy = clip_force(force * dy);
                    pos[from][0] += learning_rate * fx;
                    pos[from][1] += learning_rate * fy;
                }
            }
        }
    }

    // Center layout
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    for p in &pos {
        cx += p[0];
        cy += p[1];
    }
    cx /= n as f64;
    cy /= n as f64;
    for p in &mut pos {
        p[0] -= cx;
        p[1] -= cy;
    }

    Ok(pos)
}

/// Compute the 3D UMAP layout.
///
/// Same as [`layout_umap`] but produces 3D positions.
///
/// # Arguments
///
/// * `graph` — input graph.
/// * `seed` — optional initial `[x, y, z]` positions.
/// * `distances` — optional per-edge distances.
/// * `params` — algorithm parameters.
///
/// Returns `[x, y, z]` positions for each vertex.
pub fn layout_umap_3d(
    graph: &Graph,
    seed: Option<&[[f64; 3]]>,
    distances: Option<&[f64]>,
    params: &UmapParams,
) -> IgraphResult<Vec<[f64; 3]>> {
    let n = graph.vcount() as usize;

    if params.min_dist < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "min_dist must not be negative".into(),
        ));
    }

    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        if let Some(s) = seed {
            if s.len() != 1 {
                return Err(IgraphError::InvalidArgument(
                    "seed must have exactly vcount positions".into(),
                ));
            }
            return Ok(s.to_vec());
        }
        return Ok(vec![[0.0, 0.0, 0.0]]);
    }

    let weights = if params.distances_are_weights {
        distances
            .ok_or_else(|| {
                IgraphError::InvalidArgument(
                    "distances_are_weights=true but no distances provided".into(),
                )
            })?
            .to_vec()
    } else {
        umap_compute_weights(graph, distances)?
    };

    let mut pos: Vec<[f64; 3]> = if let Some(s) = seed {
        if s.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "seed length {} != vcount {}",
                s.len(),
                n
            )));
        }
        s.to_vec()
    } else {
        let mut rng = SplitMix64::new(42);
        (0..n)
            .map(|_| [rng.next_uniform(), rng.next_uniform(), rng.next_uniform()])
            .collect()
    };

    let (a, b) = fit_ab(params.min_dist);

    let m = graph.ecount();
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(m);
    for eid in 0..m as u32 {
        let (s, t) = graph.edge(eid)?;
        edges.push((s as usize, t as usize));
    }

    let avoid_neighbor_repulsion = n < 100;
    let adj: Vec<Vec<usize>> = if avoid_neighbor_repulsion {
        let mut a = vec![Vec::new(); n];
        for &(s, t) in &edges {
            a[s].push(t);
            if s != t {
                a[t].push(s);
            }
        }
        a
    } else {
        Vec::new()
    };

    let negative_sampling_rate = 5usize.min(n - 1);
    let mut next_epoch: Vec<f64> = vec![0.0; m];
    let mut rng = SplitMix64::new(123);

    for epoch in 0..params.epochs {
        let learning_rate = 1.0 - (epoch as f64 + 1.0) / params.epochs as f64;

        for eid in 0..m {
            if weights[eid] <= 0.0 {
                continue;
            }
            if next_epoch[eid] - epoch as f64 >= 1.0 {
                continue;
            }
            next_epoch[eid] += 1.0 / weights[eid];

            let (from_v, to_v) = edges[eid];

            for swap in 0..2u8 {
                let (from, to) = if swap == 0 {
                    (from_v, to_v)
                } else {
                    (to_v, from_v)
                };

                let dx = pos[from][0] - pos[to][0];
                let dy = pos[from][1] - pos[to][1];
                let dz = pos[from][2] - pos[to][2];
                let dsq = dx * dx + dy * dy + dz * dz;

                if dsq >= MIN_DISTANCE_ATTRACTION * MIN_DISTANCE_ATTRACTION {
                    let force = attract(dsq, a, b);
                    let fx = clip_force(force * dx);
                    let fy = clip_force(force * dy);
                    let fz = clip_force(force * dz);
                    pos[from][0] += learning_rate * fx;
                    pos[from][1] += learning_rate * fy;
                    pos[from][2] += learning_rate * fz;
                }

                for _j in 0..negative_sampling_rate {
                    let mut neg = rng.next_usize(n - 1);
                    if neg >= from {
                        neg += 1;
                    }

                    if avoid_neighbor_repulsion && adj[from].contains(&neg) {
                        continue;
                    }

                    let dx = pos[from][0] - pos[neg][0];
                    let dy = pos[from][1] - pos[neg][1];
                    let dz = pos[from][2] - pos[neg][2];
                    let dsq = dx * dx + dy * dy + dz * dz;

                    let force = repel(dsq, a, b);
                    let fx = clip_force(force * dx);
                    let fy = clip_force(force * dy);
                    let fz = clip_force(force * dz);
                    pos[from][0] += learning_rate * fx;
                    pos[from][1] += learning_rate * fy;
                    pos[from][2] += learning_rate * fz;
                }
            }
        }
    }

    // Center
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    let mut cz = 0.0_f64;
    for p in &pos {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    for p in &mut pos {
        p[0] -= cx;
        p[1] -= cy;
        p[2] -= cz;
    }

    Ok(pos)
}

// ═══════════════════════════════════════════════════════════════════
// Internal RNG
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

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u64() as usize) % max
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_umap_empty() {
        let g = Graph::with_vertices(0);
        let params = UmapParams::default();
        let pos = layout_umap(&g, None, None, &params).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_umap_single() {
        let g = Graph::with_vertices(1);
        let params = UmapParams::default();
        let pos = layout_umap(&g, None, None, &params).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].abs() < 1e-10 && pos[0][1].abs() < 1e-10);
    }

    #[test]
    fn test_umap_path() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, None, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_cycle() {
        let mut g = Graph::with_vertices(6);
        for i in 0..6 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, None, None, &params).unwrap();
        assert_eq!(pos.len(), 6);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_with_distances() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let distances = vec![1.0, 2.0, 0.5];
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, None, Some(&distances), &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_with_seed() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seed = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, Some(&seed), None, &params).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_distances_are_weights() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = vec![0.9, 0.5, 0.8];
        let params = UmapParams {
            epochs: 30,
            distances_are_weights: true,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, None, Some(&weights), &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_3d() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap_3d(&g, None, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn test_umap_negative_distances_error() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let distances = vec![1.0, -0.5];
        let params = UmapParams::default();
        assert!(layout_umap(&g, None, Some(&distances), &params).is_err());
    }

    #[test]
    fn test_umap_wrong_distances_length() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let distances = vec![1.0]; // should be 2
        let params = UmapParams::default();
        assert!(layout_umap(&g, None, Some(&distances), &params).is_err());
    }

    #[test]
    fn test_umap_wrong_seed_length() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seed = vec![[0.0, 0.0], [1.0, 0.0]]; // should be 3
        let params = UmapParams {
            epochs: 10,
            ..UmapParams::default()
        };
        assert!(layout_umap(&g, Some(&seed), None, &params).is_err());
    }

    #[test]
    fn test_umap_complete() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos = layout_umap(&g, None, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_umap_deterministic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = UmapParams {
            epochs: 30,
            ..UmapParams::default()
        };
        let pos1 = layout_umap(&g, None, None, &params).unwrap();
        let pos2 = layout_umap(&g, None, None, &params).unwrap();
        for i in 0..4 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-10);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_compute_weights_basic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let distances = vec![1.0, 2.0, 0.5];
        let w = umap_compute_weights(&g, Some(&distances)).unwrap();
        assert_eq!(w.len(), 3);
        for &wi in &w {
            assert!((0.0..=1.0).contains(&wi));
        }
    }

    #[test]
    fn test_compute_weights_no_distances() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = umap_compute_weights(&g, None).unwrap();
        assert_eq!(w.len(), 2);
        // Without distances, all weights should be 1.0
        for &wi in &w {
            assert!((wi - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_fit_ab() {
        let (a, b) = fit_ab(0.1);
        assert!(a > 0.0 && a < 100.0);
        assert!(b > 0.0 && b < 5.0);
    }
}
