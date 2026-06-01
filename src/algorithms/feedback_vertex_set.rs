//! Feedback vertex set (ALGO-CY-003).
//!
//! A feedback vertex set (FVS) is a set of vertices whose removal makes
//! the graph acyclic. Finding a minimum FVS is NP-complete on both
//! directed and undirected graphs.
//!
//! We implement a greedy heuristic: repeatedly find a cycle, remove the
//! vertex with the highest degree from it, and repeat until no cycles
//! remain. This is O((V+E)·|FVS|) and produces a reasonable
//! approximation.
//!
//! Counterpart of `igraph_feedback_vertex_set` (which uses GLPK IP in C;
//! here we provide a dependency-free greedy heuristic).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Algorithm choice for computing the feedback vertex set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FvsAlgorithm {
    /// Greedy heuristic: iteratively remove the highest-degree vertex
    /// from each discovered cycle.
    Greedy,
}

/// Compute a feedback vertex set — a set of vertex IDs whose removal
/// makes the graph acyclic.
///
/// Uses a greedy heuristic that repeatedly finds a cycle and removes
/// the vertex with the highest degree. Not guaranteed to be minimum.
///
/// # Errors
///
/// - Returns an error if `weights` length does not match `vcount`.
/// - Returns an error if weights contain non-finite values.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, feedback_vertex_set, FvsAlgorithm};
///
/// // Directed cycle 0 → 1 → 2 → 0: removing one vertex breaks it.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
/// assert_eq!(fvs.len(), 1);
///
/// // DAG: no feedback vertices needed.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
/// assert!(fvs.is_empty());
/// ```
pub fn feedback_vertex_set(
    graph: &Graph,
    weights: Option<&[f64]>,
    _algo: FvsAlgorithm,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();

    if let Some(w) = weights {
        if w.len() != n as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match vcount {}",
                w.len(),
                n
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

    fvs_greedy(graph, weights)
}

fn find_cycle_avoiding(
    graph: &Graph,
    removed: &[bool],
    directed: bool,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let mut seen = vec![0u8; n as usize]; // 0=unseen, 1=ancestor, 2=visited

    let mut stack: Vec<(VertexId, usize)> = Vec::new();
    let mut path: Vec<VertexId> = Vec::new();

    for start in 0..n {
        if removed[start as usize] || seen[start as usize] != 0 {
            continue;
        }

        stack.clear();
        stack.push((start, 0));
        path.clear();

        while let Some(&mut (v, ref mut idx)) = stack.last_mut() {
            if *idx == 0 {
                if seen[v as usize] == 1 {
                    // Found a cycle back to v
                    let mut cycle = Vec::new();
                    let pos = path.iter().rposition(|&x| x == v).unwrap_or(0);
                    for &cv in &path[pos..] {
                        cycle.push(cv);
                    }
                    cycle.push(v);
                    return Ok(cycle);
                }
                if seen[v as usize] == 2 {
                    stack.pop();
                    continue;
                }
                seen[v as usize] = 1;
                path.push(v);
            }

            let neighbors = if directed {
                graph.out_neighbors_vec(v)?
            } else {
                graph.neighbors(v)?
            };

            let mut found_next = false;
            while *idx < neighbors.len() {
                let w = neighbors[*idx];
                *idx += 1;

                if removed[w as usize] {
                    continue;
                }

                if !directed && path.len() >= 2 && w == path[path.len() - 2] {
                    continue;
                }

                if seen[w as usize] == 1 {
                    // Found cycle: path from w's position to current + w
                    let mut cycle = Vec::new();
                    let pos = path.iter().rposition(|&x| x == w).unwrap_or(0);
                    for &cv in &path[pos..] {
                        cycle.push(cv);
                    }
                    return Ok(cycle);
                }

                if seen[w as usize] == 0 {
                    stack.push((w, 0));
                    found_next = true;
                    break;
                }
            }

            if !found_next {
                seen[v as usize] = 2;
                path.pop();
                stack.pop();
            }
        }
    }

    Ok(Vec::new())
}

#[allow(clippy::cast_possible_truncation)] // degree ≤ vcount which is u32
fn effective_degree(
    graph: &Graph,
    v: VertexId,
    removed: &[bool],
    directed: bool,
) -> IgraphResult<u32> {
    if directed {
        let out_n = graph.out_neighbors_vec(v)?;
        let in_n = graph.in_neighbors_vec(v)?;
        let out_deg = out_n.iter().filter(|&&w| !removed[w as usize]).count() as u32;
        let in_deg = in_n.iter().filter(|&&w| !removed[w as usize]).count() as u32;
        Ok(out_deg.saturating_add(in_deg))
    } else {
        let n = graph.neighbors(v)?;
        Ok(n.iter().filter(|&&w| !removed[w as usize]).count() as u32)
    }
}

fn pick_best_vertex(
    graph: &Graph,
    cycle: &[VertexId],
    weights: Option<&[f64]>,
    removed: &[bool],
    directed: bool,
) -> IgraphResult<VertexId> {
    let mut best_v = cycle[0];
    if let Some(w) = weights {
        let mut best_w = w[cycle[0] as usize];
        for &v in &cycle[1..] {
            let vw = w[v as usize];
            if vw > best_w || (vw > best_w - f64::EPSILON && v < best_v) {
                best_w = vw;
                best_v = v;
            }
        }
    } else {
        let mut best_deg = effective_degree(graph, cycle[0], removed, directed)?;
        for &v in &cycle[1..] {
            let deg = effective_degree(graph, v, removed, directed)?;
            if deg > best_deg || (deg == best_deg && v < best_v) {
                best_deg = deg;
                best_v = v;
            }
        }
    }
    Ok(best_v)
}

fn fvs_greedy(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let mut removed = vec![false; n as usize];
    let mut result = Vec::new();

    loop {
        let cycle = find_cycle_avoiding(graph, &removed, directed)?;
        if cycle.is_empty() {
            break;
        }

        let best_v = pick_best_vertex(graph, &cycle, weights, &removed, directed)?;
        removed[best_v as usize] = true;
        result.push(best_v);
    }

    result.sort_unstable();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_is_acyclic_without(graph: &Graph, fvs: &[VertexId]) -> bool {
        let n = graph.vcount();
        let mut removed = vec![false; n as usize];
        for &v in fvs {
            removed[v as usize] = true;
        }
        let cycle = find_cycle_avoiding(graph, &removed, graph.is_directed()).unwrap();
        cycle.is_empty()
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert!(fvs.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert!(fvs.is_empty());
    }

    #[test]
    fn dag_no_feedback() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 3).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert!(fvs.is_empty());
    }

    #[test]
    fn simple_directed_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn directed_cycle_4() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn undirected_tree_no_feedback() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert!(fvs.is_empty());
    }

    #[test]
    fn undirected_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(graph_is_acyclic_without(&g, &fvs));
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
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        // Removing vertex 0 breaks both cycles
        assert!(fvs.len() <= 2);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn bidirectional_edges() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn self_loop_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(fvs.contains(&0));
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn self_loop_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert!(fvs.contains(&0));
    }

    #[test]
    fn undirected_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        // K4 needs at least 2 vertex removals to become acyclic
        assert!(fvs.len() >= 2);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn weighted_picks_heaviest() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Vertex 1 has highest weight — should be picked first
        let weights = vec![1.0, 10.0, 1.0];
        let fvs = feedback_vertex_set(&g, Some(&weights), FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 1);
        assert_eq!(fvs[0], 1);
    }

    #[test]
    fn invalid_weights_length() {
        let g = Graph::with_vertices(3);
        let weights = vec![1.0]; // wrong length
        assert!(feedback_vertex_set(&g, Some(&weights), FvsAlgorithm::Greedy).is_err());
    }

    #[test]
    fn nan_weights_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = vec![f64::NAN, 1.0];
        assert!(feedback_vertex_set(&g, Some(&weights), FvsAlgorithm::Greedy).is_err());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::new(5, true).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert!(fvs.is_empty());
    }

    #[test]
    fn complex_directed() {
        // Two overlapping cycles: 0→1→2→0 and 2→3→4→2
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        // Removing vertex 2 breaks both cycles
        assert!(fvs.len() <= 2);
        assert!(graph_is_acyclic_without(&g, &fvs));
    }

    #[test]
    fn sorted_output() {
        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 2).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 4).unwrap();
        let fvs = feedback_vertex_set(&g, None, FvsAlgorithm::Greedy).unwrap();
        assert_eq!(fvs.len(), 3);
        // Verify sorted
        for i in 1..fvs.len() {
            assert!(fvs[i] > fvs[i - 1]);
        }
        assert!(graph_is_acyclic_without(&g, &fvs));
    }
}
