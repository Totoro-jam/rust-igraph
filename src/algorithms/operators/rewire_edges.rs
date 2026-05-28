//! Probabilistic edge rewiring (ALGO-OP-012, ALGO-OP-028).
//!
//! Rewires each edge endpoint with a given probability.
//! Also provides directed-edge rewiring that preserves either
//! the in-degree or out-degree sequence.

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

/// Which endpoint of a directed edge to rewire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewireDirectedMode {
    /// Rewire the target (head) of each edge; preserves out-degree sequence.
    Out,
    /// Rewire the source (tail) of each edge; preserves in-degree sequence.
    In,
}

/// Rewires one endpoint of directed edges with constant probability.
///
/// For directed graphs, rewires either the source or target of each edge
/// independently with probability `prob`. This preserves the in-degree
/// sequence (when rewiring targets, i.e. `Out` mode) or the out-degree
/// sequence (when rewiring sources, i.e. `In` mode).
///
/// For undirected graphs, falls back to [`rewire_edges`] which rewires
/// both endpoints.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `prob` — rewiring probability in `[0.0, 1.0]`.
/// * `loops` — if `true`, rewired edges may form self-loops.
/// * `mode` — which endpoint to rewire (`Out` = target, `In` = source).
/// * `seed` — random seed for deterministic output.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rewire_directed_edges, RewireDirectedMode};
///
/// let mut g = Graph::new(5, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let rg = rewire_directed_edges(&g, 0.5, false, RewireDirectedMode::Out, 42).unwrap();
/// assert_eq!(rg.vcount(), 5);
/// assert_eq!(rg.ecount(), 3);
/// assert!(rg.is_directed());
/// ```
pub fn rewire_directed_edges(
    graph: &Graph,
    prob: f64,
    loops: bool,
    mode: RewireDirectedMode,
    seed: u64,
) -> IgraphResult<Graph> {
    use crate::core::error::IgraphError;

    if !(0.0..=1.0).contains(&prob) {
        return Err(IgraphError::InvalidArgument(
            "rewiring probability must be in [0.0, 1.0]".to_string(),
        ));
    }

    // For undirected graphs, fall back to rewire_edges (rewires both endpoints)
    if !graph.is_directed() {
        return rewire_edges(graph, prob, loops, seed);
    }

    let n = graph.vcount();
    let ecount = graph.ecount();

    if n == 0 || ecount == 0 || prob == 0.0 {
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let eid_u32 = eid as u32;
            edges.push(graph.edge(eid_u32)?);
        }
        let mut result = Graph::new(n, true)?;
        result.add_edges(edges)?;
        return Ok(result);
    }

    // offset: 0 = rewire source (IN mode), 1 = rewire target (OUT mode)
    let offset = match mode {
        RewireDirectedMode::In => 0,
        RewireDirectedMode::Out => 1,
    };

    let mut edge_list: Vec<[VertexId; 2]> = Vec::with_capacity(ecount);
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        edge_list.push([src, tgt]);
    }

    let mut rng_state = seed;

    let mut to_rewire = geom_sample(&mut rng_state, prob);
    while to_rewire < ecount {
        let other_vertex = edge_list[to_rewire][1 - offset];

        let new_vertex = if loops {
            random_vertex(&mut rng_state, n)
        } else {
            random_vertex_excluding(&mut rng_state, n, other_vertex)
        };

        edge_list[to_rewire][offset] = new_vertex;
        to_rewire += geom_sample(&mut rng_state, prob) + 1;
    }

    let edges: Vec<(VertexId, VertexId)> = edge_list.iter().map(|e| (e[0], e[1])).collect();

    let mut result = Graph::new(n, true)?;
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

    // --- rewire_directed_edges tests ---

    #[test]
    fn test_directed_rewire_prob_zero() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let rg = rewire_directed_edges(&g, 0.0, false, RewireDirectedMode::Out, 42).unwrap();
        assert_eq!(rg.ecount(), 3);
        for eid in 0..3u32 {
            assert_eq!(rg.edge(eid).unwrap(), g.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_directed_rewire_preserves_out_degree() {
        // In OUT mode, we rewire targets → out-degree is preserved
        let mut g = Graph::new(10, true).unwrap();
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let rg = rewire_directed_edges(&g, 1.0, false, RewireDirectedMode::Out, 42).unwrap();
        assert_eq!(rg.ecount(), 9);
        assert!(rg.is_directed());

        // Each original source still has the same out-degree
        for v in 0..10u32 {
            let orig_out: usize = (0..9).filter(|&eid| g.edge(eid).unwrap().0 == v).count();
            let new_out: usize = (0..9).filter(|&eid| rg.edge(eid).unwrap().0 == v).count();
            assert_eq!(orig_out, new_out);
        }
    }

    #[test]
    fn test_directed_rewire_preserves_in_degree() {
        // In IN mode, we rewire sources → in-degree is preserved
        let mut g = Graph::new(10, true).unwrap();
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let rg = rewire_directed_edges(&g, 1.0, false, RewireDirectedMode::In, 42).unwrap();
        assert_eq!(rg.ecount(), 9);

        // Each original target still has the same in-degree
        for v in 0..10u32 {
            let orig_in: usize = (0..9).filter(|&eid| g.edge(eid).unwrap().1 == v).count();
            let new_in: usize = (0..9).filter(|&eid| rg.edge(eid).unwrap().1 == v).count();
            assert_eq!(orig_in, new_in);
        }
    }

    #[test]
    fn test_directed_rewire_no_self_loops() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();

        for seed in 0..20u64 {
            let rg = rewire_directed_edges(&g, 1.0, false, RewireDirectedMode::Out, seed).unwrap();
            for eid in 0..4u32 {
                let (s, t) = rg.edge(eid).unwrap();
                assert_ne!(s, t);
            }
        }
    }

    #[test]
    fn test_directed_rewire_undirected_fallback() {
        // Undirected graph falls back to rewire_edges
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rg = rewire_directed_edges(&g, 0.5, false, RewireDirectedMode::Out, 42).unwrap();
        assert!(!rg.is_directed());
        assert_eq!(rg.ecount(), 2);
    }

    #[test]
    fn test_directed_rewire_invalid_prob() {
        let g = Graph::new(5, true).unwrap();
        assert!(rewire_directed_edges(&g, -0.1, false, RewireDirectedMode::Out, 42).is_err());
        assert!(rewire_directed_edges(&g, 1.1, false, RewireDirectedMode::Out, 42).is_err());
    }

    #[test]
    fn test_directed_rewire_deterministic() {
        let mut g = Graph::new(10, true).unwrap();
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let r1 = rewire_directed_edges(&g, 0.5, false, RewireDirectedMode::Out, 99).unwrap();
        let r2 = rewire_directed_edges(&g, 0.5, false, RewireDirectedMode::Out, 99).unwrap();
        for eid in 0..9u32 {
            assert_eq!(r1.edge(eid).unwrap(), r2.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_directed_rewire_empty() {
        let g = Graph::new(0, true).unwrap();
        let rg = rewire_directed_edges(&g, 0.5, false, RewireDirectedMode::Out, 42).unwrap();
        assert_eq!(rg.vcount(), 0);
    }
}
