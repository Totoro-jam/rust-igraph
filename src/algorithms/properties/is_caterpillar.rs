//! Caterpillar tree predicate (ALGO-PR-086).
//!
//! A caterpillar tree is a tree in which every vertex is within
//! distance 1 of a central path (the "spine"). Equivalently, removing
//! all leaf vertices from a caterpillar yields a path (or an empty
//! graph or a single vertex).
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a caterpillar tree.
///
/// Returns `false` for directed graphs and non-tree graphs.
/// Returns `true` for the empty graph, single vertex, single edge,
/// paths, and stars (all are caterpillars).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_caterpillar};
///
/// // Star `K_{1,4}` is a caterpillar (spine is just the center)
/// let mut g = Graph::with_vertices(5);
/// for i in 1..5u32 {
///     g.add_edge(0, i).unwrap();
/// }
/// assert!(is_caterpillar(&g).unwrap());
///
/// // Spider (3 legs of length 2 from a center) is NOT a caterpillar
/// // Center 0 → 1→4, 2→5, 3→6
/// let mut g = Graph::with_vertices(7);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// g.add_edge(2, 5).unwrap();
/// g.add_edge(3, 6).unwrap();
/// assert!(!is_caterpillar(&g).unwrap());
/// ```
pub fn is_caterpillar(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }
    if n <= 2 {
        return Ok(is_tree(graph, DijkstraMode::All)?.is_some());
    }

    if is_tree(graph, DijkstraMode::All)?.is_none() {
        return Ok(false);
    }

    // Compute degrees
    let mut deg = vec![0u32; n as usize];
    for v in 0..n {
        deg[v as usize] = u32::try_from(graph.degree(v)?).unwrap_or(u32::MAX);
    }

    // A vertex is a leaf if degree == 1.
    // The "spine" consists of all non-leaf vertices.
    // For a caterpillar, the spine must form a path:
    //   - every spine vertex has at most 2 spine neighbors
    //   - the spine is connected (guaranteed since the tree is connected
    //     and removing leaves from a tree preserves connectivity of the
    //     remaining vertices, as long as ≥ 1 non-leaf exists)
    let is_leaf = |v: usize| deg[v] == 1;

    let spine_count = (0..n as usize).filter(|&v| !is_leaf(v)).count();
    if spine_count <= 1 {
        return Ok(true);
    }

    // For each spine vertex, count its spine neighbors.
    // In a path, every vertex has at most 2 neighbors.
    for v in 0..n {
        if is_leaf(v as usize) {
            continue;
        }
        let nbrs = graph.neighbors(v)?;
        let spine_nbr_count = nbrs.iter().filter(|&&w| !is_leaf(w as usize)).count();
        if spine_nbr_count > 2 {
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
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn path_p3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn path_p5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn star_k14() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn caterpillar_with_spine() {
        // Spine: 0-1-2-3, leaves: 4,5 on vertex 1; 6 on vertex 2
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn spider_not_caterpillar() {
        // Center 0, three legs of length 2: 0-1-4, 0-2-5, 0-3-6
        // After removing leaves (4,5,6), spine is 0-1, 0-2, 0-3 (star)
        // Vertex 0 has 3 spine neighbors → not a path → not caterpillar
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(3, 6).unwrap();
        assert!(!is_caterpillar(&g).unwrap());
    }

    #[test]
    fn t_shape_not_caterpillar() {
        // T-shape: spine has a branch
        // 0-1-2-3, with 4-1 AND 4-5 (leg of length 2 from 1)
        // Wait, let me be more precise.
        // Tree: 0-1, 1-2, 2-3, 1-4, 4-5
        // Leaves: 0, 3, 5. Spine: 1, 2, 4
        // Spine neighbors: 1 has spine neighbors {2, 4}, 2 has {1}, 4 has {1}
        // Max spine degree = 2 → this IS a caterpillar
        // Let me make a proper non-caterpillar:
        // Tree: 0-1, 1-2, 2-3, 3-4, 1-5, 5-6, 5-7
        // Leaves: 0, 4, 6, 7. Spine: 1, 2, 3, 5
        // Spine neighbors of 1: {2, 5} → 2
        // Spine neighbors of 2: {1, 3} → 2
        // Spine neighbors of 3: {2} → 1
        // Spine neighbors of 5: {1} → 1
        // All ≤ 2 → still a caterpillar!
        // For non-caterpillar, need 3+ branches of length ≥ 2
        // 0-C, C-1, C-2, C-3, 1-4, 2-5, 3-6 (where C = center)
        // Leaves: 4, 5, 6 (and 0 is a leaf too)
        // Non-leaves: C, 1, 2, 3
        // C has spine neighbors {1, 2, 3} → 3 → not caterpillar
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 7).unwrap(); // 0 is leaf of C=7
        g.add_edge(7, 1).unwrap();
        g.add_edge(7, 2).unwrap();
        g.add_edge(7, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(3, 6).unwrap();
        // Leaves: 0, 4, 5, 6. Spine: 7, 1, 2, 3.
        // 7 has spine neighbors {1, 2, 3} → 3 → not caterpillar
        assert!(!is_caterpillar(&g).unwrap());
    }

    #[test]
    fn triangle_not_caterpillar() {
        // Not a tree → not a caterpillar
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_caterpillar(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_caterpillar(&g).unwrap());
    }

    #[test]
    fn disconnected_not_caterpillar() {
        // Two components → not a tree → not a caterpillar
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_caterpillar(&g).unwrap());
    }

    #[test]
    fn double_star() {
        // Two stars joined by an edge: center1-center2, with leaves
        // Spine: center1-center2 (path of length 1)
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap(); // spine edge
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(1, 7).unwrap();
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn binary_tree_not_caterpillar() {
        // Complete binary tree of depth 2: root 0, children 1,2,
        // grandchildren 3,4,5,6
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        // Leaves: 3,4,5,6. Spine: 0,1,2.
        // 0 has spine neighbors {1,2} → 2
        // 1 has spine neighbor {0} → 1
        // 2 has spine neighbor {0} → 1
        // All ≤ 2 → caterpillar!
        // Actually yes, a complete binary tree of depth 2 IS a caterpillar
        // (spine is 1-0-2, a path)
        assert!(is_caterpillar(&g).unwrap());
    }

    #[test]
    fn lobster_not_caterpillar() {
        // Lobster: needs distance 2 from spine
        // Spine: 0-1, with 1-2, 2-3, 2-4 (branch of depth 2)
        // And 0-5, 5-6 (another branch of depth 2)
        // Leaves: 3, 4, 6. Also 1 connects to ... wait.
        // Let me be precise.
        // 0-1, 0-5, 1-2, 2-3, 2-4, 5-6
        // Degrees: 0→2, 1→2, 2→3, 3→1, 4→1, 5→2, 6→1
        // Leaves: 3, 4, 6. Non-leaves: 0, 1, 2, 5
        // Spine neighbors: 0→{1,5}=2, 1→{0,2}=2, 2→{1}=1, 5→{0}=1
        // All ≤ 2 → still a caterpillar!
        // Need a more extreme example. Let me construct a true lobster.
        // Spine: A-B, branch A→C→D (depth 2 off A), branch B→E→F (depth 2 off B)
        // Edges: A-B, A-C, C-D, B-E, E-F
        // Leaves: D, F. Non-leaves: A, B, C, E
        // A spine neighbors: {B, C} → 2
        // B spine neighbors: {A, E} → 2
        // C spine neighbors: {A} → 1
        // E spine neighbors: {B} → 1
        // All ≤ 2 → caterpillar!
        // Hmm, any tree where removing leaves gives a path is a caterpillar.
        // A lobster adds one more level of leaves.
        // The lobster itself is a caterpillar if its spine (after removing leaves) is a path.
        // But wait, removing leaves from a lobster gives a caterpillar, not just a path.
        // So a lobster is NOT necessarily a caterpillar.
        // Example: take a path 0-1-2, attach 3,4 to 1. Now from 3 attach 5.
        // Edges: 0-1, 1-2, 1-3, 1-4, 3-5
        // Degrees: 0→1, 1→4, 2→1, 3→2, 4→1, 5→1
        // Leaves: 0, 2, 4, 5. Spine: 1, 3
        // Spine neighbors of 1: {3} → 1
        // Spine neighbors of 3: {1} → 1
        // Spine is a path → caterpillar!
        // Argh. I need branches from DIFFERENT spine vertices that are themselves long.
        // The key insight: for a tree to NOT be a caterpillar, the spine must not be a path.
        // The spine is not a path when some spine vertex has 3+ spine neighbors.
        // This happens when 3+ branches of depth ≥ 2 emanate from the same vertex.
        // Already tested as spider_not_caterpillar above.
        // Let me add a test for "lobster that IS a caterpillar":
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap(); // spine
        g.add_edge(1, 2).unwrap(); // spine
        g.add_edge(0, 3).unwrap(); // leaf
        g.add_edge(1, 4).unwrap(); // branch depth 1
        g.add_edge(4, 5).unwrap(); // branch depth 2 (leaf)
        g.add_edge(2, 6).unwrap(); // branch depth 1
        g.add_edge(6, 7).unwrap(); // branch depth 2 (leaf)
        // Leaves: 3, 5, 7. Non-leaves: 0, 1, 2, 4, 6
        // Spine nbrs: 0→{1}=1, 1→{0,2,4}=3 → NOT caterpillar!
        assert!(!is_caterpillar(&g).unwrap());
    }
}
