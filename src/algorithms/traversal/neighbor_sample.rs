//! K-hop neighborhood sampling for mini-batch GNN training (ALGO-TR-006).
//!
//! Implements layer-wise neighbor sampling as used by `GraphSAGE`, `PinSAGE`,
//! and similar inductive GNN architectures. Given a batch of seed vertices,
//! samples a fixed number of neighbors per vertex at each hop, producing a
//! computation graph (subgraph) suitable for message-passing.

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphResult, VertexId};

/// Result of k-hop neighborhood sampling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborSampleResult {
    /// Sampled vertices at each layer, from outermost (k-hop) to innermost
    /// (seed). `layers[0]` are the seeds, `layers[1]` are their sampled
    /// neighbors, etc.
    pub layers: Vec<Vec<VertexId>>,
    /// Edges connecting each layer pair. `edges[i]` contains `(src, dst)`
    /// pairs where `src` is in `layers[i+1]` and `dst` is in `layers[i]`.
    /// Vertex ids are original graph ids.
    pub edges: Vec<Vec<(VertexId, VertexId)>>,
}

/// Sample k-hop neighborhoods around seed vertices.
///
/// For each hop from 1 to `fan_out.len()`, samples at most `fan_out[hop-1]`
/// neighbors per frontier vertex. Sampling is uniform without replacement
/// when the number of neighbors exceeds the fan-out; otherwise all
/// neighbors are included.
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `seeds` — Starting vertices (the "batch").
/// - `fan_out` — Number of neighbors to sample at each hop. Length
///   determines the number of hops.
/// - `seed` — PRNG seed for deterministic sampling.
///
/// # Returns
///
/// A [`NeighborSampleResult`] with layers and inter-layer edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, neighbor_sample};
///
/// // Star graph: center 0 connected to 1,2,3,4
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
///
/// // 1-hop sampling from vertex 0, fan_out=2
/// let result = neighbor_sample(&g, &[0], &[2], 42).unwrap();
/// assert_eq!(result.layers[0], vec![0]);
/// assert_eq!(result.layers[1].len(), 2); // sampled 2 of 4 neighbors
/// assert_eq!(result.edges[0].len(), 2);
/// ```
pub fn neighbor_sample(
    graph: &Graph,
    seeds: &[VertexId],
    fan_out: &[usize],
    seed: u64,
) -> IgraphResult<NeighborSampleResult> {
    let n = graph.vcount();

    for &s in seeds {
        if s >= n {
            return Err(crate::core::IgraphError::VertexOutOfRange { id: s, n });
        }
    }

    if seeds.is_empty() || fan_out.is_empty() {
        return Ok(NeighborSampleResult {
            layers: vec![seeds.to_vec()],
            edges: Vec::new(),
        });
    }

    let mut rng = SplitMix64::new(seed);
    let mut layers: Vec<Vec<VertexId>> = Vec::with_capacity(fan_out.len() + 1);
    let mut edges: Vec<Vec<(VertexId, VertexId)>> = Vec::with_capacity(fan_out.len());

    layers.push(seeds.to_vec());

    for &num_samples in fan_out {
        let frontier = layers.last().unwrap();
        let mut next_layer: Vec<VertexId> = Vec::new();
        let mut layer_edges: Vec<(VertexId, VertexId)> = Vec::new();

        for &v in frontier {
            let neighbors = graph.neighbors(v)?;
            if neighbors.is_empty() {
                continue;
            }

            let sampled = if neighbors.len() <= num_samples {
                neighbors
            } else {
                sample_without_replacement(&neighbors, num_samples, &mut rng)
            };

            for &u in &sampled {
                layer_edges.push((u, v));
                next_layer.push(u);
            }
        }

        next_layer.sort_unstable();
        next_layer.dedup();
        edges.push(layer_edges);
        layers.push(next_layer);
    }

    Ok(NeighborSampleResult { layers, edges })
}

/// Sample k-hop neighborhoods with importance-weighted sampling.
///
/// Like [`neighbor_sample`] but samples neighbors proportional to edge
/// weights. Higher-weight edges are more likely to be sampled.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, neighbor_sample_weighted};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3)], false, Some(4)
/// ).unwrap();
/// let weights = vec![10.0, 1.0, 1.0]; // edge 0→1 has much higher weight
///
/// let result = neighbor_sample_weighted(&g, &[0], &[2], &weights, 42).unwrap();
/// assert_eq!(result.layers[0], vec![0]);
/// assert_eq!(result.layers[1].len(), 2);
/// ```
pub fn neighbor_sample_weighted(
    graph: &Graph,
    seeds: &[VertexId],
    fan_out: &[usize],
    weights: &[f64],
    seed: u64,
) -> IgraphResult<NeighborSampleResult> {
    let n = graph.vcount();

    for &s in seeds {
        if s >= n {
            return Err(crate::core::IgraphError::VertexOutOfRange { id: s, n });
        }
    }

    if weights.len() != graph.ecount() {
        return Err(crate::core::IgraphError::InvalidArgument(format!(
            "weights length {} != ecount {}",
            weights.len(),
            graph.ecount()
        )));
    }

    for (i, &w) in weights.iter().enumerate() {
        if w < 0.0 || w.is_nan() {
            return Err(crate::core::IgraphError::InvalidArgument(format!(
                "weight[{i}] = {w} is invalid (must be non-negative and finite)"
            )));
        }
    }

    if seeds.is_empty() || fan_out.is_empty() {
        return Ok(NeighborSampleResult {
            layers: vec![seeds.to_vec()],
            edges: Vec::new(),
        });
    }

    let mut rng = SplitMix64::new(seed);
    let mut layers: Vec<Vec<VertexId>> = Vec::with_capacity(fan_out.len() + 1);
    let mut edges: Vec<Vec<(VertexId, VertexId)>> = Vec::with_capacity(fan_out.len());

    layers.push(seeds.to_vec());

    for &num_samples in fan_out {
        let frontier = layers.last().unwrap();
        let mut next_layer: Vec<VertexId> = Vec::new();
        let mut layer_edges: Vec<(VertexId, VertexId)> = Vec::new();

        for &v in frontier {
            let incident = graph.incident(v)?;
            if incident.is_empty() {
                continue;
            }

            let mut neighbor_weights: Vec<(VertexId, f64)> = Vec::with_capacity(incident.len());
            for &eid in &incident {
                let neighbor = graph.edge_other(eid, v)?;
                neighbor_weights.push((neighbor, weights[eid as usize]));
            }

            let sampled = if neighbor_weights.len() <= num_samples {
                neighbor_weights.iter().map(|&(u, _)| u).collect()
            } else {
                weighted_sample_without_replacement(&neighbor_weights, num_samples, &mut rng)
            };

            for &u in &sampled {
                layer_edges.push((u, v));
                next_layer.push(u);
            }
        }

        next_layer.sort_unstable();
        next_layer.dedup();
        edges.push(layer_edges);
        layers.push(next_layer);
    }

    Ok(NeighborSampleResult { layers, edges })
}

// --- Internal helpers ---

fn sample_without_replacement(items: &[VertexId], k: usize, rng: &mut SplitMix64) -> Vec<VertexId> {
    let n = items.len();
    if k >= n {
        return items.to_vec();
    }

    let mut pool: Vec<VertexId> = items.to_vec();
    for i in 0..k {
        let j = i + rng.gen_index(n - i);
        pool.swap(i, j);
    }
    pool.truncate(k);
    pool
}

fn weighted_sample_without_replacement(
    items: &[(VertexId, f64)],
    k: usize,
    rng: &mut SplitMix64,
) -> Vec<VertexId> {
    let n = items.len();
    if k >= n {
        return items.iter().map(|&(v, _)| v).collect();
    }

    let mut keys: Vec<(f64, VertexId)> = items
        .iter()
        .map(|&(v, w)| {
            let u = rng.gen_unit();
            let key = if w > 0.0 { u.powf(1.0 / w) } else { 0.0 };
            (key, v)
        })
        .collect();

    keys.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    keys.iter().take(k).map(|&(_, v)| v).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
    }

    #[test]
    fn basic_one_hop() {
        let g = star5();
        let result = neighbor_sample(&g, &[0], &[2], 42).unwrap();
        assert_eq!(result.layers[0], vec![0]);
        assert_eq!(result.layers[1].len(), 2);
        assert_eq!(result.edges[0].len(), 2);
        for &(src, dst) in &result.edges[0] {
            assert_eq!(dst, 0);
            assert!((1..=4).contains(&src));
        }
    }

    #[test]
    fn all_neighbors_when_fan_out_large() {
        let g = star5();
        let result = neighbor_sample(&g, &[0], &[10], 42).unwrap();
        assert_eq!(result.layers[1].len(), 4);
    }

    #[test]
    fn two_hop_sampling() {
        let g = path5();
        let result = neighbor_sample(&g, &[0], &[2, 2], 42).unwrap();
        assert_eq!(result.layers.len(), 3);
        assert_eq!(result.edges.len(), 2);
        assert_eq!(result.layers[0], vec![0]);
    }

    #[test]
    fn multiple_seeds() {
        let g = path5();
        let result = neighbor_sample(&g, &[0, 4], &[2], 42).unwrap();
        assert_eq!(result.layers[0], vec![0, 4]);
        assert!(result.layers[1].len() >= 2);
    }

    #[test]
    fn isolated_vertex() {
        let g = Graph::with_vertices(3);
        let result = neighbor_sample(&g, &[0, 1], &[5], 42).unwrap();
        assert_eq!(result.layers[0], vec![0, 1]);
        assert!(result.layers[1].is_empty());
    }

    #[test]
    fn deterministic() {
        let g = star5();
        let r1 = neighbor_sample(&g, &[0], &[2], 99).unwrap();
        let r2 = neighbor_sample(&g, &[0], &[2], 99).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn different_seeds_different_results() {
        let g = star5();
        let r1 = neighbor_sample(&g, &[0], &[2], 1).unwrap();
        let r2 = neighbor_sample(&g, &[0], &[2], 2).unwrap();
        // With 4 neighbors and fan_out=2, different seeds should usually give different samples
        // (probability of collision = C(4,2)^{-1} = 1/6)
        // Not a hard guarantee, but overwhelming probability with these seeds
        let mut s1 = r1.layers[1].clone();
        let mut s2 = r2.layers[1].clone();
        s1.sort_unstable();
        s2.sort_unstable();
        assert_ne!(s1, s2);
    }

    #[test]
    fn empty_seeds() {
        let g = star5();
        let result = neighbor_sample(&g, &[], &[2], 42).unwrap();
        assert_eq!(result.layers.len(), 1);
        assert!(result.layers[0].is_empty());
        assert!(result.edges.is_empty());
    }

    #[test]
    fn empty_fan_out() {
        let g = star5();
        let result = neighbor_sample(&g, &[0], &[], 42).unwrap();
        assert_eq!(result.layers.len(), 1);
        assert_eq!(result.layers[0], vec![0]);
        assert!(result.edges.is_empty());
    }

    #[test]
    fn invalid_seed_vertex() {
        let g = star5();
        let result = neighbor_sample(&g, &[10], &[2], 42);
        assert!(result.is_err());
    }

    #[test]
    fn weighted_basic() {
        let g = star5();
        let weights = vec![10.0, 1.0, 1.0, 1.0]; // 4 edges
        let result = neighbor_sample_weighted(&g, &[0], &[2], &weights, 42).unwrap();
        assert_eq!(result.layers[0], vec![0]);
        assert_eq!(result.layers[1].len(), 2);
    }

    #[test]
    fn weighted_high_weight_preferred() {
        let g = star5();
        // Edge 0→1 has very high weight, others nearly zero
        let weights = vec![1000.0, 0.001, 0.001, 0.001];
        let mut vertex1_count = 0;
        for trial in 0..20u64 {
            let result = neighbor_sample_weighted(&g, &[0], &[1], &weights, trial * 137).unwrap();
            if result.layers[1].contains(&1) {
                vertex1_count += 1;
            }
        }
        // Vertex 1 should be selected in most trials
        assert!(vertex1_count >= 15);
    }

    #[test]
    fn weighted_invalid_weights_length() {
        let g = star5();
        let weights = vec![1.0, 2.0]; // wrong length
        let result = neighbor_sample_weighted(&g, &[0], &[2], &weights, 42);
        assert!(result.is_err());
    }

    #[test]
    fn weighted_negative_weight() {
        let g = star5();
        let weights = vec![1.0, -1.0, 1.0, 1.0];
        let result = neighbor_sample_weighted(&g, &[0], &[2], &weights, 42);
        assert!(result.is_err());
    }

    #[test]
    fn deduplication_across_frontier() {
        // Triangle: 0-1, 1-2, 0-2. Seeds=[0,1], fan_out=[3]
        // Both 0 and 1 have vertex 2 as neighbor
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap();
        let result = neighbor_sample(&g, &[0, 1], &[3], 42).unwrap();
        // layers[1] should be deduplicated
        let mut sorted = result.layers[1].clone();
        sorted.sort_unstable();
        let mut deduped = sorted.clone();
        deduped.dedup();
        assert_eq!(sorted, deduped);
    }
}
