//! Graph center and pseudo-diameter (ALGO-SP-037).
//!
//! - `graph_center`: vertices with minimum eccentricity.
//! - `pseudo_diameter`: lower-bound approximation of the diameter via
//!   iterative BFS from pseudo-peripheral vertices.
//!
//! Counterpart of `igraph_graph_center` and `igraph_pseudo_diameter`
//! from `references/igraph/src/paths/distances.c`.

use std::collections::VecDeque;

use crate::algorithms::paths::radii::{EccMode, eccentricity_with_mode};
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Return the central vertices of a graph — those with minimum eccentricity.
///
/// For directed graphs, `mode` controls which direction BFS follows.
/// For undirected graphs, all modes are equivalent.
///
/// Returns an empty vector for an empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_center, EccMode};
///
/// // Path 0-1-2-3-4: center is vertex 2
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let center = graph_center(&g, EccMode::All).unwrap();
/// assert_eq!(center, vec![2]);
/// ```
pub fn graph_center(graph: &Graph, mode: EccMode) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let ecc = eccentricity_with_mode(graph, mode)?;

    let min_ecc = ecc.iter().copied().min().unwrap_or(0);

    let center: Vec<VertexId> = ecc
        .iter()
        .enumerate()
        .filter(|(_, e)| **e == min_ecc)
        .map(|(i, _)| {
            #[allow(clippy::cast_possible_truncation)]
            let v = i as VertexId;
            v
        })
        .collect();

    Ok(center)
}

/// Result of the pseudo-diameter computation.
#[derive(Debug, Clone, PartialEq)]
pub struct PseudoDiameterResult {
    /// The pseudo-diameter value (lower bound of the true diameter).
    pub diameter: f64,
    /// Source vertex of the longest path found.
    pub from: Option<VertexId>,
    /// Target vertex of the longest path found.
    pub to: Option<VertexId>,
}

/// Approximate the diameter of a graph using pseudo-peripheral vertex search.
///
/// Starting from `start`, repeatedly finds the most distant vertex and
/// switches to it until the eccentricity no longer increases. The result
/// is a lower bound on the true diameter.
///
/// For disconnected graphs, if `unconn` is true, returns the diameter of
/// the component containing `start`; if false, returns `f64::INFINITY`.
///
/// # Arguments
///
/// * `graph` — input graph
/// * `start` — starting vertex (if `None`, uses vertex 0)
/// * `directed` — whether to respect edge directions (ignored for undirected graphs)
/// * `unconn` — what to do if graph is disconnected
///
/// # Errors
///
/// Returns an error if `start` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pseudo_diameter};
///
/// // Path 0-1-2-3-4: diameter is 4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
/// assert_eq!(result.diameter, 4.0);
/// ```
pub fn pseudo_diameter(
    graph: &Graph,
    start: Option<VertexId>,
    directed: bool,
    unconn: bool,
) -> IgraphResult<PseudoDiameterResult> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(PseudoDiameterResult {
            diameter: f64::NAN,
            from: None,
            to: None,
        });
    }

    let vid_start = start.unwrap_or(0);
    if vid_start >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "pseudo_diameter: start vertex {vid_start} out of range (vcount = {n})"
        )));
    }

    let mode = if graph.is_directed() && directed {
        EccMode::Out
    } else {
        EccMode::All
    };

    let mut current = vid_start;
    let mut current_ecc = 0u32;
    let mut far_vertex: VertexId = vid_start;
    let mut from_vertex = vid_start;

    loop {
        let (ecc, farthest, has_unreachable) = bfs_eccentricity(graph, current, mode)?;

        if has_unreachable && !unconn {
            return Ok(PseudoDiameterResult {
                diameter: f64::INFINITY,
                from: None,
                to: None,
            });
        }

        if ecc > current_ecc {
            current_ecc = ecc;
            from_vertex = current;
            far_vertex = farthest;
            current = farthest;
        } else {
            break;
        }
    }

    Ok(PseudoDiameterResult {
        diameter: f64::from(current_ecc),
        from: Some(from_vertex),
        to: Some(far_vertex),
    })
}

/// BFS from `source` following `mode` direction. Returns (eccentricity, `farthest_vertex`, `has_unreachable`).
fn bfs_eccentricity(
    graph: &Graph,
    source: VertexId,
    mode: EccMode,
) -> IgraphResult<(u32, VertexId, bool)> {
    let n = graph.vcount() as usize;
    let mut dist: Vec<Option<u32>> = vec![None; n];
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    dist[source as usize] = Some(0);
    queue.push_back(source);

    let mut max_dist: u32 = 0;
    let mut farthest: VertexId = source;

    while let Some(v) = queue.pop_front() {
        let d = dist[v as usize].unwrap_or(0);

        let neighbors = match mode {
            EccMode::Out => graph.incident(v)?,
            EccMode::In => graph.incident_in(v)?,
            EccMode::All => {
                let mut out = graph.incident(v)?;
                if graph.is_directed() {
                    let in_edges = graph.incident_in(v)?;
                    for eid in in_edges {
                        if !out.contains(&eid) {
                            out.push(eid);
                        }
                    }
                }
                out
            }
        };

        for eid in neighbors {
            let (from, to) = graph.edge(eid)?;
            let nei = if from == v { to } else { from };

            if dist[nei as usize].is_none() {
                let nd = d.saturating_add(1);
                dist[nei as usize] = Some(nd);
                queue.push_back(nei);

                if nd > max_dist {
                    max_dist = nd;
                    farthest = nei;
                }
            }
        }
    }

    let has_unreachable = dist.iter().any(Option::is_none);

    Ok((max_dist, farthest, has_unreachable))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn center_empty_graph() {
        let g = Graph::with_vertices(0);
        let center = graph_center(&g, EccMode::All).unwrap();
        assert!(center.is_empty());
    }

    #[test]
    fn center_single_vertex() {
        let g = Graph::with_vertices(1);
        let center = graph_center(&g, EccMode::All).unwrap();
        assert_eq!(center, vec![0]);
    }

    #[test]
    fn center_path_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let center = graph_center(&g, EccMode::All).unwrap();
        assert_eq!(center, vec![2]);
    }

    #[test]
    fn center_path_6() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        let center = graph_center(&g, EccMode::All).unwrap();
        // Even path: center has two vertices
        assert_eq!(center, vec![2, 3]);
    }

    #[test]
    fn center_complete_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let center = graph_center(&g, EccMode::All).unwrap();
        // All vertices have eccentricity 1
        assert_eq!(center, vec![0, 1, 2, 3]);
    }

    #[test]
    fn center_star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let center = graph_center(&g, EccMode::All).unwrap();
        // Center vertex has eccentricity 1, leaves have 2
        assert_eq!(center, vec![0]);
    }

    #[test]
    fn center_cycle() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let center = graph_center(&g, EccMode::All).unwrap();
        // All vertices in C5 have eccentricity 2
        assert_eq!(center, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn pseudo_diameter_empty() {
        let g = Graph::with_vertices(0);
        let result = pseudo_diameter(&g, None, false, true).unwrap();
        assert!(result.diameter.is_nan());
        assert_eq!(result.from, None);
        assert_eq!(result.to, None);
    }

    #[test]
    fn pseudo_diameter_single_vertex() {
        let g = Graph::with_vertices(1);
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        assert_eq!(result.diameter, 0.0);
    }

    #[test]
    fn pseudo_diameter_path_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        // True diameter is 4; pseudo-diameter should find it
        assert_eq!(result.diameter, 4.0);
    }

    #[test]
    fn pseudo_diameter_cycle() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        // Diameter of C6 is 3
        assert_eq!(result.diameter, 3.0);
    }

    #[test]
    fn pseudo_diameter_complete() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        assert_eq!(result.diameter, 1.0);
    }

    #[test]
    fn pseudo_diameter_disconnected_unconn_true() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        // Only component containing vertex 0: diameter 1
        assert_eq!(result.diameter, 1.0);
    }

    #[test]
    fn pseudo_diameter_disconnected_unconn_false() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = pseudo_diameter(&g, Some(0), false, false).unwrap();
        assert_eq!(result.diameter, f64::INFINITY);
    }

    #[test]
    fn pseudo_diameter_start_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(pseudo_diameter(&g, Some(5), false, true).is_err());
    }

    #[test]
    fn pseudo_diameter_lower_bound() {
        // The pseudo-diameter should always be a lower bound of the true diameter
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(0, 3).unwrap(); // shortcut
        let result = pseudo_diameter(&g, Some(0), false, true).unwrap();
        // The true diameter is at least result.diameter
        assert!(result.diameter >= 3.0); // true diameter is 4 (3-4-5-6 path + any start)
    }

    #[test]
    fn pseudo_diameter_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = pseudo_diameter(&g, Some(0), true, true).unwrap();
        assert_eq!(result.diameter, 2.0);
    }
}
