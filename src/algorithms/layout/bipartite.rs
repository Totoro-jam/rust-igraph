//! Bipartite layout (ALGO-LO-006).
//!
//! Places vertices of a bipartite graph in two rows, then optimizes
//! horizontal positions via the Sugiyama crossing minimization.

use crate::algorithms::layout::sugiyama::{SugiyamaParams, layout_sugiyama};
use crate::core::Graph;
use crate::core::{IgraphError, IgraphResult};

/// Compute a bipartite layout.
///
/// Vertices are placed in two rows based on their boolean type. Horizontal
/// positions are optimized to minimize edge crossings using the Sugiyama
/// algorithm.
///
/// # Arguments
///
/// * `graph` — the input graph (need not actually be bipartite; any graph
///   with a two-coloring works).
/// * `types` — boolean slice of length `vcount`: `true` places the vertex
///   in row 0 (top), `false` in row 1 (bottom).
/// * `hgap` — preferred minimum horizontal gap between vertices in the
///   same row.
/// * `vgap` — vertical distance between the two rows.
/// * `maxiter` — maximum crossing-minimization iterations (100 is typical).
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if `types.len() != graph.vcount()` or if
/// `hgap` is negative.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_bipartite};
///
/// // K_{2,3}: vertices 0,1 in one part, 2,3,4 in the other
/// let mut g = Graph::with_vertices(5);
/// for &u in &[0u32, 1] {
///     for &v in &[2u32, 3, 4] {
///         g.add_edge(u, v).unwrap();
///     }
/// }
///
/// let types = vec![true, true, false, false, false];
/// let pos = layout_bipartite(&g, &types, 1.0, 1.0, 100).unwrap();
/// assert_eq!(pos.len(), 5);
/// // Top-row vertices share the same y
/// assert!((pos[0][1] - pos[1][1]).abs() < 1e-10);
/// // Bottom-row vertices share a different y
/// assert!((pos[2][1] - pos[3][1]).abs() < 1e-10);
/// assert!((pos[0][1] - pos[2][1]).abs() > 0.5);
/// ```
pub fn layout_bipartite(
    graph: &Graph,
    types: &[bool],
    hgap: f64,
    vgap: f64,
    maxiter: u32,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;

    if types.len() != n {
        return Err(IgraphError::InvalidArgument(format!(
            "types length ({}) must equal vertex count ({})",
            types.len(),
            n
        )));
    }
    if hgap < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "hgap cannot be negative".into(),
        ));
    }

    let layers: Vec<u32> = types.iter().map(|&t| u32::from(!t)).collect();

    let params = SugiyamaParams {
        hgap,
        vgap,
        maxiter,
    };

    let result = layout_sugiyama(graph, Some(&layers), &params)?;
    Ok(result.positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bipartite_k23() {
        let mut g = Graph::with_vertices(5);
        for &u in &[0u32, 1] {
            for &v in &[2u32, 3, 4] {
                g.add_edge(u, v).unwrap();
            }
        }
        let types = vec![true, true, false, false, false];
        let pos = layout_bipartite(&g, &types, 1.0, 1.0, 100).unwrap();
        assert_eq!(pos.len(), 5);
        assert!((pos[0][1] - pos[1][1]).abs() < 1e-10);
        assert!((pos[2][1] - pos[3][1]).abs() < 1e-10);
        assert!((pos[2][1] - pos[4][1]).abs() < 1e-10);
        assert!((pos[0][1] - pos[2][1]).abs() > 0.5);
    }

    #[test]
    fn test_bipartite_empty() {
        let g = Graph::with_vertices(0);
        let pos = layout_bipartite(&g, &[], 1.0, 1.0, 100).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_bipartite_single_vertex() {
        let g = Graph::with_vertices(1);
        let pos = layout_bipartite(&g, &[true], 1.0, 1.0, 100).unwrap();
        assert_eq!(pos.len(), 1);
    }

    #[test]
    fn test_bipartite_wrong_types_len() {
        let g = Graph::with_vertices(3);
        let result = layout_bipartite(&g, &[true, false], 1.0, 1.0, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_bipartite_negative_hgap() {
        let g = Graph::with_vertices(2);
        let result = layout_bipartite(&g, &[true, false], -1.0, 1.0, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_bipartite_all_same_type() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let types = vec![true, true, true, true];
        let pos = layout_bipartite(&g, &types, 1.0, 1.0, 100).unwrap();
        assert_eq!(pos.len(), 4);
        // All in same row
        for i in 1..4 {
            assert!((pos[i][1] - pos[0][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_bipartite_no_edges() {
        let g = Graph::with_vertices(4);
        let types = vec![true, true, false, false];
        let pos = layout_bipartite(&g, &types, 1.0, 1.0, 100).unwrap();
        assert_eq!(pos.len(), 4);
        // Two rows
        assert!((pos[0][1] - pos[1][1]).abs() < 1e-10);
        assert!((pos[2][1] - pos[3][1]).abs() < 1e-10);
    }

    #[test]
    fn test_bipartite_separation() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let types = vec![true, true, false, false];
        let pos = layout_bipartite(&g, &types, 2.0, 3.0, 100).unwrap();
        // Vertices in the same row should be at least hgap apart
        let diff_x = (pos[0][0] - pos[1][0]).abs();
        assert!(diff_x >= 2.0 - 1e-10, "hgap not respected: diff_x={diff_x}");
        // Rows should be vgap apart
        let diff_y = (pos[0][1] - pos[2][1]).abs();
        assert!(
            (diff_y - 3.0).abs() < 1e-10,
            "vgap not respected: diff_y={diff_y}"
        );
    }
}
