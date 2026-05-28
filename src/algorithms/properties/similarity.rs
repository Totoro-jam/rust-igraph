//! Vertex similarity measures (ALGO-PR-036).
//!
//! Cocitation, bibliographic coupling, Jaccard similarity, and Dice
//! similarity for vertex pairs.

use crate::core::{Graph, IgraphResult, VertexId};

/// Computes the cocitation scores between all pairs of vertices.
///
/// Two vertices are cocited if there is a third vertex citing (having
/// outgoing edges to) both. The cocitation score of vertices `u` and `v`
/// is the number of vertices that have edges to both `u` and `v`.
///
/// For undirected graphs, this equals the number of common neighbors.
///
/// Returns a flat vector of length `n * n` in row-major order, where
/// `result[u * n + v]` is the cocitation count for the pair `(u, v)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cocitation};
///
/// // 2 → 0, 2 → 1 means vertices 0 and 1 are cocited by vertex 2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 1).unwrap();
///
/// let scores = cocitation(&g).unwrap();
/// assert_eq!(scores[0 * 3 + 1], 1); // cocitation(0,1) = 1
/// assert_eq!(scores[1 * 3 + 0], 1); // symmetric
/// ```
pub fn cocitation(graph: &Graph) -> IgraphResult<Vec<u32>> {
    cocitation_impl(graph, true)
}

/// Computes bibliographic coupling scores between all pairs of vertices.
///
/// Two vertices are bibliographically coupled if they both cite (have
/// outgoing edges to) a common third vertex. The coupling score of `u`
/// and `v` is the number of vertices that both `u` and `v` have edges to.
///
/// For undirected graphs, this equals the number of common neighbors
/// (same as cocitation).
///
/// Returns a flat vector of length `n * n` in row-major order.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bibcoupling};
///
/// // 0 → 2, 1 → 2 means vertices 0 and 1 both cite vertex 2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let scores = bibcoupling(&g).unwrap();
/// assert_eq!(scores[0 * 3 + 1], 1); // coupling(0,1) = 1
/// assert_eq!(scores[1 * 3 + 0], 1); // symmetric
/// ```
pub fn bibcoupling(graph: &Graph) -> IgraphResult<Vec<u32>> {
    cocitation_impl(graph, false)
}

/// Computes Jaccard similarity coefficients for given vertex pairs.
///
/// The Jaccard similarity of two vertices is:
/// `|N(u) ∩ N(v)| / |N(u) ∪ N(v)|`
///
/// where `N(v)` is the neighbor set of `v`. If both vertices are isolated
/// (no neighbors), the similarity is 0.
///
/// # Arguments
///
/// * `graph` — the input graph (treated as undirected for neighbor lookup).
/// * `pairs` — slice of `(u, v)` vertex pairs to compute similarity for.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_jaccard_pairs};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
///
/// let sim = similarity_jaccard_pairs(&g, &[(0, 1)]).unwrap();
/// // N(0) = {2,3}, N(1) = {2,3,4}, intersection = {2,3}, union = {2,3,4}
/// // Jaccard = 2/3
/// assert!((sim[0] - 2.0 / 3.0).abs() < 1e-10);
/// ```
pub fn similarity_jaccard_pairs(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let adj = build_sorted_adjacency(graph)?;
    let mut result = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        if u >= n || v >= n {
            return Err(crate::core::error::IgraphError::InvalidArgument(
                "vertex ID out of range in similarity_jaccard_pairs".to_string(),
            ));
        }
        if u == v {
            result.push(1.0);
            continue;
        }
        let (isect, union_size) =
            sorted_intersection_union_size(&adj[u as usize], &adj[v as usize]);
        if union_size == 0 {
            result.push(0.0);
        } else {
            #[allow(clippy::cast_precision_loss)]
            result.push(isect as f64 / union_size as f64);
        }
    }

    Ok(result)
}

/// Computes Dice similarity coefficients for given vertex pairs.
///
/// The Dice similarity of two vertices is:
/// `2 * |N(u) ∩ N(v)| / (|N(u)| + |N(v)|)`
///
/// This is related to Jaccard by: `Dice = 2*J / (1+J)`.
///
/// # Arguments
///
/// * `graph` — the input graph (treated as undirected for neighbor lookup).
/// * `pairs` — slice of `(u, v)` vertex pairs to compute similarity for.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_dice_pairs};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
///
/// let sim = similarity_dice_pairs(&g, &[(0, 1)]).unwrap();
/// // N(0)={2,3}, N(1)={2,3,4}, |intersection|=2, |N(0)|+|N(1)|=5
/// // Dice = 2*2/5 = 0.8
/// assert!((sim[0] - 0.8).abs() < 1e-10);
/// ```
pub fn similarity_dice_pairs(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let adj = build_sorted_adjacency(graph)?;
    let mut result = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        if u >= n || v >= n {
            return Err(crate::core::error::IgraphError::InvalidArgument(
                "vertex ID out of range in similarity_dice_pairs".to_string(),
            ));
        }
        if u == v {
            result.push(1.0);
            continue;
        }
        let deg_sum = adj[u as usize].len() + adj[v as usize].len();
        if deg_sum == 0 {
            result.push(0.0);
        } else {
            let (isect, _) = sorted_intersection_union_size(&adj[u as usize], &adj[v as usize]);
            #[allow(clippy::cast_precision_loss)]
            result.push(2.0 * isect as f64 / deg_sum as f64);
        }
    }

    Ok(result)
}

/// Computes the inverse log-weighted (Adamic-Adar) similarity for given vertex pairs.
///
/// The Adamic-Adar index of two vertices u and v is:
/// `sum_{w in N(u) ∩ N(v)} 1 / log(deg(w))`
///
/// where the sum runs over all common neighbors. High-degree common
/// neighbors contribute less, reflecting that a shared low-degree
/// neighbor is more informative. Isolated vertices have zero similarity
/// to any other vertex.
///
/// # Arguments
///
/// * `graph` — the input graph (treated as undirected for neighbor lookup).
/// * `pairs` — slice of `(u, v)` vertex pairs to compute similarity for.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_inverse_log_weighted_pairs};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
///
/// let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 1)]).unwrap();
/// // Common neighbors: 2 (deg=2), 3 (deg=2)
/// // AA = 1/ln(2) + 1/ln(2) = 2/ln(2)
/// assert!((sim[0] - 2.0 / 2.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn similarity_inverse_log_weighted_pairs(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let adj = build_sorted_adjacency(graph)?;

    // Precompute degrees for weight calculation
    #[allow(clippy::cast_precision_loss)]
    let weights: Vec<f64> = adj
        .iter()
        .map(|neighbors| {
            let deg = neighbors.len();
            if deg > 1 {
                1.0 / (deg as f64).ln()
            } else {
                0.0
            }
        })
        .collect();

    let mut result = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        if u >= n || v >= n {
            return Err(crate::core::error::IgraphError::InvalidArgument(
                "vertex ID out of range in similarity_inverse_log_weighted_pairs".to_string(),
            ));
        }
        if u == v {
            result.push(0.0);
            continue;
        }
        let common = sorted_intersection_weighted(&adj[u as usize], &adj[v as usize], &weights);
        result.push(common);
    }

    Ok(result)
}

fn sorted_intersection_weighted(a: &[VertexId], b: &[VertexId], weights: &[f64]) -> f64 {
    let mut i = 0;
    let mut j = 0;
    let mut sum = 0.0_f64;

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                sum += weights[a[i] as usize];
                i += 1;
                j += 1;
            }
        }
    }

    sum
}

/// Internal: cocitation (mode=OUT on neighbors → predecessors share successors)
/// or bibcoupling (mode=IN → successors share predecessors).
fn cocitation_impl(graph: &Graph, is_cocitation: bool) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut result = vec![0u32; n * n];

    if n == 0 {
        return Ok(result);
    }

    let ecount = graph.ecount();
    let directed = graph.is_directed();

    // Build adjacency: for cocitation we need in-neighbors (who cites whom)
    // For bibcoupling we need out-neighbors (whom does each vertex cite)
    // Cocitation: for each vertex w, get its out-neighbors. Each pair of
    //   out-neighbors (u,v) is cocited by w.
    // Bibcoupling: for each vertex w, get its in-neighbors. Each pair of
    //   in-neighbors (u,v) share w as a common citation target.

    // Build the relevant adjacency list
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;

        if directed {
            if is_cocitation {
                // We want: for vertex src, its out-neighbors are those it cites
                // Cocitation counts common in-neighbors, so we collect out-adj
                adj[src as usize].push(tgt);
            } else {
                // Bibcoupling: collect in-adj (who points to tgt)
                adj[tgt as usize].push(src);
            }
        } else {
            // Undirected: both directions
            adj[src as usize].push(tgt);
            if src != tgt {
                adj[tgt as usize].push(src);
            }
        }
    }

    // For each vertex w, iterate over pairs of its neighbors
    for neighbors in &adj {
        for (i, &nei_u) in neighbors.iter().enumerate() {
            let u = nei_u as usize;
            for &nei_v in &neighbors[(i + 1)..] {
                let v = nei_v as usize;
                result[u * n + v] += 1;
                result[v * n + u] += 1;
            }
        }
    }

    Ok(result)
}

fn build_sorted_adjacency(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        adj[src as usize].push(tgt);
        if src != tgt {
            adj[tgt as usize].push(src);
        }
    }

    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok(adj)
}

/// Compute the full Jaccard similarity matrix for all vertex pairs.
///
/// Returns a flat vector of length `n * n` in row-major order, where
/// `result[u * n + v]` is the Jaccard similarity between vertices `u`
/// and `v`. The diagonal is 1.0 (a vertex is perfectly similar to
/// itself). For pairs of isolated vertices, the similarity is 0.0.
///
/// Counterpart of `igraph_similarity_jaccard(_, _, vss_all(), IGRAPH_ALL, loops=false)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_jaccard};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// let sim = similarity_jaccard(&g).unwrap();
/// // N(0)={2,3}, N(1)={2,3} → Jaccard(0,1)=1.0
/// assert!((sim[0 * 4 + 1] - 1.0).abs() < 1e-10);
/// // N(0)={2,3}, N(2)={0,1} → intersection empty, union={0,1,2,3}→0
/// assert!(sim[0 * 4 + 2].abs() < 1e-10);
/// ```
pub fn similarity_jaccard(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let adj = build_sorted_adjacency(graph)?;
    let mut result = vec![0.0_f64; n * n];

    for u in 0..n {
        result[u * n + u] = 1.0;
        for v in (u + 1)..n {
            let (isect, union_size) = sorted_intersection_union_size(&adj[u], &adj[v]);
            let sim = if union_size == 0 {
                0.0
            } else {
                #[allow(clippy::cast_precision_loss)]
                {
                    isect as f64 / union_size as f64
                }
            };
            result[u * n + v] = sim;
            result[v * n + u] = sim;
        }
    }

    Ok(result)
}

/// Compute the full Dice similarity matrix for all vertex pairs.
///
/// Returns a flat vector of length `n * n` in row-major order.
/// `Dice(u, v) = 2 * |N(u) ∩ N(v)| / (|N(u)| + |N(v)|)`.
/// The diagonal is 1.0. For pairs of isolated vertices, similarity is 0.0.
///
/// Counterpart of `igraph_similarity_dice(_, _, vss_all(), IGRAPH_ALL, loops=false)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_dice};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// let sim = similarity_dice(&g).unwrap();
/// // N(0)={2,3}, N(1)={2,3}, |intersect|=2, deg_sum=4 → Dice=4/4=1.0
/// assert!((sim[0 * 4 + 1] - 1.0).abs() < 1e-10);
/// ```
pub fn similarity_dice(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let adj = build_sorted_adjacency(graph)?;
    let mut result = vec![0.0_f64; n * n];

    for u in 0..n {
        result[u * n + u] = 1.0;
        for v in (u + 1)..n {
            let deg_sum = adj[u].len() + adj[v].len();
            let sim = if deg_sum == 0 {
                0.0
            } else {
                let (isect, _) = sorted_intersection_union_size(&adj[u], &adj[v]);
                #[allow(clippy::cast_precision_loss)]
                {
                    2.0 * isect as f64 / deg_sum as f64
                }
            };
            result[u * n + v] = sim;
            result[v * n + u] = sim;
        }
    }

    Ok(result)
}

/// Compute the full inverse-log-weighted (Adamic-Adar) similarity matrix.
///
/// Returns a flat vector of length `n * n` in row-major order.
/// The diagonal is 0.0 (matching igraph convention for self-pairs in
/// inverse-log-weighted similarity).
///
/// Counterpart of `igraph_similarity_inverse_log_weighted(_, _, vss_all(), IGRAPH_ALL)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_inverse_log_weighted};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// let sim = similarity_inverse_log_weighted(&g).unwrap();
/// // Common neighbors of (0,1): 2 (deg=2), 3 (deg=2)
/// // AA = 1/ln(2) + 1/ln(2) = 2/ln(2)
/// assert!((sim[0 * 4 + 1] - 2.0 / 2.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn similarity_inverse_log_weighted(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let adj = build_sorted_adjacency(graph)?;

    #[allow(clippy::cast_precision_loss)]
    let weights: Vec<f64> = adj
        .iter()
        .map(|neighbors| {
            let deg = neighbors.len();
            if deg > 1 {
                1.0 / (deg as f64).ln()
            } else {
                0.0
            }
        })
        .collect();

    let mut result = vec![0.0_f64; n * n];

    for u in 0..n {
        for v in (u + 1)..n {
            let sim = sorted_intersection_weighted(&adj[u], &adj[v], &weights);
            result[u * n + v] = sim;
            result[v * n + u] = sim;
        }
    }

    Ok(result)
}

fn sorted_intersection_union_size(a: &[VertexId], b: &[VertexId]) -> (usize, usize) {
    let mut i = 0;
    let mut j = 0;
    let mut isect = 0usize;

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                isect += 1;
                i += 1;
                j += 1;
            }
        }
    }

    let union_size = a.len() + b.len() - isect;
    (isect, union_size)
}

/// Computes Jaccard similarity coefficients for pairs of vertices
/// connected by the given edges.
///
/// For each edge ID in `eids`, this retrieves the endpoint pair `(u, v)`
/// and computes the Jaccard similarity between `u` and `v`.
///
/// Counterpart of `igraph_similarity_jaccard_es()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_jaccard_es};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap(); // edge 0
/// g.add_edge(0, 3).unwrap(); // edge 1
/// g.add_edge(1, 2).unwrap(); // edge 2
/// g.add_edge(1, 3).unwrap(); // edge 3
/// g.add_edge(1, 4).unwrap(); // edge 4
/// g.add_edge(0, 1).unwrap(); // edge 5
///
/// // Jaccard of edge 5 endpoints (0,1):
/// // N(0)={1,2,3}, N(1)={0,2,3,4}, intersection={2,3}, union={0,1,2,3,4}
/// // Jaccard = 2/5 = 0.4
/// let sim = similarity_jaccard_es(&g, &[5]).unwrap();
/// assert!((sim[0] - 0.4).abs() < 1e-10);
/// ```
pub fn similarity_jaccard_es(graph: &Graph, eids: &[u32]) -> IgraphResult<Vec<f64>> {
    let pairs: Vec<(VertexId, VertexId)> = eids
        .iter()
        .map(|&e| graph.edge(e))
        .collect::<IgraphResult<Vec<_>>>()?;
    similarity_jaccard_pairs(graph, &pairs)
}

/// Computes Dice similarity coefficients for pairs of vertices
/// connected by the given edges.
///
/// For each edge ID in `eids`, this retrieves the endpoint pair `(u, v)`
/// and computes the Dice similarity between `u` and `v`.
///
/// The Dice similarity is related to Jaccard by: `Dice = 2*J / (1+J)`.
///
/// Counterpart of `igraph_similarity_dice_es()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, similarity_dice_es};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap(); // edge 0
/// g.add_edge(0, 3).unwrap(); // edge 1
/// g.add_edge(1, 2).unwrap(); // edge 2
/// g.add_edge(1, 3).unwrap(); // edge 3
/// g.add_edge(1, 4).unwrap(); // edge 4
/// g.add_edge(0, 1).unwrap(); // edge 5
///
/// // Dice of edge 5 endpoints (0,1):
/// // N(0)={1,2,3}, N(1)={0,2,3,4}, |intersection|=2, |N(0)|+|N(1)|=7
/// // Dice = 2*2/7 ≈ 0.5714
/// let sim = similarity_dice_es(&g, &[5]).unwrap();
/// assert!((sim[0] - 4.0 / 7.0).abs() < 1e-10);
/// ```
pub fn similarity_dice_es(graph: &Graph, eids: &[u32]) -> IgraphResult<Vec<f64>> {
    let pairs: Vec<(VertexId, VertexId)> = eids
        .iter()
        .map(|&e| graph.edge(e))
        .collect::<IgraphResult<Vec<_>>>()?;
    similarity_dice_pairs(graph, &pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cocitation_directed() {
        // 2→0, 2→1 means 0 and 1 are cocited by 2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 1).unwrap();

        let scores = cocitation(&g).unwrap();
        // scores is 3×3 row-major; check (0,1) and (1,0)
        assert_eq!(scores[1], 1); // row 0, col 1
        assert_eq!(scores[3], 1); // row 1, col 0
        assert_eq!(scores[0], 0); // row 0, col 0
    }

    #[test]
    fn test_cocitation_multiple_citers() {
        // 2→0, 2→1, 3→0, 3→1 → cocitation(0,1) = 2
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 1).unwrap();

        let scores = cocitation(&g).unwrap();
        // 4×4 matrix: (0,1) = index 1, (1,0) = index 4
        assert_eq!(scores[1], 2);
        assert_eq!(scores[4], 2);
    }

    #[test]
    fn test_bibcoupling_directed() {
        // 0→2, 1→2 means 0 and 1 share citation target 2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();

        let scores = bibcoupling(&g).unwrap();
        // 3×3 matrix: (0,1) = index 1, (1,0) = index 3
        assert_eq!(scores[1], 1);
        assert_eq!(scores[3], 1);
    }

    #[test]
    fn test_cocitation_undirected() {
        // Undirected: common neighbors
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 3).unwrap();

        let scores = cocitation(&g).unwrap();
        // 4×4 matrix: (0,1) = index 1, (1,0) = index 4
        // 0 and 1 share neighbors 2 and 3
        assert_eq!(scores[1], 2);
        assert_eq!(scores[4], 2);
    }

    #[test]
    fn test_cocitation_empty() {
        let g = Graph::with_vertices(0);
        let scores = cocitation(&g).unwrap();
        assert!(scores.is_empty());
    }

    #[test]
    fn test_cocitation_isolated() {
        let g = Graph::with_vertices(3);
        let scores = cocitation(&g).unwrap();
        assert!(scores.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_jaccard_complete_overlap() {
        // Triangle: N(0)={1,2}, N(1)={0,2} → intersection={2}, union={0,1,2}→Jaccard=1/3
        // Wait: N(0)={1,2}, N(1)={0,2}, intersection={2}, union={0,1,2}
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();

        let sim = similarity_jaccard_pairs(&g, &[(0, 1)]).unwrap();
        // N(0)={1,2}, N(1)={0,2}, intersection={2}, union={0,1,2}, J=1/3
        assert!((sim[0] - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_self() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let sim = similarity_jaccard_pairs(&g, &[(0, 0)]).unwrap();
        assert!((sim[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_isolated() {
        let g = Graph::with_vertices(3);
        let sim = similarity_jaccard_pairs(&g, &[(0, 1)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_no_overlap() {
        // 0-1, 2-3: N(0)={1}, N(2)={3}, no overlap
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let sim = similarity_jaccard_pairs(&g, &[(0, 2)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_multiple_pairs() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();

        let sim = similarity_jaccard_pairs(&g, &[(0, 1), (0, 2), (2, 3)]).unwrap();
        // (0,1): N(0)={2,3}, N(1)={2,3} → J=1.0
        assert!((sim[0] - 1.0).abs() < 1e-10);
        // (0,2): N(0)={2,3}, N(2)={0,1} → intersection=empty, union={0,1,2,3}→J=0
        assert!((sim[1]).abs() < 1e-10);
        // (2,3): N(2)={0,1}, N(3)={0,1} → J=1.0
        assert!((sim[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dice_basic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();

        let sim = similarity_dice_pairs(&g, &[(0, 1)]).unwrap();
        // N(0)={2,3}, N(1)={2,3,4}, |intersect|=2, deg_sum=5
        // Dice = 2*2/5 = 0.8
        assert!((sim[0] - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_dice_self() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let sim = similarity_dice_pairs(&g, &[(0, 0)]).unwrap();
        assert!((sim[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_dice_isolated() {
        let g = Graph::with_vertices(3);
        let sim = similarity_dice_pairs(&g, &[(0, 1)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_dice_relationship() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();

        let jac = similarity_jaccard_pairs(&g, &[(0, 1)]).unwrap();
        let dice = similarity_dice_pairs(&g, &[(0, 1)]).unwrap();
        // Dice = 2*J / (1+J)
        let expected_dice = 2.0 * jac[0] / (1.0 + jac[0]);
        assert!((dice[0] - expected_dice).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(similarity_jaccard_pairs(&g, &[(0, 5)]).is_err());
    }

    #[test]
    fn test_inverse_log_weighted_basic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();

        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 1)]).unwrap();
        // Common neighbors: 2 (deg=2), 3 (deg=2)
        // AA = 1/ln(2) + 1/ln(2) = 2/ln(2)
        assert!((sim[0] - 2.0 / 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_self() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 0)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_isolated() {
        let g = Graph::with_vertices(3);
        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 1)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_no_common() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 2)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_high_degree() {
        // Hub vertex 4 connected to all others
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();

        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 1)]).unwrap();
        // Common neighbor: 4 (deg=4)
        // AA = 1/ln(4)
        assert!((sim[0] - 1.0 / 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_degree_one_neighbor() {
        // Common neighbor with degree 1 contributes 0
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 2).unwrap();
        // N(0)={2}, N(1)={} → no common neighbors
        let sim = similarity_inverse_log_weighted_pairs(&g, &[(0, 1)]).unwrap();
        assert!((sim[0]).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(similarity_inverse_log_weighted_pairs(&g, &[(0, 5)]).is_err());
    }

    // --- Full matrix tests ---

    #[test]
    fn test_jaccard_matrix_empty() {
        let g = Graph::with_vertices(0);
        let sim = similarity_jaccard(&g).unwrap();
        assert!(sim.is_empty());
    }

    #[test]
    fn test_jaccard_matrix_isolated() {
        let g = Graph::with_vertices(3);
        let sim = similarity_jaccard(&g).unwrap();
        // Diagonal is 1.0, off-diagonal is 0.0.
        for u in 0..3 {
            assert!((sim[u * 3 + u] - 1.0).abs() < 1e-10);
            for v in 0..3 {
                if u != v {
                    assert!(sim[u * 3 + v].abs() < 1e-10);
                }
            }
        }
    }

    #[test]
    fn test_jaccard_matrix_agrees_with_pairs() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();

        let n = 5;
        let matrix = similarity_jaccard(&g).unwrap();
        let pairs = similarity_jaccard_pairs(&g, &[(0, 1), (0, 2), (2, 3)]).unwrap();
        assert!((matrix[1] - pairs[0]).abs() < 1e-10);
        assert!((matrix[2] - pairs[1]).abs() < 1e-10);
        assert!((matrix[2 * n + 3] - pairs[2]).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_matrix_symmetric() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let sim = similarity_jaccard(&g).unwrap();
        for u in 0..4 {
            for v in 0..4 {
                assert!(
                    (sim[u * 4 + v] - sim[v * 4 + u]).abs() < 1e-10,
                    "not symmetric at ({u},{v})"
                );
            }
        }
    }

    #[test]
    fn test_dice_matrix_agrees_with_pairs() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();

        let n = 5;
        let matrix = similarity_dice(&g).unwrap();
        let pairs = similarity_dice_pairs(&g, &[(0, 1), (0, 2), (2, 3)]).unwrap();
        assert!((matrix[1] - pairs[0]).abs() < 1e-10);
        assert!((matrix[2] - pairs[1]).abs() < 1e-10);
        assert!((matrix[2 * n + 3] - pairs[2]).abs() < 1e-10);
    }

    #[test]
    fn test_dice_matrix_diagonal() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let sim = similarity_dice(&g).unwrap();
        for u in 0..3 {
            assert!((sim[u * 3 + u] - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_inverse_log_weighted_matrix_agrees_with_pairs() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();

        let n = 4;
        let matrix = similarity_inverse_log_weighted(&g).unwrap();
        let pairs = similarity_inverse_log_weighted_pairs(&g, &[(0, 1), (0, 2), (2, 3)]).unwrap();
        assert!((matrix[1] - pairs[0]).abs() < 1e-10);
        assert!((matrix[2] - pairs[1]).abs() < 1e-10);
        assert!((matrix[2 * n + 3] - pairs[2]).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_log_weighted_matrix_diagonal_zero() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let sim = similarity_inverse_log_weighted(&g).unwrap();
        for u in 0..3 {
            assert!(sim[u * 3 + u].abs() < 1e-10);
        }
    }

    #[test]
    fn test_jaccard_dice_matrix_relationship() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 4).unwrap();

        let jac = similarity_jaccard(&g).unwrap();
        let dice = similarity_dice(&g).unwrap();
        for u in 0..5 {
            for v in 0..5 {
                if u != v {
                    let j = jac[u * 5 + v];
                    let d = dice[u * 5 + v];
                    let expected_d = if j == 0.0 { 0.0 } else { 2.0 * j / (1.0 + j) };
                    assert!(
                        (d - expected_d).abs() < 1e-10,
                        "Dice/Jaccard mismatch at ({u},{v}): d={d}, expected={expected_d}"
                    );
                }
            }
        }
    }

    // --- similarity_jaccard_es / similarity_dice_es tests ---

    #[test]
    fn test_jaccard_es_matches_pairs() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap(); // edge 0
        g.add_edge(0, 3).unwrap(); // edge 1
        g.add_edge(1, 2).unwrap(); // edge 2
        g.add_edge(1, 3).unwrap(); // edge 3
        g.add_edge(1, 4).unwrap(); // edge 4
        g.add_edge(0, 1).unwrap(); // edge 5

        let es_result = similarity_jaccard_es(&g, &[0, 5]).unwrap();
        let pairs_result = similarity_jaccard_pairs(&g, &[(0, 2), (0, 1)]).unwrap();
        assert!((es_result[0] - pairs_result[0]).abs() < 1e-12);
        assert!((es_result[1] - pairs_result[1]).abs() < 1e-12);
    }

    #[test]
    fn test_jaccard_es_empty() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let result = similarity_jaccard_es(&g, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_jaccard_es_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop, edge 0
        g.add_edge(0, 1).unwrap(); // edge 1
        let result = similarity_jaccard_es(&g, &[0]).unwrap();
        // self-loop → endpoints are (0,0), Jaccard(0,0) = 1.0
        assert!((result[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_jaccard_es_invalid_edge_id() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(similarity_jaccard_es(&g, &[99]).is_err());
    }

    #[test]
    fn test_dice_es_matches_pairs() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap(); // edge 0
        g.add_edge(0, 3).unwrap(); // edge 1
        g.add_edge(1, 2).unwrap(); // edge 2
        g.add_edge(1, 3).unwrap(); // edge 3
        g.add_edge(1, 4).unwrap(); // edge 4
        g.add_edge(0, 1).unwrap(); // edge 5

        let es_result = similarity_dice_es(&g, &[0, 5]).unwrap();
        let pairs_result = similarity_dice_pairs(&g, &[(0, 2), (0, 1)]).unwrap();
        assert!((es_result[0] - pairs_result[0]).abs() < 1e-12);
        assert!((es_result[1] - pairs_result[1]).abs() < 1e-12);
    }

    #[test]
    fn test_dice_es_empty() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let result = similarity_dice_es(&g, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_dice_es_jaccard_relationship() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap(); // edge 0
        g.add_edge(0, 3).unwrap(); // edge 1
        g.add_edge(1, 2).unwrap(); // edge 2
        g.add_edge(1, 3).unwrap(); // edge 3
        g.add_edge(1, 4).unwrap(); // edge 4
        g.add_edge(0, 1).unwrap(); // edge 5

        let jac = similarity_jaccard_es(&g, &[5]).unwrap();
        let dice = similarity_dice_es(&g, &[5]).unwrap();
        let j = jac[0];
        let expected_d = 2.0 * j / (1.0 + j);
        assert!((dice[0] - expected_d).abs() < 1e-12);
    }

    #[test]
    fn test_jaccard_es_all_edges() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        g.add_edge(2, 3).unwrap(); // edge 2
        g.add_edge(0, 3).unwrap(); // edge 3

        let eids: Vec<u32> = (0..4).collect();
        let es_result = similarity_jaccard_es(&g, &eids).unwrap();
        let pairs = [(0u32, 1u32), (1, 2), (2, 3), (0, 3)];
        let pairs_result = similarity_jaccard_pairs(&g, &pairs).unwrap();
        for i in 0..4 {
            assert!((es_result[i] - pairs_result[i]).abs() < 1e-12);
        }
    }
}
