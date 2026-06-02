//! `mincut` (ALGO-FL-019) — global minimum cut with partition.
//!
//! Counterpart of `igraph_mincut` in
//! `references/igraph/src/flow/flow.c:1625`. Returns not just the
//! cut value, but also the actual cut edges and vertex partitions.
//!
//! ## Algorithm
//!
//! Same fixed-vertex strategy as [`super::mincut_value`]: pick vertex
//! 0, compute `st_mincut(0, v)` for every other vertex v (and `st_mincut(v, 0)`
//! for directed graphs). Track whichever pair yields the minimum cut.
//! Return the full [`StMincut`] from that winning pair.
//!
//! ## Corner cases
//!
//! * `vcount ≤ 1` → value `f64::INFINITY`, empty cut/partitions.
//! * Disconnected graph → value `0.0`, empty cut, partition splits
//!   at first disconnected pair found.

use crate::core::{Graph, IgraphError, IgraphResult};

use super::st_mincut::{StMincut, st_mincut};

/// Result of the global minimum cut computation.
///
/// Contains the cut value, edge IDs forming the cut, and the two
/// vertex partitions.
pub type Mincut = StMincut;

/// Compute the global minimum cut of a (possibly weighted) graph.
///
/// Returns the minimum-capacity edge set whose removal maximally
/// disconnects the graph, along with the resulting vertex partitions.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `capacity` — optional per-edge capacity vector. `None` means unit
///   capacities. When `Some(c)`, `c.len()` must equal `graph.ecount()`
///   and all entries must be finite non-negative.
///
/// # Returns
///
/// A [`Mincut`] containing:
/// * `value` — total capacity of the cut (`f64::INFINITY` if `vcount ≤ 1`).
/// * `cut` — edge IDs in the cut.
/// * `partition` / `partition2` — the two sides after cutting.
///
/// # Errors
///
/// Propagates errors from [`st_mincut`] (capacity validation, vertex
/// out-of-range, etc.).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mincut};
///
/// // Undirected path: 0-1-2. Min cut = 1 edge.
/// let mut g = Graph::new(3, false).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let mc = mincut(&g, None).unwrap();
/// assert!((mc.value - 1.0).abs() < 1e-12);
/// assert_eq!(mc.cut.len(), 1);
/// assert_eq!(mc.partition.len() + mc.partition2.len(), 3);
/// ```
pub fn mincut(graph: &Graph, capacity: Option<&[f64]>) -> IgraphResult<Mincut> {
    let n = graph.vcount();

    if let Some(c) = capacity {
        let m = graph.ecount();
        if c.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "mincut: capacity length {} does not match edge count {}",
                c.len(),
                m
            )));
        }
        for (i, &v) in c.iter().enumerate() {
            if !v.is_finite() || v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "mincut: capacity[{i}] = {v} is not a finite non-negative number"
                )));
            }
        }
    }

    if n <= 1 {
        return Ok(Mincut {
            value: f64::INFINITY,
            cut: Vec::new(),
            partition: if n == 1 { vec![0] } else { Vec::new() },
            partition2: Vec::new(),
        });
    }

    let directed = graph.is_directed();
    let mut best: Option<Mincut> = None;

    for v in 1..n {
        let result = st_mincut(graph, 0, v, capacity)?;
        let is_better = best.as_ref().is_none_or(|b| result.value < b.value);
        if is_better {
            if result.value == 0.0 {
                return Ok(result);
            }
            best = Some(result);
        }

        if directed {
            let result2 = st_mincut(graph, v, 0, capacity)?;
            let is_better2 = best.as_ref().is_none_or(|b| result2.value < b.value);
            if is_better2 {
                if result2.value == 0.0 {
                    return Ok(result2);
                }
                best = Some(result2);
            }
        }
    }

    best.ok_or(IgraphError::Internal(
        "mincut: no iteration completed despite n >= 2",
    ))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn mincut_empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert_eq!(mc.value, f64::INFINITY);
        assert!(mc.cut.is_empty());
    }

    #[test]
    fn mincut_single_vertex() {
        let g = Graph::new(1, false).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert_eq!(mc.value, f64::INFINITY);
        assert_eq!(mc.partition, vec![0]);
    }

    #[test]
    fn mincut_disconnected() {
        let g = Graph::new(3, false).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert!((mc.value - 0.0).abs() < 1e-12);
        assert!(mc.cut.is_empty());
    }

    #[test]
    fn mincut_path() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert!((mc.value - 1.0).abs() < 1e-12);
        assert_eq!(mc.cut.len(), 1);
        assert_eq!(mc.partition.len() + mc.partition2.len(), 3);
    }

    #[test]
    fn mincut_ring() {
        let mut g = Graph::new(5, false).unwrap();
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
            g.add_edge(u, v).unwrap();
        }
        let mc = mincut(&g, None).unwrap();
        assert!((mc.value - 2.0).abs() < 1e-12);
        assert_eq!(mc.cut.len(), 2);
    }

    #[test]
    fn mincut_weighted() {
        // Triangle with one weak edge
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let caps = vec![10.0, 1.0, 10.0];
        let mc = mincut(&g, Some(&caps)).unwrap();
        // Minimum cut separates vertex 1 from {0, 2} or vice versa
        // cutting edges 0→1 (cap 10) and 1→2 (cap 1) costs 11
        // Actually min is to cut edge 1→2 (cap 1) + ... no, need to disconnect
        // The minimum cut of a triangle: must cut 2 edges to disconnect
        // To isolate vertex 1: cut edges 0-1 (10) and 1-2 (1) = 11
        // To isolate vertex 2: cut edges 1-2 (1) and 2-0 (10) = 11
        // To isolate vertex 0: cut edges 0-1 (10) and 2-0 (10) = 20
        // So min is 11
        assert!((mc.value - 11.0).abs() < 1e-12);
    }

    #[test]
    fn mincut_directed_ring() {
        // Directed cycle: 0→1→2→0. Cutting any single edge disconnects
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert!((mc.value - 1.0).abs() < 1e-12);
        assert_eq!(mc.cut.len(), 1);
    }

    #[test]
    fn mincut_k4() {
        // Complete graph K4 — min cut = 3 (remove 3 edges to isolate one vertex)
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let mc = mincut(&g, None).unwrap();
        assert!((mc.value - 3.0).abs() < 1e-12);
        assert_eq!(mc.cut.len(), 3);
    }

    #[test]
    fn mincut_invalid_capacity_len() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(mincut(&g, Some(&[1.0, 2.0])).is_err());
    }
}
