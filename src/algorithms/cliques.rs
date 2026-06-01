//! Clique algorithms (ALGO-CL-001).
//!
//! Bron-Kerbosch algorithm with pivot for enumerating maximal cliques,
//! plus convenience functions for clique number and largest cliques.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns the clique number of the graph (size of the largest clique).
///
/// A clique is a complete subgraph. The clique number is the vertex count
/// of the largest clique. For a graph with no edges, the clique number is 1
/// (each vertex is a clique of size 1). For an empty graph (no vertices),
/// returns 0.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clique_number};
///
/// // K4 (complete graph on 4 vertices)
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert_eq!(clique_number(&g).unwrap(), 4);
/// ```
pub fn clique_number(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let adj = build_neighbor_set(graph)?;
    let mut max_size: u32 = 0;

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_max(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut max_size,
    );

    Ok(max_size)
}

/// Returns all maximal cliques in the graph.
///
/// A maximal clique is a clique that cannot be extended by adding another
/// adjacent vertex. Uses the Bron-Kerbosch algorithm with pivoting.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let cliques = maximal_cliques(&g).unwrap();
/// // Path graph: each edge is a maximal clique
/// assert_eq!(cliques.len(), 3);
/// ```
pub fn maximal_cliques(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_neighbor_set(graph)?;
    let mut result: Vec<Vec<VertexId>> = Vec::new();

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_all(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut result,
    );

    // Include isolated vertices as cliques of size 1
    for v in 0..n {
        if adj[v as usize].is_empty() {
            result.push(vec![v]);
        }
    }

    Ok(result)
}

/// Returns only the largest cliques in the graph.
///
/// A largest clique is a maximal clique whose size equals the clique
/// number. There may be multiple largest cliques.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, largest_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let cliques = largest_cliques(&g).unwrap();
/// // The triangle {0,1,2} is the only largest clique
/// assert_eq!(cliques.len(), 1);
/// assert_eq!(cliques[0].len(), 3);
/// ```
pub fn largest_cliques(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let all = maximal_cliques(graph)?;
    let max_size = all.iter().map(Vec::len).max().unwrap_or(0);
    Ok(all.into_iter().filter(|c| c.len() == max_size).collect())
}

/// Returns the independence number of the graph (size of the largest
/// independent set).
///
/// An independent set is a set of vertices with no edges between them.
/// Equivalently, it is a clique in the complement graph. For a graph
/// with no edges, the independence number equals the vertex count.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, independence_number};
///
/// // Path 0-1-2: largest independent set is {0, 2}
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert_eq!(independence_number(&g).unwrap(), 2);
/// ```
pub fn independence_number(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let adj = build_complement_neighbor_set(graph)?;
    let mut max_size: u32 = 0;

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_max(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut max_size,
    );

    Ok(max_size)
}

/// Returns all maximal independent vertex sets in the graph.
///
/// A maximal independent set cannot be extended by adding another vertex
/// without creating an edge within the set. Uses Bron-Kerbosch on the
/// complement adjacency.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_independent_vertex_sets};
///
/// // Triangle: each single vertex is a maximal independent set
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
///
/// let sets = maximal_independent_vertex_sets(&g).unwrap();
/// assert_eq!(sets.len(), 3);
/// for s in &sets {
///     assert_eq!(s.len(), 1);
/// }
/// ```
pub fn maximal_independent_vertex_sets(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_complement_neighbor_set(graph)?;
    let mut result: Vec<Vec<VertexId>> = Vec::new();

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_all_independent(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut result,
    );

    Ok(result)
}

/// Returns only the largest independent vertex sets in the graph.
///
/// A largest independent set is a maximal independent set whose size
/// equals the independence number.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, largest_independent_vertex_sets};
///
/// // Path 0-1-2-3: largest independent sets are {0,2}, {0,3}, {1,3}
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let sets = largest_independent_vertex_sets(&g).unwrap();
/// assert_eq!(sets.len(), 3);
/// for s in &sets {
///     assert_eq!(s.len(), 2);
/// }
/// ```
pub fn largest_independent_vertex_sets(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let all = maximal_independent_vertex_sets(graph)?;
    let max_size = all.iter().map(Vec::len).max().unwrap_or(0);
    Ok(all.into_iter().filter(|s| s.len() == max_size).collect())
}

/// Bron-Kerbosch with pivot — collects all maximal cliques (independent sets).
/// Unlike `bron_kerbosch_all`, this includes cliques of size 1 (isolated
/// vertices in the complement = vertices with all edges in the original).
fn bron_kerbosch_all_independent(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    result: &mut Vec<Vec<VertexId>>,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        let mut clique = r_clique.clone();
        clique.sort_unstable();
        result.push(clique);
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_all_independent(adj, r_clique, &mut new_p, &mut new_x, result);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Build complement neighbor lists (non-neighbors, ignoring self-loops).
fn build_complement_neighbor_set(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    let adj = build_neighbor_set(graph)?;

    let mut complement: Vec<Vec<VertexId>> = Vec::with_capacity(n as usize);
    for v in 0..n {
        let neighbors = &adj[v as usize];
        let non_neighbors: Vec<VertexId> = (0..n)
            .filter(|&u| u != v && !neighbors.contains(&u))
            .collect();
        complement.push(non_neighbors);
    }

    Ok(complement)
}

/// Bron-Kerbosch with pivot — only tracks maximum clique size.
fn bron_kerbosch_max(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    max_size: &mut u32,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        #[allow(clippy::cast_possible_truncation)]
        let size = r_clique.len() as u32;
        if size > *max_size {
            *max_size = size;
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    // Choose pivot: vertex in P ∪ X with most neighbors in P
    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    // Vertices in P that are NOT neighbors of pivot
    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_max(adj, r_clique, &mut new_p, &mut new_x, max_size);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Bron-Kerbosch with pivot — collects all maximal cliques.
fn bron_kerbosch_all(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    result: &mut Vec<Vec<VertexId>>,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        if r_clique.len() >= 2 {
            let mut clique = r_clique.clone();
            clique.sort_unstable();
            result.push(clique);
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_all(adj, r_clique, &mut new_p, &mut new_x, result);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Counts the number of maximal cliques without storing them.
///
/// More memory-efficient than [`maximal_cliques`] when only the count
/// is needed.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_cliques_count};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// // Path graph: 3 maximal cliques (each edge) + 0 isolated
/// assert_eq!(maximal_cliques_count(&g).unwrap(), 3);
/// ```
pub fn maximal_cliques_count(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let adj = build_neighbor_set(graph)?;
    let mut count: u64 = 0;

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_count(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut count,
    );

    // Include isolated vertices as cliques of size 1
    for v in 0..n {
        if adj[v as usize].is_empty() {
            count = count.saturating_add(1);
        }
    }

    Ok(count)
}

/// Returns a histogram of clique sizes, counting **all** cliques.
///
/// A clique is any complete subgraph (not necessarily maximal). The result
/// is a vector where index `k` holds the number of cliques of size `k`,
/// including every single vertex (size 1) and every edge (size 2). Index 0
/// is always 0 (no cliques of size 0 exist). The vector length is
/// `clique_number + 1`.
///
/// This is the counterpart of igraph's `igraph_clique_size_hist`. Note that
/// igraph stores the size-1 count in element 0; here index equals the clique
/// size with element 0 left as a zero placeholder, matching the convention
/// of the rest of this crate. For maximal cliques only, see
/// [`maximal_cliques_hist`].
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clique_size_hist};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let hist = clique_size_hist(&g).unwrap();
/// // 4 vertices, 4 edges, 1 triangle {0,1,2}
/// assert_eq!(hist, vec![0, 4, 4, 1]);
/// ```
pub fn clique_size_hist(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_neighbor_set(graph)?;
    let mut hist: Vec<u64> = Vec::new();

    let mut current: Vec<VertexId> = Vec::new();
    count_cliques_by_size(&adj, &mut current, 0, n, &mut hist);

    Ok(hist)
}

/// Returns a histogram of **maximal** clique sizes.
///
/// A maximal clique cannot be extended by adding an adjacent vertex. The
/// result is a vector where index `k` holds the number of maximal cliques of
/// size `k`. Maximal cliques of size 1 are exactly the isolated vertices.
/// Index 0 is always 0. The vector length is `clique_number + 1`.
///
/// This is the counterpart of igraph's `igraph_maximal_cliques_hist`. As with
/// [`clique_size_hist`], index equals the clique size (element 0 is a zero
/// placeholder), whereas igraph stores the size-1 count in element 0. To count
/// all cliques (not just maximal ones), use [`clique_size_hist`].
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_cliques_hist};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let hist = maximal_cliques_hist(&g).unwrap();
/// // maximal cliques: edge {2,3} (size 2), triangle {0,1,2} (size 3)
/// assert_eq!(hist, vec![0, 0, 1, 1]);
/// ```
pub fn maximal_cliques_hist(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_neighbor_set(graph)?;
    let mut hist: Vec<u64> = Vec::new();

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_hist(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut hist,
    );

    // A size-1 maximal clique is exactly an isolated vertex.
    for v in 0..n {
        if adj[v as usize].is_empty() {
            if hist.len() < 2 {
                hist.resize(2, 0);
            }
            hist[1] = hist[1].saturating_add(1);
        }
    }

    Ok(hist)
}

/// Returns maximal cliques containing at least one vertex from `subset`.
///
/// Enumerates all maximal cliques of the graph, then filters to keep
/// only those that contain at least one vertex from the given subset.
/// Optional `min_size` / `max_size` bounds filter by clique size, and
/// `max_results` caps output count.
///
/// Edge directions are ignored for directed graphs.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `subset` — vertex IDs to seed the search. A clique is included if
///   it contains at least one vertex from this set.
/// * `min_size` — minimum clique size (0 = no bound).
/// * `max_size` — maximum clique size (0 = no bound).
/// * `max_results` — maximum number of cliques to return (`None` = unlimited).
///
/// # Errors
///
/// Returns `InvalidArgument` if any vertex in `subset` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_cliques_subset};
///
/// // Triangle {0,1,2} plus edge {2,3}
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// // Cliques touching vertex 3: only {2,3}
/// let cliques = maximal_cliques_subset(&g, &[3], 0, 0, None).unwrap();
/// assert_eq!(cliques.len(), 1);
/// assert_eq!(cliques[0].len(), 2);
///
/// // Cliques touching vertex 0: only {0,1,2}
/// let cliques = maximal_cliques_subset(&g, &[0], 0, 0, None).unwrap();
/// assert_eq!(cliques.len(), 1);
/// assert_eq!(cliques[0].len(), 3);
/// ```
pub fn maximal_cliques_subset(
    graph: &Graph,
    subset: &[VertexId],
    min_size: u32,
    max_size: u32,
    max_results: Option<usize>,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();

    for &v in subset {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex id {v} is out of range [0, {n})"
            )));
        }
    }

    if n == 0 || subset.is_empty() {
        return Ok(Vec::new());
    }

    let adj = build_neighbor_set(graph)?;
    let mut all_cliques: Vec<Vec<VertexId>> = Vec::new();

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_all(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut all_cliques,
    );

    // Include isolated vertices as cliques of size 1
    for v in 0..n {
        if adj[v as usize].is_empty() {
            all_cliques.push(vec![v]);
        }
    }

    // Build a fast lookup set for subset membership
    let mut in_subset = vec![false; n as usize];
    for &v in subset {
        in_subset[v as usize] = true;
    }

    let effective_min = if min_size == 0 { 1 } else { min_size };
    let effective_max = if max_size == 0 { u32::MAX } else { max_size };
    let limit = max_results.unwrap_or(usize::MAX);

    let mut result: Vec<Vec<VertexId>> = Vec::new();
    for clique in all_cliques {
        #[allow(clippy::cast_possible_truncation)]
        let csize = clique.len() as u32;
        if csize < effective_min || csize > effective_max {
            continue;
        }
        if !clique.iter().any(|&v| in_subset[v as usize]) {
            continue;
        }
        result.push(clique);
        if result.len() >= limit {
            break;
        }
    }

    Ok(result)
}

/// Tally every clique (complete subgraph) by size into `hist[size]`.
fn count_cliques_by_size(
    adj: &[Vec<VertexId>],
    current: &mut Vec<VertexId>,
    start: u32,
    n: u32,
    hist: &mut Vec<u64>,
) {
    let len = current.len();
    if len >= 1 {
        if hist.len() <= len {
            hist.resize(len + 1, 0);
        }
        hist[len] = hist[len].saturating_add(1);
    }

    for v in start..n {
        let v_adj = &adj[v as usize];
        if current.iter().all(|&u| v_adj.contains(&u)) {
            current.push(v);
            count_cliques_by_size(adj, current, v + 1, n, hist);
            current.pop();
        }
    }
}

/// Bron-Kerbosch with pivot — count-only variant.
fn bron_kerbosch_count(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    count: &mut u64,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        if r_clique.len() >= 2 {
            *count = count.saturating_add(1);
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_count(adj, r_clique, &mut new_p, &mut new_x, count);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Bron-Kerbosch with pivot — histogram variant.
fn bron_kerbosch_hist(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    hist: &mut Vec<u64>,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        let size = r_clique.len();
        if size >= 2 {
            if hist.len() <= size {
                hist.resize(size + 1, 0);
            }
            hist[size] = hist[size].saturating_add(1);
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_hist(adj, r_clique, &mut new_p, &mut new_x, hist);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Choose pivot vertex with maximum connections to P.
fn choose_pivot(
    adj: &[Vec<VertexId>],
    p_candidates: &[VertexId],
    x_excluded: &[VertexId],
) -> VertexId {
    let mut best = p_candidates[0];
    let mut best_count = 0usize;

    for &v in p_candidates.iter().chain(x_excluded.iter()) {
        let count = p_candidates
            .iter()
            .filter(|&&u| adj[v as usize].contains(&u))
            .count();
        if count > best_count {
            best_count = count;
            best = v;
        }
    }

    best
}

/// Finds all cliques (complete subgraphs) in the graph within a size range.
///
/// Returns all cliques of size `min_size..=max_size`. Unlike
/// [`maximal_cliques`], this includes non-maximal cliques (subsets of
/// larger cliques). If `max_results` is `Some(n)`, at most `n` cliques
/// are returned.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cliques};
///
/// // Triangle 0-1-2: contains 3 edges (cliques of size 2) + 1 triangle (size 3)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let all = cliques(&g, 2, 3, None).unwrap();
/// assert_eq!(all.len(), 4); // 3 edges + 1 triangle
/// ```
pub fn cliques(
    graph: &Graph,
    min_size: u32,
    max_size: u32,
    max_results: Option<usize>,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 || min_size > n || max_size == 0 {
        return Ok(Vec::new());
    }

    let min_sz = if min_size == 0 { 1 } else { min_size } as usize;
    let max_sz = max_size as usize;
    let limit = max_results.unwrap_or(usize::MAX);

    let adj = build_neighbor_set(graph)?;
    let mut result: Vec<Vec<VertexId>> = Vec::new();

    // Enumerate all cliques using backtracking: for each vertex v (in
    // order), extend the current clique with v if v is adjacent to all
    // vertices already in the clique.
    let mut current: Vec<VertexId> = Vec::new();
    enumerate_cliques(&adj, &mut current, 0, n, min_sz, max_sz, limit, &mut result);

    Ok(result)
}

/// Returns the weight of the maximum-weight clique.
///
/// The weight of a clique is the sum of its vertex weights. Only positive
/// integer vertex weights are supported; following igraph, each weight is
/// truncated to its integer part. Because every weight is positive, the
/// maximum-weight clique is always maximal, so the search ranges over
/// maximal cliques.
///
/// `vertex_weights` must have one entry per vertex. For an empty graph the
/// weight is `0.0`. This is the counterpart of igraph's
/// `igraph_weighted_clique_number`.
///
/// Edge directions are ignored for directed graphs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `vertex_weights` has the wrong
/// length or any (truncated) weight is not a positive integer.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, weighted_clique_number};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// // {2,3} has weight 1 + 5 = 6, beating the triangle's weight 3.
/// let w = weighted_clique_number(&g, &[1.0, 1.0, 1.0, 5.0]).unwrap();
/// assert!((w - 6.0).abs() < 1e-9);
/// ```
// Weights are truncated positive integers; igraph returns the result as a
// double, so the i64 -> f64 widening is intentional.
#[allow(clippy::cast_precision_loss)]
pub fn weighted_clique_number(graph: &Graph, vertex_weights: &[f64]) -> IgraphResult<f64> {
    let n = graph.vcount();
    let weights = validate_clique_weights(vertex_weights, n)?;
    if n == 0 {
        return Ok(0.0);
    }

    let mut best: i64 = 0;
    for clique in maximal_cliques(graph)? {
        let w = clique_weight(&weights, &clique)?;
        if w > best {
            best = w;
        }
    }
    Ok(best as f64)
}

/// Returns every clique of maximum total weight.
///
/// The weight of a clique is the sum of its vertex weights (positive integer
/// weights, truncated to their integer parts). There may be several cliques
/// tying for the largest weight; all of them are returned, each sorted
/// ascending, and the list itself is sorted lexicographically. Because all
/// weights are positive, every maximum-weight clique is maximal.
///
/// This is the counterpart of igraph's `igraph_largest_weighted_cliques`.
///
/// Edge directions are ignored for directed graphs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `vertex_weights` has the wrong
/// length or any (truncated) weight is not a positive integer.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, largest_weighted_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cs = largest_weighted_cliques(&g, &[1.0, 1.0, 1.0, 5.0]).unwrap();
/// assert_eq!(cs, vec![vec![2, 3]]);
/// ```
pub fn largest_weighted_cliques(
    graph: &Graph,
    vertex_weights: &[f64],
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    let weights = validate_clique_weights(vertex_weights, n)?;
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut best: i64 = 0;
    let mut weighted: Vec<(i64, Vec<VertexId>)> = Vec::new();
    for mut clique in maximal_cliques(graph)? {
        clique.sort_unstable();
        let w = clique_weight(&weights, &clique)?;
        if w > best {
            best = w;
        }
        weighted.push((w, clique));
    }

    let mut result: Vec<Vec<VertexId>> = weighted
        .into_iter()
        .filter(|(w, _)| *w == best)
        .map(|(_, clique)| clique)
        .collect();
    result.sort_unstable();
    Ok(result)
}

/// Finds all cliques whose total weight lies in a given range.
///
/// The weight of a clique is the sum of its vertex weights (positive integer
/// weights, truncated to their integer parts). When `maximal` is `true`, only
/// maximal cliques are considered; otherwise every clique is. A clique is
/// kept when its weight is `>= min_weight` (unless `min_weight <= 0`, meaning
/// no lower bound) and `<= max_weight` (unless `max_weight <= 0`, meaning no
/// upper bound). If `max_results` is `Some(n)`, at most `n` cliques are
/// returned. Results are sorted for determinism.
///
/// This is the counterpart of igraph's `igraph_weighted_cliques`.
///
/// Edge directions are ignored for directed graphs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `vertex_weights` has the wrong
/// length or any (truncated) weight is not a positive integer.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, weighted_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// // All maximal cliques, unbounded weight range.
/// let cs = weighted_cliques(&g, &[1.0, 1.0, 1.0, 1.0], true, 0.0, 0.0, None).unwrap();
/// assert_eq!(cs, vec![vec![0, 1, 2], vec![2, 3]]);
/// ```
// Clique weight (truncated positive integers) is compared against the f64
// weight bounds; the i64 -> f64 widening is intentional.
#[allow(clippy::cast_precision_loss)]
pub fn weighted_cliques(
    graph: &Graph,
    vertex_weights: &[f64],
    maximal: bool,
    min_weight: f64,
    max_weight: f64,
    max_results: Option<usize>,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    let weights = validate_clique_weights(vertex_weights, n)?;
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut candidates = if maximal {
        let mut cs = maximal_cliques(graph)?;
        for clique in &mut cs {
            clique.sort_unstable();
        }
        cs
    } else {
        cliques(graph, 1, n, None)?
    };
    candidates.sort_unstable();

    let has_min = min_weight > 0.0;
    let has_max = max_weight > 0.0;
    let limit = max_results.unwrap_or(usize::MAX);

    let mut result: Vec<Vec<VertexId>> = Vec::new();
    for clique in candidates {
        let w = clique_weight(&weights, &clique)? as f64;
        if (has_min && w < min_weight) || (has_max && w > max_weight) {
            continue;
        }
        result.push(clique);
        if result.len() >= limit {
            break;
        }
    }
    Ok(result)
}

/// Validate vertex-weight slice and truncate to positive integer weights.
// The f64 -> i64 conversion is range-guarded (1 <= w <= i64::MAX), and the
// i64::MAX -> f64 bound check is the standard idiom for that guard.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn validate_clique_weights(vertex_weights: &[f64], n: u32) -> IgraphResult<Vec<i64>> {
    if vertex_weights.len() != n as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "vertex_weights length {} does not match vertex count {n}",
            vertex_weights.len()
        )));
    }
    let mut weights = Vec::with_capacity(vertex_weights.len());
    for (v, &w) in vertex_weights.iter().enumerate() {
        let truncated = w.trunc();
        if !truncated.is_finite() || truncated < 1.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weighted cliques require positive integer vertex weights; vertex {v} has weight {w}"
            )));
        }
        if truncated > i64::MAX as f64 {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex weight for vertex {v} is too large"
            )));
        }
        weights.push(truncated as i64);
    }
    Ok(weights)
}

/// Sum vertex weights over a clique with overflow checking.
fn clique_weight(weights: &[i64], clique: &[VertexId]) -> IgraphResult<i64> {
    let mut total: i64 = 0;
    for &v in clique {
        total = total
            .checked_add(weights[v as usize])
            .ok_or(IgraphError::Internal("clique weight sum overflows i64"))?;
    }
    Ok(total)
}

/// Finds all independent vertex sets within a size range.
///
/// An independent set is a set of vertices with no edges between them.
/// Returns all independent sets of size `min_size..=max_size`.
/// If `max_results` is `Some(n)`, at most `n` sets are returned.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, independent_vertex_sets};
///
/// // Path 0-1-2: independent sets of size 2 are {0,2}
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let sets = independent_vertex_sets(&g, 2, 3, None).unwrap();
/// assert_eq!(sets.len(), 1);
/// assert_eq!(sets[0], vec![0, 2]);
/// ```
pub fn independent_vertex_sets(
    graph: &Graph,
    min_size: u32,
    max_size: u32,
    max_results: Option<usize>,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 || min_size > n || max_size == 0 {
        return Ok(Vec::new());
    }

    let min_sz = if min_size == 0 { 1 } else { min_size } as usize;
    let max_sz = max_size as usize;
    let limit = max_results.unwrap_or(usize::MAX);

    let adj = build_complement_neighbor_set(graph)?;
    let mut result: Vec<Vec<VertexId>> = Vec::new();

    let mut current: Vec<VertexId> = Vec::new();
    enumerate_cliques(&adj, &mut current, 0, n, min_sz, max_sz, limit, &mut result);

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn enumerate_cliques(
    adj: &[Vec<VertexId>],
    current: &mut Vec<VertexId>,
    start: u32,
    n: u32,
    min_sz: usize,
    max_sz: usize,
    limit: usize,
    result: &mut Vec<Vec<VertexId>>,
) {
    let cur_len = current.len();

    if cur_len >= min_sz {
        let mut clique = current.clone();
        clique.sort_unstable();
        result.push(clique);
        if result.len() >= limit {
            return;
        }
    }

    if cur_len >= max_sz {
        return;
    }

    for v in start..n {
        // v must be adjacent to all vertices in current.
        let v_adj = &adj[v as usize];
        let all_connected = current.iter().all(|&u| v_adj.contains(&u));
        if !all_connected {
            continue;
        }

        current.push(v);
        enumerate_cliques(adj, current, v + 1, n, min_sz, max_sz, limit, result);
        current.pop();

        if result.len() >= limit {
            return;
        }
    }
}

/// Build undirected neighbor lists (ignoring edge direction).
fn build_neighbor_set(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            adj[src as usize].push(tgt);
            adj[tgt as usize].push(src);
        }
    }

    // Deduplicate (handles multi-edges)
    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clique_number_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(clique_number(&g).unwrap(), 0);
    }

    #[test]
    fn test_clique_number_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(clique_number(&g).unwrap(), 1);
    }

    #[test]
    fn test_clique_number_single_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(clique_number(&g).unwrap(), 2);
    }

    #[test]
    fn test_clique_number_triangle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(clique_number(&g).unwrap(), 3);
    }

    #[test]
    fn test_clique_number_k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(clique_number(&g).unwrap(), 5);
    }

    #[test]
    fn test_clique_number_directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Triangle when ignoring directions
        assert_eq!(clique_number(&g).unwrap(), 3);
    }

    #[test]
    fn test_maximal_cliques_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cliques = maximal_cliques(&g).unwrap();
        assert_eq!(cliques.len(), 3);
        for c in &cliques {
            assert_eq!(c.len(), 2);
        }
    }

    #[test]
    fn test_maximal_cliques_triangle_plus_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cliques = maximal_cliques(&g).unwrap();
        // {0,1,2} and {2,3}
        assert_eq!(cliques.len(), 2);
        let sizes: Vec<usize> = cliques.iter().map(Vec::len).collect();
        assert!(sizes.contains(&3));
        assert!(sizes.contains(&2));
    }

    #[test]
    fn test_maximal_cliques_isolated_vertices() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        // vertices 2, 3, 4 are isolated

        let cliques = maximal_cliques(&g).unwrap();
        // {0,1} + 3 isolated = 4 cliques
        assert_eq!(cliques.len(), 4);
    }

    #[test]
    fn test_largest_cliques() {
        let mut g = Graph::with_vertices(6);
        // Two triangles sharing an edge
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(2, 4).unwrap();

        let cliques = largest_cliques(&g).unwrap();
        // Two triangles: {0,1,2} and {2,3,4}
        assert_eq!(cliques.len(), 2);
        for c in &cliques {
            assert_eq!(c.len(), 3);
        }
    }

    #[test]
    fn test_largest_cliques_k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }

        let cliques = largest_cliques(&g).unwrap();
        assert_eq!(cliques.len(), 1);
        assert_eq!(cliques[0].len(), 4);
    }

    #[test]
    fn test_clique_number_petersen() {
        // Petersen graph has clique number 2 (no triangles)
        let mut g = Graph::with_vertices(10);
        // Outer 5-cycle
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Inner pentagram
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        // Spokes
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();

        assert_eq!(clique_number(&g).unwrap(), 2);
    }

    // --- Independent vertex set tests ---

    #[test]
    fn test_independence_number_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(independence_number(&g).unwrap(), 0);
    }

    #[test]
    fn test_independence_number_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(independence_number(&g).unwrap(), 5);
    }

    #[test]
    fn test_independence_number_complete() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(independence_number(&g).unwrap(), 1);
    }

    #[test]
    fn test_independence_number_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Largest independent set: {0,2} or {0,3} or {1,3} — all size 2
        assert_eq!(independence_number(&g).unwrap(), 2);
    }

    #[test]
    fn test_independence_number_cycle_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // C5: independence number = 2
        assert_eq!(independence_number(&g).unwrap(), 2);
    }

    #[test]
    fn test_independence_number_bipartite() {
        // K_{2,3}: independence number = max(2, 3) = 3
        let mut g = Graph::with_vertices(5);
        for &a in &[0u32, 1] {
            for &b in &[2u32, 3, 4] {
                g.add_edge(a, b).unwrap();
            }
        }
        assert_eq!(independence_number(&g).unwrap(), 3);
    }

    #[test]
    fn test_maximal_independent_sets_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();

        let sets = maximal_independent_vertex_sets(&g).unwrap();
        // Each single vertex is a maximal independent set in K3
        assert_eq!(sets.len(), 3);
        for s in &sets {
            assert_eq!(s.len(), 1);
        }
    }

    #[test]
    fn test_maximal_independent_sets_path_4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let sets = maximal_independent_vertex_sets(&g).unwrap();
        // Maximal independent sets: {0,2}, {0,3}, {1,3}
        assert_eq!(sets.len(), 3);
        for s in &sets {
            assert_eq!(s.len(), 2);
        }
    }

    #[test]
    fn test_largest_independent_vertex_sets_star() {
        // Star K_{1,4}: center connected to all leaves
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();

        let sets = largest_independent_vertex_sets(&g).unwrap();
        // Largest independent set: {1,2,3,4} (all leaves)
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0].len(), 4);
    }

    #[test]
    fn test_independence_number_petersen() {
        // Petersen graph has independence number 4
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();

        assert_eq!(independence_number(&g).unwrap(), 4);
    }

    #[test]
    fn test_independent_sets_directed_ignores_direction() {
        // Directed triangle: independence number = 1
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(independence_number(&g).unwrap(), 1);
    }

    // --- maximal_cliques_count tests ---

    #[test]
    fn test_count_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(maximal_cliques_count(&g).unwrap(), 0);
    }

    #[test]
    fn test_count_isolated() {
        let g = Graph::with_vertices(3);
        // 3 isolated vertices → 3 cliques of size 1
        assert_eq!(maximal_cliques_count(&g).unwrap(), 3);
    }

    #[test]
    fn test_count_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Path: 3 maximal cliques (each edge)
        assert_eq!(maximal_cliques_count(&g).unwrap(), 3);
    }

    #[test]
    fn test_count_triangle_plus_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Triangle {0,1,2} + edge {2,3} = 2 maximal cliques
        assert_eq!(maximal_cliques_count(&g).unwrap(), 2);
    }

    #[test]
    fn test_count_matches_enum() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let enumerated = maximal_cliques(&g).unwrap();
        assert_eq!(maximal_cliques_count(&g).unwrap(), enumerated.len() as u64);
    }

    // --- clique_size_hist tests ---

    #[test]
    fn test_hist_empty() {
        let g = Graph::with_vertices(0);
        assert!(clique_size_hist(&g).unwrap().is_empty());
    }

    #[test]
    fn test_hist_isolated() {
        let g = Graph::with_vertices(3);
        let hist = clique_size_hist(&g).unwrap();
        // 3 cliques of size 1
        assert_eq!(hist, vec![0, 3]);
    }

    #[test]
    fn test_hist_triangle_plus_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let hist = clique_size_hist(&g).unwrap();
        // all cliques: 4 vertices, 4 edges, 1 triangle
        assert_eq!(hist, vec![0, 4, 4, 1]);
    }

    #[test]
    fn test_hist_complete_4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let hist = clique_size_hist(&g).unwrap();
        // K4: C(4,1)=4, C(4,2)=6, C(4,3)=4, C(4,4)=1
        assert_eq!(hist, vec![0, 4, 6, 4, 1]);
    }

    #[test]
    fn test_hist_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let hist = clique_size_hist(&g).unwrap();
        // 4 vertices, 3 edges, no triangles
        assert_eq!(hist, vec![0, 4, 3]);
    }

    #[test]
    fn test_hist_matches_cliques_enumeration() {
        let mut g = Graph::with_vertices(6);
        for &(a, b) in &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)] {
            g.add_edge(a, b).unwrap();
        }
        let hist = clique_size_hist(&g).unwrap();
        // Cross-check against the all-cliques enumerator.
        let all = cliques(&g, 1, g.vcount(), None).unwrap();
        let mut expected = vec![0u64; hist.len()];
        for c in &all {
            expected[c.len()] += 1;
        }
        assert_eq!(hist, expected);
    }

    // --- maximal_cliques_hist tests ---

    #[test]
    fn test_max_hist_empty() {
        let g = Graph::with_vertices(0);
        assert!(maximal_cliques_hist(&g).unwrap().is_empty());
    }

    #[test]
    fn test_max_hist_isolated() {
        let g = Graph::with_vertices(3);
        let hist = maximal_cliques_hist(&g).unwrap();
        // 3 size-1 maximal cliques (isolated vertices)
        assert_eq!(hist, vec![0, 3]);
    }

    #[test]
    fn test_max_hist_triangle_plus_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let hist = maximal_cliques_hist(&g).unwrap();
        // maximal: edge {2,3} (size 2), triangle {0,1,2} (size 3)
        assert_eq!(hist, vec![0, 0, 1, 1]);
    }

    #[test]
    fn test_max_hist_complete_4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let hist = maximal_cliques_hist(&g).unwrap();
        // K4: single maximal clique of size 4
        assert_eq!(hist, vec![0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_max_hist_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let hist = maximal_cliques_hist(&g).unwrap();
        // 3 maximal cliques, each an edge
        assert_eq!(hist, vec![0, 0, 3]);
    }

    #[test]
    fn test_max_hist_total_matches_count() {
        let mut g = Graph::with_vertices(6);
        for &(a, b) in &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)] {
            g.add_edge(a, b).unwrap();
        }
        let hist = maximal_cliques_hist(&g).unwrap();
        let total: u64 = hist.iter().sum();
        assert_eq!(total, maximal_cliques_count(&g).unwrap());
    }

    // --- weighted clique tests ---

    fn triangle_plus_pendant() -> Graph {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g
    }

    #[test]
    fn test_weighted_clique_number_pendant_wins() {
        let g = triangle_plus_pendant();
        let w = weighted_clique_number(&g, &[1.0, 1.0, 1.0, 5.0]).unwrap();
        assert!((w - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_clique_number_triangle_wins() {
        let g = triangle_plus_pendant();
        // Heavy triangle vertices outweigh the pendant edge.
        let w = weighted_clique_number(&g, &[3.0, 3.0, 3.0, 1.0]).unwrap();
        assert!((w - 9.0).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_clique_number_unit_weights_equals_clique_number() {
        let g = triangle_plus_pendant();
        let w = weighted_clique_number(&g, &[1.0, 1.0, 1.0, 1.0]).unwrap();
        assert!((w - f64::from(clique_number(&g).unwrap())).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_clique_number_truncates() {
        let g = triangle_plus_pendant();
        // 2.9 truncates to 2, so {2,3} weighs 2 + 2 = 4 < triangle's 6.
        let w = weighted_clique_number(&g, &[2.9, 2.9, 2.9, 2.9]).unwrap();
        assert!((w - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_weighted_clique_number_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(weighted_clique_number(&g, &[]).unwrap().abs() < 1e-9);
    }

    #[test]
    fn test_weighted_clique_number_isolated_vertices() {
        let g = Graph::with_vertices(3);
        let w = weighted_clique_number(&g, &[2.0, 7.0, 3.0]).unwrap();
        assert!((w - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_largest_weighted_cliques_single_winner() {
        let g = triangle_plus_pendant();
        let cs = largest_weighted_cliques(&g, &[1.0, 1.0, 1.0, 5.0]).unwrap();
        assert_eq!(cs, vec![vec![2, 3]]);
    }

    #[test]
    fn test_largest_weighted_cliques_tie() {
        // Two disjoint edges with equal weight tie for the maximum.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let cs = largest_weighted_cliques(&g, &[1.0, 1.0, 1.0, 1.0]).unwrap();
        assert_eq!(cs, vec![vec![0, 1], vec![2, 3]]);
    }

    #[test]
    fn test_weighted_cliques_all_maximal() {
        let g = triangle_plus_pendant();
        let cs = weighted_cliques(&g, &[1.0, 1.0, 1.0, 1.0], true, 0.0, 0.0, None).unwrap();
        assert_eq!(cs, vec![vec![0, 1, 2], vec![2, 3]]);
    }

    #[test]
    fn test_weighted_cliques_min_weight_filter() {
        let g = triangle_plus_pendant();
        // Only cliques with weight >= 3 among all (non-maximal) cliques.
        let cs = weighted_cliques(&g, &[1.0, 1.0, 1.0, 5.0], false, 3.0, 0.0, None).unwrap();
        // Cliques of weight >= 3: {2,3}(6), {0,1,2}(3); edges {0,2},{1,2}=2, etc. excluded.
        assert!(cs.contains(&vec![2, 3]));
        assert!(cs.contains(&vec![0, 1, 2]));
        for c in &cs {
            let total: f64 = c.iter().map(|&v| [1.0, 1.0, 1.0, 5.0][v as usize]).sum();
            assert!(total >= 3.0);
        }
    }

    #[test]
    fn test_weighted_cliques_max_weight_filter() {
        let g = triangle_plus_pendant();
        // All cliques of weight <= 2 with unit weights: singles (1) and edges (2).
        let cs = weighted_cliques(&g, &[1.0, 1.0, 1.0, 1.0], false, 0.0, 2.0, None).unwrap();
        for c in &cs {
            assert!(c.len() <= 2);
        }
        assert!(cs.contains(&vec![0]));
        assert!(cs.contains(&vec![0, 1]));
        assert!(!cs.contains(&vec![0, 1, 2]));
    }

    #[test]
    fn test_weighted_cliques_max_results() {
        let g = triangle_plus_pendant();
        let cs = weighted_cliques(&g, &[1.0, 1.0, 1.0, 1.0], false, 0.0, 0.0, Some(2)).unwrap();
        assert_eq!(cs.len(), 2);
    }

    #[test]
    fn test_weighted_cliques_wrong_weight_length_errors() {
        let g = triangle_plus_pendant();
        assert!(weighted_clique_number(&g, &[1.0, 1.0]).is_err());
        assert!(largest_weighted_cliques(&g, &[1.0, 1.0]).is_err());
        assert!(weighted_cliques(&g, &[1.0, 1.0], true, 0.0, 0.0, None).is_err());
    }

    #[test]
    fn test_weighted_cliques_non_positive_weight_errors() {
        let g = triangle_plus_pendant();
        assert!(weighted_clique_number(&g, &[1.0, 0.0, 1.0, 1.0]).is_err());
        assert!(weighted_clique_number(&g, &[1.0, -2.0, 1.0, 1.0]).is_err());
        // 0.5 truncates to 0, which is not a positive integer.
        assert!(weighted_clique_number(&g, &[0.5, 1.0, 1.0, 1.0]).is_err());
    }

    // --- cliques (all) tests ---

    #[test]
    fn test_cliques_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let all = cliques(&g, 1, 3, None).unwrap();
        // 3 singles + 3 edges + 1 triangle = 7
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_cliques_size_filter() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        // Only size 2
        let edges = cliques(&g, 2, 2, None).unwrap();
        assert_eq!(edges.len(), 3);
        for c in &edges {
            assert_eq!(c.len(), 2);
        }
    }

    #[test]
    fn test_cliques_max_results() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let limited = cliques(&g, 1, 3, Some(4)).unwrap();
        assert_eq!(limited.len(), 4);
    }

    #[test]
    fn test_cliques_k4_size_3() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // C(4,3) = 4 triangles in K4
        let tri = cliques(&g, 3, 3, None).unwrap();
        assert_eq!(tri.len(), 4);
    }

    #[test]
    fn test_cliques_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(cliques(&g, 1, 5, None).unwrap().is_empty());
    }

    #[test]
    fn test_cliques_no_edges_min_2() {
        let g = Graph::with_vertices(5);
        let result = cliques(&g, 2, 5, None).unwrap();
        assert!(result.is_empty());
    }

    // --- independent_vertex_sets tests ---

    #[test]
    fn test_ivs_path_size_2() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Independent sets of size 2: {0,2}
        let sets = independent_vertex_sets(&g, 2, 3, None).unwrap();
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0], vec![0, 2]);
    }

    #[test]
    fn test_ivs_complete_graph() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // In K4, only size-1 independent sets exist
        let sets = independent_vertex_sets(&g, 2, 4, None).unwrap();
        assert!(sets.is_empty());
    }

    #[test]
    fn test_ivs_no_edges() {
        let g = Graph::with_vertices(4);
        // All subsets of size 2 are independent: C(4,2) = 6
        let sets = independent_vertex_sets(&g, 2, 2, None).unwrap();
        assert_eq!(sets.len(), 6);
    }

    #[test]
    fn test_ivs_max_results() {
        let g = Graph::with_vertices(5);
        let sets = independent_vertex_sets(&g, 1, 5, Some(10)).unwrap();
        assert_eq!(sets.len(), 10);
    }

    // ---- maximal_cliques_subset tests ----

    #[test]
    fn test_mcs_empty_graph() {
        let g = Graph::with_vertices(0);
        let r = maximal_cliques_subset(&g, &[], 0, 0, None).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn test_mcs_empty_subset() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let r = maximal_cliques_subset(&g, &[], 0, 0, None).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn test_mcs_invalid_vertex() {
        let g = Graph::with_vertices(3);
        assert!(maximal_cliques_subset(&g, &[5], 0, 0, None).is_err());
    }

    #[test]
    fn test_mcs_triangle_plus_edge() {
        // Triangle {0,1,2} + edge {2,3}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        // Subset {3}: only {2,3} touches vertex 3
        let r = maximal_cliques_subset(&g, &[3], 0, 0, None).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].contains(&2));
        assert!(r[0].contains(&3));

        // Subset {0}: only the triangle touches vertex 0
        let r = maximal_cliques_subset(&g, &[0], 0, 0, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].len(), 3);

        // Subset {2}: both cliques touch vertex 2
        let r = maximal_cliques_subset(&g, &[2], 0, 0, None).unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_mcs_size_filter() {
        // Triangle {0,1,2} + edge {2,3}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        // Subset {2}, min_size=3 → only the triangle
        let r = maximal_cliques_subset(&g, &[2], 3, 0, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].len(), 3);

        // Subset {2}, max_size=2 → only {2,3}
        let r = maximal_cliques_subset(&g, &[2], 0, 2, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].len(), 2);
    }

    #[test]
    fn test_mcs_max_results() {
        // K4: 4 triangles + 1 clique of size 4
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // All vertices in subset → all cliques touch subset
        let all = maximal_cliques_subset(&g, &[0, 1, 2, 3], 0, 0, None).unwrap();
        let limited = maximal_cliques_subset(&g, &[0, 1, 2, 3], 0, 0, Some(1)).unwrap();
        assert!(!all.is_empty());
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn test_mcs_isolated_vertex_in_subset() {
        // 0-1, vertex 2 isolated
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        // Subset {2}: isolated vertex forms a size-1 maximal clique
        let r = maximal_cliques_subset(&g, &[2], 0, 0, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], vec![2]);
    }

    #[test]
    fn test_mcs_directed_ignores_direction() {
        // Directed triangle 0→1→2→0 treated as undirected
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let r = maximal_cliques_subset(&g, &[0], 0, 0, None).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].len(), 3);
    }

    #[test]
    fn test_mcs_all_vertices_subset() {
        // Same result as maximal_cliques when all vertices are in subset
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();

        let subset: Vec<u32> = (0..5).collect();
        let mcs = maximal_cliques_subset(&g, &subset, 0, 0, None).unwrap();
        let mc = maximal_cliques(&g).unwrap();
        assert_eq!(mcs.len(), mc.len());
    }
}
