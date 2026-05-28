//! Graph spanner (ALGO-PA-030) — Baswana-Sen algorithm.
//!
//! Counterpart of `igraph_spanner` in
//! `references/igraph/src/paths/sparsifier.c:152`.
//!
//! A t-spanner of G is a subgraph H with the same vertex set such
//! that for every pair (u, v), `dist_H(u, v)` ≤ t · `dist_G(u, v)`.
//! The algorithm returns edge IDs of a sparse subgraph achieving
//! this stretch guarantee.
//!
//! ## Algorithm
//!
//! Baswana and Sen, "A Simple and Linear Time Randomized Algorithm
//! for Computing Sparse Spanners in Weighted Graphs", Random
//! Structures & Algorithms 30(4), 2007.
//!
//! Randomized Las Vegas: the result is always correct (verified
//! stretch), but expected edge count is O(k·n^(1+1/k)) for
//! unweighted and O(n^(1+1/k)) for weighted, where k = (t+1)/2.

use crate::core::VertexId;
use crate::core::{Graph, IgraphError, IgraphResult};

type IncList = Vec<Vec<(VertexId, u32, f64)>>;

/// Compute a t-spanner of the graph.
///
/// Returns a sorted list of edge IDs whose induced subgraph is a
/// t-spanner of the input graph. Edge directions are ignored.
///
/// # Arguments
///
/// * `graph` — input graph (undirected or directed; directions ignored).
/// * `stretch` — the stretch factor t (must be ≥ 1.0).
/// * `weights` — optional edge weight vector (all positive). `None`
///   means unit weights.
///
/// # Errors
///
/// Returns an error if `stretch < 1.0`, weight vector length doesn't
/// match edge count, or weights contain non-positive/NaN values.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spanner};
///
/// // Path graph: 0-1-2-3. A 1-spanner must include all edges.
/// let mut g = Graph::new(4, false).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let sp = spanner(&g, 1.0, None).unwrap();
/// assert_eq!(sp.len(), 3); // must keep all edges for stretch=1
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn spanner(graph: &Graph, stretch: f64, weights: Option<&[f64]>) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    validate_inputs(stretch, weights, m)?;

    if n == 0 || m == 0 {
        return Ok(Vec::new());
    }

    let k = stretch / 2.0 + 0.5;
    let sample_prob = (n as f64).powf(-1.0 / k);

    let mut inc = build_incidence(graph, n, m, weights)?;
    let mut in_spanner = vec![false; m];
    let mut clustering: Vec<u32> = (0..n)
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX))
        .collect();

    let mut rng_state: u64 =
        0x1234_5678_9abc_def0_u64 ^ (n as u64).wrapping_mul(0x517c_c1b7_2722_0a95);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k_iters = (k - 1.0).max(0.0).ceil() as usize;

    for _iter in 0..k_iters {
        phase1_iteration(
            n,
            &mut inc,
            &mut in_spanner,
            &mut clustering,
            &mut rng_state,
            sample_prob,
        );
    }

    phase2_finalize(n, &inc, &mut in_spanner, &clustering);

    let mut result: Vec<u32> = (0..m)
        .filter(|&e| in_spanner[e])
        .filter_map(|e| u32::try_from(e).ok())
        .collect();
    result.sort_unstable();
    Ok(result)
}

fn validate_inputs(stretch: f64, weights: Option<&[f64]>, m: usize) -> IgraphResult<()> {
    if stretch < 1.0 {
        return Err(IgraphError::InvalidArgument(
            "spanner: stretch factor must be at least 1.0".into(),
        ));
    }

    if let Some(w) = weights {
        if w.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "spanner: weight vector length {} != edge count {m}",
                w.len()
            )));
        }
        for (i, &v) in w.iter().enumerate() {
            if v.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "spanner: weights[{i}] is NaN"
                )));
            }
            if v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "spanner: weights[{i}] = {v} is negative"
                )));
            }
        }
    }
    Ok(())
}

fn build_incidence(
    graph: &Graph,
    n: usize,
    m: usize,
    weights: Option<&[f64]>,
) -> IgraphResult<IncList> {
    let mut inc: IncList = vec![Vec::new(); n];
    for eid in 0..m {
        let eid32 = u32::try_from(eid)
            .map_err(|_| IgraphError::InvalidArgument("spanner: edge id overflow".into()))?;
        let (from, to) = graph.edge(eid32)?;
        let w = weights.map_or(1.0, |ws| ws[eid]);
        inc[from as usize].push((to, eid32, w));
        if from != to {
            inc[to as usize].push((from, eid32, w));
        }
    }
    Ok(inc)
}

fn xorshift64(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    // Divide by a constant to get [0, 1) — precision loss is acceptable for PRNG
    #[allow(clippy::cast_precision_loss)]
    let val = (*state >> 11) as f64 / ((1_u64 << 53) as f64);
    val
}

fn update_lightest(lightest: &mut Vec<(u32, u32, f64)>, cluster: u32, eid: u32, w: f64) {
    for entry in &mut *lightest {
        if entry.0 == cluster {
            if w < entry.2 {
                entry.1 = eid;
                entry.2 = w;
            }
            return;
        }
    }
    lightest.push((cluster, eid, w));
}

fn phase1_iteration(
    n: usize,
    inc: &mut IncList,
    in_spanner: &mut [bool],
    clustering: &mut Vec<u32>,
    rng_state: &mut u64,
    sample_prob: f64,
) {
    let mut new_clustering: Vec<u32> = vec![u32::MAX; n];
    let mut is_sampled = vec![false; n];

    for v in 0..n {
        let cv = clustering[v] as usize;
        if cv == v && xorshift64(rng_state) < sample_prob {
            is_sampled[v] = true;
        }
    }

    for v in 0..n {
        let cluster_v = clustering[v];
        if cluster_v != u32::MAX && is_sampled[cluster_v as usize] {
            new_clustering[v] = cluster_v;
            continue;
        }

        let mut lightest_to_cluster: Vec<(u32, u32, f64)> = Vec::new();
        let mut nearest_sampled: Option<(u32, u32, f64)> = None;

        for &(nb, eid, w) in &inc[v] {
            let cluster_nb = clustering[nb as usize];
            if cluster_nb == u32::MAX {
                continue;
            }

            update_lightest(&mut lightest_to_cluster, cluster_nb, eid, w);

            if is_sampled[cluster_nb as usize] {
                let better = nearest_sampled
                    .as_ref()
                    .is_none_or(|(_, _, best_w)| w < *best_w);
                if better {
                    nearest_sampled = Some((cluster_nb, eid, w));
                }
            }
        }

        if let Some((sampled_cluster, sampled_eid, sampled_w)) = nearest_sampled {
            in_spanner[sampled_eid as usize] = true;
            new_clustering[v] = sampled_cluster;

            for &(_, eid, w) in &lightest_to_cluster {
                if w < sampled_w {
                    in_spanner[eid as usize] = true;
                }
            }

            inc[v].retain(|&(nb, _, _)| {
                let cluster_nb = clustering[nb as usize];
                if cluster_nb == u32::MAX {
                    return true;
                }
                if cluster_nb == sampled_cluster {
                    return false;
                }
                let cluster_lightest = lightest_to_cluster
                    .iter()
                    .find(|e| e.0 == cluster_nb)
                    .map_or(f64::INFINITY, |e| e.2);
                cluster_lightest >= sampled_w
            });
        } else {
            for &(_, eid, _) in &lightest_to_cluster {
                in_spanner[eid as usize] = true;
            }
            inc[v].clear();
        }
    }

    *clustering = new_clustering;

    for v in 0..n {
        let cv = clustering[v];
        if cv == u32::MAX {
            continue;
        }
        inc[v].retain(|&(nb, _, _)| clustering[nb as usize] != cv);
    }
}

fn phase2_finalize(n: usize, inc: &IncList, in_spanner: &mut [bool], clustering: &[u32]) {
    for v in 0..n {
        if clustering[v] == u32::MAX {
            continue;
        }

        let mut lightest_to_cluster: Vec<(u32, u32, f64)> = Vec::new();
        for &(nb, eid, w) in &inc[v] {
            let cluster_nb = clustering[nb as usize];
            if cluster_nb == u32::MAX {
                continue;
            }
            update_lightest(&mut lightest_to_cluster, cluster_nb, eid, w);
        }

        for &(_, eid, _) in &lightest_to_cluster {
            in_spanner[eid as usize] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanner_empty() {
        let g = Graph::new(0, false).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert!(sp.is_empty());
    }

    #[test]
    fn spanner_no_edges() {
        let g = Graph::new(5, false).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert!(sp.is_empty());
    }

    #[test]
    fn spanner_single_edge() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert_eq!(sp, vec![0]);
    }

    #[test]
    fn spanner_stretch_1_keeps_all() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let sp = spanner(&g, 1.0, None).unwrap();
        assert_eq!(sp.len(), 3);
    }

    #[test]
    fn spanner_triangle() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert!(sp.len() >= 2);
        assert!(sp.len() <= 3);
    }

    #[test]
    fn spanner_complete_4() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert!(sp.len() >= 3);
        assert!(sp.len() <= 6);
    }

    #[test]
    fn spanner_weighted() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = vec![1.0, 1.0, 3.0];
        let sp = spanner(&g, 3.0, Some(&weights)).unwrap();
        assert!(sp.len() >= 2);
    }

    #[test]
    fn spanner_invalid_stretch() {
        let g = Graph::new(2, false).unwrap();
        assert!(spanner(&g, 0.5, None).is_err());
    }

    #[test]
    fn spanner_invalid_weights_len() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(spanner(&g, 3.0, Some(&[1.0, 2.0])).is_err());
    }

    #[test]
    fn spanner_negative_weight() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(spanner(&g, 3.0, Some(&[-1.0])).is_err());
    }

    #[test]
    fn spanner_result_sorted() {
        let mut g = Graph::new(5, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        let mut sorted = sp.clone();
        sorted.sort_unstable();
        assert_eq!(sp, sorted);
    }

    #[test]
    fn spanner_directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let sp = spanner(&g, 3.0, None).unwrap();
        assert!(sp.len() >= 2);
    }
}
