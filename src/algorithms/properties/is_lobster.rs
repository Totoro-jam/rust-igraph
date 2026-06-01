//! Lobster tree predicate (ALGO-PR-087).
//!
//! A lobster is a tree in which every vertex is within distance 2 of
//! a central path. Equivalently, removing all leaf vertices from a
//! lobster yields a caterpillar (and removing leaves again yields a
//! path).
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a lobster tree.
///
/// A lobster is a tree where every vertex is within distance 2 of a
/// central path. Equivalently, removing all leaves twice yields a path
/// (or empty/single vertex).
///
/// Returns `false` for directed graphs and non-tree graphs.
/// Returns `true` for the empty graph, single vertex, paths, stars,
/// and caterpillars (all are lobsters).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_lobster};
///
/// // Star is a lobster (it's even a caterpillar)
/// let mut g = Graph::with_vertices(5);
/// for i in 1..5u32 {
///     g.add_edge(0, i).unwrap();
/// }
/// assert!(is_lobster(&g).unwrap());
///
/// // Caterpillar with secondary branches → lobster
/// let mut g = Graph::with_vertices(8);
/// g.add_edge(0, 1).unwrap(); // spine
/// g.add_edge(1, 2).unwrap(); // spine
/// g.add_edge(0, 3).unwrap(); // leaf off spine
/// g.add_edge(1, 4).unwrap(); // branch
/// g.add_edge(4, 5).unwrap(); // leaf off branch (distance 2 from spine)
/// g.add_edge(2, 6).unwrap(); // branch
/// g.add_edge(6, 7).unwrap(); // leaf off branch
/// assert!(is_lobster(&g).unwrap());
/// ```
pub fn is_lobster(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }

    if is_tree(graph, DijkstraMode::All)?.is_none() {
        return Ok(false);
    }

    // Compute degrees
    let mut deg = vec![0u32; n as usize];
    for v in 0..n {
        deg[v as usize] = u32::try_from(graph.degree(v)?).unwrap_or(u32::MAX);
    }

    // First pass: identify leaves (degree 1) and compute "pruned degrees"
    // (degree after removing leaves).
    let is_leaf1 = |v: usize| deg[v] == 1;

    let mut pruned_deg = vec![0u32; n as usize];
    for v in 0..n {
        let vi = v as usize;
        if is_leaf1(vi) {
            continue;
        }
        let nbrs = graph.neighbors(v)?;
        pruned_deg[vi] = u32::try_from(nbrs.iter().filter(|&&w| !is_leaf1(w as usize)).count())
            .unwrap_or(u32::MAX);
    }

    // Second pass: in the pruned graph, identify new leaves (pruned_deg == 1)
    // and check that the doubly-pruned graph is a path (all remaining
    // vertices have at most 2 non-leaf-1, non-leaf-2 neighbors).
    let is_leaf2 = |v: usize| !is_leaf1(v) && pruned_deg[v] <= 1;

    for v in 0..n {
        let vi = v as usize;
        if is_leaf1(vi) || is_leaf2(vi) {
            continue;
        }
        let nbrs = graph.neighbors(v)?;
        let spine_nbrs = nbrs
            .iter()
            .filter(|&&w| !is_leaf1(w as usize) && !is_leaf2(w as usize))
            .count();
        if spine_nbrs > 2 {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn caterpillar_is_lobster() {
        // Spine: 0-1-2-3, leaves: 4,5 on 1; 6 on 2
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn lobster_with_depth_2_branches() {
        // Spine: 0-1, branches: 1→2→3, 1→4→5, 0→6→7
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 6).unwrap();
        g.add_edge(6, 7).unwrap();
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn depth_3_branch_not_lobster() {
        // Spine: 0-1, branch of depth 3: 0→2→3→4→5
        // After removing leaves (5): spine is 0-1, branch 0→2→3→4
        // After removing leaves again (1,4): 0→2→3
        // That's a path of 3 with 0 having spine-nbr 2. OK.
        // Actually let me check: tree has edges 0-1, 0-2, 2-3, 3-4, 4-5
        // Degrees: 0→2, 1→1, 2→2, 3→2, 4→2, 5→1
        // Leaf1: {1, 5}. Pruned degrees: 0→1(only 2), 2→2(0,3), 3→2(2,4), 4→1(only 3)
        // Leaf2: {0, 4} (pruned_deg ≤ 1). Doubly-pruned: {2, 3}
        // 2 has non-leaf1-non-leaf2 neighbors: check nbrs of 2: [0,3]. 0 is leaf2, 3 is not → 1
        // 3 has nbrs [2,4]. 4 is leaf2, 2 is not → 1
        // All ≤ 2 → lobster! Hmm, this IS a lobster.
        // For non-lobster, need depth ≥ 3 branches from a branching point.
        // A tree is NOT a lobster iff after removing leaves twice, the
        // remainder has a vertex of degree ≥ 3.
        // Example: center C with 3 branches of depth 3:
        // C→A→A2→A3, C→B→B2→B3, C→C'→C2→C3
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap(); // C-A
        g.add_edge(1, 2).unwrap(); // A-A2
        g.add_edge(2, 3).unwrap(); // A2-A3
        g.add_edge(0, 4).unwrap(); // C-B
        g.add_edge(4, 5).unwrap(); // B-B2
        g.add_edge(5, 6).unwrap(); // B2-B3
        g.add_edge(0, 7).unwrap(); // C-C'
        g.add_edge(7, 8).unwrap(); // C'-C2
        g.add_edge(8, 9).unwrap(); // C2-C3
        // Degrees: 0→3, 1→2, 2→2, 3→1, 4→2, 5→2, 6→1, 7→2, 8→2, 9→1
        // Leaf1: {3,6,9}. Pruned: 0→3(1,4,7), 1→2(0,2), 2→1(1), 4→2(0,5), 5→1(4), 7→2(0,8), 8→1(7)
        // Leaf2: {2,5,8} (pruned_deg==1 and not leaf1)
        // Doubly-pruned: {0,1,4,7}
        // 0: nbrs not leaf1/leaf2: {1,4,7} → 3 → NOT lobster!
        assert!(!is_lobster(&g).unwrap());
    }

    #[test]
    fn triangle_not_lobster() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_lobster(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_lobster(&g).unwrap());
    }

    #[test]
    fn spider_is_lobster() {
        // Spider with 3 legs of length 2: center→a→a2, center→b→b2, center→c→c2
        // This is NOT a caterpillar but IS a lobster
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(3, 6).unwrap();
        // Leaf1: {4,5,6}. Pruned: 0→3(1,2,3), 1→1(0), 2→1(0), 3→1(0)
        // Leaf2: {1,2,3}. Doubly-pruned: {0}
        // Single vertex → lobster!
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn binary_tree_depth3_is_lobster() {
        // Complete binary tree of depth 3: spine is 1-0-2 (path),
        // all vertices within distance 2 of spine → lobster.
        let mut g = Graph::with_vertices(15);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        g.add_edge(3, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        g.add_edge(4, 10).unwrap();
        g.add_edge(5, 11).unwrap();
        g.add_edge(5, 12).unwrap();
        g.add_edge(6, 13).unwrap();
        g.add_edge(6, 14).unwrap();
        assert!(is_lobster(&g).unwrap());
    }

    #[test]
    fn binary_tree_depth4_not_lobster() {
        // Complete binary tree of depth 4 (31 vertices).
        // After removing leaves twice, the remainder has branching → not lobster.
        let mut g = Graph::with_vertices(31);
        for i in 0..15u32 {
            g.add_edge(i, 2 * i + 1).unwrap();
            g.add_edge(i, 2 * i + 2).unwrap();
        }
        assert!(!is_lobster(&g).unwrap());
    }
}
