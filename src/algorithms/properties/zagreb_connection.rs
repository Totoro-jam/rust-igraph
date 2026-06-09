//! Zagreb connection indices (ALGO-TR-051).
//!
//! The **connection number** `τ(v)` of a vertex v is the number of
//! vertices at distance exactly 2 from v (i.e., second neighbours).
//!
//! - **First Zagreb connection index** `ZC₁(G) = Σ_{v∈V} τ(v)²`
//! - **Second Zagreb connection index** `ZC₂(G) = Σ_{(u,v)∈E} τ(u)·τ(v)`
//! - **Modified first Zagreb connection** `ZC₁*(G) = Σ_{(u,v)∈E} (τ(u)+τ(v))`
//!
//! Introduced by Ali & Trinajstić (2018). These indices account for
//! second-order connectivity and provide better predictive power than
//! classical Zagreb indices for some molecular properties.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::HashSet;

fn connection_numbers(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let mut tau = vec![0.0_f64; n];

    for v in 0..graph.vcount() {
        let mut dist1 = HashSet::new();
        for nb in graph.neighbors(v)? {
            dist1.insert(nb);
        }
        let mut dist2 = HashSet::new();
        for &nb1 in &dist1 {
            for nb2 in graph.neighbors(nb1)? {
                if nb2 != v && !dist1.contains(&nb2) {
                    dist2.insert(nb2);
                }
            }
        }
        tau[v as usize] = dist2.len() as f64;
    }

    Ok(tau)
}

/// Compute the first Zagreb connection index.
///
/// `ZC₁(G) = Σ_{v∈V} τ(v)²`
///
/// where `τ(v)` = number of vertices at distance exactly 2 from v.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_zagreb_connection};
///
/// // Path 0-1-2: τ(0)=1, τ(1)=0, τ(2)=1
/// // ZC₁ = 1+0+1 = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((first_zagreb_connection(&g).unwrap() - 2.0).abs() < 1e-10);
/// ```
pub fn first_zagreb_connection(graph: &Graph) -> IgraphResult<f64> {
    let tau = connection_numbers(graph)?;
    Ok(tau.iter().map(|t| t * t).sum())
}

/// Compute the second Zagreb connection index.
///
/// `ZC₂(G) = Σ_{(u,v)∈E} τ(u) · τ(v)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_zagreb_connection};
///
/// // Path 0-1-2: τ=[1,0,1]
/// // (0,1): 1·0=0, (1,2): 0·1=0 → ZC₂=0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((second_zagreb_connection(&g).unwrap()).abs() < 1e-10);
/// ```
pub fn second_zagreb_connection(graph: &Graph) -> IgraphResult<f64> {
    let tau = connection_numbers(graph)?;
    let mut zc2 = 0.0_f64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        zc2 += tau[u as usize] * tau[v as usize];
    }
    Ok(zc2)
}

/// Compute the modified first Zagreb connection index.
///
/// `ZC₁*(G) = Σ_{(u,v)∈E} (τ(u) + τ(v))`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modified_first_zagreb_connection};
///
/// // K_3: τ(v)=0 for all v (all pairs are direct neighbours)
/// // ZC₁* = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((modified_first_zagreb_connection(&g).unwrap()).abs() < 1e-10);
/// ```
pub fn modified_first_zagreb_connection(graph: &Graph) -> IgraphResult<f64> {
    let tau = connection_numbers(graph)?;
    let mut zcs = 0.0_f64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        zcs += tau[u as usize] + tau[v as usize];
    }
    Ok(zcs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn cycle6() -> Graph {
        Graph::from_edges(
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)],
            false,
            Some(6),
        )
        .unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- connection_numbers (helper) ---

    #[test]
    fn tau_empty() {
        let g = Graph::with_vertices(0);
        let tau = connection_numbers(&g).unwrap();
        assert!(tau.is_empty());
    }

    #[test]
    fn tau_single_vertex() {
        let g = Graph::with_vertices(1);
        let tau = connection_numbers(&g).unwrap();
        assert!((tau[0]).abs() < 1e-10);
    }

    #[test]
    fn tau_single_edge() {
        // 0-1: τ(0)=0 (no vertex at dist 2), τ(1)=0
        let tau = connection_numbers(&single_edge()).unwrap();
        assert!((tau[0]).abs() < 1e-10);
        assert!((tau[1]).abs() < 1e-10);
    }

    #[test]
    fn tau_path3() {
        // 0-1-2: τ(0)=1(v2), τ(1)=0, τ(2)=1(v0)
        let tau = connection_numbers(&path3()).unwrap();
        assert!((tau[0] - 1.0).abs() < 1e-10);
        assert!((tau[1]).abs() < 1e-10);
        assert!((tau[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tau_path4() {
        // 0-1-2-3: τ(0)=1(v2), τ(1)=1(v3), τ(2)=1(v0), τ(3)=1(v1)
        let tau = connection_numbers(&path4()).unwrap();
        assert!((tau[0] - 1.0).abs() < 1e-10);
        assert!((tau[1] - 1.0).abs() < 1e-10);
        assert!((tau[2] - 1.0).abs() < 1e-10);
        assert!((tau[3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tau_path5() {
        // 0-1-2-3-4: τ(0)=1, τ(1)=1, τ(2)=2, τ(3)=1, τ(4)=1
        let tau = connection_numbers(&path5()).unwrap();
        assert!((tau[0] - 1.0).abs() < 1e-10);
        assert!((tau[1] - 1.0).abs() < 1e-10);
        assert!((tau[2] - 2.0).abs() < 1e-10);
        assert!((tau[3] - 1.0).abs() < 1e-10);
        assert!((tau[4] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tau_k3() {
        // K_3: all vertices are adjacent, so τ(v)=0
        let tau = connection_numbers(&k3()).unwrap();
        for &t in &tau {
            assert!((t).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_k4() {
        let tau = connection_numbers(&k4()).unwrap();
        for &t in &tau {
            assert!((t).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_cycle4() {
        // C_4 (0-1-2-3-0): each vertex has 2 neighbours and 1 vertex at dist 2
        let tau = connection_numbers(&cycle4()).unwrap();
        for &t in &tau {
            assert!((t - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_cycle5() {
        // C_5: each vertex has 2 neighbours at dist 1, 2 at dist 2
        let tau = connection_numbers(&cycle5()).unwrap();
        for &t in &tau {
            assert!((t - 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_cycle6() {
        // C_6: each vertex: 2 at dist 1, 2 at dist 2, 1 at dist 3
        let tau = connection_numbers(&cycle6()).unwrap();
        for &t in &tau {
            assert!((t - 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_star5() {
        // center τ(0) = 0 (all leaves adjacent to center)
        // each leaf: τ(leaf) = 3 (other leaves at dist 2)
        let tau = connection_numbers(&star5()).unwrap();
        assert!((tau[0]).abs() < 1e-10);
        for i in 1..5 {
            assert!((tau[i] - 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn tau_paw() {
        // 0-1, 1-2, 0-2, 2-3. degrees [2,2,3,1]
        // τ(0): dist1={1,2}, dist2={3} → τ=1
        // τ(1): dist1={0,2}, dist2={3} → τ=1
        // τ(2): dist1={0,1,3}, dist2={} → τ=0
        // τ(3): dist1={2}, dist2={0,1} → τ=2
        let tau = connection_numbers(&paw()).unwrap();
        assert!((tau[0] - 1.0).abs() < 1e-10);
        assert!((tau[1] - 1.0).abs() < 1e-10);
        assert!((tau[2]).abs() < 1e-10);
        assert!((tau[3] - 2.0).abs() < 1e-10);
    }

    // --- first_zagreb_connection ---

    #[test]
    fn zc1_empty() {
        let g = Graph::with_vertices(0);
        assert!((first_zagreb_connection(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc1_single_edge() {
        assert!((first_zagreb_connection(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc1_path3() {
        // τ=[1,0,1], ZC₁=1+0+1=2
        assert!((first_zagreb_connection(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn zc1_path4() {
        // τ=[1,1,1,1], ZC₁=4
        assert!((first_zagreb_connection(&path4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn zc1_k3() {
        // τ=[0,0,0], ZC₁=0
        assert!((first_zagreb_connection(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc1_k4() {
        assert!((first_zagreb_connection(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc1_cycle4() {
        // τ=[1,1,1,1], ZC₁=4
        assert!((first_zagreb_connection(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn zc1_cycle5() {
        // τ=[2,2,2,2,2], ZC₁=20
        assert!((first_zagreb_connection(&cycle5()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn zc1_star5() {
        // τ=[0,3,3,3,3], ZC₁=0+9+9+9+9=36
        assert!((first_zagreb_connection(&star5()).unwrap() - 36.0).abs() < 1e-10);
    }

    #[test]
    fn zc1_paw() {
        // τ=[1,1,0,2], ZC₁=1+1+0+4=6
        assert!((first_zagreb_connection(&paw()).unwrap() - 6.0).abs() < 1e-10);
    }

    // --- second_zagreb_connection ---

    #[test]
    fn zc2_empty() {
        let g = Graph::with_vertices(0);
        assert!((second_zagreb_connection(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc2_single_edge() {
        assert!((second_zagreb_connection(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc2_path3() {
        // τ=[1,0,1]: edges (0,1):0, (1,2):0 → 0
        assert!((second_zagreb_connection(&path3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc2_path4() {
        // τ=[1,1,1,1]: edges (0,1):1, (1,2):1, (2,3):1 → 3
        assert!((second_zagreb_connection(&path4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn zc2_k3() {
        assert!((second_zagreb_connection(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc2_cycle4() {
        // τ=[1,1,1,1], each edge: 1·1=1, 4 edges → 4
        assert!((second_zagreb_connection(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn zc2_cycle5() {
        // τ=[2,2,2,2,2], each edge: 4, 5 edges → 20
        assert!((second_zagreb_connection(&cycle5()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn zc2_star5() {
        // τ=[0,3,3,3,3], edges are (0,leaf): 0·3=0 → ZC₂=0
        assert!((second_zagreb_connection(&star5()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zc2_paw() {
        // τ=[1,1,0,2]
        // (0,1): 1·1=1, (0,2): 1·0=0, (1,2): 1·0=0, (2,3): 0·2=0
        // ZC₂ = 1
        assert!((second_zagreb_connection(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- modified_first_zagreb_connection ---

    #[test]
    fn zcs_empty() {
        let g = Graph::with_vertices(0);
        assert!((modified_first_zagreb_connection(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zcs_single_edge() {
        assert!((modified_first_zagreb_connection(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zcs_path3() {
        // τ=[1,0,1]: (0,1):1+0=1, (1,2):0+1=1 → 2
        assert!((modified_first_zagreb_connection(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn zcs_path4() {
        // τ=[1,1,1,1]: each edge: 2, 3 edges → 6
        assert!((modified_first_zagreb_connection(&path4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn zcs_k3() {
        assert!((modified_first_zagreb_connection(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn zcs_cycle4() {
        // τ=[1,1,1,1], each edge: 2, 4 edges → 8
        assert!((modified_first_zagreb_connection(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn zcs_cycle5() {
        // τ=[2,2,2,2,2], each edge: 4, 5 edges → 20
        assert!((modified_first_zagreb_connection(&cycle5()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn zcs_star5() {
        // τ=[0,3,3,3,3], edges (0,leaf): 0+3=3, 4 edges → 12
        assert!((modified_first_zagreb_connection(&star5()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn zcs_paw() {
        // τ=[1,1,0,2]
        // (0,1):2, (0,2):1, (1,2):1, (2,3):2 → 6
        assert!((modified_first_zagreb_connection(&paw()).unwrap() - 6.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn all_zero_for_complete() {
        for g in &[k3(), k4()] {
            assert!((first_zagreb_connection(g).unwrap()).abs() < 1e-10);
            assert!((second_zagreb_connection(g).unwrap()).abs() < 1e-10);
            assert!((modified_first_zagreb_connection(g).unwrap()).abs() < 1e-10);
        }
    }

    #[test]
    fn all_nonneg() {
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            cycle4(),
            cycle5(),
            star5(),
            paw(),
        ] {
            assert!(first_zagreb_connection(g).unwrap() >= -1e-10);
            assert!(second_zagreb_connection(g).unwrap() >= -1e-10);
            assert!(modified_first_zagreb_connection(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn zcs_equals_2sum_tau_deg() {
        // ZC₁*(G) = Σ_{(u,v)∈E} (τ(u)+τ(v)) = Σ_v τ(v)·d(v)
        for g in &[path3(), cycle4(), star5(), paw()] {
            let tau = connection_numbers(g).unwrap();
            let mut vertex_sum = 0.0_f64;
            for v in 0..g.vcount() {
                vertex_sum += tau[v as usize] * g.degree(v).unwrap() as f64;
            }
            let zcs = modified_first_zagreb_connection(g).unwrap();
            assert!((zcs - vertex_sum).abs() < 1e-8);
        }
    }
}
