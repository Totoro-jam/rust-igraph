//! `canonical_permutation` (`ALGO-ISO-003`).

use super::canonicalize;
use crate::core::{Graph, IgraphResult};

/// Compute a canonical vertex permutation of `graph`.
///
/// The returned vector `labeling` has one entry per vertex: `labeling[v]` is
/// the canonical position assigned to vertex `v` (a vertex → position map).
/// Two graphs are isomorphic if and only if relabeling each by its canonical
/// permutation produces the same graph, so canonical permutations give an
/// isomorphism test by comparison of canonical forms.
///
/// `colors` is an optional per-vertex colour slice (length must equal the
/// vertex count); vertices of different colours are never identified. Pass
/// `None` for an uncoloured graph.
///
/// This mirrors igraph's `igraph_canonical_permutation` (BLISS-backed
/// upstream). The *specific* labeling is implementation-defined — only the
/// resulting canonical form is meaningful — so the labeling here need not
/// match igraph's byte-for-byte, but the induced canonical form is a valid
/// isomorphism invariant.
///
/// # Scope
///
/// Simple graphs (directed or undirected), self-loops allowed, optional
/// vertex colours. Multi-edges are rejected.
///
/// # Errors
///
/// Returns an error if `colors` has the wrong length or the graph has
/// multiple edges between the same pair of vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, canonical_permutation, permute_vertices};
///
/// // Two differently-labeled copies of the path 0-1-2.
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
///
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(2, 1).unwrap();
/// b.add_edge(1, 0).unwrap();
///
/// let pa = canonical_permutation(&a, None).unwrap();
/// let pb = canonical_permutation(&b, None).unwrap();
/// assert_eq!(pa.len(), 3);
///
/// // Relabeling each graph to canonical form yields identical edge sets.
/// // `permute_vertices` wants a position -> vertex map, the inverse of `labeling`.
/// let inv = |p: &[u32]| -> Vec<u32> {
///     let mut q = vec![0u32; p.len()];
///     for (v, &pos) in p.iter().enumerate() {
///         q[pos as usize] = v as u32;
///     }
///     q
/// };
/// let ca = permute_vertices(&a, &inv(&pa)).unwrap();
/// let cb = permute_vertices(&b, &inv(&pb)).unwrap();
/// let mut ea: Vec<_> = (0..ca.ecount()).map(|e| ca.edge(e as u32).unwrap()).collect();
/// let mut eb: Vec<_> = (0..cb.ecount()).map(|e| cb.edge(e as u32).unwrap()).collect();
/// ea.sort_unstable();
/// eb.sort_unstable();
/// assert_eq!(ea, eb);
/// ```
pub fn canonical_permutation(graph: &Graph, colors: Option<&[u32]>) -> IgraphResult<Vec<u32>> {
    Ok(canonicalize(graph, colors)?.labeling)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::operators::permute_vertices::permute_vertices;

    /// Apply a vertex → position labeling to produce the canonical graph and
    /// return its sorted, direction-aware edge list.
    fn canon_edges(g: &Graph, labeling: &[u32]) -> Vec<(u32, u32)> {
        let mut edges: Vec<(u32, u32)> = (0..g.ecount())
            .map(|e| {
                let (u, v) = g.edge(e as u32).expect("edge in range");
                let (cu, cv) = (labeling[u as usize], labeling[v as usize]);
                if g.is_directed() || cu <= cv {
                    (cu, cv)
                } else {
                    (cv, cu)
                }
            })
            .collect();
        edges.sort_unstable();
        edges
    }

    fn cycle(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    #[test]
    fn labeling_is_a_permutation() {
        let g = cycle(6, false);
        let p = canonical_permutation(&g, None).expect("canon");
        let mut sorted = p.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..6).collect::<Vec<_>>());
    }

    #[test]
    fn empty_and_singleton() {
        let e = Graph::new(0, false).expect("graph");
        assert!(canonical_permutation(&e, None).expect("canon").is_empty());

        let s = Graph::new(1, false).expect("graph");
        assert_eq!(canonical_permutation(&s, None).expect("canon"), vec![0]);
    }

    #[test]
    fn invariant_under_relabeling_undirected() {
        let g = cycle(7, false);
        // Reverse relabeling: position i holds old vertex (6 - i).
        let perm: Vec<u32> = (0..7).rev().collect();
        let h = permute_vertices(&g, &perm).expect("permute");

        let pg = canonical_permutation(&g, None).expect("canon g");
        let ph = canonical_permutation(&h, None).expect("canon h");
        assert_eq!(canon_edges(&g, &pg), canon_edges(&h, &ph));
    }

    #[test]
    fn invariant_under_relabeling_directed() {
        let mut g = Graph::new(4, true).expect("graph");
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)] {
            g.add_edge(u, v).expect("edge");
        }
        let perm = [2u32, 0, 3, 1];
        let h = permute_vertices(&g, &perm).expect("permute");

        let pg = canonical_permutation(&g, None).expect("canon g");
        let ph = canonical_permutation(&h, None).expect("canon h");
        assert_eq!(canon_edges(&g, &pg), canon_edges(&h, &ph));
    }

    #[test]
    fn non_isomorphic_graphs_differ() {
        // Path 0-1-2-3 vs star centered at 0.
        let mut path = Graph::new(4, false).expect("graph");
        for (u, v) in [(0, 1), (1, 2), (2, 3)] {
            path.add_edge(u, v).expect("edge");
        }
        let mut star = Graph::new(4, false).expect("graph");
        for v in 1..4 {
            star.add_edge(0, v).expect("edge");
        }
        let pp = canonical_permutation(&path, None).expect("canon");
        let ps = canonical_permutation(&star, None).expect("canon");
        assert_ne!(canon_edges(&path, &pp), canon_edges(&star, &ps));
    }

    #[test]
    fn respects_vertex_colors() {
        // Two-vertex single edge; distinct colours forbid swapping them, so
        // the colour-0 vertex must always map to canonical position 0.
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let colors = [0u32, 1];
        let p = canonical_permutation(&g, Some(&colors)).expect("canon");
        assert_eq!(p[0], 0);
        assert_eq!(p[1], 1);
    }

    #[test]
    fn rejects_multigraph() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        assert!(canonical_permutation(&g, None).is_err());
    }

    #[test]
    fn rejects_wrong_color_length() {
        let g = cycle(3, false);
        let colors = [0u32, 1];
        assert!(canonical_permutation(&g, Some(&colors)).is_err());
    }
}
