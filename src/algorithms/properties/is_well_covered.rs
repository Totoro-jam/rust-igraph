//! Well-covered graph predicate (ALGO-PR-112).
//!
//! A graph is well-covered if every maximal independent set has the
//! same cardinality. Equivalently, every vertex belongs to a maximum
//! independent set. Complete graphs, edgeless graphs, and `C_5` are
//! well-covered. Stars with ≥ 3 leaves are not (center alone vs all leaves).
//!
//! Directed graphs are treated as undirected (edges lose orientation).

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is well-covered.
///
/// A graph is well-covered if all maximal independent sets have the
/// same cardinality. Uses backtracking enumeration of maximal
/// independent sets.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_well_covered};
///
/// // Complete graph `K_3`: only maximal IS is any single vertex → well-covered
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_well_covered(&g).unwrap());
///
/// // Star `S_3` is NOT well-covered ({0} size 1, {1,2,3} size 3)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert!(!is_well_covered(&g).unwrap());
/// ```
pub fn is_well_covered(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
            adj[w as usize][v as usize] = true;
        }
    }

    let mut first_size: Option<usize> = None;
    let mut current = Vec::new();
    let mut result = true;
    let candidates: Vec<usize> = (0..n_usize).collect();

    bron_kerbosch_is(
        &adj,
        &mut current,
        candidates,
        Vec::new(),
        &mut first_size,
        &mut result,
    );

    Ok(result)
}

/// Bron-Kerbosch enumeration of maximal independent sets.
fn bron_kerbosch_is(
    adj: &[Vec<bool>],
    current: &mut Vec<usize>,
    mut candidates: Vec<usize>,
    mut excluded: Vec<usize>,
    first_size: &mut Option<usize>,
    result: &mut bool,
) {
    if !*result {
        return;
    }

    if candidates.is_empty() && excluded.is_empty() {
        if !current.is_empty() {
            match *first_size {
                None => *first_size = Some(current.len()),
                Some(sz) => {
                    if current.len() != sz {
                        *result = false;
                    }
                }
            }
        }
        return;
    }

    while let Some(v) = candidates.pop() {
        if !*result {
            return;
        }
        current.push(v);

        let new_cand: Vec<usize> = candidates.iter().copied().filter(|&u| !adj[v][u]).collect();
        let new_excl: Vec<usize> = excluded.iter().copied().filter(|&u| !adj[v][u]).collect();

        bron_kerbosch_is(adj, current, new_cand, new_excl, first_size, result);
        current.pop();
        excluded.push(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn triangle() {
        // K_3: maximal IS = any single vertex → size 1 → well-covered
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn edgeless_3() {
        // 3 isolated vertices: only maximal IS is {0,1,2} → well-covered
        let g = Graph::with_vertices(3);
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn c4_not_well_covered() {
        // C_4: maximal IS {0,2} size 2, {1,3} size 2 → well-covered
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn c5_well_covered() {
        // C_5 is well-covered: all maximal IS have size 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn p4_not_well_covered() {
        // P_4: 0-1-2-3. Maximal IS: {0,2}, {0,3}, {1,3} all size 2.
        // But also {0, 2} is maximal. Actually let me re-check:
        // {0,2} → can add 3? adj[2][3]=true → no. Maximal, size 2.
        // {0,3} → can add? 1 adj to 0, 2 adj to 3 → maximal, size 2.
        // {1,3} → can add? 0 adj to 1, 2 adj to 1 and 3 → maximal, size 2.
        // Hmm, all size 2. Wait, is P_4 well-covered?
        // Actually P_4 IS well-covered. Let me use P_3 ∪ K_1 instead.
        // Actually re-checking: P_4 = 0-1-2-3.
        // Independent sets: {0,2}, {0,3}, {1,3} are maximal of size 2.
        // Can we get size 1? {1} → can add 3 (not adj to 1) → not maximal.
        // So all maximal IS have size 2. P_4 IS well-covered.
        // Let me find a graph that isn't. P_3 with extra vertex:
        // 0-1-2, 3 isolated. Maximal IS: {0,2,3} size 3, but also
        // {1,3} size 2 → not well-covered.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // vertex 3 is isolated
        // Maximal IS: {0,2,3} size 3, {1,3} size 2 → not well-covered
        assert!(!is_well_covered(&g).unwrap());
    }

    #[test]
    fn k4() {
        // K_4: maximal IS = any single vertex → all size 1 → well-covered
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn star_3_not_well_covered() {
        // Star S_3: center 0, leaves 1,2,3.
        // Maximal IS: {1,2,3} size 3, {0} size 1 → not well-covered
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_well_covered(&g).unwrap());
    }

    #[test]
    fn c7_well_covered() {
        // C_7 is well-covered: all maximal IS have size 3
        let mut g = Graph::with_vertices(7);
        for i in 0..7u32 {
            g.add_edge(i, (i + 1) % 7).unwrap();
        }
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn k33_not_well_covered() {
        // K_{3,3}: maximal IS are the two sides {0,1,2} and {3,4,5}
        // both size 3 → well-covered
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_well_covered(&g).unwrap());
    }

    #[test]
    fn diamond_not_well_covered() {
        // Diamond: 0-1, 0-2, 0-3, 1-2, 1-3 (K_4 minus edge 2-3)
        // Maximal IS: {2,3} size 2. Any single vertex: {0} → can add
        // 2 or 3? adj[0][2]=true, adj[0][3]=true → {0} maximal? No,
        // wait: not adj to anything else... Let me re-check.
        // Actually {0} → is 2 independent of 0? adj[0][2]=true → no.
        // Is 3 independent? adj[0][3]=true → no. Is 1 independent? adj[0][1]=true → no.
        // So {0} is maximal of size 1. And {2,3} is maximal of size 2.
        // → NOT well-covered.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_well_covered(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        // Directed triangle: same as undirected → well-covered
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_well_covered(&g).unwrap());
    }
}
