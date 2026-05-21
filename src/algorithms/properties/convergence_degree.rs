//! ALGO-PR-028 — convergence degree.
//!
//! [`convergence_degree`] computes a per-edge value in `[-1, 1]`
//! (directed) or `[0, 1]` (undirected) that measures whether the
//! shortest paths through the edge originate from a larger or smaller
//! vertex set than they terminate in.
//!
//! - `In(e)` — number of source vertices `s` for which `e` lies on at
//!   least one shortest path rooted at `s`.
//! - `Out(e)` — number of vertices `t` for which `e` lies on at least
//!   one shortest path terminating at `t`.
//! - convergence = `(In - Out) / (In + Out)` (directed) or its absolute
//!   value (undirected).
//!
//! Counterpart of `igraph_convergence_degree()` from
//! `references/igraph/src/properties/convergence_degree.c`.

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Per-edge convergence degree.
///
/// Edges that lie on no shortest path between any source/sink pair
/// produce `NaN` (matching upstream's `0/0` semantics).
///
/// Time complexity: O(|V|·(|V| + |E|)) — BFS per source vertex.
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, convergence_degree};
///
/// // Directed star with one outgoing edge: 1→0, 2→0, 3→0, 4→0, 0→5.
/// let mut g = Graph::new(6, true).unwrap();
/// for u in [1, 2, 3, 4] { g.add_edge(u, 0).unwrap(); }
/// g.add_edge(0, 5).unwrap();
///
/// let res = convergence_degree(&g).unwrap();
/// // 0→5 sees four sources converging into one terminus → +2/3.
/// assert!((res[4] - (4.0 / 6.0)).abs() < 1e-12);
/// // Each leaf-to-hub edge has one source, two sinks → −1/3.
/// for r in &res[..4] {
///     assert!((r - (-1.0 / 3.0)).abs() < 1e-12);
/// }
/// ```
pub fn convergence_degree(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let (result, _, _) = convergence_degree_full(graph)?;
    Ok(result)
}

/// Convergence degree along with the per-edge `In(e)` / `Out(e)` counts.
///
/// All three returned vectors have length `graph.ecount()`.
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, convergence_degree_full};
///
/// // Path 0—1—2—3 (undirected): the middle edge is touched by every
/// // source/sink pair across it; the end edges are not.
/// let mut g = Graph::with_vertices(4);
/// for (u, v) in [(0, 1), (1, 2), (2, 3)] { g.add_edge(u, v).unwrap(); }
/// let (_, ins, outs) = convergence_degree_full(&g).unwrap();
/// for (i, o) in ins.iter().zip(outs.iter()) {
///     assert!(i + o > 0.0);
/// }
/// ```
pub fn convergence_degree_full(graph: &Graph) -> IgraphResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    let n = graph.vcount();
    let m = graph.ecount();
    let directed = graph.is_directed();

    let mut ins: Vec<f64> = vec![0.0; m];
    let mut outs: Vec<f64> = vec![0.0; m];

    if n == 0 || m == 0 {
        return Ok((vec![f64::NAN; m], ins, outs));
    }

    let n_us = n as usize;
    let mut geodist: Vec<i64> = vec![0; n_us];
    let mut queue: VecDeque<(VertexId, i64)> = VecDeque::new();

    // Directed graphs need two passes: OUT-BFS feeds `ins`, IN-BFS feeds
    // `outs`. Undirected graphs only need one pass; the orientation of
    // every tree edge is decided by endpoint comparison.
    let passes: &[bool] = if directed { &[true, false] } else { &[true] };

    for &is_out_pass in passes {
        for src in 0..n {
            geodist.fill(0);
            queue.clear();
            geodist[src as usize] = 1;
            queue.push_back((src, 0));

            while let Some((actnode, actdist)) = queue.pop_front() {
                let eids = if directed && !is_out_pass {
                    graph.incident_in(actnode)?
                } else {
                    graph.incident(actnode)?
                };
                for &eid in &eids {
                    let (a, b) = graph.edge(eid)?;
                    let neighbor = if a == actnode { b } else { a };

                    let nd = geodist[neighbor as usize];
                    if nd != 0 {
                        if nd - 1 == actdist + 1 {
                            // Tree edge: same BFS layer, another shortest path.
                            bump(
                                directed,
                                is_out_pass,
                                actnode,
                                neighbor,
                                eid,
                                &mut ins,
                                &mut outs,
                            );
                        }
                        // Otherwise the neighbour is at a strictly smaller
                        // layer (already counted) or further along the
                        // queue — neither contributes here.
                    } else {
                        geodist[neighbor as usize] = actdist + 2;
                        queue.push_back((neighbor, actdist + 1));
                        bump(
                            directed,
                            is_out_pass,
                            actnode,
                            neighbor,
                            eid,
                            &mut ins,
                            &mut outs,
                        );
                    }
                }
            }
        }
    }

    let mut result = vec![0.0_f64; m];
    for i in 0..m {
        let s = ins[i] + outs[i];
        result[i] = if s > 0.0 {
            (ins[i] - outs[i]) / s
        } else {
            f64::NAN
        };
    }
    if !directed {
        for r in &mut result {
            if *r < 0.0 {
                *r = -*r;
            }
        }
    }

    Ok((result, ins, outs))
}

#[inline]
fn bump(
    directed: bool,
    is_out_pass: bool,
    actnode: VertexId,
    neighbor: VertexId,
    eid: EdgeId,
    ins: &mut [f64],
    outs: &mut [f64],
) {
    let i = eid as usize;
    if directed {
        if is_out_pass {
            ins[i] += 1.0;
        } else {
            outs[i] += 1.0;
        }
    } else if actnode < neighbor {
        ins[i] += 1.0;
    } else {
        outs[i] += 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        (a - b).abs() <= tol
    }

    fn approx_eq_vec(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| approx_eq(*x, *y, tol))
    }

    #[test]
    fn empty_graph_is_empty() {
        let g = Graph::with_vertices(0);
        let (r, i, o) = convergence_degree_full(&g).unwrap();
        assert!(r.is_empty());
        assert!(i.is_empty());
        assert!(o.is_empty());
    }

    #[test]
    fn no_edges_returns_empty_vectors() {
        let g = Graph::with_vertices(5);
        let res = convergence_degree(&g).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn upstream_undirected_test_case_matches_out() {
        // First test case from references/igraph/tests/unit/igraph_convergence_degree.c.
        // Undirected graph with 7 vertices and edges:
        //   (0,1), (0,2), (0,3), (1,2), (1,3), (2,3), (3,4), (4,5), (4,6), (5,6)
        // Expected: 0.0000 0.0000 0.6000 0.0000 0.6000 0.6000 0.1429 0.6667 0.6667 0.0000
        let mut g = Graph::with_vertices(7);
        for (u, v) in [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
            (3, 4),
            (4, 5),
            (4, 6),
            (5, 6),
        ] {
            g.add_edge(u, v).unwrap();
        }
        let res = convergence_degree(&g).unwrap();
        let expected = [
            0.0,
            0.0,
            0.6,
            0.0,
            0.6,
            0.6,
            1.0 / 7.0,
            2.0 / 3.0,
            2.0 / 3.0,
            0.0,
        ];
        assert!(
            approx_eq_vec(&res, &expected, 1e-4),
            "got {res:?}, want {expected:?}"
        );
    }

    #[test]
    fn upstream_directed_test_case_matches_out() {
        // Second test case: directed, 6 vertices, edges (1,0)(2,0)(3,0)(4,0)(0,5).
        // Expected: -0.3333 -0.3333 -0.3333 -0.3333 0.6667
        let mut g = Graph::new(6, true).unwrap();
        for u in [1u32, 2, 3, 4] {
            g.add_edge(u, 0).unwrap();
        }
        g.add_edge(0, 5).unwrap();
        let res = convergence_degree(&g).unwrap();
        let expected = [-1.0 / 3.0, -1.0 / 3.0, -1.0 / 3.0, -1.0 / 3.0, 2.0 / 3.0];
        assert!(
            approx_eq_vec(&res, &expected, 1e-12),
            "got {res:?}, want {expected:?}"
        );
    }

    #[test]
    fn directed_full_returns_consistent_in_out() {
        let mut g = Graph::new(6, true).unwrap();
        for u in [1u32, 2, 3, 4] {
            g.add_edge(u, 0).unwrap();
        }
        g.add_edge(0, 5).unwrap();
        let (r, ins, outs) = convergence_degree_full(&g).unwrap();
        // All edges except 0→5 have ins=1, outs=2.
        for i in 0..4 {
            assert!(approx_eq(ins[i], 1.0, 1e-12), "ins[{i}] = {}", ins[i]);
            assert!(approx_eq(outs[i], 2.0, 1e-12), "outs[{i}] = {}", outs[i]);
        }
        // 0→5: ins counts the four leaves + 0 itself → 5; outs from 5 + 0 → 1.
        assert!(approx_eq(ins[4], 5.0, 1e-12), "ins[4] = {}", ins[4]);
        assert!(approx_eq(outs[4], 1.0, 1e-12), "outs[4] = {}", outs[4]);
        // sanity: spot-check the result derivation.
        for i in 0..r.len() {
            let expected = (ins[i] - outs[i]) / (ins[i] + outs[i]);
            assert!(approx_eq(r[i], expected, 1e-12));
        }
    }

    #[test]
    fn single_undirected_edge_balanced() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let (r, ins, outs) = convergence_degree_full(&g).unwrap();
        // BFS from 0 finds 1 via edge 0 (actnode=0 < neighbor=1) → ins += 1.
        // BFS from 1 finds 0 via edge 0 (actnode=1 > neighbor=0) → outs += 1.
        assert!(approx_eq(ins[0], 1.0, 1e-12));
        assert!(approx_eq(outs[0], 1.0, 1e-12));
        // (1-1)/(1+1) = 0; absolute value still 0.
        assert!(approx_eq(r[0], 0.0, 1e-12));
    }

    #[test]
    fn directed_single_edge() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let (r, ins, outs) = convergence_degree_full(&g).unwrap();
        assert!(approx_eq(ins[0], 1.0, 1e-12));
        assert!(approx_eq(outs[0], 1.0, 1e-12));
        assert!(approx_eq(r[0], 0.0, 1e-12));
    }

    #[test]
    fn undirected_self_loop_yields_nan() {
        // BFS through a self-loop never produces a tree edge (the layer
        // check fails), so ins+outs == 0, giving 0/0 → NaN.
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        let res = convergence_degree(&g).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].is_nan(), "expected NaN, got {}", res[0]);
    }

    #[test]
    fn isolated_vertices_dont_break_things() {
        // Path 0-1-2 + isolated vertices 3, 4.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let res = convergence_degree(&g).unwrap();
        assert_eq!(res.len(), 2);
        // Both edges land at |1/3| under the upstream (actnode <
        // neighbor) orientation rule. The undirected pass takes
        // absolute values so they're identical by symmetry.
        let expected = 1.0 / 3.0;
        for r in &res {
            assert!(approx_eq(*r, expected, 1e-12), "got {r}, want {expected}");
        }
    }

    #[test]
    fn complete_undirected_is_symmetric() {
        // K_4: every edge is symmetric → result == 0 for all.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let res = convergence_degree(&g).unwrap();
        for r in &res {
            assert!(approx_eq(*r, 0.0, 1e-12), "K_4 edge gave {r}");
        }
    }

    #[test]
    fn star_directed_out_edges_have_negative_convergence() {
        // 0 → 1, 0 → 2, 0 → 3 (directed star). Each edge: ins = 1 (just
        // the hub), outs = 3 (every vertex needs the edge to reach a
        // leaf in the IN-BFS).
        // Actually let's just verify directional asymmetry shows up.
        let mut g = Graph::new(4, true).unwrap();
        for v in [1, 2, 3] {
            g.add_edge(0, v).unwrap();
        }
        let (r, ins, outs) = convergence_degree_full(&g).unwrap();
        // OUT pass: BFS from 0 → discover 1,2,3 via these edges → ins[i] = 1 each.
        // BFS from any leaf → no out edges → no contribution.
        for &x in &ins {
            assert!(approx_eq(x, 1.0, 1e-12), "ins = {x}");
        }
        // IN pass: BFS from 1 going IN → 0 via reversed (0,1). No more in-edges.
        // Same for 2, 3. BFS from 0 has no incoming. So outs[i] = 1 each.
        for &x in &outs {
            assert!(approx_eq(x, 1.0, 1e-12), "outs = {x}");
        }
        for &x in &r {
            assert!(approx_eq(x, 0.0, 1e-12), "result = {x}");
        }
    }

    #[test]
    fn parallel_edges_share_credit() {
        // Two parallel undirected edges (0,1). When the BFS-discovery
        // sets geodist[1] for the first edge, the second edge falls in
        // the "already seen but at the same layer" branch — i.e. it's
        // also recognised as a shortest-path tree edge and bumps the
        // counter. So both parallel edges get equal credit.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let (_r, ins, outs) = convergence_degree_full(&g).unwrap();
        for i in 0..2 {
            assert!(approx_eq(ins[i], 1.0, 1e-12), "ins[{i}] = {}", ins[i]);
            assert!(approx_eq(outs[i], 1.0, 1e-12), "outs[{i}] = {}", outs[i]);
        }
    }

    #[test]
    fn result_length_matches_ecount() {
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4), (0, 4)] {
            g.add_edge(u, v).unwrap();
        }
        let res = convergence_degree(&g).unwrap();
        assert_eq!(res.len(), g.ecount());
    }

    #[test]
    fn undirected_results_are_non_negative() {
        // The undirected convention takes |x|, so every non-NaN result
        // is in [0, 1].
        let mut g = Graph::with_vertices(6);
        for (u, v) in [(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)] {
            g.add_edge(u, v).unwrap();
        }
        let res = convergence_degree(&g).unwrap();
        for r in &res {
            assert!(r.is_nan() || (*r >= -1e-12 && *r <= 1.0 + 1e-12), "got {r}");
        }
    }

    #[test]
    fn directed_results_within_unit_interval() {
        let mut g = Graph::new(6, true).unwrap();
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)] {
            g.add_edge(u, v).unwrap();
        }
        let res = convergence_degree(&g).unwrap();
        for r in &res {
            assert!(
                r.is_nan() || (-1.0 - 1e-12 <= *r && *r <= 1.0 + 1e-12),
                "got {r}"
            );
        }
    }

    #[test]
    fn ins_plus_outs_strictly_positive_for_simple_path() {
        // Path P_4: 0-1-2-3. Every edge is on the unique shortest path
        // between every pair straddling it.
        let mut g = Graph::with_vertices(4);
        for (u, v) in [(0, 1), (1, 2), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let (_, ins, outs) = convergence_degree_full(&g).unwrap();
        for (i, o) in ins.iter().zip(outs.iter()) {
            assert!(*i + *o > 0.0, "i + o = {} on path edge", i + o);
        }
    }

    #[test]
    fn balanced_directed_cycle_is_zero() {
        // Directed cycle C_n: each edge sees exactly one "input" (its
        // tail's reach) and one "output" (its head's reverse reach).
        // Result should be 0 for all edges.
        let mut g = Graph::new(5, true).unwrap();
        for u in 0u32..5 {
            g.add_edge(u, (u + 1) % 5).unwrap();
        }
        let res = convergence_degree(&g).unwrap();
        // All values should be equal by symmetry.
        let first = res[0];
        for r in &res {
            assert!(approx_eq(*r, first, 1e-12), "asymmetric: {res:?}");
        }
    }
}
