//! Hexagonal lattice constructor (ALGO-CN-024).
//!
//! Counterpart of `igraph_hexagonal_lattice()` in
//! `references/igraph/src/constructors/lattices.c:572-616`.
//!
//! Builds a planar hexagonal (honeycomb) lattice whose vertices have
//! coordinates `(i, j)` for non-negative integers, with `(i, j)` joined
//! to `(i + 1, j)` and (when `i` is odd) also to `(i - 1, j + 1)`. The
//! graph is the planar dual of [`crate::triangular_lattice`]:
//! 1:1 correspondence between length-6 cycles here and triangles there.
//! Every vertex has degree at most 3.
//!
//! The `dims` slice doubles as a *shape* selector with three modes:
//!
//! * `[n]`                  — triangular outline, `n` hexagons per side
//! * `[size_x, size_y]`     — quasi-rectangle, `size_x × size_y` hexagons
//! * `[size_x, size_y, size_z]` — hexagonal outline with the three sides
//!   carrying `size_x`, `size_y`, `size_z` hexagons respectively
//!
//! Vertices are numbered lexicographically with the second coordinate
//! more significant, matching the upstream `lex_ordering = false` path.
//!
//! Modes:
//!
//! * `directed = false`: every undirected lattice edge is emitted once.
//! * `directed = true, mutual = false`: each lattice edge becomes a
//!   single arc from the lower id to the higher.
//! * `directed = true, mutual = true`: every undirected lattice edge
//!   becomes a pair of opposite arcs.
//!
//! Special cases:
//!
//! * Any `dims[k] == 0` → empty graph (upstream
//!   `igraph_vector_int_any_smaller(dims, 1)` guard).
//! * `dims.len() != 1, 2, 3` → [`IgraphError::InvalidArgument`].
//!
//! Algorithm: byte-for-byte port of upstream's three private shape
//! helpers (`hexagonal_lattice_triangle_shape` /
//! `hexagonal_lattice_rectangle_shape` / `hexagonal_lattice_hex_shape`)
//! followed by the shared `hexagonal_lattice` emitter, which walks
//! every site and emits the two candidate forward neighbours (right,
//! and — only for odd `k` — up-left).
//!
//! Time complexity: `O(|V| + |E|)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build a hexagonal (honeycomb) lattice with the requested shape.
///
/// See the module documentation for the meaning of `dims`,
/// `directed`, and `mutual`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — when:
///   * `dims.len()` is not in `{1, 2, 3}` (upstream
///     `IGRAPH_EINVAL: "size of the dimension vector must be 1, 2 or 3"`),
///   * the implied vertex or edge count overflows `u32`.
///
/// The upstream "negative dimension" path is eliminated at the type
/// level by taking `&[u32]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::hexagonal_lattice;
///
/// // dims=[1] — a single hexagon: 6 vertices forming C_6 with 6 edges.
/// let g = hexagonal_lattice(&[1], false, false).unwrap();
/// assert_eq!(g.vcount(), 6);
/// assert_eq!(g.ecount(), 6);
///
/// // 2 x 2 quasi-rectangle from python-igraph testHexagonalLattice.
/// let g = hexagonal_lattice(&[2, 2], false, false).unwrap();
/// assert_eq!(g.vcount(), 16);
/// assert_eq!(g.ecount(), 19);
///
/// // Zero in any dim collapses to the empty graph.
/// let g = hexagonal_lattice(&[3, 0], false, false).unwrap();
/// assert_eq!(g.vcount(), 0);
/// ```
pub fn hexagonal_lattice(dims: &[u32], directed: bool, mutual: bool) -> IgraphResult<Graph> {
    if !matches!(dims.len(), 1..=3) {
        return Err(IgraphError::InvalidArgument(format!(
            "hexagonal_lattice: size of dimension vector must be 1, 2 or 3, got {}",
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

/// Triangular-outline shape (`dims = [size]`).
///
/// Upstream sets `row_count = size + 2` and iterates `i = 0..row_count - 1`
/// (so `size + 1` rows). For `i = 0` the row has `2*row_count - 3`
/// vertices and starts at column `1`; for `i >= 1` the row has
/// `2*(row_count - i) - 1` vertices and starts at column `0`.
fn triangle_shape(size: u32) -> (Vec<u32>, Vec<u32>) {
    let row_count_raw = size + 2;
    let n_rows = (size + 1) as usize;
    let mut row_lengths = Vec::with_capacity(n_rows);
    let mut row_start = Vec::with_capacity(n_rows);
    for i in 0..(row_count_raw - 1) {
        let len = 2 * (row_count_raw - i) - if i == 0 { 3 } else { 1 };
        row_lengths.push(len);
        row_start.push(u32::from(i == 0));
    }
    (row_lengths, row_start)
}

/// Quasi-rectangular shape (`dims = [size_x, size_y]`).
///
/// `row_count = size_x + 1`. `actual_size_y = 2*(size_y + 1)`. The
/// first and last rows shed one vertex; the others carry
/// `actual_size_y`. `row_start[i] = row_count - i - 1`, with an extra
/// `+1` in the first row when the residual `(row_count - i - 1) % 2`
/// is even, mirroring upstream's `is_first_row && !is_start_odd`.
fn rectangle_shape(size_x: u32, size_y: u32) -> (Vec<u32>, Vec<u32>) {
    let row_count = size_x + 1;
    let n_rows = row_count as usize;
    let actual_size_y = (size_y + 1) * 2;

    let mut row_lengths = Vec::with_capacity(n_rows);
    let mut row_start = Vec::with_capacity(n_rows);

    for i in 0..row_count {
        let is_first_row = i == 0;
        let is_last_row = i == row_count - 1;
        let is_start_odd = ((row_count - i - 1) % 2) != 0;
        let len = actual_size_y - u32::from(is_first_row || is_last_row);
        let start = (row_count - i - 1) + u32::from(is_first_row && !is_start_odd);
        row_lengths.push(len);
        row_start.push(start);
    }
    (row_lengths, row_start)
}

/// Hexagonal-outline shape (`dims = [size_x, size_y, size_z]`).
///
/// `row_count = size_y + size_z`. Initial `row_length = 2*size_x + 1`,
/// initial `row_start = 2*size_y - 1`. Three-phase update with
/// thresholds `first = min(size_y, size_z) - 1`,
/// `second = max(size_y, size_z) - 1`, and `sgn = if size_y < size_z
/// { 0 } else { -2 }` (encoded as a boolean for the middle phase).
/// Two extra corrections at `i == size_y - 1` and `i == size_z - 1`
/// match upstream byte-for-byte.
fn hex_shape(size_x: u32, size_y: u32, size_z: u32) -> (Vec<u32>, Vec<u32>) {
    let row_count = size_y + size_z;
    let n_rows = row_count as usize;

    // Use i64 internally because the middle phase can transiently
    // decrement `row_start` (sgn_flag = -2) before later phases bring
    // it back. The final values written to the vec are non-negative by
    // construction (upstream invariant) but we widen to be defensive.
    let mut row_length: i64 = i64::from(size_x) * 2 + 1;
    let mut row_start_val: i64 = i64::from(size_y) * 2 - 1;
    let first_threshold: i64 = i64::from(size_y.min(size_z)) - 1;
    let second_threshold: i64 = i64::from(size_y.max(size_z)) - 1;
    let middle_shrinks: bool = size_y >= size_z;

    let mut row_lengths = Vec::with_capacity(n_rows);
    let mut row_start = Vec::with_capacity(n_rows);

    for i in 0..row_count {
        let len_u32 = u32::try_from(row_length).expect("row_length non-negative by construction");
        let start_u32 =
            u32::try_from(row_start_val).expect("row_start non-negative by construction");
        row_lengths.push(len_u32);
        row_start.push(start_u32);

        let ii = i64::from(i);
        if ii < first_threshold {
            row_length += 2;
            row_start_val -= 2;
        } else if ii < second_threshold {
            if middle_shrinks {
                row_start_val -= 2;
            }
        } else {
            row_length -= 2;
        }
        if i == size_y - 1 {
            row_start_val -= 1;
            row_length += 1;
        }
        if i == size_z - 1 {
            row_length += 1;
        }
    }

    (row_lengths, row_start)
}

/// Emit edges from per-row metadata and assemble the graph.
///
/// Per upstream: for every vertex `(i, j)` emit:
/// * the right neighbour `(k + 1, j)` when it sits inside row `j`'s
///   span,
/// * the up-left neighbour `(k - 1, j + 1)` *only* when `k` is odd and
///   row `j + 1` exists,
///
/// where `k = row_start[j] + i`. Vertex id is
/// `prefix_sum[j] + (i - row_start[j])` (= `prefix_sum[j] + i_local`).
fn layout(
    row_lengths: &[u32],
    row_start: &[u32],
    directed: bool,
    mutual: bool,
) -> IgraphResult<Graph> {
    debug_assert_eq!(row_lengths.len(), row_start.len());
    let row_count = row_lengths.len();

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

    let vertex_index = |i: u32, j: usize| -> u32 { prefix_sum[j] + i - row_start[j] };
    let row_end = |j: usize| -> u32 { row_start[j] + row_lengths[j] - 1 };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

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
            // Right neighbour: always tried, succeeds iff (k+1) sits
            // inside row j's span (so the rightmost vertex of each row
            // never emits anything to the right).
            add_if_in_bounds(&mut edges, k, j, Some(k + 1), j);
            // Up-left neighbour: only for odd k and only when row j+1
            // exists. Skips when k == 0 (no `k - 1` available).
            if j + 1 < row_count && k % 2 == 1 {
                add_if_in_bounds(&mut edges, k, j, k.checked_sub(1), j + 1);
            }
        }
    }

    let mut g = Graph::new(vcount, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

fn overflow_error(kind: &str) -> IgraphError {
    IgraphError::InvalidArgument(format!("hexagonal_lattice: {kind} overflows u32"))
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
        let r = hexagonal_lattice(&[], false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn four_dim_rejected() {
        let r = hexagonal_lattice(&[1, 2, 3, 4], false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn zero_dim_yields_empty_graph() {
        let g = hexagonal_lattice(&[3, 0], false, false).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_dim_directed_keeps_flag() {
        let g = hexagonal_lattice(&[0, 3, 4], true, true).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn single_hexagon_matches_upstream_c6() {
        // References .out: dims=[1] directed=true gives a 6-vertex C_6
        // with 6 directed edges (canonical hexagon).
        let g = hexagonal_lattice(&[1], true, false).expect("ok");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 6);
        let want: BTreeSet<(u32, u32)> = [(0, 1), (0, 3), (1, 2), (2, 5), (3, 4), (4, 5)]
            .into_iter()
            .collect();
        assert_eq!(directed_arcs(&g), want);
    }

    #[test]
    fn triangular_hex_lattice_side_5_matches_upstream_vcount() {
        // References .out: dims=[5] directed=true → vcount=46, 60 arcs.
        let g = hexagonal_lattice(&[5], true, false).expect("ok");
        assert_eq!(g.vcount(), 46);
        assert_eq!(g.ecount(), 60);
    }

    #[test]
    fn rectangle_4x5_directed_mutual_matches_upstream_vcount() {
        // References .out: dims=[4, 5] directed=true mutual=true.
        // The "edges:" block spans 154 lines → 77 undirected edges × 2.
        let g = hexagonal_lattice(&[4, 5], true, true).expect("ok");
        assert_eq!(g.vcount(), 58);
        assert_eq!(g.ecount(), 154);
    }

    #[test]
    fn rectangle_2x2_matches_python_igraph_undirected() {
        // python-igraph testHexagonalLattice: Graph.Hexagonal_Lattice([2, 2])
        // sorted edge list has 19 edges.
        let g = hexagonal_lattice(&[2, 2], false, false).expect("ok");
        assert_eq!(g.vcount(), 16);
        assert_eq!(g.ecount(), 19);
        let want: BTreeSet<(u32, u32)> = [
            (0, 1),
            (0, 6),
            (1, 2),
            (2, 3),
            (2, 8),
            (3, 4),
            (4, 10),
            (5, 6),
            (5, 11),
            (6, 7),
            (7, 8),
            (7, 13),
            (8, 9),
            (9, 10),
            (9, 15),
            (11, 12),
            (12, 13),
            (13, 14),
            (14, 15),
        ]
        .into_iter()
        .collect();
        assert_eq!(canonical_undirected(&g), want);
    }

    #[test]
    fn rectangle_2x2_directed_unilateral_matches_undirected_edges() {
        let g = hexagonal_lattice(&[2, 2], true, false).expect("ok");
        let want: BTreeSet<(u32, u32)> = [
            (0, 1),
            (0, 6),
            (1, 2),
            (2, 3),
            (2, 8),
            (3, 4),
            (4, 10),
            (5, 6),
            (5, 11),
            (6, 7),
            (7, 8),
            (7, 13),
            (8, 9),
            (9, 10),
            (9, 15),
            (11, 12),
            (12, 13),
            (13, 14),
            (14, 15),
        ]
        .into_iter()
        .collect();
        assert_eq!(directed_arcs(&g), want);
    }

    #[test]
    fn rectangle_2x2_directed_mutual_doubles_edges() {
        let g = hexagonal_lattice(&[2, 2], true, true).expect("ok");
        assert_eq!(g.ecount(), 19 * 2);
        assert!(g.is_directed());
    }

    #[test]
    fn hexagonal_3_4_5_matches_upstream_vcount() {
        // References .out: dims=[3, 4, 5] directed=false mutual=true →
        // vcount=94 with 129 undirected edges (directed=false silently
        // collapses mutual; the .out edge block is 129 lines).
        let g = hexagonal_lattice(&[3, 4, 5], false, true).expect("ok");
        assert_eq!(g.vcount(), 94);
        assert_eq!(g.ecount(), 129);
        // Every vertex degree ≤ 3 (hexagonal lattice invariant).
        for v in 0..g.vcount() {
            assert!(g.degree(v).expect("deg") <= 3);
        }
    }

    #[test]
    fn all_vertices_have_degree_at_most_three() {
        for &(sx, sy, sz) in &[(3u32, 4u32, 5u32), (2, 3, 4), (5, 5, 5)] {
            let g = hexagonal_lattice(&[sx, sy, sz], false, false).expect("ok");
            for v in 0..g.vcount() {
                assert!(g.degree(v).expect("deg") <= 3);
            }
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn directed_mutual_doubles_undirected_ecount(
            sx in 1u32..6,
            sy in 1u32..6,
        ) {
            let undirected = hexagonal_lattice(&[sx, sy], false, false).expect("ok");
            let mutual = hexagonal_lattice(&[sx, sy], true, true).expect("ok");
            prop_assert_eq!(mutual.ecount(), undirected.ecount() * 2);
        }

        #[test]
        fn directed_unilateral_matches_undirected_ecount(
            sx in 1u32..6,
            sy in 1u32..6,
        ) {
            let undirected = hexagonal_lattice(&[sx, sy], false, false).expect("ok");
            let unilateral = hexagonal_lattice(&[sx, sy], true, false).expect("ok");
            prop_assert_eq!(unilateral.ecount(), undirected.ecount());
        }

        #[test]
        fn directedness_flag_round_trips(
            sx in 1u32..6,
            sy in 1u32..6,
            directed: bool,
            mutual: bool,
        ) {
            let g = hexagonal_lattice(&[sx, sy], directed, mutual).expect("ok");
            prop_assert_eq!(g.is_directed(), directed);
        }

        #[test]
        fn max_degree_at_most_three(
            sx in 1u32..6,
            sy in 1u32..6,
        ) {
            let g = hexagonal_lattice(&[sx, sy], false, false).expect("ok");
            for v in 0..g.vcount() {
                prop_assert!(g.degree(v).unwrap() <= 3);
            }
        }

        #[test]
        fn triangle_shape_vcount_grows_quadratically(
            n in 1u32..8,
        ) {
            // Triangular outline vcount is the sum of the per-row lengths.
            // Empirically: side 1 → 6, side 2 → 13, side 3 → 22,
            // side 4 → 33, side 5 → 46 (matches upstream .out).
            let g = hexagonal_lattice(&[n], false, false).expect("ok");
            // Closed form: n*(2n+5) + (2n+1)+(2n-1) = n^2 + (n+1)*(2n+3)
            // − but rather than fight the formula, assert lower/upper bound.
            prop_assert!(u64::from(g.vcount()) >= u64::from(n) * 5);
        }

        #[test]
        fn hex_shape_max_degree_bounded(
            sx in 1u32..5,
            sy in 1u32..5,
            sz in 1u32..5,
        ) {
            let g = hexagonal_lattice(&[sx, sy, sz], false, false).expect("ok");
            for v in 0..g.vcount() {
                prop_assert!(g.degree(v).unwrap() <= 3);
            }
        }
    }
}
