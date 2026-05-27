//! Graph adjacency matrix (ALGO-PR-042).
//!
//! Returns the adjacency matrix in dense form. Supports directed/undirected
//! graphs, optional edge weights, triangle selection (upper/lower/both)
//! for undirected graphs, and configurable self-loop handling.
//!
//! Counterpart of `igraph_get_adjacency` from
//! `references/igraph/src/misc/conversion.c`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Which triangle of the adjacency matrix to fill (undirected graphs only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjacencyType {
    /// Upper-right triangle only.
    Upper,
    /// Lower-left triangle only.
    Lower,
    /// Both triangles — full symmetric matrix.
    Both,
}

/// How to count self-loop edges in the adjacency matrix diagonal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopHandling {
    /// Ignore loops — diagonal is always zero.
    NoLoops,
    /// Count each loop once.
    Once,
    /// Count each loop twice for undirected graphs (once for directed).
    Twice,
}

/// Compute the adjacency matrix of a graph.
///
/// Returns an n×n matrix stored as `Vec<Vec<f64>>` in row-major order.
///
/// - `adj_type`: which triangle to fill (ignored for directed graphs).
/// - `weights`: optional edge weights. If `None`, each edge counts as 1.
/// - `loops`: how to handle self-loop edges on the diagonal.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_adjacency, AdjacencyType, LoopHandling};
///
/// // Triangle 0-1-2
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
/// assert!((adj[0][1] - 1.0).abs() < 1e-10);
/// assert!((adj[1][0] - 1.0).abs() < 1e-10);
/// assert!((adj[0][2] - 1.0).abs() < 1e-10);
/// ```
pub fn get_adjacency(
    graph: &Graph,
    adj_type: AdjacencyType,
    weights: Option<&[f64]>,
    loops: LoopHandling,
) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "get_adjacency: weight vector length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    let mut res = vec![vec![0.0_f64; n]; n];
    let directed = graph.is_directed();

    if directed {
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (from, to) = graph.edge(eid as u32)?;
            let f = from as usize;
            let t = to as usize;
            let w = weights.map_or(1.0, |ws| ws[eid]);

            if f != t || loops != LoopHandling::NoLoops {
                res[f][t] += w;
            }
        }
    } else {
        match adj_type {
            AdjacencyType::Upper => {
                for eid in 0..ecount {
                    #[allow(clippy::cast_possible_truncation)]
                    let (from, to) = graph.edge(eid as u32)?;
                    let f = from as usize;
                    let t = to as usize;
                    let w = weights.map_or(1.0, |ws| ws[eid]);

                    if t < f {
                        res[t][f] += w;
                    } else {
                        res[f][t] += w;
                    }
                    if f == t && loops == LoopHandling::Twice {
                        res[t][t] += w;
                    }
                }
            }
            AdjacencyType::Lower => {
                for eid in 0..ecount {
                    #[allow(clippy::cast_possible_truncation)]
                    let (from, to) = graph.edge(eid as u32)?;
                    let f = from as usize;
                    let t = to as usize;
                    let w = weights.map_or(1.0, |ws| ws[eid]);

                    if t < f {
                        res[f][t] += w;
                    } else {
                        res[t][f] += w;
                    }
                    if f == t && loops == LoopHandling::Twice {
                        res[t][t] += w;
                    }
                }
            }
            AdjacencyType::Both => {
                for eid in 0..ecount {
                    #[allow(clippy::cast_possible_truncation)]
                    let (from, to) = graph.edge(eid as u32)?;
                    let f = from as usize;
                    let t = to as usize;
                    let w = weights.map_or(1.0, |ws| ws[eid]);

                    res[f][t] += w;
                    if f == t {
                        if loops == LoopHandling::Twice {
                            res[t][f] += w;
                        }
                    } else {
                        res[t][f] += w;
                    }
                }
            }
        }
    }

    // Erase diagonal if NoLoops
    if loops == LoopHandling::NoLoops {
        for (i, row) in res.iter_mut().enumerate() {
            row[i] = 0.0;
        }
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert!(adj.is_empty());
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::with_vertices(1);
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert_eq!(adj.len(), 1);
        assert!(approx_eq(adj[0][0], 0.0));
    }

    #[test]
    fn undirected_single_edge_both() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 1.0));
        assert!(approx_eq(adj[1][0], 1.0));
        assert!(approx_eq(adj[0][0], 0.0));
        assert!(approx_eq(adj[1][1], 0.0));
    }

    #[test]
    fn undirected_single_edge_upper() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Upper, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 1.0));
        assert!(approx_eq(adj[1][0], 0.0));
    }

    #[test]
    fn undirected_single_edge_lower() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Lower, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 0.0));
        assert!(approx_eq(adj[1][0], 1.0));
    }

    #[test]
    fn undirected_triangle_both() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        // Symmetric
        for (i, row) in adj.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                assert!(approx_eq(val, adj[j][i]));
            }
        }
        assert!(approx_eq(adj[0][1], 1.0));
        assert!(approx_eq(adj[1][2], 1.0));
        assert!(approx_eq(adj[0][2], 1.0));
    }

    #[test]
    fn directed_basic() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 1.0));
        assert!(approx_eq(adj[1][0], 0.0)); // no reverse edge
        assert!(approx_eq(adj[1][2], 1.0));
        assert!(approx_eq(adj[2][1], 0.0));
    }

    #[test]
    fn self_loop_no_loops() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::NoLoops).unwrap();
        assert!(approx_eq(adj[0][0], 0.0));
        assert!(approx_eq(adj[0][1], 1.0));
    }

    #[test]
    fn self_loop_once() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][0], 1.0));
    }

    #[test]
    fn self_loop_twice_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Twice).unwrap();
        assert!(approx_eq(adj[0][0], 2.0));
    }

    #[test]
    fn self_loop_twice_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Twice).unwrap();
        // Directed: loops counted once even with Twice
        assert!(approx_eq(adj[0][0], 1.0));
    }

    #[test]
    fn weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![2.5, 3.0];
        let adj =
            get_adjacency(&g, AdjacencyType::Both, Some(&weights), LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 2.5));
        assert!(approx_eq(adj[1][0], 2.5));
        assert!(approx_eq(adj[1][2], 3.0));
        assert!(approx_eq(adj[2][1], 3.0));
    }

    #[test]
    fn multi_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Both, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][1], 2.0));
        assert!(approx_eq(adj[1][0], 2.0));
    }

    #[test]
    fn weight_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = get_adjacency(
            &g,
            AdjacencyType::Both,
            Some(&[1.0, 2.0]),
            LoopHandling::Once,
        );
        assert!(result.is_err());
    }

    #[test]
    fn upper_reversed_edge_order() {
        // Edge stored as (1, 0) internally — ensure upper still places in [0][1]
        let mut g = Graph::with_vertices(3);
        g.add_edge(2, 0).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Upper, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[0][2], 1.0));
        assert!(approx_eq(adj[2][0], 0.0));
    }

    #[test]
    fn lower_reversed_edge_order() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 2).unwrap();
        let adj = get_adjacency(&g, AdjacencyType::Lower, None, LoopHandling::Once).unwrap();
        assert!(approx_eq(adj[2][0], 1.0));
        assert!(approx_eq(adj[0][2], 0.0));
    }
}
