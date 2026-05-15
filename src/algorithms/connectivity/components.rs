//! Weakly connected components (ALGO-CC-001).
//!
//! Counterpart of `igraph_connected_components()` from
//! `references/igraph/src/connectivity/components.c:82-180` in the
//! `IGRAPH_WEAK` (or undirected) branch.
//!
//! Phase 1 minimum: returns one membership entry per vertex plus the
//! component count. Strongly-connected components (Tarjan) and
//! component-size vector are follow-up AWUs (CC-002, CC-003).

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of a weak connected-components scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedComponents {
    /// `membership[v]` == component id of vertex `v` (0-indexed).
    pub membership: Vec<u32>,
    /// Total number of components.
    pub count: u32,
}

/// Compute the weak connected components of `graph`.
///
/// Counterpart of `igraph_connected_components(_, _, _, _, IGRAPH_WEAK)`.
/// On undirected graphs this is just "the connected components"; on
/// directed graphs it ignores edge direction (each edge counts as both
/// ways).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, connected_components};
///
/// // Two components: {0, 1, 2} and {3, 4}.
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let cc = connected_components(&g).unwrap();
/// assert_eq!(cc.count, 2);
/// assert_eq!(cc.membership[0], cc.membership[1]);
/// assert_eq!(cc.membership[1], cc.membership[2]);
/// assert_eq!(cc.membership[3], cc.membership[4]);
/// assert_ne!(cc.membership[0], cc.membership[3]);
/// ```
pub fn connected_components(graph: &Graph) -> IgraphResult<ConnectedComponents> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(ConnectedComponents {
            membership: Vec::new(),
            count: 0,
        });
    }

    let mut membership = vec![u32::MAX; n as usize];
    let mut count: u32 = 0;
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    for start in 0..n {
        if membership[start as usize] != u32::MAX {
            continue;
        }
        let component_id = count;
        count = count
            .checked_add(1)
            .ok_or(crate::core::IgraphError::Internal("too many components"))?;

        membership[start as usize] = component_id;
        queue.clear();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for w in graph.neighbors(v)? {
                if membership[w as usize] == u32::MAX {
                    membership[w as usize] = component_id;
                    queue.push_back(w);
                }
            }
        }
    }

    Ok(ConnectedComponents { membership, count })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 0);
        assert!(cc.membership.is_empty());
    }

    #[test]
    fn n_isolated_vertices_yields_n_components() {
        let g = Graph::with_vertices(5);
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 5);
        // Each vertex in its own component, ids 0..5 in order.
        assert_eq!(cc.membership, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn single_path_is_one_component() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 1);
        assert_eq!(cc.membership, vec![0, 0, 0, 0]);
    }

    #[test]
    fn two_disjoint_components_are_distinct() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 2);
        // Component ids assigned in vertex-id order: {0,1,2} → 0; {3,4} → 1.
        assert_eq!(cc.membership, vec![0, 0, 0, 1, 1]);
    }

    #[test]
    fn self_loop_does_not_split_component() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 1);
        assert_eq!(cc.membership, vec![0, 0, 0]);
    }

    #[test]
    fn directed_graph_uses_weak_connectivity() {
        // 0 -> 1 -> 2: weakly connected even though no path back.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 1);
        assert_eq!(cc.membership, vec![0, 0, 0]);
    }

    #[test]
    fn membership_ids_are_dense() {
        // Components 0..k must use exactly the ids 0..k (no gaps).
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(4, 5).unwrap();
        let cc = connected_components(&g).unwrap();
        assert_eq!(cc.count, 3);
        let mut seen: Vec<u32> = cc.membership.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen, vec![0, 1, 2]);
    }
}
