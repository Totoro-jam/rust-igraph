//! Degree-preserving edge rewiring (ALGO-OP-016).
//!
//! Randomly rewires edges while preserving each vertex's degree. Uses
//! the switching algorithm: pick two edges, swap endpoints if the swap
//! doesn't violate constraints (no multi-edges, optionally no self-loops).

use std::collections::HashSet;

use crate::core::{Graph, IgraphResult, VertexId};

/// Randomly rewires a graph while preserving its degree sequence.
///
/// Performs `num_trials` switching attempts. Each trial picks two random
/// edges `(a, b)` and `(c, d)` and attempts to replace them with `(a, d)`
/// and `(c, b)`. The swap is rejected if it would create a multi-edge or
/// (when `loops` is false) a self-loop.
///
/// Returns a new graph; the original is not modified.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `num_trials` — number of rewiring trials (not all succeed).
/// * `loops` — if `true`, self-loops may be created; if `false`, swaps
///   that would create self-loops are rejected.
/// * `seed` — random seed for deterministic output.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rewire};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
///
/// let rg = rewire(&g, 100, false, 42).unwrap();
/// assert_eq!(rg.vcount(), 6);
/// assert_eq!(rg.ecount(), 5);
/// // Degree sequence is preserved
/// ```
pub fn rewire(graph: &Graph, num_trials: usize, loops: bool, seed: u64) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let ecount = graph.ecount();
    let directed = graph.is_directed();

    if ecount < 2 {
        return copy_graph(graph);
    }

    // Collect edges as mutable array
    let mut edge_list: Vec<[VertexId; 2]> = Vec::with_capacity(ecount);
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        edge_list.push([src, tgt]);
    }

    // Build edge set for O(1) existence checks
    let mut edge_set: HashSet<(VertexId, VertexId)> = HashSet::with_capacity(ecount);
    for e in &edge_list {
        edge_set.insert(canonical_edge(e[0], e[1], directed));
    }

    let mut rng_state = seed;

    for _ in 0..num_trials {
        // Pick two distinct random edge indices
        let idx1 = random_index(&mut rng_state, ecount);
        let mut idx2 = random_index(&mut rng_state, ecount - 1);
        if idx2 >= idx1 {
            idx2 += 1;
        }

        let src1 = edge_list[idx1][0];
        let tgt1 = edge_list[idx1][1];
        let (src2, tgt2) = if !directed && random_bool(&mut rng_state) {
            (edge_list[idx2][1], edge_list[idx2][0])
        } else {
            (edge_list[idx2][0], edge_list[idx2][1])
        };

        // Check: skip loops if not allowed
        if !loops && (src1 == tgt1 || src2 == tgt2) {
            continue;
        }

        // Check: swap would have no effect
        if src1 == src2 || tgt1 == tgt2 {
            continue;
        }

        // Check: swap would create a self-loop
        if !loops && (src1 == tgt2 || tgt1 == src2) {
            continue;
        }

        // For undirected: if both are self-loops, swap creates multi-edge
        if !directed && src1 == tgt1 && src2 == tgt2 {
            continue;
        }

        // Check: new edges don't already exist (no multi-edges)
        let new_edge1 = canonical_edge(src1, tgt2, directed);
        let new_edge2 = canonical_edge(src2, tgt1, directed);

        if edge_set.contains(&new_edge1) {
            continue;
        }
        if edge_set.contains(&new_edge2) {
            continue;
        }
        if new_edge1 == new_edge2 {
            continue;
        }

        // Perform the swap
        let old_edge1 = canonical_edge(src1, tgt1, directed);
        let old_edge2 = canonical_edge(src2, tgt2, directed);

        edge_set.remove(&old_edge1);
        edge_set.remove(&old_edge2);
        edge_set.insert(new_edge1);
        edge_set.insert(new_edge2);

        edge_list[idx1] = [src1, tgt2];
        edge_list[idx2] = [src2, tgt1];
    }

    // Build result graph
    let edges: Vec<(VertexId, VertexId)> = edge_list.iter().map(|e| (e[0], e[1])).collect();
    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

fn canonical_edge(src: VertexId, tgt: VertexId, directed: bool) -> (VertexId, VertexId) {
    if directed || src <= tgt {
        (src, tgt)
    } else {
        (tgt, src)
    }
}

fn copy_graph(graph: &Graph) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let ecount = graph.ecount();
    let directed = graph.is_directed();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        edges.push(graph.edge(eid as u32)?);
    }
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

fn random_index(state: &mut u64, n: usize) -> usize {
    let r = splitmix64(state);
    #[allow(clippy::cast_possible_truncation)]
    {
        (r as usize) % n
    }
}

fn random_bool(state: &mut u64) -> bool {
    splitmix64(state) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn degree_sequence(graph: &Graph) -> Vec<u32> {
        let n = graph.vcount();
        let mut deg = vec![0u32; n as usize];
        let ecount = graph.ecount();
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (src, tgt) = graph.edge(eid as u32).unwrap();
            deg[src as usize] += 1;
            if graph.is_directed() || src != tgt {
                deg[tgt as usize] += 1;
            }
        }
        deg
    }

    #[test]
    fn test_preserves_degree_sequence() {
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(6, 7).unwrap();
        g.add_edge(7, 8).unwrap();
        g.add_edge(8, 9).unwrap();

        let original_deg = degree_sequence(&g);
        let rg = rewire(&g, 200, false, 42).unwrap();
        let rewired_deg = degree_sequence(&rg);

        assert_eq!(original_deg, rewired_deg);
        assert_eq!(rg.ecount(), g.ecount());
    }

    #[test]
    fn test_preserves_edge_count() {
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(6, 7).unwrap();
        g.add_edge(7, 4).unwrap();

        let rg = rewire(&g, 100, false, 99).unwrap();
        assert_eq!(rg.ecount(), 8);
    }

    #[test]
    fn test_no_self_loops() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();

        let rg = rewire(&g, 500, false, 123).unwrap();
        for eid in 0..rg.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = rg.edge(eid as u32).unwrap();
            assert_ne!(s, t, "self-loop at edge {eid}");
        }
    }

    #[test]
    fn test_deterministic() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9u32 {
            g.add_edge(i, i + 1).unwrap();
        }

        let r1 = rewire(&g, 50, false, 42).unwrap();
        let r2 = rewire(&g, 50, false, 42).unwrap();

        for eid in 0..r1.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let e = eid as u32;
            assert_eq!(r1.edge(e).unwrap(), r2.edge(e).unwrap());
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut g = Graph::with_vertices(20);
        for i in 0..19u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        g.add_edge(19, 0).unwrap();

        let r1 = rewire(&g, 200, false, 1).unwrap();
        let r2 = rewire(&g, 200, false, 2).unwrap();

        let mut differ = false;
        for eid in 0..r1.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let e = eid as u32;
            if r1.edge(e).unwrap() != r2.edge(e).unwrap() {
                differ = true;
                break;
            }
        }
        assert!(differ, "different seeds should produce different results");
    }

    #[test]
    fn test_single_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let rg = rewire(&g, 100, false, 42).unwrap();
        assert_eq!(rg.ecount(), 1);
        assert_eq!(rg.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn test_zero_trials() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let rg = rewire(&g, 0, false, 42).unwrap();
        for eid in 0..3u32 {
            assert_eq!(rg.edge(eid).unwrap(), g.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();

        let original_deg = degree_sequence(&g);
        let rg = rewire(&g, 100, false, 77).unwrap();
        assert!(rg.is_directed());
        assert_eq!(rg.ecount(), 5);
        assert_eq!(degree_sequence(&rg), original_deg);
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let rg = rewire(&g, 100, false, 42).unwrap();
        assert_eq!(rg.vcount(), 0);
        assert_eq!(rg.ecount(), 0);
    }

    #[test]
    fn test_no_multi_edges() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let rg = rewire(&g, 1000, false, 42).unwrap();

        // Check no multi-edges
        let mut seen = HashSet::new();
        for eid in 0..rg.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = rg.edge(eid as u32).unwrap();
            let key = if s <= t { (s, t) } else { (t, s) };
            assert!(seen.insert(key), "multi-edge found: {key:?}");
        }
    }

    #[test]
    fn test_loops_allowed() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        // With loops and enough trials, some self-loops may appear
        let rg = rewire(&g, 1000, true, 42).unwrap();
        assert_eq!(rg.ecount(), 4);
    }
}
