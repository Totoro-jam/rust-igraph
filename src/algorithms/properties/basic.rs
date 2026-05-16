//! Basic graph metrics — density and mean shortest-path length (ALGO-PR-003).
//!
//! Counterparts of:
//! - `igraph_density()`             from `references/igraph/src/properties/basic_properties.c:71`
//! - `igraph_average_path_length()` from `references/igraph/src/paths/shortest_paths.c:329`
//!
//! Phase-1 minimal slice:
//! - **Density** matches upstream's default `loops = false` mode:
//!   `m / (n*(n-1)/2)` for undirected, `m / (n*(n-1))` for directed.
//!   Self-loops are not subtracted; if the graph has self-loops the
//!   result may exceed 1 (this matches upstream — caller's responsibility).
//! - **Mean distance** matches upstream's `unconn = true` default —
//!   unreachable pairs are skipped from the average. Returns `None`
//!   when no connected pairs exist (graph too small or fully disconnected).

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

/// Edge density of `graph`. Counterpart of
/// `igraph_density(_, NULL_weights, _, /*loops=*/false)`.
///
/// Returns `None` when:
/// - `vcount() == 0` (matches upstream's `IGRAPH_NAN`).
/// - `vcount() == 1` and `loops=false` (no possible edges).
///
/// Self-loops in the input graph are *not* removed; they inflate the
/// numerator. Use `simplify` (when it lands) to drop them first.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, density};
///
/// // K3 (triangle) on 3 vertices, undirected: 3 edges, max = 3 → 1.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(density(&g).unwrap(), Some(1.0));
/// ```
pub fn density(graph: &Graph) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    if n == 0 || n == 1 {
        return Ok(None);
    }
    let m = graph.ecount();
    let directed = graph.is_directed();

    // Sub-1 worst case for n: u32::try_from(n) is fine since n is u32 by type.
    let n_f = f64::from(n);
    #[allow(clippy::cast_precision_loss)]
    let m_f = m as f64;

    // Match upstream's float ordering exactly so f64 results agree to
    // the bit. See `igraph_density()` in
    // `references/igraph/src/properties/basic_properties.c:99-103`.
    let result = if directed {
        m_f / n_f / (n_f - 1.0)
    } else {
        m_f / n_f * 2.0 / (n_f - 1.0)
    };
    Ok(Some(result))
}

/// Mean unweighted shortest-path length over all reachable ordered pairs.
/// Counterpart of `igraph_average_path_length(_, NULL_weights, _, _, /*directed=*/true, /*unconn=*/true)`.
///
/// Returns `None` when no connected pairs exist (e.g. `vcount() < 2`,
/// or all pairs are unreachable). Edge directions matter for directed
/// graphs (uses `distances` which follows out-edges).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mean_distance};
///
/// // Path 0-1-2-3-4: ordered pairs at distance 1, 2, 3, 4 (4 each direction
/// // → 8, 6, 4, 2 contributions); 20 pairs total. Mean = 40/20 = 2.0.
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i + 1).unwrap(); }
/// assert_eq!(mean_distance(&g).unwrap(), Some(2.0));
/// ```
pub fn mean_distance(graph: &Graph) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(None);
    }
    let mut sum: u64 = 0;
    let mut count: u64 = 0;
    for v in 0..n {
        let d = distances(graph, v)?;
        let v_us = v as usize;
        for (target, &val) in d.iter().enumerate() {
            if target == v_us {
                continue;
            }
            if let Some(dist) = val {
                sum += u64::from(dist);
                count += 1;
            }
        }
    }
    if count == 0 {
        return Ok(None);
    }
    #[allow(clippy::cast_precision_loss)]
    let mean = (sum as f64) / (count as f64);
    Ok(Some(mean))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_empty_graph_is_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(density(&g).unwrap(), None);
    }

    #[test]
    fn density_singleton_is_none() {
        let g = Graph::with_vertices(1);
        assert_eq!(density(&g).unwrap(), None);
    }

    #[test]
    fn density_complete_undirected_is_one() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(density(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn density_no_edges_is_zero() {
        let g = Graph::with_vertices(5);
        assert_eq!(density(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn density_directed_complete_is_one() {
        // Directed complete graph (no self-loops): n*(n-1) edges = max.
        let mut g = Graph::new(3, true).unwrap();
        for u in 0..3u32 {
            for v in 0..3u32 {
                if u != v {
                    g.add_edge(u, v).unwrap();
                }
            }
        }
        assert_eq!(density(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn density_path_5_is_2_over_10() {
        // 4 edges among C(5,2)=10 possible → 0.4.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(density(&g).unwrap(), Some(0.4));
    }

    #[test]
    fn mean_distance_n_lt_2_is_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(mean_distance(&g).unwrap(), None);
        let g = Graph::with_vertices(1);
        assert_eq!(mean_distance(&g).unwrap(), None);
    }

    #[test]
    fn mean_distance_isolated_vertices_is_none() {
        let g = Graph::with_vertices(5);
        assert_eq!(mean_distance(&g).unwrap(), None);
    }

    #[test]
    fn mean_distance_path_5_is_2() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(mean_distance(&g).unwrap(), Some(2.0));
    }

    #[test]
    fn mean_distance_triangle_is_1() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(mean_distance(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn mean_distance_two_components_only_within_components() {
        // {0-1-2} and {3-4}: connected pairs are within each component.
        // {0-1-2}: 6 ordered pairs (0↔1=1, 0↔2=2, 1↔2=1) → sum 2*(1+2+1) = 8.
        // {3-4}: 2 ordered pairs (3↔4) → sum 2.
        // Total: 8 connected pairs, sum 10, mean = 10/8 = 1.25.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(mean_distance(&g).unwrap(), Some(1.25));
    }

    #[test]
    fn mean_distance_directed_uses_out_edges() {
        // 0 -> 1 -> 2: only 3 ordered reachable pairs: (0,1)=1, (0,2)=2, (1,2)=1.
        // Sum = 4, count = 3, mean = 4/3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let four_thirds = 4.0_f64 / 3.0;
        assert_eq!(mean_distance(&g).unwrap(), Some(four_thirds));
    }
}
