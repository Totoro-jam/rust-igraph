//! Minimum spanning tree (ALGO-MST-001).
//!
//! Counterpart of `igraph_minimum_spanning_tree` from
//! `references/igraph/src/misc/spanning_trees.c` (~480 lines). Returns
//! the edge IDs that constitute a spanning tree (or spanning forest if
//! the graph is disconnected) of the input graph. Three internal
//! variants:
//!
//! * `Unweighted` — arbitrary spanning tree via BFS from each unvisited
//!   vertex. `O(|V|+|E|)`.
//! * `Prim` — eager Prim with a hand-rolled binary min-heap keyed by
//!   weight; ties resolved by edge-ID order (deterministic).
//!   `O(|E| log |V|)`.
//! * `Kruskal` — sort edges by weight ascending (`f64::total_cmp` for a
//!   total order on the IEEE-754 hierarchy, then by edge ID for ties),
//!   then walk with a path-compressed union-find. `O(|E| log |E|)`.
//!
//! [`MstAlgorithm::Automatic`] picks `Unweighted` when `weights` is
//! `None` and `Kruskal` otherwise — matching the upstream C dispatch
//! comment at `spanning_trees.c:466`.
//!
//! Directed edge directions are ignored, mirroring the upstream
//! behaviour: every directed edge is treated as undirected. Self-loops
//! never enter the tree (an MST edge must connect two distinct
//! components); parallel edges are deduplicated by the algorithm
//! (Kruskal picks the lightest; Prim's already-added bit short-circuits
//! the rest).
//!
//! The algorithm is fully deterministic for a fixed input — there is no
//! RNG anywhere in the pipeline.

use std::cmp::Ordering;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Selector for the minimum-spanning-tree algorithm.
///
/// Counterpart of the C `igraph_mst_algorithm_t` enum at
/// `spanning_trees.c:461-484`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MstAlgorithm {
    /// Let the implementation pick: `Unweighted` when `weights` is
    /// `None`, otherwise `Kruskal`.
    Automatic,
    /// Ignore edge weights and run a BFS spanning forest.
    Unweighted,
    /// Prim's algorithm with a binary min-heap (eager variant).
    Prim,
    /// Kruskal's algorithm with sort + union-find.
    Kruskal,
}

/// Computes a minimum spanning tree (or forest, if disconnected) of
/// `graph` and returns the IDs of the edges that constitute the tree.
///
/// Counterpart of `igraph_minimum_spanning_tree` from
/// `references/igraph/src/misc/spanning_trees.c:461`.
///
/// # Arguments
///
/// * `graph` - Input graph. Edge directions are ignored.
/// * `weights` - Optional edge weights, indexed by edge ID. When
///   provided, `weights.len()` must equal `graph.ecount()`. NaN
///   entries are rejected. When `None`, edge weights are treated as
///   uniform and the spanning forest is an arbitrary BFS tree.
/// * `method` - Which underlying algorithm to use; see
///   [`MstAlgorithm`].
///
/// # Returns
///
/// The edge IDs (in the order the algorithm picked them) that form
/// the spanning tree / forest. For a graph with `c` connected
/// components, the result has exactly `vcount − c` edges. The order
/// reflects the algorithm: BFS layer-by-layer for `Unweighted`,
/// pop-order for `Prim`, ascending-weight for `Kruskal`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when
/// * `weights.len()` does not match `graph.ecount()`,
/// * any weight is NaN, or
/// * `method` is a weighted variant (`Prim` / `Kruskal`) but `weights`
///   is `None`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, MstAlgorithm, minimum_spanning_tree};
///
/// // Square 0-1-2-3-0 with diagonals 0-2 and 1-3, weights chosen so
/// // the unique MST is the four outer edges (weight 1) and skips both
/// // diagonals (weight 10).
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // eid 0, w=1
/// g.add_edge(1, 2).unwrap(); // eid 1, w=1
/// g.add_edge(2, 3).unwrap(); // eid 2, w=1
/// g.add_edge(3, 0).unwrap(); // eid 3, w=1
/// g.add_edge(0, 2).unwrap(); // eid 4, w=10
/// g.add_edge(1, 3).unwrap(); // eid 5, w=10
///
/// let w = [1.0, 1.0, 1.0, 1.0, 10.0, 10.0];
/// let mut tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
/// tree.sort_unstable();
/// // A 4-vertex graph needs vcount-1 = 3 tree edges (one square edge
/// // closes the cycle).
/// assert_eq!(tree.len(), 3);
/// assert!(tree.iter().all(|&e| e < 4));
/// ```
///
/// # References
///
/// * R. C. Prim, *Shortest connection networks and some
///   generalizations*, Bell System Technical Journal **36** (1957),
///   1389-1401.
/// * J. B. Kruskal, *On the shortest spanning subtree of a graph and
///   the traveling salesman problem*, Proc. Amer. Math. Soc. **7**
///   (1956), 48-50.
pub fn minimum_spanning_tree(
    graph: &Graph,
    weights: Option<&[f64]>,
    method: MstAlgorithm,
) -> IgraphResult<Vec<EdgeId>> {
    let resolved = match method {
        MstAlgorithm::Automatic => {
            if weights.is_none() {
                MstAlgorithm::Unweighted
            } else {
                MstAlgorithm::Kruskal
            }
        }
        other => other,
    };

    match resolved {
        MstAlgorithm::Unweighted => mst_unweighted(graph),
        MstAlgorithm::Prim => {
            let w = require_weights(graph, weights)?;
            mst_prim(graph, w)
        }
        MstAlgorithm::Kruskal => {
            let w = require_weights(graph, weights)?;
            mst_kruskal(graph, w)
        }
        MstAlgorithm::Automatic => unreachable!("Automatic resolved above"),
    }
}

fn require_weights<'a>(graph: &Graph, weights: Option<&'a [f64]>) -> IgraphResult<&'a [f64]> {
    let w = weights.ok_or_else(|| {
        IgraphError::InvalidArgument(
            "weights required for the chosen MST algorithm (Prim/Kruskal); supply Some(&[..])"
                .to_string(),
        )
    })?;
    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights length {} does not match edge count {}",
            w.len(),
            m
        )));
    }
    if w.iter().any(|x| x.is_nan()) {
        return Err(IgraphError::InvalidArgument(
            "weights must not contain NaN values".to_string(),
        ));
    }
    Ok(w)
}

fn max_tree_edges(vcount: usize, ecount: usize) -> usize {
    if vcount == 0 {
        0
    } else if ecount < vcount {
        ecount
    } else {
        vcount - 1
    }
}

// ---------- Unweighted: BFS spanning forest ----------

fn mst_unweighted(graph: &Graph) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut result: Vec<EdgeId> = Vec::with_capacity(max_tree_edges(n, m));
    if n == 0 {
        return Ok(result);
    }
    let mut already_added = vec![false; n];
    let mut added_edges = vec![false; m];
    let mut queue: std::collections::VecDeque<VertexId> = std::collections::VecDeque::new();

    for start in 0..n {
        if already_added[start] {
            continue;
        }
        already_added[start] = true;
        let start_v = u32::try_from(start)
            .map_err(|_| IgraphError::InvalidArgument("vertex id overflows u32".to_string()))?;
        queue.push_back(start_v);
        while let Some(v) = queue.pop_front() {
            let eids = graph.incident(v)?;
            for &edge in &eids {
                let e_idx = edge as usize;
                if added_edges[e_idx] {
                    continue;
                }
                let to = graph.edge_other(edge, v)?;
                let to_idx = to as usize;
                if already_added[to_idx] {
                    continue;
                }
                already_added[to_idx] = true;
                added_edges[e_idx] = true;
                result.push(edge);
                queue.push_back(to);
            }
        }
    }
    Ok(result)
}

// ---------- Prim: eager binary min-heap ----------

/// Min-heap entry for Prim. Ordered by `(weight asc, edge asc)` —
/// edge-ID is the tie-breaker so equal-weight runs are deterministic.
#[derive(Debug, Clone, Copy)]
struct HeapEntry {
    weight: f64,
    edge: EdgeId,
    from: VertexId,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight
            .total_cmp(&other.weight)
            .then(self.edge.cmp(&other.edge))
    }
}

fn mst_prim(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut result: Vec<EdgeId> = Vec::with_capacity(max_tree_edges(n, m));
    if n == 0 {
        return Ok(result);
    }
    let mut already_added = vec![false; n];
    let mut added_edges = vec![false; m];
    // std::collections::BinaryHeap is a max-heap; we wrap entries in
    // Reverse so the smallest weight comes out first.
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<HeapEntry>> =
        std::collections::BinaryHeap::new();

    for start in 0..n {
        if already_added[start] {
            continue;
        }
        let start_v = u32::try_from(start)
            .map_err(|_| IgraphError::InvalidArgument("vertex id overflows u32".to_string()))?;
        already_added[start] = true;
        push_incident_into_heap(graph, weights, &already_added, start_v, &mut heap)?;

        while let Some(std::cmp::Reverse(entry)) = heap.pop() {
            let edge = entry.edge;
            let e_idx = edge as usize;
            if added_edges[e_idx] {
                continue;
            }
            let to = graph.edge_other(edge, entry.from)?;
            let to_idx = to as usize;
            if already_added[to_idx] {
                // Edge would close a cycle inside the current
                // partial-tree component; skip.
                continue;
            }
            already_added[to_idx] = true;
            added_edges[e_idx] = true;
            result.push(edge);
            push_incident_into_heap(graph, weights, &already_added, to, &mut heap)?;
        }
    }
    Ok(result)
}

fn push_incident_into_heap(
    graph: &Graph,
    weights: &[f64],
    already_added: &[bool],
    v: VertexId,
    heap: &mut std::collections::BinaryHeap<std::cmp::Reverse<HeapEntry>>,
) -> IgraphResult<()> {
    let eids = graph.incident(v)?;
    for &edge in &eids {
        let other = graph.edge_other(edge, v)?;
        if already_added[other as usize] {
            continue;
        }
        let w = weights[edge as usize];
        heap.push(std::cmp::Reverse(HeapEntry {
            weight: w,
            edge,
            from: v,
        }));
    }
    Ok(())
}

// ---------- Kruskal: sort + union-find ----------

fn mst_kruskal(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut result: Vec<EdgeId> = Vec::with_capacity(max_tree_edges(n, m));
    if n == 0 {
        return Ok(result);
    }

    // Indices into the edge list, sorted by weight ascending. Ties are
    // broken by edge ID so the picked tree is deterministic across
    // platforms regardless of `sort_by`'s internal stability.
    let mut order: Vec<EdgeId> = (0..u32::try_from(m)
        .map_err(|_| IgraphError::InvalidArgument("edge count overflows u32".to_string()))?)
        .collect();
    order.sort_by(|a, b| {
        weights[*a as usize]
            .total_cmp(&weights[*b as usize])
            .then(a.cmp(b))
    });

    let mut parent: Vec<u32> = (0..u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("vertex count overflows u32".to_string()))?)
        .collect();

    let target = max_tree_edges(n, m);
    for &edge in &order {
        if result.len() == target {
            break;
        }
        let (u, v) = graph.edge(edge)?;
        let ru = uf_find(&mut parent, u as usize);
        let rv = uf_find(&mut parent, v as usize);
        if ru != rv {
            // Path-compressed merge: hang ru under rv (matches C
            // `merge_comp` which sets parent[ci] = cj).
            let rv_u32 = u32::try_from(rv)
                .map_err(|_| IgraphError::InvalidArgument("vertex id overflows u32".to_string()))?;
            parent[ru] = rv_u32;
            result.push(edge);
        }
    }
    Ok(result)
}

fn uf_find(parent: &mut [u32], mut i: usize) -> usize {
    // Mirror of the C `get_comp` — climb to the root, then collapse
    // only `i` (no full path compression, but does avoid quadratic
    // worst-case in practice).
    let start = i;
    loop {
        let next = parent[i] as usize;
        if next == i {
            // Cache the root in the original index slot, leaving the
            // rest of the chain untouched (matches the upstream C).
            // Falls back gracefully if u32::try_from would fail —
            // `parent` is sized to vcount which is bounded by u32.
            if let Ok(root_u32) = u32::try_from(i) {
                parent[start] = root_u32;
            }
            return i;
        }
        i = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_sorted(mut v: Vec<EdgeId>) -> Vec<EdgeId> {
        v.sort_unstable();
        v
    }

    fn weight_sum(weights: &[f64], edges: &[EdgeId]) -> f64 {
        edges.iter().map(|&e| weights[e as usize]).sum()
    }

    fn assert_is_forest(graph: &Graph, edges: &[EdgeId], expected_components: usize) {
        let n = graph.vcount() as usize;
        // Forest invariant: |edges| == n - components, plus union-find
        // detects no cycle on the picked edges.
        assert_eq!(
            edges.len(),
            n - expected_components,
            "expected {} tree edges for {} vertices in {} components, got {}",
            n - expected_components,
            n,
            expected_components,
            edges.len()
        );
        let mut parent: Vec<u32> = (0..u32::try_from(n).expect("vcount fits u32")).collect();
        for &eid in edges {
            let (u, v) = graph.edge(eid).expect("edge exists");
            let ru = uf_find(&mut parent, u as usize);
            let rv = uf_find(&mut parent, v as usize);
            assert_ne!(ru, rv, "edge {eid} closes a cycle in the spanning tree");
            parent[ru] = u32::try_from(rv).expect("vid fits u32");
        }
    }

    // ---------- Empty / singleton ----------

    #[test]
    fn empty_graph_returns_empty_tree() {
        let g = Graph::with_vertices(0);
        for method in [
            MstAlgorithm::Automatic,
            MstAlgorithm::Unweighted,
            MstAlgorithm::Prim,
            MstAlgorithm::Kruskal,
        ] {
            let weights: Option<&[f64]> =
                if matches!(method, MstAlgorithm::Prim | MstAlgorithm::Kruskal) {
                    Some(&[])
                } else {
                    None
                };
            let tree = minimum_spanning_tree(&g, weights, method).unwrap();
            assert!(tree.is_empty(), "{method:?} on empty graph");
        }
    }

    #[test]
    fn single_vertex_no_edges_is_empty_tree() {
        let g = Graph::with_vertices(1);
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Automatic).unwrap();
        assert!(tree.is_empty());
    }

    #[test]
    fn isolated_vertices_are_a_forest_with_no_edges() {
        let g = Graph::with_vertices(5);
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert!(tree.is_empty());
        assert_is_forest(&g, &tree, 5);
    }

    // ---------- Unweighted ----------

    #[test]
    fn unweighted_chain_picks_all_edges() {
        // 0-1-2-3 is a tree already; MST = all 3 edges.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(tree.len(), 3);
        assert_is_forest(&g, &tree, 1);
    }

    #[test]
    fn unweighted_triangle_drops_one_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(tree.len(), 2);
        assert_is_forest(&g, &tree, 1);
    }

    #[test]
    fn unweighted_forest_disjoint_components() {
        // Two disjoint trees: 0-1-2 and 3-4 ⇒ MST keeps everything.
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (3, 4)]).unwrap();
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(tree.len(), 3);
        assert_is_forest(&g, &tree, 2);
    }

    #[test]
    fn unweighted_complete_k5_keeps_n_minus_1_edges() {
        let mut g = Graph::with_vertices(5);
        for i in 0u32..5 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(tree.len(), 4);
        assert_is_forest(&g, &tree, 1);
    }

    // ---------- Prim ----------

    #[test]
    fn prim_picks_unique_mst_when_weights_are_distinct() {
        // Square + diagonals; outer edges have weight 1, diagonals 10.
        // The unique MST is the outer 4-edge cycle minus one edge, i.e.
        // any 3 of the four outer edges.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (0, 2), (1, 3)])
            .unwrap();
        let w = [1.0, 1.0, 1.0, 1.0, 10.0, 10.0];
        let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap();
        assert_eq!(tree.len(), 3);
        // None of the diagonals (eids 4, 5) should appear.
        assert!(tree.iter().all(|&e| e < 4));
        assert_is_forest(&g, &tree, 1);
        let total = weight_sum(&w, &tree);
        assert!((total - 3.0).abs() < 1e-12);
    }

    #[test]
    fn prim_handles_disconnected_graphs_as_forest() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (3, 4)]).unwrap();
        let w = [1.0, 2.0, 3.0];
        let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap();
        assert_eq!(tree.len(), 3);
        assert_is_forest(&g, &tree, 2);
    }

    #[test]
    fn prim_and_kruskal_agree_on_distinct_weight_mst() {
        // Mini road graph: 5 vertices, 7 edges, all distinct weights ⇒
        // MST is unique.
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 3),
            (3, 4),
            (2, 4),
        ])
        .unwrap();
        let w = [1.0, 4.0, 2.0, 7.0, 3.0, 5.0, 6.0];
        let prim = collect_sorted(minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap());
        let kruskal =
            collect_sorted(minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap());
        assert_eq!(prim, kruskal);
        assert_eq!(prim.len(), 4);
        // With these weights the unique MST is {(0,1)=1, (1,2)=2, (2,3)=3, (3,4)=5}
        // — eids 0, 2, 4, 5.
        assert_eq!(prim, vec![0u32, 2, 4, 5]);
    }

    // ---------- Kruskal ----------

    #[test]
    fn kruskal_breaks_ties_by_edge_id() {
        // Triangle with equal weights ⇒ Kruskal keeps the two
        // lowest-id edges (0 and 1), dropping edge 2.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let w = [1.0, 1.0, 1.0];
        let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        assert_eq!(tree, vec![0u32, 1]);
    }

    #[test]
    fn kruskal_picks_lightest_parallel_edge() {
        // Two parallel edges between 0 and 1, weights 5 and 1.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = [5.0, 1.0];
        let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        // Lightest parallel edge wins; the heavier one is dropped.
        assert_eq!(tree, vec![1u32]);
    }

    #[test]
    fn kruskal_ignores_self_loops() {
        // 0-0 self-loop + 0-1 edge: self-loop never appears in the
        // spanning tree (a tree edge must connect two distinct
        // components).
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // eid 0
        g.add_edge(0, 1).unwrap(); // eid 1
        let w = [0.5, 2.0];
        let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        assert_eq!(tree, vec![1u32]);
    }

    // ---------- Determinism / convergence ----------

    #[test]
    fn unweighted_is_deterministic_across_runs() {
        let mut g = Graph::with_vertices(5);
        for i in 0u32..5 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let a = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        let b = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn prim_and_kruskal_have_equal_total_weight_even_when_trees_differ() {
        // Square with all equal weights ⇒ many valid MSTs, but total
        // weight is invariant.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0)])
            .unwrap();
        let w = [1.0, 1.0, 1.0, 1.0];
        let prim = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap();
        let kruskal = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        assert_eq!(prim.len(), 3);
        assert_eq!(kruskal.len(), 3);
        assert!((weight_sum(&w, &prim) - weight_sum(&w, &kruskal)).abs() < 1e-12);
    }

    // ---------- Directed → undirected ----------

    #[test]
    fn directed_graph_is_treated_as_undirected() {
        // Directed 0→1, 1→2, 2→0 — three edges; the MST/forest still
        // has 2 edges (vcount-1=2 for one connected component).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(tree.len(), 2);
        assert_is_forest(&g, &tree, 1);
    }

    // ---------- Automatic dispatch ----------

    #[test]
    fn automatic_picks_unweighted_when_no_weights() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        let a = minimum_spanning_tree(&g, None, MstAlgorithm::Automatic).unwrap();
        let b = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn automatic_picks_kruskal_when_weights_given() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0)])
            .unwrap();
        let w = [1.0, 2.0, 3.0, 4.0];
        let a = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Automatic).unwrap();
        let b = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        assert_eq!(a, b);
    }

    // ---------- Error paths ----------

    #[test]
    fn prim_without_weights_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = minimum_spanning_tree(&g, None, MstAlgorithm::Prim).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn kruskal_without_weights_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = minimum_spanning_tree(&g, None, MstAlgorithm::Kruskal).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn mismatched_weight_length_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let w = [1.0, 2.0, 3.0]; // length 3, ecount = 2
        let err = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn nan_weights_are_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let w = [f64::NAN];
        let err = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    // ---------- Stress / weight ordering ----------

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn kruskal_total_weight_minimal_on_random_distinct_weights() {
        // Brute-force enumeration on a small graph: 5 vertices, 8
        // edges, all distinct weights. Compare Kruskal's total weight
        // against every spanning tree from a brute-force enumeration.
        let mut g = Graph::with_vertices(5);
        let edges = [
            (0u32, 1u32),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 4),
            (2, 3),
            (2, 4),
            (3, 4),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        let w: [f64; 8] = [4.0, 1.0, 7.0, 2.0, 5.0, 6.0, 3.0, 8.0];
        let kruskal_tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
        let kruskal_total = weight_sum(&w, &kruskal_tree);

        // Brute-force: enumerate every 4-edge subset; check tree (no
        // cycle, connected) — track min total.
        let n_vertices = 5usize;
        let n_edges = 8usize;
        let mut min_total = f64::INFINITY;
        // C(8,4) = 70; cheap.
        for mask in 0u32..(1 << n_edges) {
            if mask.count_ones() != 4 {
                continue;
            }
            let mut parent: Vec<u32> =
                (0..u32::try_from(n_vertices).expect("vcount fits u32")).collect();
            let mut total = 0.0f64;
            let mut cycle = false;
            for idx in 0..n_edges {
                if mask & (1 << idx) == 0 {
                    continue;
                }
                let (u, v) = edges[idx];
                let ru = uf_find(&mut parent, u as usize);
                let rv = uf_find(&mut parent, v as usize);
                if ru == rv {
                    cycle = true;
                    break;
                }
                parent[ru] = u32::try_from(rv).unwrap();
                total += w[idx];
            }
            if cycle {
                continue;
            }
            // Check connectivity ⇒ one component.
            let roots: std::collections::HashSet<usize> =
                (0..n_vertices).map(|x| uf_find(&mut parent, x)).collect();
            if roots.len() != 1 {
                continue;
            }
            if total < min_total {
                min_total = total;
            }
        }
        assert!((kruskal_total - min_total).abs() < 1e-12);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Spanning tree invariants that must hold for every MST result:
    ///   1. The picked edges are pairwise distinct (no duplicates).
    ///   2. They form an acyclic subgraph (union-find sees no cycle).
    ///   3. Edge count == vcount - components (forest invariant).
    fn assert_forest_invariants(graph: &Graph, edges: &[EdgeId]) -> Result<(), TestCaseError> {
        // Distinctness.
        let unique: std::collections::HashSet<EdgeId> = edges.iter().copied().collect();
        prop_assert_eq!(unique.len(), edges.len(), "duplicate edge in MST result");

        let n = graph.vcount() as usize;
        let mut parent: Vec<u32> = (0..u32::try_from(n).expect("vcount fits u32")).collect();
        for &eid in edges {
            let (u, v) = graph
                .edge(eid)
                .map_err(|e| TestCaseError::Fail(e.to_string().into()))?;
            let ru = uf_find(&mut parent, u as usize);
            let rv = uf_find(&mut parent, v as usize);
            prop_assert!(ru != rv, "edge {} closes a cycle in the MST", eid);
            let rv_u32 =
                u32::try_from(rv).map_err(|_| TestCaseError::Fail("vid overflow".into()))?;
            parent[ru] = rv_u32;
        }

        // Compute components on the spanning forest (= components of
        // the original graph because every component contributes its
        // own spanning tree).
        let mut tree_roots = std::collections::HashSet::new();
        for v in 0..n {
            tree_roots.insert(uf_find(&mut parent, v));
        }

        // Also compute components on the *full* graph for the
        // expected count.
        let mut full_parent: Vec<u32> = (0..u32::try_from(n).expect("vcount fits u32")).collect();
        for eid in 0..u32::try_from(graph.ecount()).unwrap_or(u32::MAX) {
            let (u, v) = match graph.edge(eid) {
                Ok(pair) => pair,
                Err(_) => continue,
            };
            let ru = uf_find(&mut full_parent, u as usize);
            let rv = uf_find(&mut full_parent, v as usize);
            if ru != rv {
                full_parent[ru] = u32::try_from(rv).unwrap_or(u32::MAX);
            }
        }
        let mut full_roots = std::collections::HashSet::new();
        for v in 0..n {
            full_roots.insert(uf_find(&mut full_parent, v));
        }

        prop_assert_eq!(
            tree_roots.len(),
            full_roots.len(),
            "spanning forest must have the same component count as the original graph"
        );
        prop_assert_eq!(
            edges.len(),
            n - full_roots.len(),
            "MST edge count must equal vcount - components"
        );
        Ok(())
    }

    prop_compose! {
        fn small_undirected_graph()(n in 2u32..=10u32, edges_seed in any::<u64>())
            -> Graph {
            let mut g = Graph::with_vertices(n);
            let mut rng = edges_seed;
            let target_m = ((n * (n - 1)) / 2).min(n + 6) as usize;
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

    prop_compose! {
        fn small_weighted_graph()(g in small_undirected_graph(), seed in any::<u64>())
            -> (Graph, Vec<f64>) {
            let m = g.ecount();
            let mut weights = Vec::with_capacity(m);
            let mut rng = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
            for _ in 0..m {
                rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                // Weights in [0.5, 100.5) — strictly positive, finite,
                // never NaN.
                let bits = (rng >> 11) as f64 / (1u64 << 53) as f64;
                weights.push(0.5 + 100.0 * bits);
            }
            (g, weights)
        }
    }

    proptest! {
        #[test]
        fn unweighted_is_a_spanning_forest(g in small_undirected_graph()) {
            let tree = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
            assert_forest_invariants(&g, &tree)?;
        }

        #[test]
        fn prim_is_a_spanning_forest((g, w) in small_weighted_graph()) {
            let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap();
            assert_forest_invariants(&g, &tree)?;
        }

        #[test]
        fn kruskal_is_a_spanning_forest((g, w) in small_weighted_graph()) {
            let tree = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
            assert_forest_invariants(&g, &tree)?;
        }

        #[test]
        fn prim_and_kruskal_have_equal_total_weight((g, w) in small_weighted_graph()) {
            let prim = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap();
            let kruskal = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
            let sp: f64 = prim.iter().map(|&e| w[e as usize]).sum();
            let sk: f64 = kruskal.iter().map(|&e| w[e as usize]).sum();
            // Matroid optimality: every MST has the same total weight.
            prop_assert!((sp - sk).abs() < 1e-9, "prim={} kruskal={}", sp, sk);
        }

        #[test]
        fn automatic_matches_underlying((g, w) in small_weighted_graph()) {
            // Automatic w/ weights == Kruskal.
            let auto = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Automatic).unwrap();
            let kruskal = minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap();
            prop_assert_eq!(auto, kruskal);

            // Automatic w/o weights == Unweighted.
            let auto = minimum_spanning_tree(&g, None, MstAlgorithm::Automatic).unwrap();
            let unweighted =
                minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap();
            prop_assert_eq!(auto, unweighted);
        }
    }
}
