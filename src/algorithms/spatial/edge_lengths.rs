//! Spatial edge lengths (ALGO-GEO-002).
//!
//! Counterpart of `igraph_spatial_edge_lengths()` from
//! `references/igraph/src/spatial/edge_lengths.c`.
//!
//! Computes the Euclidean or Manhattan distance for each edge based on
//! vertex coordinates in arbitrary dimensions.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Distance metric for spatial edge length computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Euclidean (L2) distance.
    Euclidean,
    /// Manhattan (L1) distance.
    Manhattan,
}

/// Compute edge lengths from spatial vertex coordinates.
///
/// Given a graph and a matrix of vertex coordinates (one row per vertex,
/// arbitrary dimensionality), returns a `Vec<f64>` of length `ecount`
/// where entry `i` is the spatial distance between the endpoints of
/// edge `i` under the chosen metric.
///
/// # Errors
///
/// - `InvalidArgument` if `coords.len() != vcount`.
/// - `InvalidArgument` if `dim == 0` and `vcount > 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spatial_edge_lengths, DistanceMetric};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// // 2D coords: (0,0), (3,0), (3,4)
/// let coords = vec![vec![0.0, 0.0], vec![3.0, 0.0], vec![3.0, 4.0]];
/// let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
/// assert!((lengths[0] - 3.0).abs() < 1e-10);
/// assert!((lengths[1] - 4.0).abs() < 1e-10);
/// ```
pub fn spatial_edge_lengths(
    graph: &Graph,
    coords: &[Vec<f64>],
    metric: DistanceMetric,
) -> IgraphResult<Vec<f64>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount();

    if coords.len() != vcount {
        return Err(IgraphError::InvalidArgument(format!(
            "spatial_edge_lengths: coords length {} != vertex count {vcount}",
            coords.len()
        )));
    }

    if vcount > 0 {
        let dim = coords[0].len();
        if dim == 0 {
            return Err(IgraphError::InvalidArgument(
                "spatial_edge_lengths: coordinates must not be zero-dimensional".into(),
            ));
        }
        for (i, row) in coords.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "spatial_edge_lengths: coord row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
    }

    let mut lengths: Vec<f64> = Vec::with_capacity(ecount);

    for eid in 0..ecount {
        let eid_u32 = u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        let (from, to) = graph.edge(eid_u32)?;
        let from_coords = &coords[from as usize];
        let to_coords = &coords[to as usize];

        let length = match metric {
            DistanceMetric::Euclidean => {
                let sum_sq: f64 = from_coords
                    .iter()
                    .zip(to_coords.iter())
                    .map(|(&a, &b)| (a - b) * (a - b))
                    .sum();
                sum_sq.sqrt()
            }
            DistanceMetric::Manhattan => from_coords
                .iter()
                .zip(to_coords.iter())
                .map(|(&a, &b)| (a - b).abs())
                .sum(),
        };

        lengths.push(length);
    }

    Ok(lengths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let lengths = spatial_edge_lengths(&g, &[], DistanceMetric::Euclidean).unwrap();
        assert!(lengths.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(3);
        let coords = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!(lengths.is_empty());
    }

    #[test]
    fn euclidean_2d() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let coords = vec![vec![0.0, 0.0], vec![3.0, 0.0], vec![0.0, 4.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 3.0).abs() < 1e-10);
        assert!((lengths[1] - 5.0).abs() < 1e-10);
        assert!((lengths[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn manhattan_2d() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let coords = vec![vec![0.0, 0.0], vec![3.0, 0.0], vec![0.0, 4.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Manhattan).unwrap();
        assert!((lengths[0] - 3.0).abs() < 1e-10);
        assert!((lengths[1] - 7.0).abs() < 1e-10); // |3-0| + |0-4| = 7
        assert!((lengths[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn euclidean_3d() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let coords = vec![vec![0.0, 0.0, 0.0], vec![1.0, 2.0, 2.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 3.0).abs() < 1e-10); // sqrt(1+4+4)=3
    }

    #[test]
    fn manhattan_3d() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let coords = vec![vec![0.0, 0.0, 0.0], vec![1.0, 2.0, 3.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Manhattan).unwrap();
        assert!((lengths[0] - 6.0).abs() < 1e-10); // 1+2+3=6
    }

    #[test]
    fn self_loop_zero_distance() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let coords = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 0.0).abs() < 1e-10);
        assert!((lengths[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn high_dimensional() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        // 5D: distance = sqrt(5*1^2) = sqrt(5)
        let coords = vec![vec![0.0; 5], vec![1.0; 5]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 5.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn coords_length_mismatch() {
        let g = Graph::with_vertices(3);
        let coords = vec![vec![0.0], vec![1.0]]; // only 2, need 3
        let err = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn zero_dimensional_error() {
        let g = Graph::with_vertices(2);
        let coords = vec![vec![], vec![]];
        let err = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let g = Graph::with_vertices(2);
        let coords = vec![vec![0.0, 0.0], vec![1.0]]; // dim mismatch
        let err = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();
        let coords = vec![vec![0.0, 0.0], vec![3.0, 4.0], vec![1.0, 0.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 5.0).abs() < 1e-10); // 0→1: sqrt(9+16)
        assert!((lengths[1] - 1.0).abs() < 1e-10); // 2→0: sqrt(1)
    }

    #[test]
    fn single_dimension() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let coords = vec![vec![0.0], vec![5.0], vec![2.0]];
        let lengths = spatial_edge_lengths(&g, &coords, DistanceMetric::Euclidean).unwrap();
        assert!((lengths[0] - 5.0).abs() < 1e-10);
        assert!((lengths[1] - 3.0).abs() < 1e-10);
    }
}
