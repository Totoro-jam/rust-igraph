//! Burt's constraint scores (ALGO-PR-032).
//!
//! Counterpart of `igraph_constraint` from
//! `references/igraph/src/properties/constraint.c` (288 lines).
//!
//! Burt's constraint measures how much a vertex's connections are
//! redundant — higher constraint means fewer structural holes and
//! less brokerage opportunity.

use crate::core::{Graph, IgraphResult};

/// Compute Burt's constraint scores for all vertices.
///
/// The constraint of vertex `i` is defined as:
///
/// ```text
/// C[i] = Σ_j (p[i,j] + Σ_{q ≠ i,j} p[i,q] · p[q,j])²
/// ```
///
/// where the proportional tie strength is:
///
/// ```text
/// p[i,j] = (a[i,j] + a[j,i]) / Σ_{k ≠ i} (a[i,k] + a[k,i])
/// ```
///
/// For isolated vertices (no incident edges excluding self-loops),
/// the constraint is `NaN`.
///
/// Optionally accepts edge weights. If `weights` is `None`, all edges
/// are treated as having unit weight.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, constraint};
///
/// // Star graph: center has constraint from 4 leaves
/// let mut g = Graph::with_vertices(5);
/// for i in 1..5 {
///     g.add_edge(0, i).unwrap();
/// }
/// let c = constraint(&g, None).unwrap();
/// // Center vertex: each leaf contributes (1/4)² = 1/16, total = 4/16 = 0.25
/// assert!((c[0] - 0.25).abs() < 1e-9);
/// // Each leaf: only neighbor is center, constraint = 1.0
/// assert!((c[1] - 1.0).abs() < 1e-9);
/// ```
pub fn constraint(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(crate::core::IgraphError::InvalidArgument(
                "weight vector length must equal edge count".to_string(),
            ));
        }
    }

    let strength = compute_strength(graph, weights)?;
    let mut result = vec![0.0_f64; n_us];
    let mut contrib = vec![0.0_f64; n_us];

    for i in 0..n {
        let i_us = i as usize;
        let in_edges = graph.incident_in(i)?;
        let out_edges = graph.incident(i)?;

        if in_edges.is_empty() && out_edges.is_empty() {
            result[i_us] = f64::NAN;
            continue;
        }

        clear_contrib(graph, i, &in_edges, &mut contrib)?;
        if graph.is_directed() {
            clear_contrib(graph, i, &out_edges, &mut contrib)?;
        }

        let deg_i = strength[i_us];

        add_direct(graph, i, &in_edges, weights, deg_i, &mut contrib)?;
        if graph.is_directed() {
            add_direct(graph, i, &out_edges, weights, deg_i, &mut contrib)?;
        }

        add_indirect(graph, i, &in_edges, weights, &strength, deg_i, &mut contrib)?;
        if graph.is_directed() {
            add_indirect(
                graph,
                i,
                &out_edges,
                weights,
                &strength,
                deg_i,
                &mut contrib,
            )?;
        }

        result[i_us] += square_and_clear(graph, i, &in_edges, &mut contrib)?;
        if graph.is_directed() {
            result[i_us] += square_and_clear(graph, i, &out_edges, &mut contrib)?;
        }
    }

    Ok(result)
}

fn clear_contrib(graph: &Graph, i: u32, edges: &[u32], contrib: &mut [f64]) -> IgraphResult<()> {
    for &eid in edges {
        let j = graph.edge_other(eid, i)?;
        contrib[j as usize] = 0.0;
    }
    Ok(())
}

fn add_direct(
    graph: &Graph,
    i: u32,
    edges: &[u32],
    weights: Option<&[f64]>,
    deg_i: f64,
    contrib: &mut [f64],
) -> IgraphResult<()> {
    for &eid in edges {
        let j = graph.edge_other(eid, i)?;
        if i != j {
            contrib[j as usize] += edge_weight(weights, eid) / deg_i;
        }
    }
    Ok(())
}

fn add_indirect(
    graph: &Graph,
    i: u32,
    edges: &[u32],
    weights: Option<&[f64]>,
    strength: &[f64],
    deg_i: f64,
    contrib: &mut [f64],
) -> IgraphResult<()> {
    for &eid in edges {
        let j = graph.edge_other(eid, i)?;
        if i == j {
            continue;
        }
        let w_ij = edge_weight(weights, eid);
        let deg_j = strength[j as usize];
        let factor = w_ij / (deg_i * deg_j);
        let j_in = graph.incident_in(j)?;
        for &eid2 in &j_in {
            let q = graph.edge_other(eid2, j)?;
            if j != q {
                contrib[q as usize] += factor * edge_weight(weights, eid2);
            }
        }
        if graph.is_directed() {
            let j_out = graph.incident(j)?;
            for &eid2 in &j_out {
                let q = graph.edge_other(eid2, j)?;
                if j != q {
                    contrib[q as usize] += factor * edge_weight(weights, eid2);
                }
            }
        }
    }
    Ok(())
}

fn square_and_clear(
    graph: &Graph,
    i: u32,
    edges: &[u32],
    contrib: &mut [f64],
) -> IgraphResult<f64> {
    let mut sum = 0.0;
    for &eid in edges {
        let j = graph.edge_other(eid, i)?;
        if i != j {
            sum += contrib[j as usize] * contrib[j as usize];
            contrib[j as usize] = 0.0;
        }
    }
    Ok(sum)
}

fn edge_weight(weights: Option<&[f64]>, eid: u32) -> f64 {
    weights.map_or(1.0, |w| w[eid as usize])
}

/// Compute weighted strength (sum of edge weights) for all vertices,
/// excluding self-loops. Uses `IGRAPH_ALL` mode (in + out for directed).
fn compute_strength(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut strength = vec![0.0_f64; n_us];

    for v in 0..n {
        let v_us = v as usize;
        let in_edges = graph.incident_in(v)?;
        for &eid in &in_edges {
            let other = graph.edge_other(eid, v)?;
            if other != v {
                strength[v_us] += edge_weight(weights, eid);
            }
        }
        if graph.is_directed() {
            let out_edges = graph.incident(v)?;
            for &eid in &out_edges {
                let other = graph.edge_other(eid, v)?;
                if other != v {
                    strength[v_us] += edge_weight(weights, eid);
                }
            }
        }
    }

    Ok(strength)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let c = constraint(&g, None).expect("ok");
        assert!(c.is_empty());
    }

    #[test]
    fn isolated_vertex_is_nan() {
        let g = Graph::with_vertices(3);
        let c = constraint(&g, None).expect("ok");
        assert!(c[0].is_nan());
        assert!(c[1].is_nan());
        assert!(c[2].is_nan());
    }

    #[test]
    fn single_edge() {
        // 0-1: each vertex has one neighbor, constraint = 1.0
        let g = create(&[(0, 1)], 2, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 1.0));
        assert!(approx_eq(c[1], 1.0));
    }

    #[test]
    fn triangle() {
        // 0-1, 1-2, 0-2: each has 2 neighbors connected to each other
        // p[i,j] = 1/2 for each neighbor
        // indirect: p[i,q]*p[q,j] = (1/2)*(1/2) = 1/4 for the single
        // intermediary
        // total for neighbor j: p[i,j] + sum_q p[i,q]*p[q,j] = 1/2 + 1/4 = 3/4
        // constraint = 2 * (3/4)^2 = 2 * 9/16 = 9/8 = 1.125
        let g = create(&[(0, 1), (1, 2), (0, 2)], 3, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 1.125), "got {}", c[0]);
        assert!(approx_eq(c[1], 1.125), "got {}", c[1]);
        assert!(approx_eq(c[2], 1.125), "got {}", c[2]);
    }

    #[test]
    fn star_4_leaves() {
        // Center 0 connected to 1,2,3,4. Leaves not connected to each other.
        // Center: 4 neighbors, p[0,j] = 1/4 each, no indirect paths
        //   (leaves not connected), constraint = 4*(1/4)^2 = 4/16 = 0.25
        // Leaf k: 1 neighbor (center), p[k,0] = 1, constraint = 1^2 = 1.0
        let g = create(&[(0, 1), (0, 2), (0, 3), (0, 4)], 5, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 0.25), "center: {}", c[0]);
        for (i, &val) in c.iter().enumerate().skip(1) {
            assert!(approx_eq(val, 1.0), "leaf {i}: {val}");
        }
    }

    #[test]
    fn path_3_vertices() {
        // 0-1-2: vertex 1 bridges 0 and 2
        // Vertex 0: p[0,1] = 1.0, no indirect path to 1, constraint = 1.0
        // Vertex 2: p[2,1] = 1.0, constraint = 1.0
        // Vertex 1: p[1,0] = 1/2, p[1,2] = 1/2, no indirect path between
        //   0 and 2, constraint = (1/2)^2 + (1/2)^2 = 1/2 = 0.5
        let g = create(&[(0, 1), (1, 2)], 3, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 1.0), "v0: {}", c[0]);
        assert!(approx_eq(c[1], 0.5), "v1: {}", c[1]);
        assert!(approx_eq(c[2], 1.0), "v2: {}", c[2]);
    }

    #[test]
    fn self_loop_excluded() {
        // 0-0 (self-loop), 0-1
        let g = create(&[(0, 0), (0, 1)], 2, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        // Self-loop doesn't count toward strength or constraint
        assert!(approx_eq(c[0], 1.0), "v0: {}", c[0]);
        assert!(approx_eq(c[1], 1.0), "v1: {}", c[1]);
    }

    #[test]
    fn weighted_single_edge() {
        let g = create(&[(0, 1)], 2, false).expect("ok");
        let c = constraint(&g, Some(&[3.0])).expect("ok");
        assert!(approx_eq(c[0], 1.0));
        assert!(approx_eq(c[1], 1.0));
    }

    #[test]
    fn weighted_path_3() {
        // 0-1 (w=1), 1-2 (w=3)
        // strength: v0=1, v1=1+3=4, v2=3
        // v0: p[0,1] = 1/1 = 1.0 → constraint = 1.0
        // v2: p[2,1] = 3/3 = 1.0 → constraint = 1.0
        // v1: p[1,0] = 1/4, p[1,2] = 3/4
        //   no indirect paths → constraint = (1/4)^2 + (3/4)^2 = 1/16 + 9/16 = 10/16 = 0.625
        let g = create(&[(0, 1), (1, 2)], 3, false).expect("ok");
        let c = constraint(&g, Some(&[1.0, 3.0])).expect("ok");
        assert!(approx_eq(c[0], 1.0), "v0: {}", c[0]);
        assert!(approx_eq(c[1], 0.625), "v1: {}", c[1]);
        assert!(approx_eq(c[2], 1.0), "v2: {}", c[2]);
    }

    #[test]
    fn invalid_weight_length() {
        let g = create(&[(0, 1)], 2, false).expect("ok");
        let r = constraint(&g, Some(&[1.0, 2.0]));
        assert!(r.is_err());
    }

    #[test]
    fn mixed_connected_isolated() {
        // 0-1, 2 isolated
        let g = create(&[(0, 1)], 3, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 1.0));
        assert!(approx_eq(c[1], 1.0));
        assert!(c[2].is_nan());
    }

    #[test]
    fn k4_complete() {
        // K4: every vertex has 3 neighbors, all connected
        // p[i,j] = 1/3 for each neighbor
        // indirect through q: p[i,q]*p[q,j] = (1/3)*(1/3) = 1/9
        // two intermediaries: 2 * 1/9 = 2/9
        // total per neighbor: 1/3 + 2/9 = 3/9 + 2/9 = 5/9
        // constraint = 3 * (5/9)^2 = 3 * 25/81 = 75/81 = 25/27
        let g = create(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4, false).expect("ok");
        let c = constraint(&g, None).expect("ok");
        let expected = 25.0 / 27.0;
        for (i, &val) in c.iter().enumerate() {
            assert!(approx_eq(val, expected), "v{i}: {val} expected {expected}");
        }
    }

    #[test]
    fn directed_path() {
        // 0->1->2 directed
        // strength (ALL, no loops):
        //   v0: out-degree to 1 = 1, in-degree = 0, strength = 1
        //   v1: out to 2 = 1, in from 0 = 1, strength = 2
        //   v2: in from 1 = 1, out = 0, strength = 1
        // v0: neighbor set = {1} (out-edge 0->1)
        //   p[0,1] = 1/1 = 1.0 → constraint = 1.0
        // v2: neighbor set = {1} (in-edge 1->2)
        //   p[2,1] = 1/1 = 1.0 → constraint = 1.0
        // v1: neighbors = {0, 2} (in-edge from 0, out-edge to 2)
        //   p[1,0] = 1/2, p[1,2] = 1/2
        //   no indirect paths → constraint = (1/2)^2 + (1/2)^2 = 0.5
        let g = create(&[(0, 1), (1, 2)], 3, true).expect("ok");
        let c = constraint(&g, None).expect("ok");
        assert!(approx_eq(c[0], 1.0), "v0: {}", c[0]);
        assert!(approx_eq(c[1], 0.5), "v1: {}", c[1]);
        assert!(approx_eq(c[2], 1.0), "v2: {}", c[2]);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::create;
    use proptest::prelude::*;

    fn arb_graph(max_v: u32) -> impl Strategy<Value = Graph> {
        (2..=max_v).prop_flat_map(|n| {
            let max_e = (n as usize)
                .saturating_mul(n.saturating_sub(1) as usize)
                .min(20);
            proptest::collection::vec((0..n, 0..n), 0..=max_e).prop_map(move |edges| {
                let edge_tuples: Vec<(u32, u32)> = edges.into_iter().collect();
                create(&edge_tuples, n, false).expect("arb graph")
            })
        })
    }

    proptest! {
        #[test]
        fn constraint_non_negative_or_nan(g in arb_graph(8)) {
            let c = constraint(&g, None).expect("ok");
            for (i, &val) in c.iter().enumerate() {
                prop_assert!(
                    val.is_nan() || val >= 0.0,
                    "negative constraint {val} at vertex {i}"
                );
            }
        }

        #[test]
        fn isolated_vertex_is_nan_prop(n in 1u32..10) {
            let g = Graph::with_vertices(n);
            let c = constraint(&g, None).expect("ok");
            for (i, &val) in c.iter().enumerate() {
                prop_assert!(val.is_nan(), "expected NaN for isolated vertex {i}, got {val}");
            }
        }

        #[test]
        fn unit_weights_match_unweighted(g in arb_graph(8)) {
            let ne = g.ecount();
            let unw = constraint(&g, None).expect("ok");
            let w = constraint(&g, Some(&vec![1.0; ne])).expect("ok");
            for (i, (&a, &b)) in unw.iter().zip(w.iter()).enumerate() {
                if a.is_nan() {
                    prop_assert!(b.is_nan(), "v{i}: unweighted NaN but weighted {b}");
                } else {
                    prop_assert!(
                        (a - b).abs() < 1e-9,
                        "v{i}: unweighted {a} != weighted {b}"
                    );
                }
            }
        }
    }
}
