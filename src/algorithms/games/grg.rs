//! Geometric random graph (ALGO-GN-005).
//!
//! Counterpart of `igraph_grg_game()` in
//! `references/igraph/src/games/grg.c:56-174`.
//!
//! ## Model
//!
//! Drop `n` points uniformly at random on the unit square
//! `[0, 1) × [0, 1)`, then connect every pair `(i, j)` whose Euclidean
//! distance is *strictly* less than `radius`. With `torus = true` the
//! square is identified along opposite edges (periodic boundary
//! conditions), so distances wrap modulo 1 along each axis.
//!
//! ## Algorithm — x-sweep with width-`radius` window
//!
//! 1. Sample `2n` independent `Uniform[0, 1)` values to populate `xs`
//!    and `ys`. The `(x, y)` pairing is *not* preserved — upstream's
//!    `igraph_vector_sort(xx)` mutates only the x array, leaving `ys`
//!    in generation order. The resulting (x, y) joint marginal is still
//!    uniform because the two axes were independent, but the original
//!    point identities are lost. We mirror this behaviour exactly.
//! 2. Sort `xs` in ascending order. For every `i` in `0..n` walk a
//!    forward window `j = i + 1, i + 2, …` while `xs[j] - xs[i] <
//!    radius`. Whenever the squared distance falls below `radius²`,
//!    emit the edge `(i, j)`.
//! 3. Torus mode adds a wrap-around tail: once the forward window
//!    crosses `j = n`, continue at `j = 0, 1, …` for as long as the
//!    wrapped x-gap `1 - xs[i] + xs[j]` is below `radius`. The y
//!    coordinate is wrapped with `min(|Δy|, 1 - |Δy|)`.
//!
//! Both modes run in `O(n + |E|)` expected time once the sort cost
//! `O(n log n)` is paid up front. The window invariant means each
//! candidate pair is inspected exactly once, regardless of how dense
//! the graph is.
//!
//! ## Determinism
//!
//! `grg_game` is fully deterministic in `(n, radius, torus, seed)`. The
//! PRNG is `SplitMix64`; reusing the same seed in
//! another generator (e.g. [`crate::tree_game_lerw`]) is safe — each
//! call owns a fresh `SplitMix64` instance.
//!
//! ## Scope
//!
//! The upstream signature accepts optional output vectors for `x` and
//! `y` coordinates. We provide both flavours:
//! [`grg_game`] returns the graph only, and [`grg_game_with_coords`]
//! returns the sorted `xs` and the original-order `ys`. Both call into
//! the same internal worker so the edge set is byte-for-byte identical
//! for the same seed.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate_inputs(n: u32, radius: f64) -> IgraphResult<()> {
    if !radius.is_finite() {
        return Err(IgraphError::Internal("grg_game radius must be finite"));
    }
    // Hard cap on potential edges. n choose 2 stays within u64 across
    // u32::MAX safely (2^63 ≫ 2^31 · (2^31 - 1) / 2 ≈ 2^61). We don't
    // pre-allocate the edge list because the actual edge count depends
    // on radius — capacity grows dynamically. The validation here is
    // purely a "non-NaN, non-infinite radius" gate so the sweep
    // comparison `dx < radius` is well-defined.
    let _ = n;
    Ok(())
}

/// Generate a geometric random graph on `n` uniformly placed points in
/// the unit square, connecting pairs strictly closer than `radius` in
/// Euclidean distance.
///
/// * `n` — vertex count. `n = 0` returns an empty undirected graph;
///   `n = 1` returns a single isolated vertex.
/// * `radius` — Euclidean cut-off in the same units as the unit square.
///   Negative values are coerced to `0` (matching upstream). The
///   maximum meaningful value is `sqrt(2)` for the open square or
///   `sqrt(0.5)` for the torus (above this every pair is in range).
/// * `torus` — periodic boundary conditions when `true`. The unit
///   square is identified with a torus and y-distances wrap by
///   `1 - |Δy|` if shorter; x-wrapping is folded into the sweep tail.
/// * `seed` — seeds the internal `SplitMix64` PRNG. Same
///   `(n, radius, torus, seed)` always yields the same graph.
///
/// The generated graph is always undirected and free of self-loops
/// (the sweep starts at `j = i + 1`). It is also simple — each pair
/// `(i, j)` is inspected at most once, so no multi-edges arise.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] if `radius` is `NaN` or
/// infinite.
///
/// # Examples
///
/// ```
/// use rust_igraph::grg_game;
///
/// // Dense regime: radius near sqrt(2) connects almost every pair.
/// let dense = grg_game(20, 1.5, false, 0xA110).unwrap();
/// assert_eq!(dense.vcount(), 20);
/// assert!(dense.ecount() > 100);
/// assert!(!dense.is_directed());
/// ```
pub fn grg_game(n: u32, radius: f64, torus: bool, seed: u64) -> IgraphResult<Graph> {
    let (graph, _xs, _ys) = grg_game_with_coords(n, radius, torus, seed)?;
    Ok(graph)
}

/// Same as [`grg_game`] but also returns the point coordinates.
///
/// The first vector is `xs` in ascending order — `xs[i]` is the
/// x-coordinate of vertex `i`. The second vector is `ys` in *generation
/// order* (i.e. not paired with `xs[i]`), mirroring upstream
/// `igraph_grg_game(..., x, y)` which sorts only the x array. Callers
/// who need the original (x, y) pairing should generate the points
/// themselves and call [`grg_game`] separately.
///
/// # Errors
///
/// Same as [`grg_game`].
pub fn grg_game_with_coords(
    n: u32,
    radius: f64,
    torus: bool,
    seed: u64,
) -> IgraphResult<(Graph, Vec<f64>, Vec<f64>)> {
    validate_inputs(n, radius)?;
    let n_usize = n as usize;
    let radius = radius.max(0.0);
    let r2 = radius * radius;

    let mut xs: Vec<f64> = Vec::with_capacity(n_usize);
    let mut ys: Vec<f64> = Vec::with_capacity(n_usize);
    let mut rng = SplitMix64::new(seed);
    for _ in 0..n_usize {
        xs.push(rng.gen_unit());
        ys.push(rng.gen_unit());
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Trivial / degenerate cases: never push any edges. `radius == 0`
    // mirrors upstream's strict `<` comparison — a pair must be
    // *strictly* closer than radius to connect, so zero-radius means
    // no edges at all even if two points coincide.
    if n_usize < 2 || radius <= 0.0 {
        let g = Graph::new(n, false)?;
        return Ok((g, xs, ys));
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    if torus {
        for i in 0..n_usize {
            let xi = xs[i];
            let yi = ys[i];
            // Forward window — no wrap yet.
            let mut j = i + 1;
            while j < n_usize {
                let dx = xs[j] - xi;
                if dx >= radius {
                    break;
                }
                let mut dy = (ys[j] - yi).abs();
                if dy > 0.5 {
                    dy = 1.0 - dy;
                }
                if dx * dx + dy * dy < r2 {
                    edges.push((i as VertexId, j as VertexId));
                }
                j += 1;
            }
            // Wrap-around tail: only enters when the forward window
            // exhausted the rest of `xs`. Upstream `grg.c:142-156`
            // gates on `j == nodes`, which is true here once `while
            // j < n_usize` exits — by either `j == n` or
            // `xs[j] - xi >= radius`. The second case implies no
            // point at `j == n - k` could close the wrap window
            // anyway, so the `j == n` gate keeps the inner loop tight.
            if j == n_usize {
                let mut k = 0usize;
                while k < i {
                    // Wrapped x-distance: (1 - xi) + xs[k]
                    let dx = 1.0 - xi + xs[k];
                    if dx >= radius {
                        break;
                    }
                    // Upstream also enforces the forward gap exceeds
                    // radius to avoid double-counting the same pair —
                    // (xi - xs[k]) > radius means k is far enough to
                    // the *left* that the forward window from k would
                    // not have reached i in the first place.
                    if xi - xs[k] < radius {
                        k += 1;
                        continue;
                    }
                    let mut dy = (ys[k] - yi).abs();
                    if dy > 0.5 {
                        dy = 1.0 - dy;
                    }
                    if dx * dx + dy * dy < r2 {
                        // Emit (k, i) rather than (i, k) so the edge
                        // list stays sorted by lower endpoint, matching
                        // the forward-window order above.
                        edges.push((k as VertexId, i as VertexId));
                    }
                    k += 1;
                }
            }
        }
    } else {
        for i in 0..n_usize {
            let xi = xs[i];
            let yi = ys[i];
            let mut j = i + 1;
            while j < n_usize {
                let dx = xs[j] - xi;
                if dx >= radius {
                    break;
                }
                let dy = ys[j] - yi;
                if dx * dx + dy * dy < r2 {
                    edges.push((i as VertexId, j as VertexId));
                }
                j += 1;
            }
        }
    }

    let mut g = Graph::new(n, false)?;
    g.add_edges(edges)?;
    Ok((g, xs, ys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn edge_set(g: &Graph) -> HashSet<(VertexId, VertexId)> {
        let n_edges = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        (0..n_edges)
            .map(|eid| {
                let (a, b) = g.edge(eid).expect("edge id in bounds");
                if a <= b { (a, b) } else { (b, a) }
            })
            .collect()
    }

    #[test]
    fn n_zero_returns_empty_graph() {
        let g = grg_game(0, 0.1, false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_returns_singleton() {
        let g = grg_game(1, 0.5, false, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_radius_yields_empty_graph() {
        // Strict inequality dx² + dy² < 0 is never satisfied.
        let g = grg_game(50, 0.0, false, 0xDEAD).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn negative_radius_treated_as_zero() {
        let g = grg_game(50, -1.0, false, 0xDEAD).unwrap();
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn always_undirected() {
        let g = grg_game(10, 0.5, false, 1).unwrap();
        assert!(!g.is_directed());
        let g = grg_game(10, 0.5, true, 1).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn no_self_loops_no_multi_edges() {
        for &(n, r, torus) in &[
            (50u32, 0.2, false),
            (50, 0.2, true),
            (200, 0.05, false),
            (200, 0.05, true),
        ] {
            let g = grg_game(n, r, torus, 0x5EED).unwrap();
            let edges = edge_set(&g);
            assert_eq!(edges.len(), g.ecount(), "multi-edges in {n} {r} {torus}");
            for &(a, b) in &edges {
                assert_ne!(a, b, "self-loop in {n} {r} {torus}");
            }
        }
    }

    #[test]
    fn dense_radius_connects_almost_everything_plane() {
        // sqrt(2) > 1.41 covers the entire unit square: every pair is
        // strictly within range. Expected edges = n(n-1)/2.
        let n = 30u32;
        let g = grg_game(n, 2.0, false, 0xBEEF).unwrap();
        let expected = (n as usize) * (n as usize - 1) / 2;
        assert_eq!(g.ecount(), expected);
    }

    #[test]
    fn dense_radius_connects_almost_everything_torus() {
        // sqrt(0.5) ≈ 0.707 is enough on the torus to reach every other
        // point — but our sweep tail only enters when xi - xs[k] >=
        // radius, so at radius > 1 the tail is entirely empty and we
        // get a *forward-only* complete graph from the main sweep
        // anyway. The expected count remains n(n-1)/2.
        let n = 30u32;
        let g = grg_game(n, 2.0, true, 0xBEEF).unwrap();
        let expected = (n as usize) * (n as usize - 1) / 2;
        assert_eq!(g.ecount(), expected);
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let a = grg_game(100, 0.15, false, 0x1234).unwrap();
        let b = grg_game(100, 0.15, false, 0x1234).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        assert_eq!(edge_set(&a), edge_set(&b));
    }

    #[test]
    fn distinct_seeds_yield_distinct_graphs() {
        let a = grg_game(200, 0.1, false, 0x1).unwrap();
        let b = grg_game(200, 0.1, false, 0x2).unwrap();
        assert_ne!(edge_set(&a), edge_set(&b));
    }

    #[test]
    fn torus_adds_edges_versus_plane() {
        // Periodic boundaries can only connect *additional* pairs that
        // wrap across the unit square — never disconnect existing ones.
        // Run several seeds because the expected delta is positive only
        // in expectation; we verify monotonicity (torus >= plane) on
        // every seed.
        for seed in 0u64..16 {
            let plane = grg_game(80, 0.12, false, seed).unwrap();
            let torus = grg_game(80, 0.12, true, seed).unwrap();
            assert!(
                torus.ecount() >= plane.ecount(),
                "seed {seed}: torus={} plane={}",
                torus.ecount(),
                plane.ecount(),
            );
        }
    }

    #[test]
    fn coords_returned_match_graph() {
        let (g, xs, ys) = grg_game_with_coords(64, 0.1, false, 0x77).unwrap();
        assert_eq!(xs.len(), 64);
        assert_eq!(ys.len(), 64);
        // xs is sorted; ys is not.
        assert!(xs.windows(2).all(|w| w[0] <= w[1]));
        // Same seed via grg_game must produce the same edge set.
        let g2 = grg_game(64, 0.1, false, 0x77).unwrap();
        assert_eq!(edge_set(&g), edge_set(&g2));
    }

    #[test]
    fn nan_radius_errors() {
        assert!(grg_game(10, f64::NAN, false, 1).is_err());
    }

    #[test]
    fn inf_radius_errors() {
        assert!(grg_game(10, f64::INFINITY, false, 1).is_err());
    }

    #[test]
    fn expected_density_in_band() {
        // E[edges] = n·(n-1)/2 · π·r² for the plane interior — boundary
        // effects pull the actual mean down by a small constant. We use
        // a loose window centred on the bulk estimate so the test is
        // robust to RNG variance: ± 50 % of the predicted count.
        let n = 400u32;
        let r = 0.05;
        let predicted = (f64::from(n) * f64::from(n - 1) / 2.0) * std::f64::consts::PI * r * r;
        let g = grg_game(n, r, true, 0xBEEF).unwrap();
        let actual = g.ecount() as f64;
        assert!(
            actual > predicted * 0.5 && actual < predicted * 1.5,
            "expected ~{predicted:.1} edges, got {actual}",
        );
    }
}
