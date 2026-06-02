//! Chain graph predicate (ALGO-PR-114).
//!
//! A bipartite graph is a chain graph if the neighborhoods of the
//! vertices in each part are totally ordered by inclusion.
//! Equivalently, a bipartite graph with no induced `2K_2`
//! (two disjoint edges as induced subgraph).
//!
//! Non-bipartite and directed graphs return `false`.

use crate::algorithms::properties::is_bipartite::is_bipartite;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a chain graph.
///
/// A bipartite graph is a chain graph if the neighborhoods of the
/// vertices in each part are totally ordered by inclusion.
/// Returns `false` for non-bipartite or directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_chain_graph};
///
/// // Complete bipartite `K_{2,3}` is a chain graph
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// assert!(is_chain_graph(&g).unwrap());
///
/// // `C_6` is bipartite but NOT a chain graph (has induced `2K_2`)
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 0).unwrap();
/// assert!(!is_chain_graph(&g).unwrap());
/// ```
pub fn is_chain_graph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 2 {
        return is_bipartite(graph).map(|r| r.is_bipartite);
    }

    let bip = is_bipartite(graph)?;
    if !bip.is_bipartite {
        return Ok(false);
    }

    let n_usize = n as usize;
    let types = &bip.types;

    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // Partition vertices by type
    let part_a: Vec<usize> = (0..n_usize).filter(|&v| !types[v]).collect();
    let part_b: Vec<usize> = (0..n_usize).filter(|&v| types[v]).collect();

    // Check neighborhoods in part A are totally ordered by inclusion
    // w.r.t. their neighbors in part B
    if !neighborhoods_nested(&adj, &part_a, &part_b) {
        return Ok(false);
    }

    // Check neighborhoods in part B are totally ordered by inclusion
    // w.r.t. their neighbors in part A
    if !neighborhoods_nested(&adj, &part_b, &part_a) {
        return Ok(false);
    }

    Ok(true)
}

/// Check that the neighborhoods of vertices in `part` (restricted to
/// `other_part`) are totally ordered by inclusion.
fn neighborhoods_nested(adj: &[Vec<bool>], part: &[usize], other_part: &[usize]) -> bool {
    if part.len() <= 1 {
        return true;
    }

    // Collect neighborhood sets (as sorted vecs of other-side indices)
    let mut nbr_sets: Vec<Vec<usize>> = part
        .iter()
        .map(|&v| other_part.iter().copied().filter(|&u| adj[v][u]).collect())
        .collect();

    // Sort by neighborhood size
    nbr_sets.sort_by_key(Vec::len);

    // Check each consecutive pair: smaller must be subset of larger
    for window in nbr_sets.windows(2) {
        let smaller = &window[0];
        let larger = &window[1];
        if !is_subset(smaller, larger) {
            return false;
        }
    }

    true
}

/// Check if `a` ⊆ `b` (both sorted).
fn is_subset(a: &[usize], b: &[usize]) -> bool {
    if a.len() > b.len() {
        return false;
    }
    let mut bi = 0;
    for &x in a {
        while bi < b.len() && b[bi] < x {
            bi += 1;
        }
        if bi >= b.len() || b[bi] != x {
            return false;
        }
        bi += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn path_p3() {
        // 0-1-2: parts {0,2} and {1}. N(0)={1}, N(2)={1} → equal → nested.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn path_p4() {
        // 0-1-2-3: parts {0,2} and {1,3}. N(0)={1}, N(2)={1,3} → {1}⊂{1,3} ✓
        // N(1)={0,2}, N(3)={2} → {2}⊂{0,2} ✓ → chain graph
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn complete_bipartite_k23() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn c4_not_chain() {
        // C_4: parts {0,2}, {1,3}. N(0)={1,3}, N(2)={1,3} → equal → nested ✓
        // N(1)={0,2}, N(3)={0,2} → equal → nested ✓
        // Actually C_4 IS a chain graph (all nbrs same size and equal).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn c6_not_chain() {
        // C_6 has induced 2K_2 → NOT a chain graph
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_chain_graph(&g).unwrap());
    }

    #[test]
    fn star() {
        // Star: parts {0} and {1,2,3}. N(0)={1,2,3} → single vertex → nested.
        // N(1)=N(2)=N(3)={0} → all equal → nested. Chain graph.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn two_disjoint_edges() {
        // 2K_2: 0-1, 2-3. Parts {0,2}, {1,3}. N(0)={1}, N(2)={3}.
        // Neither {1}⊂{3} nor {3}⊂{1} → NOT nested → NOT chain.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_chain_graph(&g).unwrap());
    }

    #[test]
    fn triangle_not_bipartite() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_chain_graph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_chain_graph(&g).unwrap());
    }

    #[test]
    fn isolated_vertices() {
        // 3 isolated vertices: bipartite, each has empty neighborhood → nested.
        let g = Graph::with_vertices(3);
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn k33_not_chain() {
        // K_{3,3}: all vertices in each part have the same neighborhood → nested ✓
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_chain_graph(&g).unwrap());
    }

    #[test]
    fn bipartite_with_induced_2k2() {
        // 0-2, 0-3, 1-3 (missing 1-2). Parts {0,1}, {2,3}.
        // N(0)={2,3}, N(1)={3}. {3}⊂{2,3} ✓
        // N(2)={0}, N(3)={0,1}. {0}⊂{0,1} ✓
        // Actually this IS a chain graph! No induced 2K_2.
        // Let me try: 0-2, 1-3 only.
        // Parts {0,1}, {2,3}. N(0)={2}, N(1)={3}. Neither subset → NOT chain.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_chain_graph(&g).unwrap());
    }

    #[test]
    fn nested_neighborhoods() {
        // 0-3, 0-4, 1-3, 1-4, 1-5, 2-3, 2-4, 2-5
        // Parts {0,1,2}, {3,4,5}
        // N(0)={3,4}, N(1)={3,4,5}, N(2)={3,4,5}
        // Sorted: {3,4} ⊂ {3,4,5} = {3,4,5} ✓
        // N(3)={0,1,2}, N(4)={0,1,2}, N(5)={1,2}
        // Sorted: {1,2} ⊂ {0,1,2} = {0,1,2} ✓
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(is_chain_graph(&g).unwrap());
    }
}
