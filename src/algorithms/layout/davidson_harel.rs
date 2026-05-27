//! Davidson-Harel simulated annealing layout (ALGO-LO-008).
//!
//! Reference: Ron Davidson, David Harel, "Drawing Graphs Nicely Using
//! Simulated Annealing", ACM Transactions on Graphics 15(4), 1996.
//!
//! Energy function components: node-node distances, border proximity,
//! edge lengths, edge crossings, node-edge distances.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Parameters for the Davidson-Harel layout.
#[derive(Debug, Clone)]
pub struct DhParams {
    /// Maximum annealing iterations. Default: 10.
    pub maxiter: u32,
    /// Fine-tuning iterations. Default: max(10, log2(n)).
    pub fineiter: u32,
    /// Cooling factor in (0, 1). Default: 0.75.
    pub cool_fact: f64,
    /// Weight for node-node distance repulsion. Default: 1.0.
    pub weight_node_dist: f64,
    /// Weight for border proximity. Default: 0.0.
    pub weight_border: f64,
    /// Weight for edge length minimization. Default: density/10.
    pub weight_edge_lengths: f64,
    /// Weight for edge crossing minimization. Default: 1 - sqrt(density).
    pub weight_edge_crossings: f64,
    /// Weight for node-edge distance (fine-tuning only). Default: (1-density)/5.
    pub weight_node_edge_dist: f64,
}

impl DhParams {
    /// Create parameters with defaults scaled to the given graph.
    pub fn for_graph(graph: &Graph) -> Self {
        let n = graph.vcount() as usize;
        let e = graph.ecount();
        let max_edges = if n > 1 { n * (n - 1) / 2 } else { 1 };
        let density = e as f64 / max_edges as f64;
        let fineiter = if n > 1 {
            (n as f64).log2().ceil().max(10.0) as u32
        } else {
            10
        };
        Self {
            maxiter: 10,
            fineiter,
            cool_fact: 0.75,
            weight_node_dist: 1.0,
            weight_border: 0.0,
            weight_edge_lengths: density / 10.0,
            weight_edge_crossings: (1.0 - density.sqrt()).max(0.0),
            weight_node_edge_dist: (1.0 - density) / 5.0,
        }
    }
}

/// Compute the Davidson-Harel simulated annealing layout.
///
/// Uses a multi-component energy function minimized via simulated
/// annealing followed by a fine-tuning phase without stochastic
/// acceptance.
///
/// # Arguments
///
/// * `graph` — input graph (edge directions ignored).
/// * `seed` — optional initial positions. If `None`, random positions
///   are generated.
/// * `params` — algorithm parameters.
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if `cool_fact` is not in (0, 1) or
/// `seed` length doesn't match vertex count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_davidson_harel, DhParams};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
///
/// let params = DhParams::for_graph(&g);
/// let pos = layout_davidson_harel(&g, None, &params).unwrap();
/// assert_eq!(pos.len(), 5);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_davidson_harel(
    graph: &Graph,
    seed: Option<&[[f64; 2]]>,
    params: &DhParams,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    if params.cool_fact <= 0.0 || params.cool_fact >= 1.0 {
        return Err(IgraphError::InvalidArgument(
            "cool_fact must be in (0, 1)".into(),
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

    let no_edges = graph.ecount();
    let width = (n as f64).sqrt() * 10.0;
    let height = width;
    let no_tries: usize = 30;
    let fine_tuning_factor = 0.01;

    // Build edge list
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(no_edges);
    for eid in 0..no_edges as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            edges.push((src as usize, tgt as usize));
        }
    }

    // Build adjacency (undirected, no self-loops)
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(src, tgt) in &edges {
        if src != tgt {
            adj[src].push(tgt);
            adj[tgt].push(src);
        }
    }

    // Initialize positions
    let mut rng = SplitMix64::new(7);
    let mut pos = if let Some(s) = seed {
        s.to_vec()
    } else {
        (0..n)
            .map(|_| {
                [
                    rng.next_uniform() * width - width / 2.0,
                    rng.next_uniform() * height - height / 2.0,
                ]
            })
            .collect()
    };

    // Compute bounding box
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in &pos {
        if p[0] < min_x {
            min_x = p[0];
        }
        if p[0] > max_x {
            max_x = p[0];
        }
        if p[1] < min_y {
            min_y = p[1];
        }
        if p[1] > max_y {
            max_y = p[1];
        }
    }

    // Pre-compute try directions (evenly spaced angles)
    let try_dirs: Vec<[f64; 2]> = (0..no_tries)
        .map(|i| {
            let phi = 2.0 * std::f64::consts::PI / no_tries as f64 * i as f64;
            [phi.cos(), phi.sin()]
        })
        .collect();

    let mut perm: Vec<usize> = (0..n).collect();
    let mut try_idx: Vec<usize> = (0..no_tries).collect();
    let mut move_radius = width / 2.0;

    let total_rounds = params.maxiter + params.fineiter;

    for round in 0..total_rounds {
        let fine_tuning = round >= params.maxiter;

        if fine_tuning && round == params.maxiter {
            let fx = fine_tuning_factor * (max_x - min_x);
            let fy = fine_tuning_factor * (max_y - min_y);
            move_radius = fx.min(fy);
        }

        fisher_yates_shuffle(&mut perm, &mut rng);

        for &v in &perm {
            fisher_yates_shuffle(&mut try_idx, &mut rng);

            for &ti in &try_idx {
                let old_x = pos[v][0];
                let old_y = pos[v][1];
                let mut new_x = old_x + move_radius * try_dirs[ti][0];
                let mut new_y = old_y + move_radius * try_dirs[ti][1];

                // Clamp to bounds
                new_x = new_x.clamp(-width / 2.0 + 1e-6, width / 2.0 - 1e-6);
                new_y = new_y.clamp(-height / 2.0 + 1e-6, height / 2.0 - 1e-6);

                let mut diff_energy = 0.0_f64;

                // Node-node distance repulsion
                if params.weight_node_dist != 0.0 {
                    for u in 0..n {
                        if u == v {
                            continue;
                        }
                        let odx = old_x - pos[u][0];
                        let ody = old_y - pos[u][1];
                        let dx = new_x - pos[u][0];
                        let dy = new_y - pos[u][1];
                        let odist2 = odx * odx + ody * ody;
                        let dist2 = dx * dx + dy * dy;
                        if dist2 > 0.0 && odist2 > 0.0 {
                            diff_energy +=
                                params.weight_node_dist / dist2 - params.weight_node_dist / odist2;
                        }
                    }
                }

                // Border proximity
                if params.weight_border != 0.0 {
                    let hw = width / 2.0;
                    let hh = height / 2.0;
                    let border_energy = |x: f64, y: f64| -> f64 {
                        let dx1 = (hw - x).max(2.0);
                        let dx2 = (x + hw).max(2.0);
                        let dy1 = (hh - y).max(2.0);
                        let dy2 = (y + hh).max(2.0);
                        1.0 / (dx1 * dx1)
                            + 1.0 / (dx2 * dx2)
                            + 1.0 / (dy1 * dy1)
                            + 1.0 / (dy2 * dy2)
                    };
                    diff_energy += params.weight_border
                        * (border_energy(new_x, new_y) - border_energy(old_x, old_y));
                }

                // Edge lengths
                if params.weight_edge_lengths != 0.0 {
                    for &u in &adj[v] {
                        let odx = old_x - pos[u][0];
                        let ody = old_y - pos[u][1];
                        let dx = new_x - pos[u][0];
                        let dy = new_y - pos[u][1];
                        let odist2 = odx * odx + ody * ody;
                        let dist2 = dx * dx + dy * dy;
                        diff_energy += params.weight_edge_lengths * (dist2 - odist2);
                    }
                }

                // Edge crossings
                if params.weight_edge_crossings != 0.0 {
                    let mut crossing_diff: i64 = 0;
                    for &u in &adj[v] {
                        let ux = pos[u][0];
                        let uy = pos[u][1];
                        for &(u1, u2) in &edges {
                            if u1 == v || u2 == v || u1 == u || u2 == u {
                                continue;
                            }
                            let u1x = pos[u1][0];
                            let u1y = pos[u1][1];
                            let u2x = pos[u2][0];
                            let u2y = pos[u2][1];
                            if segments_intersect(old_x, old_y, ux, uy, u1x, u1y, u2x, u2y) {
                                crossing_diff -= 1;
                            }
                            if segments_intersect(new_x, new_y, ux, uy, u1x, u1y, u2x, u2y) {
                                crossing_diff += 1;
                            }
                        }
                    }
                    diff_energy += params.weight_edge_crossings * crossing_diff as f64;
                }

                // Node-edge distance (fine-tuning only)
                if params.weight_node_edge_dist != 0.0 && fine_tuning {
                    // Non-incident edges from moved vertex
                    for &(u1, u2) in &edges {
                        if u1 == v || u2 == v {
                            continue;
                        }
                        let u1x = pos[u1][0];
                        let u1y = pos[u1][1];
                        let u2x = pos[u2][0];
                        let u2y = pos[u2][1];
                        let d_old = point_segment_dist2(old_x, old_y, u1x, u1y, u2x, u2y);
                        let d_new = point_segment_dist2(new_x, new_y, u1x, u1y, u2x, u2y);
                        if d_old > 0.0 {
                            diff_energy -= params.weight_node_edge_dist / d_old;
                        }
                        if d_new > 0.0 {
                            diff_energy += params.weight_node_edge_dist / d_new;
                        }
                    }

                    // All other nodes from v's incident edges
                    for &u in &adj[v] {
                        let ux = pos[u][0];
                        let uy = pos[u][1];
                        for w in 0..n {
                            if w == v || w == u {
                                continue;
                            }
                            let wx = pos[w][0];
                            let wy = pos[w][1];
                            let d_old = point_segment_dist2(wx, wy, old_x, old_y, ux, uy);
                            let d_new = point_segment_dist2(wx, wy, new_x, new_y, ux, uy);
                            if d_old > 0.0 {
                                diff_energy -= params.weight_node_edge_dist / d_old;
                            }
                            if d_new > 0.0 {
                                diff_energy += params.weight_node_edge_dist / d_new;
                            }
                        }
                    }
                }

                // Accept or reject
                let accept = if diff_energy < 0.0 {
                    true
                } else if !fine_tuning && move_radius > 0.0 {
                    rng.next_uniform() < (-diff_energy / move_radius).exp()
                } else {
                    false
                };

                if accept {
                    pos[v][0] = new_x;
                    pos[v][1] = new_y;
                    if new_x < min_x {
                        min_x = new_x;
                    }
                    if new_x > max_x {
                        max_x = new_x;
                    }
                    if new_y < min_y {
                        min_y = new_y;
                    }
                    if new_y > max_y {
                        max_y = new_y;
                    }
                    break;
                }
            }
        }

        move_radius *= params.cool_fact;
    }

    Ok(pos)
}

// ═══════════════════════════════════════════════════════════════════
// Geometry helpers
// ═══════════════════════════════════════════════════════════════════

fn segments_intersect(
    p0x: f64,
    p0y: f64,
    p1x: f64,
    p1y: f64,
    p2x: f64,
    p2y: f64,
    p3x: f64,
    p3y: f64,
) -> bool {
    let s1x = p1x - p0x;
    let s1y = p1y - p0y;
    let s2x = p3x - p2x;
    let s2y = p3y - p2y;
    let denom = -s2x * s1y + s1x * s2y;
    if denom == 0.0 {
        return false;
    }
    let s = (-s1y * (p0x - p2x) + s1x * (p0y - p2y)) / denom;
    let t = (s2x * (p0y - p2y) - s2y * (p0x - p2x)) / denom;
    s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0
}

fn point_segment_dist2(vx: f64, vy: f64, u1x: f64, u1y: f64, u2x: f64, u2y: f64) -> f64 {
    let dx = u2x - u1x;
    let dy = u2y - u1y;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return (vx - u1x) * (vx - u1x) + (vy - u1y) * (vy - u1y);
    }
    let t = ((vx - u1x) * dx + (vy - u1y) * dy) / l2;
    if t < 0.0 {
        (vx - u1x) * (vx - u1x) + (vy - u1y) * (vy - u1y)
    } else if t > 1.0 {
        (vx - u2x) * (vx - u2x) + (vy - u2y) * (vy - u2y)
    } else {
        let px = u1x + t * dx;
        let py = u1y + t * dy;
        (vx - px) * (vx - px) + (vy - py) * (vy - py)
    }
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
    fn test_dh_empty() {
        let g = Graph::with_vertices(0);
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, None, &params).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_dh_single() {
        let g = Graph::with_vertices(1);
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].is_finite());
    }

    #[test]
    fn test_dh_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_dh_path() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
    }

    #[test]
    fn test_dh_with_seed() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seed = vec![[0.0, 0.0], [5.0, 0.0], [2.5, 4.0]];
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, Some(&seed), &params).unwrap();
        assert_eq!(pos.len(), 3);
    }

    #[test]
    fn test_dh_seed_wrong_length() {
        let g = Graph::with_vertices(3);
        let seed = vec![[0.0, 0.0]];
        let params = DhParams::for_graph(&g);
        assert!(layout_davidson_harel(&g, Some(&seed), &params).is_err());
    }

    #[test]
    fn test_dh_invalid_cool_fact() {
        let g = Graph::with_vertices(3);
        let mut params = DhParams::for_graph(&g);
        params.cool_fact = 1.5;
        assert!(layout_davidson_harel(&g, None, &params).is_err());
        params.cool_fact = 0.0;
        assert!(layout_davidson_harel(&g, None, &params).is_err());
    }

    #[test]
    fn test_dh_deterministic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = DhParams::for_graph(&g);
        let pos1 = layout_davidson_harel(&g, None, &params).unwrap();
        let pos2 = layout_davidson_harel(&g, None, &params).unwrap();
        for i in 0..4 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-10);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_dh_no_edges() {
        let g = Graph::with_vertices(4);
        let params = DhParams::for_graph(&g);
        let pos = layout_davidson_harel(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_segments_intersect() {
        assert!(segments_intersect(0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0));
        assert!(!segments_intersect(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_point_segment_dist2_basic() {
        let d = point_segment_dist2(0.0, 1.0, 0.0, 0.0, 1.0, 0.0);
        assert!((d - 1.0).abs() < 1e-10);
        let d2 = point_segment_dist2(2.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        assert!((d2 - 1.0).abs() < 1e-10);
    }
}
