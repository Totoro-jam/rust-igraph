//! Distance-based topological indices (ALGO-TR-041).
//!
//! - **Szeged index** `Sz(G) = Σ_{(u,v)∈E} n_u(e)·n_v(e)`
//!   where `n_u(e)` = number of vertices strictly closer to `u` than `v`.
//! - **Revised Szeged index**
//!   `Sz*(G) = Σ_{(u,v)∈E} (n_u + n₀/2)·(n_v + n₀/2)`
//!   where `n₀` = vertices equidistant from `u` and `v`.
//! - **Padmakar-Ivan (PI) index** `PI(G) = Σ_{(u,v)∈E} (n_u + n_v)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::comparison_chain,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the Szeged index of a graph.
///
/// `Sz(G) = Σ_{(u,v)∈E} n_u(e) · n_v(e)`
///
/// For each edge `(u, v)`, `n_u(e)` counts vertices strictly closer to
/// `u` than to `v`, and `n_v(e)` counts vertices strictly closer to `v`
/// than to `u`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, szeged_index};
///
/// // Path 0-1-2: edge (0,1): n0=1, n1=2 → 2; edge (1,2): n1=2, n2=1 → 2
/// // Sz = 2 + 2 = 4
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(szeged_index(&g).unwrap(), 4);
/// ```
pub fn szeged_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut sz: u64 = 0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let (nu, nv) = count_closer(&dist, n, ui, vi);
        sz = sz.saturating_add((nu as u64).saturating_mul(nv as u64));
    }

    Ok(sz)
}

/// Compute the revised Szeged index of a graph.
///
/// `Sz*(G) = Σ_{(u,v)∈E} (n_u + n₀/2) · (n_v + n₀/2)`
///
/// Vertices equidistant from both endpoints are split evenly between
/// the two sides.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, revised_szeged_index};
///
/// // Single edge 0-1: n0=1, n1=1, equidistant=0 → (1)(1) = 1.0
/// let g = Graph::from_edges(&[(0,1)], false, Some(2)).unwrap();
/// assert!((revised_szeged_index(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn revised_szeged_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut sz_star: f64 = 0.0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let (nu, nv, n0) = count_closer_with_equidistant(&dist, n, ui, vi);
        let su = nu as f64 + n0 as f64 / 2.0;
        let sv = nv as f64 + n0 as f64 / 2.0;
        sz_star += su * sv;
    }

    Ok(sz_star)
}

/// Compute the Padmakar-Ivan (PI) index of a graph.
///
/// `PI(G) = Σ_{(u,v)∈E} (n_u(e) + n_v(e))`
///
/// Equivalently, `PI(G) = Σ_{(u,v)∈E} (n - n₀(e))` where `n₀(e)` is
/// the number of vertices equidistant from both endpoints (including
/// unreachable vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pi_index};
///
/// // Path 0-1-2: edge (0,1): nu=1,nv=2 → 3; edge (1,2): nu=2,nv=1 → 3
/// // PI = 6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(pi_index(&g).unwrap(), 6);
/// ```
pub fn pi_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut pi: u64 = 0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let (nu, nv) = count_closer(&dist, n, ui, vi);
        pi = pi.saturating_add((nu as u64).saturating_add(nv as u64));
    }

    Ok(pi)
}

fn count_closer(dist: &[u32], n: usize, u: usize, v: usize) -> (usize, usize) {
    let mut nu = 0_usize;
    let mut nv = 0_usize;
    for w in 0..n {
        let du = dist[u * n + w];
        let dv = dist[v * n + w];
        if du == u32::MAX || dv == u32::MAX {
            continue;
        }
        if du < dv {
            nu += 1;
        } else if dv < du {
            nv += 1;
        }
    }
    (nu, nv)
}

fn count_closer_with_equidistant(
    dist: &[u32],
    n: usize,
    u: usize,
    v: usize,
) -> (usize, usize, usize) {
    let mut nu = 0_usize;
    let mut nv = 0_usize;
    let mut n0 = 0_usize;
    for w in 0..n {
        let du = dist[u * n + w];
        let dv = dist[v * n + w];
        if du == u32::MAX || dv == u32::MAX {
            continue;
        }
        if du < dv {
            nu += 1;
        } else if dv < du {
            nv += 1;
        } else {
            n0 += 1;
        }
    }
    (nu, nv, n0)
}

fn all_pairs_bfs(graph: &Graph, n: usize) -> Vec<u32> {
    let adj = build_adj_list(graph, n);
    let mut dist = vec![u32::MAX; n * n];
    for src in 0..n {
        bfs_distances(&adj, n, src, &mut dist);
    }
    dist
}

fn build_adj_list(graph: &Graph, n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui].push(vi);
        if !graph.is_directed() && ui != vi {
            adj[vi].push(ui);
        }
    }
    adj
}

fn bfs_distances(adj: &[Vec<usize>], n: usize, src: usize, dist: &mut [u32]) {
    dist[src * n + src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);
    while let Some(v) = queue.pop_front() {
        let d = dist[src * n + v];
        for &w in &adj[v] {
            if dist[src * n + w] == u32::MAX {
                dist[src * n + w] = d.saturating_add(1);
                queue.push_back(w);
            }
        }
    }
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

    // --- szeged_index ---

    #[test]
    fn sz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(szeged_index(&g).unwrap(), 0);
    }

    #[test]
    fn sz_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(szeged_index(&g).unwrap(), 0);
    }

    #[test]
    fn sz_single_edge() {
        // n_u=1, n_v=1 → 1
        assert_eq!(szeged_index(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn sz_path3() {
        // edge (0,1): closer to 0={0}, closer to 1={1,2} → 1·2 = 2
        // edge (1,2): closer to 1={0,1}, closer to 2={2} → 2·1 = 2
        // Sz = 4
        assert_eq!(szeged_index(&path3()).unwrap(), 4);
    }

    #[test]
    fn sz_path4() {
        // edge (0,1): closer to 0={0}, closer to 1={2,3} → 1·2 = 2
        // edge (1,2): closer to 1={0}, closer to 2={3} → 1·1 = 1 ... wait
        // Actually: d(0,1)=1,d(0,2)=2 so d(w=0,u=1)=1 < d(w=0,v=2)=2 → w=0 closer to u=1
        // d(w=3,u=1)=2, d(w=3,v=2)=1 → w=3 closer to v=2
        // d(w=1,u=1)=0, d(w=1,v=2)=1 → w=1 closer to u=1
        // d(w=2,u=1)=1, d(w=2,v=2)=0 → w=2 closer to v=2
        // n_u=2, n_v=2 → 4
        // edge (0,1): d(w=0,0)=0<d(w=0,1)=1 → closer to 0
        //             d(w=1,0)=1>d(w=1,1)=0 → closer to 1
        //             d(w=2,0)=2>d(w=2,1)=1 → closer to 1
        //             d(w=3,0)=3>d(w=3,1)=2 → closer to 1
        //             n0=1, n1=3 → 3
        // edge (1,2): n1=2(w=0,w=1), n2=2(w=2,w=3) → 4
        // edge (2,3): n2=3(w=0,w=1,w=2), n3=1(w=3) → 3
        // Sz = 3 + 4 + 3 = 10
        assert_eq!(szeged_index(&path4()).unwrap(), 10);
    }

    #[test]
    fn sz_k3() {
        // All distances = 1. For edge (0,1):
        // d(w=0,0)=0 < d(w=0,1)=1 → closer to 0
        // d(w=1,0)=1 > d(w=1,1)=0 → closer to 1
        // d(w=2,0)=1 = d(w=2,1)=1 → equidistant
        // n0=1, n1=1 → 1
        // By symmetry all 3 edges give 1·1 = 1 → Sz = 3
        assert_eq!(szeged_index(&k3()).unwrap(), 3);
    }

    #[test]
    fn sz_k4() {
        // For any edge (u,v) in K4: n_u=1(u itself), n_v=1(v itself),
        // other 2 vertices equidistant → 1·1 = 1
        // 6 edges → Sz = 6
        assert_eq!(szeged_index(&k4()).unwrap(), 6);
    }

    #[test]
    fn sz_cycle4() {
        // C4: vertices 0-1-2-3-0
        // edge (0,1): d(0,0)=0<d(0,1)=1 closer to 0
        //             d(1,0)=1>d(1,1)=0 closer to 1
        //             d(2,0)=2>d(2,1)=1 closer to 1
        //             d(3,0)=1<d(3,1)=2 closer to 0
        //             n0=2, n1=2 → 4
        // By symmetry all 4 edges give 4 → Sz = 16
        assert_eq!(szeged_index(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn sz_star5() {
        // Star: center=0, leaves 1,2,3,4
        // edge (0,1): closer to 0={0,2,3,4}(d to 0 < d to 1),
        //             closer to 1={1} → 4·1 = 4
        // By symmetry all 4 edges give 4 → Sz = 16
        assert_eq!(szeged_index(&star5()).unwrap(), 16);
    }

    #[test]
    fn sz_cycle6() {
        // C6: for edge (0,1):
        // d(w,0) vs d(w,1):
        // w=0: 0<1 → closer to 0
        // w=1: 1>0 → closer to 1
        // w=2: 2>1 → closer to 1
        // w=3: 3=3 (diameter vertex, equidistant) → tie  ... wait
        // Actually in C6, d(3,0)=3, d(3,1)=2 → closer to 1
        // w=4: d(4,0)=2, d(4,1)=3 → closer to 0
        // w=5: d(5,0)=1, d(5,1)=2 → closer to 0
        // n0=3, n1=3 → 9
        // All 6 edges by symmetry → 54
        assert_eq!(szeged_index(&cycle6()).unwrap(), 54);
    }

    // --- revised_szeged_index ---

    #[test]
    fn rsz_empty() {
        let g = Graph::with_vertices(0);
        assert!((revised_szeged_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn rsz_single_edge() {
        // n_u=1, n_v=1, n_0=0 → (1)(1) = 1
        assert!((revised_szeged_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rsz_k3() {
        // For edge (0,1): n_u=1, n_v=1, n_0=1
        // (1+0.5)(1+0.5) = 2.25
        // 3 edges → 6.75
        let r = revised_szeged_index(&k3()).unwrap();
        assert!((r - 6.75).abs() < 1e-10);
    }

    #[test]
    fn rsz_path3() {
        // edge (0,1): closer to 0={0}, closer to 1={2}, equidist={1}(d=0,d=1 → not equal)
        // Wait: d(w=1,u=0)=1, d(w=1,v=1)=0 → closer to 1
        // So n_u=1, n_v=2, n_0=0 → (1)(2) = 2? No...
        // edge(0,1): w=0: d(0,0)=0 < d(0,1)=1 → closer to 0 ✓
        //            w=1: d(1,0)=1 > d(1,1)=0 → closer to 1 ✓
        //            w=2: d(2,0)=2 > d(2,1)=1 → closer to 1 ✓
        // n_u=1, n_v=2, n_0=0 → revised = (1)(2) = 2
        // edge(1,2): w=0: d(0,1)=1 < d(0,2)=2 → closer to 1
        //            w=1: d(1,1)=0 < d(1,2)=1 → closer to 1
        //            w=2: d(2,1)=1 > d(2,2)=0 → closer to 2
        // n_u=2, n_v=1, n_0=0 → revised = (2)(1) = 2
        // Total = 4
        let r = revised_szeged_index(&path3()).unwrap();
        assert!((r - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rsz_equals_sz_for_trees() {
        // Trees have no equidistant vertices (for endpoints of any edge
        // in a tree, every vertex is strictly closer to one endpoint)
        // So revised Szeged = Szeged for trees
        for g in &[single_edge(), path3(), path4(), star5()] {
            let sz = szeged_index(g).unwrap() as f64;
            let rsz = revised_szeged_index(g).unwrap();
            assert!(
                (sz - rsz).abs() < 1e-10,
                "Sz={sz}, Sz*={rsz} should be equal for trees"
            );
        }
    }

    #[test]
    fn rsz_geq_sz() {
        // Revised Szeged >= Szeged always
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), cycle6()] {
            let sz = szeged_index(g).unwrap() as f64;
            let rsz = revised_szeged_index(g).unwrap();
            assert!(rsz >= sz - 1e-10, "Sz*={rsz} < Sz={sz}");
        }
    }

    // --- pi_index ---

    #[test]
    fn pi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(pi_index(&g).unwrap(), 0);
    }

    #[test]
    fn pi_single_edge() {
        // n_u=1, n_v=1 → 2
        assert_eq!(pi_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn pi_path3() {
        // edge(0,1): nu=1, nv=2 → 3
        // edge(1,2): nu=2, nv=1 → 3
        // PI = 6
        assert_eq!(pi_index(&path3()).unwrap(), 6);
    }

    #[test]
    fn pi_k3() {
        // Each edge: nu=1, nv=1 → 2
        // 3 edges → 6
        assert_eq!(pi_index(&k3()).unwrap(), 6);
    }

    #[test]
    fn pi_cycle4() {
        // Each edge: nu=2, nv=2 → 4
        // 4 edges → 16
        assert_eq!(pi_index(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn pi_star5() {
        // Each edge: nu=4, nv=1 → 5
        // 4 edges → 20
        assert_eq!(pi_index(&star5()).unwrap(), 20);
    }

    // --- cross-consistency ---

    #[test]
    fn szeged_leq_wiener_squared_over_m() {
        // For connected graphs: Sz(G) >= W(G) (Klavžar-Rajapakse-Gutman)
        // where W is the Wiener index
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            let sz = szeged_index(g).unwrap() as f64;
            let w = crate::algorithms::properties::distance_spectrum::wiener_index(g).unwrap();
            assert!(
                sz >= w - 1e-10,
                "Szeged {sz} < Wiener {w}, violates Sz >= W"
            );
        }
    }

    #[test]
    fn pi_equals_sum_nu_nv() {
        // PI = Σ (nu + nv) and Sz = Σ nu·nv
        // Verify PI and Sz are consistent
        for g in &[path3(), k3(), cycle4(), star5()] {
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut sum_product: u64 = 0;
            let mut sum_sum: u64 = 0;
            for (u, v) in g.edges() {
                let ui = u as usize;
                let vi = v as usize;
                if ui == vi {
                    continue;
                }
                let (nu, nv) = count_closer(&dist, n, ui, vi);
                sum_product += (nu as u64) * (nv as u64);
                sum_sum += (nu as u64) + (nv as u64);
            }
            assert_eq!(sum_product, szeged_index(g).unwrap());
            assert_eq!(sum_sum, pi_index(g).unwrap());
        }
    }

    #[test]
    fn szeged_equals_wiener_for_trees() {
        // For trees, Szeged index = Wiener index
        for g in &[single_edge(), path3(), path4(), star5()] {
            let sz = szeged_index(g).unwrap() as f64;
            let w = crate::algorithms::properties::distance_spectrum::wiener_index(g).unwrap();
            assert!((sz - w).abs() < 1e-10, "Szeged {sz} != Wiener {w} for tree");
        }
    }
}
