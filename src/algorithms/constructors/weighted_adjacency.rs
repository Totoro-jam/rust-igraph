//! Dense weighted adjacency-matrix constructor (ALGO-CN-030).
//!
//! Counterpart of `igraph_weighted_adjacency()` in
//! `references/igraph/src/constructors/adjacency.c:785-878`.
//!
//! Builds a [`Graph`] and a parallel weight vector from a square
//! `n × n` real-valued matrix. Differences from the integer
//! [`adjacency`](crate::adjacency) variant:
//!
//! * Entries are **edge weights**, not multiplicities: every non-zero
//!   cell becomes exactly **one** edge whose weight equals the cell
//!   value (the integer variant duplicates edges by cell value).
//! * Negative weights are **allowed**.
//! * `NaN` propagation rules apply to [`AdjacencyMode::Max`] and
//!   [`AdjacencyMode::Min`]; [`AdjacencyMode::Undirected`] uses a
//!   NaN-tolerant symmetry check (`NaN == NaN` on the same diagonal
//!   reflection is treated as symmetric, matching upstream's
//!   `! (isnan(M1) && isnan(M2))` predicate).
//!
//! The shape of the resulting graph (directed / undirected, which
//! triangle drives the edge count, how the diagonal becomes self-loops)
//! is controlled by the two enums that [`adjacency`](crate::adjacency)
//! exposes — [`AdjacencyMode`] and [`LoopsMode`]. They are re-exported
//! here for convenience.
//!
//! For consistency with upstream igraph the `Twice` request is silently
//! collapsed to `Once` for the `Directed`, `Upper` and `Lower` modes —
//! the matrix only stores one copy of each loop in those layouts (see
//! the issue cited in the upstream source, igraph#2501).
//!
//! For [`LoopsMode::NoLoops`], the diagonal contribution is zeroed and
//! consequently skipped by the post-adjust `!= 0.0` filter so no
//! self-loops are emitted.
//!
//! Matrix layout: `&[&[f64]]` — a slice of equal-length rows in
//! row-major form. Every row must have the same length as the outer
//! slice; ragged input is rejected. A `0 × 0` matrix produces an empty
//! graph and an empty weight vector (matches the C semantics for an
//! `IGRAPH_MATRIX_NULL` of shape `0 × 0`).
//!
//! Time complexity: `O(|V|² + |E|)`.

// The double-loop traversals over the square matrix are clearer with
// explicit `i, j` indices than with iterator+enumerate chains, and the
// `i as VertexId` casts are safe because the caller validates
// `nrow ≤ u32::MAX` before any index is produced.
#![allow(clippy::cast_possible_truncation, clippy::needless_range_loop)]

use crate::algorithms::constructors::adjacency::{AdjacencyMode, LoopsMode};
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build a graph and a per-edge weight vector from a dense real-valued
/// adjacency matrix.
///
/// Matches `igraph_weighted_adjacency()` semantics exactly. The matrix
/// is passed as `&[&[f64]]` — outer slice indexed by row, inner slice
/// by column. The matrix must be square (each row length equal to the
/// outer length). Negative entries are allowed.
///
/// Returns `(graph, weights)` where `weights[k]` is the weight of the
/// `k`-th emitted edge (`graph.edge(k)`).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the matrix is non-square (a
///   row's length differs from the row count), or
///   ([`AdjacencyMode::Undirected`] only) is not NaN-tolerantly
///   symmetric, or the vertex count exceeds [`u32::MAX`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{weighted_adjacency, AdjacencyMode, LoopsMode};
///
/// // Directed K₃ with no loops, weights = 0.5 on every arc.
/// let m: &[&[f64]] = &[&[0.0, 0.5, 0.5], &[0.5, 0.0, 0.5], &[0.5, 0.5, 0.0]];
/// let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 6);
/// assert!(g.is_directed());
/// assert_eq!(w.len(), 6);
/// for &x in &w {
///     assert!((x - 0.5).abs() < 1e-12);
/// }
/// ```
pub fn weighted_adjacency(
    matrix: &[&[f64]],
    mode: AdjacencyMode,
    loops: LoopsMode,
) -> IgraphResult<(Graph, Vec<f64>)> {
    let nrow = matrix.len();

    // Square check — every row must have exactly `nrow` columns.
    if matrix.iter().any(|row| row.len() != nrow) {
        return Err(IgraphError::InvalidArgument(
            "weighted_adjacency: matrix must be square (every row has the same length as the row count)"
                .into(),
        ));
    }

    if nrow == 0 {
        let directed = matches!(mode, AdjacencyMode::Directed);
        return Ok((Graph::new(0, directed)?, Vec::new()));
    }

    let no_of_nodes = u32::try_from(nrow).map_err(|_| {
        IgraphError::InvalidArgument("weighted_adjacency: vertex count exceeds u32::MAX".into())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();

    match mode {
        AdjacencyMode::Directed => {
            emit_directed(matrix, loops, &mut edges, &mut weights);
        }
        AdjacencyMode::Max => {
            emit_max(matrix, loops, &mut edges, &mut weights);
        }
        AdjacencyMode::Undirected => {
            emit_undirected(matrix, loops, &mut edges, &mut weights)?;
        }
        AdjacencyMode::Upper => {
            emit_upper(matrix, loops, &mut edges, &mut weights);
        }
        AdjacencyMode::Lower => {
            emit_lower(matrix, loops, &mut edges, &mut weights);
        }
        AdjacencyMode::Min => {
            emit_min(matrix, loops, &mut edges, &mut weights);
        }
        AdjacencyMode::Plus => {
            emit_plus(matrix, loops, &mut edges, &mut weights);
        }
    }

    let directed = matches!(mode, AdjacencyMode::Directed);
    let mut graph = Graph::new(no_of_nodes, directed)?;
    graph.add_edges(edges)?;
    Ok((graph, weights))
}

/// Per-mode `Twice → Once` collapse for the modes that store the loop
/// only once in the matrix (Directed, Upper, Lower).
fn effective_loops(mode_collapses_twice: bool, loops: LoopsMode) -> LoopsMode {
    if mode_collapses_twice && matches!(loops, LoopsMode::Twice) {
        LoopsMode::Once
    } else {
        loops
    }
}

/// Diagonal entry → emitted self-loop weight.
///
/// Mirrors `igraph_i_adjust_loop_edge_weight()`:
/// * `NoLoops` → 0.0  (caller filters out 0.0)
/// * `Twice`   → `weight / 2`
/// * `Once`    → `weight`
fn adjust_loop_weight(weight: f64, loops: LoopsMode) -> f64 {
    match loops {
        LoopsMode::NoLoops => 0.0,
        LoopsMode::Twice => weight / 2.0,
        LoopsMode::Once => weight,
    }
}

fn emit_directed(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    let loops = effective_loops(true, loops);

    // Upstream walks j (column) outer, i (row) inner; we mirror so the
    // emitted edge order matches the .out fixtures.
    for j in 0..n {
        for i in 0..n {
            let mut m = matrix[i][j];
            if m == 0.0 {
                continue;
            }
            if i == j {
                m = adjust_loop_weight(m, loops);
                if m == 0.0 {
                    continue;
                }
            }
            edges.push((i as VertexId, j as VertexId));
            weights.push(m);
        }
    }
}

fn emit_plus(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    for i in 0..n {
        if !matches!(loops, LoopsMode::NoLoops) {
            let m = matrix[i][i];
            if m != 0.0 {
                let adj = adjust_loop_weight(m, loops);
                edges.push((i as VertexId, i as VertexId));
                weights.push(adj);
            }
        }
        for j in (i + 1)..n {
            let m = matrix[i][j] + matrix[j][i];
            if m != 0.0 {
                edges.push((i as VertexId, j as VertexId));
                weights.push(m);
            }
        }
    }
}

fn emit_max(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    for i in 0..n {
        if !matches!(loops, LoopsMode::NoLoops) {
            let m1 = matrix[i][i];
            if m1 != 0.0 {
                let adj = adjust_loop_weight(m1, loops);
                edges.push((i as VertexId, i as VertexId));
                weights.push(adj);
            }
        }
        for j in (i + 1)..n {
            let mut m1 = matrix[i][j];
            let m2 = matrix[j][i];
            // Upstream rule: `if (M1 < M2 || isnan(M2)) M1 = M2;`
            // i.e. NaN on the j,i side propagates and wins.
            if m1 < m2 || m2.is_nan() {
                m1 = m2;
            }
            if m1 != 0.0 {
                edges.push((i as VertexId, j as VertexId));
                weights.push(m1);
            }
        }
    }
}

fn emit_min(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    for i in 0..n {
        if !matches!(loops, LoopsMode::NoLoops) {
            let m1 = matrix[i][i];
            if m1 != 0.0 {
                let adj = adjust_loop_weight(m1, loops);
                edges.push((i as VertexId, i as VertexId));
                weights.push(adj);
            }
        }
        for j in (i + 1)..n {
            let mut m1 = matrix[i][j];
            let m2 = matrix[j][i];
            // Upstream rule: `if (M1 > M2 || isnan(M2)) M1 = M2;`
            // i.e. NaN on the j,i side propagates and wins.
            if m1 > m2 || m2.is_nan() {
                m1 = m2;
            }
            if m1 != 0.0 {
                edges.push((i as VertexId, j as VertexId));
                weights.push(m1);
            }
        }
    }
}

fn emit_undirected(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) -> IgraphResult<()> {
    let n = matrix.len();
    // Upstream walks `for i in 0..n; for j in 0..i` — lower triangle,
    // row-major. NaN-tolerant symmetry: a pair where both sides are
    // NaN counts as symmetric (NaN == NaN is false, so a plain
    // `matrix[i][j] != matrix[j][i]` would over-reject).
    for i in 0..n {
        if !matches!(loops, LoopsMode::NoLoops) {
            let m = matrix[i][i];
            if m != 0.0 {
                let adj = adjust_loop_weight(m, loops);
                edges.push((i as VertexId, i as VertexId));
                weights.push(adj);
            }
        }
        for j in 0..i {
            let m1 = matrix[i][j];
            let m2 = matrix[j][i];
            // Bit-equality match (NaN-aware) matches upstream C's
            // `M1 != M2` semantics: NaN == NaN counts as symmetric.
            #[allow(clippy::float_cmp)]
            let asymmetric = m1 != m2 && !(m1.is_nan() && m2.is_nan());
            if asymmetric {
                return Err(IgraphError::InvalidArgument(
                    "weighted_adjacency: matrix must be symmetric for AdjacencyMode::Undirected"
                        .into(),
                ));
            }
            if m1 != 0.0 {
                // Note upstream emits (i, j) — i is the outer loop, so
                // we push the larger index first (i.e. lower triangle
                // emission order).
                edges.push((i as VertexId, j as VertexId));
                weights.push(m1);
            }
        }
    }
    Ok(())
}

fn emit_upper(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    let loops = effective_loops(true, loops);
    // Upstream walks j outer, i < j inner, then the diagonal at the
    // end of each column.
    for j in 0..n {
        for i in 0..j {
            let m = matrix[i][j];
            if m != 0.0 {
                edges.push((i as VertexId, j as VertexId));
                weights.push(m);
            }
        }
        if !matches!(loops, LoopsMode::NoLoops) {
            let m = matrix[j][j];
            if m != 0.0 {
                let adj = adjust_loop_weight(m, loops);
                edges.push((j as VertexId, j as VertexId));
                weights.push(adj);
            }
        }
    }
}

fn emit_lower(
    matrix: &[&[f64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
    weights: &mut Vec<f64>,
) {
    let n = matrix.len();
    let loops = effective_loops(true, loops);
    // Upstream walks j outer; diagonal first then i > j inner.
    for j in 0..n {
        if !matches!(loops, LoopsMode::NoLoops) {
            let m = matrix[j][j];
            if m != 0.0 {
                let adj = adjust_loop_weight(m, loops);
                edges.push((j as VertexId, j as VertexId));
                weights.push(adj);
            }
        }
        for i in (j + 1)..n {
            let m = matrix[i][j];
            if m != 0.0 {
                edges.push((i as VertexId, j as VertexId));
                weights.push(m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    fn edges_in_order(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    /// 3x3 sample used to exercise asymmetric modes.
    const M3: &[&[f64]] = &[&[2.0, 0.5, 0.0], &[1.5, 0.0, 2.0], &[0.0, 2.5, 3.0]];

    /// 3x3 symmetric sample for Undirected.
    const M3_SYM: &[&[f64]] = &[&[2.0, 0.5, 0.0], &[0.5, 0.0, 2.0], &[0.0, 2.0, 3.0]];

    #[test]
    fn empty_matrix_yields_empty_graph() {
        let m: &[&[f64]] = &[];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
        assert!(w.is_empty());

        let (g2, w2) = weighted_adjacency(m, AdjacencyMode::Undirected, LoopsMode::Once).unwrap();
        assert_eq!(g2.vcount(), 0);
        assert!(!g2.is_directed());
        assert!(w2.is_empty());
    }

    #[test]
    fn one_by_one_directed_no_loops() {
        let m: &[&[f64]] = &[&[1.25]];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(w.is_empty());
    }

    #[test]
    fn one_by_one_directed_loops_once() {
        let m: &[&[f64]] = &[&[1.25]];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        assert_eq!(edges_in_order(&g), vec![(0, 0)]);
        assert_eq!(w.len(), 1);
        assert!(approx_eq(w[0], 1.25));
    }

    #[test]
    fn one_by_one_directed_loops_twice_collapses() {
        // Twice → Once for Directed: weight passes through unhalved.
        let m: &[&[f64]] = &[&[1.25]];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::Twice).unwrap();
        assert_eq!(edges_in_order(&g), vec![(0, 0)]);
        assert!(approx_eq(w[0], 1.25));
    }

    #[test]
    fn directed_no_loops_emits_in_column_major_order() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.vcount(), 3);
        // M3 has four non-zero off-diagonal entries:
        // (1,0)=1.5, (0,1)=0.5, (2,1)=2.5, (1,2)=2.0. Column-major
        // walk: j=0 -> i=1 (1.5); j=1 -> i=0 (0.5), i=2 (2.5); j=2 ->
        // i=1 (2.0). Diagonal entries (2.0 and 3.0) skipped by
        // NoLoops.
        assert_eq!(edges_in_order(&g), vec![(1, 0), (0, 1), (2, 1), (1, 2)]);
        assert_eq!(w.len(), 4);
        assert!(approx_eq(w[0], 1.5));
        assert!(approx_eq(w[1], 0.5));
        assert!(approx_eq(w[2], 2.5));
        assert!(approx_eq(w[3], 2.0));
    }

    #[test]
    fn directed_loops_once_includes_diagonal() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        // Column-major: j=0 -> (0,0)=2.0 (loop, adjusted from 2 → 2),
        // (1,0)=1.5; j=1 -> (0,1)=0.5, (2,1)=2.5; j=2 -> (1,2)=2.0,
        // (2,2)=3.0. M[1][1]=0 stays skipped.
        assert_eq!(
            edges_in_order(&g),
            vec![(0, 0), (1, 0), (0, 1), (2, 1), (1, 2), (2, 2)]
        );
        assert_eq!(w, vec![2.0, 1.5, 0.5, 2.5, 2.0, 3.0]);
    }

    #[test]
    fn undirected_lower_triangle_walk() {
        let (g, w) =
            weighted_adjacency(M3_SYM, AdjacencyMode::Undirected, LoopsMode::Once).unwrap();
        // Upstream walks i outer, j < i inner. Diagonal first per i.
        // i=0: diag 2.0 -> (0,0)=2.0.
        // i=1: diag 0 (skip); j=0 -> M[1][0]=0.5 -> push (1,0)=0.5.
        // i=2: diag 3.0 -> (2,2)=3.0; j=0 -> 0 (skip); j=1 -> 2.0 -> (2,1)=2.0.
        // Graph canonicalises undirected edges so `from <= to`, hence
        // (1,0) → (0,1) and (2,1) → (1,2) on read-back.
        assert_eq!(edges_in_order(&g), vec![(0, 0), (0, 1), (2, 2), (1, 2)]);
        assert_eq!(w, vec![2.0, 0.5, 3.0, 2.0]);
        assert!(!g.is_directed());
    }

    #[test]
    fn undirected_rejects_asymmetric() {
        let res = weighted_adjacency(M3, AdjacencyMode::Undirected, LoopsMode::NoLoops);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn undirected_accepts_nan_symmetric() {
        // Both (i, j) and (j, i) are NaN — counts as symmetric, the
        // emit path skips them because `NaN != 0.0` is true so the
        // edge would be pushed BUT the upstream rule emits the
        // (lower-index, upper-index) pair with weight = NaN. We test
        // that the build does not error and that the weights vector
        // contains the NaN.
        let m: &[&[f64]] = &[
            &[0.0, f64::NAN, 0.0],
            &[f64::NAN, 0.0, 1.0],
            &[0.0, 1.0, 0.0],
        ];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Undirected, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 2);
        assert!(w[0].is_nan());
        assert!(approx_eq(w[1], 1.0));
    }

    #[test]
    fn undirected_rejects_one_sided_nan() {
        // One side NaN, other side numeric — NOT symmetric.
        let m: &[&[f64]] = &[&[0.0, f64::NAN, 0.0], &[1.0, 0.0, 0.0], &[0.0, 0.0, 0.0]];
        let res = weighted_adjacency(m, AdjacencyMode::Undirected, LoopsMode::NoLoops);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn max_picks_larger_of_two_sides() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Max, LoopsMode::NoLoops).unwrap();
        // For each (i,j) with i < j: M = max(M[i][j], M[j][i]).
        // (0,1): max(0.5, 1.5) = 1.5
        // (0,2): max(0.0, 0.0) = 0.0 -> skip
        // (1,2): max(2.0, 2.5) = 2.5
        assert_eq!(edges_in_order(&g), vec![(0, 1), (1, 2)]);
        assert_eq!(w, vec![1.5, 2.5]);
    }

    #[test]
    fn max_nan_propagates() {
        // (j, i) is NaN -> M2.is_nan() triggers M1 = M2 = NaN.
        let m: &[&[f64]] = &[&[0.0, 1.0], &[f64::NAN, 0.0]];
        let (_, w) = weighted_adjacency(m, AdjacencyMode::Max, LoopsMode::NoLoops).unwrap();
        // M1 = 1.0, M2 = NaN -> M1 < M2 is false but M2.is_nan() is
        // true -> M1 = NaN; then `M1 != 0.0` (NaN != 0.0 is true) so
        // we emit.
        assert_eq!(w.len(), 1);
        assert!(w[0].is_nan());
    }

    #[test]
    fn min_picks_smaller_of_two_sides() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Min, LoopsMode::NoLoops).unwrap();
        // (0,1): min(0.5, 1.5) = 0.5
        // (0,2): min(0.0, 0.0) = 0.0 -> skip
        // (1,2): min(2.0, 2.5) = 2.0
        assert_eq!(edges_in_order(&g), vec![(0, 1), (1, 2)]);
        assert_eq!(w, vec![0.5, 2.0]);
    }

    #[test]
    fn min_nan_propagates() {
        let m: &[&[f64]] = &[&[0.0, 1.0], &[f64::NAN, 0.0]];
        let (_, w) = weighted_adjacency(m, AdjacencyMode::Min, LoopsMode::NoLoops).unwrap();
        assert_eq!(w.len(), 1);
        assert!(w[0].is_nan());
    }

    #[test]
    fn plus_sums_both_sides() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Plus, LoopsMode::NoLoops).unwrap();
        // (0,1): 0.5 + 1.5 = 2.0
        // (0,2): 0.0 + 0.0 = 0.0 -> skip
        // (1,2): 2.0 + 2.5 = 4.5
        assert_eq!(edges_in_order(&g), vec![(0, 1), (1, 2)]);
        assert_eq!(w, vec![2.0, 4.5]);
    }

    #[test]
    fn plus_loops_once_includes_diagonal() {
        let (_, w) = weighted_adjacency(M3, AdjacencyMode::Plus, LoopsMode::Once).unwrap();
        // i=0: diag 2.0 (Once -> 2.0); off (0,1)=2.0, (0,2)=0 skip.
        // i=1: diag 0 skip; off (1,2)=4.5.
        // i=2: diag 3.0; no more off-diagonal.
        assert_eq!(w, vec![2.0, 2.0, 4.5, 3.0]);
    }

    #[test]
    fn plus_loops_twice_halves_diagonal() {
        let (_, w) = weighted_adjacency(M3, AdjacencyMode::Plus, LoopsMode::Twice).unwrap();
        // Diagonals: 2.0 -> 1.0 (halved), 3.0 -> 1.5 (halved).
        assert!(approx_eq(w[0], 1.0));
        assert!(approx_eq(w[3], 1.5));
    }

    #[test]
    fn upper_column_major_walk() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Upper, LoopsMode::NoLoops).unwrap();
        // j=0: no i<0 inner; diag NoLoops skip.
        // j=1: i=0 -> M[0][1]=0.5; diag 0 skip.
        // j=2: i=0 -> M[0][2]=0 skip; i=1 -> M[1][2]=2.0; diag NoLoops skip.
        assert_eq!(edges_in_order(&g), vec![(0, 1), (1, 2)]);
        assert_eq!(w, vec![0.5, 2.0]);
    }

    #[test]
    fn upper_loops_twice_collapses_to_once() {
        // Upper + Twice -> Once: diagonal NOT halved.
        let (_, w) = weighted_adjacency(M3, AdjacencyMode::Upper, LoopsMode::Twice).unwrap();
        // Same emission order as Once. Diagonals 2.0 and 3.0 emitted
        // un-halved.
        // j=0: diag 2.0
        // j=1: i=0 -> 0.5; diag 0 skip
        // j=2: i=0 -> 0 skip; i=1 -> 2.0; diag 3.0
        assert_eq!(w, vec![2.0, 0.5, 2.0, 3.0]);
    }

    #[test]
    fn lower_column_major_walk() {
        let (g, w) = weighted_adjacency(M3, AdjacencyMode::Lower, LoopsMode::NoLoops).unwrap();
        // j=0: diag skip; i=1 -> M[1][0]=1.5; i=2 -> M[2][0]=0 skip.
        // j=1: diag skip; i=2 -> M[2][1]=2.5.
        // j=2: diag skip; no i>2.
        // Lower yields an undirected graph, so Graph canonicalises
        // each (i, j) with i > j to (j, i) on read-back.
        assert_eq!(edges_in_order(&g), vec![(0, 1), (1, 2)]);
        assert_eq!(w, vec![1.5, 2.5]);
    }

    #[test]
    fn lower_loops_twice_collapses_to_once() {
        let (_, w) = weighted_adjacency(M3, AdjacencyMode::Lower, LoopsMode::Twice).unwrap();
        // Diagonals 2.0 and 3.0 emitted un-halved (Twice -> Once
        // collapse for Lower).
        // j=0: diag 2.0; i=1 -> 1.5; i=2 -> 0 skip.
        // j=1: diag 0 skip; i=2 -> 2.5.
        // j=2: diag 3.0; no more inner.
        assert_eq!(w, vec![2.0, 1.5, 2.5, 3.0]);
    }

    #[test]
    fn negative_weights_pass_through() {
        let m: &[&[f64]] = &[&[0.0, -1.0], &[-2.0, 0.0]];
        let (g, w) = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 2);
        // Column-major: j=0 -> (1,0)=-2.0; j=1 -> (0,1)=-1.0.
        assert_eq!(w, vec![-2.0, -1.0]);
    }

    #[test]
    fn ragged_matrix_rejected() {
        let m: &[&[f64]] = &[&[1.0, 2.0], &[3.0]];
        let res = weighted_adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn weights_length_equals_ecount_always() {
        for mode in [
            AdjacencyMode::Directed,
            AdjacencyMode::Max,
            AdjacencyMode::Min,
            AdjacencyMode::Plus,
            AdjacencyMode::Upper,
            AdjacencyMode::Lower,
        ] {
            for loops in [LoopsMode::NoLoops, LoopsMode::Once, LoopsMode::Twice] {
                let (g, w) = weighted_adjacency(M3, mode, loops).unwrap();
                assert_eq!(g.ecount(), w.len(), "mode={mode:?}, loops={loops:?}");
            }
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_dense_matrix(max_n: usize) -> impl Strategy<Value = Vec<Vec<f64>>> {
        (0..=max_n).prop_flat_map(|n| {
            // Each cell in [-2.0, 2.0], discrete steps to keep
            // arithmetic exact.
            let cell = (-4..=4i32).prop_map(|x| f64::from(x) * 0.5);
            prop::collection::vec(prop::collection::vec(cell, n), n)
        })
    }

    fn rows_view(matrix: &[Vec<f64>]) -> Vec<&[f64]> {
        matrix.iter().map(Vec::as_slice).collect()
    }

    proptest! {
        #[test]
        fn vcount_equals_matrix_side(matrix in arb_dense_matrix(6)) {
            let rows = rows_view(&matrix);
            for mode in [
                AdjacencyMode::Directed,
                AdjacencyMode::Max,
                AdjacencyMode::Min,
                AdjacencyMode::Plus,
                AdjacencyMode::Upper,
                AdjacencyMode::Lower,
            ] {
                for loops in [LoopsMode::NoLoops, LoopsMode::Once, LoopsMode::Twice] {
                    let (g, _) = weighted_adjacency(&rows, mode, loops).unwrap();
                    prop_assert_eq!(g.vcount() as usize, matrix.len());
                }
            }
        }

        #[test]
        fn weights_length_equals_ecount(matrix in arb_dense_matrix(6)) {
            let rows = rows_view(&matrix);
            for mode in [
                AdjacencyMode::Directed,
                AdjacencyMode::Max,
                AdjacencyMode::Min,
                AdjacencyMode::Plus,
                AdjacencyMode::Upper,
                AdjacencyMode::Lower,
            ] {
                for loops in [LoopsMode::NoLoops, LoopsMode::Once, LoopsMode::Twice] {
                    let (g, w) = weighted_adjacency(&rows, mode, loops).unwrap();
                    prop_assert_eq!(g.ecount(), w.len());
                }
            }
        }

        #[test]
        fn upper_lower_twice_equals_once(matrix in arb_dense_matrix(5)) {
            // Twice collapses to Once for Upper / Lower / Directed
            // -> same edge multiset & same weights.
            let rows = rows_view(&matrix);
            for mode in [
                AdjacencyMode::Upper,
                AdjacencyMode::Lower,
                AdjacencyMode::Directed,
            ] {
                let (g_once, w_once) =
                    weighted_adjacency(&rows, mode, LoopsMode::Once).unwrap();
                let (g_twice, w_twice) =
                    weighted_adjacency(&rows, mode, LoopsMode::Twice).unwrap();
                prop_assert_eq!(g_once.ecount(), g_twice.ecount());
                prop_assert_eq!(w_once, w_twice);
            }
        }

        #[test]
        fn plus_twice_halves_diagonal(matrix in arb_dense_matrix(5)) {
            // Plus is a non-collapsing mode -> Twice halves the
            // diagonal weights.
            let rows = rows_view(&matrix);
            let (g_once, w_once) =
                weighted_adjacency(&rows, AdjacencyMode::Plus, LoopsMode::Once).unwrap();
            let (g_twice, w_twice) =
                weighted_adjacency(&rows, AdjacencyMode::Plus, LoopsMode::Twice).unwrap();
            prop_assert_eq!(g_once.ecount(), g_twice.ecount());
            // Walk emitted edges in the same order; loop edges should
            // be halved, off-diagonal weights identical.
            for k in 0..g_once.ecount() {
                let (u, v) = g_once.edge(u32::try_from(k).unwrap()).unwrap();
                if u == v {
                    let diff = w_once[k] - 2.0 * w_twice[k];
                    prop_assert!(diff.abs() < 1e-9);
                } else {
                    let diff = w_once[k] - w_twice[k];
                    prop_assert!(diff.abs() < 1e-12);
                }
            }
        }

        #[test]
        fn directed_no_loops_excludes_diagonal(matrix in arb_dense_matrix(5)) {
            let rows = rows_view(&matrix);
            let (g, _) =
                weighted_adjacency(&rows, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
            for k in 0..g.ecount() {
                let (u, v) = g.edge(u32::try_from(k).unwrap()).unwrap();
                prop_assert!(u != v, "self-loop emitted under NoLoops");
            }
        }

        #[test]
        fn max_min_bounded_by_off_diagonal_pairs(matrix in arb_dense_matrix(5)) {
            // The only universally true MAX/MIN invariant for signed
            // weights is the off-diagonal cap: each pair contributes
            // at most one edge under NoLoops, so ecount <= n*(n-1)/2.
            // (Element-wise domination breaks with mixed signs: e.g.
            // pair (-0.5, 0) -> MAX=0 skipped, MIN=-0.5 emitted, so
            // MIN can emit more edges than MAX.)
            let rows = rows_view(&matrix);
            let n = matrix.len();
            let cap = n * n.saturating_sub(1) / 2;
            let (g_max, _) =
                weighted_adjacency(&rows, AdjacencyMode::Max, LoopsMode::NoLoops).unwrap();
            let (g_min, _) =
                weighted_adjacency(&rows, AdjacencyMode::Min, LoopsMode::NoLoops).unwrap();
            prop_assert!(g_max.ecount() <= cap);
            prop_assert!(g_min.ecount() <= cap);
        }
    }
}
