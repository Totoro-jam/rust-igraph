//! Batch random walks for graph embedding training (ALGO-TR-005).
//!
//! Generates multiple random walks from specified starting vertices.
//! This is the workhorse for training graph embedding models like
//! `DeepWalk`, `Node2Vec`, and LINE.
//!
//! Each call produces a corpus of vertex sequences that can be fed
//! directly into a skip-gram or similar model.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::paths::random_walk::random_walk;
use crate::algorithms::paths::random_walk_node2vec::random_walk_node2vec;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphResult, VertexId};

/// Generate multiple random walks from every vertex in the graph.
///
/// For each vertex `v` in `0..graph.vcount()`, generates
/// `walks_per_vertex` walks of length `walk_length` starting at `v`.
/// The order of starting vertices is shuffled independently for each
/// round using the deterministic PRNG.
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `weights` — Optional edge weights (positive). `None` for unweighted.
/// - `mode` — Direction mode for directed graphs.
/// - `walks_per_vertex` — Number of walks to generate per vertex.
/// - `walk_length` — Number of steps per walk.
/// - `seed` — Deterministic PRNG seed.
///
/// # Returns
///
/// A `Vec<Vec<VertexId>>` where each inner vector is a walk (vertex
/// sequence of length ≤ `walk_length + 1`). Total number of walks is
/// `vcount * walks_per_vertex` (unless walks get stuck early).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_walks, DijkstraMode};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// let corpus = random_walks(&g, None, DijkstraMode::Out, 2, 5, 42).unwrap();
/// assert_eq!(corpus.len(), 8); // 4 vertices * 2 walks
/// for walk in &corpus {
///     assert!(walk.len() <= 6); // at most walk_length + 1
///     assert!(walk[0] < 4);
/// }
/// ```
pub fn random_walks(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    walks_per_vertex: u32,
    walk_length: u32,
    seed: u64,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 || walks_per_vertex == 0 || walk_length == 0 {
        return Ok(Vec::new());
    }

    let total = u64::from(n)
        .checked_mul(u64::from(walks_per_vertex))
        .ok_or_else(|| {
            crate::core::IgraphError::InvalidArgument(
                "walks_per_vertex * vcount overflows u64".into(),
            )
        })?;

    let capacity = usize::try_from(total).unwrap_or(usize::MAX);
    let mut rng = SplitMix64::new(seed);
    let mut corpus: Vec<Vec<VertexId>> = Vec::with_capacity(capacity);

    let mut order: Vec<VertexId> = (0..n).collect();

    for _ in 0..walks_per_vertex {
        for i in (1..order.len()).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }

        for &start in &order {
            let walk_seed = rng.next_u64();
            let (vs, _) = random_walk(graph, weights, start, mode, walk_length, walk_seed)?;
            corpus.push(vs);
        }
    }

    Ok(corpus)
}

/// Generate multiple `Node2Vec` random walks from every vertex.
///
/// Same as [`random_walks`] but uses second-order biased walks with
/// parameters `p` (return) and `q` (in-out). See
/// [`random_walk_node2vec`] for details on the bias.
///
/// # Parameters
///
/// - `p` — Return parameter (higher = less backtracking).
/// - `q` — In-out parameter (higher = more BFS-like).
/// - Other parameters as in [`random_walks`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_walks_node2vec, DijkstraMode};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0),(0,2)], false, Some(4)
/// ).unwrap();
/// let corpus = random_walks_node2vec(
///     &g, None, DijkstraMode::Out, 2, 5, 1.0, 1.0, 42
/// ).unwrap();
/// assert_eq!(corpus.len(), 8); // 4 vertices * 2 walks
/// ```
#[allow(clippy::too_many_arguments)]
pub fn random_walks_node2vec(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    walks_per_vertex: u32,
    walk_length: u32,
    p: f64,
    q: f64,
    seed: u64,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 || walks_per_vertex == 0 || walk_length == 0 {
        return Ok(Vec::new());
    }

    let total = u64::from(n)
        .checked_mul(u64::from(walks_per_vertex))
        .ok_or_else(|| {
            crate::core::IgraphError::InvalidArgument(
                "walks_per_vertex * vcount overflows u64".into(),
            )
        })?;

    let capacity = usize::try_from(total).unwrap_or(usize::MAX);
    let mut rng = SplitMix64::new(seed);
    let mut corpus: Vec<Vec<VertexId>> = Vec::with_capacity(capacity);

    let mut order: Vec<VertexId> = (0..n).collect();

    for _ in 0..walks_per_vertex {
        for i in (1..order.len()).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }

        for &start in &order {
            let walk_seed = rng.next_u64();
            let (vs, _) =
                random_walk_node2vec(graph, weights, start, mode, walk_length, p, q, walk_seed)?;
            corpus.push(vs);
        }
    }

    Ok(corpus)
}

/// Generate random walks from a specific set of starting vertices.
///
/// Unlike [`random_walks`] which walks from every vertex, this function
/// generates walks only from the provided `starts` list.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_walks_from, DijkstraMode};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// let corpus = random_walks_from(
///     &g, &[0, 2], None, DijkstraMode::Out, 3, 5, 42
/// ).unwrap();
/// assert_eq!(corpus.len(), 6); // 2 starts * 3 walks
/// for walk in &corpus {
///     assert!(walk[0] == 0 || walk[0] == 2);
/// }
/// ```
pub fn random_walks_from(
    graph: &Graph,
    starts: &[VertexId],
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    walks_per_vertex: u32,
    walk_length: u32,
    seed: u64,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if starts.is_empty() || walks_per_vertex == 0 || walk_length == 0 {
        return Ok(Vec::new());
    }

    for &s in starts {
        if s >= n {
            return Err(crate::core::IgraphError::VertexOutOfRange { id: s, n });
        }
    }

    let total = (starts.len() as u64)
        .checked_mul(u64::from(walks_per_vertex))
        .ok_or_else(|| {
            crate::core::IgraphError::InvalidArgument(
                "walks_per_vertex * starts.len() overflows u64".into(),
            )
        })?;

    let capacity = usize::try_from(total).unwrap_or(usize::MAX);
    let mut rng = SplitMix64::new(seed);
    let mut corpus: Vec<Vec<VertexId>> = Vec::with_capacity(capacity);

    let mut order: Vec<VertexId> = starts.to_vec();

    for _ in 0..walks_per_vertex {
        for i in (1..order.len()).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }

        for &start in &order {
            let walk_seed = rng.next_u64();
            let (vs, _) = random_walk(graph, weights, start, mode, walk_length, walk_seed)?;
            corpus.push(vs);
        }
    }

    Ok(corpus)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cycle_graph(n: u32) -> Graph {
        let edges: Vec<(u32, u32)> = (0..n).map(|i| (i, (i + 1) % n)).collect();
        Graph::from_edges(&edges, false, Some(n)).unwrap()
    }

    #[test]
    fn random_walks_correct_count() {
        let g = cycle_graph(5);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 3, 10, 42).unwrap();
        assert_eq!(corpus.len(), 15); // 5 * 3
    }

    #[test]
    fn random_walks_walk_length() {
        let g = cycle_graph(4);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 1, 7, 42).unwrap();
        for walk in &corpus {
            assert_eq!(walk.len(), 8); // cycle never gets stuck
        }
    }

    #[test]
    fn random_walks_all_vertices_covered() {
        let g = cycle_graph(6);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 1, 3, 42).unwrap();
        let mut seen = [false; 6];
        for walk in &corpus {
            seen[walk[0] as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn random_walks_deterministic() {
        let g = cycle_graph(5);
        let c1 = random_walks(&g, None, DijkstraMode::Out, 2, 5, 99).unwrap();
        let c2 = random_walks(&g, None, DijkstraMode::Out, 2, 5, 99).unwrap();
        assert_eq!(c1, c2);
    }

    #[test]
    fn random_walks_empty_graph() {
        let g = Graph::with_vertices(0);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 5, 10, 42).unwrap();
        assert!(corpus.is_empty());
    }

    #[test]
    fn random_walks_zero_walks() {
        let g = cycle_graph(4);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 0, 10, 42).unwrap();
        assert!(corpus.is_empty());
    }

    #[test]
    fn random_walks_zero_length() {
        let g = cycle_graph(4);
        let corpus = random_walks(&g, None, DijkstraMode::Out, 2, 0, 42).unwrap();
        assert!(corpus.is_empty());
    }

    #[test]
    fn random_walks_node2vec_correct_count() {
        let g = cycle_graph(5);
        let corpus =
            random_walks_node2vec(&g, None, DijkstraMode::Out, 2, 5, 1.0, 1.0, 42).unwrap();
        assert_eq!(corpus.len(), 10); // 5 * 2
    }

    #[test]
    fn random_walks_node2vec_deterministic() {
        let g = cycle_graph(4);
        let c1 = random_walks_node2vec(&g, None, DijkstraMode::Out, 3, 8, 2.0, 0.5, 77).unwrap();
        let c2 = random_walks_node2vec(&g, None, DijkstraMode::Out, 3, 8, 2.0, 0.5, 77).unwrap();
        assert_eq!(c1, c2);
    }

    #[test]
    fn random_walks_from_specific_starts() {
        let g = cycle_graph(6);
        let corpus = random_walks_from(&g, &[1, 3, 5], None, DijkstraMode::Out, 2, 4, 42).unwrap();
        assert_eq!(corpus.len(), 6); // 3 * 2
        for walk in &corpus {
            assert!(walk[0] == 1 || walk[0] == 3 || walk[0] == 5);
        }
    }

    #[test]
    fn random_walks_from_invalid_start() {
        let g = cycle_graph(4);
        let result = random_walks_from(&g, &[0, 10], None, DijkstraMode::Out, 1, 5, 42);
        assert!(result.is_err());
    }

    #[test]
    fn random_walks_from_empty_starts() {
        let g = cycle_graph(4);
        let corpus = random_walks_from(&g, &[], None, DijkstraMode::Out, 3, 5, 42).unwrap();
        assert!(corpus.is_empty());
    }

    #[test]
    fn random_walks_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        g.add_edge(0, 2).unwrap(); // edge 2
        let weights = vec![10.0, 1.0, 0.001];
        let corpus = random_walks(&g, Some(&weights), DijkstraMode::Out, 5, 3, 42).unwrap();
        assert_eq!(corpus.len(), 15); // 3 * 5
    }

    #[test]
    fn random_walks_directed() {
        // Directed cycle: 0→1→2→3→0
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], true, Some(4)).unwrap();
        let corpus = random_walks(&g, None, DijkstraMode::Out, 2, 5, 42).unwrap();
        assert_eq!(corpus.len(), 8);
        // All walks from vertex v should follow the directed cycle
        for walk in &corpus {
            for i in 1..walk.len() {
                assert_eq!(walk[i], (walk[i - 1] + 1) % 4);
            }
        }
    }
}
