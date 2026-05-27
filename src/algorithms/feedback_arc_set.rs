//! Feedback arc set (ALGO-CY-002).
//!
//! A feedback arc set (FAS) is a set of edges whose removal makes the
//! graph acyclic. For undirected graphs the minimum FAS equals the set
//! of edges not in any spanning tree. For directed graphs this is
//! NP-hard; we implement the Eades-Lin-Smyth (1993) heuristic which
//! is guaranteed to find a FAS smaller than |E|/2 - |V|/6 in O(|E|).
//!
//! Counterpart of `igraph_feedback_arc_set`.

use std::collections::VecDeque;

use crate::algorithms::spanning::mst::{MstAlgorithm, minimum_spanning_tree};
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Algorithm choice for computing the feedback arc set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FasAlgorithm {
    /// For undirected graphs: non-tree edges of a maximum-weight
    /// spanning tree. For directed graphs: Eades-Lin-Smyth heuristic.
    EadesLinSmyth,
}

/// Compute a feedback arc set — a set of edge IDs whose removal
/// makes the graph acyclic.
///
/// For undirected graphs, the result is a minimum FAS (non-tree edges
/// of a maximum-weight spanning forest). For directed graphs, the
/// Eades-Lin-Smyth heuristic provides a good approximation.
///
/// # Errors
///
/// - Returns an error if `weights` length does not match `ecount`.
/// - Returns an error if weights contain non-finite values.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, feedback_arc_set, FasAlgorithm};
///
/// // Directed cycle 0 → 1 → 2 → 0: removing one edge breaks it.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
/// assert!(!fas.is_empty());
/// assert!(fas.len() <= 2);
///
/// // DAG: no feedback edges needed.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
/// assert!(fas.is_empty());
/// ```
pub fn feedback_arc_set(
    graph: &Graph,
    weights: Option<&[f64]>,
    _algo: FasAlgorithm,
) -> IgraphResult<Vec<EdgeId>> {
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {}",
                w.len(),
                graph.ecount()
            )));
        }
        if !w.iter().all(|x| x.is_finite()) {
            return Err(IgraphError::InvalidArgument(
                "weights must be finite".into(),
            ));
        }
    }

    if graph.ecount() == 0 {
        return Ok(Vec::new());
    }

    if graph.is_directed() {
        fas_eades(graph, weights)
    } else {
        fas_undirected(graph, weights)
    }
}

fn fas_undirected(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<EdgeId>> {
    let negated: Vec<f64>;
    let mst_weights = match weights {
        Some(w) => {
            negated = w.iter().map(|x| -x).collect();
            Some(negated.as_slice())
        }
        None => None,
    };

    let tree_edges = minimum_spanning_tree(graph, mst_weights, MstAlgorithm::Automatic)?;

    let mut in_tree = vec![false; graph.ecount()];
    for &eid in &tree_edges {
        in_tree[eid as usize] = true;
    }

    let mut result = Vec::new();
    for (eid, &is_tree) in in_tree.iter().enumerate() {
        if !is_tree {
            #[allow(clippy::cast_possible_truncation)]
            result.push(eid as EdgeId);
        }
    }
    Ok(result)
}

#[allow(clippy::too_many_lines)]
fn fas_eades(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount() as usize;
    let edge_count = graph.ecount();

    let mut indegrees: Vec<i32> = vec![0; n];
    let mut outdegrees: Vec<i32> = vec![0; n];
    let mut instrengths: Vec<f64> = vec![0.0; n];
    let mut outstrengths: Vec<f64> = vec![0.0; n];

    for eid in 0..edge_count {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as EdgeId)?;
        if from == to {
            continue;
        }
        let w = weights.map_or(1.0, |ws| ws[eid]);
        outdegrees[from as usize] += 1;
        outstrengths[from as usize] += w;
        indegrees[to as usize] += 1;
        instrengths[to as usize] += w;
    }

    let mut ordering: Vec<i32> = vec![0; n];
    let mut order_next_pos: i32 = 0;
    let mut order_next_neg: i32 = -1;

    let mut sources: VecDeque<VertexId> = VecDeque::new();
    let mut sinks: VecDeque<VertexId> = VecDeque::new();

    let mut nodes_left: usize = n;

    for u in 0..n {
        if indegrees[u] == 0 {
            if outdegrees[u] == 0 {
                #[allow(clippy::cast_possible_truncation)]
                {
                    ordering[u] = order_next_pos;
                    order_next_pos += 1;
                    indegrees[u] = -1;
                    outdegrees[u] = -1;
                    nodes_left -= 1;
                }
            } else {
                #[allow(clippy::cast_possible_truncation)]
                sources.push_back(u as VertexId);
            }
        } else if outdegrees[u] == 0 {
            #[allow(clippy::cast_possible_truncation)]
            sinks.push_back(u as VertexId);
        }
    }

    while nodes_left > 0 {
        while let Some(u) = sources.pop_front() {
            let ui = u as usize;
            ordering[ui] = order_next_pos;
            order_next_pos += 1;
            indegrees[ui] = -1;
            outdegrees[ui] = -1;
            nodes_left -= 1;

            let out_edges = graph.incident(u)?;
            for &eid in &out_edges {
                let to = graph.edge_target(eid)?;
                if to == u {
                    continue;
                }
                let wi = to as usize;
                if indegrees[wi] <= 0 {
                    continue;
                }
                indegrees[wi] -= 1;
                instrengths[wi] -= weights.map_or(1.0, |ws| ws[eid as usize]);
                if indegrees[wi] == 0 {
                    sources.push_back(to);
                }
            }
        }

        while let Some(u) = sinks.pop_front() {
            let ui = u as usize;
            if indegrees[ui] < 0 {
                continue;
            }
            ordering[ui] = order_next_neg;
            order_next_neg -= 1;
            indegrees[ui] = -1;
            outdegrees[ui] = -1;
            nodes_left -= 1;

            let in_edges = graph.incident_in(u)?;
            for &eid in &in_edges {
                let from = graph.edge_source(eid)?;
                if from == u {
                    continue;
                }
                let wi = from as usize;
                if outdegrees[wi] <= 0 {
                    continue;
                }
                outdegrees[wi] -= 1;
                outstrengths[wi] -= weights.map_or(1.0, |ws| ws[eid as usize]);
                if outdegrees[wi] == 0 {
                    sinks.push_back(from);
                }
            }
        }

        let mut best_v: Option<usize> = None;
        let mut max_diff = f64::NEG_INFINITY;
        for u in 0..n {
            if outdegrees[u] < 0 {
                continue;
            }
            let diff = outstrengths[u] - instrengths[u];
            if diff > max_diff {
                max_diff = diff;
                best_v = Some(u);
            }
        }

        if let Some(v) = best_v {
            ordering[v] = order_next_pos;
            order_next_pos += 1;

            #[allow(clippy::cast_possible_truncation)]
            let vv = v as VertexId;

            let out_edges = graph.incident(vv)?;
            for &eid in &out_edges {
                let to = graph.edge_target(eid)?;
                if to == vv {
                    continue;
                }
                let wi = to as usize;
                if indegrees[wi] <= 0 {
                    continue;
                }
                indegrees[wi] -= 1;
                instrengths[wi] -= weights.map_or(1.0, |ws| ws[eid as usize]);
                if indegrees[wi] == 0 {
                    sources.push_back(to);
                }
            }

            let in_edges = graph.incident_in(vv)?;
            for &eid in &in_edges {
                let from = graph.edge_source(eid)?;
                if from == vv {
                    continue;
                }
                let wi = from as usize;
                if outdegrees[wi] <= 0 {
                    continue;
                }
                outdegrees[wi] -= 1;
                outstrengths[wi] -= weights.map_or(1.0, |ws| ws[eid as usize]);
                if outdegrees[wi] == 0 && indegrees[wi] > 0 {
                    sinks.push_back(from);
                }
            }

            indegrees[v] = -1;
            outdegrees[v] = -1;
            nodes_left -= 1;
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let n_i32 = n as i32;
    for pos in &mut ordering {
        if *pos < 0 {
            *pos += n_i32;
        }
    }

    let mut result = Vec::new();
    for eid in 0..edge_count {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as EdgeId)?;
        if from == to || ordering[from as usize] > ordering[to as usize] {
            #[allow(clippy::cast_possible_truncation)]
            result.push(eid as EdgeId);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::cycles::{CycleMode, find_cycle};

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.is_empty());
    }

    #[test]
    fn dag_no_feedback() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 3).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.is_empty());
    }

    #[test]
    fn simple_directed_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(!fas.is_empty());
        assert!(fas.len() <= 2);
    }

    #[test]
    fn directed_cycle_4() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(!fas.is_empty());
        assert!(fas.len() <= 2);
    }

    #[test]
    fn undirected_tree_no_feedback() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.is_empty());
    }

    #[test]
    fn undirected_single_cycle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert_eq!(fas.len(), 1);
    }

    #[test]
    fn undirected_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        // K4 has 6 edges, spanning tree has 3 edges → FAS = 3
        assert_eq!(fas.len(), 3);
    }

    #[test]
    fn self_loops_in_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.contains(&0)); // self-loop is always a feedback edge
    }

    #[test]
    fn weighted_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // eid 0, weight 1.0
        g.add_edge(1, 2).unwrap(); // eid 1, weight 10.0
        g.add_edge(2, 0).unwrap(); // eid 2, weight 10.0
        let weights = vec![1.0, 10.0, 10.0];
        let fas = feedback_arc_set(&g, Some(&weights), FasAlgorithm::EadesLinSmyth).unwrap();
        // MST should pick the heaviest edges (1,2) and (2,0), leaving (0,1) out
        assert_eq!(fas.len(), 1);
        assert_eq!(fas[0], 0); // lightest edge not in max-weight spanning tree
    }

    #[test]
    fn weighted_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(1, 2).unwrap(); // eid 1
        g.add_edge(2, 0).unwrap(); // eid 2
        let weights = vec![1.0, 1.0, 100.0];
        let fas = feedback_arc_set(&g, Some(&weights), FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(!fas.is_empty());
    }

    #[test]
    fn invalid_weights_length() {
        let g = Graph::with_vertices(3);
        let weights = vec![1.0]; // wrong length
        assert!(feedback_arc_set(&g, Some(&weights), FasAlgorithm::EadesLinSmyth).is_err());
    }

    #[test]
    fn nan_weights_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![f64::NAN];
        assert!(feedback_arc_set(&g, Some(&weights), FasAlgorithm::EadesLinSmyth).is_err());
    }

    #[test]
    fn two_directed_cycles_sharing_vertex() {
        // 0→1→2→0, 0→3→4→0
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.len() >= 2);
        assert!(fas.len() <= 4);
    }

    #[test]
    fn bidirectional_edges() {
        // 0↔1 — both directions
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert_eq!(fas.len(), 1);
    }

    #[test]
    fn isolated_vertices_directed() {
        let g = Graph::new(5, true).unwrap();
        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();
        assert!(fas.is_empty());
    }

    #[test]
    fn removes_all_cycles() {
        // Verify the FAS actually removes all cycles by checking
        // the remaining graph is a DAG
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();

        let fas = feedback_arc_set(&g, None, FasAlgorithm::EadesLinSmyth).unwrap();

        // Rebuild graph without FAS edges and check it's acyclic
        let fas_set: std::collections::HashSet<EdgeId> = fas.into_iter().collect();
        let mut g2 = Graph::new(5, true).unwrap();
        for eid in 0..g.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let eid32 = eid as EdgeId;
            if !fas_set.contains(&eid32) {
                let (from, to) = g.edge(eid32).unwrap();
                g2.add_edge(from, to).unwrap();
            }
        }
        // The remaining graph should be a DAG
        let cycle = find_cycle(&g2, CycleMode::Out);
        assert!(
            cycle.is_ok() && cycle.unwrap().vertices.is_empty(),
            "FAS did not break all cycles"
        );
    }
}
