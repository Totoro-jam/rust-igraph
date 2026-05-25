//! Square (multi-dimensional grid) lattice constructor (ALGO-CN-009).
//!
//! Counterpart of `igraph_square_lattice()` in
//! `references/igraph/src/constructors/regular.c:337-459`.
//!
//! Builds a `d`-dimensional square lattice with arbitrary per-dimension
//! cardinalities and optional per-dimension torus wrap-around. The
//! vertex at lattice position `(i_0, i_1, …, i_{d-1})` in a lattice of
//! shape `(n_0, n_1, …, n_{d-1})` carries vertex id
//! `i_0 + n_0 · i_1 + n_0·n_1 · i_2 + …` (little-endian, matching the
//! upstream convention).
//!
//! Modes:
//!
//! * `directed = false`: each lattice edge is emitted once with the
//!   lower vertex id as the source.
//! * `directed = true, mutual = false`: each edge becomes a single
//!   arc pointing from the lower-indexed vertex to the higher one.
//! * `directed = true, mutual = true`: every undirected lattice edge
//!   becomes a pair of opposite arcs. For periodic dimensions of size
//!   `2` the upstream code suppresses the duplicate so the
//!   forward/back wrap edges are not double-emitted; this port matches
//!   that behaviour bit-for-bit.
//!
//! Limitations of this first cut:
//!
//! * `nei` (neighbour-distance radius) is restricted to `0` or `1`.
//!   Higher values require the future `igraph_connect_neighborhood`
//!   helper (a separate AWU) — they are rejected with
//!   `InvalidArgument` for now.
//!
//! Special cases:
//!
//! * `dim = []` (zero dimensions) → singleton (one vertex).
//! * any `dim[k] == 0` → null graph (zero vertices, zero edges).
//! * any `dim[k] == 1` contributes no edges along that dimension.
//!
//! Algorithm: walk every vertex in little-endian lattice order. At
//! each step, for every dimension `j`, emit the canonical
//! "forward" edge to the neighbour with `coords[j] + 1` (or the
//! wrap-around when `coords[j] == dim[j] − 1` and that dimension is
//! periodic). When `directed && mutual`, additionally emit the
//! "backward" reverse arc. Self-loops (only possible when a wrap
//! produces the same vertex) and duplicate dim-2 edges are filtered
//! out exactly as upstream does. Mirrors the C loop.
//!
//! Time complexity: `O(|V| · d) = O(|E|)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build a `d`-dimensional square lattice graph.
///
/// Returns a graph on `Π dim[k]` vertices arranged on a multi-dimensional
/// grid, with edges joining vertices that are one step apart along any
/// single axis. `periodic` (when supplied) selects which dimensions
/// wrap around (torus mode).
///
/// # Parameters
///
/// * `dim` — per-dimension cardinalities. Empty slice is the
///   zero-dimensional case (singleton).
/// * `nei` — neighbour-distance radius. Only `0` and `1` are supported
///   in this initial port (higher values require
///   `igraph_connect_neighborhood`, a separate AWU).
/// * `directed` — whether to produce a directed graph.
/// * `mutual` — when `directed`, emit both directions of every edge.
///   Ignored when `directed == false`.
/// * `periodic` — optional per-dimension wrap-around flags. Must have
///   the same length as `dim` when supplied; otherwise treat all
///   dimensions as non-periodic.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] —
///   * `nei >= 2` (not yet supported),
///   * length of `periodic` does not match `dim`,
///   * vertex count `Π dim[k]` overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::square_lattice;
///
/// // 3 × 3 grid: 9 vertices, 12 edges, 2D bounded.
/// let g = square_lattice(&[3, 3], 1, false, false, None).unwrap();
/// assert_eq!(g.vcount(), 9);
/// assert_eq!(g.ecount(), 12);
///
/// // 3 × 3 torus: same vertex count, 18 edges (each cell contributes
/// // one forward edge per dim).
/// let g = square_lattice(&[3, 3], 1, false, false, Some(&[true, true])).unwrap();
/// assert_eq!(g.vcount(), 9);
/// assert_eq!(g.ecount(), 18);
/// ```
pub fn square_lattice(
    dim: &[u32],
    nei: u32,
    directed: bool,
    mutual: bool,
    periodic: Option<&[bool]>,
) -> IgraphResult<Graph> {
    if nei >= 2 {
        return Err(IgraphError::InvalidArgument(format!(
            "square_lattice: nei={nei} not yet supported (only nei in {{0, 1}}); \
             higher radii require igraph_connect_neighborhood (future AWU)"
        )));
    }
    if let Some(p) = periodic {
        if p.len() != dim.len() {
            return Err(IgraphError::InvalidArgument(format!(
                "square_lattice: periodic vector length {} must match dim length {}",
                p.len(),
                dim.len()
            )));
        }
    }

    let dims = dim.len();

    // vcount = Π dim[k]. Empty product is 1 (zero-dim → singleton).
    let mut vcount: u32 = 1;
    for &d in dim {
        vcount = vcount.checked_mul(d).ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "square_lattice: vertex count Π dim overflows u32 for dim={dim:?}"
            ))
        })?;
    }

    // Any zero in dim collapses everything to the null graph.
    if vcount == 0 {
        return Graph::new(0, directed);
    }

    // weights[i] = Π dim[0..i].
    let mut weights: Vec<u32> = Vec::with_capacity(dims);
    let mut acc: u32 = 1;
    for &d in dim {
        weights.push(acc);
        acc = acc.checked_mul(d).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("square_lattice: weight overflow for dim={dim:?}"))
        })?;
    }

    let mut coords: Vec<u32> = vec![0; dims];
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    let is_periodic = |idx: usize| -> bool { periodic.is_some_and(|p| p[idx]) };

    for i in 0..vcount {
        for j in 0..dims {
            let dj = dim[j];
            let pj = is_periodic(j);
            let cj = coords[j];

            // Forward neighbour: step +1 along axis j (or wrap when at boundary).
            if pj || cj != dj - 1 {
                let new_nei: u32 = if cj == dj - 1 {
                    i - (dj - 1) * weights[j]
                } else {
                    i + weights[j]
                };
                // Skip self-loops (possible when wrap lands on the same id,
                // e.g. dim[j] == 1) and duplicate dim-2 wrap edges in
                // undirected mode.
                if new_nei != i && (dj != 2 || cj != 1 || directed) {
                    edges.push((i, new_nei));
                }
            }

            // Backward neighbour, only when directed && mutual: emit the
            // reverse arc so every undirected edge becomes a pair.
            if mutual && directed && (pj || cj != 0) {
                let new_nei: u32 = if cj != 0 {
                    i - weights[j]
                } else {
                    i + (dj - 1) * weights[j]
                };
                // Skip self-loops and the dim-2 periodic case where
                // forward and reverse wraps would point to the same edge.
                if new_nei != i && (dj != 2 || !pj) {
                    edges.push((i, new_nei));
                }
            }
        }

        // Little-endian carry: increment coords[0] first; on overflow
        // reset and ripple to the next axis. After the last vertex this
        // wraps back to all zeros, which is harmless since the outer
        // loop terminates.
        for pos in 0..dims {
            if coords[pos] != dim[pos] - 1 {
                coords[pos] += 1;
                break;
            }
            coords[pos] = 0;
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
    fn zero_dim_is_singleton() {
        let g = square_lattice(&[], 1, false, false, None).expect("0-dim");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn dim_one_is_singleton() {
        let g = square_lattice(&[1], 1, false, false, None).expect("dim=[1]");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_in_dim_collapses_to_null() {
        let g = square_lattice(&[0, 3], 1, false, false, None).expect("null");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn one_d_path() {
        // 1-D non-periodic dim=[3] → path P_3.
        let g = square_lattice(&[3], 1, false, false, None).expect("P_3");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        assert_eq!(dump_edges(&g), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn one_d_periodic_is_cycle() {
        // 1-D periodic dim=[3] → cycle C_3. Undirected wrap edge (2,0)
        // gets normalized to (0,2) by Graph::add_edges.
        let g = square_lattice(&[3], 1, false, false, Some(&[true])).expect("C_3");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert_eq!(dump_edges(&g), vec![(0, 1), (1, 2), (0, 2)]);
    }

    #[test]
    fn two_d_2x2_is_four_cycle() {
        let g = square_lattice(&[2, 2], 1, false, false, None).expect("2x2");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(dump_edges(&g), vec![(0, 1), (0, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn two_d_3x3_grid() {
        // 3x3 grid non-periodic: 12 edges; vertex 4 (center) has degree 4.
        let g = square_lattice(&[3, 3], 1, false, false, None).expect("3x3");
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 12);
        // Corner vertex 0 has degree 2; center 4 has degree 4; edge midpoint 1 has degree 3.
        assert_eq!(degree_of(&g, 0), 2);
        assert_eq!(degree_of(&g, 1), 3);
        assert_eq!(degree_of(&g, 4), 4);
        assert_eq!(degree_of(&g, 8), 2);
    }

    #[test]
    fn two_d_3x3_torus() {
        // 3x3 torus: each vertex has degree 4; ecount = 9*2 = 18.
        let g = square_lattice(&[3, 3], 1, false, false, Some(&[true, true])).expect("torus");
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 18);
        for v in 0..g.vcount() {
            assert_eq!(degree_of(&g, v), 4, "torus vertex {v} should be 4-regular");
        }
    }

    #[test]
    fn two_d_2x2_torus_does_not_double_edge() {
        // For dim=2 periodic, wrap edges would duplicate — upstream
        // suppresses them; the 2x2 torus is still just the 4-cycle.
        let g = square_lattice(&[2, 2], 1, false, false, Some(&[true, true])).expect("torus 2x2");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(dump_edges(&g), vec![(0, 1), (0, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn three_d_2x2x2_is_cube() {
        // dim=[2,2,2] non-periodic = Q_3 (8v, 12e).
        let g = square_lattice(&[2, 2, 2], 1, false, false, None).expect("Q_3");
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 12);
        for v in 0..g.vcount() {
            assert_eq!(degree_of(&g, v), 3, "Q_3 vertex {v} should be 3-regular");
        }
    }

    #[test]
    fn directed_low_to_high_only() {
        // Directed non-mutual path: each edge points from lower to higher.
        let g = square_lattice(&[3], 1, true, false, None).expect("dir P_3");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 2);
        assert_eq!(dump_edges(&g), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn directed_mutual_emits_both_arcs() {
        // dim=[3] directed mutual: arcs in both directions.
        let g = square_lattice(&[3], 1, true, true, None).expect("dir mut P_3");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 4);
        // Sorted by source-vertex order in the C loop: forward (0,1) and
        // (1,2) from i=0 and i=1, and backward (1,0) and (2,1) from i=1
        // and i=2 respectively.
        let edges = dump_edges(&g);
        assert!(edges.contains(&(0, 1)));
        assert!(edges.contains(&(1, 0)));
        assert!(edges.contains(&(1, 2)));
        assert!(edges.contains(&(2, 1)));
    }

    #[test]
    fn directed_mutual_periodic_3cycle() {
        // dim=[3] periodic directed mutual: arcs both ways around C_3.
        let g = square_lattice(&[3], 1, true, true, Some(&[true])).expect("dir mut C_3");
        assert!(g.is_directed());
        // Forward arcs (0,1),(1,2),(2,0) plus backward (1,0),(2,1),(0,2).
        assert_eq!(g.ecount(), 6);
    }

    #[test]
    fn mixed_periodicity() {
        // dim=[3,3] with only first dim wrapping. Each row is a 3-cycle,
        // each column is a path. Edge count: 3 cycles * 3 edges + 2 column
        // edges * 3 cols = 9 + 6 = 15.
        let g =
            square_lattice(&[3, 3], 1, false, false, Some(&[true, false])).expect("mixed periodic");
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 15);
    }

    #[test]
    fn nei_two_or_more_rejected() {
        let err = square_lattice(&[3, 3], 2, false, false, None).expect_err("nei=2");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("nei")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn periodic_length_mismatch_rejected() {
        let err = square_lattice(&[3, 3], 1, false, false, Some(&[true])).expect_err("mismatch");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("periodic")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn vcount_overflow_rejected() {
        // 65536 * 65536 = 2^32 → overflow.
        let err = square_lattice(&[65536, 65536], 1, false, false, None).expect_err("overflow");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("overflow")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // For non-periodic 2-D grids, ecount = a*(b-1) + b*(a-1).
        #[test]
        fn two_d_grid_edge_count(a in 1u32..=6, b in 1u32..=6) {
            let g = square_lattice(&[a, b], 1, false, false, None).unwrap();
            let expected = a * (b - 1) + b * (a - 1);
            prop_assert_eq!(g.vcount(), a * b);
            prop_assert_eq!(g.ecount() as u32, expected);
        }

        // For fully-periodic 2-D torus with both dims >= 3, every vertex
        // is 4-regular.
        #[test]
        fn two_d_torus_is_four_regular(a in 3u32..=6, b in 3u32..=6) {
            let g = square_lattice(
                &[a, b], 1, false, false, Some(&[true, true]),
            ).unwrap();
            for v in 0..g.vcount() {
                prop_assert_eq!(g.degree(v).unwrap(), 4);
            }
        }

        // dim=[2, 2, …] non-periodic with k dims is the hypercube Q_k.
        #[test]
        fn power_of_two_lattice_is_hypercube(k in 0u32..=5) {
            use crate::hypercube;
            let dims = vec![2u32; k as usize];
            let g = square_lattice(&dims, 1, false, false, None).unwrap();
            let q = hypercube(k, false).unwrap();
            prop_assert_eq!(g.vcount(), q.vcount());
            prop_assert_eq!(g.ecount(), q.ecount());
        }

        // Directed + mutual doubles the undirected edge count on a
        // non-periodic 2-D grid.
        #[test]
        fn directed_mutual_doubles_undirected(a in 2u32..=5, b in 2u32..=5) {
            let u = square_lattice(&[a, b], 1, false, false, None).unwrap();
            let dm = square_lattice(&[a, b], 1, true, true, None).unwrap();
            prop_assert_eq!(dm.ecount(), u.ecount() * 2);
        }
    }
}
