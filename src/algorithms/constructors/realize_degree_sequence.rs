//! Degree sequence realization (ALGO-CO-033).
//!
//! Counterpart of `igraph_realize_degree_sequence()` from
//! `references/igraph/src/misc/degree_sequence.cpp`.
//!
//! Constructs a simple graph with a given degree sequence using the
//! Havel-Hakimi algorithm.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Method for selecting the next vertex during degree sequence realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizeDegseqMethod {
    /// Select the vertex with largest remaining degree first.
    /// Classic Havel-Hakimi; tends to produce high positive assortativity.
    Largest,
    /// Select the vertex with smallest remaining degree first.
    /// Tends to produce connected graphs when a connected realization exists.
    Smallest,
    /// Select vertices in index order (position in the degree vector).
    Index,
}

/// Realize an undirected simple graph from a degree sequence.
///
/// Uses the Havel-Hakimi algorithm: repeatedly pick a vertex and connect
/// it to the vertices with the highest remaining degrees.
///
/// # Arguments
///
/// * `degrees` — the target degree for each vertex.
/// * `method` — vertex selection order (see [`RealizeDegseqMethod`]).
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - The degree sum is odd (no realization exists).
/// - Any degree exceeds `n - 1` (no simple realization exists).
/// - The sequence is not graphical (Havel-Hakimi fails).
///
/// # Examples
///
/// ```
/// use rust_igraph::{realize_degree_sequence, RealizeDegseqMethod};
///
/// // Realize a path graph: degrees [1, 2, 2, 1]
/// let g = realize_degree_sequence(&[1, 2, 2, 1], RealizeDegseqMethod::Largest).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 3);
/// ```
pub fn realize_degree_sequence(
    degrees: &[u32],
    method: RealizeDegseqMethod,
) -> IgraphResult<Graph> {
    let n = degrees.len();

    if n == 0 {
        return Graph::new(0, false);
    }

    // Validate: no degree >= n (simple graph constraint)
    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("vertex count exceeds u32::MAX".to_string()))?;

    for (i, &d) in degrees.iter().enumerate() {
        if d >= n_u32 {
            return Err(IgraphError::InvalidArgument(format!(
                "degree[{i}] = {d} >= n = {n_u32}, cannot realize as simple graph"
            )));
        }
    }

    // Sum of degrees must be even
    let deg_sum: u64 = degrees.iter().map(|&d| u64::from(d)).sum();
    if deg_sum % 2 != 0 {
        return Err(IgraphError::InvalidArgument(
            "degree sum must be even for an undirected graph".to_string(),
        ));
    }

    let num_edges = (deg_sum / 2) as usize;

    if num_edges == 0 {
        return Graph::new(n_u32, false);
    }

    // Build (vertex_index, remaining_degree) pairs
    let mut vd: Vec<(u32, u32)> = degrees
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as u32;
            (idx, d)
        })
        .collect();

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(num_edges);

    for _ in 0..n {
        // Select hub vertex based on method
        let hub_pos = select_hub(&vd, method);
        if hub_pos.is_none() {
            break;
        }
        let hub_pos = hub_pos.unwrap();
        let (hub_vertex, hub_degree) = vd[hub_pos];

        if hub_degree == 0 {
            break;
        }

        // Remove hub from the working list
        vd.swap_remove(hub_pos);

        // Sort remaining by degree descending to find top-d neighbors
        vd.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let d = hub_degree as usize;
        if d > vd.len() {
            return Err(IgraphError::InvalidArgument(
                "degree sequence is not graphical (not realizable as simple graph)".to_string(),
            ));
        }

        // Check that the d-th highest degree vertex has degree > 0
        if vd[d - 1].1 == 0 {
            return Err(IgraphError::InvalidArgument(
                "degree sequence is not graphical (not realizable as simple graph)".to_string(),
            ));
        }

        // Connect hub to the d vertices with highest remaining degree
        for item in vd.iter_mut().take(d) {
            edges.push((hub_vertex, item.0));
            item.1 -= 1;
        }
    }

    // Verify all degrees are exhausted
    for &(v, d) in &vd {
        if d != 0 {
            return Err(IgraphError::InvalidArgument(format!(
                "degree sequence is not graphical: vertex {v} has {d} remaining degree"
            )));
        }
    }

    let mut graph = Graph::new(n_u32, false)?;
    graph.add_edges(edges)?;
    Ok(graph)
}

fn select_hub(vd: &[(u32, u32)], method: RealizeDegseqMethod) -> Option<usize> {
    if vd.is_empty() {
        return None;
    }

    match method {
        RealizeDegseqMethod::Largest => {
            let mut best = 0;
            for (i, item) in vd.iter().enumerate().skip(1) {
                if item.1 > vd[best].1 {
                    best = i;
                }
            }
            if vd[best].1 == 0 { None } else { Some(best) }
        }
        RealizeDegseqMethod::Smallest => {
            let mut best: Option<usize> = None;
            for (i, item) in vd.iter().enumerate() {
                if item.1 == 0 {
                    continue;
                }
                match best {
                    None => best = Some(i),
                    Some(b) => {
                        if item.1 < vd[b].1 {
                            best = Some(i);
                        }
                    }
                }
            }
            best
        }
        RealizeDegseqMethod::Index => {
            // Find the first non-zero degree by original index order
            let mut best: Option<usize> = None;
            for (i, item) in vd.iter().enumerate() {
                if item.1 == 0 {
                    continue;
                }
                match best {
                    None => best = Some(i),
                    Some(b) => {
                        if item.0 < vd[b].0 {
                            best = Some(i);
                        }
                    }
                }
            }
            best
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_degrees(graph: &Graph, expected: &[u32]) {
        let n = graph.vcount();
        assert_eq!(n as usize, expected.len());
        for v in 0..n {
            let deg = graph.degree(v).unwrap();
            assert_eq!(
                deg, expected[v as usize] as usize,
                "vertex {v}: got degree {deg}, expected {}",
                expected[v as usize]
            );
        }
    }

    #[test]
    fn empty_sequence() {
        let g = realize_degree_sequence(&[], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn all_zeros() {
        let g = realize_degree_sequence(&[0, 0, 0], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_edge() {
        let g = realize_degree_sequence(&[1, 1], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        verify_degrees(&g, &[1, 1]);
    }

    #[test]
    fn path_graph() {
        let g = realize_degree_sequence(&[1, 2, 2, 1], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 3);
        verify_degrees(&g, &[1, 2, 2, 1]);
    }

    #[test]
    fn complete_graph_k4() {
        let g = realize_degree_sequence(&[3, 3, 3, 3], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 6);
        verify_degrees(&g, &[3, 3, 3, 3]);
    }

    #[test]
    fn star_graph() {
        let g = realize_degree_sequence(&[4, 1, 1, 1, 1], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        verify_degrees(&g, &[4, 1, 1, 1, 1]);
    }

    #[test]
    fn regular_graph_3() {
        // 3-regular graph on 4 vertices = K4
        let g = realize_degree_sequence(&[3, 3, 3, 3], RealizeDegseqMethod::Smallest).unwrap();
        assert_eq!(g.ecount(), 6);
        verify_degrees(&g, &[3, 3, 3, 3]);
    }

    #[test]
    fn odd_sum_fails() {
        let result = realize_degree_sequence(&[1, 2, 2], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn degree_too_large_fails() {
        // degree 4 with only 4 vertices → degree >= n
        let result = realize_degree_sequence(&[4, 1, 1, 1], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn non_graphical_sequence_fails() {
        // [3, 3, 3, 1] has even sum (10) but is not graphical
        let result = realize_degree_sequence(&[3, 3, 3, 1], RealizeDegseqMethod::Largest);
        assert!(result.is_err());
    }

    #[test]
    fn method_smallest_connected() {
        // The smallest method tends to produce connected graphs
        let g = realize_degree_sequence(&[2, 2, 2, 2, 2], RealizeDegseqMethod::Smallest).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 5);
        verify_degrees(&g, &[2, 2, 2, 2, 2]);
    }

    #[test]
    fn method_index() {
        let g = realize_degree_sequence(&[2, 2, 1, 1], RealizeDegseqMethod::Index).unwrap();
        assert_eq!(g.ecount(), 3);
        verify_degrees(&g, &[2, 2, 1, 1]);
    }

    #[test]
    fn larger_graph() {
        // Realize a degree sequence for a graph with 8 vertices
        let g = realize_degree_sequence(&[3, 3, 2, 2, 2, 2, 1, 1], RealizeDegseqMethod::Largest)
            .unwrap();
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 8);
        verify_degrees(&g, &[3, 3, 2, 2, 2, 2, 1, 1]);
    }

    #[test]
    fn single_vertex_zero_degree() {
        let g = realize_degree_sequence(&[0], RealizeDegseqMethod::Largest).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }
}
