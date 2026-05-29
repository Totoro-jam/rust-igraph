//! Uniform random labelled tree generators (ALGO-GN-004).
//!
//! Counterpart of `igraph_tree_game()` in
//! `references/igraph/src/games/tree.c`. Two methods are available:
//!
//! * **LERW** ([`tree_game_lerw`]) — Wilson's loop-erased random walk on
//!   `K_n`. Supports both directed (out-rooted) and undirected trees.
//! * **Prüfer** ([`tree_game_prufer`]) — samples a random Prüfer sequence,
//!   then decodes it via [`from_prufer`](crate::from_prufer). Undirected
//!   only, matching the C upstream restriction.
//!
//! Both methods produce each labelled tree with equal probability.
//!
//! ## LERW algorithm
//!
//! Wilson's loop-erased random walk on the complete graph `K_n` uniformly
//! samples spanning trees. The igraph implementation collapses the naive
//! "walk until you hit a visited vertex, then erase the loop" formulation
//! into a single linear pass:
//!
//! 1. Maintain an array `vertices = [0, 1, …, n - 1]` partitioned so that
//!    positions `[0, k)` hold the visited vertices and `[k, n)` hold the
//!    unvisited ones.
//! 2. Pick an initial vertex `i` uniformly, mark it visited, swap it to
//!    position `0`, set `k = 1`.
//! 3. For each remaining step `k = 1 .. n`: draw `j ∈ [0, n)`. If
//!    `vertices[j]` is already visited (its slot lies in `[0, k)`),
//!    advance the walk by setting `i = vertices[j]` and resample
//!    `j ∈ [k, n)` so the next visited vertex is guaranteed to be a
//!    fresh one. Then mark `vertices[j]` visited, swap it to position
//!    `k`, emit the edge `(i, vertices[k])`, and update
//!    `i = vertices[k]`.
//!
//! Each iteration produces exactly one edge, so the tree has `n - 1`
//! edges. Runtime is `O(n)` walk steps amortised — there is no rejection
//! loop because step 3a covers the "already visited" branch in one shot.
//!
//! ## Prüfer algorithm
//!
//! Generate `n - 2` uniform random values in `[0, n)`, forming a Prüfer
//! sequence, then decode it with the linear-time algorithm from
//! [`from_prufer`](crate::from_prufer). The result is always an undirected
//! labelled tree.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::algorithms::constructors::prufer::from_prufer;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Generate a uniformly random labelled tree on `n` vertices using
/// Wilson's loop-erased random walk.
///
/// * `n` — vertex count. `n = 0` returns an empty graph, `n = 1`
///   returns a single isolated vertex.
/// * `directed` — if `true`, edges are stored with parent-to-child
///   orientation in walk order (the resulting tree is rooted at the
///   randomly chosen initial vertex). If `false`, the same edges are
///   stored as an undirected tree.
/// * `seed` — initialises an internal [`SplitMix64`] PRNG. Same
///   `(n, directed, seed)` always yields the same tree.
///
/// The output has exactly `max(0, n - 1)` edges, is acyclic, has no
/// self-loops, and (in the undirected case) is connected.
///
/// # Examples
///
/// ```
/// use rust_igraph::tree_game_lerw;
///
/// // 30-vertex undirected uniform random tree.
/// let g = tree_game_lerw(30, false, 0xC0FF_EE00).unwrap();
/// assert_eq!(g.vcount(), 30);
/// assert_eq!(g.ecount(), 29);  // n - 1 edges
/// assert!(!g.is_directed());
/// ```
pub fn tree_game_lerw(n: u32, directed: bool, seed: u64) -> IgraphResult<Graph> {
    if n < 2 {
        return Graph::new(n, directed);
    }

    let n_usize = n as usize;
    let no_edges = n_usize - 1;

    let mut vertices: Vec<VertexId> = (0..n).collect();
    let mut visited: Vec<bool> = vec![false; n_usize];
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(no_edges);

    let mut rng = SplitMix64::new(seed);

    // Pick the initial vertex uniformly, mark it visited, swap to slot 0.
    let i0 = rng.gen_index(n_usize);
    visited[vertices[i0] as usize] = true;
    vertices.swap(0, i0);

    let mut prev: VertexId = vertices[0];

    for k in 1..n_usize {
        // Draw a candidate slot from [0, n).
        let mut j = rng.gen_index(n_usize);
        if visited[vertices[j] as usize] {
            // Already visited — advance the walk and resample from the
            // unvisited tail [k, n) so the next emit is fresh.
            prev = vertices[j];
            j = k + rng.gen_index(n_usize - k);
        }
        visited[vertices[j] as usize] = true;
        vertices.swap(k, j);
        let new_v = vertices[k];
        edges.push((prev, new_v));
        prev = new_v;
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Generate a uniformly random labelled tree on `n` vertices using the
/// Prüfer sequence method.
///
/// Samples a random Prüfer sequence of length `n - 2` (each entry
/// uniform in `[0, n)`), then decodes it with [`from_prufer`] to
/// produce an undirected labelled tree.
///
/// * `n` — vertex count. `n = 0` returns an empty graph, `n = 1`
///   returns a single isolated vertex.
/// * `seed` — initialises an internal [`SplitMix64`] PRNG. Same
///   `(n, seed)` always yields the same tree.
///
/// The output has exactly `max(0, n - 1)` edges, is acyclic, has no
/// self-loops, and is connected.
///
/// Counterpart of the `IGRAPH_RANDOM_TREE_PRUFER` branch of
/// `igraph_tree_game()` in `references/igraph/src/games/tree.c:37-56`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `n` overflows internal
///   allocation limits.
///
/// # Examples
///
/// ```
/// use rust_igraph::tree_game_prufer;
///
/// let g = tree_game_prufer(30, 0xC0FF_EE00).unwrap();
/// assert_eq!(g.vcount(), 30);
/// assert_eq!(g.ecount(), 29);
/// assert!(!g.is_directed());
/// ```
pub fn tree_game_prufer(n: u32, seed: u64) -> IgraphResult<Graph> {
    if n < 2 {
        return Graph::new(n, false);
    }

    let seq_len = (n - 2) as usize;
    let mut rng = SplitMix64::new(seed);
    let n_usize = n as usize;

    let mut prufer: Vec<u32> = Vec::with_capacity(seq_len);
    for _ in 0..seq_len {
        let val = rng.gen_index(n_usize);
        let val_u32 = u32::try_from(val).map_err(|_| {
            IgraphError::InvalidArgument("tree_game_prufer: vertex index overflow".to_string())
        })?;
        prufer.push(val_u32);
    }

    from_prufer(&prufer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let n_edges = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        (0..n_edges)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    /// Union-find for connectivity / acyclicity invariants.
    struct UnionFind {
        parent: Vec<u32>,
    }
    impl UnionFind {
        fn new(n: usize) -> Self {
            Self {
                parent: (0..n as u32).collect(),
            }
        }
        fn find(&mut self, mut x: u32) -> u32 {
            while self.parent[x as usize] != x {
                let p = self.parent[x as usize];
                self.parent[x as usize] = self.parent[p as usize];
                x = self.parent[x as usize];
            }
            x
        }
        /// Returns `true` when the union created a new bridge,
        /// `false` if the two ends were already in the same component.
        fn union(&mut self, a: u32, b: u32) -> bool {
            let ra = self.find(a);
            let rb = self.find(b);
            if ra == rb {
                false
            } else {
                self.parent[ra as usize] = rb;
                true
            }
        }
    }

    #[test]
    fn n_zero_returns_empty_graph() {
        let g = tree_game_lerw(0, false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_returns_singleton() {
        let g = tree_game_lerw(1, false, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_two_returns_single_edge() {
        let g = tree_game_lerw(2, false, 0xBEEF).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        let (a, b) = g.edge(0).unwrap();
        assert_ne!(a, b, "tree edge must not be a self-loop");
        let lo = a.min(b);
        let hi = a.max(b);
        assert_eq!(lo, 0);
        assert_eq!(hi, 1);
    }

    #[test]
    fn exact_edge_count() {
        for &n in &[5u32, 10, 50, 200, 1000] {
            let g = tree_game_lerw(n, false, 0xC0FF_EE00 + u64::from(n)).unwrap();
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount() as u32, n - 1);
        }
    }

    #[test]
    fn no_self_loops() {
        let g = tree_game_lerw(150, false, 0xFACE).unwrap();
        for (a, b) in collect_edges(&g) {
            assert_ne!(a, b, "Wilson LERW must not emit self-loops");
        }
    }

    #[test]
    fn output_is_acyclic_and_connected_undirected() {
        let g = tree_game_lerw(200, false, 0xCAFE).unwrap();
        let n = g.vcount() as usize;
        let mut uf = UnionFind::new(n);
        for (a, b) in collect_edges(&g) {
            assert!(
                uf.union(a, b),
                "edge ({a}, {b}) closed a cycle — output is not a tree"
            );
        }
        // After n - 1 distinct unions all vertices must share one root.
        let root = uf.find(0);
        for v in 1..n as u32 {
            assert_eq!(
                uf.find(v),
                root,
                "vertex {v} is in a disconnected component"
            );
        }
    }

    #[test]
    fn output_is_acyclic_and_connected_directed() {
        // Directed mode stores edges with parent→child orientation, but
        // connectivity should still hold on the underlying undirected
        // graph.
        let g = tree_game_lerw(150, true, 0xDEAD).unwrap();
        assert!(g.is_directed());
        let n = g.vcount() as usize;
        let mut uf = UnionFind::new(n);
        for (a, b) in collect_edges(&g) {
            assert!(
                uf.union(a, b),
                "directed tree edge ({a}, {b}) closed a cycle"
            );
        }
        let root = uf.find(0);
        for v in 1..n as u32 {
            assert_eq!(uf.find(v), root);
        }
    }

    #[test]
    fn directed_mode_has_no_duplicate_targets() {
        // Each freshly visited vertex appears exactly once as a target,
        // which means in-degree of every non-root vertex is exactly 1.
        let g = tree_game_lerw(100, true, 0xBAD_F00D).unwrap();
        let n = g.vcount() as usize;
        let mut indeg = vec![0u32; n];
        for (_a, b) in collect_edges(&g) {
            indeg[b as usize] += 1;
        }
        let roots: Vec<_> = indeg.iter().enumerate().filter(|&(_, &d)| d == 0).collect();
        assert_eq!(roots.len(), 1, "directed Wilson tree has exactly one root");
        for (v, &d) in indeg.iter().enumerate() {
            if v == roots[0].0 {
                continue;
            }
            assert_eq!(d, 1, "non-root vertex {v} must have in-degree 1, got {d}");
        }
    }

    #[test]
    fn deterministic_with_seed() {
        let a = tree_game_lerw(80, false, 0xABCD).unwrap();
        let b = tree_game_lerw(80, false, 0xABCD).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        assert_eq!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn different_seeds_yield_different_trees() {
        let a = tree_game_lerw(60, false, 1).unwrap();
        let b = tree_game_lerw(60, false, 2).unwrap();
        assert_ne!(
            collect_edges(&a),
            collect_edges(&b),
            "different seeds must produce different trees"
        );
    }

    #[test]
    fn all_vertices_appear_in_some_edge_for_n_ge_2() {
        // A spanning tree touches every vertex.
        let g = tree_game_lerw(40, false, 0x5EED).unwrap();
        let mut seen = vec![false; g.vcount() as usize];
        for (a, b) in collect_edges(&g) {
            seen[a as usize] = true;
            seen[b as usize] = true;
        }
        for (v, &s) in seen.iter().enumerate() {
            assert!(s, "vertex {v} missing from spanning tree");
        }
    }

    // ── tree_game_prufer tests ──────────────────────────────────────

    #[test]
    fn prufer_n_zero() {
        let g = tree_game_prufer(0, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn prufer_n_one() {
        let g = tree_game_prufer(1, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn prufer_n_two() {
        let g = tree_game_prufer(2, 0xBEEF).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert!(!g.is_directed());
    }

    #[test]
    fn prufer_always_undirected() {
        let g = tree_game_prufer(50, 0xFACE).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn prufer_exact_edge_count() {
        for &n in &[5u32, 10, 50, 200, 1000] {
            let g = tree_game_prufer(n, 0xC0DE + u64::from(n)).unwrap();
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount() as u32, n - 1);
        }
    }

    #[test]
    fn prufer_no_self_loops() {
        let g = tree_game_prufer(150, 0xFACE).unwrap();
        for (a, b) in collect_edges(&g) {
            assert_ne!(a, b, "Prüfer tree must not emit self-loops");
        }
    }

    #[test]
    fn prufer_is_acyclic_and_connected() {
        let g = tree_game_prufer(200, 0xCAFE).unwrap();
        let n = g.vcount() as usize;
        let mut uf = UnionFind::new(n);
        for (a, b) in collect_edges(&g) {
            assert!(
                uf.union(a, b),
                "edge ({a}, {b}) closed a cycle — not a tree"
            );
        }
        let root = uf.find(0);
        for v in 1..n as u32 {
            assert_eq!(uf.find(v), root, "vertex {v} in disconnected component");
        }
    }

    #[test]
    fn prufer_deterministic_with_seed() {
        let a = tree_game_prufer(80, 0xABCD).unwrap();
        let b = tree_game_prufer(80, 0xABCD).unwrap();
        assert_eq!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn prufer_different_seeds_yield_different_trees() {
        let a = tree_game_prufer(60, 1).unwrap();
        let b = tree_game_prufer(60, 2).unwrap();
        assert_ne!(
            collect_edges(&a),
            collect_edges(&b),
            "different seeds must produce different trees"
        );
    }

    #[test]
    fn prufer_all_vertices_touched() {
        let g = tree_game_prufer(40, 0x5EED).unwrap();
        let mut seen = vec![false; g.vcount() as usize];
        for (a, b) in collect_edges(&g) {
            seen[a as usize] = true;
            seen[b as usize] = true;
        }
        for (v, &s) in seen.iter().enumerate() {
            assert!(s, "vertex {v} missing from spanning tree");
        }
    }
}
