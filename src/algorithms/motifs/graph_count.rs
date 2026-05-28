//! Graph count (ALGO-MO-003) — number of non-isomorphic graphs.
//!
//! Counterpart of `igraph_graph_count` in
//! `references/igraph/src/isomorphism/isoclasses.c:2920`.
//!
//! Returns the number of unlabelled simple graphs on n vertices.
//! Used with isoclass and motif-finder functions.
//!
//! OEIS A000088 (undirected), A000273 (directed).

use crate::core::{IgraphError, IgraphResult};

/// Undirected graph counts: number of non-isomorphic simple graphs on n vertices.
/// Index 0..=14 (OEIS A000088).
const UNDIRECTED_GRAPH_COUNTS: &[u64] = &[
    1,
    1,
    2,
    4,
    11,
    34,
    156,
    1_044,
    12_346,
    274_668,
    12_005_168,
    1_018_997_864,
    165_091_172_592,
    50_502_031_367_952,
    29_054_155_657_235_488,
];

/// Directed graph counts: number of non-isomorphic directed simple graphs on n vertices.
/// Index 0..=9 (OEIS A000273).
const DIRECTED_GRAPH_COUNTS: &[u64] = &[
    1,
    1,
    3,
    16,
    218,
    9_608,
    1_540_944,
    882_033_440,
    1_793_359_192_848,
    13_027_956_824_399_552,
];

/// Return the number of non-isomorphic simple graphs on `n` vertices.
///
/// For undirected graphs, supports `n` up to 14.
/// For directed graphs, supports `n` up to 9.
///
/// # Arguments
///
/// * `n` — number of vertices.
/// * `directed` — whether to count directed graphs.
///
/// # Errors
///
/// Returns an error if `n` is too large for the lookup table.
///
/// # Examples
///
/// ```
/// use rust_igraph::graph_count;
///
/// // 2 non-isomorphic undirected graphs on 2 vertices: empty and K2
/// assert_eq!(graph_count(2, false).unwrap(), 2);
/// // 3 non-isomorphic directed graphs on 2 vertices
/// assert_eq!(graph_count(2, true).unwrap(), 3);
/// // 4 undirected graphs on 3 vertices
/// assert_eq!(graph_count(3, false).unwrap(), 4);
/// ```
pub fn graph_count(n: u32, directed: bool) -> IgraphResult<u64> {
    let idx = n as usize;
    if directed {
        if idx >= DIRECTED_GRAPH_COUNTS.len() {
            return Err(IgraphError::InvalidArgument(format!(
                "graph_count: n={n} too large for directed graphs (max {})",
                DIRECTED_GRAPH_COUNTS.len() - 1
            )));
        }
        Ok(DIRECTED_GRAPH_COUNTS[idx])
    } else {
        if idx >= UNDIRECTED_GRAPH_COUNTS.len() {
            return Err(IgraphError::InvalidArgument(format!(
                "graph_count: n={n} too large for undirected graphs (max {})",
                UNDIRECTED_GRAPH_COUNTS.len() - 1
            )));
        }
        Ok(UNDIRECTED_GRAPH_COUNTS[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_count_undirected_basic() {
        assert_eq!(graph_count(0, false).unwrap(), 1);
        assert_eq!(graph_count(1, false).unwrap(), 1);
        assert_eq!(graph_count(2, false).unwrap(), 2);
        assert_eq!(graph_count(3, false).unwrap(), 4);
        assert_eq!(graph_count(4, false).unwrap(), 11);
        assert_eq!(graph_count(5, false).unwrap(), 34);
    }

    #[test]
    fn graph_count_directed_basic() {
        assert_eq!(graph_count(0, true).unwrap(), 1);
        assert_eq!(graph_count(1, true).unwrap(), 1);
        assert_eq!(graph_count(2, true).unwrap(), 3);
        assert_eq!(graph_count(3, true).unwrap(), 16);
        assert_eq!(graph_count(4, true).unwrap(), 218);
    }

    #[test]
    fn graph_count_undirected_max() {
        assert!(graph_count(14, false).is_ok());
        assert!(graph_count(15, false).is_err());
    }

    #[test]
    fn graph_count_directed_max() {
        assert!(graph_count(9, true).is_ok());
        assert!(graph_count(10, true).is_err());
    }
}
