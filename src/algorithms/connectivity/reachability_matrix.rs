//! Reachability matrix (ALGO-CC-021).
//!
//! Returns a dense `Vec<Vec<bool>>` where `r[i][j]` is `true` iff vertex
//! `j` is reachable from vertex `i` in 0 or more steps (with `r[i][i]`
//! always `true`).
//!
//! Counterpart of `igraph_reachability()` from
//! `references/igraph/src/connectivity/reachability.c:72-148` (which
//! returns a `bitset_list` indexed by SCC). Phase-1 minimal slice
//! returns the dense per-vertex bitmap and uses BFS-from-each-vertex
//! (O(|V|*(|V|+|E|))). The SCC-condensation O(|C||V|/w + |V|+|E|)
//! optimisation lands later.
//!
//! Edge directions follow `crate::distances`: `IGRAPH_OUT` for
//! directed graphs, full undirected reachability otherwise.

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

/// Dense reachability matrix of `graph`.
/// `result[i][j]` is `true` iff vertex `j` is reachable from `i` in
/// zero or more steps.
///
/// On directed graphs follows out-edges (matches upstream's
/// `IGRAPH_OUT` mode); on undirected graphs reachability is symmetric.
///
/// Counterpart of `igraph_reachability(_, _, _, _, _, IGRAPH_OUT)`
/// (dense output; upstream returns a per-SCC bitset list).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reachability_matrix};
///
/// // Directed 0->1->2: 0 can reach all; 1 can reach 1, 2; 2 only 2.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let m = reachability_matrix(&g).unwrap();
/// assert_eq!(m[0], vec![true, true, true]);
/// assert_eq!(m[1], vec![false, true, true]);
/// assert_eq!(m[2], vec![false, false, true]);
/// ```
pub fn reachability_matrix(graph: &Graph) -> IgraphResult<Vec<Vec<bool>>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut out = Vec::with_capacity(n_us);
    for v in 0..n {
        let d = distances(graph, v)?;
        // `distances[w] == Some(_)` ⇔ `w` is reachable from `v`.
        let row: Vec<bool> = d.into_iter().map(|x| x.is_some()).collect();
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_empty_matrix() {
        let g = Graph::with_vertices(0);
        let m = reachability_matrix(&g).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn isolated_vertices_only_self_reachable() {
        let g = Graph::with_vertices(3);
        let m = reachability_matrix(&g).unwrap();
        assert_eq!(m[0], vec![true, false, false]);
        assert_eq!(m[1], vec![false, true, false]);
        assert_eq!(m[2], vec![false, false, true]);
    }

    #[test]
    fn undirected_path_is_symmetric_and_full() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let m = reachability_matrix(&g).unwrap();
        for (i, row) in m.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                assert!(val, "{i}->{j} should be reachable");
                assert_eq!(val, m[j][i], "asymmetry on undirected");
            }
        }
    }

    #[test]
    fn directed_chain_is_one_way() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let m = reachability_matrix(&g).unwrap();
        assert_eq!(m[0], vec![true, true, true]);
        assert_eq!(m[1], vec![false, true, true]);
        assert_eq!(m[2], vec![false, false, true]);
    }

    #[test]
    fn directed_cycle_makes_all_pairs_reachable() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let m = reachability_matrix(&g).unwrap();
        for (i, row) in m.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                assert!(val, "{i}->{j} should be reachable in a 3-cycle");
            }
        }
    }

    #[test]
    fn disconnected_components_isolate_pairs() {
        // {0,1,2} ∪ {3,4}: cross-component pairs must be false.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        let m = reachability_matrix(&g).unwrap();
        // Within-component {0,1,2}: all true.
        for row in &m[..3] {
            for &v in &row[..3] {
                assert!(v);
            }
        }
        // Cross-component {0,1,2} x {3,4}: all false.
        for (i, row) in m.iter().enumerate().take(3) {
            for (j, &val) in row.iter().enumerate().skip(3).take(2) {
                assert!(!val, "{i}->{j} should be unreachable");
                assert!(!m[j][i], "{j}->{i} should be unreachable");
            }
        }
    }

    #[test]
    fn diagonal_is_always_true() {
        let g = Graph::with_vertices(7);
        let m = reachability_matrix(&g).unwrap();
        for (i, row) in m.iter().enumerate() {
            assert!(row[i], "vertex {i} not self-reachable");
        }
    }
}
