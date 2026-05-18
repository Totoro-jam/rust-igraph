//! Reciprocity (ALGO-PR-004 + ALGO-PR-004b).
//!
//! Counterpart of `igraph_reciprocity()` from
//! `references/igraph/src/properties/basic_properties.c:325-406`.
//!
//! For directed graphs, reciprocity is the proportion of mutual
//! connections — formally `1 - (sum_ij |A_ij - A_ji|) / (2 sum_ij A_ij)`.
//! Equivalent to (number of edges with a reverse counterpart) / (total
//! edges). For undirected graphs it is 1.0 by definition. For graphs
//! with no edges, the value is undefined (`None` here, matching upstream's
//! `IGRAPH_NAN`).
//!
//! [`reciprocity`] — the canonical Phase-1 entry — fixes
//! `mode = Default` and `ignore_loops = false`.
//! [`reciprocity_with_mode`] (PR-004b) exposes both knobs.

use crate::core::{Graph, IgraphResult};

/// Reciprocity formula choice. Counterpart of upstream's
/// `igraph_reciprocity_t` (`IGRAPH_RECIPROCITY_DEFAULT` /
/// `IGRAPH_RECIPROCITY_RATIO`).
///
/// - [`ReciprocityMode::Default`] — `rec / total_edges`. The fraction
///   of directed edges that have a reverse counterpart.
/// - [`ReciprocityMode::Ratio`] — `rec / (rec + nonrec)`. The fraction
///   of *connected ordered vertex pairs* that are reciprocal. The two
///   formulas agree on graphs with no parallel edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReciprocityMode {
    /// `rec / total_edges`.
    Default,
    /// `rec / (rec + nonrec)`.
    Ratio,
}

/// Reciprocity of `graph`. Returns `None` for graphs with no edges
/// (matches upstream's `IGRAPH_NAN`).
///
/// Counterpart of `igraph_reciprocity(_, _, /*ignore_loops=*/false,
/// IGRAPH_RECIPROCITY_DEFAULT)`. For undirected graphs returns
/// `Some(1.0)` unconditionally.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocity};
///
/// // Directed mutual pair: 0 -> 1, 1 -> 0. Both edges have a reverse → 1.0.
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
///
/// // One-way: 0 -> 1 only. No reverse → 0.0.
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
/// ```
pub fn reciprocity(graph: &Graph) -> IgraphResult<Option<f64>> {
    reciprocity_with_mode(graph, false, ReciprocityMode::Default)
}

/// Reciprocity with explicit mode + `ignore_loops` (ALGO-PR-004b).
///
/// Counterpart of `igraph_reciprocity(_, _, ignore_loops, mode)`.
/// See [`ReciprocityMode`] for the formula.
///
/// `ignore_loops`:
/// - `false` (matches [`reciprocity`]) — self-loops count as mutual
///   in the numerator (a self-loop is its own reverse) and stay in the
///   denominator.
/// - `true` — self-loops drop out of both numerator and denominator.
///
/// Returns `None` for graphs with no edges (matches upstream's
/// `IGRAPH_NAN`), including the `Default` case where `m == 0` and the
/// `Ratio` case where `rec + nonrec == 0`. Undirected graphs always
/// return `Some(1.0)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocity_with_mode, ReciprocityMode};
///
/// // Mutual pair + one-way edge: rec = 2 edges, nonrec = 2 (the
/// // one-way edge contributes once at source AND once at target).
/// // Default = 2/3, Ratio = 2/(2+2) = 0.5.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// g.add_edge(0, 2).unwrap();
/// assert_eq!(reciprocity_with_mode(&g, false, ReciprocityMode::Default).unwrap(),
///            Some(2.0 / 3.0));
/// assert_eq!(reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap(),
///            Some(0.5));
/// ```
pub fn reciprocity_with_mode(
    graph: &Graph,
    ignore_loops: bool,
    mode: ReciprocityMode,
) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }
    if !graph.is_directed() {
        // Undirected graphs are reciprocal by construction; `ignore_loops`
        // and `mode` don't change that.
        return Ok(Some(1.0));
    }

    // Mirrors upstream's two-pointer merge over sorted in-/out-neighbours
    // per vertex, classifying matches as reciprocal and mismatches /
    // leftovers as non-reciprocal.
    let mut rec: u64 = 0;
    let mut nonrec: u64 = 0;
    let mut loops: u64 = 0;

    for v in 0..n {
        let outneis = graph.out_neighbors_vec(v)?;
        let inneis = graph.in_neighbors_vec(v)?;

        let mut ip = 0usize;
        let mut op = 0usize;
        while ip < inneis.len() && op < outneis.len() {
            match inneis[ip].cmp(&outneis[op]) {
                std::cmp::Ordering::Less => {
                    nonrec += 1;
                    ip += 1;
                }
                std::cmp::Ordering::Greater => {
                    nonrec += 1;
                    op += 1;
                }
                std::cmp::Ordering::Equal => {
                    if inneis[ip] == v {
                        // Self-loop: contributes to `loops` always; only
                        // joins `rec` if we are not ignoring loops.
                        loops += 1;
                        if !ignore_loops {
                            rec += 1;
                        }
                    } else {
                        rec += 1;
                    }
                    ip += 1;
                    op += 1;
                }
            }
        }
        // Tail: anything left over in either side is, by definition,
        // a one-way edge.
        nonrec += (inneis.len() - ip) as u64 + (outneis.len() - op) as u64;
    }

    #[allow(clippy::cast_precision_loss)]
    let result = match mode {
        ReciprocityMode::Default => {
            let denom = if ignore_loops {
                m as u64 - loops
            } else {
                m as u64
            };
            if denom == 0 {
                return Ok(None);
            }
            rec as f64 / denom as f64
        }
        ReciprocityMode::Ratio => {
            let denom = rec + nonrec;
            if denom == 0 {
                return Ok(None);
            }
            rec as f64 / denom as f64
        }
    };
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_returns_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(reciprocity(&g).unwrap(), None);
    }

    #[test]
    fn isolated_vertices_return_none() {
        let g = Graph::with_vertices(5);
        assert_eq!(reciprocity(&g).unwrap(), None);
    }

    #[test]
    fn undirected_graph_is_always_1() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn directed_one_way_edge_has_zero() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn directed_mutual_pair_has_one() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn directed_partial_reciprocity() {
        // 0 -> 1, 1 -> 0 (mutual), 0 -> 2 (one-way). 3 edges, 2 reciprocal → 2/3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        let two_thirds = 2.0_f64 / 3.0;
        assert_eq!(reciprocity(&g).unwrap(), Some(two_thirds));
    }

    #[test]
    fn directed_3_cycle_no_reciprocity() {
        // 0 -> 1 -> 2 -> 0: each edge has no reverse → 0.0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn directed_self_loop_is_counted_as_mutual() {
        // 0 -> 0 self-loop: 1 edge, mutual → 1.0.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }

    // ----- ALGO-PR-004b: ratio mode + ignore_loops -----

    #[test]
    fn ratio_and_default_diverge_with_one_way_edges() {
        // 0 → 1, 1 → 0 (mutual), 0 → 2 (one-way).
        //   m = 3, rec = 2, nonrec = 2 (the one-way 0→2 contributes
        //                               1 at v=0 and 1 at v=2).
        //   Default = rec / m         = 2/3 ≈ 0.667
        //   Ratio   = rec/(rec+nonrec) = 2/4 = 0.5
        // The two formulas only agree when every edge is mutual
        // (nonrec == 0 → ratio = rec/rec = 1.0 = m/m = default).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        let two_thirds = 2.0_f64 / 3.0;
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Default).unwrap(),
            Some(two_thirds)
        );
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap(),
            Some(0.5)
        );
    }

    #[test]
    fn ratio_and_default_agree_on_fully_mutual_graph() {
        // Every directed edge has its reverse → rec = m, nonrec = 0.
        let mut g = Graph::new(3, true).unwrap();
        for &(u, v) in &[(0u32, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Default).unwrap(),
            Some(1.0)
        );
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap(),
            Some(1.0)
        );
    }

    #[test]
    fn ignore_loops_drops_self_loop_from_default_denominator() {
        // 0 → 0 self-loop + 0 → 1 + 1 → 0: m = 3.
        // Default ignore_loops=false: rec = 3 (loop counted), denom = 3 → 1.0
        // Default ignore_loops=true:  rec = 2,                 denom = 2 → 1.0
        // Ratio  ignore_loops=true:   rec = 2, nonrec = 0      → 1.0
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Default).unwrap(),
            Some(1.0)
        );
        assert_eq!(
            reciprocity_with_mode(&g, true, ReciprocityMode::Default).unwrap(),
            Some(1.0)
        );
        assert_eq!(
            reciprocity_with_mode(&g, true, ReciprocityMode::Ratio).unwrap(),
            Some(1.0)
        );
    }

    #[test]
    fn ratio_mode_with_one_way_edge() {
        // 0 → 1 only: nonrec = 2 (both endpoints see one-way), rec = 0.
        // Ratio = 0 / (0 + 2) = 0.0.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap(),
            Some(0.0)
        );
    }

    #[test]
    fn ratio_mode_self_loop_only_with_ignore_loops_returns_none() {
        // Single self-loop, ignore_loops=true: rec = 0, nonrec = 0
        // (the self-loop is dropped from rec, and a self-loop produces
        // no `nonrec` increments because it is matched once on both
        // sides). Denom = 0 → result undefined → None.
        let mut g = Graph::new(1, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert_eq!(
            reciprocity_with_mode(&g, true, ReciprocityMode::Ratio).unwrap(),
            None
        );
    }

    #[test]
    fn default_mode_ignore_loops_only_loop_returns_none() {
        // Single self-loop, ignore_loops=true, Default mode:
        // denom = m - loops = 1 - 1 = 0 → None.
        let mut g = Graph::new(1, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert_eq!(
            reciprocity_with_mode(&g, true, ReciprocityMode::Default).unwrap(),
            None
        );
    }

    #[test]
    fn undirected_unconditional_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 0).unwrap();
        for &mode in &[ReciprocityMode::Default, ReciprocityMode::Ratio] {
            for &skip in &[false, true] {
                assert_eq!(
                    reciprocity_with_mode(&g, skip, mode).unwrap(),
                    Some(1.0),
                    "undirected mode={mode:?} ignore_loops={skip}"
                );
            }
        }
    }

    #[test]
    fn ratio_mode_3_cycle_zero() {
        // 0 → 1 → 2 → 0: rec = 0, nonrec = 6 (each vertex contributes
        // 1 in-tail + 1 out-tail with no match) → 0.0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            reciprocity_with_mode(&g, false, ReciprocityMode::Ratio).unwrap(),
            Some(0.0)
        );
    }
}
