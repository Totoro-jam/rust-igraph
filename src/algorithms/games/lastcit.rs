//! Last-citation game (ALGO-GN-018).
//!
//! Counterpart of `igraph_lastcit_game()` from
//! `references/igraph/src/games/citations.c:91-209`. Models an
//! evolving citation network where a vertex's attractivity decays with
//! the time elapsed since its last citation, binned into `agebins` age
//! buckets.
//!
//! ## Algorithm
//!
//! Time is measured by vertex arrival; `binwidth = nodes / agebins + 1`.
//! Every vertex `v` carries a weight in a prefix-sum tree:
//!
//! 1. Vertex `0` enters with the "never-cited" weight `preference[agebins]`.
//! 2. For each step `i ∈ [1, nodes)`:
//!    - For each of `edges_per_node` outgoing edges, draw `u ∈ [0, sum)`
//!      uniformly and `psumtree.search(u)` returns the cited vertex `to`.
//!      Record `lastcit[to] = i + 1`, bump `to`'s weight to
//!      `preference[0]` (just-cited bucket).
//!    - When `sum == 0` (no positive weight anywhere — only possible if
//!      `preference[agebins]` *and* all `preference[..agebins]` happen
//!      to be zero, but the validator already rejects that), the C
//!      reference falls back to `RNG_INTEGER(0, i-1)`; we mirror it.
//!    - Insert vertex `i` with weight `preference[agebins]`.
//!    - **Age sweep**: for each `k ≥ 1` such that `shnode = i - k·binwidth ≥ 1`,
//!      walk `shnode`'s emitted citations and, for every cited vertex
//!      `c` whose `lastcit[c] == shnode + 1` (i.e. its last citation
//!      really was at `shnode`), demote its weight to `preference[k]`.
//!
//! The structure is a binary-indexed (Fenwick) tree with binary-lifting
//! prefix-search, giving O(log n) per update / search.
//!
//! ## Self-loops & multi-edges
//!
//! * **Self-loops**: impossible. `psumtree.search` and the `sum=0`
//!   fallback both operate over the `[0, i)` index range; vertex `i` is
//!   added *after* its outgoing edges are drawn, so it cannot be picked.
//! * **Multi-edges**: yes, when `edges_per_node ≥ 2`. Two draws at the
//!   same step can land on the same target. Caller should `simplify()`
//!   if simple output is required (matches upstream behaviour).
//!
//! ## Determinism
//!
//! Reproducible against the shared `SplitMix64`
//! PRNG. Not bit-portable to upstream igraph's GLIBC RNG; conformance
//! asserts structural invariants only.
//!
//! ## References
//!
//! * Upstream igraph C `igraph_lastcit_game`, MIT-licensed reference port.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::many_single_char_names
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search.
///
/// `values[i]` is the current weight at index `i`; `bit[i]` is the
/// Fenwick storage (1-indexed). `total` is maintained eagerly so callers
/// don't need to query when checking the `sum == 0` branch.
struct PsumTree {
    n: usize,
    bit: Vec<f64>,
    values: Vec<f64>,
    total: f64,
}

impl PsumTree {
    fn new(n: usize) -> Self {
        Self {
            n,
            bit: vec![0.0; n + 1],
            values: vec![0.0; n],
            total: 0.0,
        }
    }

    /// Set `values[i] = v`; updates the Fenwick storage and `total` by
    /// the delta `v - values[i]`.
    fn set(&mut self, i: usize, v: f64) {
        let delta = v - self.values[i];
        self.values[i] = v;
        self.total += delta;
        let mut k = i + 1;
        while k <= self.n {
            self.bit[k] += delta;
            k += k & k.wrapping_neg();
        }
    }

    fn total(&self) -> f64 {
        self.total
    }

    /// Smallest index `i` such that `prefix_sum(i+1) > target` for
    /// `target ∈ [0, total)`. Returns `n - 1` on numerical edge cases
    /// (target very close to total) — the caller guarantees
    /// `target = u * total` with `u ∈ [0, 1)`.
    fn search(&self, target: f64) -> usize {
        if self.n == 0 {
            return 0;
        }
        let mut idx: usize = 0;
        let mut remaining = target;
        let mut step = (self.n + 1).next_power_of_two() >> 1;
        while step > 0 {
            let next = idx + step;
            if next <= self.n && self.bit[next] <= remaining {
                idx = next;
                remaining -= self.bit[next];
            }
            step >>= 1;
        }
        // `idx` is the largest position where prefix_sum is still ≤ target,
        // so the smallest position with prefix_sum > target is `idx + 1`,
        // which maps to 0-based vertex `idx`.
        idx.min(self.n - 1)
    }
}

fn validate(nodes: u32, edges_per_node: u32, agebins: u32, preference: &[f64]) -> IgraphResult<()> {
    if agebins < 1 {
        return Err(IgraphError::InvalidArgument(format!(
            "agebins must be at least 1 (got {agebins})"
        )));
    }
    let required = (agebins as usize)
        .checked_add(1)
        .ok_or_else(|| IgraphError::InvalidArgument("agebins overflow".into()))?;
    if preference.len() != required {
        return Err(IgraphError::InvalidArgument(format!(
            "preference vector length ({}) must equal agebins + 1 = {required}",
            preference.len()
        )));
    }
    for (i, &p) in preference.iter().enumerate() {
        if p.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "preference[{i}] is NaN"
            )));
        }
        if !p.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "preference[{i}] is not finite (got {p})"
            )));
        }
        if p < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "preference[{i}] is negative ({p})"
            )));
        }
    }
    if preference[agebins as usize] <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "preference[agebins] (never-cited weight) must be strictly positive, got {}",
            preference[agebins as usize]
        )));
    }
    let _ = (nodes, edges_per_node);
    Ok(())
}

/// Last-citation citation-network game. See module docs.
///
/// # Examples
///
/// ```
/// use rust_igraph::lastcit_game;
///
/// // 5 vertices, 1 edge per step, 2 age bins.
/// // preference: [bin0, bin1, never_cited] — 3 entries.
/// let g = lastcit_game(5, 1, 2, &[1.0, 0.5, 0.1], true, 42).unwrap();
/// assert_eq!(g.vcount(), 5);
/// assert_eq!(g.ecount(), 4);
/// ```
pub fn lastcit_game(
    nodes: u32,
    edges_per_node: u32,
    agebins: u32,
    preference: &[f64],
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(nodes, edges_per_node, agebins, preference)?;

    let mut graph = Graph::new(nodes, directed)?;
    if nodes == 0 || edges_per_node == 0 || nodes < 2 {
        return Ok(graph);
    }

    let n = nodes as usize;
    let eps = edges_per_node as usize;
    let agebins_u = agebins as usize;
    let binwidth = n / agebins_u + 1;

    let mut rng = SplitMix64::new(seed);
    let mut psum = PsumTree::new(n);
    // lastcit[v] = (step at which v was last cited) + 1; 0 means "never".
    let mut lastcit: Vec<u32> = vec![0; n];

    // Vertex 0 starts in the "never cited" bin.
    psum.set(0, preference[agebins_u]);

    let edges_capacity = (n - 1).saturating_mul(eps);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edges_capacity);

    for i in 1..n {
        // Draw `eps` citations from currently-alive vertices [0, i).
        for _ in 0..eps {
            let sum = psum.total();
            let to = if sum > 0.0 {
                let target = rng.gen_unit() * sum;
                psum.search(target)
            } else {
                // Fallback: uniform over [0, i-1].
                let span = i as u64;
                (rng.next_u64() % span) as usize
            };
            let src = u32::try_from(i)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            let dst = u32::try_from(to)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            edges.push((src, dst));
            // Cited at step i+1 (1-based to distinguish from 0 = never).
            let i_plus_one = u32::try_from(i + 1)
                .map_err(|_| IgraphError::InvalidArgument("step counter overflow".into()))?;
            lastcit[to] = i_plus_one;
            psum.set(to, preference[0]);
        }

        // Insert vertex i in the "never cited" bin.
        psum.set(i, preference[agebins_u]);

        // Age sweep: for every previously-emitted citation that originated
        // at vertex `shnode = i - k*binwidth` (with k ≥ 1, shnode ≥ 1), if
        // the cited vertex's last citation is still that one, demote its
        // weight to bucket `k`. Bucket `agebins` would mean "never-cited
        // again" — but if lastcit > 0 the vertex *has* been cited at least
        // once, so the maximum live bucket here is `min(k, agebins - 1)`.
        let mut k: usize = 1;
        while k * binwidth <= i.saturating_sub(1) + binwidth {
            let lhs = i.saturating_sub(k * binwidth);
            if lhs < 1 {
                break;
            }
            let shnode = lhs;
            // Edges emitted at step `shnode` occupy indices
            // [(shnode-1)*eps, shnode*eps) in `edges`.
            let start = (shnode - 1) * eps;
            let end = shnode * eps;
            let shnode_marker = u32::try_from(shnode + 1)
                .map_err(|_| IgraphError::InvalidArgument("step counter overflow".into()))?;
            // Pick the demotion bucket: bucket index `k` saturates at
            // `agebins - 1`; the `agebins` slot is reserved for the
            // never-cited case which doesn't apply here (these vertices
            // have been cited).
            let bucket = k.min(agebins_u - 1);
            let new_weight = preference[bucket];
            for edge in &edges[start..end] {
                let cnode = edge.1 as usize;
                if lastcit[cnode] == shnode_marker {
                    psum.set(cnode, new_weight);
                }
            }
            k += 1;
        }
    }

    graph.add_edges(edges)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nodes_zero_returns_empty_graph() {
        let g = lastcit_game(0, 3, 2, &[1.0, 0.5, 0.1], false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn nodes_one_returns_single_vertex_no_edges() {
        let g = lastcit_game(1, 5, 2, &[1.0, 0.5, 0.1], false, 2).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn edges_per_node_zero_yields_edgeless() {
        let g = lastcit_game(50, 0, 2, &[1.0, 0.5, 0.1], false, 3).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_ecount() {
        let n = 40u32;
        let eps = 3u32;
        let g = lastcit_game(n, eps, 3, &[1.0, 0.5, 0.2, 0.1], false, 4).unwrap();
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), ((n - 1) * eps) as usize);
    }

    #[test]
    fn determinism() {
        let pref = vec![1.0, 0.7, 0.3, 0.1];
        let g1 = lastcit_game(50, 3, 3, &pref, true, 12345).unwrap();
        let g2 = lastcit_game(50, 3, 3, &pref, true, 12345).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        let n_e = u32::try_from(g1.ecount()).unwrap();
        for eid in 0..n_e {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let pref = vec![1.0, 0.5, 0.1];
        let g1 = lastcit_game(40, 2, 2, &pref, false, 1).unwrap();
        let g2 = lastcit_game(40, 2, 2, &pref, false, 2).unwrap();
        let n_e = u32::try_from(g1.ecount()).unwrap();
        let mut any_diff = false;
        for eid in 0..n_e {
            if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                any_diff = true;
                break;
            }
        }
        assert!(any_diff);
    }

    #[test]
    fn no_self_loops() {
        let pref = vec![1.0, 0.5, 0.1];
        let g = lastcit_game(80, 3, 2, &pref, true, 999).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "lastcit game must never self-loop");
        }
    }

    #[test]
    fn src_in_step_order() {
        let pref = vec![1.0, 0.5];
        let n = 20u32;
        let eps = 3u32;
        let g = lastcit_game(n, eps, 1, &pref, true, 7).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (src, _dst) = g.edge(eid).unwrap();
            let expected_src = 1 + eid / eps;
            assert_eq!(src, expected_src);
        }
    }

    #[test]
    fn dst_strictly_less_than_src() {
        let pref = vec![1.0, 0.5, 0.1];
        let n = 50u32;
        let g = lastcit_game(n, 2, 2, &pref, true, 11).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (src, dst) = g.edge(eid).unwrap();
            assert!(dst < src);
        }
    }

    #[test]
    fn directed_flag_propagates() {
        let pref = vec![1.0, 0.5];
        let g = lastcit_game(20, 2, 1, &pref, true, 11).unwrap();
        assert!(g.is_directed());
    }

    #[test]
    fn undirected_flag_propagates() {
        let pref = vec![1.0, 0.5];
        let g = lastcit_game(20, 2, 1, &pref, false, 12).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn err_agebins_zero() {
        let err = lastcit_game(10, 1, 0, &[1.0], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_preference_length_mismatch() {
        let err = lastcit_game(10, 1, 3, &[1.0, 0.5], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_never_cited_weight_zero() {
        let err = lastcit_game(10, 1, 2, &[1.0, 0.5, 0.0], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_preference_negative() {
        let err = lastcit_game(10, 1, 2, &[1.0, -0.5, 1.0], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_preference_nan() {
        let err = lastcit_game(10, 1, 2, &[1.0, f64::NAN, 1.0], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_preference_inf() {
        let err = lastcit_game(10, 1, 2, &[1.0, 0.5, f64::INFINITY], false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn only_never_cited_weight_positive_concentrates_on_uncited() {
        // preference = [0, 0, 0, 0, 1]: only the never-cited bucket
        // carries weight. Every citation goes to a not-yet-cited vertex.
        // After being cited once, a vertex's weight drops to 0, so the
        // psumtree never re-picks it. Hence the resulting graph is
        // simple (no multi-edges among the existing pool — each vertex
        // can only be cited at most `degree(v) = #times it appears as
        // dst within a single source's eps draws` times, but multiple
        // draws within the SAME step still hit the same uncited vertex
        // only after intermediate updates... actually for eps=1 every
        // dst is unique).
        let pref = vec![0.0, 0.0, 0.0, 0.0, 1.0];
        let n = 50u32;
        let g = lastcit_game(n, 1, 4, &pref, true, 17).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        // Count: vertex must have been "fresh" (never cited) at the time
        // it was cited. Equivalent statement: each dst appears at most
        // once across the whole edge list.
        let mut seen = vec![false; n as usize];
        for eid in 0..n_e {
            let (_s, d) = g.edge(eid).unwrap();
            assert!(
                !seen[d as usize],
                "vertex {d} cited twice under never-cited-only preference"
            );
            seen[d as usize] = true;
        }
    }

    #[test]
    fn high_recent_weight_concentrates_in_degree_on_recent_vertices() {
        // preference = [1000, 0.001, 0.001]: just-cited vertices are
        // overwhelmingly preferred. After the warm-up phase, in-degree
        // should be very concentrated on the most recently cited pool.
        let pref = vec![1000.0, 0.001, 0.001];
        let n = 200u32;
        let g = lastcit_game(n, 3, 2, &pref, true, 31).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        let mut in_deg = vec![0u32; n as usize];
        for eid in 0..n_e {
            let (_s, d) = g.edge(eid).unwrap();
            in_deg[d as usize] += 1;
        }
        let max_in = *in_deg.iter().max().unwrap();
        // With ~600 edges and heavy concentration, the top vertex should
        // accumulate well above the uniform mean of ~3.
        assert!(
            max_in > 10,
            "expected concentration, got max in-degree {max_in}"
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ecount_exact(
            n in 2u32..40,
            eps in 1u32..5,
            seed in any::<u64>(),
        ) {
            let pref = vec![1.0, 0.5, 0.1];
            let g = lastcit_game(n, eps, 2, &pref, false, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), ((n - 1) * eps) as usize);
        }

        #[test]
        fn no_self_loops(
            n in 2u32..30,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let pref = vec![1.0, 0.5];
            let g = lastcit_game(n, eps, 1, &pref, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (a, b) = g.edge(eid).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn src_step_order(
            n in 2u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let pref = vec![1.0, 0.5];
            let g = lastcit_game(n, eps, 1, &pref, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (src, _dst) = g.edge(eid).unwrap();
                let expected_src = 1 + eid / eps;
                prop_assert_eq!(src, expected_src);
            }
        }

        #[test]
        fn dst_lt_src(
            n in 2u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let pref = vec![1.0, 0.5, 0.1];
            let g = lastcit_game(n, eps, 2, &pref, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (src, dst) = g.edge(eid).unwrap();
                prop_assert!(dst < src);
            }
        }

        #[test]
        fn determinism_under_proptest(
            n in 2u32..30,
            eps in 1u32..4,
            agebins in 1u32..4,
            seed in any::<u64>(),
        ) {
            let pref: Vec<f64> = (0..=agebins as usize)
                .map(|i| 1.0 / (i as f64 + 1.0))
                .collect();
            let g1 = lastcit_game(n, eps, agebins, &pref, false, seed).unwrap();
            let g2 = lastcit_game(n, eps, agebins, &pref, false, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let n_e = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..n_e {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }
    }
}
