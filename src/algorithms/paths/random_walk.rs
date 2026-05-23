//! Random walk on a graph (ALGO-TR-003).
//!
//! Counterpart of `igraph_random_walk()` from
//! `references/igraph/src/paths/random_walk.c:288`. Performs a
//! discrete-time random walk starting at `start` for at most `steps`
//! transitions. At each vertex the next neighbour is chosen
//! - uniformly at random when `weights == None`,
//! - with probability proportional to edge weight when `weights ==
//!   Some(_)`.
//!
//! If the walk reaches a vertex with no outgoing neighbours under the
//! chosen `mode`, the walk stops early (matches upstream's
//! `IGRAPH_RANDOM_WALK_STUCK_RETURN` variant). The returned vertex
//! chain always starts with `start`; the parallel edge chain has
//! length `vertices.len() - 1`.
//!
//! For deterministic reproducibility the API takes a `seed: u64`
//! parameter. We seed an inline `SplitMix64` PRNG — no external
//! `rand` dependency. Callers wanting non-deterministic behaviour can
//! pass `seed = std::time::SystemTime::now()`-derived bytes.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate_weights(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<()> {
    let Some(w) = weights else {
        return Ok(());
    };
    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            w.len(),
            m
        )));
    }
    for (e, &v) in w.iter().enumerate() {
        if v.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
        if v < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({v}); random walk weights must be non-negative"
            )));
        }
    }
    Ok(())
}

fn incident_for_mode(graph: &Graph, v: VertexId, mode: DijkstraMode) -> IgraphResult<Vec<EdgeId>> {
    if !graph.is_directed() {
        return graph.incident(v);
    }
    match mode {
        DijkstraMode::Out => graph.incident(v),
        DijkstraMode::In => graph.incident_in(v),
        DijkstraMode::All => {
            let mut out = graph.incident(v)?;
            out.extend(graph.incident_in(v)?);
            Ok(out)
        }
    }
}

/// Random walk on `graph` starting at `start`, taking up to `steps`
/// transitions.
///
/// `weights = None` selects neighbours uniformly. `weights = Some(_)`
/// selects each candidate edge with probability proportional to its
/// weight; a candidate edge with weight `0.0` is never chosen, but at
/// least one outgoing edge of every visited vertex must have positive
/// weight or the walk gets stuck. Negative or NaN weights return
/// [`IgraphError::InvalidArgument`].
///
/// Returns `(vertex_chain, edge_chain)`. `vertex_chain[0] == start`;
/// when the walk completes all `steps` transitions, the chain has
/// length `steps + 1` and `edge_chain` has length `steps`. If the walk
/// gets stuck early (no admissible outgoing edges), the result is
/// truncated to the actual prefix (matches upstream's
/// `IGRAPH_RANDOM_WALK_STUCK_RETURN`).
///
/// `seed` deterministically initialises the internal `SplitMix64`
/// PRNG — same `(graph, start, mode, steps, weights, seed)` always
/// produces the same chain.
///
/// Counterpart of `igraph_random_walk(_, &weights, _, _, start, mode,
/// steps, IGRAPH_RANDOM_WALK_STUCK_RETURN)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_walk, DijkstraMode};
///
/// // 4-cycle 0-1-2-3-0: every vertex has two neighbours so the walk
/// // never gets stuck.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// let (vs, es) = random_walk(&g, None, 0, DijkstraMode::Out, 5, 42).unwrap();
/// assert_eq!(vs.len(), 6);
/// assert_eq!(es.len(), 5);
/// assert_eq!(vs[0], 0);
/// ```
pub fn random_walk(
    graph: &Graph,
    weights: Option<&[f64]>,
    start: VertexId,
    mode: DijkstraMode,
    steps: u32,
    seed: u64,
) -> IgraphResult<(Vec<VertexId>, Vec<EdgeId>)> {
    let n = graph.vcount();
    if start >= n {
        return Err(IgraphError::VertexOutOfRange { id: start, n });
    }
    validate_weights(graph, weights)?;

    let mut rng = SplitMix64::new(seed);
    let mut vs: Vec<VertexId> = Vec::with_capacity(steps as usize + 1);
    let mut es: Vec<EdgeId> = Vec::with_capacity(steps as usize);
    vs.push(start);

    let mut current = start;
    for _ in 0..steps {
        let incidents = incident_for_mode(graph, current, mode)?;
        if incidents.is_empty() {
            break;
        }

        let next_eid = match weights {
            None => {
                let idx = rng.gen_index(incidents.len());
                Some(incidents[idx])
            }
            Some(ws) => {
                // Sum admissible weights (skip non-finite). If sum is 0,
                // there is no admissible outgoing edge and the walk
                // stops early.
                let mut total = 0.0_f64;
                for &eid in &incidents {
                    let w = ws[eid as usize];
                    if w.is_finite() && w > 0.0 {
                        total += w;
                    }
                }
                if total <= 0.0 {
                    None
                } else {
                    let target = rng.gen_unit() * total;
                    let mut acc = 0.0_f64;
                    let mut chosen: Option<EdgeId> = None;
                    for &eid in &incidents {
                        let w = ws[eid as usize];
                        if !(w.is_finite() && w > 0.0) {
                            continue;
                        }
                        acc += w;
                        if acc >= target {
                            chosen = Some(eid);
                            break;
                        }
                    }
                    // Floating-point edge case: if rounding put `acc` a
                    // hair below `target`, fall back to the last
                    // positive-weight edge.
                    if chosen.is_none() {
                        for &eid in incidents.iter().rev() {
                            let w = ws[eid as usize];
                            if w.is_finite() && w > 0.0 {
                                chosen = Some(eid);
                                break;
                            }
                        }
                    }
                    chosen
                }
            }
        };

        let Some(eid) = next_eid else {
            break;
        };
        let next_vertex = graph.edge_other(eid, current)?;
        es.push(eid);
        vs.push(next_vertex);
        current = next_vertex;
    }

    Ok((vs, es))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_weights_4_cycle_walk_length() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let (vs, es) = random_walk(&g, None, 0, DijkstraMode::Out, 5, 42).unwrap();
        assert_eq!(vs.len(), 6);
        assert_eq!(es.len(), 5);
        assert_eq!(vs[0], 0);
        // Every step is along an edge: consecutive vertices are
        // adjacent in the input graph.
        for w in vs.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(
                (g.find_eid(a, b).unwrap()).is_some() || (g.find_eid(b, a).unwrap()).is_some(),
                "non-adjacent step {a}→{b}"
            );
        }
    }

    #[test]
    fn stuck_returns_shorter_walk() {
        // Sink vertex 1 has no outgoing edges in OUT mode.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        let (vs, es) = random_walk(&g, None, 0, DijkstraMode::Out, 10, 7).unwrap();
        // 0→1 is the only step; sink 1 has no out-edges → stops.
        assert_eq!(vs, vec![0, 1]);
        assert_eq!(es.len(), 1);
    }

    #[test]
    fn zero_steps_returns_singleton() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let (vs, es) = random_walk(&g, None, 1, DijkstraMode::Out, 0, 0).unwrap();
        assert_eq!(vs, vec![1]);
        assert!(es.is_empty());
    }

    #[test]
    fn isolated_vertex_stuck_immediately() {
        let g = Graph::with_vertices(3);
        let (vs, es) = random_walk(&g, None, 1, DijkstraMode::Out, 5, 0).unwrap();
        assert_eq!(vs, vec![1]);
        assert!(es.is_empty());
    }

    #[test]
    fn deterministic_with_same_seed() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
            g.add_edge(i, (i + 2) % 5).unwrap();
        }
        let r1 = random_walk(&g, None, 0, DijkstraMode::Out, 20, 12345).unwrap();
        let r2 = random_walk(&g, None, 0, DijkstraMode::Out, 20, 12345).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn different_seeds_yield_different_walks() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
            g.add_edge(i, (i + 2) % 5).unwrap();
        }
        let r1 = random_walk(&g, None, 0, DijkstraMode::Out, 20, 1).unwrap();
        let r2 = random_walk(&g, None, 0, DijkstraMode::Out, 20, 2).unwrap();
        // Probabilistically these almost surely differ; deterministic
        // because the PRNG is seeded.
        assert_ne!(r1, r2);
    }

    #[test]
    fn weighted_walk_picks_only_positive_weight_edges() {
        // 0-1 weight 1, 0-2 weight 0: 0→2 must never be chosen.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = [1.0_f64, 0.0];
        // Need a return path from 1; add 1-0 reverse via the same edge
        // (undirected, so 0-1 ≡ 1-0).
        // Run a couple of one-step walks; every one must end at 1.
        for seed in 0..16u64 {
            let (vs, _) = random_walk(&g, Some(&weights), 0, DijkstraMode::Out, 1, seed).unwrap();
            assert_eq!(vs, vec![0, 1], "seed {seed}");
        }
    }

    #[test]
    fn weighted_zero_total_weight_stops_early() {
        // All outgoing edges have weight 0 → no admissible edge.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = [0.0_f64, 0.0];
        let (vs, es) = random_walk(&g, Some(&weights), 0, DijkstraMode::Out, 5, 0).unwrap();
        assert_eq!(vs, vec![0]);
        assert!(es.is_empty());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [-1.0_f64];
        assert!(random_walk(&g, Some(&weights), 0, DijkstraMode::Out, 1, 0).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [f64::NAN];
        assert!(random_walk(&g, Some(&weights), 0, DijkstraMode::Out, 1, 0).is_err());
    }

    #[test]
    fn out_of_range_start_errors() {
        let g = Graph::with_vertices(3);
        assert!(random_walk(&g, None, 99, DijkstraMode::Out, 1, 0).is_err());
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [1.0_f64, 2.0];
        assert!(random_walk(&g, Some(&weights), 0, DijkstraMode::Out, 1, 0).is_err());
    }

    #[test]
    fn directed_in_mode_walks_reverse_edges() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // From sink 2 with IN-mode: reaches 1 in step 1, 0 in step 2.
        let (vs, _) = random_walk(&g, None, 2, DijkstraMode::In, 2, 0).unwrap();
        assert_eq!(vs, vec![2, 1, 0]);
    }
}
