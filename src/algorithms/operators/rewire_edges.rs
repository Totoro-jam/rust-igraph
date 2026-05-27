//! Probabilistic edge rewiring (ALGO-OP-012).
//!
//! Rewires each edge endpoint with a given probability.

use crate::core::{Graph, IgraphResult, VertexId};

/// Rewires graph edges with constant probability.
///
/// Each endpoint of each edge is independently rewired to a uniformly
/// random vertex with probability `prob`. The result may contain self-loops
/// and multi-edges depending on the `loops` parameter.
///
/// Uses the provided RNG seed for reproducibility.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `prob` — rewiring probability in `[0.0, 1.0]`.
/// * `loops` — if `true`, rewired edges may form self-loops; otherwise,
///   the new endpoint is guaranteed different from the other endpoint.
/// * `seed` — random seed for deterministic output.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rewire_edges};
///
/// let mut g = Graph::with_vertices(10);
/// for i in 0..9u32 {
///     g.add_edge(i, i + 1).unwrap();
/// }
///
/// let rg = rewire_edges(&g, 0.5, false, 42).unwrap();
/// assert_eq!(rg.vcount(), 10);
/// assert_eq!(rg.ecount(), 9);
/// ```
pub fn rewire_edges(graph: &Graph, prob: f64, loops: bool, seed: u64) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let ecount = graph.ecount();

    if n == 0 || ecount == 0 || prob == 0.0 {
        // Return a structural copy
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let eid_u32 = eid as u32;
            edges.push(graph.edge(eid_u32)?);
        }
        let mut result = Graph::new(n, directed)?;
        result.add_edges(edges)?;
        return Ok(result);
    }

    // Collect edge list
    let mut edge_list: Vec<[VertexId; 2]> = Vec::with_capacity(ecount);
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        edge_list.push([src, tgt]);
    }

    // Simple splitmix64 RNG for reproducibility
    let mut rng_state = seed;

    let endpoints = ecount * 2;
    // Use geometric distribution to skip to next rewired endpoint
    let mut to_rewire = geom_sample(&mut rng_state, prob);

    while to_rewire < endpoints {
        let edge_idx = to_rewire / 2;
        let endpoint_idx = to_rewire % 2; // 0 = src, 1 = tgt

        let other_idx = 1 - endpoint_idx;
        let other_vertex = edge_list[edge_idx][other_idx];

        let new_vertex = if loops {
            random_vertex(&mut rng_state, n)
        } else {
            random_vertex_excluding(&mut rng_state, n, other_vertex)
        };

        edge_list[edge_idx][endpoint_idx] = new_vertex;
        to_rewire += geom_sample(&mut rng_state, prob) + 1;
    }

    // Build result graph
    let edges: Vec<(VertexId, VertexId)> = edge_list.iter().map(|e| (e[0], e[1])).collect();

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn random_f64(state: &mut u64) -> f64 {
    let r = splitmix64(state);
    // Convert to [0, 1) using top 53 bits
    (r >> 11) as f64 / (1u64 << 53) as f64
}

fn random_vertex(state: &mut u64, n: u32) -> VertexId {
    let r = splitmix64(state);
    #[allow(clippy::cast_possible_truncation)]
    let v = (r % u64::from(n)) as u32;
    v
}

fn random_vertex_excluding(state: &mut u64, n: u32, exclude: VertexId) -> VertexId {
    let r = splitmix64(state);
    #[allow(clippy::cast_possible_truncation)]
    let mut v = (r % u64::from(n - 1)) as u32;
    if v >= exclude {
        v += 1;
    }
    v
}

/// Sample from geometric distribution: number of failures before first success.
fn geom_sample(state: &mut u64, prob: f64) -> usize {
    if prob >= 1.0 {
        return 0;
    }
    let u = random_f64(state);
    if u == 0.0 {
        return 0;
    }
    // floor(log(1-u) / log(1-p))
    let result = ((1.0 - u).ln() / (1.0 - prob).ln()).floor();
    // Clamp to reasonable range
    if result < 0.0 || result.is_nan() {
        0
    } else if result > usize::MAX as f64 {
        usize::MAX
    } else {
        result as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prob_zero() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rg = rewire_edges(&g, 0.0, false, 42).unwrap();
        assert_eq!(rg.vcount(), 5);
        assert_eq!(rg.ecount(), 2);
        assert_eq!(rg.edge(0).unwrap(), g.edge(0).unwrap());
        assert_eq!(rg.edge(1).unwrap(), g.edge(1).unwrap());
    }

    #[test]
    fn test_prob_one_no_loops() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let rg = rewire_edges(&g, 1.0, false, 123).unwrap();
        assert_eq!(rg.vcount(), 10);
        assert_eq!(rg.ecount(), 9);
        // No self-loops
        for eid in 0..9u32 {
            let (s, t) = rg.edge(eid).unwrap();
            assert_ne!(s, t);
        }
    }

    #[test]
    fn test_deterministic() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let r1 = rewire_edges(&g, 0.5, false, 42).unwrap();
        let r2 = rewire_edges(&g, 0.5, false, 42).unwrap();
        // Same seed → same result
        for eid in 0..9u32 {
            assert_eq!(r1.edge(eid).unwrap(), r2.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut g = Graph::with_vertices(100);
        for i in 0..99u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let r1 = rewire_edges(&g, 0.5, false, 1).unwrap();
        let r2 = rewire_edges(&g, 0.5, false, 2).unwrap();
        // Different seeds → likely different results
        let mut same_count = 0;
        for eid in 0..99u32 {
            if r1.edge(eid).unwrap() == r2.edge(eid).unwrap() {
                same_count += 1;
            }
        }
        // With 99 edges and prob 0.5, extremely unlikely all same
        assert!(same_count < 99);
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let rg = rewire_edges(&g, 0.5, false, 42).unwrap();
        assert_eq!(rg.vcount(), 0);
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(5);
        let rg = rewire_edges(&g, 0.5, false, 42).unwrap();
        assert_eq!(rg.ecount(), 0);
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let rg = rewire_edges(&g, 0.5, false, 99).unwrap();
        assert!(rg.is_directed());
        assert_eq!(rg.ecount(), 3);
    }

    #[test]
    fn test_loops_allowed() {
        // With prob=1.0 and loops allowed, some edges may become self-loops
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rg = rewire_edges(&g, 1.0, true, 42).unwrap();
        assert_eq!(rg.vcount(), 3);
        assert_eq!(rg.ecount(), 2);
    }

    #[test]
    fn test_preserves_edge_count() {
        let mut g = Graph::with_vertices(20);
        for i in 0..19u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        for seed in 0..10u64 {
            let rg = rewire_edges(&g, 0.3, false, seed).unwrap();
            assert_eq!(rg.ecount(), 19);
        }
    }
}
