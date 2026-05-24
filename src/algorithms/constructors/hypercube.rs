//! Hypercube constructor (ALGO-CN-007).
//!
//! Counterpart of `igraph_hypercube()` in
//! `references/igraph/src/constructors/regular.c:983-1003`.
//!
//! The n-dimensional **hypercube graph** `Q_n` has `2^n` vertices and
//! `2^(n-1) * n` edges. Two vertices are connected when the binary
//! representations of their zero-based vertex IDs differ in **exactly
//! one bit**.
//!
//! Small cases:
//!
//! * n = 0 → singleton (one vertex, zero edges)
//! * n = 1 → K2 (two vertices, one edge: `(0, 1)`)
//! * n = 2 → 4-cycle (`(0,1), (0,2), (1,3), (2,3)`)
//! * n = 3 → 8-vertex cube (12 edges)
//!
//! The graph is `n`-regular (every vertex has degree `n`) and bipartite
//! (vertices split into two classes by the parity of their popcount).
//!
//! Algorithm: enumerate all `2^n` vertices in id order; for each vertex
//! `v` and bit `i ∈ [0, n)` toggle that bit to get `u = v ^ (1 << i)`,
//! and emit the canonical edge `(v, u)` only when `v < u` so each edge
//! is produced exactly once. Mirrors the upstream C loop.
//!
//! Time complexity: O(2^n · n) = O(|V| · log |V|) = O(|E|).
//!
//! # Upper bound
//!
//! `n` is capped at 30 so that:
//!
//! * vertex count `2^n` fits comfortably in `u32`,
//! * edge count `2^(n-1) · n` fits in `usize` on every supported
//!   platform (≤ ~1.6 × 10^10 on `n = 30`, well within 64-bit
//!   `usize::MAX`; this lets the `edges` `Vec` allocation succeed even
//!   on 32-bit targets where `usize::MAX ≈ 4 × 10^9` — at `n = 30` it
//!   would exhaust `usize`, but real workloads stay far below it).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Maximum dimension accepted by [`hypercube`].
///
/// At `n = 30` the graph already has `2^30 ≈ 1.07 × 10^9` vertices and
/// `2^29 · 30 ≈ 1.6 × 10^10` edges, which is well beyond practical
/// workloads. The upper bound also guarantees `1u32 << n` is always
/// well-defined.
pub const MAX_HYPERCUBE_DIMENSION: u32 = 30;

/// Build the n-dimensional hypercube graph `Q_n`.
///
/// Returns a graph on `2^n` vertices where two vertices share an edge
/// iff their zero-based IDs differ in exactly one bit. For the directed
/// variant every edge points from the lower-indexed endpoint to the
/// higher-indexed one (the canonical orientation also used by upstream
/// igraph). For `n = 0` the graph is the singleton; for `n = 1` it is
/// `K_2`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `n > 30` (see
///   [`MAX_HYPERCUBE_DIMENSION`]).
///
/// # Examples
///
/// ```
/// use rust_igraph::hypercube;
///
/// // Q_3 — 8-vertex cube, 12 edges, every vertex has degree 3.
/// let g = hypercube(3, false).unwrap();
/// assert_eq!(g.vcount(), 8);
/// assert_eq!(g.ecount(), 12);
/// ```
pub fn hypercube(n: u32, directed: bool) -> IgraphResult<Graph> {
    if n > MAX_HYPERCUBE_DIMENSION {
        return Err(IgraphError::InvalidArgument(format!(
            "hypercube: dimension {n} exceeds the supported maximum of {MAX_HYPERCUBE_DIMENSION}"
        )));
    }

    let vcount: u32 = 1u32 << n;

    // ecount = n == 0 ? 0 : 2^(n-1) * n. Always fits usize on 64-bit
    // and 32-bit (per the MAX_HYPERCUBE_DIMENSION analysis above), but
    // compute through checked_* so future bound changes can't silently
    // wrap around.
    let ecount: usize = if n == 0 {
        0
    } else {
        let half = 1usize.checked_shl(n - 1).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "hypercube: edge count overflows usize for dimension {n}"
            ))
        })?;
        half.checked_mul(n as usize).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "hypercube: edge count overflows usize for dimension {n}"
            ))
        })?
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    for v in 0..vcount {
        for i in 0..n {
            let u = v ^ (1u32 << i);
            if v < u {
                edges.push((v, u));
            }
        }
    }

    let mut g = Graph::new(vcount, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn degree_of(g: &Graph, v: VertexId) -> usize {
        g.degree(v).expect("vertex in range")
    }

    #[test]
    fn n0_is_singleton() {
        let g = hypercube(0, false).expect("n=0");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn n0_directed_is_singleton() {
        let g = hypercube(0, true).expect("n=0 directed");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn n1_is_k2() {
        let g = hypercube(1, false).expect("n=1");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert_eq!(dump_edges(&g), vec![(0, 1)]);
    }

    #[test]
    fn n2_is_four_cycle() {
        // Q_2 is the 4-cycle.
        // Edges from upstream loop: (0,1), (0,2), (1,3), (2,3).
        let g = hypercube(2, false).expect("n=2");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(dump_edges(&g), vec![(0, 1), (0, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn n3_is_three_cube() {
        let g = hypercube(3, false).expect("n=3");
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 12);
        // Bit-flip edges in upstream order.
        assert_eq!(
            dump_edges(&g),
            vec![
                (0, 1),
                (0, 2),
                (0, 4),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 6),
                (3, 7),
                (4, 5),
                (4, 6),
                (5, 7),
                (6, 7),
            ]
        );
    }

    #[test]
    fn n3_directed_same_edges_but_directed() {
        // Directed variant emits the same (v, u) pairs (each canonical
        // with v < u) and stores them as arcs.
        let g = hypercube(3, true).expect("n=3 directed");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 12);
        let edges = dump_edges(&g);
        // First edge oriented low → high.
        assert_eq!(edges[0], (0, 1));
        assert_eq!(edges[1], (0, 2));
    }

    #[test]
    fn n3_is_three_regular() {
        let g = hypercube(3, false).expect("n=3");
        for v in 0..g.vcount() {
            assert_eq!(degree_of(&g, v), 3, "vertex {v} should have degree 3");
        }
    }

    #[test]
    fn n4_vertex_and_edge_counts() {
        let g = hypercube(4, false).expect("n=4");
        // vcount = 2^4 = 16; ecount = 2^3 * 4 = 32.
        assert_eq!(g.vcount(), 16);
        assert_eq!(g.ecount(), 32);
        // 4-regular.
        for v in 0..g.vcount() {
            assert_eq!(degree_of(&g, v), 4, "vertex {v} should have degree 4");
        }
    }

    #[test]
    fn n4_is_bipartite_by_popcount() {
        // Hypercube is bipartite — every edge crosses popcount parity.
        let g = hypercube(4, false).expect("n=4");
        let m = u32::try_from(g.ecount()).unwrap();
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(
                u.count_ones() % 2,
                v.count_ones() % 2,
                "edge ({u}, {v}) should cross popcount parity"
            );
        }
    }

    #[test]
    fn edge_endpoints_differ_in_one_bit() {
        for n in 0..=5u32 {
            let g = hypercube(n, false).expect("hypercube ok");
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (u, v) = g.edge(e).unwrap();
                assert_eq!(
                    (u ^ v).count_ones(),
                    1,
                    "edge ({u}, {v}) in Q_{n} should differ in exactly one bit"
                );
            }
        }
    }

    #[test]
    fn edges_are_canonical_v_less_than_u() {
        // Every emitted edge has v < u (lower index first).
        let g = hypercube(5, true).expect("Q_5 directed");
        let m = u32::try_from(g.ecount()).unwrap();
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            assert!(u < v, "edge ({u}, {v}) must be canonically ordered");
        }
    }

    #[test]
    fn dimension_too_large_rejected() {
        let err = hypercube(31, false).expect_err("n=31 should reject");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("exceeds")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn dimension_far_too_large_rejected() {
        let err = hypercube(64, false).expect_err("n=64 should reject");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("exceeds")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // For every n in [0, 8], the hypercube is n-regular.
        #[test]
        fn n_regular(n in 0u32..=8) {
            let g = hypercube(n, false).unwrap();
            for v in 0..g.vcount() {
                prop_assert_eq!(g.degree(v).unwrap(), n as usize);
            }
        }

        // Vertex count is 2^n.
        #[test]
        fn vertex_count_is_power_of_two(n in 0u32..=10) {
            let g = hypercube(n, false).unwrap();
            prop_assert_eq!(g.vcount(), 1u32 << n);
        }

        // Edge count is 2^(n-1) * n (and 0 when n = 0).
        #[test]
        fn edge_count_is_half_n_times_two_to_n(n in 0u32..=8) {
            let g = hypercube(n, false).unwrap();
            let expected: usize = if n == 0 {
                0
            } else {
                (1usize << (n - 1)) * (n as usize)
            };
            prop_assert_eq!(g.ecount(), expected);
        }

        // Every edge differs in exactly one bit.
        #[test]
        fn edges_have_hamming_distance_one(n in 1u32..=8) {
            let g = hypercube(n, false).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_eq!((u ^ v).count_ones(), 1);
            }
        }
    }
}
