//! `all_st_mincuts` (ALGO-FL-032) — list every *minimum* (s,t) edge cut of a
//! directed graph.
//!
//! Counterpart of `igraph_all_st_mincuts` from
//! `references/igraph/src/flow/st-cuts.c:1335`.
//!
//! Like [`all_st_cuts`](super::all_st_cuts) this uses the Provan-Shier
//! paradigm (J. S. Provan and D. R. Shier, *A paradigm for listing
//! (s,t)-cuts in graphs*, Algorithmica 15:351–372, 1996), but it enumerates
//! only the cuts of *minimum total capacity*. The pipeline:
//!
//! 1. Compute a maximum `source → target` flow (value + per-edge flow).
//! 2. Build the **reverse residual** graph of that flow.
//! 3. Contract the residual by its **strongly connected components** and
//!    simplify, then relabel into topological order. Every minimum cut
//!    corresponds to a *closed set* (down-set) of this contracted DAG.
//! 4. Mark the projected vertices that are endpoints of a positive-flow edge
//!    as *active*.
//! 5. Run the shared Provan-Shier recursion
//!    ([`provan_shier_list`]) with an active-set pivot that walks in-neighbours
//!    of the topologically ordered residual.
//! 6. Map each closed set back to original vertices; the cut edges are the
//!    positive-flow edges leaving the source-side set.

// The contracted/residual relabelling maps go through `u32`/`usize` casts
// bounded by `vcount`, which `max_flow` already constrains.
#![allow(clippy::cast_possible_truncation)]

use std::collections::VecDeque;

use crate::algorithms::connectivity::strong::strongly_connected_components;
use crate::algorithms::flow::max_flow::max_flow;
use crate::algorithms::flow::provan_shier::{EStack, MarkedQueue, Pivot, provan_shier_list};
use crate::algorithms::operators::contract_vertices::contract_vertices;
use crate::algorithms::operators::permute_vertices::permute_vertices;
use crate::algorithms::operators::residual_graph::reverse_residual_graph;
use crate::algorithms::operators::simplify::simplify;
use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::topological_sorting::topological_sorting;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`all_st_mincuts`].
///
/// `cuts[i]` and `partition1s[i]` describe the same minimum cut:
/// `partition1s[i]` is the source-side vertex set `X` (always contains
/// `source`, never `target`), and `cuts[i]` lists the ids of the
/// positive-flow edges leaving `X` (`from ∈ X`, `to ∉ X`). Every listed cut
/// has total capacity equal to [`value`](Self::value). Vertices within each
/// partition and edge ids within each cut are sorted ascending.
#[derive(Clone, Debug, PartialEq)]
pub struct StMinCuts {
    /// The minimum cut value (maximum `source → target` flow).
    pub value: f64,
    /// Edge-id list of each minimum cut, sorted ascending.
    pub cuts: Vec<Vec<EdgeId>>,
    /// Source-side vertex set generating each cut, sorted ascending.
    pub partition1s: Vec<Vec<VertexId>>,
}

/// Pivot for listing **minimum** (s,t) cuts (`igraph_i_all_st_mincuts_pivot`).
///
/// Operates on the topologically ordered, SCC-contracted reverse residual
/// graph. A vertex is *active* iff it is an endpoint of a positive-flow edge
/// in the original graph (projected through the contraction).
struct MincutsPivot {
    /// In-neighbour adjacency of the residual graph (new/topological ids).
    in_adj: Vec<Vec<VertexId>>,
    /// `active[v]` — `v` is an endpoint of a positive-flow projected edge.
    active: Vec<bool>,
}

impl MincutsPivot {
    fn new(residual: &Graph, active: Vec<bool>) -> IgraphResult<Self> {
        let n = residual.vcount() as usize;
        let mut in_adj = vec![Vec::new(); n];
        for (v, slot) in in_adj.iter_mut().enumerate() {
            *slot = residual.in_neighbors_vec(v as VertexId)?;
        }
        Ok(Self { in_adj, active })
    }

    /// `igraph_i_all_st_mincuts_minimal`: the set of active vertices that are
    /// minimal w.r.t. reachability in the residual DAG (no in-path from
    /// another already-minimal vertex). The residual is in topological order,
    /// so a single forward pass suffices.
    fn minimal(&self, s: &MarkedQueue) -> Vec<VertexId> {
        let n = self.in_adj.len();
        let mut connected = vec![false; n];
        let mut minimal: Vec<VertexId> = Vec::new();
        for i in 0..n {
            if s.iselement(i as VertexId) {
                continue;
            }
            let conn = self.in_adj[i].iter().any(|&nei| connected[nei as usize]);
            if conn {
                connected[i] = true;
            } else if self.active[i] {
                minimal.push(i as VertexId);
                connected[i] = true;
            }
        }
        minimal
    }

    /// In-direction BFS from `root`, visiting only vertices in `keep`.
    fn bfs_in(&self, root: VertexId, keep: &[bool]) -> Vec<VertexId> {
        let n = self.in_adj.len();
        let mut visited = vec![false; n];
        let mut order: Vec<VertexId> = Vec::new();
        if !keep[root as usize] {
            return order;
        }
        let mut queue: VecDeque<VertexId> = VecDeque::new();
        visited[root as usize] = true;
        order.push(root);
        queue.push_back(root);
        while let Some(u) = queue.pop_front() {
            for &nei in &self.in_adj[u as usize] {
                if keep[nei as usize] && !visited[nei as usize] {
                    visited[nei as usize] = true;
                    order.push(nei);
                    queue.push_back(nei);
                }
            }
        }
        order
    }
}

impl Pivot for MincutsPivot {
    fn pivot(
        &mut self,
        s: &MarkedQueue,
        t: &EStack,
        _source: VertexId,
        target: VertexId,
    ) -> IgraphResult<(VertexId, Vec<VertexId>)> {
        let n = self.in_adj.len();
        if s.size() == n {
            return Ok((0, Vec::new()));
        }

        let mut keep = vec![false; n];
        for (i, slot) in keep.iter_mut().enumerate() {
            *slot = !s.iselement(i as VertexId);
        }

        let m = self.minimal(s);

        // Find a minimal element that is neither the target nor forbidden.
        let chosen = m
            .iter()
            .copied()
            .find(|&min| min != target && !t.iselement(min));

        if let Some(v) = chosen {
            // I(S,v): all residual vertices that can reach v (within `keep`)
            // and are not already in S.
            let isv_min = self.bfs_in(v, &keep);
            let isv: Vec<VertexId> = isv_min.into_iter().filter(|&u| !s.iselement(u)).collect();
            return Ok((v, isv));
        }

        Ok((0, Vec::new()))
    }
}

/// List all *minimum* (s,t) edge cuts of a directed graph (Provan-Shier).
///
/// # Arguments
///
/// * `graph`    — directed input graph. Undirected graphs are rejected.
/// * `source`   — source vertex id.
/// * `target`   — target vertex id (must differ from `source`).
/// * `capacity` — optional per-edge capacities in edge-id order. When `None`,
///   every edge has unit capacity. All capacities must be strictly positive.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] when `graph` is undirected, when
///   `source == target`, when `capacity.len() != ecount()`, or when any
///   capacity is not strictly positive.
/// * [`IgraphError::VertexOutOfRange`] when `source` or `target` is not a
///   valid vertex id.
///
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
///
/// # Examples
///
/// ```
/// use rust_igraph::all_st_mincuts;
/// use rust_igraph::Graph;
///
/// // Path 0→1→2→3→4: the single min cut value is 1 and there are four
/// // distinct minimum cuts, one per edge.
/// let mut g = Graph::new(5, true).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let res = all_st_mincuts(&g, 0, 4, None).unwrap();
/// assert_eq!(res.value, 1.0);
/// assert_eq!(res.cuts.len(), 4);
/// ```
pub fn all_st_mincuts(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    capacity: Option<&[f64]>,
) -> IgraphResult<StMinCuts> {
    let n = graph.vcount();
    let m = graph.ecount();

    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "all_st_mincuts: listing s-t cuts is only implemented for directed graphs".to_string(),
        ));
    }
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    if source == target {
        return Err(IgraphError::InvalidArgument(
            "all_st_mincuts: source and target must differ".to_string(),
        ));
    }
    if let Some(cap) = capacity {
        if cap.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "all_st_mincuts: capacity length {} != edge count {m}",
                cap.len()
            )));
        }
        if cap.iter().any(|&c| !c.is_finite() || c <= 0.0) {
            return Err(IgraphError::InvalidArgument(
                "all_st_mincuts: all capacities must be strictly positive".to_string(),
            ));
        }
    }

    // 1. Maximum flow.
    let mf = max_flow(graph, source, target, capacity)?;
    let value = mf.value;
    let flow = mf.flow;

    // 2. Reverse residual graph.
    let residual = reverse_residual_graph(graph, capacity, &flow)?;

    // 3. Contract strongly connected components, simplify, relabel into
    //    topological order.
    let scc = strongly_connected_components(&residual)?;
    let mut ntol = scc.membership; // original vertex -> contracted (SCC) id
    let residual = contract_vertices(&residual, &ntol)?;
    let residual = simplify(&residual, true, true)?;

    let n_res = residual.vcount() as usize;
    let order = topological_sorting(&residual, DijkstraMode::Out)?; // order[pos] = old id
    let mut inv_order = vec![0u32; n_res];
    for (pos, &v) in order.iter().enumerate() {
        inv_order[v as usize] = pos as u32;
    }
    // Relabel the projection map so contracted ids follow topological order.
    for slot in &mut ntol {
        *slot = inv_order[*slot as usize];
    }
    // `permute_vertices(perm)[new] = old`, so feeding `order` makes new id
    // `pos` correspond to old id `order[pos]` — exactly the topological
    // numbering captured in `inv_order`.
    let residual = permute_vertices(&residual, &order)?;

    let newsource = ntol[source as usize];
    let newtarget = ntol[target as usize];

    // 4. Active projected vertices: endpoints of a positive-flow edge.
    let mut active = vec![false; n_res];
    for (e, &f) in flow.iter().enumerate() {
        if f > 0.0 {
            let (from, to) = graph.edge(e as EdgeId)?;
            active[ntol[from as usize] as usize] = true;
            active[ntol[to as usize] as usize] = true;
        }
    }

    // 5. Enumerate closed sets via the shared Provan-Shier recursion.
    let mut pivot = MincutsPivot::new(&residual, active)?;
    let closedsets = provan_shier_list(n_res, newsource, newtarget, &mut pivot)?;

    // 6a. Expand each closed set (contracted ids) back to original vertices.
    let mut revmap: Vec<Vec<VertexId>> = vec![Vec::new(); n_res];
    for i in 0..n as usize {
        revmap[ntol[i] as usize].push(i as VertexId);
    }

    let mut partition1s: Vec<Vec<VertexId>> = Vec::with_capacity(closedsets.len());
    for supercut in &closedsets {
        let mut part: Vec<VertexId> = Vec::new();
        for &vtx in supercut {
            part.extend_from_slice(&revmap[vtx as usize]);
        }
        part.sort_unstable();
        partition1s.push(part);
    }

    // 6b. The cut edges are the positive-flow edges leaving each partition.
    let mut cuts: Vec<Vec<EdgeId>> = Vec::with_capacity(partition1s.len());
    let mut in_s = vec![false; n as usize];
    for part in &partition1s {
        for &v in part {
            in_s[v as usize] = true;
        }
        let mut cut: Vec<EdgeId> = Vec::new();
        for (e, &f) in flow.iter().enumerate() {
            if f > 0.0 {
                let (from, to) = graph.edge(e as EdgeId)?;
                if in_s[from as usize] && !in_s[to as usize] {
                    cut.push(e as EdgeId);
                }
            }
        }
        cut.sort_unstable();
        cuts.push(cut);
        for &v in part {
            in_s[v as usize] = false;
        }
    }

    Ok(StMinCuts {
        value,
        cuts,
        partition1s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "actual = {actual}, expected = {expected}"
        );
    }

    fn build(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, true).expect("directed graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("add edge");
        }
        g
    }

    /// A canonicalised minimum cut: `(sorted partition, sorted endpoint pairs)`.
    type CanonCut = (Vec<u32>, Vec<(u32, u32)>);
    /// `(partition slice, cut-endpoint slice)` literals for the expected set.
    type ExpectedPairs<'a> = [(&'a [u32], &'a [(u32, u32)])];

    /// Canonical set of `(sorted partition, sorted cut-endpoint-pairs)`.
    /// The set of minimum cuts is unique, so this comparison is independent
    /// of enumeration order and internal vertex relabelling.
    fn cut_set(g: &Graph, res: &StMinCuts) -> HashSet<CanonCut> {
        let mut out = HashSet::new();
        for (part, cut) in res.partition1s.iter().zip(res.cuts.iter()) {
            let mut endpoints: Vec<(u32, u32)> =
                cut.iter().map(|&e| g.edge(e).expect("edge")).collect();
            endpoints.sort_unstable();
            out.insert((part.clone(), endpoints));
        }
        out
    }

    fn expected_set(pairs: &ExpectedPairs) -> HashSet<CanonCut> {
        pairs
            .iter()
            .map(|(part, cut)| {
                let mut p = part.to_vec();
                p.sort_unstable();
                let mut c = cut.to_vec();
                c.sort_unstable();
                (p, c)
            })
            .collect()
    }

    /// Every cut must (a) have total positive-flow weight == value and
    /// (b) actually disconnect target from source.
    fn assert_consistent(g: &Graph, source: VertexId, target: VertexId, res: &StMinCuts) {
        assert_eq!(res.cuts.len(), res.partition1s.len());
        let n = g.vcount() as usize;
        for (part, cut) in res.partition1s.iter().zip(res.cuts.iter()) {
            assert!(part.contains(&source), "source missing from {part:?}");
            assert!(!part.contains(&target), "target present in {part:?}");
            // Removing the cut disconnects target from source.
            let removed: HashSet<EdgeId> = cut.iter().copied().collect();
            let mut adj = vec![Vec::new(); n];
            for e in 0..g.ecount() as EdgeId {
                if removed.contains(&e) {
                    continue;
                }
                let (from, to) = g.edge(e).expect("edge");
                adj[from as usize].push(to);
            }
            let mut visited = vec![false; n];
            let mut stack = vec![source];
            visited[source as usize] = true;
            while let Some(u) = stack.pop() {
                for &w in &adj[u as usize] {
                    if !visited[w as usize] {
                        visited[w as usize] = true;
                        stack.push(w);
                    }
                }
            }
            assert!(
                !visited[target as usize],
                "target reachable after removing cut {cut:?}"
            );
        }
    }

    #[test]
    fn graph1_path_four_cuts() {
        let g = build(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
        let res = all_st_mincuts(&g, 0, 4, None).expect("compute");
        assert_close(res.value, 1.0);
        let expected = expected_set(&[
            (&[0], &[(0, 1)]),
            (&[0, 1], &[(1, 2)]),
            (&[0, 1, 2], &[(2, 3)]),
            (&[0, 1, 2, 3], &[(3, 4)]),
        ]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 0, 4, &res);
    }

    #[test]
    fn graph2_diamond_two_cuts() {
        let g = build(6, &[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4), (4, 5)]);
        let res = all_st_mincuts(&g, 0, 5, None).expect("compute");
        assert_close(res.value, 1.0);
        let expected = expected_set(&[(&[0], &[(0, 1)]), (&[0, 1, 2, 3, 4], &[(4, 5)])]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 0, 5, &res);
    }

    #[test]
    fn graph3_single_cut() {
        let g = build(6, &[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4), (4, 5)]);
        let res = all_st_mincuts(&g, 0, 4, None).expect("compute");
        assert_close(res.value, 1.0);
        let expected = expected_set(&[(&[0], &[(0, 1)])]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 0, 4, &res);
    }

    #[test]
    fn graph4_parallel_middle_three_cuts() {
        let g = build(
            9,
            &[
                (0, 1),
                (0, 2),
                (1, 3),
                (2, 3),
                (1, 4),
                (4, 2),
                (1, 5),
                (5, 2),
                (1, 6),
                (6, 2),
                (1, 7),
                (7, 2),
                (1, 8),
                (8, 2),
            ],
        );
        let res = all_st_mincuts(&g, 0, 3, None).expect("compute");
        assert_close(res.value, 2.0);
        let expected = expected_set(&[
            (&[0], &[(0, 1), (0, 2)]),
            (&[0, 2], &[(0, 1), (2, 3)]),
            (&[0, 1, 2, 4, 5, 6, 7, 8], &[(1, 3), (2, 3)]),
        ]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 0, 3, &res);
    }

    #[test]
    fn graph5_reverse_two_cuts() {
        let g = build(4, &[(1, 0), (2, 0), (2, 1), (3, 2)]);
        let res = all_st_mincuts(&g, 2, 0, None).expect("compute");
        assert_close(res.value, 2.0);
        let expected = expected_set(&[(&[2], &[(2, 0), (2, 1)]), (&[2, 1], &[(1, 0), (2, 0)])]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 2, 0, &res);
    }

    #[test]
    fn graph7_chain_five_cuts() {
        let g = build(
            9,
            &[
                (0, 4),
                (0, 7),
                (1, 6),
                (2, 1),
                (3, 8),
                (4, 0),
                (4, 2),
                (4, 5),
                (5, 0),
                (5, 3),
                (6, 7),
                (7, 8),
            ],
        );
        let res = all_st_mincuts(&g, 0, 8, None).expect("compute");
        assert_close(res.value, 2.0);
        let expected = expected_set(&[
            (&[0], &[(0, 4), (0, 7)]),
            (&[0, 7], &[(0, 4), (7, 8)]),
            (&[0, 7, 4, 2, 1, 6], &[(4, 5), (7, 8)]),
            (&[0, 7, 4, 2, 1, 6, 5], &[(5, 3), (7, 8)]),
            (&[0, 7, 4, 2, 1, 6, 5, 3], &[(3, 8), (7, 8)]),
        ]);
        assert_eq!(cut_set(&g, &res), expected);
        assert_consistent(&g, 0, 8, &res);
    }

    #[test]
    fn rejects_undirected_graph() {
        let g = Graph::with_vertices(4);
        let err = all_st_mincuts(&g, 0, 3, None).expect_err("must reject undirected");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_equal_source_and_target() {
        let g = build(3, &[(0, 1), (1, 2)]);
        let err = all_st_mincuts(&g, 1, 1, None).expect_err("must reject s == t");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_out_of_range_endpoints() {
        let g = build(3, &[(0, 1), (1, 2)]);
        match all_st_mincuts(&g, 5, 1, None).expect_err("bad source") {
            IgraphError::VertexOutOfRange { id, n } => assert_eq!((id, n), (5, 3)),
            other => panic!("unexpected error: {other:?}"),
        }
        match all_st_mincuts(&g, 0, 9, None).expect_err("bad target") {
            IgraphError::VertexOutOfRange { id, n } => assert_eq!((id, n), (9, 3)),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_bad_capacity() {
        let g = build(3, &[(0, 1), (1, 2)]);
        // Wrong length.
        let err = all_st_mincuts(&g, 0, 2, Some(&[1.0])).expect_err("bad length");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // Non-positive capacity.
        let err = all_st_mincuts(&g, 0, 2, Some(&[1.0, 0.0])).expect_err("non-positive");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }
}
