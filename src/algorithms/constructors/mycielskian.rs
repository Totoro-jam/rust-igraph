//! Mycielski construction (ALGO-CN-019).
//!
//! Counterpart of `igraph_mycielskian()` and `igraph_mycielski_graph()` in
//! `references/igraph/src/constructors/mycielskian.c:97-244`.
//!
//! The Mycielski construction `M(G)` takes a graph `G` on `n` vertices and
//! `m` edges and produces a larger graph with `2n + 1` vertices and `3m + n`
//! edges that increases the chromatic number by one while preserving the
//! triangle-free property.
//!
//! Per iteration, label the original vertices `v_0..v_{n-1}` and add:
//! * `n` shadow vertices `u_0..u_{n-1}`, where `u_i` is connected to every
//!   original neighbour of `v_i` (so for each `(v_i, v_j)` in `G` we emit
//!   `(v_i, u_j)` and `(v_j, u_i)`);
//! * a single apex vertex `w` connected to every `u_i`.
//!
//! Two corner cases are folded inline so iterative chains stay connected:
//! the null graph (`vcount = 0`) consumes one `k` step by becoming the
//! singleton, and the singleton (`vcount = 1`) consumes one `k` step by
//! becoming the two-path `P_2`. After those reductions the loop runs the
//! literal recurrence `vcount' = 2*vcount + 1`, `ecount' = 3*ecount + vcount`.
//!
//! The canonical Mycielski graphs `M_k` are produced by [`mycielski_graph`]:
//! `M_0` = null, `M_1` = singleton, `M_2` = `P_2`, `M_3` = `C_5`, `M_4` =
//! Grötzsch graph (11 vertices, 20 edges, triangle-free, chromatic number 4).
//!
//! Time complexity: `O(|V| · 2^k + |E| · 3^k)`.

use crate::algorithms::constructors::ring::ring_graph;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Apply `k` Mycielski iterations to `graph`.
///
/// Returns a new graph that preserves the input's directedness. Self-loops
/// and parallel edges in the input are propagated through the construction
/// rules literally — Mycielski is defined on simple graphs but the upstream
/// extension to multigraphs is preserved here.
///
/// For `k == 0` the result is a structural copy of `graph` (same vertices,
/// same edges, same directedness).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the computed `2^k` vcount or
///   `3^k`-scaled ecount overflows `u32` / `usize`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mycielskian};
///
/// // Mycielski of the null graph + 1 iteration = singleton.
/// let null = Graph::new(0, false).unwrap();
/// let m1 = mycielskian(&null, 1).unwrap();
/// assert_eq!(m1.vcount(), 1);
/// assert_eq!(m1.ecount(), 0);
///
/// // Mycielski of P_2 + 1 iteration = C_5 (Mycielski M_3).
/// let mut p2 = Graph::new(2, false).unwrap();
/// p2.add_edges([(0u32, 1u32)]).unwrap();
/// let c5 = mycielskian(&p2, 1).unwrap();
/// assert_eq!(c5.vcount(), 5);
/// assert_eq!(c5.ecount(), 5);
/// ```
pub fn mycielskian(graph: &Graph, k: u32) -> IgraphResult<Graph> {
    let directed = graph.is_directed();
    let (mut vcount, mut ecount, mut edges) = stage_input_edges(graph)?;
    let mut k_remaining = k;

    // Special case: null graph. Promote to singleton and consume one step.
    if vcount == 0 && k_remaining > 0 {
        vcount = 1;
        k_remaining -= 1;
    }

    // Special case: singleton. Promote to P_2 and consume one step.
    if vcount == 1 && k_remaining > 0 {
        vcount = 2;
        ecount = ecount.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(
                "mycielskian: singleton promotion ecount + 1 overflows u32".to_string(),
            )
        })?;
        edges.push(0);
        edges.push(1);
        k_remaining -= 1;
    }

    let (final_v, final_e) = project_counts(vcount, ecount, k_remaining)?;

    // Resize the flat edge buffer to its final capacity (2 entries per edge).
    let final_buffer_len = usize::try_from(final_e)
        .ok()
        .and_then(|m| m.checked_mul(2))
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "mycielskian: final edge buffer size 2 * {final_e} overflows usize"
            ))
        })?;
    edges.resize(final_buffer_len, 0);

    run_iterations(&mut edges, vcount, ecount, k_remaining)?;

    let mut result = Graph::new(final_v, directed)?;
    let pairs: Vec<(VertexId, VertexId)> = edges
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    result.add_edges(pairs)?;
    Ok(result)
}

/// Returns `(vcount, ecount, flat-edges)` for `graph` as a row-major
/// `[src0, dst0, src1, dst1, …]` buffer. Errors when `ecount` exceeds
/// `u32::MAX` or when the buffer length overflows `usize`.
fn stage_input_edges(graph: &Graph) -> IgraphResult<(u32, u32, Vec<u32>)> {
    let ecount = u32::try_from(graph.ecount()).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "mycielskian: input ecount = {} exceeds u32::MAX",
            graph.ecount()
        ))
    })?;
    let cap = usize::try_from(ecount)
        .ok()
        .and_then(|m| m.checked_mul(2))
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "mycielskian: input edge buffer size 2 * {ecount} overflows usize"
            ))
        })?;
    let mut edges: Vec<u32> = Vec::with_capacity(cap);
    for eid in 0..graph.ecount() {
        let id = u32::try_from(eid).map_err(|_| {
            IgraphError::InvalidArgument(format!("mycielskian: edge id {eid} exceeds u32::MAX"))
        })?;
        let (u, v) = graph.edge(id).map_err(|_| {
            IgraphError::InvalidArgument(format!("mycielskian: edge id {eid} not in range"))
        })?;
        edges.push(u);
        edges.push(v);
    }
    Ok((graph.vcount(), ecount, edges))
}

/// Apply the closed-form recurrence
/// `(vcount', ecount') = (2*vcount + 1, 3*ecount + vcount)` for `k` steps,
/// with overflow checks on every intermediate value.
fn project_counts(vcount: u32, ecount: u32, k: u32) -> IgraphResult<(u32, u32)> {
    let mut v = vcount;
    let mut e = ecount;
    for _ in 0..k {
        e = e
            .checked_mul(3)
            .and_then(|x| x.checked_add(v))
            .ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "mycielskian: ecount overflow at iter with vcount = {v}, ecount = {e}"
                ))
            })?;
        v = v
            .checked_mul(2)
            .and_then(|x| x.checked_add(1))
            .ok_or_else(|| {
                IgraphError::InvalidArgument(format!("mycielskian: vcount overflow (2 * {v} + 1)"))
            })?;
    }
    Ok((v, e))
}

/// Run `k` Mycielski iterations in place on the pre-sized `edges` buffer
/// starting from `(vcount, ecount)` after any null/singleton promotions.
fn run_iterations(edges: &mut [u32], vcount: u32, ecount: u32, k: u32) -> IgraphResult<()> {
    let mut edge_index: usize = 2usize
        .checked_mul(usize::try_from(ecount).map_err(|_| {
            IgraphError::InvalidArgument(format!("mycielskian: ecount {ecount} too large"))
        })?)
        .ok_or_else(|| {
            IgraphError::InvalidArgument("mycielskian: edge_index overflow".to_string())
        })?;
    let mut offset: u32 = vcount;

    for _ in 0..k {
        let prev_vcount: u32 = offset;
        let w: u32 = offset.checked_mul(2).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("mycielskian: w = 2 * {offset} overflows u32"))
        })?;
        let last_edge_index: usize = edge_index;

        // For each existing edge (v1, v2), emit (v1, offset + v2) and (v2, offset + v1).
        let mut j = 0usize;
        while j < last_edge_index {
            let v1 = edges[j];
            let v2 = edges[j + 1];
            let v1_shadow = offset.checked_add(v1).ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "mycielskian: shadow id {offset} + {v1} overflows u32"
                ))
            })?;
            let v2_shadow = offset.checked_add(v2).ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "mycielskian: shadow id {offset} + {v2} overflows u32"
                ))
            })?;
            edges[edge_index] = v1;
            edges[edge_index + 1] = v2_shadow;
            edges[edge_index + 2] = v2;
            edges[edge_index + 3] = v1_shadow;
            edge_index += 4;
            j += 2;
        }

        // Star edges: (j, w) for j in [prev_vcount, w).
        let mut star = prev_vcount;
        while star < w {
            edges[edge_index] = star;
            edges[edge_index + 1] = w;
            edge_index += 2;
            star += 1;
        }

        // offset' = offset * 2 + 1.
        offset = offset
            .checked_mul(2)
            .and_then(|o| o.checked_add(1))
            .ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "mycielskian: offset update (2 * {offset} + 1) overflows u32"
                ))
            })?;
    }
    Ok(())
}

/// Construct the canonical Mycielski graph `M_k`.
///
/// `M_0` is the null graph, `M_1` is a single vertex, `M_2` is the two-path
/// `P_2`, `M_3` is the five-cycle `C_5`, `M_4` is the Grötzsch graph
/// (11 vertices, 20 edges, triangle-free, chromatic number 4), and `M_k`
/// for `k > 4` is the `(k - 2)`-fold Mycielskian of `P_2`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the computed vcount / ecount for
///   the given `k` overflows `u32` / `usize`.
///
/// # Examples
///
/// ```
/// use rust_igraph::mycielski_graph;
///
/// // M_3 is the 5-cycle.
/// let c5 = mycielski_graph(3).unwrap();
/// assert_eq!(c5.vcount(), 5);
/// assert_eq!(c5.ecount(), 5);
///
/// // M_4 is the Grötzsch graph: 11 vertices, 20 edges, triangle-free.
/// let g = mycielski_graph(4).unwrap();
/// assert_eq!(g.vcount(), 11);
/// assert_eq!(g.ecount(), 20);
/// ```
pub fn mycielski_graph(k: u32) -> IgraphResult<Graph> {
    if k <= 1 {
        // M_0 = null, M_1 = singleton.
        return Graph::new(k, false);
    }
    // M_k for k >= 2 is the (k - 2)-fold Mycielskian of P_2.
    let p2 = ring_graph(2, false, false, false)?;
    mycielskian(&p2, k - 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_edges(g: &Graph) -> BTreeSet<(VertexId, VertexId)> {
        let mut s = BTreeSet::new();
        for eid in 0..u32::try_from(g.ecount()).unwrap() {
            let (a, b) = g.edge(eid).unwrap();
            s.insert(if a <= b { (a, b) } else { (b, a) });
        }
        s
    }

    fn degree(g: &Graph, v: VertexId) -> usize {
        g.neighbors(v).unwrap().len()
    }

    fn assert_simple_undirected(g: &Graph) {
        assert!(!g.is_directed());
        let mut seen = BTreeSet::new();
        for eid in 0..u32::try_from(g.ecount()).unwrap() {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "self-loop on edge {eid}: ({a},{b})");
            let key = if a <= b { (a, b) } else { (b, a) };
            assert!(seen.insert(key), "parallel edge {key:?}");
        }
    }

    #[test]
    fn k_zero_is_identity_undirected() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges([(0u32, 1u32), (1, 2), (2, 3), (3, 0)]).unwrap();
        let r = mycielskian(&g, 0).unwrap();
        assert_eq!(r.vcount(), 4);
        assert_eq!(r.ecount(), 4);
        assert!(!r.is_directed());
        assert_eq!(canonical_edges(&r), canonical_edges(&g));
    }

    #[test]
    fn k_zero_is_identity_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges([(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let r = mycielskian(&g, 0).unwrap();
        assert_eq!(r.vcount(), 3);
        assert_eq!(r.ecount(), 3);
        assert!(r.is_directed());
    }

    #[test]
    fn null_graph_promoted_to_singleton() {
        let null = Graph::new(0, false).unwrap();
        let r = mycielskian(&null, 1).unwrap();
        assert_eq!(r.vcount(), 1);
        assert_eq!(r.ecount(), 0);
    }

    #[test]
    fn null_graph_two_iterations_is_p2() {
        // M_0 + 1 → singleton; singleton + 1 → P_2.
        let null = Graph::new(0, false).unwrap();
        let r = mycielskian(&null, 2).unwrap();
        assert_eq!(r.vcount(), 2);
        assert_eq!(r.ecount(), 1);
        assert_eq!(canonical_edges(&r), [(0u32, 1u32)].into_iter().collect());
    }

    #[test]
    fn singleton_promoted_to_p2() {
        let single = Graph::new(1, false).unwrap();
        let r = mycielskian(&single, 1).unwrap();
        assert_eq!(r.vcount(), 2);
        assert_eq!(r.ecount(), 1);
        assert_eq!(canonical_edges(&r), [(0u32, 1u32)].into_iter().collect());
    }

    #[test]
    fn p2_one_iteration_is_c5() {
        // M(P_2) = C_5 = M_3.
        let mut p2 = Graph::new(2, false).unwrap();
        p2.add_edges([(0u32, 1u32)]).unwrap();
        let r = mycielskian(&p2, 1).unwrap();
        assert_eq!(r.vcount(), 5);
        assert_eq!(r.ecount(), 5);
        assert_simple_undirected(&r);
        // Every vertex in C_5 has degree 2.
        for v in 0..r.vcount() {
            assert_eq!(degree(&r, v), 2, "C_5 vertex {v} degree");
        }
    }

    #[test]
    fn mycielski_graph_m0_null() {
        let m = mycielski_graph(0).unwrap();
        assert_eq!(m.vcount(), 0);
        assert_eq!(m.ecount(), 0);
        assert!(!m.is_directed());
    }

    #[test]
    fn mycielski_graph_m1_singleton() {
        let m = mycielski_graph(1).unwrap();
        assert_eq!(m.vcount(), 1);
        assert_eq!(m.ecount(), 0);
    }

    #[test]
    fn mycielski_graph_m2_p2() {
        let m = mycielski_graph(2).unwrap();
        assert_eq!(m.vcount(), 2);
        assert_eq!(m.ecount(), 1);
        assert_eq!(canonical_edges(&m), [(0u32, 1u32)].into_iter().collect());
    }

    #[test]
    fn mycielski_graph_m3_c5() {
        let m = mycielski_graph(3).unwrap();
        assert_eq!(m.vcount(), 5);
        assert_eq!(m.ecount(), 5);
        assert_simple_undirected(&m);
        for v in 0..m.vcount() {
            assert_eq!(degree(&m, v), 2);
        }
    }

    #[test]
    fn mycielski_graph_m4_grotzsch() {
        // Grötzsch graph: 11 vertices, 20 edges, triangle-free, chromatic 4.
        let m = mycielski_graph(4).unwrap();
        assert_eq!(m.vcount(), 11);
        assert_eq!(m.ecount(), 20);
        assert_simple_undirected(&m);
        // Triangle-free check: for every undirected edge (u, v), no common neighbour.
        for eid in 0..u32::try_from(m.ecount()).unwrap() {
            let (u, v) = m.edge(eid).unwrap();
            let nu: BTreeSet<VertexId> = m.neighbors(u).unwrap().into_iter().collect();
            let nv: BTreeSet<VertexId> = m.neighbors(v).unwrap().into_iter().collect();
            let inter: BTreeSet<&VertexId> = nu.intersection(&nv).collect();
            assert!(inter.is_empty(), "triangle through edge ({u},{v})");
        }
    }

    #[test]
    fn mycielski_graph_m5_counts() {
        // Per upstream docs: n_k = 3 * 2^(k-2) - 1 = 23, m_k = (7 * 3^(k-2) + 1)/2 - 3*2^(k-2)
        //              = (7*9+1)/2 - 12 = 32 - 12 = 20? Recompute:
        // M_4 = mycielskian(P_2, 2): start vcount=2 ecount=1 → after 1 iter v=5 e=5 →
        //       after 2 iters v=11 e=20 ✓.
        // M_5 = mycielskian(P_2, 3): after 3 iters v=23 e=70.
        // Verify with the recurrence: e' = 3e + v; v' = 2v + 1.
        // Step 1: v=5, e=5. Step 2: v=11, e=3*5+5=20. Step 3: v=23, e=3*20+11=71. So 71.
        // Note: my hand-calc above was off; trust the recurrence.
        let m = mycielski_graph(5).unwrap();
        assert_eq!(m.vcount(), 23);
        assert_eq!(m.ecount(), 71);
        assert_simple_undirected(&m);
    }

    #[test]
    fn directed_iteration_preserves_directedness() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges([(0u32, 1u32)]).unwrap();
        let r = mycielskian(&g, 1).unwrap();
        assert!(r.is_directed());
        // Directed iteration: 2 vcount→5, 1 ecount + new = 3*1 + 2 = 5 arcs.
        assert_eq!(r.vcount(), 5);
        assert_eq!(r.ecount(), 5);
    }

    #[test]
    fn recurrence_after_two_iterations_on_k4() {
        // K_4: 4 vertices, 6 edges. After 1 iter v=9 e=22; after 2 iters v=19 e=75.
        let mut k4 = Graph::new(4, false).unwrap();
        k4.add_edges([(0u32, 1u32), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])
            .unwrap();
        let one = mycielskian(&k4, 1).unwrap();
        assert_eq!(one.vcount(), 9);
        assert_eq!(one.ecount(), 22);
        let two = mycielskian(&k4, 2).unwrap();
        assert_eq!(two.vcount(), 19);
        assert_eq!(two.ecount(), 75);
    }

    #[test]
    fn apex_vertex_degree_equals_prev_vcount() {
        // After applying mycielskian to G with vcount = n, the new apex is
        // the last vertex (id = 2n) and connects to the n shadow vertices.
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges([(0u32, 1u32), (1, 2), (2, 3)]).unwrap(); // P_4
        let r = mycielskian(&g, 1).unwrap();
        // vcount 4 → 9; apex = 8; shadow ids 4..8.
        let apex = r.vcount() - 1;
        let nbrs: BTreeSet<VertexId> = r.neighbors(apex).unwrap().into_iter().collect();
        assert_eq!(nbrs, (4u32..8).collect());
    }

    #[test]
    fn shadow_vertex_mirrors_original_neighbours_plus_apex() {
        // After M on K_3 (triangle): n=3, m=3. New v=7, edges:
        //  - 3 original triangle edges
        //  - per (i,j) emit (i, 3+j) and (j, 3+i): 6 new "shadow" edges
        //  - star: (3, 6), (4, 6), (5, 6) = 3 apex edges
        // Total: 3 + 6 + 3 = 12 = 3m + n = 9 + 3 ✓.
        let mut k3 = Graph::new(3, false).unwrap();
        k3.add_edges([(0u32, 1u32), (1, 2), (0, 2)]).unwrap();
        let r = mycielskian(&k3, 1).unwrap();
        assert_eq!(r.vcount(), 7);
        assert_eq!(r.ecount(), 12);
        // Shadow vertex u_0 (id=3) should connect to v_1 and v_2 (original
        // neighbours of v_0) plus apex w (id=6).
        let nbrs_u0: BTreeSet<VertexId> = r.neighbors(3).unwrap().into_iter().collect();
        assert_eq!(nbrs_u0, [1u32, 2, 6].into_iter().collect());
    }

    #[test]
    fn negative_k_rejected_at_type_level() {
        // u32 cannot represent negative; the C `< 0` rejection is enforced
        // by the type system. Smoke-test that k = u32::MAX without input
        // overflows cleanly via the InvalidArgument error path.
        let null = Graph::new(0, false).unwrap();
        let r = mycielskian(&null, 64);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn empty_input_k_zero_is_null_graph() {
        let null = Graph::new(0, false).unwrap();
        let r = mycielskian(&null, 0).unwrap();
        assert_eq!(r.vcount(), 0);
        assert_eq!(r.ecount(), 0);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// Small random simple graph with `n ∈ [0, 8]` and a Bernoulli edge.
    fn arb_small_graph() -> impl Strategy<Value = Graph> {
        (0u32..=8).prop_flat_map(|n| {
            // For each unordered pair (i, j) with i < j, decide independently.
            let pairs = if n <= 1 {
                0
            } else {
                (n * (n - 1) / 2) as usize
            };
            prop::collection::vec(any::<bool>(), pairs).prop_map(move |flags| {
                let mut g = Graph::new(n, false).unwrap();
                if n >= 2 {
                    let mut pair_idx = 0usize;
                    let mut edges = Vec::new();
                    for i in 0..n {
                        for j in (i + 1)..n {
                            if flags[pair_idx] {
                                edges.push((i, j));
                            }
                            pair_idx += 1;
                        }
                    }
                    g.add_edges(edges).unwrap();
                }
                g
            })
        })
    }

    proptest! {
        #[test]
        fn k_zero_preserves_counts(g in arb_small_graph()) {
            let r = mycielskian(&g, 0).unwrap();
            prop_assert_eq!(r.vcount(), g.vcount());
            prop_assert_eq!(r.ecount(), g.ecount());
            prop_assert_eq!(r.is_directed(), g.is_directed());
        }

        #[test]
        fn vcount_recurrence_holds(g in arb_small_graph(), k in 0u32..=4) {
            let r = mycielskian(&g, k).unwrap();
            // After the inline null→singleton (free vcount=1) and singleton→P_2
            // (free v=2,e=1) reductions, apply v' = 2v + 1 per remaining step.
            let mut vc = g.vcount();
            let mut ec = u32::try_from(g.ecount()).unwrap();
            let mut k_left = k;
            if vc == 0 && k_left > 0 { vc = 1; k_left -= 1; }
            if vc == 1 && k_left > 0 { vc = 2; ec += 1; k_left -= 1; }
            for _ in 0..k_left {
                ec = ec.checked_mul(3).unwrap().checked_add(vc).unwrap();
                vc = vc.checked_mul(2).unwrap().checked_add(1).unwrap();
            }
            prop_assert_eq!(r.vcount(), vc);
            prop_assert_eq!(u32::try_from(r.ecount()).unwrap(), ec);
        }

        #[test]
        fn no_edge_references_out_of_range(g in arb_small_graph(), k in 0u32..=3) {
            let r = mycielskian(&g, k).unwrap();
            for eid in 0..u32::try_from(r.ecount()).unwrap() {
                let (u, v) = r.edge(eid).unwrap();
                prop_assert!(u < r.vcount());
                prop_assert!(v < r.vcount());
            }
        }

        #[test]
        fn mycielski_graph_recurrence(k in 0u32..=6) {
            let m = mycielski_graph(k).unwrap();
            // Same recurrence rooted at P_2 for k >= 2; null/singleton otherwise.
            let (vc, ec) = match k {
                0 => (0u32, 0u32),
                1 => (1, 0),
                _ => {
                    let mut vc = 2u32;
                    let mut ec = 1u32;
                    for _ in 0..(k - 2) {
                        ec = ec.checked_mul(3).unwrap().checked_add(vc).unwrap();
                        vc = vc.checked_mul(2).unwrap().checked_add(1).unwrap();
                    }
                    (vc, ec)
                }
            };
            prop_assert_eq!(m.vcount(), vc);
            prop_assert_eq!(u32::try_from(m.ecount()).unwrap(), ec);
            prop_assert!(!m.is_directed());
        }

        #[test]
        fn shadow_of_simple_graph_is_simple(g in arb_small_graph(), k in 0u32..=3) {
            // For simple undirected input with no self-loops, every iteration
            // also produces simple undirected output (no self-loops, no
            // parallels). This is a structural property of the construction.
            let r = mycielskian(&g, k).unwrap();
            let mut seen = BTreeSet::new();
            for eid in 0..u32::try_from(r.ecount()).unwrap() {
                let (u, v) = r.edge(eid).unwrap();
                prop_assert_ne!(u, v);
                let key = if u <= v { (u, v) } else { (v, u) };
                prop_assert!(seen.insert(key));
            }
        }

        #[test]
        fn apex_vertex_is_universal_to_shadow_block(g in arb_small_graph(), k in 1u32..=3) {
            let r = mycielskian(&g, k).unwrap();
            // Final iteration's apex is the last vertex, and its degree
            // equals the per-iteration shadow-block size (i.e. half of
            // (vcount - 1) at that step). Quick smoke: apex degree > 0
            // whenever there's at least one iteration AND the post-reduction
            // vcount was > 0.
            let apex = r.vcount() - 1;
            prop_assert!(r.neighbors(apex).unwrap().len() > 0 || g.vcount() == 0);
        }
    }
}
