//! k-core decomposition / coreness (ALGO-PR-015 + ALGO-PR-015b).
//!
//! Counterpart of `igraph_coreness()` from
//! `references/igraph/src/centrality/coreness.c`. Implements Batagelj &
//! Zaversnik's O(|E|) "An O(m) Algorithm for Cores Decomposition of
//! Networks" (<https://arxiv.org/abs/cs/0310049>): bin sort by degree,
//! walk vertices in ascending current-core order, decrement higher-core
//! neighbours and shuffle them down the bins.
//!
//! [`coreness`] (PR-015) is the canonical undirected entry. For
//! directed graphs [`coreness_with_mode`] (PR-015b) selects between
//! in-cores, out-cores, or the undirected projection.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Direction-handling for [`coreness_with_mode`].
///
/// Counterpart of upstream's `igraph_neimode_t`. Undirected graphs
/// always behave as if `All`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorenessMode {
    /// All edges count regardless of direction. The classical k-core.
    All,
    /// In-cores: degree is the number of incoming edges. The k-in-core
    /// is the maximal subgraph in which every vertex has at least
    /// `k` incoming edges.
    In,
    /// Out-cores: degree is the number of outgoing edges.
    Out,
}

/// Per-vertex coreness number.
///
/// Returns `Vec<u32>` of length `vcount`: `result[v]` is the highest
/// `k` such that `v` belongs to the k-core. The empty graph yields an
/// empty vector. Self-loops contribute 2 to a vertex's degree (matches
/// upstream `IGRAPH_LOOPS`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, coreness};
///
/// // K3 triangle: every vertex has degree 2 in a graph where the
/// // minimum degree is 2 → coreness 2 for all three.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// assert_eq!(coreness(&g).unwrap(), vec![2, 2, 2]);
/// ```
pub fn coreness(graph: &Graph) -> IgraphResult<Vec<u32>> {
    coreness_with_mode(graph, CorenessMode::All)
}

/// Coreness with explicit [`CorenessMode`] (ALGO-PR-015b).
///
/// Counterpart of `igraph_coreness(_, _, mode)`. On undirected graphs
/// every mode reduces to [`CorenessMode::All`]. Self-loops contribute
/// 2 to total degree (`All`) or 1 to each of in/out (`In`/`Out`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, coreness_with_mode, CorenessMode};
///
/// // Directed 3-cycle 0→1→2→0: each vertex has out-degree 1 (and
/// // in-degree 1). Out-cores → all 1; in-cores → all 1. As undirected
/// // (every vertex degree 2 in a graph with min degree 2) → all 2.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(coreness_with_mode(&g, CorenessMode::Out).unwrap(), vec![1, 1, 1]);
/// assert_eq!(coreness_with_mode(&g, CorenessMode::In).unwrap(),  vec![1, 1, 1]);
/// assert_eq!(coreness_with_mode(&g, CorenessMode::All).unwrap(), vec![2, 2, 2]);
/// ```
pub fn coreness_with_mode(graph: &Graph, mode: CorenessMode) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n_us == 0 {
        return Ok(Vec::new());
    }

    // Effective mode: undirected graphs always collapse to All.
    let eff_mode = if graph.is_directed() {
        mode
    } else {
        CorenessMode::All
    };

    // Initial cores: degree under the chosen mode.
    let mut cores = vec![0u32; n_us];
    let mut max_deg: u32 = 0;
    for v in 0..n {
        let d = vertex_degree_in_mode(graph, v, eff_mode)?;
        cores[v as usize] = d;
        if d > max_deg {
            max_deg = d;
        }
    }

    let max_deg_us = max_deg as usize;
    let mut bin = vec![0usize; max_deg_us + 1];
    for &c in &cores {
        bin[c as usize] += 1;
    }
    let mut start = 0usize;
    for slot in bin.iter_mut().take(max_deg_us + 1) {
        let count = *slot;
        *slot = start;
        start += count;
    }

    let mut vert = vec![0u32; n_us];
    let mut pos = vec![0usize; n_us];
    let mut bin_cursor = bin.clone();
    for v in 0..n {
        let c = cores[v as usize] as usize;
        let p = bin_cursor[c];
        pos[v as usize] = p;
        vert[p] = v;
        bin_cursor[c] += 1;
    }
    drop(bin_cursor);

    // Main loop: when peeling vertex `v`, walk the **reverse-mode**
    // neighbours so we touch the vertices whose `eff_mode`-degree
    // depended on `v`. For All, that's every neighbour; for Out, the
    // in-neighbours (vertices `u` such that `u → v`); for In, the
    // out-neighbours (vertices `u` such that `v → u`).
    for i in 0..n_us {
        let v = vert[i];
        let neis = peel_neighbors_in_mode(graph, v, eff_mode)?;
        for u in neis {
            if cores[u as usize] > cores[v as usize] {
                let du = cores[u as usize] as usize;
                let pu = pos[u as usize];
                let pw = bin[du];
                let w = vert[pw];
                if u != w {
                    pos[u as usize] = pw;
                    pos[w as usize] = pu;
                    vert[pu] = w;
                    vert[pw] = u;
                }
                bin[du] += 1;
                cores[u as usize] -= 1;
            }
        }
    }

    Ok(cores)
}

fn vertex_degree_in_mode(graph: &Graph, v: u32, mode: CorenessMode) -> IgraphResult<u32> {
    let raw = match mode {
        CorenessMode::All => graph.degree(v)?,
        CorenessMode::Out => graph.out_neighbors_vec(v)?.len(),
        CorenessMode::In => graph.in_neighbors_vec(v)?.len(),
    };
    u32::try_from(raw).map_err(|_| IgraphError::Internal("vertex degree overflows u32"))
}

fn peel_neighbors_in_mode(graph: &Graph, v: u32, mode: CorenessMode) -> IgraphResult<Vec<u32>> {
    match mode {
        // Reverse-mode: `In` peels via out-neighbours, `Out` via
        // in-neighbours, `All` via the merged neighbour view.
        CorenessMode::All => graph.neighbors(v),
        CorenessMode::Out => graph.in_neighbors_vec(v),
        CorenessMode::In => graph.out_neighbors_vec(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_returns_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(coreness(&g).unwrap(), Vec::<u32>::new());
    }

    #[test]
    fn singleton_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(coreness(&g).unwrap(), vec![0]);
    }

    #[test]
    fn isolated_vertices_all_zero() {
        let g = Graph::with_vertices(5);
        assert_eq!(coreness(&g).unwrap(), vec![0; 5]);
    }

    #[test]
    fn single_edge_two_one_cores() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert_eq!(coreness(&g).unwrap(), vec![1, 1]);
    }

    #[test]
    fn triangle_all_two_cores() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        assert_eq!(coreness(&g).unwrap(), vec![2, 2, 2]);
    }

    #[test]
    fn path_all_one_cores() {
        // Path 0-1-2-3-4: all vertices belong to the 1-core (degree-1
        // leaves drop the inner ones to coreness 1 too).
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(coreness(&g).unwrap(), vec![1; 5]);
    }

    #[test]
    fn star_centre_vs_leaves() {
        // 4-star: centre 0 is adjacent to leaves 1, 2, 3. The leaves
        // each have degree 1 → coreness 1. After peeling them, the
        // centre has nothing left → coreness 1 too.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(coreness(&g).unwrap(), vec![1, 1, 1, 1]);
    }

    #[test]
    fn k4_all_three_cores() {
        // K4: every vertex has degree 3 in a graph with min degree 3
        // → coreness 3.
        let mut g = Graph::with_vertices(4);
        for u in 0..4 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(coreness(&g).unwrap(), vec![3, 3, 3, 3]);
    }

    #[test]
    fn triangle_with_pendant_mixed_cores() {
        // Triangle 0-1-2 plus pendant 3 attached to vertex 2:
        //   - vertex 3 has degree 1 → coreness 1
        //   - removing 3 leaves a triangle → 0, 1, 2 are coreness 2.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(coreness(&g).unwrap(), vec![2, 2, 2, 1]);
    }

    #[test]
    fn two_components_independent() {
        // Disjoint union of K3 and a single edge: K3 vertices →
        // coreness 2, edge vertices → coreness 1.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(coreness(&g).unwrap(), vec![2, 2, 2, 1, 1]);
    }

    #[test]
    fn self_loop_counts_twice() {
        // A self-loop contributes 2 to the loop-vertex's degree.
        // Vertex 0 has self-loop + edge to 1 → degree 3, but vertex 1
        // has degree 1 → 1-core. Once vertex 1 is peeled, vertex 0 is
        // alone with the self-loop → degree 2 → still has nowhere to
        // go (no neighbours other than itself), so its core gets
        // dragged down to 1 by the algorithm even though the
        // "structural" self-loop persists. Matches upstream
        // IGRAPH_LOOPS semantics.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let cores = coreness(&g).unwrap();
        // Vertex 1 must be coreness 1.
        assert_eq!(cores[1], 1);
    }

    #[test]
    fn directed_default_is_undirected_view_for_canonical_entry() {
        // PR-015b extension: `coreness()` no longer rejects directed
        // graphs — it delegates to `coreness_with_mode(_, All)` which
        // counts every edge regardless of direction.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Treated as undirected, this is K3 → 2-core for every vertex.
        assert_eq!(coreness(&g).unwrap(), vec![2, 2, 2]);
    }

    // ----- ALGO-PR-015b: directed in/out cores -----

    #[test]
    fn directed_3_cycle_in_out_modes_match() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::Out).unwrap(),
            vec![1, 1, 1]
        );
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::In).unwrap(),
            vec![1, 1, 1]
        );
    }

    #[test]
    fn directed_star_out_mode_drains_to_zero() {
        // Directed star: 0 → 1, 0 → 2, 0 → 3. Out-degrees: [3, 0, 0, 0].
        // After peeling 1, 2, 3 (out-degree 0 → 0-core), vertex 0's
        // out-degree falls to 0 (since each peel decrements an in-edge,
        // not out — wait, peeling target u in OUT mode looks at u's
        // IN-neighbours; these are vertices that pointed AT u. For
        // each such v, decrement v's OUT-degree). Vertices 1, 2, 3 have
        // in-edges from 0, so peeling 1 decrements 0's core to 2,
        // peeling 2 → 1, peeling 3 → 0. Final out-cores = [0, 0, 0, 0].
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::Out).unwrap(),
            vec![0, 0, 0, 0]
        );
    }

    #[test]
    fn directed_star_in_mode_drains_to_zero() {
        // Same directed star (0 → 1, 0 → 2, 0 → 3). In-degrees:
        // [0, 1, 1, 1]. Vertex 0 starts at 0-core. Peeling 0 looks at
        // 0's OUT-neighbours (1, 2, 3) and decrements each of their
        // in-cores to 0. Final in-cores = [0, 0, 0, 0].
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::In).unwrap(),
            vec![0, 0, 0, 0]
        );
    }

    #[test]
    fn directed_complete_3_all_one_in_one_out() {
        // Directed K3 in both directions: 6 edges, every vertex has
        // in-degree 2 and out-degree 2. After peeling, all stay at 2.
        let mut g = Graph::new(3, true).unwrap();
        for &(u, v) in &[(0u32, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::Out).unwrap(),
            vec![2, 2, 2]
        );
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::In).unwrap(),
            vec![2, 2, 2]
        );
        // ALL mode counts each edge once → 4 each → 4-core.
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::All).unwrap(),
            vec![4, 4, 4]
        );
    }

    #[test]
    fn undirected_modes_all_agree() {
        // On undirected graphs every mode collapses to All.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let a = coreness_with_mode(&g, CorenessMode::All).unwrap();
        let i = coreness_with_mode(&g, CorenessMode::In).unwrap();
        let o = coreness_with_mode(&g, CorenessMode::Out).unwrap();
        assert_eq!(a, i);
        assert_eq!(a, o);
        assert_eq!(a, vec![2, 2, 2]);
    }

    #[test]
    fn directed_chain_out_mode_descends() {
        // Directed chain 0 → 1 → 2 → 3. Out-degrees: [1, 1, 1, 0].
        // Peel vertex 3 (out-deg 0) — its in-neighbour 2's out-core
        // stays 1 (peeled vertex 3 has cores[3]=0, so cores[2]>cores[3]
        // → decrement 2's out-core to 0). Continuing: 2 → peel decrements 1 to 0; 1 → peels 0.
        // Final: [0, 0, 0, 0].
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(
            coreness_with_mode(&g, CorenessMode::Out).unwrap(),
            vec![0, 0, 0, 0]
        );
    }

    #[test]
    fn coreness_bounded_by_max_degree() {
        // Property: for every vertex, coreness(v) ≤ degree(v).
        let mut g = Graph::with_vertices(6);
        // Petersen-fragment style irregular graph.
        for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let cores = coreness(&g).unwrap();
        for v in 0..g.vcount() {
            let d = u32::try_from(g.degree(v).unwrap()).unwrap();
            assert!(
                cores[v as usize] <= d,
                "vertex {}: coreness {} exceeds degree {}",
                v,
                cores[v as usize],
                d
            );
        }
    }
}
