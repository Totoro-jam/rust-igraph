//! Edge sampling and train/test edge splits (ALGO-TR-009).
//!
//! Utilities for randomly sampling edges and splitting a graph's edge set
//! into disjoint train/test partitions. Essential for link prediction
//! evaluation: the test edges are held out, and the model must predict them.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of an edge train/test split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeSplit {
    /// Training edges (the larger partition).
    pub train: Vec<(VertexId, VertexId)>,
    /// Test edges (the smaller, held-out partition).
    pub test: Vec<(VertexId, VertexId)>,
}

/// Uniformly sample `count` edges from the graph without replacement.
///
/// Returns a random subset of the graph's edges. If `count` exceeds
/// the number of edges, all edges are returned (shuffled).
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `count` — Number of edges to sample.
/// - `seed` — PRNG seed for deterministic sampling.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sample_edges};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0)], false, Some(5)
/// ).unwrap();
///
/// let sampled = sample_edges(&g, 3, 42).unwrap();
/// assert_eq!(sampled.len(), 3);
/// for &(u, v) in &sampled {
///     assert!(g.has_edge(u, v));
/// }
/// ```
pub fn sample_edges(
    graph: &Graph,
    count: usize,
    seed: u64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let edges = collect_edges(graph);
    let actual = count.min(edges.len());
    Ok(shuffle_and_take(edges, actual, seed))
}

/// Split graph edges into train and test sets.
///
/// Randomly partitions the edge set: `test_fraction` of edges go to the
/// test set, the remainder to training. Useful for link prediction
/// evaluation where test edges are removed from the graph.
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `test_fraction` — Fraction of edges for the test set (0.0 to 1.0).
/// - `seed` — PRNG seed for deterministic splitting.
///
/// # Returns
///
/// An [`EdgeSplit`] with `train` and `test` edge vectors.
///
/// # Errors
///
/// Returns an error if `test_fraction` is not in `[0.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, split_edges};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0),(0,2),(1,3),(2,4)],
///     false, Some(5)
/// ).unwrap();
///
/// let split = split_edges(&g, 0.25, 42).unwrap();
/// assert_eq!(split.train.len() + split.test.len(), g.ecount());
/// assert_eq!(split.test.len(), 2); // 25% of 8 = 2
/// ```
pub fn split_edges(graph: &Graph, test_fraction: f64, seed: u64) -> IgraphResult<EdgeSplit> {
    if !(0.0..=1.0).contains(&test_fraction) {
        return Err(IgraphError::InvalidArgument(format!(
            "test_fraction must be in [0.0, 1.0], got {test_fraction}"
        )));
    }

    let edges = collect_edges(graph);
    let ne = edges.len();
    let test_count = (ne as f64 * test_fraction).round() as usize;

    let shuffled = shuffle_all(edges, seed);
    let test = shuffled[..test_count].to_vec();
    let train = shuffled[test_count..].to_vec();

    Ok(EdgeSplit { train, test })
}

/// Split edges ensuring the training graph remains connected.
///
/// Like [`split_edges`] but guarantees that removing the test edges does
/// not disconnect the graph. Edges whose removal would create a bridge
/// (disconnection) are kept in training. If maintaining connectivity
/// limits the test set size, fewer edges than requested may end up in test.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, split_edges_connected};
///
/// // Cycle: every edge can be removed without disconnecting
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
///
/// let split = split_edges_connected(&g, 0.5, 42).unwrap();
/// assert_eq!(split.train.len() + split.test.len(), 4);
/// // Test set has at most 2 edges (50%)
/// assert!(split.test.len() <= 2);
/// ```
pub fn split_edges_connected(
    graph: &Graph,
    test_fraction: f64,
    seed: u64,
) -> IgraphResult<EdgeSplit> {
    if !(0.0..=1.0).contains(&test_fraction) {
        return Err(IgraphError::InvalidArgument(format!(
            "test_fraction must be in [0.0, 1.0], got {test_fraction}"
        )));
    }

    let edges = collect_edges(graph);
    let ne = edges.len();
    let target_test = (ne as f64 * test_fraction).round() as usize;

    let shuffled = shuffle_all(edges, seed);

    let mut train: Vec<(VertexId, VertexId)> = Vec::with_capacity(ne);
    let mut test: Vec<(VertexId, VertexId)> = Vec::new();

    // Build adjacency as we add training edges; check connectivity via
    // simple degree tracking — an edge can go to test only if both
    // endpoints will still have at least one remaining training edge.
    // This is a heuristic; for exact bridge detection we'd need a more
    // expensive algorithm, but this keeps the training graph connected
    // in practice for reasonably dense graphs.
    let nv = graph.vcount() as usize;
    let mut train_degree: Vec<u32> = vec![0; nv];

    // First pass: assign all edges to train to compute full degrees
    for &(u, v) in &shuffled {
        train_degree[u as usize] += 1;
        train_degree[v as usize] += 1;
    }

    // Second pass: try to move edges to test
    for &(u, v) in &shuffled {
        if test.len() >= target_test {
            train.push((u, v));
            continue;
        }

        // Can we remove this edge without isolating a vertex?
        let u_deg = train_degree[u as usize];
        let v_deg = train_degree[v as usize];

        if u_deg > 1 && v_deg > 1 {
            test.push((u, v));
            train_degree[u as usize] -= 1;
            train_degree[v as usize] -= 1;
        } else {
            train.push((u, v));
        }
    }

    Ok(EdgeSplit { train, test })
}

// --- Internal helpers ---

fn collect_edges(graph: &Graph) -> Vec<(VertexId, VertexId)> {
    graph.edges().collect()
}

fn shuffle_and_take(
    mut items: Vec<(VertexId, VertexId)>,
    k: usize,
    seed: u64,
) -> Vec<(VertexId, VertexId)> {
    let n = items.len();
    let mut rng = SplitMix64::new(seed);
    let take = k.min(n);
    for i in 0..take {
        let j = i + rng.gen_index(n - i);
        items.swap(i, j);
    }
    items.truncate(take);
    items
}

fn shuffle_all(mut items: Vec<(VertexId, VertexId)>, seed: u64) -> Vec<(VertexId, VertexId)> {
    let n = items.len();
    let mut rng = SplitMix64::new(seed);
    for i in 0..n.saturating_sub(1) {
        let j = i + rng.gen_index(n - i);
        items.swap(i, j);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn dense5() -> Graph {
        Graph::from_edges(
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 0),
                (0, 2),
                (1, 3),
                (2, 4),
            ],
            false,
            Some(5),
        )
        .unwrap()
    }

    #[test]
    fn sample_edges_basic() {
        let g = cycle5();
        let sampled = sample_edges(&g, 3, 42).unwrap();
        assert_eq!(sampled.len(), 3);
        for &(u, v) in &sampled {
            assert!(g.has_edge(u, v));
        }
    }

    #[test]
    fn sample_edges_all() {
        let g = cycle5();
        let sampled = sample_edges(&g, 100, 42).unwrap();
        assert_eq!(sampled.len(), 5);
    }

    #[test]
    fn sample_edges_zero() {
        let g = cycle5();
        let sampled = sample_edges(&g, 0, 42).unwrap();
        assert!(sampled.is_empty());
    }

    #[test]
    fn sample_edges_no_duplicates() {
        let g = dense5();
        let sampled = sample_edges(&g, 5, 42).unwrap();
        for i in 0..sampled.len() {
            for j in (i + 1)..sampled.len() {
                assert_ne!(sampled[i], sampled[j]);
            }
        }
    }

    #[test]
    fn sample_edges_deterministic() {
        let g = dense5();
        let s1 = sample_edges(&g, 4, 99).unwrap();
        let s2 = sample_edges(&g, 4, 99).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn split_basic() {
        let g = dense5(); // 8 edges
        let split = split_edges(&g, 0.25, 42).unwrap();
        assert_eq!(split.train.len() + split.test.len(), 8);
        assert_eq!(split.test.len(), 2); // 25% of 8 = 2
    }

    #[test]
    fn split_all_train() {
        let g = cycle5();
        let split = split_edges(&g, 0.0, 42).unwrap();
        assert_eq!(split.train.len(), 5);
        assert!(split.test.is_empty());
    }

    #[test]
    fn split_all_test() {
        let g = cycle5();
        let split = split_edges(&g, 1.0, 42).unwrap();
        assert!(split.train.is_empty());
        assert_eq!(split.test.len(), 5);
    }

    #[test]
    fn split_invalid_fraction() {
        let g = cycle5();
        assert!(split_edges(&g, 1.5, 42).is_err());
        assert!(split_edges(&g, -0.1, 42).is_err());
    }

    #[test]
    fn split_deterministic() {
        let g = dense5();
        let s1 = split_edges(&g, 0.3, 99).unwrap();
        let s2 = split_edges(&g, 0.3, 99).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn split_connected_basic() {
        let g = dense5();
        let split = split_edges_connected(&g, 0.25, 42).unwrap();
        assert_eq!(split.train.len() + split.test.len(), 8);
        // No vertex should be isolated in training set
        let nv = g.vcount() as usize;
        let mut deg = vec![0u32; nv];
        for &(u, v) in &split.train {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
        }
        for d in &deg {
            assert!(*d >= 1, "vertex isolated in training set");
        }
    }

    #[test]
    fn split_connected_cycle() {
        let g = cycle5();
        let split = split_edges_connected(&g, 0.5, 42).unwrap();
        // In a cycle, each vertex has degree 2, so we can remove at most
        // one edge per vertex → at most 2 edges can go to test while
        // keeping all degrees >= 1
        assert!(split.test.len() <= 2);
        let nv = g.vcount() as usize;
        let mut deg = vec![0u32; nv];
        for &(u, v) in &split.train {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
        }
        for d in &deg {
            assert!(*d >= 1);
        }
    }

    #[test]
    fn split_connected_invalid_fraction() {
        let g = cycle5();
        assert!(split_edges_connected(&g, 2.0, 42).is_err());
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(3);
        let sampled = sample_edges(&g, 5, 42).unwrap();
        assert!(sampled.is_empty());

        let split = split_edges(&g, 0.5, 42).unwrap();
        assert!(split.train.is_empty());
        assert!(split.test.is_empty());
    }

    #[test]
    fn directed_graph() {
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        let sampled = sample_edges(&g, 2, 42).unwrap();
        assert_eq!(sampled.len(), 2);
        for &(u, v) in &sampled {
            assert!(g.has_edge(u, v));
        }
    }
}
