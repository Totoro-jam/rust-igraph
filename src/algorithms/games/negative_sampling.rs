//! Negative edge sampling for link prediction training (ALGO-TR-007).
//!
//! Generates random vertex pairs that do NOT have an edge in the graph,
//! used as negative examples for training link prediction models (e.g.,
//! knowledge graph embeddings, GNN-based link predictors).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphResult, VertexId};

/// Sample non-edges (negative examples) uniformly at random.
///
/// Generates `count` random `(u, v)` pairs where no edge exists between
/// `u` and `v` in the graph. For undirected graphs, only generates pairs
/// with `u < v`. Self-loops are never generated.
///
/// Uses rejection sampling: generates random pairs and keeps those that
/// are non-edges. Efficient when the graph is sparse (density ≪ 1).
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `count` — Number of negative samples to generate.
/// - `seed` — PRNG seed for deterministic sampling.
///
/// # Returns
///
/// A vector of `(u, v)` pairs representing non-edges. May return fewer
/// than `count` pairs if the graph is too dense (all pairs are edges).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sample_negative_edges};
///
/// // Path: 0-1-2-3 (missing edges: 0-2, 0-3, 1-3)
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3)], false, Some(4)
/// ).unwrap();
///
/// let neg = sample_negative_edges(&g, 3, 42).unwrap();
/// assert_eq!(neg.len(), 3);
/// for &(u, v) in &neg {
///     assert!(!g.has_edge(u, v));
///     assert!(u < v); // undirected: canonical order
/// }
/// ```
pub fn sample_negative_edges(
    graph: &Graph,
    count: usize,
    seed: u64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let nv = graph.vcount();
    if nv < 2 || count == 0 {
        return Ok(Vec::new());
    }

    let directed = graph.is_directed();
    let nv64 = u64::from(nv);
    let max_possible = if directed {
        nv64 * (nv64 - 1)
    } else {
        nv64 * (nv64 - 1) / 2
    };

    let existing_edges = graph.ecount() as u64;
    let available = max_possible.saturating_sub(existing_edges);
    if available == 0 {
        return Ok(Vec::new());
    }

    let target = count.min(available as usize);
    let mut rng = SplitMix64::new(seed);
    let mut result: Vec<(VertexId, VertexId)> = Vec::with_capacity(target);

    let max_attempts = target as u64 * 20 + 1000;
    let mut attempts: u64 = 0;

    while result.len() < target && attempts < max_attempts {
        attempts += 1;

        let u = rng.gen_index(nv as usize) as VertexId;
        let v = rng.gen_index(nv as usize) as VertexId;

        if u == v {
            continue;
        }

        let (a, b) = if !directed && u > v { (v, u) } else { (u, v) };

        if graph.has_edge(a, b) {
            continue;
        }

        if result.contains(&(a, b)) {
            continue;
        }

        result.push((a, b));
    }

    Ok(result)
}

/// Sample negative edges avoiding a set of positive edges.
///
/// Like [`sample_negative_edges`] but also avoids generating pairs
/// that appear in an additional `exclude` set. Useful when you have
/// a train/test split and want negatives that don't overlap with
/// either set.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sample_negative_edges_excluding};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4)], false, Some(5)
/// ).unwrap();
///
/// // Also exclude edge 0-4 from negatives
/// let exclude = vec![(0, 4)];
/// let neg = sample_negative_edges_excluding(&g, 3, &exclude, 42).unwrap();
/// for &(u, v) in &neg {
///     assert!(!g.has_edge(u, v));
///     assert_ne!((u, v), (0, 4));
/// }
/// ```
pub fn sample_negative_edges_excluding(
    graph: &Graph,
    count: usize,
    exclude: &[(VertexId, VertexId)],
    seed: u64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let nv = graph.vcount();
    if nv < 2 || count == 0 {
        return Ok(Vec::new());
    }

    let directed = graph.is_directed();
    let mut rng = SplitMix64::new(seed);
    let mut result: Vec<(VertexId, VertexId)> = Vec::with_capacity(count);

    let max_attempts = count as u64 * 20 + 1000;
    let mut attempts: u64 = 0;

    while result.len() < count && attempts < max_attempts {
        attempts += 1;

        let u = rng.gen_index(nv as usize) as VertexId;
        let v = rng.gen_index(nv as usize) as VertexId;

        if u == v {
            continue;
        }

        let (a, b) = if !directed && u > v { (v, u) } else { (u, v) };

        if graph.has_edge(a, b) {
            continue;
        }

        if exclude.contains(&(a, b)) {
            continue;
        }

        if result.contains(&(a, b)) {
            continue;
        }

        result.push((a, b));
    }

    Ok(result)
}

/// Sample negative edges proportional to vertex degree (popularity-biased).
///
/// Higher-degree vertices are more likely to appear in sampled non-edges.
/// This produces harder negatives for link prediction since they involve
/// popular nodes that could plausibly have edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sample_negative_edges_degree_biased};
///
/// // Star: vertex 0 has degree 4, others degree 1
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
///
/// let neg = sample_negative_edges_degree_biased(&g, 3, 42).unwrap();
/// assert_eq!(neg.len(), 3);
/// for &(u, v) in &neg {
///     assert!(!g.has_edge(u, v));
/// }
/// ```
pub fn sample_negative_edges_degree_biased(
    graph: &Graph,
    count: usize,
    seed: u64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let nv = graph.vcount();
    if nv < 2 || count == 0 {
        return Ok(Vec::new());
    }

    let directed = graph.is_directed();
    let mut rng = SplitMix64::new(seed);

    let mut cumulative_degree: Vec<u64> = Vec::with_capacity(nv as usize);
    let mut total_degree: u64 = 0;
    for vid in 0..nv {
        let deg = graph.degree(vid)?;
        total_degree += (deg as u64) + 1;
        cumulative_degree.push(total_degree);
    }

    let mut result: Vec<(VertexId, VertexId)> = Vec::with_capacity(count);
    let max_attempts = count as u64 * 20 + 1000;
    let mut attempts: u64 = 0;

    while result.len() < count && attempts < max_attempts {
        attempts += 1;

        let u = sample_by_cumulative(&cumulative_degree, total_degree, &mut rng);
        let v = sample_by_cumulative(&cumulative_degree, total_degree, &mut rng);

        if u == v {
            continue;
        }

        let (a, b) = if !directed && u > v { (v, u) } else { (u, v) };

        if graph.has_edge(a, b) {
            continue;
        }

        if result.contains(&(a, b)) {
            continue;
        }

        result.push((a, b));
    }

    Ok(result)
}

// --- Internal helpers ---

fn sample_by_cumulative(cumulative: &[u64], total: u64, rng: &mut SplitMix64) -> VertexId {
    let r = rng.next_u64() % total;
    // Both arms produce the same result — binary_search returns the
    // insertion point either way.
    let idx = match cumulative.binary_search(&(r + 1)) {
        Ok(i) | Err(i) => i,
    };
    idx as VertexId
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn complete4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    #[test]
    fn basic_negative_sampling() {
        let g = path4();
        let neg = sample_negative_edges(&g, 3, 42).unwrap();
        assert_eq!(neg.len(), 3);
        for &(u, v) in &neg {
            assert!(!g.has_edge(u, v));
            assert!(u < v);
            assert_ne!(u, v);
        }
    }

    #[test]
    fn no_duplicates() {
        let g = path4();
        let neg = sample_negative_edges(&g, 3, 42).unwrap();
        for i in 0..neg.len() {
            for j in (i + 1)..neg.len() {
                assert_ne!(neg[i], neg[j]);
            }
        }
    }

    #[test]
    fn complete_graph_no_negatives() {
        let g = complete4();
        let neg = sample_negative_edges(&g, 10, 42).unwrap();
        assert!(neg.is_empty());
    }

    #[test]
    fn deterministic() {
        let g = path4();
        let n1 = sample_negative_edges(&g, 3, 99).unwrap();
        let n2 = sample_negative_edges(&g, 3, 99).unwrap();
        assert_eq!(n1, n2);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let neg = sample_negative_edges(&g, 5, 42).unwrap();
        assert!(neg.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let neg = sample_negative_edges(&g, 5, 42).unwrap();
        assert!(neg.is_empty());
    }

    #[test]
    fn zero_count() {
        let g = path4();
        let neg = sample_negative_edges(&g, 0, 42).unwrap();
        assert!(neg.is_empty());
    }

    #[test]
    fn directed_graph() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        let neg = sample_negative_edges(&g, 4, 42).unwrap();
        assert_eq!(neg.len(), 4);
        for &(u, v) in &neg {
            assert!(!g.has_edge(u, v));
            assert_ne!(u, v);
        }
    }

    #[test]
    fn excluding_works() {
        let g = path4(); // edges: 0-1, 1-2, 2-3
        // Non-edges: (0,2), (0,3), (1,3) — exclude (0,3)
        let exclude = vec![(0, 3)];
        let neg = sample_negative_edges_excluding(&g, 10, &exclude, 42).unwrap();
        for &(u, v) in &neg {
            assert_ne!((u, v), (0, 3));
            assert!(!g.has_edge(u, v));
        }
    }

    #[test]
    fn degree_biased_valid() {
        let g = path4();
        let neg = sample_negative_edges_degree_biased(&g, 3, 42).unwrap();
        assert_eq!(neg.len(), 3);
        for &(u, v) in &neg {
            assert!(!g.has_edge(u, v));
            assert!(u < v);
        }
    }

    #[test]
    fn degree_biased_prefers_high_degree() {
        // Vertex 0 connected to 1,2,3,4 (degree 4); vertices 5..19 isolated.
        // Non-edges involving vertex 0: (0,5)...(0,19) — 15 possible.
        // Non-edges NOT involving vertex 0: C(15,2) + 15*4 = 105+60 = 165 possible.
        // Degree-biased should oversample vertex 0 relative to uniform.
        let edges: Vec<(u32, u32)> = (1..5).map(|i| (0, i)).collect();
        let g = Graph::from_edges(&edges, false, Some(20)).unwrap();

        let neg = sample_negative_edges_degree_biased(&g, 30, 42).unwrap();
        let v0_count = neg.iter().filter(|&&(u, v)| u == 0 || v == 0).count();
        // Vertex 0 has degree 4 (+ 1 bias = weight 5), others have 0 or 1.
        // Should appear more often than purely uniform would predict.
        assert!(v0_count >= 3);
    }

    #[test]
    fn respects_max_available() {
        // Path 0-1-2: only 1 non-edge: (0,2)
        let g = Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap();
        let neg = sample_negative_edges(&g, 100, 42).unwrap();
        assert_eq!(neg.len(), 1);
        assert_eq!(neg[0], (0, 2));
    }
}
