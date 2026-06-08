//! Second-order biased random walk (`Node2Vec`) on a graph (ALGO-TR-004).
//!
//! Implements the biased walk described in Grover & Leskovec (2016) "node2vec:
//! Scalable Feature Learning for Networks". At each step the transition
//! probability from vertex `t → v → x` is proportional to `α(t, x) · w(v, x)`,
//! where `w(v, x)` is the edge weight and `α` is:
//!
//! - `1/p` if `x == t`  (return to previous vertex — controlled by `p`)
//! - `1`   if `x` is a neighbor of `t` (stay close — BFS-like when `q > 1`)
//! - `1/q` if `x` is NOT a neighbor of `t` (move away — DFS-like when `q < 1`)
//!
//! When `p = q = 1` this reduces to a standard (first-order) random walk.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::random_walk::validate_weights;

/// Result of a `Node2Vec` random walk: the vertex chain and the edge chain.
pub type Node2VecWalkResult = (Vec<VertexId>, Vec<EdgeId>);

/// Performs a second-order biased random walk (`Node2Vec`) starting at `start`
/// for up to `steps` transitions.
///
/// # Parameters
///
/// - `p` — Return parameter. Higher `p` makes it less likely to return to the
///   previously visited vertex. `p > 1` discourages backtracking; `p < 1`
///   encourages it.
/// - `q` — In-out parameter. Higher `q` biases towards BFS-like exploration
///   (staying close to `t`); `q < 1` biases towards DFS-like exploration
///   (moving away from `t`).
/// - `weights` — Optional edge weights (positive). `None` for unweighted.
/// - `mode` — Direction mode for directed graphs (`Out`, `In`, `All`).
/// - `seed` — Deterministic RNG seed.
///
/// # Returns
///
/// `(vertex_chain, edge_chain)` where `vertex_chain[0] == start`. If the walk
/// gets stuck (no admissible outgoing edges), the result is truncated.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `p` or `q` are not positive
/// finite, if `start` is out of range, or if weights are invalid.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_walk_node2vec, DijkstraMode};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(0, 2).unwrap(); // shortcut
///
/// // With q < 1: biased towards DFS (exploring further away)
/// let (vs, es) = random_walk_node2vec(
///     &g, None, 0, DijkstraMode::Out, 10, 1.0, 0.5, 42
/// ).unwrap();
/// assert_eq!(vs[0], 0);
/// assert!(vs.len() <= 11);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn random_walk_node2vec(
    graph: &Graph,
    weights: Option<&[f64]>,
    start: VertexId,
    mode: DijkstraMode,
    steps: u32,
    p: f64,
    q: f64,
    seed: u64,
) -> IgraphResult<Node2VecWalkResult> {
    let n = graph.vcount();
    if start >= n {
        return Err(IgraphError::VertexOutOfRange { id: start, n });
    }
    if !p.is_finite() || p <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "p must be positive and finite, got {p}"
        )));
    }
    if !q.is_finite() || q <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "q must be positive and finite, got {q}"
        )));
    }
    validate_weights(graph, weights)?;

    let mut rng = SplitMix64::new(seed);
    let mut vs: Vec<VertexId> = Vec::with_capacity(steps as usize + 1);
    let mut es: Vec<EdgeId> = Vec::with_capacity(steps as usize);
    vs.push(start);

    if steps == 0 {
        return Ok((vs, es));
    }

    // First step: standard (unbiased or weight-proportional) since there's
    // no "previous" vertex yet.
    let first_next = pick_neighbor(graph, start, weights, mode, &mut rng)?;
    let Some((first_eid, first_v)) = first_next else {
        return Ok((vs, es));
    };
    es.push(first_eid);
    vs.push(first_v);

    // Subsequent steps: second-order biased
    let inv_p = 1.0 / p;
    let inv_q = 1.0 / q;

    for _ in 1..steps {
        let prev = vs[vs.len() - 2]; // t
        let current = *vs.last().unwrap(); // v

        let next =
            pick_biased_neighbor(graph, prev, current, weights, mode, inv_p, inv_q, &mut rng)?;
        let Some((eid, next_v)) = next else {
            break;
        };
        es.push(eid);
        vs.push(next_v);
    }

    Ok((vs, es))
}

/// Pick a neighbor uniformly (unweighted) or proportional to weight.
fn pick_neighbor(
    graph: &Graph,
    v: VertexId,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    rng: &mut SplitMix64,
) -> IgraphResult<Option<(EdgeId, VertexId)>> {
    let incidents = incident_for_mode(graph, v, mode)?;
    if incidents.is_empty() {
        return Ok(None);
    }

    let eid = match weights {
        None => {
            let idx = rng.gen_index(incidents.len());
            incidents[idx]
        }
        Some(ws) => {
            let chosen = weighted_pick(&incidents, ws, rng);
            let Some(e) = chosen else {
                return Ok(None);
            };
            e
        }
    };
    let next = graph.edge_other(eid, v)?;
    Ok(Some((eid, next)))
}

/// Pick a neighbor with `Node2Vec` second-order bias.
#[allow(clippy::too_many_arguments)]
fn pick_biased_neighbor(
    graph: &Graph,
    prev: VertexId,
    current: VertexId,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    inv_p: f64,
    inv_q: f64,
    rng: &mut SplitMix64,
) -> IgraphResult<Option<(EdgeId, VertexId)>> {
    let incidents = incident_for_mode(graph, current, mode)?;
    if incidents.is_empty() {
        return Ok(None);
    }

    // Collect neighbors of `prev` for the distance check.
    let prev_neighbors = neighbor_set(graph, prev, mode)?;

    // Compute biased weights for each candidate edge.
    let mut biased_weights: Vec<f64> = Vec::with_capacity(incidents.len());
    let mut total = 0.0_f64;

    for &eid in &incidents {
        let base_weight = match weights {
            None => 1.0,
            Some(ws) => {
                let w = ws[eid as usize];
                if !(w.is_finite() && w > 0.0) {
                    biased_weights.push(0.0);
                    continue;
                }
                w
            }
        };

        let neighbor = graph.edge_other(eid, current)?;

        // Apply `Node2Vec` bias based on distance from `prev` to `neighbor`
        let alpha = if neighbor == prev {
            inv_p // d_tx = 0: returning to previous
        } else if prev_neighbors.contains(&neighbor) {
            1.0 // d_tx = 1: neighbor of prev
        } else {
            inv_q // d_tx = 2: not neighbor of prev
        };

        let w = alpha * base_weight;
        biased_weights.push(w);
        total += w;
    }

    if total <= 0.0 {
        return Ok(None);
    }

    // Weighted random selection
    let target = rng.gen_unit() * total;
    let mut acc = 0.0_f64;
    for (i, &w) in biased_weights.iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        acc += w;
        if acc >= target {
            let eid = incidents[i];
            let next = graph.edge_other(eid, current)?;
            return Ok(Some((eid, next)));
        }
    }

    // Floating-point fallback: pick last positive-weight edge
    for (i, &w) in biased_weights.iter().enumerate().rev() {
        if w > 0.0 {
            let eid = incidents[i];
            let next = graph.edge_other(eid, current)?;
            return Ok(Some((eid, next)));
        }
    }

    Ok(None)
}

/// Get a set of neighbors for efficient lookup.
fn neighbor_set(graph: &Graph, v: VertexId, mode: DijkstraMode) -> IgraphResult<Vec<VertexId>> {
    let incidents = incident_for_mode(graph, v, mode)?;
    let mut neighbors: Vec<VertexId> = Vec::with_capacity(incidents.len());
    for &eid in &incidents {
        let other = graph.edge_other(eid, v)?;
        neighbors.push(other);
    }
    neighbors.sort_unstable();
    neighbors.dedup();
    Ok(neighbors)
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

fn weighted_pick(incidents: &[EdgeId], ws: &[f64], rng: &mut SplitMix64) -> Option<EdgeId> {
    let mut total = 0.0_f64;
    for &eid in incidents {
        let w = ws[eid as usize];
        if w.is_finite() && w > 0.0 {
            total += w;
        }
    }
    if total <= 0.0 {
        return None;
    }
    let target = rng.gen_unit() * total;
    let mut acc = 0.0_f64;
    for &eid in incidents {
        let w = ws[eid as usize];
        if !(w.is_finite() && w > 0.0) {
            continue;
        }
        acc += w;
        if acc >= target {
            return Some(eid);
        }
    }
    // Fallback
    for &eid in incidents.iter().rev() {
        let w = ws[eid as usize];
        if w.is_finite() && w > 0.0 {
            return Some(eid);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_graph(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n - 1 {
            g.add_edge(i, i + 1).unwrap();
        }
        g
    }

    fn grid_graph() -> Graph {
        // 3x3 grid: 0-1-2 / 3-4-5 / 6-7-8
        let mut g = Graph::with_vertices(9);
        let edges = [
            (0, 1),
            (1, 2),
            (3, 4),
            (4, 5),
            (6, 7),
            (7, 8),
            (0, 3),
            (1, 4),
            (2, 5),
            (3, 6),
            (4, 7),
            (5, 8),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn unit_basic_walk_length() {
        let g = path_graph(10);
        let (vs, es) =
            random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, 1.0, 1.0, 42).unwrap();
        assert_eq!(vs[0], 0);
        assert!(vs.len() <= 6);
        assert_eq!(es.len(), vs.len() - 1);
    }

    #[test]
    fn unit_p_q_one_reduces_to_standard() {
        // With p=q=1 on a path graph, the walk should behave like a standard
        // random walk (all alphas are 1.0 regardless of distance).
        let g = path_graph(5);
        let (vs, _) =
            random_walk_node2vec(&g, None, 2, DijkstraMode::Out, 20, 1.0, 1.0, 123).unwrap();
        assert_eq!(vs[0], 2);
        // On an undirected path, every vertex has at most 2 neighbors
        for v in &vs {
            assert!(*v < 5);
        }
    }

    #[test]
    fn unit_high_p_discourages_return() {
        // With very high p, the walk should rarely return immediately.
        // On a grid starting from center (4), test over many walks.
        let g = grid_graph();
        let mut immediate_returns = 0;
        for seed in 0..100 {
            let (vs, _) =
                random_walk_node2vec(&g, None, 4, DijkstraMode::Out, 3, 100.0, 1.0, seed).unwrap();
            if vs.len() >= 3 && vs[2] == vs[0] {
                immediate_returns += 1;
            }
        }
        // With p=100, returning should be very rare (< 10% of walks)
        assert!(
            immediate_returns < 15,
            "expected few immediate returns with high p, got {immediate_returns}/100"
        );
    }

    #[test]
    fn unit_low_p_encourages_return() {
        // With very low p, the walk should often return immediately.
        let g = grid_graph();
        let mut immediate_returns = 0;
        for seed in 0..100 {
            let (vs, _) =
                random_walk_node2vec(&g, None, 4, DijkstraMode::Out, 3, 0.01, 1.0, seed).unwrap();
            if vs.len() >= 3 && vs[2] == vs[0] {
                immediate_returns += 1;
            }
        }
        // With p=0.01, returning should be very common (> 50% of walks)
        assert!(
            immediate_returns > 40,
            "expected many immediate returns with low p, got {immediate_returns}/100"
        );
    }

    #[test]
    fn unit_invalid_p() {
        let g = path_graph(5);
        let result = random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, 0.0, 1.0, 42);
        assert!(result.is_err());

        let result = random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, -1.0, 1.0, 42);
        assert!(result.is_err());

        let result = random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, f64::NAN, 1.0, 42);
        assert!(result.is_err());
    }

    #[test]
    fn unit_invalid_q() {
        let g = path_graph(5);
        let result = random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, 1.0, 0.0, 42);
        assert!(result.is_err());

        let result =
            random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, 1.0, f64::INFINITY, 42);
        assert!(result.is_err());
    }

    #[test]
    fn unit_start_out_of_range() {
        let g = path_graph(5);
        let result = random_walk_node2vec(&g, None, 10, DijkstraMode::Out, 5, 1.0, 1.0, 42);
        assert!(result.is_err());
    }

    #[test]
    fn unit_zero_steps() {
        let g = path_graph(5);
        let (vs, es) =
            random_walk_node2vec(&g, None, 2, DijkstraMode::Out, 0, 1.0, 1.0, 42).unwrap();
        assert_eq!(vs, vec![2]);
        assert!(es.is_empty());
    }

    #[test]
    fn unit_stuck_at_leaf() {
        // Directed path: 0→1→2→3. Starting from 3, walk gets stuck immediately.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let (vs, es) =
            random_walk_node2vec(&g, None, 3, DijkstraMode::Out, 10, 1.0, 1.0, 42).unwrap();
        assert_eq!(vs, vec![3]);
        assert!(es.is_empty());
    }

    #[test]
    fn unit_weighted_walk() {
        // Triangle 0-1-2 with weights favoring 0→1
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, weight 10
        g.add_edge(1, 2).unwrap(); // edge 1, weight 1
        g.add_edge(0, 2).unwrap(); // edge 2, weight 1

        let weights = vec![10.0, 1.0, 1.0];
        let (vs, _) =
            random_walk_node2vec(&g, Some(&weights), 0, DijkstraMode::Out, 1, 1.0, 1.0, 42)
                .unwrap();
        assert_eq!(vs[0], 0);
        assert!(vs.len() == 2);
    }

    #[test]
    fn unit_deterministic() {
        let g = grid_graph();
        let r1 = random_walk_node2vec(&g, None, 4, DijkstraMode::Out, 20, 2.0, 0.5, 99).unwrap();
        let r2 = random_walk_node2vec(&g, None, 4, DijkstraMode::Out, 20, 2.0, 0.5, 99).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn unit_single_vertex_graph() {
        let g = Graph::with_vertices(1);
        let (vs, es) =
            random_walk_node2vec(&g, None, 0, DijkstraMode::Out, 5, 1.0, 1.0, 42).unwrap();
        assert_eq!(vs, vec![0]);
        assert!(es.is_empty());
    }
}
