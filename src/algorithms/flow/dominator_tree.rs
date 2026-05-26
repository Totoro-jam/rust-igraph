//! `dominator_tree` (ALGO-FL-030) — Lengauer-Tarjan dominator tree of a
//! directed flowgraph.
//!
//! Counterpart of `igraph_dominator_tree` from
//! `references/igraph/src/flow/st-cuts.c:434-609` (with helpers at
//! lines 286-375: the `dbucket` linked-list, and the `LINK` /
//! `COMPRESS` / `EVAL` path-compression primitives).
//!
//! A *flowgraph* is a directed graph with a distinguished root `r`
//! such that every vertex is conceptually reachable from `r`; in
//! practice we tolerate unreachable vertices and surface them in the
//! `leftout` list. A vertex `v` *dominates* `w` if every path from `r`
//! to `w` passes through `v`; the *immediate dominator* `idom(w)` is
//! the dominator closest to `w`. The edges `{(idom(w), w) | w ≠ r}`
//! form a directed tree rooted at `r`, the dominator tree.
//!
//! Reference: Thomas Lengauer & Robert E. Tarjan, *A fast algorithm
//! for finding dominators in a flowgraph*, ACM TOPLAS 1(1):121-141,
//! 1979. <https://doi.org/10.1145/357062.357071>
//!
//! Complexity: `O(|V| + |E| · α(|E|, |V|))` with path-compression
//! `EVAL`, where `α` is the inverse Ackermann function — effectively
//! linear in `|V| + |E|`.
//!
//! ## Differences from the C port
//!
//! * No `+1` sentinel scheme. Where the C code stores `x + 1` to use
//!   `0` as "unset", the Rust port uses signed `i32` with `-1` as the
//!   sentinel — the additional arithmetic and the +1/-1 dance in the
//!   C code are gone.
//! * The `igraph_adjlist_t` predecessor/successor lookups become two
//!   `Vec<Vec<u32>>` built once up front from
//!   [`Graph::out_neighbors_vec`] / [`Graph::in_neighbors_vec`]; the
//!   filter step that drops unreachable predecessors mirrors
//!   `st-cuts.c:522-535`.
//! * The DFS is inlined here (instead of delegating to
//!   `algorithms::traversal::dfs::dfs`) so we capture both the parent
//!   pointer and the DFS pre-order in a single pass.

// `i32 <-> usize` and `u32 <-> i32` casts are pervasive because we use
// signed sentinels (`-1` = root/unset, `-2` = unreachable). The
// validation at the top of `dominator_tree` rejects any `vcount` that
// would not round-trip through `i32`, so every cast in this module is
// bounded by `i32::MAX`. `too_many_lines` is allowed for the main
// Lengauer-Tarjan kernel which mirrors the upstream C source.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Traversal direction for [`dominator_tree`].
///
/// Mirrors the `IGRAPH_OUT` / `IGRAPH_IN` choice of igraph C's
/// `igraph_dominator_tree`. `IGRAPH_ALL` is not a valid mode for the
/// dominator-tree problem and is therefore excluded from this enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DominatorMode {
    /// Follow out-edges from `root`. This is the standard control-flow
    /// dominator tree: an edge `u → v` in the input means `u` precedes
    /// `v` in the flow.
    Out,
    /// Treat in-edges as forward edges (i.e. reverse every edge before
    /// computing). Used to compute the *post-dominator* tree of a
    /// control-flow graph and to mirror the upstream `IGRAPH_IN` mode.
    In,
}

/// Result of [`dominator_tree`].
#[derive(Clone, Debug)]
pub struct DominatorTree {
    /// Immediate-dominator vector of length `vcount`.
    ///
    /// * `idom[root] == -1`,
    /// * `idom[v] == -2` for every vertex `v` unreachable from `root`,
    /// * `idom[v] == idom_vertex_id` otherwise (always `>= 0` and
    ///   `< vcount`).
    pub idom: Vec<i32>,
    /// Directed graph on the same vertex set as the input. Contains
    /// exactly one edge per non-root reachable vertex; orientation is
    /// dictated by `mode` (`Out` ⇒ `idom(w) → w`, `In` ⇒ `w → idom(w)`).
    /// Vertices in [`Self::leftout`] appear as isolates.
    pub tree: Graph,
    /// Vertex ids unreachable from `root`, in ascending order. Empty
    /// when the graph is a proper flowgraph (every vertex reachable
    /// from `root`).
    pub leftout: Vec<VertexId>,
}

/// Compute the Lengauer-Tarjan dominator tree of a directed flowgraph.
///
/// See [`DominatorTree`] for the result shape and [`DominatorMode`] for
/// the direction selector.
///
/// # Arguments
///
/// * `graph`  — directed input graph. Undirected graphs are rejected.
/// * `root`   — root vertex id; must satisfy `root < graph.vcount()`.
/// * `mode`   — `Out` for the classical control-flow dominator tree,
///   `In` for the post-dominator tree (every edge reversed).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] when `graph` is undirected or
///   when `graph.vcount()` exceeds `i32::MAX` (which would make the
///   internal DFS-order index overflow).
/// * [`IgraphError::VertexOutOfRange`] when `root` is not a valid
///   vertex id.
///
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dominator_tree, DominatorMode};
///
/// // 0 → 1: the root dominates every other reachable vertex trivially.
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// let dt = dominator_tree(&g, 0, DominatorMode::Out).unwrap();
/// assert_eq!(dt.idom, vec![-1, 0]);
/// assert_eq!(dt.leftout, Vec::<u32>::new());
/// assert_eq!(dt.tree.ecount(), 1);
/// ```
pub fn dominator_tree(
    graph: &Graph,
    root: VertexId,
    mode: DominatorMode,
) -> IgraphResult<DominatorTree> {
    let n = graph.vcount();

    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "dominator tree of an undirected graph requested".to_string(),
        ));
    }
    if root >= n {
        return Err(IgraphError::VertexOutOfRange { id: root, n });
    }
    // We use i32 sentinels (-1 = root / unset, -2 = unreachable) and store
    // vertex ids in i32 throughout; reject any input whose vcount cannot
    // round-trip through i32.
    if i32::try_from(n).is_err() {
        return Err(IgraphError::InvalidArgument(format!(
            "dominator_tree: vcount {n} exceeds i32::MAX"
        )));
    }

    let n_us = n as usize;

    // Build adjacency lists. `succ` is the forward direction (DFS walks
    // this); `pred` is the reverse (used in the bucket loop). For
    // mode == In every edge is conceptually reversed.
    let mut succ: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    let mut pred: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let (s, p) = match mode {
            DominatorMode::Out => (graph.out_neighbors_vec(v)?, graph.in_neighbors_vec(v)?),
            DominatorMode::In => (graph.in_neighbors_vec(v)?, graph.out_neighbors_vec(v)?),
        };
        succ.push(s);
        pred.push(p);
    }

    // ----- Step 1: DFS from `root`, set `parent` + DFS-order -----
    // parent[v] = -1 for root, -2 for unreachable, else the DFS parent
    // vertex id (always >= 0).
    let mut parent: Vec<i32> = vec![-2; n_us];
    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut visited = vec![false; n_us];

    parent[root as usize] = -1;
    visited[root as usize] = true;
    order.push(root);

    let mut stack: Vec<(VertexId, usize)> = Vec::with_capacity(n_us);
    stack.push((root, 0));

    while let Some(&(v, cursor)) = stack.last() {
        let mut next_cursor = cursor;
        let neis = &succ[v as usize];
        let mut found: Option<VertexId> = None;
        while next_cursor < neis.len() {
            let u = neis[next_cursor];
            next_cursor += 1;
            if !visited[u as usize] {
                found = Some(u);
                break;
            }
        }
        let top_idx = stack.len() - 1;
        if let Some(u) = found {
            stack[top_idx].1 = next_cursor;
            visited[u as usize] = true;
            // Safe because we rejected n > i32::MAX up front, and v < n.
            parent[u as usize] = v as i32;
            order.push(u);
            stack.push((u, 0));
        } else {
            stack.pop();
        }
    }

    // semi[v] = DFS number of v (0..component_size); -1 if unreachable.
    // vertex[i] = vertex id with DFS number i; only the first
    // `component_size` slots are valid.
    let component_size = order.len();
    let mut semi: Vec<i32> = vec![-1; n_us];
    let mut vertex: Vec<i32> = vec![-1; component_size];
    for (i, &v) in order.iter().enumerate() {
        // Both bounded by n <= i32::MAX, checked above.
        semi[v as usize] = i as i32;
        vertex[i] = v as i32;
    }

    // leftout: vertices unreachable from root, in ascending order.
    let mut leftout: Vec<VertexId> = Vec::with_capacity(n_us - component_size);
    for v in 0..n {
        if parent[v as usize] == -2 {
            leftout.push(v);
        }
    }

    // Drop predecessors that lie in the unreachable set — they cannot
    // contribute to any dominator path from `root` (mirrors
    // st-cuts.c:520-535).
    for plist in pred.iter_mut().take(n_us) {
        plist.retain(|&u| parent[u as usize] >= -1);
    }

    // ----- Steps 2 & 3: bucket-driven semi-dominator + LINK / EVAL -----
    // ancestor[v] = LINK-forest parent (-1 if v is its own tree root).
    // label[v]    = candidate minimum-semi vertex along the path.
    let mut ancestor: Vec<i32> = vec![-1; n_us];
    let mut label: Vec<i32> = (0..n_us).map(|i| i as i32).collect();

    // Per-vertex bucket: bucket_head[bid] = -1 (empty) or first elem;
    // bucket_next[elem] = -1 (last) or next elem id.
    let mut bucket_head: Vec<i32> = vec![-1; n_us];
    let mut bucket_next: Vec<i32> = vec![-1; n_us];

    // Working idom array (sentinel -2 = unset; root + unreachable stay
    // -2 until the tail of the function patches them).
    let mut idom: Vec<i32> = vec![-2; n_us];

    for i in (1..component_size).rev() {
        let w_i32 = vertex[i];
        let w_us = w_i32 as usize;

        // Compute semi-dominator of w via every reachable predecessor.
        let preds_len = pred[w_us].len();
        for j in 0..preds_len {
            let v_pred = pred[w_us][j] as i32;
            let u = dom_eval(v_pred, &mut ancestor, &mut label, &semi);
            let su = semi[u as usize];
            let sw = semi[w_us];
            if su >= 0 && (sw < 0 || su < sw) {
                semi[w_us] = su;
            }
        }

        // Insert w into bucket[ vertex[semi(w)] ].
        let semi_w = semi[w_us];
        let bucket_owner = vertex[semi_w as usize] as usize;
        bucket_next[w_us] = bucket_head[bucket_owner];
        bucket_head[bucket_owner] = w_i32;

        // LINK(parent[w], w) — ancestor[w] := parent[w].
        let pw = parent[w_us];
        ancestor[w_us] = pw;

        // Drain bucket[parent[w]], finalising each entry's idom.
        let pw_us = pw as usize;
        while bucket_head[pw_us] != -1 {
            let v_b = bucket_head[pw_us];
            bucket_head[pw_us] = bucket_next[v_b as usize];
            let u = dom_eval(v_b, &mut ancestor, &mut label, &semi);
            if semi[u as usize] < semi[v_b as usize] {
                idom[v_b as usize] = u;
            } else {
                idom[v_b as usize] = pw;
            }
        }
    }

    // ----- Step 4: forward fixup pass -----
    for i in 1..component_size {
        let w_us = vertex[i] as usize;
        let chk = vertex[semi[w_us] as usize];
        if idom[w_us] != chk {
            idom[w_us] = idom[idom[w_us] as usize];
        }
    }
    idom[root as usize] = -1;

    // ----- Materialise the dominator tree -----
    let mut tree = Graph::new(n, true)?;
    for w in 0..n {
        if w == root {
            continue;
        }
        let d = idom[w as usize];
        if d < 0 {
            // Unreachable vertex: appears as an isolate in the tree.
            continue;
        }
        match mode {
            DominatorMode::Out => tree.add_edge(d as VertexId, w)?,
            DominatorMode::In => tree.add_edge(w, d as VertexId)?,
        }
    }

    Ok(DominatorTree {
        idom,
        tree,
        leftout,
    })
}

/// `EVAL` primitive: finds the vertex on the LINK-forest path from `v`
/// to its tree root with the smallest `semi` value, performing path
/// compression along the way.
fn dom_eval(v: i32, ancestor: &mut [i32], label: &mut [i32], semi: &[i32]) -> i32 {
    if ancestor[v as usize] == -1 {
        v
    } else {
        dom_compress(v, ancestor, label, semi);
        label[v as usize]
    }
}

/// `COMPRESS` primitive: walks up the LINK-forest path from `v` to the
/// tree root (exclusive), propagating the minimum-`semi` `label`
/// downward and rewiring every visited `ancestor` to point directly at
/// the tree root.
fn dom_compress(v: i32, ancestor: &mut [i32], label: &mut [i32], semi: &[i32]) {
    // Collect the path v -> parent(v) -> ... -> last vertex whose
    // ancestor is still non-null. The tree root itself (ancestor == -1)
    // is NOT pushed.
    let mut path: Vec<i32> = Vec::new();
    let mut w = v;
    while ancestor[w as usize] != -1 {
        path.push(w);
        w = ancestor[w as usize];
    }
    // Process from the shallowest pushed (last) to the deepest (first),
    // matching the LIFO order of the C stack.
    let mut iter = path.into_iter().rev();
    let Some(mut top) = iter.next() else {
        return;
    };
    for pretop in iter {
        if semi[label[top as usize] as usize] < semi[label[pretop as usize] as usize] {
            label[pretop as usize] = label[top as usize];
        }
        ancestor[pretop as usize] = ancestor[top as usize];
        top = pretop;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_undirected_graph() {
        let g = Graph::with_vertices(2);
        let err = dominator_tree(&g, 0, DominatorMode::Out).expect_err("must reject undirected");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_out_of_range_root() {
        let g = Graph::new(3, true).expect("directed graph");
        let err = dominator_tree(&g, 5, DominatorMode::Out).expect_err("must reject bad root");
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 3);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn root_alone_in_directed_graph() {
        let mut g = Graph::new(3, true).expect("directed graph");
        // 0 has no outgoing edges; vertices 1 and 2 are unreachable.
        g.add_edge(1, 2).expect("add edge");
        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("compute");
        assert_eq!(dt.idom, vec![-1, -2, -2]);
        assert_eq!(dt.leftout, vec![1, 2]);
        assert_eq!(dt.tree.ecount(), 0);
        assert_eq!(dt.tree.vcount(), 3);
    }

    /// Classical 13-vertex Lengauer-Tarjan example (numbered 0..12 in
    /// the order used by `references/igraph/tests/unit/igraph_dominator_tree.c:23-46`).
    /// In this graph: idom values are taken directly from
    /// `references/igraph/tests/unit/igraph_dominator_tree.c:54-58`
    /// after stripping the +1 the reference vector uses.
    #[test]
    fn lengauer_tarjan_classical_13v() {
        let edges = [
            (0u32, 1u32),
            (0, 2),
            (0, 3),
            (1, 4),
            (2, 1),
            (2, 4),
            (2, 5),
            (3, 6),
            (3, 7),
            (4, 12),
            (5, 8),
            (6, 9),
            (7, 9),
            (7, 10),
            (8, 5),
            (8, 11),
            (9, 11),
            (10, 9),
            (11, 0),
            (11, 9),
            (12, 8),
        ];
        let mut g = Graph::new(13, true).expect("directed graph");
        for (u, v) in edges {
            g.add_edge(u, v).expect("add edge");
        }
        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("compute");
        // Reference (C unit test, +1 stripped):
        //   idom = [-1, 0, 0, 0, 0, 0, 3, 3, 0, 0, 7, 0, 4]
        assert_eq!(dt.idom, vec![-1, 0, 0, 0, 0, 0, 3, 3, 0, 0, 7, 0, 4]);
        assert!(dt.leftout.is_empty());
        // Tree has |V| - 1 edges (every non-root reachable vertex
        // contributes one).
        assert_eq!(dt.tree.ecount(), 12);
    }

    /// Reversed-edge variant of the classical 13-vertex example with
    /// `mode = In`: idom values match
    /// `references/igraph/tests/unit/igraph_dominator_tree.c:80-91`.
    #[test]
    fn lengauer_tarjan_in_mode_matches_reverse_out_mode() {
        // Build the *reverse* of the classical graph and run with OUT,
        // then build the classical graph and run with IN. Both must
        // produce the same idom (post-dominator tree of the classical
        // example).
        let edges = [
            (0u32, 1u32),
            (0, 2),
            (0, 3),
            (1, 4),
            (2, 1),
            (2, 4),
            (2, 5),
            (3, 6),
            (3, 7),
            (4, 12),
            (5, 8),
            (6, 9),
            (7, 9),
            (7, 10),
            (8, 5),
            (8, 11),
            (9, 11),
            (10, 9),
            (11, 0),
            (11, 9),
            (12, 8),
        ];
        let mut g_forward = Graph::new(13, true).expect("forward");
        for (u, v) in edges {
            g_forward.add_edge(u, v).expect("add edge");
        }
        let mut g_reverse = Graph::new(13, true).expect("reverse");
        for (u, v) in edges {
            g_reverse.add_edge(v, u).expect("add reverse edge");
        }

        let in_mode = dominator_tree(&g_forward, 0, DominatorMode::In).expect("In mode");
        let out_mode_reversed =
            dominator_tree(&g_reverse, 0, DominatorMode::Out).expect("Out on reversed");

        assert_eq!(in_mode.idom, out_mode_reversed.idom);
    }

    #[test]
    fn unreachable_vertices_land_in_leftout() {
        // Same shape as the 20-vertex test in
        // references/igraph/tests/unit/igraph_dominator_tree.c:104-117:
        // a tiny 4-vertex flowgraph plus a disconnected K3 cluster.
        let mut g = Graph::new(7, true).expect("directed graph");
        g.add_edge(0, 1).expect("e");
        g.add_edge(0, 2).expect("e");
        g.add_edge(1, 3).expect("e");
        g.add_edge(2, 3).expect("e");
        // Disconnected cluster: 4 → 5 → 6 → 4.
        g.add_edge(4, 5).expect("e");
        g.add_edge(5, 6).expect("e");
        g.add_edge(6, 4).expect("e");

        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("compute");
        // 0 is the root, 1/2/3 reachable via 0, 4/5/6 unreachable.
        assert_eq!(dt.idom[0], -1);
        assert_eq!(dt.idom[1], 0);
        assert_eq!(dt.idom[2], 0);
        assert_eq!(dt.idom[3], 0);
        assert_eq!(dt.idom[4], -2);
        assert_eq!(dt.idom[5], -2);
        assert_eq!(dt.idom[6], -2);
        assert_eq!(dt.leftout, vec![4, 5, 6]);
        assert_eq!(dt.tree.ecount(), 3);
    }

    #[test]
    fn idom_lies_on_every_root_to_w_path_brute_force() {
        // Small directed graph; brute-force verify the dominator
        // property: for each w != root, every root→w path must pass
        // through idom(w).
        let mut g = Graph::new(6, true).expect("directed graph");
        // A diamond with a side spur.
        g.add_edge(0, 1).expect("e");
        g.add_edge(0, 2).expect("e");
        g.add_edge(1, 3).expect("e");
        g.add_edge(2, 3).expect("e");
        g.add_edge(3, 4).expect("e");
        g.add_edge(4, 5).expect("e");
        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("compute");
        // Predictable shape: 0 dominates {1,2,3,4,5}; 3 dominates {4,5};
        // 4 dominates {5}.
        assert_eq!(dt.idom, vec![-1, 0, 0, 0, 3, 4]);

        // Verify with simple DFS path enumeration (n is tiny).
        for w in 1..6u32 {
            let d = dt.idom[w as usize];
            assert!(d >= 0, "vertex {w} should be reachable");
            let d_u = d as u32;
            let paths = enumerate_simple_paths(&g, 0, w);
            assert!(!paths.is_empty(), "no path 0 -> {w}?");
            for p in &paths {
                if w != 0 {
                    assert!(
                        p.contains(&d_u),
                        "idom({w}) = {d_u} must appear on every path: {p:?}"
                    );
                }
            }
        }
    }

    fn enumerate_simple_paths(g: &Graph, s: VertexId, t: VertexId) -> Vec<Vec<VertexId>> {
        let mut out: Vec<Vec<VertexId>> = Vec::new();
        let mut stack: Vec<VertexId> = vec![s];
        let mut on_stack = vec![false; g.vcount() as usize];
        on_stack[s as usize] = true;
        dfs_paths(g, s, t, &mut stack, &mut on_stack, &mut out);
        out
    }

    fn dfs_paths(
        g: &Graph,
        cur: VertexId,
        t: VertexId,
        stack: &mut Vec<VertexId>,
        on_stack: &mut [bool],
        out: &mut Vec<Vec<VertexId>>,
    ) {
        if cur == t {
            out.push(stack.clone());
            return;
        }
        let neis = g.out_neighbors_vec(cur).expect("out neis");
        for u in neis {
            if !on_stack[u as usize] {
                on_stack[u as usize] = true;
                stack.push(u);
                dfs_paths(g, u, t, stack, on_stack, out);
                stack.pop();
                on_stack[u as usize] = false;
            }
        }
    }
}
