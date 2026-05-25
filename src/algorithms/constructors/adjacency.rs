//! Dense integer adjacency-matrix constructor (ALGO-CN-029).
//!
//! Counterpart of `igraph_adjacency()` in
//! `references/igraph/src/constructors/adjacency.c:335-386`.
//!
//! Builds a [`Graph`] from a square `n × n` integer matrix whose
//! entries are **edge multiplicities** (non-negative). The shape of the
//! resulting graph (directed / undirected, which triangle drives the
//! edge count, how the diagonal becomes self-loops) is controlled by
//! two enums:
//!
//! * [`AdjacencyMode`] — one of seven dispatch flavours
//!   (`Directed`, `Undirected`, `Max`, `Min`, `Plus`, `Upper`, `Lower`).
//! * [`LoopsMode`] — how to interpret the diagonal: `NoLoops` zeroes
//!   it, `Once` treats `A(i,i)` as the loop count, `Twice` treats it
//!   as twice the loop count (must be even).
//!
//! For consistency with upstream igraph the `Twice` request is silently
//! collapsed to `Once` for the `Directed`, `Upper` and `Lower` modes —
//! the matrix only stores one copy of each loop in those layouts.
//!
//! Matrix layout: `&[&[i64]]` — a slice of equal-length rows in
//! row-major form. Every row must have the same length as the outer
//! slice; ragged input is rejected. A `0 × 0` matrix produces an empty
//! graph (matching the C semantics for an `IGRAPH_MATRIX_NULL` of
//! shape `0 × 0`).
//!
//! Time complexity: `O(|V|² + |E|)`.

// `i as VertexId` casts in the per-mode emitters are safe because the
// caller validates `nrow ≤ u32::MAX` before any index is produced.
// The double-loop traversals over the square matrix are clearer with
// explicit `i, j` indices than with iterator+enumerate chains.
#![allow(clippy::cast_possible_truncation, clippy::needless_range_loop)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// How to interpret the input matrix as an adjacency matrix.
///
/// Matches `igraph_adjacency_t` from `include/igraph_constructors.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdjacencyMode {
    /// Directed graph; `A(i, j)` is the multiplicity of the arc
    /// `i → j`.
    Directed,
    /// Undirected graph; `A(i, j)` is the multiplicity of the edge
    /// `{i, j}`. The matrix must be symmetric.
    Undirected,
    /// Undirected; the edge multiplicity of `{i, j}` is
    /// `max(A(i, j), A(j, i))`.
    Max,
    /// Undirected; the edge multiplicity of `{i, j}` is
    /// `min(A(i, j), A(j, i))`.
    Min,
    /// Undirected; the edge multiplicity of `{i, j}` is
    /// `A(i, j) + A(j, i)`.
    Plus,
    /// Undirected; only entries with `i < j` (the strict upper
    /// triangle) plus the diagonal contribute edges.
    Upper,
    /// Undirected; only entries with `i > j` (the strict lower
    /// triangle) plus the diagonal contribute edges.
    Lower,
}

/// How to convert diagonal entries into self-loops.
///
/// Matches `igraph_loops_t` from `include/igraph_constructors.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopsMode {
    /// Ignore the diagonal; the result contains no self-loops.
    NoLoops,
    /// `A(i, i)` is the number of self-loops at vertex `i`.
    Once,
    /// `A(i, i)` is **twice** the number of self-loops at vertex `i`
    /// (must be even; odd values are rejected with
    /// [`IgraphError::InvalidArgument`]). Collapses to [`Once`] for
    /// the `Directed`, `Upper` and `Lower` modes per upstream.
    ///
    /// [`Once`]: LoopsMode::Once
    Twice,
}

/// Build a graph from a dense integer adjacency matrix.
///
/// Matches `igraph_adjacency()` semantics exactly. The matrix is
/// passed as `&[&[i64]]` — outer slice indexed by row, inner slice by
/// column. The matrix must be square (each row length equal to the
/// outer length); negative entries are rejected.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the matrix is non-square (a
///   row's length differs from the row count), contains a negative
///   entry, or (`Undirected` only) is not symmetric, or
///   (`LoopsMode::Twice` with a symmetric mode) has an odd diagonal
///   entry, or the vertex count exceeds [`u32::MAX`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{adjacency, AdjacencyMode, LoopsMode};
///
/// // Directed K₃ with no loops.
/// let m: &[&[i64]] = &[&[0, 1, 1], &[1, 0, 1], &[1, 1, 0]];
/// let g = adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 6);
/// assert!(g.is_directed());
/// ```
pub fn adjacency(matrix: &[&[i64]], mode: AdjacencyMode, loops: LoopsMode) -> IgraphResult<Graph> {
    let nrow = matrix.len();

    // Square check — every row must have exactly `nrow` columns.
    if matrix.iter().any(|row| row.len() != nrow) {
        return Err(IgraphError::InvalidArgument(
            "adjacency: matrix must be square (every row has the same length as the row count)"
                .into(),
        ));
    }

    if nrow == 0 {
        // Empty graph — matches C's behaviour for a 0×0 matrix. Mode
        // still selects directedness.
        let directed = matches!(mode, AdjacencyMode::Directed);
        return Graph::new(0, directed);
    }

    // Non-negative check.
    for row in matrix {
        if let Some(&min) = row.iter().min() {
            if min < 0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "adjacency: edge counts must be non-negative, found {min}"
                )));
            }
        }
    }

    let no_of_nodes = u32::try_from(nrow).map_err(|_| {
        IgraphError::InvalidArgument("adjacency: vertex count exceeds u32::MAX".into())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    match mode {
        AdjacencyMode::Directed | AdjacencyMode::Plus => {
            emit_directed_or_plus(matrix, mode, loops, &mut edges)?;
        }
        AdjacencyMode::Max => {
            emit_max(matrix, loops, &mut edges)?;
        }
        AdjacencyMode::Undirected => {
            if !is_symmetric(matrix) {
                return Err(IgraphError::InvalidArgument(
                    "adjacency: matrix must be symmetric for AdjacencyMode::Undirected".into(),
                ));
            }
            emit_max(matrix, loops, &mut edges)?;
        }
        AdjacencyMode::Upper => {
            emit_upper(matrix, loops, &mut edges)?;
        }
        AdjacencyMode::Lower => {
            emit_lower(matrix, loops, &mut edges)?;
        }
        AdjacencyMode::Min => {
            emit_min(matrix, loops, &mut edges)?;
        }
    }

    let directed = matches!(mode, AdjacencyMode::Directed);
    let mut graph = Graph::new(no_of_nodes, directed)?;
    graph.add_edges(edges)?;
    Ok(graph)
}

/// Per-mode loop-handling collapse: `Directed`, `Upper`, `Lower` all
/// flatten `Twice` to `Once` (matches upstream lines 84, 168, 209).
fn effective_loops(mode_collapses_twice: bool, loops: LoopsMode) -> LoopsMode {
    if mode_collapses_twice && matches!(loops, LoopsMode::Twice) {
        LoopsMode::Once
    } else {
        loops
    }
}

/// Diagonal entry → number of self-loops to emit at that vertex.
/// Returns an error if `Twice` is requested and the entry is odd.
fn adjust_loop_edge_count(count: i64, loops: LoopsMode) -> IgraphResult<i64> {
    match loops {
        LoopsMode::NoLoops => Ok(0),
        LoopsMode::Twice => {
            if count & 1 == 1 {
                Err(IgraphError::InvalidArgument(
                    "adjacency: odd number found on the diagonal under LoopsMode::Twice".into(),
                ))
            } else {
                Ok(count >> 1)
            }
        }
        LoopsMode::Once => Ok(count),
    }
}

fn push_multi(edges: &mut Vec<(VertexId, VertexId)>, from: VertexId, to: VertexId, count: i64) {
    // `count` is guaranteed non-negative at the call site (checked by
    // adjust_loop_edge_count or by the non-negativity validation).
    for _ in 0..count {
        edges.push((from, to));
    }
}

fn emit_directed_or_plus(
    matrix: &[&[i64]],
    mode: AdjacencyMode,
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    let n = matrix.len();
    let collapse = matches!(mode, AdjacencyMode::Directed);
    let loops = effective_loops(collapse, loops);

    // Upstream walks j (column) outer, i (row) inner — matching
    // column-major storage. We mirror that order so the resulting
    // edge sequence matches the .out fixtures byte-for-byte.
    for j in 0..n {
        for (i, row) in matrix.iter().enumerate() {
            let mut m = row[j];
            if i == j {
                m = adjust_loop_edge_count(m, loops)?;
            }
            // Safe casts: `n ≤ u32::MAX` enforced by caller.
            let from = i as VertexId;
            let to = j as VertexId;
            push_multi(edges, from, to, m);
        }
    }
    Ok(())
}

fn emit_max(
    matrix: &[&[i64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    let n = matrix.len();
    for i in 0..n {
        let m1 = matrix[i][i];
        if m1 != 0 {
            let count = adjust_loop_edge_count(m1, loops)?;
            push_multi(edges, i as VertexId, i as VertexId, count);
        }
        for j in (i + 1)..n {
            let a = matrix[i][j];
            let b = matrix[j][i];
            let m = a.max(b);
            push_multi(edges, i as VertexId, j as VertexId, m);
        }
    }
    Ok(())
}

fn emit_min(
    matrix: &[&[i64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    let n = matrix.len();
    for i in 0..n {
        let m1 = matrix[i][i];
        if m1 != 0 {
            let count = adjust_loop_edge_count(m1, loops)?;
            push_multi(edges, i as VertexId, i as VertexId, count);
        }
        for j in (i + 1)..n {
            let a = matrix[i][j];
            let b = matrix[j][i];
            let m = a.min(b);
            push_multi(edges, i as VertexId, j as VertexId, m);
        }
    }
    Ok(())
}

fn emit_upper(
    matrix: &[&[i64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    let n = matrix.len();
    let loops = effective_loops(true, loops);
    // Outer over column j, inner over rows i < j; loops appended once
    // per outer step. Matches upstream lines 172-190.
    for j in 0..n {
        for i in 0..j {
            let m = matrix[i][j];
            push_multi(edges, i as VertexId, j as VertexId, m);
        }
        let diag = matrix[j][j];
        if diag != 0 {
            let count = adjust_loop_edge_count(diag, loops)?;
            push_multi(edges, j as VertexId, j as VertexId, count);
        }
    }
    Ok(())
}

fn emit_lower(
    matrix: &[&[i64]],
    loops: LoopsMode,
    edges: &mut Vec<(VertexId, VertexId)>,
) -> IgraphResult<()> {
    let n = matrix.len();
    let loops = effective_loops(true, loops);
    // Outer over column j, loops first then rows i > j. Matches
    // upstream lines 213-231.
    for j in 0..n {
        let diag = matrix[j][j];
        if diag != 0 {
            let count = adjust_loop_edge_count(diag, loops)?;
            push_multi(edges, j as VertexId, j as VertexId, count);
        }
        for i in (j + 1)..n {
            let m = matrix[i][j];
            push_multi(edges, i as VertexId, j as VertexId, m);
        }
    }
    Ok(())
}

fn is_symmetric(matrix: &[&[i64]]) -> bool {
    let n = matrix.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if matrix[i][j] != matrix[j][i] {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges_in_order(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    /// Canonical 3×3 from `tests/unit/igraph_adjacency.c`:
    ///   `{4, 2, 0,
    ///     3, 0, 4,
    ///     0, 5, 6}`.
    const M3: &[&[i64]] = &[&[4, 2, 0], &[3, 0, 4], &[0, 5, 6]];
    /// Symmetric 3×3 used by the `IGRAPH_ADJ_UNDIRECTED` test cases.
    const M3_SYM: &[&[i64]] = &[&[4, 2, 0], &[2, 0, 4], &[0, 4, 6]];
    /// 3×3 used by the `IGRAPH_ADJ_MIN, NO_LOOPS` test case.
    const M3_MIN_NL: &[&[i64]] = &[&[4, 2, 0], &[3, 0, 5], &[0, 4, 6]];

    #[test]
    fn empty_matrix_yields_empty_graph() {
        let m: &[&[i64]] = &[];
        let g = adjacency(m, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());

        // Same matrix but undirected → still empty undirected graph.
        let g2 = adjacency(m, AdjacencyMode::Undirected, LoopsMode::Once).unwrap();
        assert_eq!(g2.vcount(), 0);
        assert!(!g2.is_directed());
    }

    #[test]
    fn one_by_one_directed_no_loops() {
        let m: &[&[i64]] = &[&[1]];
        let g = adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn one_by_one_directed_loops_once() {
        let m: &[&[i64]] = &[&[1]];
        let g = adjacency(m, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(edges_in_order(&g), vec![(0, 0)]);
    }

    #[test]
    fn one_by_one_directed_loops_twice_collapses_to_once() {
        // Upstream: LOOPS_TWICE treated as LOOPS_ONCE for DIRECTED,
        // so `A(0,0) = 1` becomes a single self-loop (NOT halved).
        let m: &[&[i64]] = &[&[1]];
        let g = adjacency(m, AdjacencyMode::Directed, LoopsMode::Twice).unwrap();
        assert_eq!(edges_in_order(&g), vec![(0, 0)]);
    }

    #[test]
    fn three_by_three_directed_no_loops_matches_fixture() {
        let g = adjacency(M3, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.vcount(), 3);
        // Expected edge multi-set (from .out file, lines 31-45):
        // 0->1 ×2, 1->0 ×3, 1->2 ×4, 2->1 ×5.
        assert_eq!(g.ecount(), 14);
        let edges = edges_in_order(&g);
        let count = |from: u32, to: u32| edges.iter().filter(|&&e| e == (from, to)).count();
        assert_eq!(count(0, 1), 2);
        assert_eq!(count(1, 0), 3);
        assert_eq!(count(1, 2), 4);
        assert_eq!(count(2, 1), 5);
    }

    #[test]
    fn three_by_three_directed_loops_once_matches_fixture() {
        let g = adjacency(M3, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        // Loops: 0->0 ×4, 2->2 ×6 (diagonal entries 4, 0, 6).
        assert_eq!(g.ecount(), 14 + 4 + 6);
        let edges = edges_in_order(&g);
        let count = |from: u32, to: u32| edges.iter().filter(|&&e| e == (from, to)).count();
        assert_eq!(count(0, 0), 4);
        assert_eq!(count(2, 2), 6);
    }

    #[test]
    fn three_by_three_directed_loops_twice_equals_loops_once() {
        // Collapse behaviour: LOOPS_TWICE → LOOPS_ONCE for DIRECTED.
        let g_once = adjacency(M3, AdjacencyMode::Directed, LoopsMode::Once).unwrap();
        let g_twice = adjacency(M3, AdjacencyMode::Directed, LoopsMode::Twice).unwrap();
        assert_eq!(edges_in_order(&g_once), edges_in_order(&g_twice));
    }

    #[test]
    fn three_by_three_undirected_no_loops() {
        let g = adjacency(M3_SYM, AdjacencyMode::Undirected, LoopsMode::NoLoops).unwrap();
        assert!(!g.is_directed());
        // Off-diagonals: 0-1 ×2, 1-2 ×4.
        assert_eq!(g.ecount(), 2 + 4);
    }

    #[test]
    fn three_by_three_undirected_loops_twice_halves_diagonal() {
        // Diagonal {4, 0, 6} under TWICE → 2, 0, 3 loops.
        let g = adjacency(M3_SYM, AdjacencyMode::Undirected, LoopsMode::Twice).unwrap();
        let edges = edges_in_order(&g);
        let count = |from: u32, to: u32| edges.iter().filter(|&&e| e == (from, to)).count();
        assert_eq!(count(0, 0), 2);
        assert_eq!(count(2, 2), 3);
    }

    #[test]
    fn three_by_three_undirected_rejects_nonsymmetric() {
        let err = adjacency(M3, AdjacencyMode::Undirected, LoopsMode::Once)
            .expect_err("non-symmetric must error");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn three_by_three_max_no_loops_matches_fixture() {
        let g = adjacency(M3, AdjacencyMode::Max, LoopsMode::NoLoops).unwrap();
        // For pair (i,j) edge count = max(M[i,j], M[j,i]):
        // (0,1): max(2, 3) = 3; (0,2): max(0, 0) = 0; (1,2): max(4, 5) = 5.
        assert_eq!(g.ecount(), 3 + 5);
    }

    #[test]
    fn three_by_three_min_loops_once_matches_fixture() {
        // .out shows the M3 fixture under MIN+LOOPS_ONCE:
        // diag {4, 0, 6} → 4 + 6 = 10 loops.
        // off-diag (0,1) min(2,3)=2, (0,2) min(0,0)=0, (1,2) min(4,5)=4.
        let g = adjacency(M3, AdjacencyMode::Min, LoopsMode::Once).unwrap();
        assert_eq!(g.ecount(), 4 + 6 + 2 + 4);
    }

    #[test]
    fn three_by_three_min_no_loops_matches_fixture() {
        // .out fixture uses a different matrix for MIN/NO_LOOPS:
        // pair (0,1): min(2,3)=2; (1,2): min(5,4)=4; (0,2): min(0,0)=0.
        let g = adjacency(M3_MIN_NL, AdjacencyMode::Min, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 2 + 4);
    }

    #[test]
    fn three_by_three_plus_no_loops_matches_fixture() {
        // PLUS: A(i,j)+A(j,i) per pair. Diagonal ignored under NoLoops.
        // (0,1): 2+3=5; (0,2): 0+0=0; (1,2): 4+5=9.
        let g = adjacency(M3, AdjacencyMode::Plus, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 5 + 9);
    }

    #[test]
    fn three_by_three_upper_no_loops_matches_fixture() {
        // Upper triangle (i<j): (0,1)=2, (0,2)=0, (1,2)=4.
        let g = adjacency(M3, AdjacencyMode::Upper, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 2 + 4);
    }

    #[test]
    fn three_by_three_upper_loops_twice_collapses_to_once() {
        // For UPPER, Twice collapses to Once → diag {4, 0, 6} = 10 loops.
        let g = adjacency(M3, AdjacencyMode::Upper, LoopsMode::Twice).unwrap();
        assert_eq!(g.ecount(), 2 + 4 + 4 + 6);
    }

    #[test]
    fn three_by_three_lower_no_loops_matches_fixture() {
        // Lower triangle (i>j): (1,0)=3, (2,0)=0, (2,1)=5.
        let g = adjacency(M3, AdjacencyMode::Lower, LoopsMode::NoLoops).unwrap();
        assert_eq!(g.ecount(), 3 + 5);
    }

    #[test]
    fn rejects_non_square_matrix() {
        let m: &[&[i64]] = &[&[1, 2, 0]];
        let err = adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops)
            .expect_err("non-square must error");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_negative_entry() {
        let m: &[&[i64]] = &[&[1, 2, 0], &[-3, 0, 4], &[0, 5, 6]];
        let err = adjacency(m, AdjacencyMode::Directed, LoopsMode::NoLoops)
            .expect_err("negative entry must error");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_odd_diagonal_under_loops_twice() {
        // Under TWICE for a symmetric mode (UNDIRECTED), diagonal must
        // be even. Here {1, 0, 6} is symmetric+positive but A(0,0)=1
        // is odd.
        let m: &[&[i64]] = &[&[1, 2, 0], &[2, 0, 4], &[0, 4, 6]];
        let err = adjacency(m, AdjacencyMode::Undirected, LoopsMode::Twice)
            .expect_err("odd diagonal under Twice must error");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random square non-negative matrix `n × n` with
    /// `1 ≤ n ≤ 6` and entries in `[0, 4]`.
    fn small_square_matrix() -> impl Strategy<Value = Vec<Vec<i64>>> {
        (1usize..=6).prop_flat_map(|n| prop::collection::vec(prop::collection::vec(0i64..=4, n), n))
    }

    /// View a `Vec<Vec<i64>>` as `&[&[i64]]`.
    fn as_slice(m: &[Vec<i64>]) -> Vec<&[i64]> {
        m.iter().map(|r| r.as_slice()).collect()
    }

    /// Symmetrise a matrix into `(A + A^T) / 2 — wait, we need integer.
    /// Just enforce `A(j,i) = A(i,j)` from the upper triangle.
    fn symmetrise(m: &mut [Vec<i64>]) {
        let n = m.len();
        for i in 0..n {
            for j in 0..i {
                m[i][j] = m[j][i];
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// DIRECTED with NoLoops: edge count equals sum of off-diagonal entries.
        #[test]
        fn directed_no_loops_edge_count_matches(m in small_square_matrix()) {
            let view = as_slice(&m);
            let g = adjacency(&view, AdjacencyMode::Directed, LoopsMode::NoLoops).unwrap();
            let expected: i64 = (0..m.len())
                .flat_map(|i| (0..m.len()).map(move |j| (i, j)))
                .filter(|(i, j)| i != j)
                .map(|(i, j)| m[i][j])
                .sum();
            prop_assert_eq!(g.ecount() as i64, expected);
            prop_assert!(g.is_directed());
        }

        /// PLUS with NoLoops: edge count equals sum over i<j of A(i,j)+A(j,i).
        #[test]
        fn plus_no_loops_edge_count_matches(m in small_square_matrix()) {
            let view = as_slice(&m);
            let g = adjacency(&view, AdjacencyMode::Plus, LoopsMode::NoLoops).unwrap();
            let n = m.len();
            let expected: i64 = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .map(|(i, j)| m[i][j] + m[j][i])
                .sum();
            prop_assert_eq!(g.ecount() as i64, expected);
            prop_assert!(!g.is_directed());
        }

        /// MAX with NoLoops: edge count equals sum over i<j of max(A(i,j), A(j,i)).
        #[test]
        fn max_no_loops_edge_count_matches(m in small_square_matrix()) {
            let view = as_slice(&m);
            let g = adjacency(&view, AdjacencyMode::Max, LoopsMode::NoLoops).unwrap();
            let n = m.len();
            let expected: i64 = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .map(|(i, j)| m[i][j].max(m[j][i]))
                .sum();
            prop_assert_eq!(g.ecount() as i64, expected);
        }

        /// MIN with NoLoops: edge count equals sum over i<j of min(A(i,j), A(j,i)).
        #[test]
        fn min_no_loops_edge_count_matches(m in small_square_matrix()) {
            let view = as_slice(&m);
            let g = adjacency(&view, AdjacencyMode::Min, LoopsMode::NoLoops).unwrap();
            let n = m.len();
            let expected: i64 = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .map(|(i, j)| m[i][j].min(m[j][i]))
                .sum();
            prop_assert_eq!(g.ecount() as i64, expected);
        }

        /// UNDIRECTED on a symmetric matrix == MAX on the same matrix
        /// (in edge count and edge multi-set, since both go through emit_max).
        #[test]
        fn undirected_equals_max_on_symmetric(mut m in small_square_matrix()) {
            symmetrise(&mut m);
            let view = as_slice(&m);
            let g_und = adjacency(&view, AdjacencyMode::Undirected, LoopsMode::Once).unwrap();
            let g_max = adjacency(&view, AdjacencyMode::Max, LoopsMode::Once).unwrap();
            prop_assert_eq!(g_und.ecount(), g_max.ecount());
        }

        /// UPPER + LOWER edges on the same matrix together cover every
        /// non-diagonal A(i,j): edge_count(upper, NoLoops) = sum over
        /// i<j of A(i,j); edge_count(lower, NoLoops) = sum over i>j.
        #[test]
        fn upper_lower_partition_off_diagonal(m in small_square_matrix()) {
            let view = as_slice(&m);
            let g_up = adjacency(&view, AdjacencyMode::Upper, LoopsMode::NoLoops).unwrap();
            let g_lo = adjacency(&view, AdjacencyMode::Lower, LoopsMode::NoLoops).unwrap();
            let n = m.len();
            let total_offdiag: i64 = (0..n)
                .flat_map(|i| (0..n).map(move |j| (i, j)))
                .filter(|(i, j)| i != j)
                .map(|(i, j)| m[i][j])
                .sum();
            prop_assert_eq!((g_up.ecount() + g_lo.ecount()) as i64, total_offdiag);
        }
    }
}
