//! Graph-level centralization indices (ALGO-PR-033 + ALGO-PR-043).
//!
//! Counterpart of `igraph_centralization*` from
//! `references/igraph/src/centrality/centralization.c` (723 lines).
//!
//! Centralization measures how much a graph's structure revolves
//! around a single vertex. Given per-vertex centrality scores, the
//! graph-level centralization is:
//!
//! ```text
//! C = Σ_v (max_u c_u - c_v)
//! ```
//!
//! Optionally divided by the theoretical maximum for the most
//! centralized structure (usually a star graph) of the same size.
//!
//! ## Convenience wrappers (ALGO-PR-043)
//!
//! [`centralization_degree_wrapper`], [`centralization_betweenness_wrapper`],
//! [`centralization_closeness_wrapper`], and
//! [`centralization_eigenvector_wrapper`] compose the per-vertex score
//! functions with the tmax + centralization formula into single calls.

use super::betweenness::betweenness;
use super::closeness::closeness;
use super::degree::DegreeMode;
use super::eigenvector::{EigenvectorMode, eigenvector_centrality_full};
use super::strength::{StrengthMode, strength_with_mode};
use crate::core::{Graph, IgraphResult};

/// How loops are counted when computing degree centralization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// Ignore loop edges entirely.
    NoLoops,
    /// Count each loop edge once.
    LoopsOnce,
    /// Count each loop edge twice (undirected) or once (directed).
    LoopsTwice,
}

/// Whether centralization considers in-degree, out-degree, or total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CentralizationMode {
    In,
    Out,
    All,
}

/// Compute the graph-level centralization score from per-vertex scores.
///
/// Returns `C = n * max(scores) - sum(scores)` when `normalized` is
/// `false`, or `C / theoretical_max` when `normalized` is `true`.
///
/// Returns `NaN` for empty score vectors.
///
/// # Examples
///
/// ```
/// use rust_igraph::centralization;
///
/// // Star graph degree scores: center=4, leaves=1,1,1,1
/// let scores = [4.0, 1.0, 1.0, 1.0, 1.0];
/// let c = centralization(&scores, 12.0, true);
/// assert!((c - 1.0).abs() < 1e-9);
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn centralization(scores: &[f64], theoretical_max: f64, normalized: bool) -> f64 {
    if scores.is_empty() {
        return f64::NAN;
    }

    let max_score = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let n = scores.len() as f64;
    let sum: f64 = scores.iter().sum();
    let cent = n * max_score - sum;

    if normalized {
        cent / theoretical_max
    } else {
        cent
    }
}

/// Theoretical maximum degree centralization for a star graph.
///
/// Returns `NaN` for `n == 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{CentralizationMode, LoopMode, centralization_degree_tmax};
///
/// // Undirected, no loops, 5 vertices: (5-1)*(5-2) = 12
/// let tmax = centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::NoLoops);
/// assert!((tmax - 12.0).abs() < 1e-9);
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn centralization_degree_tmax(
    n: u32,
    directed: bool,
    mode: CentralizationMode,
    loops: LoopMode,
) -> f64 {
    if n == 0 {
        return f64::NAN;
    }

    let nf = f64::from(n);

    if directed {
        match mode {
            CentralizationMode::In | CentralizationMode::Out => {
                if matches!(loops, LoopMode::NoLoops) {
                    (nf - 1.0) * (nf - 1.0)
                } else {
                    (nf - 1.0) * nf
                }
            }
            CentralizationMode::All => {
                if matches!(loops, LoopMode::NoLoops) {
                    2.0 * (nf - 1.0) * (nf - 2.0)
                } else {
                    2.0 * (nf - 1.0) * (nf - 1.0)
                }
            }
        }
    } else {
        match loops {
            LoopMode::NoLoops => (nf - 1.0) * (nf - 2.0),
            LoopMode::LoopsOnce => (nf - 1.0) * (nf - 1.0),
            LoopMode::LoopsTwice => (nf - 1.0) * nf,
        }
    }
}

/// Theoretical maximum betweenness centralization for a star graph.
///
/// Returns `NaN` for `n == 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::centralization_betweenness_tmax;
///
/// // Undirected, 5 vertices: (5-1)*(5-1)*(5-2)/2 = 24
/// let tmax = centralization_betweenness_tmax(5, false);
/// assert!((tmax - 24.0).abs() < 1e-9);
/// ```
pub fn centralization_betweenness_tmax(n: u32, directed: bool) -> f64 {
    if n == 0 {
        return f64::NAN;
    }

    let nf = f64::from(n);

    if directed {
        (nf - 1.0) * (nf - 1.0) * (nf - 2.0)
    } else {
        (nf - 1.0) * (nf - 1.0) * (nf - 2.0) / 2.0
    }
}

/// Theoretical maximum closeness centralization for a star graph.
///
/// For directed graphs, `mode` distinguishes IN/OUT from ALL.
/// For undirected, `mode` should be `All`.
///
/// Returns `NaN` for `n == 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{CentralizationMode, centralization_closeness_tmax};
///
/// // Undirected, 5 vertices: (5-1)*(5-2)/(2*5-3) = 12/7
/// let tmax = centralization_closeness_tmax(5, CentralizationMode::All);
/// assert!((tmax - 12.0 / 7.0).abs() < 1e-9);
/// ```
pub fn centralization_closeness_tmax(n: u32, mode: CentralizationMode) -> f64 {
    if n == 0 {
        return f64::NAN;
    }

    let nf = f64::from(n);

    match mode {
        CentralizationMode::In | CentralizationMode::Out => (nf - 1.0) * (1.0 - 1.0 / nf),
        CentralizationMode::All => (nf - 1.0) * (nf - 2.0) / (2.0 * nf - 3.0),
    }
}

/// Theoretical maximum eigenvector centralization for a star graph.
///
/// For directed graphs, `mode` distinguishes IN/OUT from ALL.
/// For undirected, `mode` should be `All`.
///
/// Returns `NaN` for `n == 0`, `0.0` for `n == 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{CentralizationMode, centralization_eigenvector_tmax};
///
/// // Undirected, 5 vertices: 5-2 = 3
/// let tmax = centralization_eigenvector_tmax(5, CentralizationMode::All);
/// assert!((tmax - 3.0).abs() < 1e-9);
/// ```
pub fn centralization_eigenvector_tmax(n: u32, mode: CentralizationMode) -> f64 {
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return 0.0;
    }

    let nf = f64::from(n);

    match mode {
        CentralizationMode::In | CentralizationMode::Out => nf - 1.0,
        CentralizationMode::All => nf - 2.0,
    }
}

/// Result of a centralization convenience wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct CentralizationResult {
    /// Per-vertex centrality scores.
    pub scores: Vec<f64>,
    /// Graph-level centralization value.
    pub centralization: f64,
    /// Theoretical maximum for the most centralized graph of this size.
    pub theoretical_max: f64,
}

fn degree_to_strength_mode(mode: DegreeMode) -> StrengthMode {
    match mode {
        DegreeMode::Out => StrengthMode::Out,
        DegreeMode::In => StrengthMode::In,
        DegreeMode::All => StrengthMode::All,
    }
}

fn degree_to_cent_mode(mode: DegreeMode) -> CentralizationMode {
    match mode {
        DegreeMode::Out => CentralizationMode::Out,
        DegreeMode::In => CentralizationMode::In,
        DegreeMode::All => CentralizationMode::All,
    }
}

/// Degree centralization: compute per-vertex degree scores and the
/// graph-level centralization in one call.
///
/// `mode` selects in-/out-/total degree for directed graphs (ignored
/// for undirected). `loops` controls whether self-loops are counted.
///
/// Counterpart of `igraph_centralization_degree()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DegreeMode, centralization_degree_wrapper};
///
/// // Star K_{1,4}: normalized degree centralization = 1.0.
/// let mut g = Graph::with_vertices(5);
/// for v in 1..5u32 { g.add_edge(0, v).unwrap(); }
/// let r = centralization_degree_wrapper(&g, DegreeMode::All, false, true).unwrap();
/// assert!((r.centralization - 1.0).abs() < 1e-9);
/// assert!((r.scores[0] - 4.0).abs() < 1e-9);
/// ```
pub fn centralization_degree_wrapper(
    graph: &Graph,
    mode: DegreeMode,
    loops: bool,
    normalized: bool,
) -> IgraphResult<CentralizationResult> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    let scores = if ecount == 0 {
        vec![0.0_f64; n as usize]
    } else {
        let unit_w = vec![1.0_f64; ecount];
        strength_with_mode(graph, &unit_w, degree_to_strength_mode(mode), loops)?
    };

    let loop_mode = if loops {
        LoopMode::LoopsTwice
    } else {
        LoopMode::NoLoops
    };
    let tmax =
        centralization_degree_tmax(n, graph.is_directed(), degree_to_cent_mode(mode), loop_mode);
    let cent = centralization(&scores, tmax, normalized);

    Ok(CentralizationResult {
        scores,
        centralization: cent,
        theoretical_max: tmax,
    })
}

/// Betweenness centralization: compute per-vertex betweenness scores
/// and the graph-level centralization in one call.
///
/// Directedness is derived from `graph.is_directed()`.
///
/// Counterpart of `igraph_centralization_betweenness()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centralization_betweenness_wrapper};
///
/// // Star K_{1,4}: normalized betweenness centralization = 1.0.
/// let mut g = Graph::with_vertices(5);
/// for v in 1..5u32 { g.add_edge(0, v).unwrap(); }
/// let r = centralization_betweenness_wrapper(&g, true).unwrap();
/// assert!((r.centralization - 1.0).abs() < 1e-9);
/// ```
pub fn centralization_betweenness_wrapper(
    graph: &Graph,
    normalized: bool,
) -> IgraphResult<CentralizationResult> {
    let scores = betweenness(graph)?;
    let directed = graph.is_directed();
    let tmax = centralization_betweenness_tmax(graph.vcount(), directed);
    let cent = centralization(&scores, tmax, normalized);

    Ok(CentralizationResult {
        scores,
        centralization: cent,
        theoretical_max: tmax,
    })
}

/// Closeness centralization: compute per-vertex closeness scores and
/// the graph-level centralization in one call.
///
/// Vertices with no reachable neighbors produce `NaN` scores; if any
/// vertex is unreachable the centralization itself will be `NaN`
/// (matching igraph C semantics for disconnected graphs).
///
/// Counterpart of `igraph_centralization_closeness()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centralization_closeness_wrapper};
///
/// // Star K_{1,4}: normalized closeness centralization = 1.0.
/// let mut g = Graph::with_vertices(5);
/// for v in 1..5u32 { g.add_edge(0, v).unwrap(); }
/// let r = centralization_closeness_wrapper(&g, true).unwrap();
/// assert!((r.centralization - 1.0).abs() < 1e-9);
/// ```
pub fn centralization_closeness_wrapper(
    graph: &Graph,
    normalized: bool,
) -> IgraphResult<CentralizationResult> {
    let raw = closeness(graph)?;
    let scores: Vec<f64> = raw.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();

    let cent_mode = if graph.is_directed() {
        CentralizationMode::Out
    } else {
        CentralizationMode::All
    };
    let tmax = centralization_closeness_tmax(graph.vcount(), cent_mode);
    let cent = centralization(&scores, tmax, normalized);

    Ok(CentralizationResult {
        scores,
        centralization: cent,
        theoretical_max: tmax,
    })
}

/// Eigenvector centralization: compute per-vertex eigenvector centrality
/// scores and the graph-level centralization in one call.
///
/// For directed graphs, uses `EigenvectorMode::Out` (the standard
/// convention matching upstream). The eigenvector is max-1 normalised.
///
/// Counterpart of `igraph_centralization_eigenvector_centrality()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centralization_eigenvector_wrapper};
///
/// // Star K_{1,4}: center has eigenvector centrality 1.0, leaves ≈ 0.5.
/// let mut g = Graph::with_vertices(5);
/// for v in 1..5u32 { g.add_edge(0, v).unwrap(); }
/// let r = centralization_eigenvector_wrapper(&g, true).unwrap();
/// assert!(r.centralization > 0.0);
/// assert!((r.scores[0] - 1.0).abs() < 1e-9);
/// ```
pub fn centralization_eigenvector_wrapper(
    graph: &Graph,
    normalized: bool,
) -> IgraphResult<CentralizationResult> {
    let eig_mode = if graph.is_directed() {
        EigenvectorMode::Out
    } else {
        EigenvectorMode::All
    };
    let eig = eigenvector_centrality_full(graph, eig_mode, None)?;
    let scores = eig.vector;

    let cent_mode = if graph.is_directed() {
        CentralizationMode::Out
    } else {
        CentralizationMode::All
    };
    let tmax = centralization_eigenvector_tmax(graph.vcount(), cent_mode);
    let cent = centralization(&scores, tmax, normalized);

    Ok(CentralizationResult {
        scores,
        centralization: cent,
        theoretical_max: tmax,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn centralization_empty() {
        let c = centralization(&[], 1.0, false);
        assert!(c.is_nan());
    }

    #[test]
    fn centralization_single() {
        let c = centralization(&[5.0], 1.0, false);
        assert!(approx_eq(c, 0.0));
    }

    #[test]
    fn centralization_star_degree() {
        let scores = [4.0, 1.0, 1.0, 1.0, 1.0];
        let c = centralization(&scores, 12.0, false);
        assert!(approx_eq(c, 12.0));

        let c_norm = centralization(&scores, 12.0, true);
        assert!(approx_eq(c_norm, 1.0));
    }

    #[test]
    fn centralization_uniform() {
        let scores = [3.0, 3.0, 3.0, 3.0];
        let c = centralization(&scores, 6.0, false);
        assert!(approx_eq(c, 0.0));
    }

    #[test]
    fn centralization_normalized() {
        let scores = [4.0, 2.0, 1.0];
        let unnorm = centralization(&scores, 1.0, false);
        let norm = centralization(&scores, 6.0, true);
        assert!(approx_eq(unnorm, 5.0));
        assert!(approx_eq(norm, 5.0 / 6.0));
    }

    #[test]
    fn degree_tmax_zero() {
        assert!(
            centralization_degree_tmax(0, false, CentralizationMode::All, LoopMode::NoLoops)
                .is_nan()
        );
    }

    #[test]
    fn degree_tmax_undirected_no_loops() {
        assert!(approx_eq(
            centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::NoLoops),
            12.0
        ));
        assert!(approx_eq(
            centralization_degree_tmax(10, false, CentralizationMode::All, LoopMode::NoLoops),
            72.0
        ));
    }

    #[test]
    fn degree_tmax_undirected_loops_once() {
        assert!(approx_eq(
            centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::LoopsOnce),
            16.0
        ));
    }

    #[test]
    fn degree_tmax_undirected_loops_twice() {
        assert!(approx_eq(
            centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::LoopsTwice),
            20.0
        ));
    }

    #[test]
    fn degree_tmax_directed_in_no_loops() {
        assert!(approx_eq(
            centralization_degree_tmax(5, true, CentralizationMode::In, LoopMode::NoLoops),
            16.0
        ));
    }

    #[test]
    fn degree_tmax_directed_all_no_loops() {
        assert!(approx_eq(
            centralization_degree_tmax(5, true, CentralizationMode::All, LoopMode::NoLoops),
            24.0
        ));
    }

    #[test]
    fn degree_tmax_directed_in_with_loops() {
        assert!(approx_eq(
            centralization_degree_tmax(5, true, CentralizationMode::In, LoopMode::LoopsTwice),
            20.0
        ));
    }

    #[test]
    fn degree_tmax_directed_all_with_loops() {
        assert!(approx_eq(
            centralization_degree_tmax(5, true, CentralizationMode::All, LoopMode::LoopsTwice),
            32.0
        ));
    }

    #[test]
    fn betweenness_tmax_zero() {
        assert!(centralization_betweenness_tmax(0, false).is_nan());
    }

    #[test]
    fn betweenness_tmax_undirected() {
        assert!(approx_eq(centralization_betweenness_tmax(5, false), 24.0));
        assert!(approx_eq(centralization_betweenness_tmax(3, false), 2.0));
    }

    #[test]
    fn betweenness_tmax_directed() {
        assert!(approx_eq(centralization_betweenness_tmax(5, true), 48.0));
        assert!(approx_eq(centralization_betweenness_tmax(3, true), 4.0));
    }

    #[test]
    fn closeness_tmax_zero() {
        assert!(centralization_closeness_tmax(0, CentralizationMode::All).is_nan());
    }

    #[test]
    fn closeness_tmax_undirected() {
        let tmax = centralization_closeness_tmax(5, CentralizationMode::All);
        assert!(approx_eq(tmax, 12.0 / 7.0));
    }

    #[test]
    fn closeness_tmax_directed() {
        let tmax = centralization_closeness_tmax(5, CentralizationMode::In);
        assert!(approx_eq(tmax, 4.0 * (1.0 - 1.0 / 5.0)));
    }

    #[test]
    fn eigenvector_tmax_zero() {
        assert!(centralization_eigenvector_tmax(0, CentralizationMode::All).is_nan());
    }

    #[test]
    fn eigenvector_tmax_one() {
        assert!(approx_eq(
            centralization_eigenvector_tmax(1, CentralizationMode::All),
            0.0
        ));
    }

    #[test]
    fn eigenvector_tmax_undirected() {
        assert!(approx_eq(
            centralization_eigenvector_tmax(5, CentralizationMode::All),
            3.0
        ));
    }

    #[test]
    fn eigenvector_tmax_directed() {
        assert!(approx_eq(
            centralization_eigenvector_tmax(5, CentralizationMode::In),
            4.0
        ));
    }

    #[test]
    fn star5_degree_centralization() {
        let scores = [4.0, 1.0, 1.0, 1.0, 1.0];
        let tmax = centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::NoLoops);
        let c = centralization(&scores, tmax, true);
        assert!(approx_eq(c, 1.0), "star degree centralization = {c}");
    }

    #[test]
    fn ring_degree_centralization() {
        let scores = [2.0, 2.0, 2.0, 2.0, 2.0];
        let tmax = centralization_degree_tmax(5, false, CentralizationMode::All, LoopMode::NoLoops);
        let c = centralization(&scores, tmax, true);
        assert!(approx_eq(c, 0.0), "ring degree centralization = {c}");
    }

    #[test]
    fn star_betweenness_centralization() {
        let n = 5u32;
        let scores = [6.0, 0.0, 0.0, 0.0, 0.0];
        let tmax = centralization_betweenness_tmax(n, false);
        let c = centralization(&scores, tmax, true);
        assert!(approx_eq(c, 1.0), "star betweenness centralization = {c}");
    }

    // ── wrapper tests ────────────────────────────────────────────────

    fn make_star(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for v in 1..n {
            g.add_edge(0, v).unwrap();
        }
        g
    }

    fn make_ring(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for v in 0..n {
            g.add_edge(v, (v + 1) % n).unwrap();
        }
        g
    }

    fn make_path(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for v in 0..n - 1 {
            g.add_edge(v, v + 1).unwrap();
        }
        g
    }

    // -- degree wrapper --

    #[test]
    fn wrapper_degree_star5() {
        let g = make_star(5);
        let r = centralization_degree_wrapper(&g, DegreeMode::All, false, true).unwrap();
        assert!(approx_eq(r.centralization, 1.0), "got {}", r.centralization);
        assert!(approx_eq(r.scores[0], 4.0));
        for i in 1..5 {
            assert!(approx_eq(r.scores[i], 1.0));
        }
    }

    #[test]
    fn wrapper_degree_ring5() {
        let g = make_ring(5);
        let r = centralization_degree_wrapper(&g, DegreeMode::All, false, true).unwrap();
        assert!(approx_eq(r.centralization, 0.0), "got {}", r.centralization);
    }

    #[test]
    fn wrapper_degree_empty() {
        let g = Graph::with_vertices(0);
        let r = centralization_degree_wrapper(&g, DegreeMode::All, false, true).unwrap();
        assert!(r.scores.is_empty());
        assert!(r.centralization.is_nan());
    }

    #[test]
    fn wrapper_degree_single_vertex() {
        let g = Graph::with_vertices(1);
        let r = centralization_degree_wrapper(&g, DegreeMode::All, false, false).unwrap();
        assert_eq!(r.scores.len(), 1);
        assert!(approx_eq(r.scores[0], 0.0));
        assert!(approx_eq(r.centralization, 0.0));
    }

    #[test]
    fn wrapper_degree_unnormalized() {
        let g = make_star(5);
        let r = centralization_degree_wrapper(&g, DegreeMode::All, false, false).unwrap();
        assert!(approx_eq(r.centralization, 12.0));
        assert!(approx_eq(r.theoretical_max, 12.0));
    }

    #[test]
    fn wrapper_degree_with_self_loops() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r_loops = centralization_degree_wrapper(&g, DegreeMode::All, true, false).unwrap();
        let r_no_loops = centralization_degree_wrapper(&g, DegreeMode::All, false, false).unwrap();
        // With loops: vertex 0 degree = 2+1 = 3 (self-loop twice + edge)
        assert!(r_loops.scores[0] > r_no_loops.scores[0]);
    }

    // -- betweenness wrapper --

    #[test]
    fn wrapper_betweenness_star5() {
        let g = make_star(5);
        let r = centralization_betweenness_wrapper(&g, true).unwrap();
        assert!(approx_eq(r.centralization, 1.0), "got {}", r.centralization);
    }

    #[test]
    fn wrapper_betweenness_path5() {
        let g = make_path(5);
        let r = centralization_betweenness_wrapper(&g, false).unwrap();
        assert!(r.centralization > 0.0);
        // Center vertex (2) should have highest betweenness
        assert!(r.scores[2] > r.scores[0]);
        assert!(r.scores[2] > r.scores[4]);
    }

    #[test]
    fn wrapper_betweenness_ring() {
        let g = make_ring(5);
        let r = centralization_betweenness_wrapper(&g, true).unwrap();
        assert!(approx_eq(r.centralization, 0.0), "got {}", r.centralization);
    }

    #[test]
    fn wrapper_betweenness_empty() {
        let g = Graph::with_vertices(0);
        let r = centralization_betweenness_wrapper(&g, true).unwrap();
        assert!(r.scores.is_empty());
    }

    // -- closeness wrapper --

    #[test]
    fn wrapper_closeness_star5() {
        let g = make_star(5);
        let r = centralization_closeness_wrapper(&g, true).unwrap();
        assert!(approx_eq(r.centralization, 1.0), "got {}", r.centralization);
        // Center has closeness 1.0
        assert!(approx_eq(r.scores[0], 1.0));
    }

    #[test]
    fn wrapper_closeness_ring() {
        let g = make_ring(5);
        let r = centralization_closeness_wrapper(&g, true).unwrap();
        assert!(approx_eq(r.centralization, 0.0), "got {}", r.centralization);
    }

    #[test]
    fn wrapper_closeness_disconnected_is_nan() {
        // Two isolated vertices → closeness is None → centralization is NaN
        let g = Graph::with_vertices(2);
        let r = centralization_closeness_wrapper(&g, true).unwrap();
        assert!(r.scores[0].is_nan());
        assert!(r.scores[1].is_nan());
        assert!(r.centralization.is_nan());
    }

    // -- eigenvector wrapper --

    #[test]
    fn wrapper_eigenvector_star5() {
        let g = make_star(5);
        let r = centralization_eigenvector_wrapper(&g, true).unwrap();
        assert!(r.centralization > 0.0);
        // Center should have highest score (1.0)
        assert!(approx_eq(r.scores[0], 1.0));
        // Leaves should have equal score
        for i in 1..5 {
            assert!(approx_eq(r.scores[i], r.scores[1]));
        }
    }

    #[test]
    fn wrapper_eigenvector_ring() {
        let g = make_ring(5);
        let r = centralization_eigenvector_wrapper(&g, true).unwrap();
        // Regular graph: all vertices have same eigenvector centrality
        assert!(approx_eq(r.centralization, 0.0), "got {}", r.centralization);
    }

    #[test]
    fn wrapper_eigenvector_empty() {
        let g = Graph::with_vertices(0);
        let r = centralization_eigenvector_wrapper(&g, true).unwrap();
        assert!(r.scores.is_empty());
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn centralization_non_negative(
            scores in proptest::collection::vec(0.0f64..100.0, 1..20)
        ) {
            let c = centralization(&scores, 1.0, false);
            prop_assert!(c >= 0.0, "centralization should be >= 0, got {c}");
        }

        #[test]
        fn centralization_uniform_is_zero(val in 0.0f64..100.0, n in 1usize..20) {
            let scores = vec![val; n];
            let c = centralization(&scores, 1.0, false);
            prop_assert!(
                c.abs() < 1e-9,
                "uniform scores should give centralization 0, got {c}"
            );
        }

        #[test]
        fn tmax_non_negative(n in 3u32..100) {
            let deg = centralization_degree_tmax(n, false, CentralizationMode::All, LoopMode::NoLoops);
            let bet = centralization_betweenness_tmax(n, false);
            let clo = centralization_closeness_tmax(n, CentralizationMode::All);
            let eig = centralization_eigenvector_tmax(n, CentralizationMode::All);
            prop_assert!(deg > 0.0, "degree tmax should be > 0 for n={n}");
            prop_assert!(bet > 0.0, "betweenness tmax should be > 0 for n={n}");
            prop_assert!(clo > 0.0, "closeness tmax should be > 0 for n={n}");
            prop_assert!(eig > 0.0, "eigenvector tmax should be > 0 for n={n}");
        }
    }
}
