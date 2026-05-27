//! GraphOpt force-directed layout (ALGO-LO-009).
//!
//! Port of the graphopt layout algorithm by Michael Schmuhl.
//! Uses physical analogies: Coulomb repulsion between all vertex pairs
//! and Hooke spring attraction along edges, iterated until equilibrium.
//!
//! Reference: <https://web.archive.org/web/20220611030748/http://www.schmuhl.org/graphopt/>

use crate::core::{Graph, IgraphError, IgraphResult};

/// Parameters for the GraphOpt layout.
#[derive(Debug, Clone)]
pub struct GraphoptParams {
    /// Number of iterations. Default: 500.
    pub niter: u32,
    /// Charge of each vertex (Coulomb repulsion). Default: 0.001.
    pub node_charge: f64,
    /// Mass of each vertex (inertia). Default: 30.0.
    pub node_mass: f64,
    /// Rest length of springs (edges). Default: 0.0.
    pub spring_length: f64,
    /// Spring constant (Hooke's law). Default: 1.0.
    pub spring_constant: f64,
    /// Maximum displacement per axis per iteration. Default: 5.0.
    pub max_sa_movement: f64,
}

impl Default for GraphoptParams {
    fn default() -> Self {
        Self {
            niter: 500,
            node_charge: 0.001,
            node_mass: 30.0,
            spring_length: 0.0,
            spring_constant: 1.0,
            max_sa_movement: 5.0,
        }
    }
}

/// Compute the GraphOpt force-directed layout.
///
/// Vertices repel each other via Coulomb force and are attracted along
/// edges via Hooke spring force. The system is iterated for `niter`
/// steps; each step accumulates forces, divides by mass, clamps
/// displacement, and moves vertices.
///
/// Edge directions are ignored (treated as undirected).
///
/// # Arguments
///
/// * `graph` — input graph.
/// * `seed` — optional initial positions. If `None`, random positions
///   are generated.
/// * `params` — algorithm parameters.
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if `node_mass` is zero (division by zero)
/// or if `seed` length doesn't match vertex count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_graphopt, GraphoptParams};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
///
/// let params = GraphoptParams::default();
/// let pos = layout_graphopt(&g, None, &params).unwrap();
/// assert_eq!(pos.len(), 5);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_graphopt(
    graph: &Graph,
    seed: Option<&[[f64; 2]]>,
    params: &GraphoptParams,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    if params.node_mass == 0.0 {
        return Err(IgraphError::InvalidArgument(
            "node_mass must be non-zero".into(),
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
    let apply_electric = params.node_charge != 0.0;

    // Coulomb's constant
    const COULOMBS_CONSTANT: f64 = 8_987_500_000.0;

    // Build edge list
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(no_edges);
    for eid in 0..no_edges as u32 {
        if let Ok((src, tgt)) = graph.edge(eid) {
            edges.push((src as usize, tgt as usize));
        }
    }

    // Initialize positions
    let mut pos = if let Some(s) = seed {
        s.to_vec()
    } else {
        let mut rng = SplitMix64::new(42);
        (0..n)
            .map(|_| [rng.next_uniform() - 0.5, rng.next_uniform() - 0.5])
            .collect()
    };

    let mut forces_x = vec![0.0_f64; n];
    let mut forces_y = vec![0.0_f64; n];

    for _iter in 0..params.niter {
        // Clear forces
        for fx in forces_x.iter_mut() {
            *fx = 0.0;
        }
        for fy in forces_y.iter_mut() {
            *fy = 0.0;
        }

        // Electrical repulsion (all pairs)
        if apply_electric {
            for this in 0..n {
                for other in (this + 1)..n {
                    let dx = pos[this][0] - pos[other][0];
                    let dy = pos[this][1] - pos[other][1];
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance == 0.0 || distance >= 500.0 {
                        continue;
                    }

                    let directed_force = COULOMBS_CONSTANT
                        * (params.node_charge * params.node_charge)
                        / (distance * distance);

                    let fx = directed_force * dx.abs() / distance;
                    let fy = directed_force * dy.abs() / distance;

                    // Force on this_node away from other_node
                    let sign_x = if pos[other][0] < pos[this][0] {
                        1.0
                    } else {
                        -1.0
                    };
                    let sign_y = if pos[other][1] < pos[this][1] {
                        1.0
                    } else {
                        -1.0
                    };

                    forces_x[this] -= sign_x * fx;
                    forces_y[this] -= sign_y * fy;
                    forces_x[other] += sign_x * fx;
                    forces_y[other] += sign_y * fy;
                }
            }
        }

        // Spring forces along edges
        for &(src, tgt) in &edges {
            let dx = pos[src][0] - pos[tgt][0];
            let dy = pos[src][1] - pos[tgt][1];
            let distance = (dx * dx + dy * dy).sqrt();

            if distance == 0.0 {
                continue;
            }

            let displacement = (distance - params.spring_length).abs();
            let directed_force = -params.spring_constant * displacement;

            // Determine spring axial forces
            if distance == params.spring_length {
                continue;
            }

            let fx_abs = directed_force.abs() * dx.abs() / distance;
            let fy_abs = directed_force.abs() * dy.abs() / distance;

            // Base direction: force on src away from tgt (like electric)
            let sign_x = if pos[tgt][0] < pos[src][0] { 1.0 } else { -1.0 };
            let sign_y = if pos[tgt][1] < pos[src][1] { 1.0 } else { -1.0 };

            // If spring is too long, pull toward; if too short, push away
            let spring_sign = if distance > params.spring_length {
                1.0
            } else {
                -1.0
            };

            // Half force to each node (spring is shared)
            let hfx = 0.5 * spring_sign * sign_x * fx_abs;
            let hfy = 0.5 * spring_sign * sign_y * fy_abs;

            // src is pulled toward tgt (negative of repulsion direction)
            forces_x[src] += hfx;
            forces_y[src] += hfy;
            forces_x[tgt] -= hfx;
            forces_y[tgt] -= hfy;
        }

        // Move nodes
        for v in 0..n {
            let mut x_move = forces_x[v] / params.node_mass;
            let mut y_move = forces_y[v] / params.node_mass;

            x_move = x_move.clamp(-params.max_sa_movement, params.max_sa_movement);
            y_move = y_move.clamp(-params.max_sa_movement, params.max_sa_movement);

            pos[v][0] += x_move;
            pos[v][1] += y_move;
        }
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
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphopt_empty() {
        let g = Graph::with_vertices(0);
        let params = GraphoptParams::default();
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_graphopt_single() {
        let g = Graph::with_vertices(1);
        let params = GraphoptParams::default();
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].is_finite());
    }

    #[test]
    fn test_graphopt_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let params = GraphoptParams {
            niter: 100,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_graphopt_path() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let params = GraphoptParams {
            niter: 50,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_graphopt_with_seed() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seed = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.866]];
        let params = GraphoptParams {
            niter: 50,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, Some(&seed), &params).unwrap();
        assert_eq!(pos.len(), 3);
    }

    #[test]
    fn test_graphopt_seed_wrong_length() {
        let g = Graph::with_vertices(3);
        let seed = vec![[0.0, 0.0]];
        let params = GraphoptParams::default();
        assert!(layout_graphopt(&g, Some(&seed), &params).is_err());
    }

    #[test]
    fn test_graphopt_zero_mass() {
        let g = Graph::with_vertices(3);
        let params = GraphoptParams {
            node_mass: 0.0,
            ..GraphoptParams::default()
        };
        assert!(layout_graphopt(&g, None, &params).is_err());
    }

    #[test]
    fn test_graphopt_no_charge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = GraphoptParams {
            niter: 50,
            node_charge: 0.0,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_graphopt_deterministic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = GraphoptParams {
            niter: 50,
            ..GraphoptParams::default()
        };
        let pos1 = layout_graphopt(&g, None, &params).unwrap();
        let pos2 = layout_graphopt(&g, None, &params).unwrap();
        for i in 0..4 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-10);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_graphopt_no_edges() {
        let g = Graph::with_vertices(4);
        let params = GraphoptParams {
            niter: 50,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, None, &params).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_graphopt_vertices_spread() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let params = GraphoptParams {
            niter: 100,
            ..GraphoptParams::default()
        };
        let pos = layout_graphopt(&g, None, &params).unwrap();
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
}
