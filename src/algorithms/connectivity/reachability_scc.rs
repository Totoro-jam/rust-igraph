//! SCC-based reachability (ALGO-CC-023).
//!
//! Counterpart of `igraph_reachability()` from
//! `references/igraph/src/connectivity/reachability.c:72-149`.
//!
//! Computes which vertices are reachable from each vertex via
//! SCC-condensation + bitset propagation. For directed graphs the
//! condensation DAG is traversed in reverse topological order; for
//! undirected graphs (or `All` mode) each weak component trivially
//! reaches itself.
//!
//! Time: O(|C|·|V|/w + |V| + |E|) where |C| is the SCC count and w is
//! the machine word size.

use crate::algorithms::connectivity::components::connected_components;
use crate::algorithms::connectivity::strong::strongly_connected_components;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for reachability in directed graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReachabilityMode {
    /// Follow edges forward (out-neighbours).
    Out,
    /// Follow edges backward (in-neighbours).
    In,
    /// Ignore direction (treat as undirected).
    All,
}

/// Result of a reachability analysis.
#[derive(Debug, Clone)]
pub struct ReachabilityResult {
    /// `membership[v]` — component id for vertex `v`.
    pub membership: Vec<u32>,
    /// `csize[c]` — number of vertices in component `c`.
    pub csize: Vec<u32>,
    /// Number of components.
    pub num_components: u32,
    /// `reach[c]` — bitvector of length `vcount`; bit `v` is set if
    /// vertex `v` is reachable from any vertex in component `c`.
    pub reach: Vec<Vec<u64>>,
}

impl ReachabilityResult {
    /// Returns `true` if vertex `target` is reachable from vertex `source`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, reachability, ReachabilityMode};
    ///
    /// let mut g = Graph::new(3, true).unwrap();
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// let r = reachability(&g, ReachabilityMode::Out).unwrap();
    /// assert!(r.is_reachable(0, 2));
    /// assert!(!r.is_reachable(2, 0));
    /// ```
    pub fn is_reachable(&self, source: VertexId, target: VertexId) -> bool {
        let comp = self.membership[source as usize] as usize;
        let word = target as usize / 64;
        let bit = target as usize % 64;
        if word >= self.reach[comp].len() {
            return false;
        }
        (self.reach[comp][word] >> bit) & 1 == 1
    }

    /// Count how many vertices are reachable from `v` (including `v`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_igraph::{Graph, reachability, ReachabilityMode};
    ///
    /// let mut g = Graph::new(3, true).unwrap();
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    /// let r = reachability(&g, ReachabilityMode::Out).unwrap();
    /// assert_eq!(r.count_from(0), 3); // reaches 0, 1, 2
    /// assert_eq!(r.count_from(2), 1); // reaches only itself
    /// ```
    pub fn count_from(&self, v: VertexId) -> u32 {
        let comp = self.membership[v as usize] as usize;
        let mut count = 0u32;
        for &word in &self.reach[comp] {
            count = count.wrapping_add(word.count_ones());
        }
        count
    }
}

fn bitvec_words(n: u32) -> usize {
    (n as usize).div_ceil(64)
}

fn bitvec_set(bv: &mut [u64], bit: u32) {
    let w = bit as usize / 64;
    let b = bit as usize % 64;
    bv[w] |= 1u64 << b;
}

fn bitvec_or(dst: &mut [u64], src: &[u64]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d |= *s;
    }
}

/// Compute per-component reachability bitsets.
///
/// Counterpart of `igraph_reachability()`. For directed graphs with
/// `Out` mode, vertex `v` reaches vertex `u` if there is a directed
/// path from `v` to `u`. With `In` mode, edges are traversed in
/// reverse. With `All` mode (or undirected graphs), direction is
/// ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reachability, ReachabilityMode};
///
/// // Directed chain 0→1→2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let r = reachability(&g, ReachabilityMode::Out).unwrap();
/// assert!(r.is_reachable(0, 2));
/// assert!(!r.is_reachable(2, 0));
/// assert_eq!(r.count_from(0), 3);
/// assert_eq!(r.count_from(2), 1);
/// ```
pub fn reachability(graph: &Graph, mode: ReachabilityMode) -> IgraphResult<ReachabilityResult> {
    let n = graph.vcount();
    let words = bitvec_words(n);

    let effective_mode = if graph.is_directed() {
        mode
    } else {
        ReachabilityMode::All
    };

    // Step 1: compute components
    let (membership, num_components) = if effective_mode == ReachabilityMode::All {
        let cc = connected_components(graph)?;
        (cc.membership, cc.count)
    } else {
        let scc = strongly_connected_components(graph)?;
        (scc.membership, scc.count)
    };

    // Step 2: compute component sizes
    let mut csize = vec![0u32; num_components as usize];
    for &m in &membership {
        csize[m as usize] = csize[m as usize]
            .checked_add(1)
            .ok_or(IgraphError::Internal("component size overflow"))?;
    }

    // Step 3: initialise reach bitsets — each component contains its own vertices
    let mut reach = vec![vec![0u64; words]; num_components as usize];
    for v in 0..n {
        bitvec_set(&mut reach[membership[v as usize] as usize], v);
    }

    // For undirected / All mode, components are self-contained — done
    if effective_mode == ReachabilityMode::All {
        return Ok(ReachabilityResult {
            membership,
            csize,
            num_components,
            reach,
        });
    }

    // Step 4: build condensation DAG adjacency list
    // For each SCC, collect which other SCCs it can reach directly
    let mut dag: Vec<Vec<u32>> = vec![Vec::new(); num_components as usize];

    for v in 0..n {
        let v_comp = membership[v as usize];
        let neighbors = match effective_mode {
            ReachabilityMode::Out => graph.out_neighbors_vec(v)?,
            ReachabilityMode::In => graph.in_neighbors_vec(v)?,
            ReachabilityMode::All => unreachable!(),
        };
        for w in neighbors {
            let w_comp = membership[w as usize];
            if v_comp != w_comp {
                dag[v_comp as usize].push(w_comp);
            }
        }
    }

    // Step 5: propagate in reverse topological order
    // SCCs from strongly_connected_components are indexed in topological order
    // (Kosaraju guarantees this). For Out mode, iterate from last to first;
    // for In mode, iterate from first to last.
    let nc = num_components as usize;
    for i in 0..nc {
        let comp = if effective_mode == ReachabilityMode::In {
            i
        } else {
            nc - 1 - i
        };

        // Deduplicate DAG neighbors for this component
        dag[comp].sort_unstable();
        dag[comp].dedup();

        // Collect neighbor bitsets to OR in
        // We need to avoid borrowing reach mutably and immutably at the same time
        let neighbor_comps: Vec<u32> = dag[comp].clone();
        for &nc_id in &neighbor_comps {
            // Copy the neighbor's bitset, then OR into ours
            let src = reach[nc_id as usize].clone();
            bitvec_or(&mut reach[comp], &src);
        }
    }

    Ok(ReachabilityResult {
        membership,
        csize,
        num_components,
        reach,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.num_components, 0);
        assert!(r.reach.is_empty());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(5);
        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.num_components, 5);
        for v in 0..5u32 {
            assert_eq!(r.count_from(v), 1);
            assert!(r.is_reachable(v, v));
        }
        assert!(!r.is_reachable(0, 1));
    }

    #[test]
    fn undirected_path() {
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = reachability(&g, ReachabilityMode::All).unwrap();
        assert_eq!(r.num_components, 1);
        for u in 0..4u32 {
            for v in 0..4u32 {
                assert!(r.is_reachable(u, v));
            }
        }
    }

    #[test]
    fn undirected_two_components() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();

        let r = reachability(&g, ReachabilityMode::All).unwrap();
        assert_eq!(r.num_components, 2);
        assert!(r.is_reachable(0, 2));
        assert!(r.is_reachable(2, 0));
        assert!(r.is_reachable(3, 4));
        assert!(!r.is_reachable(0, 3));
        assert!(!r.is_reachable(3, 0));
    }

    #[test]
    fn directed_chain_out() {
        // 0→1→2→3
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.count_from(0), 4);
        assert_eq!(r.count_from(1), 3);
        assert_eq!(r.count_from(2), 2);
        assert_eq!(r.count_from(3), 1);

        assert!(r.is_reachable(0, 3));
        assert!(!r.is_reachable(3, 0));
    }

    #[test]
    fn directed_chain_in() {
        // 0→1→2→3 with In mode = reverse edges
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = reachability(&g, ReachabilityMode::In).unwrap();
        // In mode reverses: 3 can reach everything backwards
        assert_eq!(r.count_from(3), 4);
        assert_eq!(r.count_from(2), 3);
        assert_eq!(r.count_from(1), 2);
        assert_eq!(r.count_from(0), 1);

        assert!(r.is_reachable(3, 0));
        assert!(!r.is_reachable(0, 3));
    }

    #[test]
    fn directed_cycle() {
        // 0→1→2→0: all vertices in one SCC, all reach all
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.num_components, 1);
        for u in 0..3u32 {
            assert_eq!(r.count_from(u), 3);
        }
    }

    #[test]
    fn directed_diamond() {
        // 0→1, 0→2, 1→3, 2→3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.count_from(0), 4);
        assert_eq!(r.count_from(1), 2); // 1, 3
        assert_eq!(r.count_from(2), 2); // 2, 3
        assert_eq!(r.count_from(3), 1); // 3

        assert!(r.is_reachable(0, 3));
        assert!(!r.is_reachable(3, 0));
    }

    #[test]
    fn directed_two_sccs_connected() {
        // SCC1: 0→1→0, SCC2: 2→3→2, bridge: 1→2
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 2).unwrap();
        g.add_edge(1, 2).unwrap(); // bridge SCC1→SCC2

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        // 0 and 1 reach everything
        assert_eq!(r.count_from(0), 4);
        assert_eq!(r.count_from(1), 4);
        // 2 and 3 only reach SCC2
        assert_eq!(r.count_from(2), 2);
        assert_eq!(r.count_from(3), 2);
    }

    #[test]
    fn self_loop_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.count_from(0), 2);
        assert_eq!(r.count_from(1), 1);
    }

    #[test]
    fn mode_all_on_directed() {
        // Directed chain 0→1→2, but All mode ignores direction
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let r = reachability(&g, ReachabilityMode::All).unwrap();
        assert_eq!(r.num_components, 1);
        for v in 0..3u32 {
            assert_eq!(r.count_from(v), 3);
        }
    }

    #[test]
    fn large_bitvec_more_than_64_vertices() {
        // 70 vertices in a chain: 0→1→2→...→69
        let mut g = Graph::new(70, true).unwrap();
        for i in 0..69u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        assert_eq!(r.count_from(0), 70);
        assert_eq!(r.count_from(69), 1);
        assert!(r.is_reachable(0, 69));
        assert!(!r.is_reachable(69, 0));
    }

    #[test]
    fn csize_correct() {
        // Two weak components: {0,1,2} and {3,4}
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();

        let r = reachability(&g, ReachabilityMode::All).unwrap();
        let mut sizes: Vec<u32> = r.csize.clone();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![2, 3]);
    }

    #[test]
    fn agrees_with_count_reachable() {
        use crate::algorithms::connectivity::reachability::count_reachable;

        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(5, 3).unwrap();

        let r = reachability(&g, ReachabilityMode::Out).unwrap();
        let cr = count_reachable(&g).unwrap();

        for v in 0..6u32 {
            assert_eq!(r.count_from(v), cr[v as usize], "mismatch at vertex {v}");
        }
    }
}
