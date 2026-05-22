//! Edge clustering coefficient (ALGO-PR-031).
//!
//! Counterpart of `igraph_ecc` from
//! `references/igraph/src/properties/ecc.c` (lines 33-385). Given an
//! edge `(i, j)`, the **edge clustering coefficient** (Radicchi 2004)
//! counts the `k`-cycles the edge participates in and normalises by the
//! maximum number of such cycles allowed by the endpoint degrees:
//!
//! - `k = 3`: `s = min(d_i, d_j) - 1`
//! - `k = 4`: `s = (d_i - 1) · (d_j - 1)`
//!
//! With the canonical Radicchi definition,
//! `C^(k)_ij = (z^(k)_ij + 1) / s^(k)_ij`. The `offset` / `normalize`
//! flags toggle the `+1` and the `/s` respectively; passing `(offset=
//! true, normalize=true)` reproduces the paper's definition exactly.
//!
//! Reference: F. Radicchi, C. Castellano, F. Cecconi, V. Loreto,
//! D. Parisi (2004) "Defining and identifying communities in
//! networks", PNAS 101 (9) 2658-2663.
//!
//! Multi-edges and self-loops:
//! - cycle counts `z` ignore multi-edges (the simple adjacency is used);
//! - the normaliser `s` uses the **loop-aware** degree, matching the C
//!   reference's `IGRAPH_LOOPS` mode (each self-loop contributes 2 to
//!   the undirected degree);
//! - a self-loop edge yields `NaN`;
//! - any edge with `s <= 0` (e.g. a degree-1 endpoint) yields
//!   `NaN` when normalising.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute the edge clustering coefficient for `eids` in `graph`.
///
/// - `eids = None` → every edge (in id order, `0..graph.ecount()`).
/// - `eids = Some(&[...])` → just those edges, in the order given.
///
/// `k` must be `3` or `4`. `offset = true` adds the canonical Radicchi
/// `+1` to the cycle count; `normalize = true` divides by the
/// degree-derived maximum.
///
/// Edge directions are ignored — the function treats the input as the
/// underlying undirected graph, the same way `igraph_ecc` does.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] when `k < 3`.
/// - [`IgraphError::Unsupported`] when `k > 4` (Radicchi only defines
///   `k = 3` and `k = 4`).
/// - [`IgraphError::EdgeOutOfRange`] when any `eid` is `>= ecount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ecc};
///
/// // K_4: every edge sits in 2 triangles, max is 2 → normalised value
/// // (without offset) is 1.0.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// let c = ecc(&g, None, 3, false, true).unwrap();
/// assert_eq!(c, vec![1.0; 6]);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn ecc(
    graph: &Graph,
    eids: Option<&[EdgeId]>,
    k: u32,
    offset: bool,
    normalize: bool,
) -> IgraphResult<Vec<f64>> {
    if k < 3 {
        return Err(IgraphError::InvalidArgument(format!(
            "cycle size for edge clustering coefficient must be at least 3, got {k}"
        )));
    }
    if k > 4 {
        return Err(IgraphError::Unsupported(
            "edge clustering coefficient is only implemented for k = 3 and k = 4",
        ));
    }

    let m = graph.ecount();
    let owned: Vec<EdgeId>;
    let eids: &[EdgeId] = if let Some(slice) = eids {
        slice
    } else {
        owned = (0..u32::try_from(m)
            .map_err(|_| IgraphError::Internal("ecc: ecount() exceeds u32::MAX"))?)
            .collect();
        &owned
    };

    let (simple_adj, loop_degree) = build_simple_adjacency(graph)?;
    let c = if offset { 1.0 } else { 0.0 };
    let mut result = Vec::with_capacity(eids.len());
    let m_u32 = u32::try_from(m).unwrap_or(u32::MAX);

    for &eid in eids {
        if eid as usize >= m {
            return Err(IgraphError::EdgeOutOfRange { id: eid, m: m_u32 });
        }
        let (v1, v2) = graph.edge(eid)?;
        let value = if v1 == v2 {
            // Self-loops sit in no cycles and have no meaningful denominator.
            if normalize { f64::NAN } else { c }
        } else {
            let (z, s) = match k {
                3 => ecc3_pair(&simple_adj, &loop_degree, v1, v2),
                4 => ecc4_pair(&simple_adj, &loop_degree, v1, v2),
                _ => unreachable!("k validated above"),
            };
            let zc = z + c;
            if normalize { zc / s } else { zc }
        };
        result.push(value);
    }

    Ok(result)
}

/// Build the simple-adjacency view (no self-loops, no parallel edges,
/// neighbours sorted ascending) plus the loop-aware degree for every
/// vertex. The latter counts each self-loop **twice** to mirror the C
/// reference's `IGRAPH_LOOPS` mode.
fn build_simple_adjacency(graph: &Graph) -> IgraphResult<(Vec<Vec<VertexId>>, Vec<u32>)> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    let mut deg: Vec<u32> = vec![0; n_us];
    for v in 0..n {
        // `Graph::degree` already implements IGRAPH_LOOPS_TWICE semantics
        // (each self-loop counts 2 in an undirected graph), so reuse it
        // verbatim instead of re-deriving from `neighbors()` (which
        // returns the loop endpoint twice and would double-count).
        let d = u32::try_from(graph.degree(v)?)
            .map_err(|_| IgraphError::Internal("ecc: degree exceeds u32::MAX"))?;
        deg[v as usize] = d;

        let raw = graph.neighbors(v)?;
        let mut simple: Vec<VertexId> = raw.into_iter().filter(|&u| u != v).collect();
        simple.sort_unstable();
        simple.dedup();
        adj.push(simple);
    }
    Ok((adj, deg))
}

/// k=3 inner kernel: returns `(z, s)` for an edge `(v1, v2)`.
#[allow(clippy::cast_precision_loss)]
fn ecc3_pair(adj: &[Vec<VertexId>], deg: &[u32], v1: VertexId, v2: VertexId) -> (f64, f64) {
    let a1 = &adj[v1 as usize];
    let a2 = &adj[v2 as usize];
    let z = intersection_size_sorted(a1, a2) as f64;
    let d1 = f64::from(deg[v1 as usize]);
    let d2 = f64::from(deg[v2 as usize]);
    let s = d1.min(d2) - 1.0;
    (z, s)
}

/// k=4 inner kernel: returns `(z, s)` for an edge `(v1, v2)`.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
fn ecc4_pair(adj: &[Vec<VertexId>], deg: &[u32], v1: VertexId, v2: VertexId) -> (f64, f64) {
    // Iterate from the smaller-degree endpoint so we visit fewer
    // intermediates. Matches the swap in `igraph_i_ecc4_*`.
    let (lo, hi) = if deg[v1 as usize] <= deg[v2 as usize] {
        (v1, v2)
    } else {
        (v2, v1)
    };
    let a_lo = &adj[lo as usize];
    let a_hi = &adj[hi as usize];
    let mut z = 0.0_f64;
    for &v3 in a_lo {
        if v3 == hi {
            continue;
        }
        let a3 = &adj[v3 as usize];
        // |N(hi) ∩ N(v3)| - 1 — the -1 strips the (hi, v3) edge itself
        // when it exists; for non-neighbours the term is still
        // intersection-size minus one, matching the C reference.
        let inter = intersection_size_sorted(a_hi, a3) as i64;
        z += (inter - 1) as f64;
    }
    let d_lo = f64::from(deg[lo as usize]);
    let d_hi = f64::from(deg[hi as usize]);
    let s = (d_lo - 1.0) * (d_hi - 1.0);
    (z, s)
}

/// Linear-merge intersection size for two sorted, deduplicated lists.
fn intersection_size_sorted(a: &[VertexId], b: &[VertexId]) -> usize {
    let mut i = 0usize;
    let mut j = 0usize;
    let mut count = 0usize;
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        (a - b).abs() <= eps
    }

    fn vec_approx_eq(got: &[f64], want: &[f64]) {
        assert_eq!(
            got.len(),
            want.len(),
            "length mismatch: {got:?} vs {want:?}"
        );
        for (i, (&g, &w)) in got.iter().zip(want.iter()).enumerate() {
            assert!(
                approx_eq(g, w, 1e-6),
                "index {i}: got {g}, want {w}\nfull: {got:?}\nwant: {want:?}"
            );
        }
    }

    fn k5() -> Graph {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        g
    }

    fn p2() -> Graph {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g
    }

    #[test]
    fn null_graph_returns_empty_for_all_edges() {
        let g = Graph::with_vertices(0);
        for k in [3u32, 4] {
            let r = ecc(&g, None, k, false, true).unwrap();
            assert!(r.is_empty());
        }
    }

    #[test]
    fn singleton_returns_empty_for_all_edges() {
        let g = Graph::with_vertices(1);
        for k in [3u32, 4] {
            let r = ecc(&g, None, k, false, true).unwrap();
            assert!(r.is_empty());
        }
    }

    #[test]
    fn p2_normalises_to_nan() {
        // P_2: single edge, both endpoints have degree 1 → s = 0 → NaN.
        let g = p2();
        for k in [3u32, 4] {
            let r = ecc(&g, None, k, false, true).unwrap();
            assert_eq!(r.len(), 1);
            assert!(r[0].is_nan(), "k = {k}: got {r:?}");
        }
        // Without normalisation we just get z (= 0).
        for k in [3u32, 4] {
            let r = ecc(&g, None, k, false, false).unwrap();
            assert_eq!(r, vec![0.0]);
        }
    }

    #[test]
    fn k5_k3_offset_false_normalize_true() {
        let g = k5();
        let r = ecc(&g, None, 3, false, true).unwrap();
        // Matches igraph_ecc.out for K_5: every edge yields 1.
        vec_approx_eq(&r, &[1.0; 10]);
    }

    #[test]
    fn k5_k4_offset_false_normalize_true() {
        let g = k5();
        let r = ecc(&g, None, 4, false, true).unwrap();
        // Matches igraph_ecc.out for K_5: every edge yields 2/3.
        vec_approx_eq(&r, &[2.0 / 3.0; 10]);
    }

    #[test]
    fn k5_offset_true_normalize_false_returns_z_plus_one() {
        let g = k5();
        let r = ecc(&g, None, 3, true, false).unwrap();
        // z = 3 (each edge sits in 3 triangles in K_5) → 3 + 1 = 4.
        vec_approx_eq(&r, &[4.0; 10]);
    }

    #[test]
    fn k5_offset_true_normalize_true_is_radicchi_canonical() {
        let g = k5();
        let r = ecc(&g, None, 3, true, true).unwrap();
        // In K_5 each non-loop edge sits in 3 triangles; loop-aware
        // endpoint degree is 4 → s = min(4,4) - 1 = 3.
        // Radicchi canonical = (z + 1) / s = (3 + 1) / 3 = 4/3.
        vec_approx_eq(&r, &[4.0 / 3.0; 10]);
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn k5_with_self_loops_k3_yields_nan_for_loops_and_0_6_for_others() {
        // K_5 plus a self-loop on every vertex (5 + 5 = 10 edges? No,
        // 5 self-loops + C(5,2) = 5 + 10 = 15 edges). The C .out
        // confirms 15 entries, with NaN at the self-loop positions.
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            g.add_edge(u, u).unwrap();
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = ecc(&g, None, 3, false, true).unwrap();
        assert_eq!(r.len(), 15);
        // The five self-loops are at indices where edge_source == edge_target.
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (a, b) = g.edge(eid).unwrap();
            if a == b {
                assert!(
                    r[eid as usize].is_nan(),
                    "edge {eid} (self-loop): got {}",
                    r[eid as usize]
                );
            } else {
                // z = 3 (common neighbours, excluding self-loops); loop-aware
                // degree = 4 + 2 = 6; s = 5 → 3/5 = 0.6.
                assert!(
                    approx_eq(r[eid as usize], 0.6, 1e-9),
                    "edge {eid}: got {}",
                    r[eid as usize]
                );
            }
        }
    }

    #[test]
    fn k5_subset_eids_matches_all_eids_order() {
        let g = k5();
        let all = ecc(&g, None, 3, false, true).unwrap();
        let subset_ids = vec![0u32, 3, 7];
        let sub = ecc(&g, Some(&subset_ids), 3, false, true).unwrap();
        assert_eq!(sub.len(), 3);
        for (i, &eid) in subset_ids.iter().enumerate() {
            assert!(
                approx_eq(sub[i], all[eid as usize], 1e-9),
                "subset[{i}] = {} vs all[{eid}] = {}",
                sub[i],
                all[eid as usize]
            );
        }
    }

    #[test]
    fn out_of_range_edge_errors() {
        let g = k5();
        let r = ecc(&g, Some(&[99u32]), 3, false, true);
        assert!(matches!(r, Err(IgraphError::EdgeOutOfRange { .. })));
    }

    #[test]
    fn k_below_three_errors() {
        let g = k5();
        let r = ecc(&g, None, 2, false, true);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn k_above_four_errors() {
        let g = k5();
        let r = ecc(&g, None, 5, false, true);
        assert!(matches!(r, Err(IgraphError::Unsupported(_))));
    }

    #[test]
    fn empty_eids_slice_returns_empty() {
        let g = k5();
        let r = ecc(&g, Some(&[]), 3, false, true).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn cycle_c4_k4_normalised_is_1() {
        // 4-cycle: each edge sits in exactly one 4-cycle. Endpoints have
        // degree 2, so s = (2-1)*(2-1) = 1 → C^(4) = 1.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let r = ecc(&g, None, 4, false, true).unwrap();
        vec_approx_eq(&r, &[1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn star_k3_yields_nan_then_normalised_zero_without_normalize() {
        // 4-star: center has degree 3, leaves have degree 1. s = min(d)-1
        // = 0 → NaN for every edge. z is always 0 (no triangles).
        let mut g = Graph::with_vertices(4);
        for v in 1..4u32 {
            g.add_edge(0, v).unwrap();
        }
        let r = ecc(&g, None, 3, false, true).unwrap();
        assert_eq!(r.len(), 3);
        for x in &r {
            assert!(x.is_nan(), "got {x}");
        }
        // Without normalisation z is just 0.
        let r = ecc(&g, None, 3, false, false).unwrap();
        vec_approx_eq(&r, &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn karate_first_few_match_c_reference() {
        // First 4 edges of Zachary's karate (k=3, offset=false, normalize=true)
        // from `references/igraph/tests/unit/igraph_ecc.out` line 73:
        //   0.875 0.555556 1 1 ...
        use std::fs::File;
        use std::path::PathBuf;
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures/karate.edges");
        let g = crate::algorithms::io::read_edgelist(File::open(path).unwrap()).unwrap();
        let r = ecc(&g, None, 3, false, true).unwrap();
        let want_first = [0.875, 0.555_555_555_555_555_6, 1.0, 1.0];
        for (i, &w) in want_first.iter().enumerate() {
            assert!(
                approx_eq(r[i], w, 1e-6),
                "karate[{i}]: got {} want {w}",
                r[i]
            );
        }
    }

    #[test]
    fn ess_subset_order_preserved() {
        let g = k5();
        let r = ecc(&g, Some(&[7u32, 0, 4]), 3, false, true).unwrap();
        let all = ecc(&g, None, 3, false, true).unwrap();
        assert_eq!(r.len(), 3);
        assert!(approx_eq(r[0], all[7], 1e-9));
        assert!(approx_eq(r[1], all[0], 1e-9));
        assert!(approx_eq(r[2], all[4], 1e-9));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        (a - b).abs() <= eps
    }

    prop_compose! {
        fn small_undirected_graph()(n in 2u32..=8u32, edges_seed in any::<u64>()) -> Graph {
            let mut g = Graph::with_vertices(n);
            let mut rng = edges_seed;
            let target_m = ((n * (n - 1)) / 2).min(n + 4) as usize;
            let mut added = 0usize;
            let mut guard = 0usize;
            while added < target_m && guard < target_m * 8 + 4 {
                rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let u = ((rng >> 33) % u64::from(n)) as u32;
                rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let v = ((rng >> 33) % u64::from(n)) as u32;
                guard += 1;
                if u == v {
                    continue;
                }
                if g.add_edge(u, v).is_ok() {
                    added += 1;
                }
            }
            g
        }
    }

    proptest! {
        #[test]
        fn unnormalised_z_is_non_negative_integer(g in small_undirected_graph()) {
            for k in [3u32, 4] {
                let r = ecc(&g, None, k, false, false).unwrap();
                for (i, &x) in r.iter().enumerate() {
                    prop_assert!(x >= 0.0, "edge {i} k={k}: got {x}");
                    prop_assert!(x.fract() == 0.0, "edge {i} k={k}: expected integer, got {x}");
                }
            }
        }

        #[test]
        fn offset_shifts_by_one(g in small_undirected_graph()) {
            for k in [3u32, 4] {
                let without = ecc(&g, None, k, false, false).unwrap();
                let with = ecc(&g, None, k, true, false).unwrap();
                prop_assert_eq!(without.len(), with.len());
                for (i, (&a, &b)) in without.iter().zip(with.iter()).enumerate() {
                    prop_assert!(approx_eq(b - a, 1.0, 1e-9),
                        "edge {} k={}: with - without = {}", i, k, b - a);
                }
            }
        }

        #[test]
        fn subset_matches_full_sweep(g in small_undirected_graph()) {
            let m = g.ecount();
            if m == 0 { return Ok(()); }
            let all = ecc(&g, None, 3, false, true).unwrap();
            let sub_ids: Vec<u32> = (0..m as u32).rev().collect();
            let sub = ecc(&g, Some(&sub_ids), 3, false, true).unwrap();
            for (i, &eid) in sub_ids.iter().enumerate() {
                prop_assert!(approx_eq(sub[i], all[eid as usize], 1e-9),
                    "sub[{}] = {} all[{}] = {}", i, sub[i], eid, all[eid as usize]);
            }
        }
    }
}
