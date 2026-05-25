//! Line graph constructor (ALGO-CN-015).
//!
//! Counterpart of `igraph_linegraph()` in
//! `references/igraph/src/constructors/linegraph.c:31-174`.
//!
//! The line graph `L(G)` of `G` has one vertex for every edge of `G`
//! (edge id `e` ↔ L-vertex id `e`). Adjacency depends on the directedness
//! of `G`:
//!
//! * **Undirected `G`**: two distinct L-vertices `e1`, `e2` are connected
//!   by an L-edge for every shared endpoint between the two G-edges.
//!   Multigraphs therefore yield multi-L-edges (one per shared endpoint).
//!   A G-self-loop counts its endpoint twice — so an L-vertex from a
//!   self-loop has a single L-self-loop on itself, plus *two* L-edges
//!   to every other G-edge sharing the same vertex (since the loop "uses
//!   two endpoints" at that vertex).
//! * **Directed `G`**: an L-arc goes from `e ↦ e1` whenever the target of
//!   `e` equals the source of `e1` — i.e. the two G-arcs can be chained.
//!
//! The result is directed iff `G` is directed.
//!
//! Edge emission order in both branches matches upstream byte-for-byte
//! so the three-source conformance test can compare raw ordered edge
//! lists rather than canonicalised multisets.
//!
//! Time complexity: `O(|V(G)| + |E(G)|)` — exactly the same as upstream;
//! the inner walks are bounded by `Σ_v deg(v) = 2|E|` so amortised cost
//! per L-edge is constant.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Build the line graph `L(G)` of `graph`.
///
/// `L(G)` has one vertex for every edge of `graph` (edge id `e` maps to
/// L-vertex id `e`). For an undirected input two L-vertices are
/// connected by one L-edge per shared endpoint between their G-edges
/// (so multigraphs and self-loops behave per the upstream docstring).
/// For a directed input an L-arc `e → e1` exists exactly when the target
/// of `e` equals the source of `e1`. The result inherits `graph`'s
/// directedness.
///
/// # Errors
///
/// Propagates errors from [`Graph::new`] / [`Graph::add_edges`]; the
/// algorithm itself does not introduce new failure modes.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, linegraph};
///
/// // L of the path P_4 (edges e0=(0,1), e1=(1,2), e2=(2,3)) is the path
/// // P_3 on the three L-vertices 0, 1, 2.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let l = linegraph(&g).unwrap();
/// assert_eq!(l.vcount(), 3);
/// assert_eq!(l.ecount(), 2);
/// assert!(!l.is_directed());
/// ```
pub fn linegraph(graph: &Graph) -> IgraphResult<Graph> {
    if graph.is_directed() {
        linegraph_directed(graph)
    } else {
        linegraph_undirected(graph)
    }
}

/// Build a per-vertex incidence table with LOOPS-TWICE semantics — the
/// same semantics upstream's `igraph_incident(_, IGRAPH_ALL, IGRAPH_LOOPS)`
/// produces (where `IGRAPH_LOOPS == IGRAPH_LOOPS_TWICE`). A self-loop on
/// `v` appears in `incident[v]` exactly twice; a non-loop edge `(u, v)`
/// with `u != v` appears once in each of `incident[u]` and `incident[v]`.
/// Edges are pushed in global edge-id order, which makes per-source
/// incidence lists monotonic in edge id and lets `e2 < e1` comparisons
/// short-circuit naturally.
fn build_incident_loops_twice(graph: &Graph) -> IgraphResult<Vec<Vec<EdgeId>>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut incident: Vec<Vec<EdgeId>> = vec![Vec::new(); vcount];
    for eid in 0..u32::try_from(ecount).map_err(|_| {
        crate::core::IgraphError::InvalidArgument(format!(
            "linegraph: ecount {ecount} does not fit u32"
        ))
    })? {
        let (u, v) = graph.edge(eid)?;
        incident[u as usize].push(eid);
        // Push at `v` too. For a self-loop u == v this pushes a second
        // copy at the same vertex (LOOPS-TWICE).
        incident[v as usize].push(eid);
    }
    Ok(incident)
}

fn linegraph_undirected(graph: &Graph) -> IgraphResult<Graph> {
    let ecount = graph.ecount();
    let new_vcount = u32::try_from(ecount).map_err(|_| {
        crate::core::IgraphError::InvalidArgument(format!(
            "linegraph: source ecount {ecount} cannot index a u32 vertex space"
        ))
    })?;

    let incident = build_incident_loops_twice(graph)?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    for e1 in 0..new_vcount {
        let (from, to) = graph.edge(e1)?;

        // Walk edges incident at `from`: emit (e1, e2) for every e2 < e1.
        // With LOOPS-TWICE incidence a self-loop at `from` shows up here
        // twice — which is exactly the "self-loop counts its endpoint
        // twice" rule from the upstream docstring.
        for &e2 in &incident[from as usize] {
            if e2 < e1 {
                edges.push((e1, e2));
            }
        }
        // Walk edges incident at `to`. For a non-loop G-edge this is a
        // distinct vertex from `from`; for a G-self-loop (from == to)
        // this re-walks the same list — adding a second pass for every
        // shared edge there.
        for &e2 in &incident[to as usize] {
            if e2 < e1 {
                edges.push((e1, e2));
            }
        }
        // A G-self-loop yields one L-self-loop on its L-vertex.
        if from == to {
            edges.push((e1, e1));
        }
    }

    let mut out = Graph::new(new_vcount, false)?;
    out.add_edges(edges)?;
    Ok(out)
}

fn linegraph_directed(graph: &Graph) -> IgraphResult<Graph> {
    let ecount = graph.ecount();
    let new_vcount = u32::try_from(ecount).map_err(|_| {
        crate::core::IgraphError::InvalidArgument(format!(
            "linegraph: source ecount {ecount} cannot index a u32 vertex space"
        ))
    })?;

    // Per-vertex incoming edge list. Build it by walking every edge in
    // edge-id order so the per-vertex lists are sorted, exactly matching
    // the order `igraph_incident(_, IGRAPH_IN, _)` would have produced in
    // upstream.
    let vcount = graph.vcount() as usize;
    let mut incoming: Vec<Vec<EdgeId>> = vec![Vec::new(); vcount];
    for eid in 0..new_vcount {
        let (_from, to) = graph.edge(eid)?;
        incoming[to as usize].push(eid);
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    for e1 in 0..new_vcount {
        let (from, _to) = graph.edge(e1)?;
        for &e in &incoming[from as usize] {
            // Upstream pushes (e, e1) — the incoming-at-`from` edge as
            // source, the current edge as target, since arc `e` ends at
            // `from` and arc `e1` starts at `from`.
            edges.push((e, e1));
        }
    }

    let mut out = Graph::new(new_vcount, true)?;
    out.add_edges(edges)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut out = Vec::with_capacity(g.ecount());
        for e in 0..ec {
            out.push(g.edge(e).expect("edge in range"));
        }
        out
    }

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).expect("empty G");
        let l = linegraph(&g).expect("L(G)");
        assert_eq!(l.vcount(), 0);
        assert_eq!(l.ecount(), 0);
        assert!(!l.is_directed());
    }

    #[test]
    fn no_edges_graph() {
        let g = Graph::new(5, false).expect("isolated vertices");
        let l = linegraph(&g).expect("L(G)");
        assert_eq!(l.vcount(), 0);
        assert_eq!(l.ecount(), 0);
    }

    #[test]
    fn path_p4_undirected() {
        // P_4: 0-1-2-3 with edges e0=(0,1), e1=(1,2), e2=(2,3).
        // L(P_4) = P_3 on vertices {0, 1, 2} with edges {0-1, 1-2}.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let l = linegraph(&g).expect("L(P_4)");
        assert_eq!(l.vcount(), 3);
        assert_eq!(l.ecount(), 2);
        // Emission order per upstream: when e1=1 we push (1, 0) (shared
        // endpoint at vertex 1); when e1=2 we push (2, 1) (shared at 2).
        // Undirected Graph canonicalises endpoints to (min, max).
        assert_eq!(dump_edges(&l), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn triangle_k3_undirected() {
        // K_3: vertices 0,1,2, edges e0=(0,1), e1=(0,2), e2=(1,2).
        // Every pair shares an endpoint → L(K_3) = K_3.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let l = linegraph(&g).expect("L(K_3)");
        assert_eq!(l.vcount(), 3);
        assert_eq!(l.ecount(), 3);
        // Upstream emission: e1=1 walks from=0 → e2=0 < 1: (1,0); walks
        // to=2 → no e2<1 there. e1=2 walks from=1 → e2=0 < 2: (2,0);
        // walks to=2 → e2=1 < 2: (2,1).
        // Canonicalised to (min, max) by the undirected Graph store.
        assert_eq!(dump_edges(&l), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn star_s4_undirected() {
        // Star with centre 0, leaves 1..=3. All three edges share vertex
        // 0, so L = K_3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let l = linegraph(&g).expect("L(star)");
        assert_eq!(l.vcount(), 3);
        assert_eq!(l.ecount(), 3);
    }

    #[test]
    fn self_loop_undirected() {
        // Single self-loop at vertex 0. L(G) has one vertex and one
        // self-loop (the self-loop-on-self-loop case).
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        let l = linegraph(&g).expect("L(self-loop)");
        assert_eq!(l.vcount(), 1);
        assert_eq!(l.ecount(), 1);
        assert_eq!(dump_edges(&l), vec![(0, 0)]);
    }

    #[test]
    fn self_loop_plus_non_loop_undirected() {
        // e0 = (0, 0) self-loop; e1 = (0, 1) non-loop. With LOOPS-TWICE:
        //   incident[0] = [0, 0, 1], incident[1] = [1].
        // - e1=0 (self-loop): walks incident[0] twice, no e2 < 0 in
        //   either walk; from==to → push (0, 0).
        // - e1=1 (non-loop): walks incident[from=0] = [0, 0, 1] → e2=0,
        //   e2=0 both < 1 → push (1, 0), (1, 0). Walks incident[to=1] =
        //   [1] → no e2 < 1.
        // So output edges = [(0,0), (1,0), (1,0)] in raw order. After
        // the undirected store canonicalises to (min, max):
        // [(0,0), (0,1), (0,1)] — two parallel L-edges between the
        // self-loop's L-vertex and the non-loop's L-vertex.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 2);
        assert_eq!(l.ecount(), 3);
        assert_eq!(dump_edges(&l), vec![(0, 0), (0, 1), (0, 1)]);
    }

    #[test]
    fn non_loop_then_self_loop_undirected() {
        // Now swap order: e0 = (0, 1) non-loop, e1 = (0, 0) self-loop at 0.
        // - e1=0: from=0, to=1; no e2<0 in either incident list; from!=to.
        //   No edges emitted.
        // - e1=1: from=0, to=0 (self-loop). Walk from=0 → e2=0<1: (1,0).
        //   Walk to=0 → e2=0<1: (1,0). Push (1,1) for the self-loop.
        // Output edges = [(1,0), (1,0), (1,1)] — TWO edges between L-vertex
        // 1 (the self-loop) and L-vertex 0 (the non-loop) + a self-loop.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 0).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 2);
        assert_eq!(l.ecount(), 3);
        // Canonicalised: (1,0) → (0,1); the self-loop on 1 stays as (1,1).
        assert_eq!(dump_edges(&l), vec![(0, 1), (0, 1), (1, 1)]);
    }

    #[test]
    fn multi_edge_undirected() {
        // Two parallel edges between 0 and 1: e0 = (0, 1), e1 = (0, 1).
        // Each shares BOTH endpoints, so L emits two edges (1, 0).
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 2);
        assert_eq!(l.ecount(), 2);
        assert_eq!(dump_edges(&l), vec![(0, 1), (0, 1)]);
    }

    #[test]
    fn directed_chain() {
        // 0 -> 1 -> 2: e0=(0,1), e1=(1,2). L has arc (e0 -> e1) because
        // target(e0) = 1 = source(e1).
        let mut g = Graph::new(3, true).expect("directed");
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 2);
        assert_eq!(l.ecount(), 1);
        assert!(l.is_directed());
        assert_eq!(dump_edges(&l), vec![(0, 1)]);
    }

    #[test]
    fn directed_fan_in() {
        // Multiple incoming arcs at the fan vertex: 0->2, 1->2, 2->3.
        // Edges: e0=(0,2), e1=(1,2), e2=(2,3). target(e0)=target(e1)=2,
        // source(e2)=2 — so L gets (e0->e2), (e1->e2).
        let mut g = Graph::new(4, true).expect("directed");
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 3);
        assert_eq!(l.ecount(), 2);
        // Upstream emits per-source: for e1=2 (source=2), incoming at 2
        // = [e0, e1] → push (e0, 2), (e1, 2).
        assert_eq!(dump_edges(&l), vec![(0, 2), (1, 2)]);
    }

    #[test]
    fn directed_self_loop() {
        // Self-loop at 0: e0 = (0, 0). target(e0) = 0 = source(e0), so
        // L(G) has a self-loop on the single L-vertex.
        let mut g = Graph::new(1, true).expect("directed");
        g.add_edge(0, 0).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 1);
        assert_eq!(l.ecount(), 1);
        assert!(l.is_directed());
        assert_eq!(dump_edges(&l), vec![(0, 0)]);
    }

    #[test]
    fn directed_isolated_arcs() {
        // 0->1 and 2->3 — no chaining possible.
        let mut g = Graph::new(4, true).expect("directed");
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let l = linegraph(&g).expect("L");
        assert_eq!(l.vcount(), 2);
        assert_eq!(l.ecount(), 0);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::collection::vec as pvec;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn arb_undirected_graph() -> impl Strategy<Value = Graph> {
        // 0..=8 vertices, 0..=12 edges. Edges may form multi-edges and
        // self-loops since linegraph is documented for both.
        (1u32..=8u32).prop_flat_map(|n| {
            let edge = (0u32..n, 0u32..n);
            pvec(edge, 0..=12).prop_map(move |raw| {
                let mut g = Graph::with_vertices(n);
                for (u, v) in raw {
                    g.add_edge(u, v).expect("add_edge");
                }
                g
            })
        })
    }

    fn arb_directed_graph() -> impl Strategy<Value = Graph> {
        (1u32..=8u32).prop_flat_map(|n| {
            let edge = (0u32..n, 0u32..n);
            pvec(edge, 0..=12).prop_map(move |raw| {
                let mut g = Graph::new(n, true).expect("directed");
                for (u, v) in raw {
                    g.add_edge(u, v).expect("add_edge");
                }
                g
            })
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// L(G) has exactly `ecount(G)` vertices and inherits `G`'s directedness.
        #[test]
        fn vcount_and_directedness_match(g in arb_undirected_graph()) {
            let l = linegraph(&g).expect("L(G)");
            prop_assert_eq!(l.vcount() as usize, g.ecount());
            prop_assert!(!l.is_directed());
        }

        #[test]
        fn directed_vcount_and_directedness_match(g in arb_directed_graph()) {
            let l = linegraph(&g).expect("L(G)");
            prop_assert_eq!(l.vcount() as usize, g.ecount());
            prop_assert!(l.is_directed());
        }

        /// Reference oracle for undirected L(G): build the expected multiset
        /// of L-edges directly from the per-vertex endpoint multiset of G
        /// and compare against the algorithm's output as a multiset.
        #[test]
        fn undirected_edge_set_matches_endpoint_oracle(g in arb_undirected_graph()) {
            let ec = u32::try_from(g.ecount()).unwrap();

            // Build LOOPS-TWICE per-vertex incidence: every edge eid is
            // pushed at *both* endpoints; a self-loop therefore appears
            // twice in the list of its single endpoint.
            let vcount = g.vcount() as usize;
            let mut incident: Vec<Vec<u32>> = vec![Vec::new(); vcount];
            for eid in 0..ec {
                let (u, v) = g.edge(eid).unwrap();
                incident[u as usize].push(eid);
                incident[v as usize].push(eid);
            }

            // Expected multiset, by closed-form rule: for every unordered
            // pair (e1, e2) with e2 < e1, the multiplicity equals the
            // number of shared endpoints (0, 1, or 2). For e2 == e1
            // (self-loop in G at v), one L-self-loop is added.
            let mut expected: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for e1 in 0..ec {
                let (a, b) = g.edge(e1).unwrap();
                for &e2 in &incident[a as usize] {
                    if e2 < e1 {
                        *expected.entry((e1, e2)).or_insert(0) += 1;
                    }
                }
                for &e2 in &incident[b as usize] {
                    if e2 < e1 {
                        *expected.entry((e1, e2)).or_insert(0) += 1;
                    }
                }
                if a == b {
                    *expected.entry((e1, e1)).or_insert(0) += 1;
                }
            }

            let l = linegraph(&g).expect("L(G)");
            let lec = u32::try_from(l.ecount()).unwrap();
            let mut got: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for e in 0..lec {
                let (u, v) = l.edge(e).unwrap();
                // Canonicalise the unordered key: smaller, larger.
                let key = if u >= v { (u, v) } else { (v, u) };
                *got.entry(key).or_insert(0) += 1;
            }
            prop_assert_eq!(got, expected);
        }

        /// Reference oracle for directed L(G): for every G-edge `e1` with
        /// source `s`, every incoming G-arc `e` at `s` produces exactly
        /// one L-arc `(e, e1)`.
        #[test]
        fn directed_arc_set_matches_chain_oracle(g in arb_directed_graph()) {
            let ec = u32::try_from(g.ecount()).unwrap();
            let vcount = g.vcount() as usize;
            let mut incoming: Vec<Vec<u32>> = vec![Vec::new(); vcount];
            for eid in 0..ec {
                let (_u, v) = g.edge(eid).unwrap();
                incoming[v as usize].push(eid);
            }
            let mut expected: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for e1 in 0..ec {
                let (s, _t) = g.edge(e1).unwrap();
                for &e in &incoming[s as usize] {
                    *expected.entry((e, e1)).or_insert(0) += 1;
                }
            }

            let l = linegraph(&g).expect("L(G)");
            let lec = u32::try_from(l.ecount()).unwrap();
            let mut got: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for e in 0..lec {
                let (u, v) = l.edge(e).unwrap();
                *got.entry((u, v)).or_insert(0) += 1;
            }
            prop_assert_eq!(got, expected);
        }
    }
}
