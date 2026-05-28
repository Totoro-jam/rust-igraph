//! Complete bipartite graph constructor (ALGO-CN-035).
//!
//! Counterpart of `igraph_full_bipartite()` in
//! `references/igraph/src/misc/bipartite.c:448-528`.
//!
//! A *complete bipartite graph* K_{n1, n2} has two partitions of sizes
//! `n1` and `n2` with every possible edge between the two partitions
//! (no edges within a partition).
//!
//! Vertices `[0, n1)` form partition 1 and vertices `[n1, n1+n2)` form
//! partition 2.
//!
//! Direction is governed by [`BipartiteMode`]:
//!
//! * [`BipartiteMode::Out`] — arcs flow from partition 1 to partition 2.
//! * [`BipartiteMode::In`]  — arcs flow from partition 2 to partition 1.
//! * [`BipartiteMode::All`] — undirected, or directed with mutual arcs.
//!
//! Edge counts:
//!
//! * Undirected / directed `Out` / directed `In`: `n1 × n2`.
//! * Directed `All`: `2 × n1 × n2`.

pub use crate::algorithms::games::bipartite::BipartiteMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Bundled return type of [`full_bipartite`].
#[derive(Debug, Clone)]
pub struct FullBipartite {
    /// The constructed graph.
    pub graph: Graph,
    /// Per-vertex type: `false` = partition 1, `true` = partition 2.
    pub types: Vec<bool>,
}

/// Build the complete bipartite graph K_{n1, n2}.
///
/// `n1` vertices in partition 1, `n2` vertices in partition 2.
/// The result graph has `n1 + n2` vertices and `n1 × n2` edges
/// (or `2 × n1 × n2` arcs when directed mutual).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — vertex count `n1 + n2`
///   overflows `u32`, or the edge count overflows `usize`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{full_bipartite, BipartiteMode};
///
/// // K_{2,3} undirected: 5 vertices, 6 edges.
/// let r = full_bipartite(2, 3, false, BipartiteMode::All).unwrap();
/// assert_eq!(r.graph.vcount(), 5);
/// assert_eq!(r.graph.ecount(), 6);
/// assert_eq!(r.types, vec![false, false, true, true, true]);
///
/// // K_{2,3} directed out: 5 vertices, 6 arcs.
/// let d = full_bipartite(2, 3, true, BipartiteMode::Out).unwrap();
/// assert_eq!(d.graph.ecount(), 6);
/// assert!(d.graph.is_directed());
///
/// // K_{2,3} directed mutual: 12 arcs.
/// let m = full_bipartite(2, 3, true, BipartiteMode::All).unwrap();
/// assert_eq!(m.graph.ecount(), 12);
/// ```
pub fn full_bipartite(
    n1: u32,
    n2: u32,
    directed: bool,
    mode: BipartiteMode,
) -> IgraphResult<FullBipartite> {
    let total = n1.checked_add(n2).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "full_bipartite: total vertex count {n1} + {n2} overflows u32"
        ))
    })?;

    if total == 0 {
        let graph = Graph::new(0, directed)?;
        return Ok(FullBipartite {
            graph,
            types: Vec::new(),
        });
    }

    let n1_us = n1 as usize;
    let n2_us = n2 as usize;
    let base_edges = n1_us.checked_mul(n2_us).ok_or_else(|| {
        IgraphError::InvalidArgument("full_bipartite: edge count overflows usize".to_string())
    })?;
    let num_edges = if directed && mode == BipartiteMode::All {
        base_edges.checked_mul(2).ok_or_else(|| {
            IgraphError::InvalidArgument(
                "full_bipartite: mutual edge count overflows usize".to_string(),
            )
        })?
    } else {
        base_edges
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(num_edges);

    match mode {
        BipartiteMode::Out | BipartiteMode::All if !directed => {
            for i in 0..n1 {
                for j in 0..n2 {
                    edges.push((i, n1 + j));
                }
            }
        }
        BipartiteMode::Out => {
            for i in 0..n1 {
                for j in 0..n2 {
                    edges.push((i, n1 + j));
                }
            }
        }
        BipartiteMode::In => {
            for i in 0..n1 {
                for j in 0..n2 {
                    edges.push((n1 + j, i));
                }
            }
        }
        BipartiteMode::All => {
            for i in 0..n1 {
                for j in 0..n2 {
                    edges.push((i, n1 + j));
                    edges.push((n1 + j, i));
                }
            }
        }
    }

    let mut graph = Graph::new(total, directed)?;
    graph.add_edges(edges)?;

    let mut types = vec![false; total as usize];
    for i in n1..total {
        types[i as usize] = true;
    }

    Ok(FullBipartite { graph, types })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let r = full_bipartite(0, 0, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.types.is_empty());
    }

    #[test]
    fn one_partition_empty() {
        let r = full_bipartite(3, 0, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 3);
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.types, vec![false, false, false]);
    }

    #[test]
    fn other_partition_empty() {
        let r = full_bipartite(0, 4, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.types, vec![true, true, true, true]);
    }

    #[test]
    fn k23_undirected() {
        let r = full_bipartite(2, 3, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 5);
        assert_eq!(r.graph.ecount(), 6);
        assert!(!r.graph.is_directed());
        assert_eq!(r.types, vec![false, false, true, true, true]);
    }

    #[test]
    fn k11_undirected() {
        let r = full_bipartite(1, 1, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.types, vec![false, true]);
    }

    #[test]
    fn k33_undirected() {
        let r = full_bipartite(3, 3, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 6);
        assert_eq!(r.graph.ecount(), 9);
    }

    #[test]
    fn directed_out() {
        let r = full_bipartite(2, 3, true, BipartiteMode::Out).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 6);
        // All edges go from partition 1 → partition 2
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let e = eid as u32;
            let (s, t) = r.graph.edge(e).unwrap();
            assert!(!r.types[s as usize], "source should be partition 1");
            assert!(r.types[t as usize], "target should be partition 2");
        }
    }

    #[test]
    fn directed_in() {
        let r = full_bipartite(2, 3, true, BipartiteMode::In).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 6);
        // All edges go from partition 2 → partition 1
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let e = eid as u32;
            let (s, t) = r.graph.edge(e).unwrap();
            assert!(r.types[s as usize], "source should be partition 2");
            assert!(!r.types[t as usize], "target should be partition 1");
        }
    }

    #[test]
    fn directed_mutual() {
        let r = full_bipartite(2, 3, true, BipartiteMode::All).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 12); // 2 × 2 × 3
    }

    #[test]
    fn types_correctness() {
        let r = full_bipartite(4, 2, false, BipartiteMode::All).unwrap();
        assert_eq!(r.types, vec![false, false, false, false, true, true]);
    }

    #[test]
    fn bipartite_structure_no_within_partition_edges() {
        let r = full_bipartite(3, 4, false, BipartiteMode::All).unwrap();
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let e = eid as u32;
            let (s, t) = r.graph.edge(e).unwrap();
            assert_ne!(
                r.types[s as usize], r.types[t as usize],
                "edge ({s}, {t}) connects same partition"
            );
        }
    }

    #[test]
    fn overflow_vertex_count_errors() {
        let result = full_bipartite(u32::MAX, 1, false, BipartiteMode::All);
        assert!(result.is_err());
    }
}
