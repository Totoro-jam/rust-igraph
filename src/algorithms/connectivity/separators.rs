//! Vertex separators (ALGO-CN-015).
//!
//! Counterpart of `igraph_is_separator()` and
//! `igraph_is_minimal_separator()` from
//! `references/igraph/src/connectivity/separators.c`.
//!
//! A *vertex separator* of a connected graph is a set of vertices
//! whose removal disconnects the graph (or isolates a vertex from
//! the rest). A separator is *minimal* if no proper subset of it is
//! also a separator.

use std::collections::VecDeque;

use crate::algorithms::connectivity::articulation::articulation_points;
use crate::algorithms::flow::all_st_mincuts::all_st_mincuts;
use crate::algorithms::flow::max_flow::max_flow_value;
use crate::algorithms::flow::vertex_connectivity::vertex_connectivity;
use crate::algorithms::operators::even_tarjan::even_tarjan_reduction;
use crate::algorithms::operators::simplify::simplify;
use crate::algorithms::properties::are_adjacent::are_adjacent;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Check whether a set of vertices is a separator of the graph.
///
/// A vertex set S is a separator if removing S (and all incident edges)
/// makes the remaining graph disconnected, OR if removing S leaves
/// fewer vertices than the original graph minus |S| (i.e., some vertex
/// becomes isolated). For a graph that is already disconnected, any
/// set is technically a separator — this function returns `true` for
/// the empty set in that case.
///
/// For undirected graphs only.
///
/// # Errors
///
/// - `InvalidArgument` if the graph is directed.
/// - `InvalidArgument` if any vertex ID in `candidates` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_separator};
///
/// // Path 0-1-2: removing vertex 1 disconnects the graph.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_separator(&g, &[1]).unwrap());
/// assert!(!is_separator(&g, &[0]).unwrap()); // leaf removal doesn't disconnect
/// ```
pub fn is_separator(graph: &Graph, candidates: &[VertexId]) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "is_separator: only defined for undirected graphs".into(),
        ));
    }

    let n = graph.vcount();
    for &v in candidates {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "is_separator: vertex {v} out of range (vcount={n})"
            )));
        }
    }

    if n == 0 {
        return Ok(false);
    }

    // Mark candidates in a set for O(1) lookup.
    let n_us = n as usize;
    let mut removed = vec![false; n_us];
    for &v in candidates {
        removed[v as usize] = true;
    }

    // Count remaining vertices (those not in the removed set).
    let remaining = (0..n_us).filter(|&v| !removed[v]).count();

    if remaining == 0 {
        return Ok(false);
    }

    // BFS from the first non-removed vertex. If it can't reach all
    // remaining vertices, the graph is disconnected → separator.
    let start = (0..n_us).find(|&v| !removed[v]).unwrap();
    #[allow(clippy::cast_possible_truncation)] // start < n which is u32
    let reached = bfs_count(graph, start as u32, &removed)?;

    Ok(reached < remaining)
}

/// Check whether a set of vertices is a *minimal* separator.
///
/// A separator is minimal if no proper subset is also a separator.
/// Equivalently, S is a minimal separator if it is a separator and
/// for every vertex v in S, removing S \ {v} does NOT disconnect the
/// graph.
///
/// # Errors
///
/// - `InvalidArgument` if the graph is directed.
/// - `InvalidArgument` if any vertex ID is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_minimal_separator};
///
/// // 4-cycle: {1,3} is a minimal separator (removing both disconnects 0 from 2).
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(is_minimal_separator(&g, &[1, 3]).unwrap());
/// // {0,1,3} leaves only vertex 2 — not a separator, hence not minimal.
/// assert!(!is_minimal_separator(&g, &[0, 1, 3]).unwrap());
/// ```
pub fn is_minimal_separator(graph: &Graph, candidates: &[VertexId]) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "is_minimal_separator: only defined for undirected graphs".into(),
        ));
    }

    let n = graph.vcount();
    for &v in candidates {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "is_minimal_separator: vertex {v} out of range (vcount={n})"
            )));
        }
    }

    // First check: is it a separator at all?
    if !is_separator(graph, candidates)? {
        return Ok(false);
    }

    // For each vertex in the candidate set, check if removing it
    // still leaves a separator. If yes for any vertex, it's not minimal.
    let n_us = n as usize;
    for (idx, _) in candidates.iter().enumerate() {
        // Build the "removed" set without vertex v.
        let mut removed = vec![false; n_us];
        for (j, &u) in candidates.iter().enumerate() {
            if j != idx {
                removed[u as usize] = true;
            }
        }

        let remaining = (0..n_us).filter(|&x| !removed[x]).count();
        if remaining == 0 {
            continue;
        }

        let start = (0..n_us).find(|&x| !removed[x]).unwrap();
        #[allow(clippy::cast_possible_truncation)] // start < n which is u32
        let reached = bfs_count(graph, start as u32, &removed)?;

        if reached < remaining {
            // S \ {v} is still a separator → S is not minimal.
            return Ok(false);
        }
    }

    Ok(true)
}

/// BFS from `start`, skipping removed vertices. Returns count of reachable vertices.
fn bfs_count(graph: &Graph, start: u32, removed: &[bool]) -> IgraphResult<usize> {
    let n_us = graph.vcount() as usize;
    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();
    let mut count = 0usize;

    visited[start as usize] = true;
    queue.push_back(start);
    count += 1;

    while let Some(cur) = queue.pop_front() {
        let neighbors = graph.neighbors(cur)?;
        for &nb in &neighbors {
            let nidx = nb as usize;
            if !visited[nidx] && !removed[nidx] {
                visited[nidx] = true;
                queue.push_back(nb);
                count += 1;
            }
        }
    }

    Ok(count)
}

/// List all vertex sets that are minimal (s,t) separators for some s and t.
///
/// A vertex set S is a *minimal (s,t) separator* if removing S disconnects
/// s from t, and no proper subset of S does the same for that pair.
///
/// This function enumerates ALL such sets (for all possible pairs s,t).
/// Note that a returned separator may not be minimal with respect to
/// *disconnecting the graph* — see the igraph docs for details.
///
/// Based on Berry, Bordat & Cogis (1999): "Generating All the Minimal
/// Separators of a Graph".
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, all_minimal_st_separators};
///
/// // Path 0-1-2-3-4-1 (pentagon with chord):
/// // edges: 0-1, 1-2, 2-3, 3-4, 4-1
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 1).unwrap();
/// let seps = all_minimal_st_separators(&g).unwrap();
/// // Should contain {1}, {2,4}, {1,3}
/// assert!(seps.iter().any(|s| s == &[1]));
/// assert!(seps.iter().any(|s| s == &[2, 4]));
/// assert!(seps.iter().any(|s| s == &[1, 3]));
/// ```
pub fn all_minimal_st_separators(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_adj_undirected(graph)?;

    let mut separators: Vec<Vec<VertexId>> = Vec::new();
    let mut mark: Vec<u32> = vec![0; n];
    let mut stamp: u32 = 1;

    // Phase 1 (Initialization): For each vertex v, mark N[v] as removed,
    // find components in remaining graph, compute N(C) for each component.
    for v in 0..n {
        advance_stamp(&mut mark, &mut stamp, n);
        mark[v] = stamp;
        for &nb in &adj[v] {
            mark[nb as usize] = stamp;
        }

        let components = find_components_leaveout(&adj, &mark, stamp, n);
        store_separators(&adj, &components, &mut mark, &mut separators, &mut stamp, n);
    }

    // Phase 2 (Generation): Use found separators as basis to find more.
    let mut try_next = 0;
    while try_next < separators.len() {
        let basis = separators[try_next].clone();
        for &x in &basis {
            advance_stamp(&mut mark, &mut stamp, n);
            for &sv in &basis {
                mark[sv as usize] = stamp;
            }
            for &nb in &adj[x as usize] {
                mark[nb as usize] = stamp;
            }

            let components = find_components_leaveout(&adj, &mark, stamp, n);
            store_separators(&adj, &components, &mut mark, &mut separators, &mut stamp, n);
        }
        try_next += 1;
    }

    Ok(separators)
}

fn build_adj_undirected(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let m = graph.ecount();
    for eid in 0..m {
        let eid32 = u32::try_from(eid).map_err(|_| {
            IgraphError::InvalidArgument("all_minimal_st_separators: edge id overflow".into())
        })?;
        let (from, to) = graph.edge(eid32)?;
        adj[from as usize].push(to);
        if from != to {
            adj[to as usize].push(from);
        }
    }
    Ok(adj)
}

/// Find connected components among vertices not marked with `stamp`.
/// Returns a list of components; each component is a Vec of vertex ids.
fn find_components_leaveout(
    adj: &[Vec<VertexId>],
    mark: &[u32],
    stamp: u32,
    n: usize,
) -> Vec<Vec<VertexId>> {
    let mut visited = vec![false; n];
    let mut components: Vec<Vec<VertexId>> = Vec::new();
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    for i in 0..n {
        if mark[i] == stamp || visited[i] {
            continue;
        }

        let mut comp: Vec<VertexId> = Vec::new();
        #[allow(clippy::cast_possible_truncation)]
        let i_v = i as VertexId;
        visited[i] = true;
        queue.push_back(i_v);
        comp.push(i_v);

        while let Some(cur) = queue.pop_front() {
            for &nb in &adj[cur as usize] {
                let nb_us = nb as usize;
                if mark[nb_us] == stamp || visited[nb_us] {
                    continue;
                }
                visited[nb_us] = true;
                queue.push_back(nb);
                comp.push(nb);
            }
        }

        components.push(comp);
    }

    components
}

/// For each component C, compute N(C) = vertices adjacent to C but not in C.
/// Since C is a connected component of G - S, N(C) ⊆ S. Store as a new
/// separator if not already seen. Advances `cur_stamp` afterward.
fn store_separators(
    adj: &[Vec<VertexId>],
    components: &[Vec<VertexId>],
    mark: &mut [u32],
    separators: &mut Vec<Vec<VertexId>>,
    cur_stamp: &mut u32,
    n: usize,
) {
    for comp in components {
        advance_stamp(mark, cur_stamp, n);
        let comp_stamp = *cur_stamp;

        // Mark component vertices
        for &v in comp {
            mark[v as usize] = comp_stamp;
        }

        // Collect neighbors not in C (they must be in the separator S)
        let mut neighborhood: Vec<VertexId> = Vec::new();
        for &v in comp {
            for &nb in &adj[v as usize] {
                let nb_us = nb as usize;
                if mark[nb_us] != comp_stamp {
                    mark[nb_us] = comp_stamp;
                    neighborhood.push(nb);
                }
            }
        }

        if neighborhood.is_empty() {
            continue;
        }

        neighborhood.sort_unstable();

        if !separators.contains(&neighborhood) {
            separators.push(neighborhood);
        }
    }

    advance_stamp(mark, cur_stamp, n);
}

fn advance_stamp(mark: &mut [u32], stamp: &mut u32, _n: usize) {
    *stamp = stamp.wrapping_add(1);
    if *stamp == 0 {
        for m in mark.iter_mut() {
            *m = 0;
        }
        *stamp = 1;
    }
}

/// Find the `k` vertices of largest degree (ties broken by descending
/// vertex id, matching igraph's `igraph_i_vector_int_order` convention:
/// a stable ascending sort whose tail is read back). The graph is
/// assumed simple.
fn topk_degree(graph: &Graph, k: usize) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let mut deg: Vec<(usize, VertexId)> = Vec::with_capacity(n as usize);
    for v in 0..n {
        deg.push((graph.degree(v)?, v));
    }
    // Stable ascending order by (degree, id); the k highest-degree
    // vertices are the last k. Reading them back high-to-low yields the
    // higher-id member first among equal degrees.
    deg.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut res = Vec::with_capacity(k);
    for i in 0..k {
        res.push(deg[n as usize - 1 - i].1);
    }
    Ok(res)
}

/// Append every separator in `new` to `acc` that is not already present.
/// Each separator is compared as an ordered vector (igraph
/// `igraph_vector_int_all_e` semantics); callers pass canonically
/// (ascending) sorted vectors so set-equality reduces to vector-equality.
fn append_unique(acc: &mut Vec<Vec<VertexId>>, new: Vec<Vec<VertexId>>) {
    for sep in new {
        if !acc.iter().any(|existing| existing == &sep) {
            acc.push(sep);
        }
    }
}

/// Find all minimum-size separating vertex sets.
///
/// A vertex set is a *separator* if its removal disconnects the graph.
/// This function lists every separator of minimum cardinality (the
/// minimum equals the graph's vertex connectivity).
///
/// Conventions, matching `igraph_minimum_size_separators`:
///
/// * If the graph is already disconnected, no separators are returned
///   (this differs from [`all_minimal_st_separators`]).
/// * Complete graphs have no vertex separators, so the result is empty.
/// * For a graph whose connectivity is `1`, the minimum separators are
///   exactly the articulation points (each returned as a singleton).
///
/// Each returned separator is sorted ascending, and the list contains no
/// duplicates. The separators themselves are returned in an arbitrary
/// order.
///
/// The implementation follows Arkady Kanevsky, "Finding all minimum-size
/// separating vertex sets in a graph", Networks 23 (1993), 533–541. It
/// computes the vertex connectivity `k`, takes the `k` largest-degree
/// vertices `X`, and for each `x ∈ X` and each non-adjacent vertex `j`
/// computes a maximum flow on the Even–Tarjan reduction; whenever the
/// flow equals `k` it enumerates all minimum `(x, j)` vertex cuts via
/// [`all_st_mincuts`], deduplicating as it goes. After each `(x, j)`
/// pair an edge `(x, j)` is added so subsequent flows discover only new
/// separators.
///
/// For undirected graphs only.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if the graph is directed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minimum_size_separators};
///
/// // Path 0-1-2: vertex 1 is the unique minimum separator.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let seps = minimum_size_separators(&g).unwrap();
/// assert_eq!(seps, vec![vec![1]]);
/// ```
#[allow(clippy::too_many_lines)]
pub fn minimum_size_separators(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "minimum_size_separators: only defined for undirected graphs".into(),
        ));
    }

    let no_of_nodes = graph.vcount();

    // Step 1: vertex connectivity k.
    let conn = vertex_connectivity(graph, true)?;

    // Special cases (three early exits). Disconnected graphs have no
    // separators; connectivity 1 ⇒ articulation points; a complete graph
    // (connectivity n-1) has every (n-1)-subset of vertices as a
    // minimum separator.
    if conn <= 0 {
        return Ok(Vec::new());
    }
    if conn == 1 {
        let aps = articulation_points(graph)?;
        return Ok(aps.into_iter().map(|v| vec![v]).collect());
    }
    if conn == i64::from(no_of_nodes) - 1 {
        let mut separators = Vec::with_capacity(no_of_nodes as usize);
        for i in 0..no_of_nodes {
            separators.push((0..no_of_nodes).filter(|&j| j != i).collect());
        }
        return Ok(separators);
    }

    let k = usize::try_from(conn).map_err(|_| {
        IgraphError::InvalidArgument("minimum_size_separators: connectivity overflow".into())
    })?;

    // Work on a simple copy of the graph (multi-edges and loops removed).
    let mut graph_copy = simplify(graph, true, true)?;

    let mut separators: Vec<Vec<VertexId>> = Vec::new();

    // Step 2: the k largest-degree vertices. If they form a separator,
    // record it.
    let x = topk_degree(&graph_copy, k)?;
    if is_separator(&graph_copy, &x)? {
        let mut sep = x.clone();
        sep.sort_unstable();
        append_unique(&mut separators, vec![sep]);
    }

    // Build Gbar, the Even–Tarjan reduction; we extend both graph_copy
    // and Gbar incrementally in step 8. igraph uses an infinite capacity
    // for the (uncuttable) original edges, but our max-flow only accepts
    // finite capacities. Replacing ∞ with `n` is exact here: the main
    // branch only runs for `2 ≤ k ≤ n-2`, so any value-`k` min-cut is
    // composed solely of unit-capacity vertex-split edges — an original
    // (or step-8-added) edge of capacity `n > k` can never appear in it.
    let reduction = even_tarjan_reduction(&graph_copy)?;
    let mut gbar = reduction.graph;
    let big = f64::from(no_of_nodes);
    let mut capacity: Vec<f64> = reduction
        .capacity
        .into_iter()
        .map(|c| if c.is_finite() { c } else { big })
        .collect();

    // Steps 3–8: for each x_i and each non-adjacent j, find all minimum
    // (x_i, j) vertex cuts of size k.
    // `conn` is in `[2, no_of_nodes - 2]` here, so it always fits in a u32.
    let k_f = f64::from(u32::try_from(conn).map_err(|_| {
        IgraphError::InvalidArgument("minimum_size_separators: connectivity overflow".into())
    })?);
    for &xi in &x {
        for j in 0..no_of_nodes {
            if xi == j {
                continue;
            }
            if are_adjacent(&graph_copy, xi, j)? {
                continue;
            }

            // Step 4: max flow in Gbar from x_i'' (= xi + n) to j'.
            let source = xi + no_of_nodes;
            let target = j;
            let phivalue = max_flow_value(&gbar, source, target, Some(&capacity))?;

            if (phivalue - k_f).abs() < 0.5 {
                // Steps 5–7: enumerate all minimum (x_i, j) cuts. Each
                // cut is a set of unit-capacity vertex-split edges; in
                // the Even–Tarjan reduction edge id e (< n) is the split
                // edge of vertex e, so cut edge ids map directly to the
                // separating vertex set.
                let cuts = all_st_mincuts(&gbar, source, target, Some(&capacity))?;
                let mapped: Vec<Vec<VertexId>> = cuts
                    .cuts
                    .into_iter()
                    .map(|cut| {
                        let mut sep: Vec<VertexId> = cut;
                        sep.sort_unstable();
                        sep
                    })
                    .collect();
                append_unique(&mut separators, mapped);
            }

            // Step 8: add edge (x_i, j) so later flows find new cuts only.
            graph_copy.add_edge(xi, j)?;
            gbar.add_edge(xi + no_of_nodes, j)?;
            gbar.add_edge(j + no_of_nodes, xi)?;
            capacity.push(big);
            capacity.push(big);
        }
    }

    Ok(separators)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_separator(&g, &[]).unwrap());
    }

    #[test]
    fn singleton_not_separator() {
        let g = Graph::with_vertices(1);
        assert!(!is_separator(&g, &[0]).unwrap());
    }

    #[test]
    fn path_middle_is_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_separator(&g, &[1]).unwrap());
    }

    #[test]
    fn path_leaf_not_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn triangle_no_single_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // No single vertex disconnects a triangle.
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[1]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn triangle_pair_not_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Removing two vertices leaves a single vertex — trivially connected.
        assert!(!is_separator(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn cycle_4_opposite_vertices() {
        // 0-1-2-3-0. {1,3} separates 0 from 2.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn cycle_4_adjacent_not_separator() {
        // 0-1-2-3-0. {0,1} does NOT disconnect (2-3 is still connected).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_separator(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn already_disconnected_empty_set_is_separator() {
        // Two components: {0,1}, {2,3}. Empty set "separates".
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_separator(&g, &[]).unwrap());
    }

    #[test]
    fn k4_articulation() {
        // K4 minus one edge: 0-1, 0-2, 0-3, 1-2, 2-3 (missing 1-3).
        // Vertex 2 is not an articulation point because 0 connects to all others.
        // Actually: adjacencies: 0→{1,2,3}, 1→{0,2}, 2→{0,1,3}, 3→{0,2}
        // Remove 0 → remaining {1,2,3}: 1-2, 2-3 → connected. Not separator.
        // Remove 2 → remaining {0,1,3}: 0-1, 0-3 → connected. Not separator.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn bowtie_articulation() {
        // Two triangles sharing vertex 2: {0,1,2} and {2,3,4}.
        // Vertex 2 is an articulation point → separator.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn directed_rejected() {
        let g = Graph::new(3, true).unwrap();
        assert!(is_separator(&g, &[0]).is_err());
        assert!(is_minimal_separator(&g, &[0]).is_err());
    }

    #[test]
    fn out_of_range_rejected() {
        let g = Graph::with_vertices(3);
        assert!(is_separator(&g, &[5]).is_err());
    }

    // --- Minimal separator tests ---

    #[test]
    fn minimal_path_middle() {
        // Path 0-1-2: {1} is a minimal separator.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_minimal_separator(&g, &[1]).unwrap());
    }

    #[test]
    fn minimal_cycle_4_opposite() {
        // 0-1-2-3-0: {1,3} is a minimal separator.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_minimal_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn not_minimal_superset() {
        // Path 0-1-2-3-4: {1,3} is a separator but NOT minimal ({1} alone suffices).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_separator(&g, &[1, 3]).unwrap());
        assert!(!is_minimal_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn not_separator_not_minimal() {
        // Triangle: {0} is not a separator → not a minimal separator.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_minimal_separator(&g, &[0]).unwrap());
    }

    #[test]
    fn minimal_bowtie_articulation() {
        // Bowtie: {2} is a minimal separator.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_minimal_separator(&g, &[2]).unwrap());
    }

    // --- all_minimal_st_separators tests ---

    #[test]
    fn all_min_sep_empty_graph() {
        let g = Graph::with_vertices(0);
        let seps = all_minimal_st_separators(&g).unwrap();
        assert!(seps.is_empty());
    }

    #[test]
    fn all_min_sep_single_vertex() {
        let g = Graph::with_vertices(1);
        let seps = all_minimal_st_separators(&g).unwrap();
        assert!(seps.is_empty());
    }

    #[test]
    fn all_min_sep_single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        // Complete graph K2 has no separators
        assert!(seps.is_empty());
    }

    #[test]
    fn all_min_sep_path_3() {
        // Path 0-1-2: {1} is the only minimal (s,t) separator
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        assert_eq!(seps.len(), 1);
        assert_eq!(seps[0], vec![1]);
    }

    #[test]
    fn all_min_sep_path_5() {
        // Path 0-1-2-3-4: separators {1}, {2}, {3}
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        assert_eq!(seps.len(), 3);
        assert!(seps.contains(&vec![1]));
        assert!(seps.contains(&vec![2]));
        assert!(seps.contains(&vec![3]));
    }

    #[test]
    fn all_min_sep_cycle_4() {
        // C4: 0-1-2-3-0. Minimal separators: {0,2} and {1,3}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        assert_eq!(seps.len(), 2);
        assert!(seps.contains(&vec![0, 2]));
        assert!(seps.contains(&vec![1, 3]));
    }

    #[test]
    fn all_min_sep_pentagon_with_chord() {
        // 0-1, 1-2, 2-3, 3-4, 4-1 (pentagon where vertex 1 connects to 0 and 4)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        // Should contain {1}, {2,4}, {1,3}
        assert_eq!(seps.len(), 3);
        assert!(seps.contains(&vec![1]));
        assert!(seps.contains(&vec![2, 4]));
        assert!(seps.contains(&vec![1, 3]));
    }

    #[test]
    fn all_min_sep_bowtie() {
        // Bowtie: triangles {0,1,2} and {2,3,4} sharing vertex 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        // Only separator is {2}
        assert_eq!(seps.len(), 1);
        assert_eq!(seps[0], vec![2]);
    }

    #[test]
    fn all_min_sep_complete_graph() {
        // K4: no vertex set can be a minimal separator
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        assert!(seps.is_empty());
    }

    #[test]
    fn all_min_sep_disconnected() {
        // Two disconnected edges: 0-1, 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let seps = all_minimal_st_separators(&g).unwrap();
        // Empty set separates. Also {0}, {1}, {2}, {3} separate.
        // But the algorithm finds minimal (s,t) separators which can include
        // empty set — however igraph convention skips empty separators.
        // Just check we don't crash and all returned are non-empty.
        for s in &seps {
            assert!(!s.is_empty());
        }
    }

    // --- Minimum-size separator tests (Kanevsky) ---

    /// Canonicalise: sort each separator ascending, then sort the list.
    fn canon(mut seps: Vec<Vec<VertexId>>) -> Vec<Vec<VertexId>> {
        for s in &mut seps {
            s.sort_unstable();
        }
        seps.sort();
        seps
    }

    fn undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_vertices(n);
        for &(a, b) in edges {
            g.add_edge(a, b).unwrap();
        }
        g
    }

    #[test]
    fn mss_directed_rejected() {
        let g = Graph::new(3, true).unwrap();
        assert!(minimum_size_separators(&g).is_err());
    }

    #[test]
    fn mss_path3() {
        // 0-1-2: vertex 1 is the unique minimum separator.
        let g = undirected(3, &[(0, 1), (1, 2)]);
        assert_eq!(canon(minimum_size_separators(&g).unwrap()), vec![vec![1]]);
    }

    #[test]
    fn mss_disconnected_empty() {
        // Already disconnected ⇒ no separators (igraph convention).
        let g = undirected(4, &[(0, 1), (2, 3)]);
        assert!(minimum_size_separators(&g).unwrap().is_empty());
    }

    #[test]
    fn mss_articulation_singletons() {
        // Bowtie: two triangles sharing vertex 2. conn == 1, the only
        // articulation point is vertex 2.
        let g = undirected(5, &[(0, 1), (1, 2), (0, 2), (2, 3), (3, 4), (2, 4)]);
        assert_eq!(canon(minimum_size_separators(&g).unwrap()), vec![vec![2]]);
    }

    #[test]
    fn mss_complete_k4() {
        // K4 (conn == n-1 == 3): every 3-subset of vertices is a minimum
        // separator.
        let g = undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        assert_eq!(
            canon(minimum_size_separators(&g).unwrap()),
            vec![vec![0, 1, 2], vec![0, 1, 3], vec![0, 2, 3], vec![1, 2, 3],]
        );
    }

    #[test]
    fn mss_complete_k3() {
        // K3 (conn == n-1 == 2): every pair is a minimum separator.
        let g = undirected(3, &[(0, 1), (0, 2), (1, 2)]);
        assert_eq!(
            canon(minimum_size_separators(&g).unwrap()),
            vec![vec![0, 1], vec![0, 2], vec![1, 2]]
        );
    }

    #[test]
    fn mss_cycle5() {
        // C5: each pair of non-adjacent vertices is a minimum 2-separator.
        let g = undirected(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]);
        assert_eq!(
            canon(minimum_size_separators(&g).unwrap()),
            vec![vec![0, 2], vec![0, 3], vec![1, 3], vec![1, 4], vec![2, 4],]
        );
    }

    #[test]
    fn mss_grid3x3() {
        // 3×3 grid; the four 2-separators are the mid-edge pairs.
        let g = undirected(
            9,
            &[
                (0, 1),
                (1, 2),
                (3, 4),
                (4, 5),
                (6, 7),
                (7, 8),
                (0, 3),
                (3, 6),
                (1, 4),
                (4, 7),
                (2, 5),
                (5, 8),
            ],
        );
        assert_eq!(
            canon(minimum_size_separators(&g).unwrap()),
            vec![vec![1, 3], vec![1, 5], vec![3, 7], vec![5, 7]]
        );
    }

    #[test]
    fn mss_complete_bipartite_k23() {
        // K_{2,3}: the two-vertex side {0,1} is the unique minimum
        // separator.
        let g = undirected(5, &[(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)]);
        assert_eq!(
            canon(minimum_size_separators(&g).unwrap()),
            vec![vec![0, 1]]
        );
    }

    #[test]
    fn mss_separators_are_valid() {
        // Every returned set must actually separate the graph, and all
        // must share the same (minimum) cardinality.
        let g = undirected(
            7,
            &[
                (0, 1),
                (1, 2),
                (2, 0),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 3),
                (5, 6),
                (6, 3),
            ],
        );
        let seps = minimum_size_separators(&g).unwrap();
        assert!(!seps.is_empty());
        let k = seps[0].len();
        for s in &seps {
            assert_eq!(s.len(), k, "all minimum separators share size");
            assert!(is_separator(&g, s).unwrap(), "{s:?} must separate");
        }
    }
}
