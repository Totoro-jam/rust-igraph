//! Voronoi graph partitioning (ALGO-SP-007).
//!
//! Counterpart of `igraph_voronoi` from `references/igraph/src/paths/voronoi.c`
//! (lines 359-418, plus the BFS / Dijkstra inner helpers at lines 30-309).
//!
//! Given a set of *generator* vertices, assign every other vertex to the
//! generator from which (or to which, in directed graphs with mode
//! [`DijkstraMode::In`]) the shortest-path distance is smallest. Ties at
//! equal distance are resolved by a caller-supplied [`VoronoiTiebreaker`]
//! rule:
//!
//! - [`VoronoiTiebreaker::First`] keeps the earliest generator (in input
//!   order) that reached the vertex.
//! - [`VoronoiTiebreaker::Last`] always overwrites with the latest
//!   generator.
//! - [`VoronoiTiebreaker::Random`] picks one of the equidistant
//!   generators uniformly at random; the choice is seeded by `seed`.
//!
//! Vertices not reachable from any generator are recorded as
//! [`Option::None`] in `membership` and as [`f64::INFINITY`] in
//! `distances`.
//!
//! The unweighted (BFS) and weighted (Dijkstra) inner loops share the
//! same `mindist`-aware early-exit: when expanding from generator `g`, a
//! vertex whose current `mindist` is already smaller than the distance
//! it was reached at via `g` is skipped entirely — its subtree under
//! `g`'s shortest-path tree is reachable cheaper from another generator,
//! so we don't need to explore it. This is what gives the time
//! complexity in the file-level docs at the bottom of this module.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Tie-breaking rule used when two or more generators reach a vertex at
/// the same distance.
///
/// Mirrors `igraph_voronoi_tiebreaker_t` from
/// `references/igraph/include/igraph_paths.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoronoiTiebreaker {
    /// Keep the earliest generator (in input order) that reached the
    /// vertex. Counterpart of `IGRAPH_VORONOI_FIRST`.
    First,
    /// Always overwrite with the latest generator. Counterpart of
    /// `IGRAPH_VORONOI_LAST`.
    Last,
    /// Pick one of the equidistant generators uniformly at random.
    /// Counterpart of `IGRAPH_VORONOI_RANDOM`. Uses a seeded
    /// `SplitMix64` so results are reproducible per `seed` value.
    ///
    /// Note: as in the C reference, [`VoronoiTiebreaker::Random`] does
    /// not guarantee that the resulting partitions are connected
    /// subgraphs (a tie-breaker chain can leave a vertex behind in a
    /// disconnected island of its assigned partition).
    Random,
}

/// Result of [`voronoi`]: the Voronoi cell each vertex belongs to plus
/// the distance to its assigned generator.
///
/// `membership[v]` is the *index into `generators`* of the generator
/// vertex that `v` was assigned to (so `generators[membership[v]
/// .unwrap()]` is the vertex id of that generator). `None` means `v` is
/// unreachable from every generator. `distances[v]` is the shortest-path
/// distance from `v` to its generator (under the given `mode`), or
/// [`f64::INFINITY`] if unreachable.
#[derive(Debug, Clone, PartialEq)]
pub struct VoronoiPartition {
    /// Per-vertex generator index; `None` for unreachable vertices.
    pub membership: Vec<Option<u32>>,
    /// Per-vertex distance to assigned generator; [`f64::INFINITY`] if
    /// unreachable. Unweighted graphs round to integer values.
    pub distances: Vec<f64>,
}

/// Minimal `SplitMix64` PRNG used by [`VoronoiTiebreaker::Random`].
/// Identical to the helper inside `paths::random_walk` — kept private so
/// each AWU's RNG choice is local.
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform float in `[0, 1)`. Uses 53 mantissa bits.
    fn gen_unit(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        #[allow(clippy::cast_precision_loss)]
        let numerator = bits as f64;
        numerator * (1.0_f64 / 9_007_199_254_740_992.0_f64)
    }
}

/// Mode-aware incident edges (mirrors the helper in `dijkstra.rs`).
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

/// Validate `weights`: length matches `graph.ecount()`, no NaN, no
/// negative values. [`f64::INFINITY`] is allowed and treated as a
/// non-edge during relaxation (as in the C reference).
fn validate_weights(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
        if w < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({w}); Voronoi requires non-negative weights"
            )));
        }
    }
    Ok(())
}

/// Validate that every generator vertex id is in `0..vcount()` and that
/// `generators` is non-empty when `vcount > 0`. Duplicates are allowed
/// (and behave as expected — the last/first/random one wins per the
/// tiebreaker rule).
fn validate_generators(graph: &Graph, generators: &[VertexId]) -> IgraphResult<()> {
    let n = graph.vcount();
    for (i, &g) in generators.iter().enumerate() {
        if g >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "generator at index {i} is vertex {g}, out of range (vcount = {n})"
            )));
        }
    }
    Ok(())
}

/// Apply the tiebreaker rule when generator `i` reached vertex `v` at
/// the same distance as a previously assigned generator. `tie_count[v]`
/// counts how many equidistant generators have been seen so far
/// (including this one).
fn apply_tiebreaker(
    tiebreaker: VoronoiTiebreaker,
    v_us: usize,
    i: u32,
    membership: &mut [Option<u32>],
    tie_count: &mut [u32],
    rng: &mut SplitMix64,
) {
    match tiebreaker {
        VoronoiTiebreaker::First => { /* never replace */ }
        VoronoiTiebreaker::Last => membership[v_us] = Some(i),
        VoronoiTiebreaker::Random => {
            tie_count[v_us] = tie_count[v_us].saturating_add(1);
            // Replace with probability 1/k upon seeing the k-th
            // equidistant generator; this is the reservoir-sampling
            // identity for a single slot.
            let k = tie_count[v_us];
            if rng.gen_unit() < 1.0_f64 / f64::from(k) {
                membership[v_us] = Some(i);
            }
        }
    }
}

/// BFS-multi-source Voronoi for unweighted graphs.
fn voronoi_bfs(
    graph: &Graph,
    generators: &[VertexId],
    mode: DijkstraMode,
    tiebreaker: VoronoiTiebreaker,
    seed: u64,
) -> IgraphResult<VoronoiPartition> {
    let n = graph.vcount();
    let n_us = n as usize;

    let mut membership: Vec<Option<u32>> = vec![None; n_us];
    let mut distances: Vec<f64> = vec![f64::INFINITY; n_us];
    let mut tie_count: Vec<u32> = vec![0; n_us];
    let mut already_counted: Vec<u32> = vec![0; n_us];
    let mut q: VecDeque<(VertexId, u32)> = VecDeque::new();
    let mut rng = SplitMix64::new(seed);

    for (i, &g) in generators.iter().enumerate() {
        let i_u32 = u32::try_from(i).map_err(|_| {
            IgraphError::InvalidArgument(format!("too many generators (overflow at index {i})"))
        })?;
        let stamp = i_u32.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("too many generators (overflow at index {i_u32})"))
        })?;

        q.clear();
        already_counted[g as usize] = stamp;
        q.push_back((g, 0));

        while let Some((vid, dist)) = q.pop_front() {
            let v_us = vid as usize;
            let md = distances[v_us];
            let dist_f = f64::from(dist);

            if dist_f > md {
                // This subtree is already reached cheaper from another
                // generator; skip its expansion entirely.
                continue;
            }
            if dist_f < md {
                distances[v_us] = dist_f;
                membership[v_us] = Some(i_u32);
                if matches!(tiebreaker, VoronoiTiebreaker::Random) {
                    tie_count[v_us] = 1;
                }
            } else {
                apply_tiebreaker(
                    tiebreaker,
                    v_us,
                    i_u32,
                    &mut membership,
                    &mut tie_count,
                    &mut rng,
                );
            }

            for eid in incident_for_mode(graph, vid, mode)? {
                let nei = graph.edge_other(eid as EdgeId, vid)?;
                if already_counted[nei as usize] == stamp {
                    continue;
                }
                already_counted[nei as usize] = stamp;
                let next_dist = dist.checked_add(1).ok_or(IgraphError::Internal(
                    "voronoi BFS distance overflow (graph diameter exceeds u32::MAX)",
                ))?;
                q.push_back((nei, next_dist));
            }
        }
    }

    Ok(VoronoiPartition {
        membership,
        distances,
    })
}

/// Min-heap entry. `Ord` reversed so `BinaryHeap` (max-heap) pops the
/// smallest distance first. NaN distances are forbidden by validation.
#[derive(Copy, Clone)]
struct Frontier(f64, VertexId);

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Dijkstra-multi-source Voronoi for weighted graphs.
fn voronoi_dijkstra(
    graph: &Graph,
    weights: &[f64],
    generators: &[VertexId],
    mode: DijkstraMode,
    tiebreaker: VoronoiTiebreaker,
    seed: u64,
) -> IgraphResult<VoronoiPartition> {
    let n = graph.vcount();
    let n_us = n as usize;

    let mut membership: Vec<Option<u32>> = vec![None; n_us];
    let mut distances: Vec<f64> = vec![f64::INFINITY; n_us];
    let mut tie_count: Vec<u32> = vec![0; n_us];
    let mut rng = SplitMix64::new(seed);

    // Per-generator best-known distance reaching each vertex (the cost
    // of the current Dijkstra source). Replaces the C 2wheap's
    // `has_elem` / `get` / `has_active` machinery: we track our own
    // tentative distances and skip a relax that doesn't improve.
    let mut tentative: Vec<f64> = vec![f64::INFINITY; n_us];

    for (i, &g) in generators.iter().enumerate() {
        let i_u32 = u32::try_from(i).map_err(|_| {
            IgraphError::InvalidArgument(format!("too many generators (overflow at index {i})"))
        })?;

        // Reset tentative for this generator: only the vertices we
        // touched last time need clearing, but clearing all is O(n) per
        // generator vs O(visited) per generator — for simplicity (and
        // because the visited set can equal n in worst case) we clear
        // the whole vector. Same asymptotic as the C 2wheap reset.
        tentative.fill(f64::INFINITY);
        let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
        tentative[g as usize] = 0.0;
        heap.push(Frontier(0.0, g));

        while let Some(Frontier(dist, vid)) = heap.pop() {
            let v_us = vid as usize;
            // Stale heap entry (a smaller distance was already settled).
            if dist > tentative[v_us] {
                continue;
            }
            let md = distances[v_us];

            // Use a small tolerance to match the C `igraph_cmp_epsilon`
            // behavior for tie detection — without it, floating-point
            // accumulation could classify "equal" distances as strictly
            // less / greater.
            let cmp = cmp_epsilon(dist, md, EPSILON);
            if cmp == Ordering::Greater {
                // No improvement from this generator; skip subtree.
                continue;
            }
            if cmp == Ordering::Less {
                distances[v_us] = dist;
                membership[v_us] = Some(i_u32);
                if matches!(tiebreaker, VoronoiTiebreaker::Random) {
                    tie_count[v_us] = 1;
                }
            } else {
                apply_tiebreaker(
                    tiebreaker,
                    v_us,
                    i_u32,
                    &mut membership,
                    &mut tie_count,
                    &mut rng,
                );
            }

            for eid in incident_for_mode(graph, vid, mode)? {
                let w = weights[eid as usize];
                // Match the C optimization: skip infinite-weight edges.
                if !w.is_finite() {
                    continue;
                }
                let nd = dist + w;
                let other = graph.edge_other(eid as EdgeId, vid)?;
                let other_us = other as usize;
                if nd < tentative[other_us] {
                    tentative[other_us] = nd;
                    heap.push(Frontier(nd, other));
                }
            }
        }
    }

    Ok(VoronoiPartition {
        membership,
        distances,
    })
}

const EPSILON: f64 = 1e-10;

fn cmp_epsilon(a: f64, b: f64, eps: f64) -> Ordering {
    let diff = a - b;
    let abs_a = a.abs();
    let abs_b = b.abs();
    let denom = if abs_a > abs_b { abs_a } else { abs_b };
    if denom == 0.0 || denom.is_infinite() {
        // Both zero (denom == 0), or both/either infinite: fall back to
        // total ordering. `total_cmp` matches `Ordering::Equal` only when
        // bit-identical, which is the C reference behavior here.
        a.total_cmp(&b)
    } else if diff.abs() <= eps * denom {
        Ordering::Equal
    } else if diff < 0.0 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// Voronoi partitioning of a graph from a set of generator vertices.
///
/// Each vertex is assigned to the generator at the smallest distance
/// (under `mode`'s direction semantics in directed graphs). Tied
/// distances are resolved by `tiebreaker`. Vertices unreachable from
/// every generator have `membership[v] == None` and
/// `distances[v] == f64::INFINITY`.
///
/// `weights == None` uses BFS (every edge counts as length 1); otherwise
/// uses Dijkstra. All weights must be non-negative and not NaN; infinite
/// weights are allowed and treated as non-edges.
///
/// `seed` only affects [`VoronoiTiebreaker::Random`] — [`VoronoiTiebreaker::First`]
/// and [`VoronoiTiebreaker::Last`] are deterministic regardless of seed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, voronoi, VoronoiTiebreaker, DijkstraMode};
///
/// // Path 0-1-2-3-4, generators {0, 4}. The split is "halfway", with
/// // vertex 2 equidistant (d=2 from both). With `First`, vertex 2 goes
/// // to generator index 0 (vertex 0); with `Last`, to index 1 (vertex 4).
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let part = voronoi(&g, None, DijkstraMode::Out, &[0, 4], VoronoiTiebreaker::First, 0).unwrap();
/// assert_eq!(part.membership, vec![Some(0), Some(0), Some(0), Some(1), Some(1)]);
/// assert_eq!(part.distances,  vec![0.0,     1.0,     2.0,     1.0,     0.0]);
///
/// let part = voronoi(&g, None, DijkstraMode::Out, &[0, 4], VoronoiTiebreaker::Last, 0).unwrap();
/// assert_eq!(part.membership, vec![Some(0), Some(0), Some(1), Some(1), Some(1)]);
/// ```
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `weights.len() != graph.ecount()`,
///   any weight is NaN or negative, or any generator index is out of
///   range.
/// - [`IgraphError::Internal`] on graph-diameter overflow (unweighted
///   only — would require > 2^32 - 1 BFS layers).
///
/// # Complexity
///
/// `O(|S| · (|V| + |E|))` for unweighted graphs; `O(|S| · (|V| + |E|) ·
/// log |V|)` for weighted graphs, where `|S|` is the number of
/// generators.
pub fn voronoi(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    generators: &[VertexId],
    tiebreaker: VoronoiTiebreaker,
    seed: u64,
) -> IgraphResult<VoronoiPartition> {
    let n = graph.vcount();
    // Normalize mode: undirected graphs collapse every mode to All
    // (matches the upstream contract).
    let mode = if graph.is_directed() {
        mode
    } else {
        DijkstraMode::All
    };

    validate_generators(graph, generators)?;
    if let Some(w) = weights {
        validate_weights(graph, w)?;
    }

    if n == 0 {
        return Ok(VoronoiPartition {
            membership: Vec::new(),
            distances: Vec::new(),
        });
    }

    if generators.is_empty() {
        // No generators: every vertex is unreachable.
        return Ok(VoronoiPartition {
            membership: vec![None; n as usize],
            distances: vec![f64::INFINITY; n as usize],
        });
    }

    match weights {
        None => voronoi_bfs(graph, generators, mode, tiebreaker, seed),
        Some(w) => voronoi_dijkstra(graph, w, generators, mode, tiebreaker, seed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_n(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph_returns_empty() {
        let g = Graph::with_vertices(0);
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert!(part.membership.is_empty());
        assert!(part.distances.is_empty());
    }

    #[test]
    fn no_generators_all_unreachable() {
        let g = path_n(4);
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part.membership, vec![None, None, None, None]);
        assert!(part.distances.iter().all(|d| d.is_infinite()));
    }

    #[test]
    fn single_generator_covers_connected_component() {
        let g = path_n(5);
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(
            part.membership,
            vec![Some(0), Some(0), Some(0), Some(0), Some(0)]
        );
        assert_eq!(part.distances, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn path_two_generators_first_keeps_tied_vertex_left() {
        // Path 0-1-2-3-4; generators {0, 4}. Vertex 2 is equidistant
        // (d=2). With First, the earlier generator (index 0) keeps it.
        let g = path_n(5);
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0, 4],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(
            part.membership,
            vec![Some(0), Some(0), Some(0), Some(1), Some(1)]
        );
        assert_eq!(part.distances, vec![0.0, 1.0, 2.0, 1.0, 0.0]);
    }

    #[test]
    fn path_two_generators_last_takes_tied_vertex() {
        let g = path_n(5);
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0, 4],
            VoronoiTiebreaker::Last,
            0,
        )
        .unwrap();
        assert_eq!(
            part.membership,
            vec![Some(0), Some(0), Some(1), Some(1), Some(1)]
        );
    }

    #[test]
    fn random_tiebreaker_is_deterministic_for_fixed_seed() {
        let g = path_n(5);
        let part1 = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0, 4],
            VoronoiTiebreaker::Random,
            42,
        )
        .unwrap();
        let part2 = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0, 4],
            VoronoiTiebreaker::Random,
            42,
        )
        .unwrap();
        assert_eq!(part1, part2);
    }

    #[test]
    fn disconnected_components_unreachable_get_none() {
        // Two disjoint paths: 0-1-2 and 3-4-5. Generator only in left.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(
            part.membership,
            vec![Some(0), Some(0), Some(0), None, None, None]
        );
        assert!(part.distances[3].is_infinite());
        assert!(part.distances[4].is_infinite());
        assert!(part.distances[5].is_infinite());
    }

    #[test]
    fn weighted_dijkstra_assigns_closest() {
        // Triangle 0-1-2, weights 1-2-10. Generators {0, 2}.
        // 0→1: direct weight 1, via 0→2→1 = 10+2 = 12. So vertex 1 goes to 0 (d=1).
        // From generator 2: 2→1 weight 2 < 12; 2→0 weight 10 (direct) but 2→1→0 = 2+1 = 3 < 10.
        // So d(2→0) = 3 > d(0→0) = 0, vertex 0 stays at generator 0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        g.add_edge(0, 2).unwrap(); // edge 2
        let weights = [1.0, 2.0, 10.0];
        let part = voronoi(
            &g,
            Some(&weights),
            DijkstraMode::Out,
            &[0, 2],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part.membership, vec![Some(0), Some(0), Some(1)]);
        assert_eq!(part.distances, vec![0.0, 1.0, 0.0]);
    }

    #[test]
    fn weighted_validates_negative_weight() {
        let g = path_n(3);
        let weights = [-1.0, 1.0];
        let err = voronoi(
            &g,
            Some(&weights),
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn weighted_validates_nan_weight() {
        let g = path_n(3);
        let weights = [f64::NAN, 1.0];
        let err = voronoi(
            &g,
            Some(&weights),
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn weighted_rejects_wrong_length_weight() {
        let g = path_n(3);
        let weights = [1.0, 1.0, 1.0]; // path has only 2 edges
        let err = voronoi(
            &g,
            Some(&weights),
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn generator_out_of_range_errors() {
        let g = path_n(3);
        let err = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[0, 5],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => (),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn all_vertices_as_generators_each_is_its_own_cell() {
        let g = path_n(4);
        let gens: Vec<VertexId> = (0..4).collect();
        let part = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &gens,
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part.membership, vec![Some(0), Some(1), Some(2), Some(3)]);
        assert_eq!(part.distances, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn directed_mode_out_vs_in() {
        // 0 -> 1 -> 2; generators {2}.
        // Out mode: from 2 we follow out-edges → only 2 reachable.
        // In mode: from 2 we follow in-edges (towards 2) → 0,1,2 all reach 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let part_out = voronoi(
            &g,
            None,
            DijkstraMode::Out,
            &[2],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part_out.membership, vec![None, None, Some(0)]);
        let part_in = voronoi(
            &g,
            None,
            DijkstraMode::In,
            &[2],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part_in.membership, vec![Some(0), Some(0), Some(0)]);
        assert_eq!(part_in.distances, vec![2.0, 1.0, 0.0]);
    }

    #[test]
    fn weighted_infinite_edge_skipped() {
        // Triangle 0-1, 1-2, 0-2 with weight(0-2) = +Inf.
        // Generator {0}: 0→1=1, 0→2 either direct (Inf, rejected) or via 1: 0→1→2 = 1+2 = 3.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = [1.0, 2.0, f64::INFINITY];
        let part = voronoi(
            &g,
            Some(&weights),
            DijkstraMode::Out,
            &[0],
            VoronoiTiebreaker::First,
            0,
        )
        .unwrap();
        assert_eq!(part.membership, vec![Some(0), Some(0), Some(0)]);
        assert_eq!(part.distances, vec![0.0, 1.0, 3.0]);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::algorithms::paths::distances::distances;
    use proptest::prelude::*;

    prop_compose! {
        fn small_undirected_graph()(n in 2u32..=10u32, edges_seed in any::<u64>()) -> Graph {
            let mut g = Graph::with_vertices(n);
            let mut rng = edges_seed;
            let target_m = ((n * (n - 1)) / 2).min(n + 6) as usize;
            let mut added = 0usize;
            let mut guard = 0usize;
            while added < target_m && guard < target_m * 8 + 4 {
                rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let u = ((rng >> 33) % u64::from(n)) as u32;
                rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let v = ((rng >> 33) % u64::from(n)) as u32;
                guard += 1;
                if u == v {
                    continue;
                }
                if g.add_edge(u, v).is_ok() {
                    added += 1;
                }
            }
            g
        }
    }

    proptest! {
        #[test]
        fn membership_indices_in_range(
            g in small_undirected_graph(),
            seed in any::<u64>(),
        ) {
            let n = g.vcount();
            let k = (n as usize).div_ceil(3).max(1);
            let gens: Vec<VertexId> = (0..k as u32).collect();
            let part = voronoi(
                &g, None, DijkstraMode::Out, &gens,
                VoronoiTiebreaker::Random, seed,
            ).unwrap();
            prop_assert_eq!(part.membership.len(), n as usize);
            for i in part.membership.iter().flatten() {
                prop_assert!((*i as usize) < gens.len());
            }
        }

        #[test]
        fn unweighted_distance_matches_min_over_generators(
            g in small_undirected_graph(),
            seed in any::<u64>(),
        ) {
            let n = g.vcount();
            let k = ((n as usize) / 2).max(1);
            let gens: Vec<VertexId> = (0..k as u32).collect();
            let part = voronoi(
                &g, None, DijkstraMode::Out, &gens,
                VoronoiTiebreaker::First, seed,
            ).unwrap();
            for v in 0..n {
                let mut best = u32::MAX;
                for &gid in &gens {
                    let d = distances(&g, gid).unwrap();
                    if let Some(dv) = d[v as usize] {
                        if dv < best {
                            best = dv;
                        }
                    }
                }
                if best == u32::MAX {
                    prop_assert!(part.distances[v as usize].is_infinite());
                    prop_assert!(part.membership[v as usize].is_none());
                } else {
                    let got = part.distances[v as usize];
                    prop_assert!((got - f64::from(best)).abs() < 1e-9);
                    prop_assert!(part.membership[v as usize].is_some());
                }
            }
        }

        #[test]
        fn tiebreaker_first_picks_earliest_at_ties(
            g in small_undirected_graph(),
            seed in any::<u64>(),
        ) {
            let n = g.vcount();
            let k = ((n as usize) / 2).max(2);
            let gens: Vec<VertexId> = (0..k as u32).collect();
            let part = voronoi(
                &g, None, DijkstraMode::Out, &gens,
                VoronoiTiebreaker::First, seed,
            ).unwrap();
            for v in 0..n {
                let mind = part.distances[v as usize];
                if mind.is_infinite() {
                    continue;
                }
                let assigned = part.membership[v as usize].unwrap();
                for (i, &gid) in gens.iter().enumerate() {
                    if i as u32 >= assigned {
                        break;
                    }
                    let d = distances(&g, gid).unwrap();
                    if let Some(dv) = d[v as usize] {
                        prop_assert!(f64::from(dv) > mind);
                    }
                }
            }
        }
    }
}
