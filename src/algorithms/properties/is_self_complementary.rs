//! Self-complementary graph predicate (ALGO-PR-076).
//!
//! A graph G is self-complementary if it is isomorphic to its
//! complement. Self-complementary graphs have n(n-1)/4 edges (so n
//! must be 0 or 1 mod 4), and include P4, C5, and the Paley graphs.
//!
//! Recognition: build the complement and test isomorphism via VF2.
//! Returns `false` for directed graphs (the complement is only
//! well-defined for simple undirected graphs in this context).

use crate::algorithms::isomorphism::vf2::isomorphic_vf2;
use crate::algorithms::operators::complementer::complementer;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is self-complementary.
///
/// A graph is self-complementary if it is isomorphic to its complement.
///
/// Returns `false` for directed graphs. Handles simple undirected
/// graphs; for multigraphs or graphs with self-loops, the complement
/// is computed without loops.
///
/// An empty graph (0 vertices) is self-complementary (vacuously).
/// A single vertex is self-complementary.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_self_complementary};
///
/// // P4 (0-1-2-3) is self-complementary
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_self_complementary(&g).unwrap());
///
/// // Triangle K3 is NOT self-complementary
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(!is_self_complementary(&g).unwrap());
/// ```
pub fn is_self_complementary(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    let n64 = u64::from(n);
    let total_possible = n64.saturating_mul(n64.saturating_sub(1)) / 2;
    let ecount = graph.ecount() as u64;
    if total_possible % 2 != 0 || ecount != total_possible / 2 {
        return Ok(false);
    }

    let comp = complementer(graph, false)?;
    let result = isomorphic_vf2(graph, &comp, None, None, None, None)?;

    Ok(result.iso)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_self_complementary(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_self_complementary(&g).unwrap());
    }

    #[test]
    fn k2_not_self_complementary() {
        // K2: 1 edge, complement has 0 edges, not isomorphic
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn k3_not_self_complementary() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn path_4_is_self_complementary() {
        // P4: 0-1-2-3, complement: 0-2, 0-3, 1-3 → isomorphic to P4
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_self_complementary(&g).unwrap());
    }

    #[test]
    fn cycle_5_is_self_complementary() {
        // C5: 5 edges, complement is also C5 (Petersen inner star)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_self_complementary(&g).unwrap());
    }

    #[test]
    fn cycle_4_not_self_complementary() {
        // C4: 4 edges, n(n-1)/2 = 6, need 3 edges for self-comp → wrong edge count
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn edgeless_2_not_self_complementary() {
        // 2 vertices, 0 edges: complement has 1 edge → not iso
        let g = Graph::with_vertices(2);
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn edgeless_4_not_self_complementary() {
        // 4 vertices, 0 edges: need 3 edges → wrong count
        let g = Graph::with_vertices(4);
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn k4_not_self_complementary() {
        // K4: 6 edges, need 3 → wrong count
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn star_k13_not_self_complementary() {
        // Star on 4 vertices: 3 edges = n(n-1)/4 = 3 → right count
        // But star is not isomorphic to its complement (which is K3 + isolated)
        // Wait: complement of star K_{1,3} on 4 verts is triangle on {1,2,3}
        // with vertex 0 isolated. Star has degree sequence [3,1,1,1],
        // complement has [0,2,2,2]. Not isomorphic.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn three_vertices_not_possible() {
        // n=3: n(n-1)/2 = 3, 3/2 is not integer → no self-comp graph on 3 verts
        // P3: 2 edges ≠ 1.5 → early false
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_self_complementary(&g).unwrap());
    }

    #[test]
    fn bull_graph_is_self_complementary() {
        // Bull: triangle 0-1-2 + pendants 0-3, 1-4: complement is also a bull
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_self_complementary(&g).unwrap());
    }
}
