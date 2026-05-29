//! `all_st_cuts` (ALGO-FL-031) — list every (s,t) edge cut of a directed
//! graph.
//!
//! Counterpart of `igraph_all_st_cuts` from
//! `references/igraph/src/flow/st-cuts.c:1030`.
//!
//! Implements the Provan-Shier paradigm for listing (s,t)-cuts
//! (J. S. Provan and D. R. Shier, *A paradigm for listing (s,t)-cuts in
//! graphs*, Algorithmica 15:351–372, 1996). Every distinct edge cut
//! between `source` and `target` is enumerated exactly once.
//!
//! The enumeration walks a binary search tree over *closed* vertex sets
//! `S` (the source side of a cut). At each node a **pivot** element `v`
//! and its *closure increment* `I(S,v)` are computed using the dominator
//! tree of the residual `Sbar = V \ S` subgraph (rooted at `target`,
//! post-dominator orientation). Recursing "right" adds `I(S,v)` to `S`;
//! recursing "left" forbids `v` (pushes it on `T`). A leaf with a proper,
//! non-trivial `S` yields one cut.
//!
//! Only the OUT-neighbour adjacency of the original graph is needed; the
//! dominator computation and induced-subgraph mapping are delegated to
//! the existing [`dominator_tree`] and [`induced_subgraph`] AWUs.

// Signed sentinels from `dominator_tree` (`idom`: -1 root, -2 unreachable)
// are walked as `i32`; the induced-subgraph maps use `u32::MAX` as the
// "absent" sentinel. All casts are bounded by `vcount` which the dominator
// AWU already constrains to `i32::MAX`.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use crate::algorithms::flow::dominator_tree::{DominatorMode, DominatorTree, dominator_tree};
use crate::algorithms::operators::induced_subgraph::{InducedSubgraphResult, induced_subgraph};
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`all_st_cuts`].
///
/// `cuts[i]` and `partition1s[i]` describe the same cut: `partition1s[i]`
/// is the source-side vertex set `X` (always contains `source`, never
/// `target`), and `cuts[i]` lists the ids of the edges leaving `X`
/// (`from ∈ X`, `to ∉ X`). Vertices within each partition and edge ids
/// within each cut are sorted ascending.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StCuts {
    /// Edge-id list of each cut, sorted ascending.
    pub cuts: Vec<Vec<EdgeId>>,
    /// Source-side vertex set generating each cut, sorted ascending.
    pub partition1s: Vec<Vec<VertexId>>,
}

/// A batch stack with O(1) membership test (`igraph_marked_queue_int_t`).
///
/// Elements are pushed individually but removed a whole batch at a time.
/// `start_batch` records a batch boundary; `pop_back_batch` rewinds to the
/// most recent boundary, clearing membership for everything popped.
struct MarkedQueue {
    queue: Vec<i64>,
    member: Vec<bool>,
    size: usize,
}

impl MarkedQueue {
    fn new(n: usize) -> Self {
        Self {
            queue: Vec::new(),
            member: vec![false; n],
            size: 0,
        }
    }

    fn iselement(&self, e: VertexId) -> bool {
        self.member[e as usize]
    }

    fn size(&self) -> usize {
        self.size
    }

    fn push(&mut self, e: VertexId) {
        if !self.member[e as usize] {
            self.queue.push(i64::from(e));
            self.member[e as usize] = true;
            self.size += 1;
        }
    }

    fn start_batch(&mut self) {
        self.queue.push(-1);
    }

    fn pop_back_batch(&mut self) {
        while let Some(top) = self.queue.pop() {
            if top == -1 {
                break;
            }
            self.member[top as usize] = false;
            self.size -= 1;
        }
    }

    /// Snapshot of the live elements in insertion order (markers skipped).
    fn as_vector(&self) -> Vec<VertexId> {
        self.queue
            .iter()
            .filter(|&&e| e != -1)
            .map(|&e| e as VertexId)
            .collect()
    }
}

/// A stack with O(1) membership test (`igraph_estack_t`).
struct EStack {
    stack: Vec<VertexId>,
    isin: Vec<bool>,
}

impl EStack {
    fn new(n: usize) -> Self {
        Self {
            stack: Vec::new(),
            isin: vec![false; n],
        }
    }

    fn iselement(&self, e: VertexId) -> bool {
        self.isin[e as usize]
    }

    fn push(&mut self, e: VertexId) {
        if !self.isin[e as usize] {
            self.stack.push(e);
            self.isin[e as usize] = true;
        }
    }

    fn pop(&mut self) {
        if let Some(e) = self.stack.pop() {
            self.isin[e as usize] = false;
        }
    }
}

/// List all (s,t) edge cuts of a directed graph (Provan-Shier).
///
/// # Arguments
///
/// * `graph`  — directed input graph. Undirected graphs are rejected.
/// * `source` — source vertex id.
/// * `target` — target vertex id (must differ from `source`).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] when `graph` is undirected or when
///   `source == target`.
/// * [`IgraphError::VertexOutOfRange`] when `source` or `target` is not a
///   valid vertex id.
///
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
///
/// # Examples
///
/// ```
/// use rust_igraph::all_st_cuts;
/// use rust_igraph::Graph;
///
/// // Two parallel routes 0 ⇒ 2: 0→1→2 and 0→2 (here a single path 0→1→2).
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let res = all_st_cuts(&g, 0, 2).unwrap();
/// // Two cuts: {0} (edge 0) and {0,1} (edge 1).
/// assert_eq!(res.partition1s, vec![vec![0], vec![0, 1]]);
/// assert_eq!(res.cuts, vec![vec![0], vec![1]]);
/// ```
pub fn all_st_cuts(graph: &Graph, source: VertexId, target: VertexId) -> IgraphResult<StCuts> {
    let n = graph.vcount();

    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "all_st_cuts: listing all s-t cuts is only implemented for directed graphs".to_string(),
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
            "all_st_cuts: source and target must differ".to_string(),
        ));
    }

    let n_us = n as usize;
    let mut s = MarkedQueue::new(n_us);
    let mut t = EStack::new(n_us);
    let mut partitions: Vec<Vec<VertexId>> = Vec::new();

    provan_shier(graph, &mut s, &mut t, source, target, &mut partitions)?;

    // The recursion appends partitions leaf-first; igraph reverses to keep
    // the historical ordering. We canonicalise each partition (sorted) and
    // derive the matching edge cut.
    partitions.reverse();

    let mut cuts: Vec<Vec<EdgeId>> = Vec::with_capacity(partitions.len());
    let mut in_s = vec![false; n_us];
    for part in &mut partitions {
        part.sort_unstable();
        for &v in part.iter() {
            in_s[v as usize] = true;
        }
        let mut cut: Vec<EdgeId> = Vec::new();
        for e in 0..graph.ecount() {
            let (from, to) = graph.edge(e as EdgeId)?;
            if in_s[from as usize] && !in_s[to as usize] {
                cut.push(e as EdgeId);
            }
        }
        cut.sort_unstable();
        cuts.push(cut);
        for &v in part.iter() {
            in_s[v as usize] = false;
        }
    }

    Ok(StCuts {
        cuts,
        partition1s: partitions,
    })
}

/// The recursive Provan-Shier enumeration
/// (`igraph_i_provan_shier_list_recursive`).
fn provan_shier(
    graph: &Graph,
    s: &mut MarkedQueue,
    t: &mut EStack,
    source: VertexId,
    target: VertexId,
    result: &mut Vec<Vec<VertexId>>,
) -> IgraphResult<()> {
    let n = graph.vcount() as usize;
    let (v, isv) = pivot(graph, s, t, source, target)?;

    if isv.is_empty() {
        let sz = s.size();
        if sz != 0 && sz != n {
            result.push(s.as_vector());
        }
        return Ok(());
    }

    // Down-right: add I(S,v) to S.
    s.start_batch();
    for &x in &isv {
        s.push(x);
    }
    provan_shier(graph, s, t, source, target, result)?;
    s.pop_back_batch();

    // Down-left: forbid v.
    t.push(v);
    provan_shier(graph, s, t, source, target, result)?;
    t.pop();

    Ok(())
}

/// The pivot function (`igraph_i_all_st_cuts_pivot`).
///
/// Returns `(v, I(S,v))`. When `I(S,v)` is empty `v` is meaningless and the
/// caller treats `S` as a leaf.
// `s`/`t`/`n`/`m`/`v` mirror the Provan-Shier paper notation (set S, set T,
// |V|, minimal element set M, pivot v) and the igraph C source one-to-one.
#[allow(clippy::many_single_char_names)]
fn pivot(
    graph: &Graph,
    s: &MarkedQueue,
    t: &EStack,
    source: VertexId,
    target: VertexId,
) -> IgraphResult<(VertexId, Vec<VertexId>)> {
    let n = graph.vcount() as usize;

    // Sbar = V \ S, with old<->new id maps.
    let mut keep: Vec<VertexId> = Vec::new();
    for i in 0..n {
        if !s.iselement(i as VertexId) {
            keep.push(i as VertexId);
        }
    }
    let sbar: InducedSubgraphResult = induced_subgraph(graph, &keep)?;
    let root = sbar.map[target as usize];

    // Post-dominator tree of Sbar rooted at target.
    let dt = dominator_tree(&sbar.graph, root, DominatorMode::In)?;

    // Gamma(S): the OUT-boundary of S (old ids). When S is empty the
    // induced map is the identity and Gamma(S) = {source}.
    let mut gamma_s = vec![false; n];
    if s.size() == 0 {
        gamma_s[source as usize] = true;
    } else {
        for i in 0..n {
            if s.iselement(i as VertexId) {
                for nei in graph.out_neighbors_vec(i as VertexId)? {
                    if !s.iselement(nei) {
                        gamma_s[nei as usize] = true;
                    }
                }
            }
        }
    }

    // K = vertices left out of the dominator tree (cannot reach target).
    // Relabel to old ids and ensure Gamma(S) ⊆ L (the dominator-tree nodes).
    let mut leftout_old: Vec<VertexId> = Vec::with_capacity(dt.leftout.len());
    for &lo in &dt.leftout {
        let old = sbar.invmap[lo as usize];
        leftout_old.push(old);
        gamma_s[old as usize] = false;
    }

    // M = minimal elements of Gamma(S) under the dominator relation.
    let m = if dt.tree.ecount() > 0 {
        minimal_gamma(&dt, &sbar, &gamma_s, n)
    } else {
        Vec::new()
    };

    if m.is_empty() {
        return Ok((0, Vec::new()));
    }

    // Children adjacency of the dominator tree (new-id space).
    let sbar_n = sbar.graph.vcount() as usize;
    let mut children: Vec<Vec<u32>> = vec![Vec::new(); sbar_n];
    for v in 0..sbar_n {
        let p = dt.idom[v];
        if p >= 0 {
            children[p as usize].push(v as u32);
        }
    }

    let gamma_s_vec: Vec<VertexId> = (0..n)
        .filter(|&i| gamma_s[i])
        .map(|i| i as VertexId)
        .collect();

    for &m_old in &m {
        let min_new = sbar.map[m_old as usize];
        // Nu(v) = subtree of the dominator tree rooted at v (old ids).
        let mut nuv: Vec<VertexId> = dom_subtree(&children, min_new)
            .into_iter()
            .map(|x| sbar.invmap[x as usize])
            .collect();

        // I(S,v) − K: vertices of Nu(v) reachable from Gamma(S) within Nu(v).
        let isv_min = restricted_bfs(graph, &gamma_s_vec, &nuv, n)?;

        // Accept v if none of those vertices is forbidden (in T) or is the
        // target.
        let blocked = isv_min.iter().any(|&u| t.iselement(u) || u == target);
        if !blocked {
            // Real I(S,v): BFS from v restricted to Nu(v) ∪ K.
            nuv.extend_from_slice(&leftout_old);
            let isv = restricted_bfs(graph, &[m_old], &nuv, n)?;
            return Ok((m_old, isv));
        }
    }

    Ok((0, Vec::new()))
}

/// Minimal elements of `Gamma(S)` w.r.t. the dominator relation
/// (`igraph_i_all_st_cuts_minimal`). A Gamma(S) vertex is minimal iff it
/// has no proper Gamma(S) descendant in the dominator tree.
fn minimal_gamma(
    dt: &DominatorTree,
    sbar: &InducedSubgraphResult,
    gamma_s: &[bool],
    n: usize,
) -> Vec<VertexId> {
    // nomark[i] = true ⇒ vertex i is not minimal. Start with every non-Γ
    // vertex marked, then mark any Γ ancestor that dominates another Γ
    // vertex.
    let mut nomark: Vec<bool> = (0..n).map(|i| !gamma_s[i]).collect();

    for g_old in 0..n {
        if !gamma_s[g_old] {
            continue;
        }
        let gn = sbar.map[g_old];
        if gn == u32::MAX {
            continue;
        }
        let mut cur = dt.idom[gn as usize];
        while cur >= 0 {
            let anc_old = sbar.invmap[cur as usize] as usize;
            if gamma_s[anc_old] {
                nomark[anc_old] = true;
            }
            cur = dt.idom[cur as usize];
        }
    }

    (0..n)
        .filter(|&i| !nomark[i])
        .map(|i| i as VertexId)
        .collect()
}

/// Vertices of the dominator-tree subtree rooted at `root` (new ids),
/// including `root` itself.
fn dom_subtree(children: &[Vec<u32>], root: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        out.push(v);
        for &c in &children[v as usize] {
            stack.push(c);
        }
    }
    out
}

/// Multi-root OUT-direction BFS over `graph`, visiting only vertices in the
/// `restricted` set (`igraph_bfs` with `roots` + `restricted`). Roots
/// outside the restricted set are skipped. Returns the visit order.
fn restricted_bfs(
    graph: &Graph,
    roots: &[VertexId],
    restricted: &[VertexId],
    n: usize,
) -> IgraphResult<Vec<VertexId>> {
    let mut allowed = vec![false; n];
    for &r in restricted {
        allowed[r as usize] = true;
    }
    let mut visited = vec![false; n];
    let mut order: Vec<VertexId> = Vec::new();
    let mut queue: std::collections::VecDeque<VertexId> = std::collections::VecDeque::new();

    for &root in roots {
        if !allowed[root as usize] || visited[root as usize] {
            continue;
        }
        visited[root as usize] = true;
        order.push(root);
        queue.push_back(root);
        while let Some(u) = queue.pop_front() {
            for nei in graph.out_neighbors_vec(u)? {
                if allowed[nei as usize] && !visited[nei as usize] {
                    visited[nei as usize] = true;
                    order.push(nei);
                    queue.push_back(nei);
                }
            }
        }
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a directed graph from an explicit `(from, to)` edge list.
    /// Edge ids follow insertion order.
    fn build(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, true).expect("directed graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("add edge");
        }
        g
    }

    /// Can `target` still be reached from `source` after deleting the edge
    /// ids in `cut`? Forward (OUT) BFS over the original graph minus the
    /// cut edges.
    fn target_reachable_minus_cut(
        g: &Graph,
        source: VertexId,
        target: VertexId,
        cut: &[EdgeId],
    ) -> bool {
        let n = g.vcount() as usize;
        let removed: std::collections::HashSet<EdgeId> = cut.iter().copied().collect();
        let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
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
            if u == target {
                return true;
            }
            for &w in &adj[u as usize] {
                if !visited[w as usize] {
                    visited[w as usize] = true;
                    stack.push(w);
                }
            }
        }
        false
    }

    /// Assert every structural invariant that `all_st_cuts` must satisfy
    /// for a given `source`/`target`, independent of the exact cut list.
    fn assert_cuts_consistent(g: &Graph, source: VertexId, target: VertexId, res: &StCuts) {
        assert_eq!(
            res.cuts.len(),
            res.partition1s.len(),
            "cuts and partition1s must have equal length"
        );
        let n = g.vcount() as usize;
        let mut seen: std::collections::HashSet<Vec<VertexId>> = std::collections::HashSet::new();
        for (part, cut) in res.partition1s.iter().zip(res.cuts.iter()) {
            // Strict ascending order.
            for w in part.windows(2) {
                assert!(w[0] < w[1], "partition not strictly ascending: {part:?}");
            }
            for w in cut.windows(2) {
                assert!(w[0] < w[1], "cut not strictly ascending: {cut:?}");
            }
            // Source/target membership.
            assert!(
                part.contains(&source),
                "source {source} missing from {part:?}"
            );
            assert!(
                !part.contains(&target),
                "target {target} present in {part:?}"
            );
            // Uniqueness of source-side sets.
            assert!(seen.insert(part.clone()), "duplicate partition {part:?}");

            let mut in_s = vec![false; n];
            for &v in part {
                in_s[v as usize] = true;
            }
            // Completeness: cut equals exactly the crossing-out edges.
            let mut crossing: Vec<EdgeId> = Vec::new();
            for e in 0..g.ecount() as EdgeId {
                let (from, to) = g.edge(e).expect("edge");
                if in_s[from as usize] && !in_s[to as usize] {
                    crossing.push(e);
                }
            }
            crossing.sort_unstable();
            assert_eq!(cut, &crossing, "cut != crossing-out edges of {part:?}");
            // Deleting the cut must disconnect target from source.
            assert!(
                !target_reachable_minus_cut(g, source, target, cut),
                "target still reachable after removing cut {cut:?} (partition {part:?})"
            );
        }
    }

    /// The verified golden fixture from the igraph C reference output.
    #[test]
    fn golden_six_node_nine_cuts() {
        let g = build(6, &[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4), (1, 5), (5, 4)]);
        let res = all_st_cuts(&g, 0, 4).expect("compute");

        let expected = StCuts {
            partition1s: vec![
                vec![0],
                vec![0, 1],
                vec![0, 1, 5],
                vec![0, 1, 3],
                vec![0, 1, 3, 5],
                vec![0, 1, 2],
                vec![0, 1, 2, 5],
                vec![0, 1, 2, 3],
                vec![0, 1, 2, 3, 5],
            ],
            cuts: vec![
                vec![0],
                vec![1, 2, 5],
                vec![1, 2, 6],
                vec![1, 4, 5],
                vec![1, 4, 6],
                vec![2, 3, 5],
                vec![2, 3, 6],
                vec![3, 4, 5],
                vec![3, 4, 6],
            ],
        };
        assert_eq!(res, expected, "golden fixture mismatch");
        assert_cuts_consistent(&g, 0, 4, &res);
    }

    /// Simple path 0 → 1 → 2: exactly two cuts.
    #[test]
    fn simple_path_two_cuts() {
        let g = build(3, &[(0, 1), (1, 2)]);
        let res = all_st_cuts(&g, 0, 2).expect("compute");
        assert_eq!(res.partition1s, vec![vec![0], vec![0, 1]]);
        assert_eq!(res.cuts, vec![vec![0], vec![1]]);
        assert_cuts_consistent(&g, 0, 2, &res);
    }

    /// Complete directed graph K5: every subset `X` with `s ∈ X`, `t ∉ X`
    /// is a valid distinct source side, so there are `2^(n-2) = 8` cuts.
    #[test]
    fn complete_digraph_k5_has_eight_cuts() {
        let mut edges = Vec::new();
        for u in 0..5 {
            for v in 0..5 {
                if u != v {
                    edges.push((u, v));
                }
            }
        }
        let g = build(5, &edges);
        let res = all_st_cuts(&g, 0, 4).expect("compute");
        assert_eq!(res.cuts.len(), 8, "K5 should yield 2^(5-2) = 8 cuts");
        assert_cuts_consistent(&g, 0, 4, &res);
    }

    /// Empty graph (n = 0): any source id is out of range.
    #[test]
    fn empty_graph_source_out_of_range() {
        let g = Graph::new(0, true).expect("empty directed graph");
        let err = all_st_cuts(&g, 0, 1).expect_err("must reject empty graph");
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 0);
                assert_eq!(n, 0);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    /// Single vertex (n = 1): the only in-range call has source == target,
    /// which is rejected; there is no valid (s, t) pair.
    #[test]
    fn single_vertex_rejects_equal_endpoints() {
        let g = Graph::new(1, true).expect("single vertex directed graph");
        let err = all_st_cuts(&g, 0, 0).expect_err("must reject source == target");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // The only other id (target = 1) is out of range.
        let err2 = all_st_cuts(&g, 0, 1).expect_err("must reject out-of-range target");
        assert!(matches!(
            err2,
            IgraphError::VertexOutOfRange { id: 1, n: 1 }
        ));
    }

    /// Undirected graphs are rejected.
    #[test]
    fn rejects_undirected_graph() {
        let g = Graph::with_vertices(4);
        let err = all_st_cuts(&g, 0, 3).expect_err("must reject undirected");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    /// `source == target` is rejected.
    #[test]
    fn rejects_equal_source_and_target() {
        let g = build(3, &[(0, 1), (1, 2)]);
        let err = all_st_cuts(&g, 1, 1).expect_err("must reject source == target");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    /// Out-of-range source and target are rejected with the right id/n.
    #[test]
    fn rejects_out_of_range_endpoints() {
        let g = build(3, &[(0, 1), (1, 2)]);
        match all_st_cuts(&g, 5, 1).expect_err("bad source") {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!((id, n), (5, 3));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        match all_st_cuts(&g, 0, 9).expect_err("bad target") {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!((id, n), (9, 3));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    /// Source cannot reach target at all. The algorithm produces no proper
    /// non-trivial source side here (the recursion bottoms out with an empty
    /// `S`), so the result is the empty cut list. Whatever it returns must
    /// be internally consistent.
    #[test]
    fn unreachable_target_is_consistent() {
        // Edge (1, 2) only; vertex 0 has no outgoing edges.
        let g = build(3, &[(1, 2)]);
        let res = all_st_cuts(&g, 0, 2).expect("compute");
        // Observed behaviour: no cuts are enumerated when the target is
        // unreachable from the source.
        assert!(res.cuts.is_empty(), "expected empty cut list, got {res:?}");
        // Consistency holds vacuously, but keep the check as a guard.
        assert_cuts_consistent(&g, 0, 2, &res);
    }
}
