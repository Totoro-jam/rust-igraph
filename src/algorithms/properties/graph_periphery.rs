//! Graph periphery and pseudo-peripheral vertices (ALGO-TR-038).
//!
//! The **periphery** of a graph is the set of vertices whose
//! eccentricity equals the diameter (maximum eccentricity).
//! A vertex is **pseudo-peripheral** if its eccentricity equals
//! the eccentricity of every vertex at maximum distance from it.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::algorithms::paths::radii::{EccMode, eccentricity_with_mode};
use crate::core::{Graph, IgraphResult, VertexId};

/// Return the periphery vertices of a graph.
///
/// A vertex belongs to the periphery if its eccentricity equals the
/// graph diameter (maximum eccentricity). For directed graphs, `mode`
/// controls BFS direction.
///
/// Returns an empty vector for an empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_periphery, EccMode};
///
/// // Path 0-1-2-3-4: periphery is {0, 4} (eccentricity 4 = diameter)
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,4)], false, Some(5)).unwrap();
/// let peri = graph_periphery(&g, EccMode::All).unwrap();
/// assert_eq!(peri, vec![0, 4]);
/// ```
pub fn graph_periphery(graph: &Graph, mode: EccMode) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let ecc = eccentricity_with_mode(graph, mode)?;

    let max_ecc = ecc.iter().copied().max().unwrap_or(0);

    let periphery: Vec<VertexId> = ecc
        .iter()
        .enumerate()
        .filter(|(_, e)| **e == max_ecc)
        .map(|(i, _)| i as VertexId)
        .collect();

    Ok(periphery)
}

/// Classify each vertex by its eccentricity class.
///
/// Returns a struct with center, periphery, and the eccentricity
/// vector for further analysis.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eccentricity_classes, EccMode};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,4)], false, Some(5)).unwrap();
/// let classes = eccentricity_classes(&g, EccMode::All).unwrap();
/// assert_eq!(classes.center, vec![2]);
/// assert_eq!(classes.periphery, vec![0, 4]);
/// assert_eq!(classes.radius, 2);
/// assert_eq!(classes.diameter, 4);
/// ```
pub fn eccentricity_classes(graph: &Graph, mode: EccMode) -> IgraphResult<EccentricityClasses> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(EccentricityClasses {
            eccentricities: Vec::new(),
            center: Vec::new(),
            periphery: Vec::new(),
            radius: 0,
            diameter: 0,
        });
    }

    let ecc = eccentricity_with_mode(graph, mode)?;

    let min_ecc = ecc.iter().copied().min().unwrap_or(0);
    let max_ecc = ecc.iter().copied().max().unwrap_or(0);

    let center: Vec<VertexId> = ecc
        .iter()
        .enumerate()
        .filter(|(_, e)| **e == min_ecc)
        .map(|(i, _)| i as VertexId)
        .collect();

    let periphery: Vec<VertexId> = ecc
        .iter()
        .enumerate()
        .filter(|(_, e)| **e == max_ecc)
        .map(|(i, _)| i as VertexId)
        .collect();

    Ok(EccentricityClasses {
        eccentricities: ecc,
        center,
        periphery,
        radius: min_ecc,
        diameter: max_ecc,
    })
}

/// Result of [`eccentricity_classes`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EccentricityClasses {
    /// Eccentricity of each vertex.
    pub eccentricities: Vec<u32>,
    /// Center vertices (minimum eccentricity).
    pub center: Vec<VertexId>,
    /// Periphery vertices (maximum eccentricity).
    pub periphery: Vec<VertexId>,
    /// Radius (minimum eccentricity).
    pub radius: u32,
    /// Diameter (maximum eccentricity).
    pub diameter: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
    }

    fn path6() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], false, Some(6)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    // --- graph_periphery ---

    #[test]
    fn gp_empty() {
        let g = Graph::with_vertices(0);
        assert!(graph_periphery(&g, EccMode::All).unwrap().is_empty());
    }

    #[test]
    fn gp_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(graph_periphery(&g, EccMode::All).unwrap(), vec![0]);
    }

    #[test]
    fn gp_path5() {
        // Endpoints have eccentricity 4 = diameter
        let peri = graph_periphery(&path5(), EccMode::All).unwrap();
        assert_eq!(peri, vec![0, 4]);
    }

    #[test]
    fn gp_path6() {
        let peri = graph_periphery(&path6(), EccMode::All).unwrap();
        assert_eq!(peri, vec![0, 5]);
    }

    #[test]
    fn gp_k4() {
        // All vertices have eccentricity 1 → all are periphery (and center)
        let peri = graph_periphery(&k4(), EccMode::All).unwrap();
        assert_eq!(peri, vec![0, 1, 2, 3]);
    }

    #[test]
    fn gp_cycle5() {
        // All vertices have eccentricity 2 → all periphery
        let peri = graph_periphery(&cycle5(), EccMode::All).unwrap();
        assert_eq!(peri, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn gp_star5() {
        // Leaves have eccentricity 2, center has 1 → periphery = leaves
        let peri = graph_periphery(&star5(), EccMode::All).unwrap();
        assert_eq!(peri, vec![1, 2, 3, 4]);
    }

    #[test]
    fn gp_no_edges() {
        // All vertices eccentricity 0 → all periphery
        let g = Graph::with_vertices(3);
        let peri = graph_periphery(&g, EccMode::All).unwrap();
        assert_eq!(peri, vec![0, 1, 2]);
    }

    // --- eccentricity_classes ---

    #[test]
    fn ec_path5() {
        let ec = eccentricity_classes(&path5(), EccMode::All).unwrap();
        assert_eq!(ec.radius, 2);
        assert_eq!(ec.diameter, 4);
        assert_eq!(ec.center, vec![2]);
        assert_eq!(ec.periphery, vec![0, 4]);
        assert_eq!(ec.eccentricities, vec![4, 3, 2, 3, 4]);
    }

    #[test]
    fn ec_k4() {
        let ec = eccentricity_classes(&k4(), EccMode::All).unwrap();
        assert_eq!(ec.radius, 1);
        assert_eq!(ec.diameter, 1);
        assert_eq!(ec.center, vec![0, 1, 2, 3]);
        assert_eq!(ec.periphery, vec![0, 1, 2, 3]);
    }

    #[test]
    fn ec_star5() {
        let ec = eccentricity_classes(&star5(), EccMode::All).unwrap();
        assert_eq!(ec.radius, 1);
        assert_eq!(ec.diameter, 2);
        assert_eq!(ec.center, vec![0]);
        assert_eq!(ec.periphery, vec![1, 2, 3, 4]);
    }

    #[test]
    fn ec_empty() {
        let g = Graph::with_vertices(0);
        let ec = eccentricity_classes(&g, EccMode::All).unwrap();
        assert_eq!(ec.radius, 0);
        assert_eq!(ec.diameter, 0);
        assert!(ec.center.is_empty());
        assert!(ec.periphery.is_empty());
    }

    #[test]
    fn ec_single() {
        let g = Graph::with_vertices(1);
        let ec = eccentricity_classes(&g, EccMode::All).unwrap();
        assert_eq!(ec.radius, 0);
        assert_eq!(ec.diameter, 0);
        assert_eq!(ec.center, vec![0]);
        assert_eq!(ec.periphery, vec![0]);
    }

    // --- cross-consistency ---

    #[test]
    fn center_and_periphery_cover_self_regular() {
        // In a vertex-transitive graph (cycle, complete), center == periphery
        let ec = eccentricity_classes(&cycle5(), EccMode::All).unwrap();
        assert_eq!(ec.center, ec.periphery);
    }

    #[test]
    fn radius_leq_diameter() {
        for g in &[path5(), path6(), k4(), cycle5(), star5()] {
            let ec = eccentricity_classes(g, EccMode::All).unwrap();
            assert!(ec.radius <= ec.diameter);
        }
    }

    #[test]
    fn diameter_leq_2_radius() {
        // For connected graphs: diameter <= 2 * radius
        for g in &[path5(), path6(), k4(), cycle5(), star5()] {
            let ec = eccentricity_classes(g, EccMode::All).unwrap();
            assert!(ec.diameter <= 2 * ec.radius);
        }
    }

    #[test]
    fn center_periphery_partition() {
        // Center ∩ periphery should be non-empty only when radius == diameter
        let ec = eccentricity_classes(&path5(), EccMode::All).unwrap();
        let center_set: std::collections::HashSet<_> = ec.center.iter().collect();
        let peri_set: std::collections::HashSet<_> = ec.periphery.iter().collect();
        let overlap: Vec<_> = center_set.intersection(&peri_set).collect();
        if ec.radius == ec.diameter {
            assert!(!overlap.is_empty());
        } else {
            assert!(overlap.is_empty());
        }
    }
}
