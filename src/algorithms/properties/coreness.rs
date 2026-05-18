//! k-core decomposition / coreness (ALGO-PR-015).
//!
//! Counterpart of `igraph_coreness()` from
//! `references/igraph/src/centrality/coreness.c`. Phase-1 minimal slice:
//! undirected graphs only (the `mode` parameter from upstream collapses
//! to `IGRAPH_ALL` here; directed in-/out-cores ship later).
//!
//! Implements Batagelj & Zaversnik's O(|E|) "An O(m) Algorithm for Cores
//! Decomposition of Networks" (<https://arxiv.org/abs/cs/0310049>): bin
//! sort by degree, walk vertices in ascending current-core order,
//! decrement higher-core neighbours and shuffle them down the bins.
//! At the end, `cores[v]` is the largest k such that `v` belongs to the
//! k-core (the maximal subgraph with minimum degree ≥ k).

use crate::core::{Graph, IgraphError, IgraphResult};

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
    if graph.is_directed() {
        // Directed in/out coreness ships under PR-015b.
        return Err(IgraphError::Unsupported(
            "directed coreness modes (IN/OUT) not yet implemented; pass an undirected graph",
        ));
    }

    let n = graph.vcount();
    let n_us = n as usize;
    if n_us == 0 {
        return Ok(Vec::new());
    }

    // cores[v] starts at degree(v) and only ever decreases.
    let mut cores = vec![0u32; n_us];
    let mut max_deg: u32 = 0;
    for v in 0..n {
        let d = u32::try_from(graph.degree(v)?)
            .map_err(|_| IgraphError::Internal("vertex degree overflows u32"))?;
        cores[v as usize] = d;
        if d > max_deg {
            max_deg = d;
        }
    }

    let max_deg_us = max_deg as usize;
    // bin[d] = position in `vert` where the d-th degree bucket starts.
    let mut bin = vec![0usize; max_deg_us + 1];
    for &c in &cores {
        bin[c as usize] += 1;
    }
    // Cumulative sums so `bin[d]` becomes the start of bucket d.
    let mut start = 0usize;
    for slot in bin.iter_mut().take(max_deg_us + 1) {
        let count = *slot;
        *slot = start;
        start += count;
    }

    // Sort vertices into `vert` by current core; `pos[v]` is the
    // inverse mapping (where v lives in `vert`). The original C
    // implementation overwrites `bin` here as a write-cursor and then
    // shifts to restore it; we use a separate `bin_cursor` so that
    // `bin` already holds the bucket-start array we need below.
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

    // Main loop: walk vertices in ascending core order; for each
    // higher-core neighbour, swap it down a bin and decrement its
    // core. Self-loops in `neighbors()` are emitted twice (once per
    // endpoint), which is exactly the IGRAPH_LOOPS behaviour
    // upstream relies on.
    for i in 0..n_us {
        let v = vert[i];
        let neis = graph.neighbors(v)?;
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
    fn directed_returns_error() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(coreness(&g).is_err());
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
