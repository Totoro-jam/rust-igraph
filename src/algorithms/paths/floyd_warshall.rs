//! Floyd-Warshall all-pairs shortest distances (ALGO-SP-004).
//!
//! Counterpart of `igraph_distances_floyd_warshall()` from
//! `references/igraph/src/paths/floyd_warshall.c`. Phase-1 minimal
//! slice: returns the dense distance matrix only (no path
//! reconstruction), `IGRAPH_OUT` mode on directed graphs, undirected
//! graphs walk both endpoints symmetrically. Implements the textbook
//! O(V³) variant; the Brodnik-tree speedup ships later.
//!
//! Negative weights are accepted on directed graphs except for self-
//! loops (a self-loop with `w < 0` is itself a negative cycle).
//! Undirected graphs reject `w < 0` outright because every undirected
//! edge would form a 2-cycle of weight `2w < 0`. After relaxation,
//! `dist[i][i] < 0` triggers a negative-cycle error.
//!
//! `+inf` weights are silently ignored (matches igraph C).

use crate::core::{Graph, IgraphError, IgraphResult};

/// All-pairs shortest distances via Floyd-Warshall.
///
/// `weights = None` collapses to unit weights and reproduces unweighted
/// BFS distances (modulo cost: BFS is O(V·(V+E)) vs FW's O(V³)). When
/// `weights = Some(w)`, `w.len()` must equal `graph.ecount()`.
///
/// Returns a `vcount × vcount` matrix where `out[i][j] = Some(d)` if
/// there is a path `i → j` of weighted length `d`, and `None` if `j`
/// is unreachable from `i`. The diagonal is always `Some(0.0)` (a
/// non-trivial cycle through `i` of total weight `0` is *not*
/// surfaced — this matches igraph's C contract; only *negative* cycles
/// are flagged as errors).
///
/// Errors:
/// - `weights.len() != ecount` → `InvalidArgument`.
/// - any `w == NaN` → `InvalidArgument`.
/// - undirected graph with any `w < 0` → `InvalidArgument` (negative
///   2-cycle).
/// - directed graph with self-loop of `w < 0` → `InvalidArgument`.
/// - negative cycle detected during relaxation → `InvalidArgument`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, floyd_warshall_distances};
///
/// // Triangle 0-1-2 with weights 1, 4, 2 → dist(0,2) = 1+2 = 3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let d = floyd_warshall_distances(&g, Some(&[1.0, 4.0, 2.0])).unwrap();
/// assert_eq!(d[0][2], Some(3.0));
/// assert_eq!(d[0][0], Some(0.0));
/// ```
pub fn floyd_warshall_distances(
    graph: &Graph,
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    let n = graph.vcount();
    let m = graph.ecount();
    let n_us = n as usize;
    let directed = graph.is_directed();

    if let Some(ws) = weights {
        if ws.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "weights vector size ({}) differs from edge count ({})",
                ws.len(),
                m
            )));
        }
        for (e, &w) in ws.iter().enumerate() {
            if w.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "weight at edge {e} is NaN"
                )));
            }
        }
    }

    // Flat n×n row-major matrix with f64::INFINITY sentinel for "no
    // edge yet" so `min(d[i][k] + d[k][j], d[i][j])` falls back
    // cleanly. Diagonal seeded to 0.
    let mut dist = vec![f64::INFINITY; n_us * n_us];
    for v in 0..n_us {
        dist[v * n_us + v] = 0.0;
    }

    // Seed direct edges (taking the minimum across multi-edges).
    let m_u32 = u32::try_from(m).map_err(|_| IgraphError::Internal("edge count overflows u32"))?;
    for eid in 0..m_u32 {
        let (src, tgt) = graph.edge(eid)?;
        let edge_w = weights.map_or(1.0, |ws| ws[eid as usize]);

        if edge_w < 0.0 {
            if !directed {
                return Err(IgraphError::InvalidArgument(format!(
                    "negative edge weight ({edge_w}) on undirected graph: every undirected edge induces a 2-cycle, so this is a negative cycle"
                )));
            }
            if src == tgt {
                return Err(IgraphError::InvalidArgument(format!(
                    "self-loop with negative weight ({edge_w}) at vertex {src} is a negative cycle"
                )));
            }
        }
        if edge_w == f64::INFINITY {
            // Match igraph C: ignore +inf edges.
            continue;
        }

        let s_us = src as usize;
        let t_us = tgt as usize;
        // OUT mode: forward only on directed; both ways on undirected.
        let fwd = s_us * n_us + t_us;
        if edge_w < dist[fwd] {
            dist[fwd] = edge_w;
        }
        if !directed {
            let rev = t_us * n_us + s_us;
            if edge_w < dist[rev] {
                dist[rev] = edge_w;
            }
        }
    }

    if n_us > 1 {
        // Textbook Floyd-Warshall: relax via every intermediate `k`.
        for k in 0..n_us {
            for i in 0..n_us {
                let dik = dist[i * n_us + k];
                if dik.is_infinite() {
                    continue;
                }
                for j in 0..n_us {
                    let dkj = dist[k * n_us + j];
                    if dkj.is_infinite() {
                        continue;
                    }
                    let candidate = dik + dkj;
                    let cell = i * n_us + j;
                    if candidate < dist[cell] {
                        dist[cell] = candidate;
                    }
                }
                let diag = i * n_us + i;
                if dist[diag] < 0.0 {
                    return Err(IgraphError::InvalidArgument(format!(
                        "negative cycle found while computing Floyd-Warshall distances (vertex {i} closes a cycle of weight {})",
                        dist[diag]
                    )));
                }
            }
        }
    }

    let mut out = Vec::with_capacity(n_us);
    for i in 0..n_us {
        let mut row = Vec::with_capacity(n_us);
        for j in 0..n_us {
            let d = dist[i * n_us + j];
            row.push(if d.is_infinite() { None } else { Some(d) });
        }
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_directed(n: u32) -> Graph {
        Graph::new(n, true).expect("directed graph")
    }

    #[test]
    fn empty_graph_returns_empty_matrix() {
        let g = Graph::with_vertices(0);
        let d = floyd_warshall_distances(&g, None).unwrap();
        assert!(d.is_empty());
    }

    #[test]
    fn singleton_distance_zero() {
        let g = Graph::with_vertices(1);
        let d = floyd_warshall_distances(&g, None).unwrap();
        assert_eq!(d, vec![vec![Some(0.0)]]);
    }

    #[test]
    fn unweighted_triangle_unit_distances() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let d = floyd_warshall_distances(&g, None).unwrap();
        for (i, row) in d.iter().enumerate() {
            assert_eq!(row[i], Some(0.0));
            for (j, cell) in row.iter().enumerate() {
                if i != j {
                    assert_eq!(*cell, Some(1.0));
                }
            }
        }
    }

    #[test]
    fn weighted_triangle_picks_shortcut() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // weight 1
        g.add_edge(0, 2).unwrap(); // weight 4
        g.add_edge(1, 2).unwrap(); // weight 2
        let d = floyd_warshall_distances(&g, Some(&[1.0, 4.0, 2.0])).unwrap();
        assert_eq!(d[0][2], Some(3.0)); // 0→1→2 cheaper than direct
        assert_eq!(d[2][0], Some(3.0)); // undirected → symmetric
    }

    #[test]
    fn unreachable_pair_yields_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let d = floyd_warshall_distances(&g, None).unwrap();
        assert_eq!(d[0][2], None);
        assert_eq!(d[2][0], None);
        assert_eq!(d[1][2], None);
    }

    #[test]
    fn directed_graph_respects_orientation() {
        let mut g = make_directed(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = floyd_warshall_distances(&g, None).unwrap();
        assert_eq!(d[0][2], Some(2.0));
        assert_eq!(d[2][0], None); // backward unreachable
    }

    #[test]
    fn directed_graph_with_negative_weight() {
        // Directed weights can be negative if they don't induce a cycle.
        let mut g = make_directed(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = floyd_warshall_distances(&g, Some(&[-5.0, 10.0])).unwrap();
        assert_eq!(d[0][2], Some(5.0));
        assert_eq!(d[0][1], Some(-5.0));
    }

    #[test]
    fn directed_negative_cycle_errors() {
        // 0→1→2→0 with weights -1, -1, -1 is a negative cycle.
        let mut g = make_directed(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = floyd_warshall_distances(&g, Some(&[-1.0, -1.0, -1.0]));
        assert!(result.is_err());
    }

    #[test]
    fn directed_self_loop_negative_errors() {
        let mut g = make_directed(2);
        g.add_edge(0, 0).unwrap();
        let result = floyd_warshall_distances(&g, Some(&[-0.5]));
        assert!(result.is_err());
    }

    #[test]
    fn undirected_negative_weight_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = floyd_warshall_distances(&g, Some(&[-1.0]));
        assert!(result.is_err());
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = floyd_warshall_distances(&g, Some(&[1.0, 2.0]));
        assert!(result.is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = floyd_warshall_distances(&g, Some(&[f64::NAN]));
        assert!(result.is_err());
    }

    #[test]
    fn infinity_weight_silently_ignored() {
        // 0—1 with weight inf, 0—2 with weight 1, 1—2 with weight 1.
        // Without the inf edge, 0→1 must route via 2: dist(0,1) = 2.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = floyd_warshall_distances(&g, Some(&[f64::INFINITY, 1.0, 1.0])).unwrap();
        assert_eq!(d[0][1], Some(2.0));
    }

    #[test]
    fn parallel_edges_pick_minimum() {
        // Two 0—1 edges with weights 5 and 1; FW must pick the lighter.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = floyd_warshall_distances(&g, Some(&[5.0, 1.0])).unwrap();
        assert_eq!(d[0][1], Some(1.0));
    }

    #[test]
    fn diagonal_always_zero_for_isolated_vertex() {
        let g = Graph::with_vertices(5);
        let d = floyd_warshall_distances(&g, None).unwrap();
        for (i, row) in d.iter().enumerate() {
            assert_eq!(row[i], Some(0.0));
            for (j, cell) in row.iter().enumerate() {
                if i != j {
                    assert_eq!(*cell, None);
                }
            }
        }
    }

    #[test]
    fn unit_weights_match_unweighted() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 3).unwrap();
        let unweighted = floyd_warshall_distances(&g, None).unwrap();
        let weighted = floyd_warshall_distances(&g, Some(&[1.0, 1.0, 1.0, 1.0])).unwrap();
        assert_eq!(unweighted, weighted);
    }
}
