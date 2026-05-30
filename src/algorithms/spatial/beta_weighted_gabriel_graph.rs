//! β-weighted Gabriel graph of a spatial point set (ALGO-GEO-008).
//!
//! Counterpart of `igraph_beta_weighted_gabriel_graph()` from
//! `references/igraph/src/spatial/beta_skeleton.cpp:651`.
//!
//! This builds the [Gabriel graph](crate::gabriel_graph) and, for every edge,
//! computes the **threshold β** at which that edge ceases to belong to the
//! lune-based β-skeleton: the smallest β for which some other point first
//! enters the edge's lune. Edges that survive for arbitrarily large β (no
//! point ever enters) get weight `f64::INFINITY`. A `max_beta` cutoff caps the
//! search — any edge whose threshold is at or above `max_beta` is reported as
//! `INFINITY` instead of its exact (large) value, which bounds the work.
//!
//! ## Equivalence to the reference
//!
//! Upstream seeds the candidate edges from the Delaunay triangulation (a
//! superset of the Gabriel graph) and prunes the per-edge point scan with a
//! k-d tree. The k-d tree's `dist < max_radius` cutoff is a pure search bound,
//! never a correctness filter: for the edge `AB` with `|AB|² = ab²`, a point
//! `P` first blocks the edge at parameter `β₀ = pointBeta(P)`, and one can show
//! (placing the midpoint at the origin, `A = (−h, 0)`, `B = (h, 0)`,
//! `h² = ab²/4`) that
//!
//! ```text
//!   dist²(P, midpoint) ≤ luneHalfHeight²(β₀)  ⟺  x·(x² + y² − h²) ≤ 0,
//! ```
//!
//! which holds strictly for every point that yields a finite `β₀ ≥ 1` (those
//! lie strictly outside the diametral ball, where `x² + y² > h²`). Points
//! inside that ball produce `β₀ = 0`, marking a non-Gabriel edge. Hence
//! scanning **all** `O(n²)` candidate pairs and **all** other points, with no
//! k-d tree and no Delaunay seed, yields exactly the reference's surviving
//! edges and identical weights — the same output-equivalence that the plain
//! [`gabriel_graph`](crate::gabriel_graph) relies on. Works in any dimension
//! (the formulas use only squared distances).

use crate::core::{Graph, IgraphError, IgraphResult};

/// β-threshold tolerance, mirroring the C reference's
/// `#define TOLERANCE (128 * DBL_EPSILON)`. A computed β below `1 + TOLERANCE`
/// is treated as `0` (the blocking point is inside the closed Gabriel ball,
/// so the edge is not in the Gabriel graph), matching the reference's
/// `beta < 1 + tol ? 0 : beta`.
const TOLERANCE: f64 = 128.0 * f64::EPSILON;

/// A Gabriel graph together with the per-edge β-threshold weights.
///
/// `weights[e]` is aligned with edge index `e` of `graph` (the edges are
/// emitted in increasing `(min, max)` endpoint order). Each weight is the β
/// value at which the edge leaves the lune-based β-skeleton, or
/// [`f64::INFINITY`] for an edge that persists beyond `max_beta`.
#[derive(Clone, Debug)]
pub struct BetaWeightedGabriel {
    /// The Gabriel graph on `points.len()` vertices.
    pub graph: Graph,
    /// Per-edge β-thresholds, aligned with `graph`'s edge indices.
    pub weights: Vec<f64>,
}

/// Build the β-weighted Gabriel graph of a point set.
///
/// `points` holds one row per point with a shared, arbitrary dimensionality
/// (inferred from the first row). `max_beta` is the search cutoff and must be
/// at least `1.0` (it may be [`f64::INFINITY`] for no cutoff); the threshold β
/// of any edge is itself `≥ 1`, so a smaller cutoff would be meaningless.
///
/// The result's `graph` is exactly the Gabriel graph; `weights[e]` is the
/// β-threshold of edge `e`.
///
/// This is an `O(n²)` candidate enumeration with an `O(n·d)` per-pair point
/// scan (`O(n³·d)` overall). See the module docs for why this reproduces the
/// reference's Delaunay-seeded, k-d-tree-pruned result exactly.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if `max_beta` is `NaN` or below `1.0`.
/// - [`IgraphError::InvalidArgument`] if the points are zero-dimensional (and
///   there is at least one point), or the rows have inconsistent
///   dimensionality.
/// - [`IgraphError::InvalidArgument`] if two points coincide, mirroring the
///   reference's rejection of duplicate points.
///
/// # Examples
///
/// ```
/// use rust_igraph::beta_weighted_gabriel_graph;
///
/// // The four corners of a unit square. The Gabriel graph keeps the four
/// // sides; each side leaves the β-skeleton only as β → ∞ (no point ever
/// // enters its lune), so every weight is infinite.
/// let pts = vec![
///     vec![0.0, 0.0],
///     vec![1.0, 0.0],
///     vec![0.0, 1.0],
///     vec![1.0, 1.0],
/// ];
/// let res = beta_weighted_gabriel_graph(&pts, f64::INFINITY).unwrap();
/// assert_eq!(res.graph.ecount(), 4); // sides only
/// assert!(res.weights.iter().all(|w| w.is_infinite()));
/// ```
// Single-character names (`a`, `b`, `c` points, `d` axis) are natural here.
#[allow(clippy::many_single_char_names)]
pub fn beta_weighted_gabriel_graph(
    points: &[Vec<f64>],
    max_beta: f64,
) -> IgraphResult<BetaWeightedGabriel> {
    if max_beta.is_nan() || max_beta < 1.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "beta_weighted_gabriel_graph: max_beta must be >= 1 (or infinity), got {max_beta}"
        )));
    }

    let n = points.len();
    let dim = if n == 0 { 0 } else { points[0].len() };
    if n > 0 {
        if dim == 0 {
            return Err(IgraphError::InvalidArgument(
                "beta_weighted_gabriel_graph: points must not be zero-dimensional".into(),
            ));
        }
        for (i, row) in points.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "beta_weighted_gabriel_graph: point row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument("beta_weighted_gabriel_graph: too many points".into())
    })?;
    let mut graph = Graph::with_vertices(n_u32);
    let mut weights: Vec<f64> = Vec::new();

    let beta_floor = 1.0 + TOLERANCE;

    for i in 0..n {
        let a = &points[i];
        for j in (i + 1)..n {
            let b = &points[j];

            let ab2 = sq_dist(a, b);
            if ab2 == 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "beta_weighted_gabriel_graph: duplicate points are not allowed".into(),
                ));
            }

            // BetaFinder: smallest β at which any other point first enters the
            // lune of edge a-b, capped at max_beta. INFINITY means no point
            // ever enters (below the cutoff).
            let mut smallest = f64::INFINITY;
            for (k, c) in points.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let mut ap2 = sq_dist(a, c);
                let mut bp2 = sq_dist(b, c);
                if ap2 > bp2 {
                    std::mem::swap(&mut ap2, &mut bp2);
                }
                let denom = ab2 + ap2 - bp2;
                let beta = if denom <= 0.0 {
                    f64::INFINITY
                } else {
                    let raw = 2.0 * ap2 / denom;
                    // A point inside the closed Gabriel ball gives β < 1; the
                    // reference collapses it to 0, marking a non-Gabriel edge.
                    if raw < beta_floor { 0.0 } else { raw }
                };
                if beta < smallest && beta < max_beta {
                    smallest = beta;
                }
            }

            // weight == 0 means the edge is not in the Gabriel graph: drop it.
            if smallest != 0.0 {
                let u = u32::try_from(i).map_err(|_| {
                    IgraphError::Internal("beta_weighted_gabriel_graph: vertex id overflow")
                })?;
                let v = u32::try_from(j).map_err(|_| {
                    IgraphError::Internal("beta_weighted_gabriel_graph: vertex id overflow")
                })?;
                graph.add_edge(u, v)?;
                weights.push(smallest);
            }
        }
    }

    Ok(BetaWeightedGabriel { graph, weights })
}

/// Squared Euclidean distance between two equal-length coordinate rows.
fn sq_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let t = x - y;
            t * t
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collect the undirected edge set as a sorted `(min, max)` vec.
    fn edge_set(g: &Graph) -> Vec<(u32, u32)> {
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

    /// Compare a computed weight vector against an authentic golden vector
    /// (order-independent): the finite weights must match as a sorted multiset
    /// within relative tolerance, and the infinite-weight counts must agree.
    fn assert_weights_match(actual: &[f64], golden: &[f64]) {
        assert_eq!(
            actual.len(),
            golden.len(),
            "edge/weight count mismatch: {} vs {}",
            actual.len(),
            golden.len()
        );

        let inf_actual = actual.iter().filter(|w| w.is_infinite()).count();
        let inf_golden = golden.iter().filter(|w| w.is_infinite()).count();
        assert_eq!(
            inf_actual, inf_golden,
            "infinite-weight count mismatch: {inf_actual} vs {inf_golden}"
        );

        let mut a: Vec<f64> = actual.iter().copied().filter(|w| w.is_finite()).collect();
        let mut g: Vec<f64> = golden.iter().copied().filter(|w| w.is_finite()).collect();
        a.sort_by(|x, y| x.partial_cmp(y).expect("finite"));
        g.sort_by(|x, y| x.partial_cmp(y).expect("finite"));

        for (x, y) in a.iter().zip(g.iter()) {
            // Golden values are printed to ~6 significant figures; a 1e-4
            // relative tolerance covers the rounding comfortably.
            let tol = 1e-4 * y.abs().max(1.0);
            assert!(
                (x - y).abs() <= tol,
                "weight mismatch: got {x}, expected {y} (tol {tol})"
            );
        }
    }

    /// The 25-point 2-D set from igraph's `beta_skeletons.c` (row-major 25×2).
    #[allow(clippy::unreadable_literal)]
    fn points_25() -> Vec<Vec<f64>> {
        let flat = [
            0.474217, 0.0314797, 0.208089, 0.439308, 0.967367, 0.530466, 0.177005, 0.426713,
            0.568462, 0.57507, 0.441834, 0.284514, 0.479224, 0.817988, 0.720209, 0.225744,
            0.204941, 0.44297, 0.285318, 0.912984, 0.831097, 0.0176603, 0.827154, 0.472702,
            0.173059, 0.561858, 0.156276, 0.88019, 0.65935, 0.538207, 0.570379, 0.518081, 0.900553,
            0.656416, 0.726631, 0.863709, 0.380264, 0.287159, 0.31098, 0.230773, 0.243089,
            0.164584, 0.967974, 0.524992, 0.726605, 0.0724703, 0.739752, 0.447069, 0.0443581,
            0.444839,
        ];
        flat.chunks_exact(2).map(<[f64]>::to_vec).collect()
    }

    /// The 10-point 3-D set from igraph's `beta_skeletons.c`: the same flat
    /// coordinate array reshaped row-major as 10×3 (first 30 values).
    #[allow(clippy::unreadable_literal)]
    fn points_10_3d() -> Vec<Vec<f64>> {
        let flat = [
            0.474217, 0.0314797, 0.208089, 0.439308, 0.967367, 0.530466, 0.177005, 0.426713,
            0.568462, 0.57507, 0.441834, 0.284514, 0.479224, 0.817988, 0.720209, 0.225744,
            0.204941, 0.44297, 0.285318, 0.912984, 0.831097, 0.0176603, 0.827154, 0.472702,
            0.173059, 0.561858, 0.156276, 0.88019, 0.65935, 0.538207,
        ];
        flat.chunks_exact(3).map(<[f64]>::to_vec).collect()
    }

    #[test]
    fn empty_point_set() {
        let res = beta_weighted_gabriel_graph(&[], f64::INFINITY).expect("empty ok");
        assert_eq!(res.graph.vcount(), 0);
        assert_eq!(res.graph.ecount(), 0);
        assert!(res.weights.is_empty());
    }

    #[test]
    fn single_point() {
        let res = beta_weighted_gabriel_graph(&[vec![0.5, 0.5]], f64::INFINITY).expect("one ok");
        assert_eq!(res.graph.vcount(), 1);
        assert_eq!(res.graph.ecount(), 0);
        assert!(res.weights.is_empty());
    }

    #[test]
    fn two_points_persist_to_infinity() {
        let res = beta_weighted_gabriel_graph(&[vec![0.0, 0.0], vec![3.0, 4.0]], f64::INFINITY)
            .expect("pair ok");
        assert_eq!(edge_set(&res.graph), vec![(0, 1)]);
        assert_eq!(res.weights, vec![f64::INFINITY]);
    }

    #[test]
    fn invalid_max_beta_is_error() {
        for bad in [f64::NAN, 0.0, 0.5, -1.0] {
            assert!(
                matches!(
                    beta_weighted_gabriel_graph(&[vec![0.0, 0.0]], bad).unwrap_err(),
                    IgraphError::InvalidArgument(_)
                ),
                "max_beta = {bad} should be rejected"
            );
        }
    }

    #[test]
    fn zero_dimensional_error() {
        let err = beta_weighted_gabriel_graph(&[vec![], vec![]], f64::INFINITY).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err =
            beta_weighted_gabriel_graph(&[vec![0.0, 0.0], vec![1.0]], f64::INFINITY).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn duplicate_points_error() {
        let err = beta_weighted_gabriel_graph(&[vec![0.0, 0.0], vec![0.0, 0.0]], f64::INFINITY)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn edge_set_equals_gabriel() {
        // The surviving edges are exactly the Gabriel graph, regardless of the
        // cutoff (the cutoff only changes finite weights to infinity).
        let pts = points_25();
        let gabriel = crate::gabriel_graph(&pts).expect("gabriel");
        for cutoff in [f64::INFINITY, 5.0, 1.0] {
            let res = beta_weighted_gabriel_graph(&pts, cutoff).expect("weighted");
            assert_eq!(
                edge_set(&res.graph),
                edge_set(&gabriel),
                "edge set differs from Gabriel graph at cutoff {cutoff}"
            );
            assert_eq!(res.weights.len(), res.graph.ecount() as usize);
        }
    }

    #[test]
    fn cutoff_below_one_threshold_all_infinite() {
        // With max_beta == 1, no edge's finite threshold (all >= 1 + tol) is
        // below the cutoff, so every Gabriel edge persists to infinity.
        let res = beta_weighted_gabriel_graph(&points_25(), 1.0).expect("ok");
        assert!(res.weights.iter().all(|w| w.is_infinite()));
    }

    // --- Authentic igraph C anchors (beta_skeletons.out) ---

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_2d_25_points_cutoff_infinity() {
        let res = beta_weighted_gabriel_graph(&points_25(), f64::INFINITY).expect("ok");
        let golden = [
            2.13771,
            1.87896,
            1.30643,
            10.5357,
            f64::INFINITY,
            1.04766,
            2.38206,
            1.72356,
            10.9882,
            41555.3,
            115.458,
            6.14845,
            4.87967,
            1.32814,
            1.42918,
            8.52199,
            1.20967,
            2.72446,
            3.66988,
            f64::INFINITY,
            11.4047,
            1.33013,
            3.50412,
            36.5966,
            10.6973,
            4.82793,
            48.4413,
            628.646,
            1.11477,
            8.32087,
            f64::INFINITY,
            4.28868,
            1.19886,
            3.33,
            1.31892,
            11.3292,
            3.3514,
            15.7597,
            29.7302,
        ];
        assert_weights_match(&res.weights, &golden);
    }

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_2d_25_points_cutoff_5() {
        let res = beta_weighted_gabriel_graph(&points_25(), 5.0).expect("ok");
        let golden = [
            2.13771,
            1.87896,
            1.30643,
            f64::INFINITY,
            f64::INFINITY,
            1.04766,
            2.38206,
            1.72356,
            f64::INFINITY,
            f64::INFINITY,
            f64::INFINITY,
            f64::INFINITY,
            4.87967,
            1.32814,
            1.42918,
            f64::INFINITY,
            1.20967,
            2.72446,
            3.66988,
            f64::INFINITY,
            f64::INFINITY,
            1.33013,
            3.50412,
            f64::INFINITY,
            f64::INFINITY,
            4.82793,
            f64::INFINITY,
            f64::INFINITY,
            1.11477,
            f64::INFINITY,
            f64::INFINITY,
            4.28868,
            1.19886,
            3.33,
            1.31892,
            f64::INFINITY,
            3.3514,
            f64::INFINITY,
            f64::INFINITY,
        ];
        assert_weights_match(&res.weights, &golden);
    }

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_3d_10_points_cutoff_infinity() {
        let res = beta_weighted_gabriel_graph(&points_10_3d(), f64::INFINITY).expect("ok");
        let golden = [
            2.29424, 2.87805, 1.51364, 15.0303, 1.01534, 2.12088, 1.05443, 1.30234, 2.07368,
            8.72711, 1.0848, 1.99727, 2.0717, 1.25544, 1.77266, 2.21701, 4.16352, 58.0727, 4.91485,
            1.47137, 1.89216, 2.00274,
        ];
        assert_weights_match(&res.weights, &golden);
    }

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_3d_10_points_cutoff_5() {
        let res = beta_weighted_gabriel_graph(&points_10_3d(), 5.0).expect("ok");
        let golden = [
            2.29424,
            2.87805,
            1.51364,
            f64::INFINITY,
            1.01534,
            2.12088,
            1.05443,
            1.30234,
            2.07368,
            f64::INFINITY,
            1.0848,
            1.99727,
            2.0717,
            1.25544,
            1.77266,
            2.21701,
            4.16352,
            f64::INFINITY,
            4.91485,
            1.47137,
            1.89216,
            2.00274,
        ];
        assert_weights_match(&res.weights, &golden);
    }
}
