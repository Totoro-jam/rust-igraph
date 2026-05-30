//! k-nearest-neighbor graph of a spatial point set (ALGO-GEO-005).
//!
//! Counterpart of `igraph_nearest_neighbor_graph()` from
//! `references/igraph/src/spatial/nearest_neighbor.cpp:144`.
//!
//! Each point is joined to at most `k` of its nearest *other* points, subject
//! to an optional `cutoff` radius. The result is a directed graph where every
//! arc `i → j` means "`j` is one of `i`'s `k` nearest neighbours within
//! `cutoff`". When `directed` is `false` the directed graph is collapsed to
//! undirected, merging a reciprocal pair `i → j`, `j → i` into a single
//! undirected edge.
//!
//! The reference uses a k-d tree (nanoflann) for `O(n·log n)` queries; we use
//! a straightforward `O(n²·d)` all-pairs scan, which yields the identical edge
//! set on every tie-free configuration. The cutoff comparison is **strict**
//! (`dist < cutoff`), mirroring nanoflann's `dist < worstDist()` admission
//! test: a point lying exactly at distance `cutoff` is *not* connected. For
//! the L2 metric distances are compared squared (so the squared radius
//! `cutoff²` is the threshold), exactly as the reference squares the cutoff
//! before the search.

use crate::algorithms::spatial::edge_lengths::DistanceMetric;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the k-nearest-neighbor graph of a point set.
///
/// `points` holds one row per point with a shared, arbitrary dimensionality
/// (inferred from the first row). For each point the `k` nearest *other*
/// points within `cutoff` are connected by an outgoing arc.
///
/// * `metric` — [`DistanceMetric::Euclidean`] (L2) or
///   [`DistanceMetric::Manhattan`] (L1).
/// * `k` — maximum neighbours per point; a negative value means "no limit"
///   (every point within `cutoff`). `k == 0` produces an edgeless graph.
/// * `cutoff` — maximum connection distance; a negative value (or
///   [`f64::INFINITY`]) means "no cutoff". The comparison is strict, so a
///   point exactly at distance `cutoff` is excluded.
/// * `directed` — when `false`, the directed neighbour graph is collapsed to
///   undirected, merging reciprocal arcs into one edge.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if the points are zero-dimensional
///   (and there is at least one point).
/// - [`IgraphError::InvalidArgument`] if the rows have inconsistent
///   dimensionality.
///
/// # Examples
///
/// ```
/// use rust_igraph::{nearest_neighbor_graph, DistanceMetric};
///
/// // Three points on a line: each links to its single nearest neighbour.
/// let pts = vec![vec![0.0], vec![1.0], vec![5.0]];
/// let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 1, -1.0, true).unwrap();
/// assert!(g.is_directed());
/// assert_eq!(g.vcount(), 3);
/// // 0→1, 1→0, 2→1
/// assert_eq!(g.ecount(), 3);
/// ```
// `a`/`b` index points and `d` the dimension axis — the natural notation
// for an all-pairs geometric scan.
#[allow(clippy::many_single_char_names)]
pub fn nearest_neighbor_graph(
    points: &[Vec<f64>],
    metric: DistanceMetric,
    k: i64,
    cutoff: f64,
    directed: bool,
) -> IgraphResult<Graph> {
    let n = points.len();

    // Null graph short-circuit: with zero points the dimensionality is
    // meaningless, so a 0-dimensional empty set is not an error (matches the
    // reference, which calls igraph_empty() before the dimension check).
    if n == 0 {
        return Graph::new(0, directed);
    }

    let dim = points[0].len();
    if dim == 0 {
        return Err(IgraphError::InvalidArgument(
            "nearest_neighbor_graph: 0-dimensional points are not supported".into(),
        ));
    }
    for (i, row) in points.iter().enumerate().skip(1) {
        if row.len() != dim {
            return Err(IgraphError::InvalidArgument(format!(
                "nearest_neighbor_graph: point row {i} has dimension {} but expected {dim}",
                row.len()
            )));
        }
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument("nearest_neighbor_graph: too many points".into())
    })?;

    // Negative cutoff => no cutoff. For L2 the threshold is the squared
    // radius (distances are accumulated squared); for L1 it is the radius
    // itself. INFINITY squared stays INFINITY, so an unbounded search admits
    // every finite distance.
    let radius = if cutoff < 0.0 { f64::INFINITY } else { cutoff };
    let threshold = match metric {
        DistanceMetric::Euclidean => radius * radius,
        DistanceMetric::Manhattan => radius,
    };

    // k < 0 means "no neighbour limit": at most n - 1 others can qualify.
    let neighbor_count = if k < 0 {
        n
    } else {
        usize::try_from(k).unwrap_or(usize::MAX)
    };

    let mut directed_edges: Vec<(u32, u32)> = Vec::new();
    let mut cands: Vec<(f64, usize)> = Vec::new();

    for a in 0..n {
        cands.clear();
        for b in 0..n {
            if a == b {
                continue;
            }
            let dist = match metric {
                DistanceMetric::Euclidean => points[a]
                    .iter()
                    .zip(points[b].iter())
                    .map(|(&x, &y)| (x - y) * (x - y))
                    .sum::<f64>(),
                DistanceMetric::Manhattan => points[a]
                    .iter()
                    .zip(points[b].iter())
                    .map(|(&x, &y)| (x - y).abs())
                    .sum::<f64>(),
            };
            // Strict admission, mirroring nanoflann's `dist < worstDist()`.
            if dist < threshold {
                cands.push((dist, b));
            }
        }
        // Keep the `neighbor_count` smallest; ties broken by point index so
        // the output is deterministic (the reference leaves ties to k-d tree
        // visit order, which is unspecified).
        cands.sort_by(|p, q| p.0.total_cmp(&q.0).then(p.1.cmp(&q.1)));
        let take = neighbor_count.min(cands.len());
        let u = u32::try_from(a)
            .map_err(|_| IgraphError::Internal("nearest_neighbor_graph: vertex id overflow"))?;
        for &(_, b) in &cands[..take] {
            let v = u32::try_from(b)
                .map_err(|_| IgraphError::Internal("nearest_neighbor_graph: vertex id overflow"))?;
            directed_edges.push((u, v));
        }
    }

    if directed {
        let mut graph = Graph::new(n_u32, true)?;
        graph.add_edges(directed_edges)?;
        Ok(graph)
    } else {
        // COLLAPSE to undirected: deduplicate unordered endpoint pairs so a
        // reciprocal arc pair becomes a single edge.
        let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
        let mut undirected_edges: Vec<(u32, u32)> = Vec::new();
        for (u, v) in directed_edges {
            let key = (u.min(v), u.max(v));
            if seen.insert(key) {
                undirected_edges.push(key);
            }
        }
        let mut graph = Graph::new(n_u32, false)?;
        graph.add_edges(undirected_edges)?;
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Directed arc set as a sorted `(from, to)` vec.
    fn arc_set(g: &Graph) -> Vec<(u32, u32)> {
        let mut edges: Vec<(u32, u32)> = (0..g.ecount())
            .map(|e| {
                g.edge(u32::try_from(e).expect("edge id fits in u32"))
                    .expect("edge")
            })
            .collect();
        edges.sort_unstable();
        edges
    }

    /// Undirected edge set as a sorted `(min, max)` vec.
    fn undirected_edge_set(g: &Graph) -> Vec<(u32, u32)> {
        let mut edges: Vec<(u32, u32)> = (0..g.ecount())
            .map(|e| {
                let (u, v) = g
                    .edge(u32::try_from(e).expect("edge id fits in u32"))
                    .expect("edge");
                (u.min(v), u.max(v))
            })
            .collect();
        edges.sort_unstable();
        edges
    }

    #[test]
    fn empty_point_set() {
        let g =
            nearest_neighbor_graph(&[], DistanceMetric::Euclidean, 2, 5.0, true).expect("empty");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn single_point_no_edges() {
        let g = nearest_neighbor_graph(
            &[vec![1.0, 6.0, 4.0]],
            DistanceMetric::Euclidean,
            -1,
            100.0,
            true,
        )
        .expect("singleton");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_dimensional_error() {
        let err = nearest_neighbor_graph(&[vec![]], DistanceMetric::Euclidean, 1, 10.0, true)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err = nearest_neighbor_graph(
            &[vec![0.0, 0.0], vec![1.0]],
            DistanceMetric::Euclidean,
            1,
            10.0,
            true,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn zero_neighbors_is_edgeless() {
        let pts = vec![vec![12.0, 8.0], vec![8.0, 6.0], vec![5.0, 12.0]];
        let g =
            nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 0, 5.0, true).expect("zero k");
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_cutoff_is_edgeless() {
        let pts = vec![vec![12.0, 8.0], vec![8.0, 6.0], vec![5.0, 12.0]];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, -1, 0.0, true)
            .expect("zero cut");
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn strict_cutoff_excludes_boundary() {
        // 1d points 8 and 5 are exactly distance 3 apart; with cutoff 3 the
        // strict admission test (dist < cutoff) drops that connection.
        let pts = vec![vec![8.0], vec![5.0]];
        let g =
            nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, -1, 3.0, true).expect("strict");
        assert_eq!(g.ecount(), 0);
        // Loosening the cutoff past 3 admits both reciprocal arcs.
        let g2 =
            nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, -1, 3.5, true).expect("loose");
        assert_eq!(arc_set(&g2), vec![(0, 1), (1, 0)]);
    }

    // Authentic igraph C anchor (igraph_nearest_neighbor_graph.out, "2d 2
    // neighbors, cutoff 5", L2). No k-boundary ties, so the directed edge set
    // is unambiguous.
    #[test]
    fn c_anchor_2d_k2_cutoff5_l2() {
        let pts = vec![
            vec![12.0, 8.0],
            vec![8.0, 6.0],
            vec![5.0, 12.0],
            vec![10.0, 1.0],
            vec![12.0, 2.0],
        ];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 2, 5.0, true).expect("2d");
        assert_eq!(arc_set(&g), vec![(0, 1), (1, 0), (3, 4), (4, 3)]);
    }

    // Authentic igraph C anchor ("3d, 2 neighbors, cutoff INFINITY", L2): a
    // dense, tie-free configuration exercising the k = 2 selection.
    #[test]
    fn c_anchor_3d_k2_cutoff_inf_l2() {
        let pts = vec![
            vec![1.0, 6.0, 4.0],
            vec![6.0, 2.0, 3.0],
            vec![3.0, 6.0, 6.0],
            vec![3.0, 2.0, 2.0],
            vec![2.0, 3.0, 3.0],
        ];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 2, -1.0, true).expect("3d");
        assert_eq!(
            arc_set(&g),
            vec![
                (0, 2),
                (0, 4),
                (1, 3),
                (1, 4),
                (2, 0),
                (2, 4),
                (3, 1),
                (3, 4),
                (4, 0),
                (4, 3),
            ]
        );
    }

    // Authentic igraph C anchor ("4d 1 neighbors, cutoff INFINITY", L2).
    #[test]
    fn c_anchor_4d_k1_cutoff_inf_l2() {
        let pts = vec![
            vec![1.0, 6.0, 4.0, 4.0],
            vec![6.0, 2.0, 3.0, 3.0],
            vec![3.0, 6.0, 6.0, 5.0],
            vec![3.0, 2.0, 2.0, 3.0],
            vec![2.0, 3.0, 3.0, 4.0],
        ];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 1, -1.0, true).expect("4d");
        assert_eq!(arc_set(&g), vec![(0, 2), (1, 3), (2, 0), (3, 4), (4, 3)]);
    }

    // Authentic igraph C anchor ("2d 2 neighbors, cutoff 5", L1).
    #[test]
    fn c_anchor_2d_k2_cutoff5_l1() {
        let pts = vec![
            vec![12.0, 8.0],
            vec![8.0, 6.0],
            vec![5.0, 12.0],
            vec![10.0, 1.0],
            vec![12.0, 2.0],
        ];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Manhattan, 2, 5.0, true).expect("l1");
        assert_eq!(arc_set(&g), vec![(3, 4), (4, 3)]);
    }

    #[test]
    fn undirected_collapses_reciprocal_arcs() {
        // The 2d/k2/cutoff5/L2 anchor has two reciprocal pairs; COLLAPSE
        // merges each into a single undirected edge.
        let pts = vec![
            vec![12.0, 8.0],
            vec![8.0, 6.0],
            vec![5.0, 12.0],
            vec![10.0, 1.0],
            vec![12.0, 2.0],
        ];
        let g =
            nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, 2, 5.0, false).expect("undir");
        assert!(!g.is_directed());
        assert_eq!(undirected_edge_set(&g), vec![(0, 1), (3, 4)]);
    }

    #[test]
    fn unlimited_neighbors_infinite_cutoff_is_complete() {
        let pts = vec![vec![0.0], vec![1.0], vec![2.0]];
        let g = nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, -1, -1.0, true)
            .expect("complete");
        // Every ordered pair of distinct points: 3 * 2 = 6 arcs.
        assert_eq!(g.ecount(), 6);
    }
}
