//! `count_automorphisms` (`ALGO-ISO-004`).

use super::canonicalize;
use crate::core::{Graph, IgraphResult};

/// Count the automorphisms of `graph` — the order `|Aut(G)|` of its
/// automorphism group.
///
/// An automorphism is a vertex permutation that maps `graph` onto itself
/// (preserving adjacency and, when supplied, vertex colours). The count can
/// grow factorially, so it is returned as an `f64`, exactly as igraph's
/// `igraph_count_automorphisms` returns an `igraph_real_t`. The value is
/// exact up to `2^53`; beyond that the `f64` loses precision (igraph has the
/// same limit and offers a string-returning variant for larger groups, which
/// is out of scope here).
///
/// `colors` is an optional per-vertex colour slice (length must equal the
/// vertex count); only colour-preserving permutations are counted. Pass
/// `None` for an uncoloured graph.
///
/// The count is computed by the shared individualization-refinement engine
/// (see [`canonical_permutation`](super::canonical_permutation::canonical_permutation)):
/// `|Aut(G)|` equals the number of leaves of the I-R search tree whose
/// relabeled-adjacency certificate equals the canonical (lexicographically
/// maximal) one.
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
/// use rust_igraph::{Graph, count_automorphisms};
///
/// // The 4-cycle C_4 has the dihedral group of order 2*4 = 8.
/// let mut c4 = Graph::new(4, false).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
///     c4.add_edge(u, v).unwrap();
/// }
/// assert_eq!(count_automorphisms(&c4, None).unwrap(), 8.0);
/// ```
pub fn count_automorphisms(graph: &Graph, colors: Option<&[u32]>) -> IgraphResult<f64> {
    Ok(canonicalize(graph, colors)?.group_order)
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)] // small factorial counts, exact below 2^52
mod tests {
    use super::*;

    fn cycle(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    fn complete(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for u in 0..n {
            for v in (u + 1)..n {
                g.add_edge(u, v).expect("edge");
            }
        }
        g
    }

    fn path(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    fn petersen() -> Graph {
        let mut g = Graph::new(10, false).expect("graph");
        for i in 0..5 {
            g.add_edge(i, (i + 1) % 5).expect("edge"); // outer cycle
            g.add_edge(i + 5, ((i + 2) % 5) + 5).expect("edge"); // inner pentagram
            g.add_edge(i, i + 5).expect("edge"); // spokes
        }
        g
    }

    #[test]
    fn complete_graph_is_factorial() {
        let fact = [1u64, 1, 2, 6, 24, 120, 720];
        for n in 1u32..=6 {
            assert_eq!(
                count_automorphisms(&complete(n), None).expect("count"),
                fact[n as usize] as f64
            );
        }
    }

    #[test]
    fn cycle_is_dihedral() {
        for n in 3u32..=8 {
            assert_eq!(
                count_automorphisms(&cycle(n, false), None).expect("count"),
                f64::from(2 * n)
            );
        }
    }

    #[test]
    fn path_is_two() {
        for n in 2u32..=8 {
            assert_eq!(count_automorphisms(&path(n), None).expect("count"), 2.0);
        }
    }

    #[test]
    fn petersen_is_120() {
        assert_eq!(
            count_automorphisms(&petersen(), None).expect("count"),
            120.0
        );
    }

    #[test]
    fn directed_cycle_is_n() {
        // A directed cycle has only its n rotations as automorphisms.
        for n in 3u32..=8 {
            assert_eq!(
                count_automorphisms(&cycle(n, true), None).expect("count"),
                f64::from(n)
            );
        }
    }

    #[test]
    fn empty_graph_is_factorial() {
        // No edges: every permutation is an automorphism, so |Aut| = n!.
        let g = Graph::new(4, false).expect("graph");
        assert_eq!(count_automorphisms(&g, None).expect("count"), 24.0);
    }

    #[test]
    fn null_graph_is_one() {
        let g = Graph::new(0, false).expect("graph");
        assert_eq!(count_automorphisms(&g, None).expect("count"), 1.0);
    }

    #[test]
    fn colors_restrict_the_group() {
        // K_4 has |Aut| = 24; colouring one vertex distinctly fixes it,
        // leaving S_3 on the other three -> |Aut| = 6.
        let g = complete(4);
        let colors = [1u32, 0, 0, 0];
        assert_eq!(count_automorphisms(&g, Some(&colors)).expect("count"), 6.0);
    }

    #[test]
    fn rejects_multigraph() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        assert!(count_automorphisms(&g, None).is_err());
    }

    #[test]
    fn rejects_wrong_color_length() {
        let g = cycle(3, false);
        let colors = [0u32, 1];
        assert!(count_automorphisms(&g, Some(&colors)).is_err());
    }
}
