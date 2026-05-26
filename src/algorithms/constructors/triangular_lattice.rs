//! Triangular lattice constructor (ALGO-CN-023).
//!
//! Counterpart of `igraph_triangular_lattice()` in
//! `references/igraph/src/constructors/lattices.c:290-319`.
//!
//! Builds a planar triangular lattice whose vertices have coordinates
//! `(i, j)` for non-negative integers and where each `(i, j)` is
//! connected to `(i + 1, j)`, `(i, j + 1)`, and `(i - 1, j + 1)`
//! whenever those neighbours exist on the lattice. Every vertex thus
//! has degree at most six, and the graph is the planar dual of the
//! hexagonal lattice over the same `dims`.
//!
//! The `dims` slice doubles as a *shape* selector with three modes:
//!
//! * `[n]`           — triangle with side length `n`
//! * `[size_x, size_y]` — quasi-rectangle, `size_x` rows of `size_y`
//!   vertices each
//! * `[size_x, size_y, size_z]` — hexagon with side lengths
//!   `size_x`, `size_y`, `size_z`
//!
//! Vertices are numbered lexicographically with the second coordinate
//! more significant (the C `lex_ordering = false` path), matching the
//! upstream library.
//!
//! Modes:
//!
//! * `directed = false`: every undirected lattice edge is emitted once
//!   with the lower vertex id as the source.
//! * `directed = true, mutual = false`: each lattice edge becomes a
//!   single arc oriented from the lower vertex id to the higher.
//! * `directed = true, mutual = true`: every undirected lattice edge
//!   becomes a pair of opposite arcs (the canonical orientation plus
//!   its reverse), matching the C `ADD_EDGE_IJ_KL_IF_EXISTS` macro's
//!   double push.
//!
//! Special cases:
//!
//! * `dims = []` and `dims = [...]` with any zero component both
//!   collapse to the empty graph (`vcount = 0`), matching the upstream
//!   "if `vector_int_contains(dims, 0)` return `igraph_empty`" guard.
//! * `dims.len() != 1, 2, 3` is rejected with [`IgraphError::InvalidArgument`].
//!
//! Algorithm: build per-row `row_lengths` and `row_start` arrays from
//! the requested shape, then walk every lattice vertex and emit the
//! three canonical "forward" neighbours that fall inside the lattice.
//! Boundary checks reproduce the upstream `ADD_EDGE_IJ_KL_IF_EXISTS`
//! macro byte-for-byte. The vertex index for `(i, j)` is
//! `prefix_sum[j] + (i - row_start[j])`.
//!
//! Time complexity: `O(|V| + |E|)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build a triangular lattice with the requested shape.
///
/// See the module documentation for the meaning of `dims`,
/// `directed`, and `mutual`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — when:
///   * `dims.len()` is not in `{1, 2, 3}` (the C
///     `IGRAPH_EINVAL: "size of dimension vector must be 1, 2 or 3"`
///     path),
///   * the implied vertex or edge count overflows `u32`.
///
/// The upstream "negative dimension" path is eliminated at the type
/// level by taking `&[u32]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::triangular_lattice;
///
/// // Triangle with side 5: 1 + 2 + 3 + 4 + 5 = 15 vertices.
/// let g = triangular_lattice(&[5], false, false).unwrap();
/// assert_eq!(g.vcount(), 15);
/// assert_eq!(g.ecount(), 30);
///
/// // 2 x 2 "rectangle": four vertices, five edges (matches python-igraph).
/// let g = triangular_lattice(&[2, 2], false, false).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 5);
///
/// // A zero in any coordinate collapses to the empty graph.
/// let g = triangular_lattice(&[3, 0], false, false).unwrap();
/// assert_eq!(g.vcount(), 0);
/// ```
pub fn triangular_lattice(dims: &[u32], directed: bool, mutual: bool) -> IgraphResult<Graph> {
    if !matches!(dims.len(), 1..=3) {
        return Err(IgraphError::InvalidArgument(format!(
            "triangular_lattice: size of dimension vector must be 1, 2 or 3, got {}",
            dims.len()
        )));
    }

    // Upstream short-circuit: any zero dim → empty graph.
    if dims.contains(&0) {
        return Graph::new(0, directed);
    }

    let (row_lengths, row_start) = match dims.len() {
        1 => triangle_shape(dims[0]),
        2 => rectangle_shape(dims[0], dims[1]),
        3 => hex_shape(dims[0], dims[1], dims[2]),
        _ => unreachable!("dims length already checked"),
    };

    layout(&row_lengths, &row_start, directed, mutual)
}

/// Compute `(row_lengths, row_start)` for the triangular-triangle shape.
///
/// Row `j` has `n - j` vertices starting at `0`, so the resulting
/// triangle has `n(n+1)/2` vertices in total.
fn triangle_shape(n: u32) -> (Vec<u32>, Vec<u32>) {
    let mut row_lengths = Vec::with_capacity(n as usize);
    let mut row_start = Vec::with_capacity(n as usize);
    for j in 0..n {
        row_lengths.push(n - j);
        row_start.push(0);
    }
    (row_lengths, row_start)
}

/// Compute `(row_lengths, row_start)` for the rectangular shape.
///
/// `size_x` rows of `size_y` vertices each. Row `j` starts at
/// `(row_count - j) / 2` (matching `igraph` integer division), giving
/// the lattice its characteristic skewed-rectangle outline.
fn rectangle_shape(size_x: u32, size_y: u32) -> (Vec<u32>, Vec<u32>) {
    let mut row_lengths = Vec::with_capacity(size_x as usize);
    let mut row_start = Vec::with_capacity(size_x as usize);
    for j in 0..size_x {
        row_lengths.push(size_y);
        row_start.push((size_x - j) / 2);
    }
    (row_lengths, row_start)
}

/// Compute `(row_lengths, row_start)` for the hexagonal shape.
///
/// `row_count = size_y + size_z - 1`. The first `size_y - 1` rows
/// grow the row length by one and shrink the row start by one
/// (expanding side `size_y`); the middle band of `|size_y - size_z|`
/// rows applies a sign-flagged shift to `row_start`; the final rows
/// shrink the row length back (contracting side `size_z`).
///
/// All intermediate values stay non-negative by construction:
/// `row_start_val` starts at `sy - 1` and decreases by at most
/// `sy.min(sz) - 1` in phase 1, leaving `sy - sy.min(sz) ≥ 0`;
/// phase 2 only decreases it when `sy >= sz`, by `sy - sz`, leaving
/// `0`; phase 3 doesn't touch `row_start_val`. Similarly `row_length`
/// grows in phase 1 and shrinks back in phase 3 to its starting value.
fn hex_shape(size_x: u32, size_y: u32, size_z: u32) -> (Vec<u32>, Vec<u32>) {
    // Caller has already filtered any zero dim out, so subtractions are safe.
    let row_count = size_y + size_z - 1;

    let mut row_length: u32 = size_x;
    let mut row_start_val: u32 = size_y - 1;
    let first_threshold: u32 = size_y.min(size_z) - 1;
    let second_threshold: u32 = size_y.max(size_z) - 1;
    let shrink_in_middle: bool = size_y >= size_z;

    let mut row_lengths = Vec::with_capacity(row_count as usize);
    let mut row_start = Vec::with_capacity(row_count as usize);

    for i in 0..row_count {
        row_lengths.push(row_length);
        row_start.push(row_start_val);

        if i < first_threshold {
            row_length += 1;
            row_start_val -= 1;
        } else if i < second_threshold {
            if shrink_in_middle {
                row_start_val -= 1;
            }
        } else {
            row_length -= 1;
        }
    }

    (row_lengths, row_start)
}

/// Lay out the lattice from per-row metadata, emit edges, and build
/// the graph.
///
/// Mirrors the upstream `triangular_lattice()` helper (with
/// `lex_ordering = false`). The vertex index for `(i, j)` is
/// `prefix_sum[j] + (i - row_start[j])`. For every vertex we try the
/// three "forward" lattice neighbours and only emit those that fall
/// inside the lattice.
fn layout(
    row_lengths: &[u32],
    row_start: &[u32],
    directed: bool,
    mutual: bool,
) -> IgraphResult<Graph> {
    debug_assert_eq!(row_lengths.len(), row_start.len());
    let row_count = row_lengths.len();

    // Prefix sum of row lengths → vertex index base for each row.
    let mut prefix_sum: Vec<u32> = Vec::with_capacity(row_count + 1);
    prefix_sum.push(0);
    for &len in row_lengths {
        let last = *prefix_sum.last().expect("non-empty");
        let next = last
            .checked_add(len)
            .ok_or_else(|| overflow_error("vertex count"))?;
        prefix_sum.push(next);
    }
    let vcount = *prefix_sum.last().expect("non-empty");

    // Vertex index helper: (i, j) → prefix_sum[j] + (i - row_start[j]).
    let vertex_index = |i: u32, j: usize| -> u32 { prefix_sum[j] + i - row_start[j] };
    let row_end = |j: usize| -> u32 {
        // row_lengths[j] >= 1 since j only ranges over non-empty rows
        // produced by triangle_shape / rectangle_shape / hex_shape.
        row_start[j] + row_lengths[j] - 1
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    // Try to emit the edge (i, j) → (k, l). The neighbour candidate is
    // skipped if l is out of bounds or k falls outside the row span at
    // l. `k_opt` is `None` when the candidate would have a negative i
    // coordinate (the "up-left" case at column 0).
    let add_if_in_bounds =
        |edges: &mut Vec<(VertexId, VertexId)>, i: u32, j: usize, k_opt: Option<u32>, l: usize| {
            let Some(k) = k_opt else {
                return;
            };
            if l >= row_count {
                return;
            }
            let l_start = row_start[l];
            let l_end = row_end(l);
            if k < l_start || k > l_end {
                return;
            }
            let src = vertex_index(i, j);
            let dst = vertex_index(k, l);
            edges.push((src, dst));
            if directed && mutual {
                edges.push((dst, src));
            }
        };

    for j in 0..row_count {
        let row_len = row_lengths[j];
        let start = row_start[j];
        for i in 0..row_len {
            let k = start + i;
            // Right neighbour in the same row.
            add_if_in_bounds(&mut edges, k, j, Some(k + 1), j);
            if j + 1 < row_count {
                // "Up" neighbour: (k, j+1).
                add_if_in_bounds(&mut edges, k, j, Some(k), j + 1);
                // "Up-left" neighbour: (k - 1, j + 1). Skips when k == 0.
                add_if_in_bounds(&mut edges, k, j, k.checked_sub(1), j + 1);
            }
        }
    }

    let mut g = Graph::new(vcount, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

fn overflow_error(kind: &str) -> IgraphError {
    IgraphError::InvalidArgument(format!("triangular_lattice: {kind} overflows u32"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_undirected(g: &Graph) -> BTreeSet<(u32, u32)> {
        let mut s = BTreeSet::new();
        for v in 0..g.vcount() {
            for &u in &g.neighbors(v).expect("neighbors") {
                let key = if v <= u { (v, u) } else { (u, v) };
                s.insert(key);
            }
        }
        s
    }

    fn directed_arcs(g: &Graph) -> BTreeSet<(u32, u32)> {
        (0..u32::try_from(g.ecount()).expect("ecount fits u32"))
            .map(|eid| g.edge(eid).expect("edge"))
            .collect()
    }

    #[test]
    fn empty_dims_rejected() {
        let r = triangular_lattice(&[], false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn four_dim_rejected() {
        let r = triangular_lattice(&[1, 2, 3, 4], false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn zero_dim_yields_empty_graph() {
        let g = triangular_lattice(&[3, 0], false, false).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_dim_directed_keeps_flag() {
        let g = triangular_lattice(&[0, 3, 4], true, true).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn triangle_single_vertex() {
        let g = triangular_lattice(&[1], true, false).expect("ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn triangle_side_five_matches_upstream_canon() {
        // Reproduces references/igraph/tests/unit/igraph_triangular_lattice.out
        // "Triangular triangular lattice" block, with vcount=15 and 30 arcs.
        let g = triangular_lattice(&[5], true, false).expect("ok");
        assert_eq!(g.vcount(), 15);
        assert_eq!(g.ecount(), 30);
        let want: BTreeSet<(u32, u32)> = [
            (0, 1),
            (0, 5),
            (1, 2),
            (1, 5),
            (1, 6),
            (2, 3),
            (2, 6),
            (2, 7),
            (3, 4),
            (3, 7),
            (3, 8),
            (4, 8),
            (5, 6),
            (5, 9),
            (6, 7),
            (6, 9),
            (6, 10),
            (7, 8),
            (7, 10),
            (7, 11),
            (8, 11),
            (9, 10),
            (9, 12),
            (10, 11),
            (10, 12),
            (10, 13),
            (11, 13),
            (12, 13),
            (12, 14),
            (13, 14),
        ]
        .into_iter()
        .collect();
        assert_eq!(directed_arcs(&g), want);
    }

    #[test]
    fn rectangle_two_by_two_matches_python_igraph_undirected() {
        // python-igraph test: Graph.Triangular_Lattice([2, 2]) gives
        // sorted edgelist [(0,1), (0,2), (0,3), (1,3), (2,3)].
        let g = triangular_lattice(&[2, 2], false, false).expect("ok");
        let want: BTreeSet<(u32, u32)> = [(0, 1), (0, 2), (0, 3), (1, 3), (2, 3)]
            .into_iter()
            .collect();
        assert_eq!(canonical_undirected(&g), want);
    }

    #[test]
    fn rectangle_two_by_two_directed_mutual_doubles_edges() {
        // python-igraph: mutual=true should emit each undirected edge twice.
        let g = triangular_lattice(&[2, 2], true, true).expect("ok");
        let want: BTreeSet<(u32, u32)> = [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 3),
            (2, 0),
            (2, 3),
            (3, 0),
            (3, 1),
            (3, 2),
        ]
        .into_iter()
        .collect();
        assert_eq!(directed_arcs(&g), want);
        assert!(g.is_directed());
    }

    #[test]
    fn rectangle_two_by_two_directed_unilateral_matches_undirected_set() {
        // python-igraph: directed=true,mutual=false emits the canonical
        // direction once — same edge set as the undirected build.
        let g = triangular_lattice(&[2, 2], true, false).expect("ok");
        let want: BTreeSet<(u32, u32)> = [(0, 1), (0, 2), (0, 3), (1, 3), (2, 3)]
            .into_iter()
            .collect();
        assert_eq!(directed_arcs(&g), want);
    }

    #[test]
    fn rectangle_four_by_five_matches_upstream_vcount() {
        // References .out file: directed=true,mutual=true, vcount=20,
        // 86 arcs (43 undirected × 2 mutual orientations).
        let g = triangular_lattice(&[4, 5], true, true).expect("ok");
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 86);
    }

    #[test]
    fn hexagonal_3_4_5_matches_upstream_vcount() {
        // References .out file: vcount=36, 87 undirected edges
        // (directed=false silently collapses `mutual`).
        let g = triangular_lattice(&[3, 4, 5], false, true).expect("ok");
        assert_eq!(g.vcount(), 36);
        assert_eq!(g.ecount(), 87);
    }

    #[test]
    fn triangle_three_full_topology() {
        // Triangle side 3: 6 vertices, 9 edges. Layout:
        //   row 0: (0,0)=0  (1,0)=1  (2,0)=2
        //   row 1: (0,1)=3  (1,1)=4
        //   row 2: (0,2)=5
        // Edges: right (0-1,1-2,3-4), up (0-3,1-4,3-5),
        //         up-left (1-3, 2-4, 4-5).
        let g = triangular_lattice(&[3], false, false).expect("ok");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 9);
        let want: BTreeSet<(u32, u32)> = [
            (0, 1),
            (0, 3),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
        ]
        .into_iter()
        .collect();
        assert_eq!(canonical_undirected(&g), want);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn triangle_vcount_is_triangular_number(
            n in 1u32..30,
        ) {
            let g = triangular_lattice(&[n], false, false).expect("ok");
            let expected = (u64::from(n) * (u64::from(n) + 1)) / 2;
            prop_assert_eq!(u64::from(g.vcount()), expected);
        }

        #[test]
        fn rectangle_vcount_is_product(
            sx in 1u32..15,
            sy in 1u32..15,
        ) {
            let g = triangular_lattice(&[sx, sy], false, false).expect("ok");
            prop_assert_eq!(u64::from(g.vcount()), u64::from(sx) * u64::from(sy));
        }

        #[test]
        fn directed_mutual_doubles_undirected_ecount(
            sx in 1u32..10,
            sy in 1u32..10,
        ) {
            let undirected = triangular_lattice(&[sx, sy], false, false).expect("ok");
            let mutual = triangular_lattice(&[sx, sy], true, true).expect("ok");
            prop_assert_eq!(mutual.ecount(), undirected.ecount() * 2);
        }

        #[test]
        fn directed_unilateral_matches_undirected_ecount(
            sx in 1u32..10,
            sy in 1u32..10,
        ) {
            let undirected = triangular_lattice(&[sx, sy], false, false).expect("ok");
            let unilateral = triangular_lattice(&[sx, sy], true, false).expect("ok");
            prop_assert_eq!(unilateral.ecount(), undirected.ecount());
        }

        #[test]
        fn directedness_flag_round_trips(
            sx in 1u32..8,
            sy in 1u32..8,
            directed: bool,
            mutual: bool,
        ) {
            let g = triangular_lattice(&[sx, sy], directed, mutual).expect("ok");
            prop_assert_eq!(g.is_directed(), directed);
        }

        #[test]
        fn max_degree_at_most_six(
            sx in 1u32..8,
            sy in 1u32..8,
        ) {
            let g = triangular_lattice(&[sx, sy], false, false).expect("ok");
            for v in 0..g.vcount() {
                prop_assert!(g.degree(v).unwrap() <= 6);
            }
        }
    }
}
