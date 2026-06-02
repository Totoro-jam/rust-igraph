//! k-regular random graph generator (ALGO-GN-008).
//!
//! Counterpart of `igraph_k_regular_game()` in
//! `references/igraph/src/games/k_regular.c:57-83`, which itself is a
//! thin wrapper over `igraph_degree_sequence_game()` (see
//! `references/igraph/src/games/degree_sequence.c`). We self-roll the
//! two methods used by the wrapper rather than port the full
//! degree-sequence machinery (~870 C lines covering eight sampling
//! modes); that keeps the dependency footprint focused on what
//! k-regular actually needs.
//!
//! ## Model
//!
//! Every vertex of the returned graph has degree exactly `k`. In the
//! directed case both the in- and out-degree are `k`.
//!
//! Two sampling regimes are exposed via the `multiple` flag, mirroring
//! the upstream split:
//!
//! * `multiple = true` — **configuration model**. Build `n·k` stubs
//!   (one per half-edge), pair them uniformly at random *without
//!   replacement*. Self-loops and parallel edges are allowed. Runtime
//!   `O(n + |E|)`.
//! * `multiple = false` — **fast-heuristic simple**. Same stub bag, but
//!   reject any pair that would form a self-loop or duplicate an
//!   existing edge; failed stubs are queued into the next sweep over
//!   the residual degree sequence; if a sweep ends with no feasible
//!   pair the whole attempt is restarted from scratch. This is the
//!   sampling scheme `IGRAPH_DEGSEQ_FAST_HEUR_SIMPLE` uses, and like
//!   the C reference it does **not** sample uniformly from the space
//!   of all simple k-regular graphs — every realisable graph appears
//!   with positive probability but not necessarily equal probability.
//!
//! ## Validation
//!
//! * `k = 0` always succeeds and returns an edgeless graph.
//! * Undirected simple: requires `k ≤ n - 1` and `n·k` even (handshake).
//! * Undirected multigraph: requires `n·k` even (handshake).
//! * Directed simple: requires `k ≤ n - 1`. The handshake parity
//!   constraint is automatically satisfied because in- and out-degrees
//!   match by construction.
//! * Directed multigraph: no degree-bound constraint, no parity
//!   constraint.
//! * `n·k` must fit in `u32` (otherwise the stub vector overflows).
//!
//! ## Determinism
//!
//! Fully deterministic in `(n, k, directed, multiple, seed)` via
//! `SplitMix64`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Hard cap on outer restarts of the simple-graph fast-heur loop.
///
/// In practice every realisable `(n, k)` with `k ≪ n − 1` succeeds on
/// the first attempt; the cap exists only so an adversarial input
/// (e.g. `k = n − 2` where the simple realisation is essentially
/// unique) cannot lock the caller into an unbounded retry loop.
const FAST_HEUR_MAX_RESTARTS: u32 = 1024;

fn validate(n: u32, k: u32, directed: bool, multiple: bool) -> IgraphResult<()> {
    let total = u64::from(n) * u64::from(k);
    if total > u64::from(u32::MAX) {
        return Err(IgraphError::InvalidArgument(format!(
            "k_regular_game: n * k ({total}) overflows u32"
        )));
    }
    if !directed && !multiple && k % 2 != 0 && n % 2 != 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "k_regular_game: undirected simple requires n*k even (got n={n}, k={k})"
        )));
    }
    if !directed && multiple && total % 2 != 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "k_regular_game: undirected multigraph requires n*k even (got n={n}, k={k})"
        )));
    }
    if !multiple && k > 0 && k > n.saturating_sub(1) {
        return Err(IgraphError::InvalidArgument(format!(
            "k_regular_game: simple graph requires k <= n - 1 (got n={n}, k={k})"
        )));
    }
    Ok(())
}

fn fisher_yates_shuffle<T>(slice: &mut [T], rng: &mut SplitMix64) {
    let len = slice.len();
    if len < 2 {
        return;
    }
    for i in (1..len).rev() {
        let j = rng.gen_index(i + 1);
        slice.swap(i, j);
    }
}

/// Build the per-vertex stub bag `[0, 0, ..., 1, 1, ..., n-1, n-1, ...]`
/// where each vertex appears exactly `k` times.
fn build_stubs(n: u32, k: u32) -> Vec<VertexId> {
    let n_us = n as usize;
    let k_us = k as usize;
    let mut stubs = Vec::with_capacity(n_us * k_us);
    for v in 0..n {
        for _ in 0..k {
            stubs.push(v);
        }
    }
    stubs
}

/// Configuration-model sampler (multigraph allowed). For the undirected
/// case it picks pairs from a single bag without replacement; for the
/// directed case it pairs one shuffled out-bag against one shuffled
/// in-bag.
fn configuration(
    n: u32,
    k: u32,
    directed: bool,
    rng: &mut SplitMix64,
) -> Vec<(VertexId, VertexId)> {
    if k == 0 {
        return Vec::new();
    }
    let n_us = n as usize;
    let k_us = k as usize;
    let total_stubs = n_us * k_us;

    if directed {
        let mut out_bag = build_stubs(n, k);
        let mut in_bag = build_stubs(n, k);
        fisher_yates_shuffle(&mut out_bag, rng);
        fisher_yates_shuffle(&mut in_bag, rng);
        out_bag.into_iter().zip(in_bag).collect()
    } else {
        // Undirected: pop two random stubs at a time; replace-with-last
        // keeps remaining draws uniform. Permits self-loops and
        // parallel edges by design.
        let mut bag = build_stubs(n, k);
        let m = total_stubs / 2;
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(m);
        for _ in 0..m {
            let i = rng.gen_index(bag.len());
            let from = bag[i];
            let last = bag.len() - 1;
            bag.swap(i, last);
            bag.pop();
            let j = rng.gen_index(bag.len());
            let to = bag[j];
            let last = bag.len() - 1;
            bag.swap(j, last);
            bag.pop();
            edges.push((from, to));
        }
        edges
    }
}

/// Fast-heur simple sampler — undirected.
fn fast_heur_undirected(
    n: u32,
    k: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    if k == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;
    let total_edges = (n_us * k as usize) / 2;

    for _restart in 0..FAST_HEUR_MAX_RESTARTS {
        let mut residual: Vec<u32> = vec![k; n_us];
        let mut adj: Vec<HashSet<VertexId>> = vec![HashSet::new(); n_us];
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

        let mut attempt_failed = false;

        loop {
            // Rebuild stub bag from the residual degree sequence.
            let mut stubs: Vec<VertexId> = Vec::new();
            for (v, &d) in residual.iter().enumerate() {
                for _ in 0..d {
                    stubs.push(v as VertexId);
                }
            }
            if stubs.is_empty() {
                break;
            }
            fisher_yates_shuffle(&mut stubs, rng);

            // Zero residuals; carry forward only the stubs that fail to
            // pair this sweep.
            residual.fill(0);
            let mut incomplete: HashSet<VertexId> = HashSet::new();

            // Walk the shuffled stubs two at a time.
            for pair in stubs.chunks_exact(2) {
                let mut from = pair[0];
                let mut to = pair[1];
                if from > to {
                    std::mem::swap(&mut from, &mut to);
                }
                if from == to || adj[from as usize].contains(&to) {
                    residual[from as usize] += 1;
                    residual[to as usize] += 1;
                    incomplete.insert(from);
                    incomplete.insert(to);
                } else {
                    adj[from as usize].insert(to);
                    edges.push((from, to));
                }
            }

            if incomplete.is_empty() {
                return Ok(edges);
            }

            // Feasibility: is there at least one pair (a, b) in
            // `incomplete` with a != b and no existing edge? If not,
            // this attempt is wedged — restart from scratch.
            if !undirected_pair_feasible(&incomplete, &adj) {
                attempt_failed = true;
                break;
            }
        }

        if !attempt_failed {
            // Logically unreachable — we either return inside the loop
            // when `incomplete` is empty or break with `attempt_failed`.
            // The `break` happens only on the `stubs.is_empty()` early
            // exit, which itself implies success. Guard against future
            // refactors.
            // No edges produced and no incomplete vertices → empty graph.
            return Ok(edges);
        }
    }

    Err(IgraphError::Internal(
        "k_regular_game: fast-heur exceeded restart budget; try multiple=true",
    ))
}

fn undirected_pair_feasible(incomplete: &HashSet<VertexId>, adj: &[HashSet<VertexId>]) -> bool {
    let mut sorted: Vec<VertexId> = incomplete.iter().copied().collect();
    sorted.sort_unstable();
    for (i, &a) in sorted.iter().enumerate() {
        for &b in sorted.iter().skip(i + 1) {
            // `a < b` by construction after sort, so the adjacency
            // lookup mirrors the canonical (min, max) edge encoding
            // used while emitting edges above.
            if !adj[a as usize].contains(&b) {
                return true;
            }
        }
    }
    false
}

/// Fast-heur simple sampler — directed.
fn fast_heur_directed(
    n: u32,
    k: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    if k == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;
    let total_edges = n_us * k as usize;

    for _restart in 0..FAST_HEUR_MAX_RESTARTS {
        let mut residual_out: Vec<u32> = vec![k; n_us];
        let mut residual_in: Vec<u32> = vec![k; n_us];
        let mut adj: Vec<HashSet<VertexId>> = vec![HashSet::new(); n_us];
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

        let mut attempt_failed = false;

        loop {
            let mut out_stubs: Vec<VertexId> = Vec::new();
            let mut in_stubs: Vec<VertexId> = Vec::new();
            for v in 0..n_us {
                for _ in 0..residual_out[v] {
                    out_stubs.push(v as VertexId);
                }
                for _ in 0..residual_in[v] {
                    in_stubs.push(v as VertexId);
                }
            }
            if out_stubs.is_empty() {
                break;
            }
            fisher_yates_shuffle(&mut out_stubs, rng);

            residual_out.fill(0);
            residual_in.fill(0);
            let mut incomplete_out: HashSet<VertexId> = HashSet::new();
            let mut incomplete_in: HashSet<VertexId> = HashSet::new();

            for (&from, &to) in out_stubs.iter().zip(in_stubs.iter()) {
                if from == to || adj[from as usize].contains(&to) {
                    residual_out[from as usize] += 1;
                    residual_in[to as usize] += 1;
                    incomplete_out.insert(from);
                    incomplete_in.insert(to);
                } else {
                    adj[from as usize].insert(to);
                    edges.push((from, to));
                }
            }

            if incomplete_out.is_empty() {
                return Ok(edges);
            }
            if !directed_pair_feasible(&incomplete_out, &incomplete_in, &adj) {
                attempt_failed = true;
                break;
            }
        }

        if !attempt_failed {
            return Ok(edges);
        }
    }

    Err(IgraphError::Internal(
        "k_regular_game: fast-heur exceeded restart budget; try multiple=true",
    ))
}

fn directed_pair_feasible(
    incomplete_out: &HashSet<VertexId>,
    incomplete_in: &HashSet<VertexId>,
    adj: &[HashSet<VertexId>],
) -> bool {
    for &from in incomplete_out {
        for &to in incomplete_in {
            if from != to && !adj[from as usize].contains(&to) {
                return true;
            }
        }
    }
    false
}

/// Generate a random k-regular graph on `n` vertices.
///
/// Every vertex has degree exactly `k` (in the directed case, both
/// in-degree and out-degree are `k`).
///
/// * `n` — vertex count.
/// * `k` — degree.
/// * `directed` — if `true`, returns a directed graph where every
///   vertex has out-degree = in-degree = `k`.
/// * `multiple` — if `true`, samples via the configuration model and
///   permits self-loops and parallel edges; if `false`, samples via the
///   fast-heuristic simple-graph sampler.
/// * `seed` — initialises an internal `SplitMix64` PRNG. The same
///   `(n, k, directed, multiple, seed)` always yields the same graph.
///
/// # Errors
///
/// * `InvalidArgument` — `n * k` overflows `u32`.
/// * `InvalidArgument` — undirected and `n * k` is odd (handshake
///   violation).
/// * `InvalidArgument` — `multiple = false` and `k > n - 1` (no simple
///   realisation exists).
/// * `Internal` — `multiple = false` and the fast-heur sampler
///   exceeded its restart budget. Switch to `multiple = true` or pick a
///   smaller `k`.
///
/// # Examples
///
/// ```
/// use rust_igraph::k_regular_game;
///
/// // 20-vertex 4-regular simple undirected graph.
/// let g = k_regular_game(20, 4, false, false, 0xC0FF_EE00).unwrap();
/// assert_eq!(g.vcount(), 20);
/// assert_eq!(g.ecount(), 40);  // n * k / 2
/// assert!(!g.is_directed());
/// ```
pub fn k_regular_game(
    n: u32,
    k: u32,
    directed: bool,
    multiple: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(n, k, directed, multiple)?;
    if n == 0 {
        return Graph::new(0, directed);
    }
    let mut rng = SplitMix64::new(seed);
    let edges = if multiple {
        configuration(n, k, directed, &mut rng)
    } else if directed {
        fast_heur_directed(n, k, &mut rng)?
    } else {
        fast_heur_undirected(n, k, &mut rng)?
    };

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn degree_sequence(g: &Graph) -> Vec<u32> {
        let mut deg = vec![0u32; g.vcount() as usize];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("edge id in bounds");
            if g.is_directed() {
                deg[u as usize] += 1;
            } else {
                // Self-loop contributes 2 to its endpoint's undirected
                // degree; the two increments coincide on u == v.
                deg[u as usize] += 1;
                deg[v as usize] += 1;
            }
        }
        deg
    }

    fn out_degree_sequence(g: &Graph) -> Vec<u32> {
        let mut deg = vec![0u32; g.vcount() as usize];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        for eid in 0..m {
            let (u, _v) = g.edge(eid).expect("edge id in bounds");
            deg[u as usize] += 1;
        }
        deg
    }

    fn in_degree_sequence(g: &Graph) -> Vec<u32> {
        let mut deg = vec![0u32; g.vcount() as usize];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        for eid in 0..m {
            let (_u, v) = g.edge(eid).expect("edge id in bounds");
            deg[v as usize] += 1;
        }
        deg
    }

    fn is_simple(g: &Graph) -> bool {
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("edge id in bounds");
            if u == v {
                return false;
            }
            let key = if g.is_directed() {
                (u, v)
            } else {
                (u.min(v), u.max(v))
            };
            if !seen.insert(key) {
                return false;
            }
        }
        true
    }

    #[test]
    fn empty_graph() {
        let g = k_regular_game(0, 0, false, false, 1).expect("empty");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn k_zero_returns_edgeless() {
        let g = k_regular_game(7, 0, false, false, 1).expect("k=0");
        assert_eq!(g.vcount(), 7);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_simple_2regular_on_6() {
        let g = k_regular_game(6, 2, false, false, 0xABCD).expect("ok");
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 6); // n*k/2
        assert!(is_simple(&g));
        for d in degree_sequence(&g) {
            assert_eq!(d, 2);
        }
    }

    #[test]
    fn undirected_simple_4regular_on_10() {
        let g = k_regular_game(10, 4, false, false, 0xDEAD_BEEF).expect("ok");
        assert_eq!(g.ecount(), 20);
        assert!(is_simple(&g));
        for d in degree_sequence(&g) {
            assert_eq!(d, 4);
        }
    }

    #[test]
    fn directed_simple_3regular_on_8() {
        let g = k_regular_game(8, 3, true, false, 0x1234_5678).expect("ok");
        assert_eq!(g.ecount(), 24); // n*k
        assert!(is_simple(&g));
        for d in out_degree_sequence(&g) {
            assert_eq!(d, 3);
        }
        for d in in_degree_sequence(&g) {
            assert_eq!(d, 3);
        }
    }

    #[test]
    fn undirected_multi_2regular_on_3() {
        // n*k = 6 even, so the handshake is satisfied. Self-loops and
        // parallel edges are allowed.
        let g = k_regular_game(3, 2, false, true, 0xFEED).expect("ok");
        assert_eq!(g.ecount(), 3);
        for d in degree_sequence(&g) {
            assert_eq!(d, 2);
        }
    }

    #[test]
    fn directed_multi_3regular_on_4() {
        let g = k_regular_game(4, 3, true, true, 0xCAFE).expect("ok");
        assert_eq!(g.ecount(), 12);
        for d in out_degree_sequence(&g) {
            assert_eq!(d, 3);
        }
        for d in in_degree_sequence(&g) {
            assert_eq!(d, 3);
        }
    }

    #[test]
    fn undirected_simple_handshake_violation() {
        // n=5, k=3 → n*k=15 odd
        let err = k_regular_game(5, 3, false, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn undirected_multi_handshake_violation() {
        let err = k_regular_game(5, 3, false, true, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn simple_degree_too_high() {
        // k=n-1 is OK (complete graph); k=n is not.
        let g = k_regular_game(5, 4, false, false, 7).expect("k = n - 1 ok");
        assert_eq!(g.ecount(), 10);
        assert!(is_simple(&g));

        let err = k_regular_game(5, 5, false, false, 7).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let a = k_regular_game(20, 4, false, false, 0x42).expect("a");
        let b = k_regular_game(20, 4, false, false, 0x42).expect("b");
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
            let m = u32::try_from(g.ecount()).expect("fits");
            (0..m).map(|e| g.edge(e).expect("ok")).collect()
        };
        assert_eq!(collect(&a), collect(&b));
    }

    #[test]
    fn different_seeds_different_graphs() {
        let a = k_regular_game(20, 4, false, false, 1).expect("a");
        let b = k_regular_game(20, 4, false, false, 2).expect("b");
        let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
            let m = u32::try_from(g.ecount()).expect("fits");
            (0..m).map(|e| g.edge(e).expect("ok")).collect()
        };
        // With n=20, k=4 there are vastly more than 2 realisations,
        // so two distinct seeds should almost certainly produce
        // distinct edge lists.
        assert_ne!(collect(&a), collect(&b));
    }

    #[test]
    fn undirected_simple_k_equals_n_minus_one_is_complete() {
        // 4-regular on 5 vertices = K_5.
        let g = k_regular_game(5, 4, false, false, 0).expect("ok");
        assert_eq!(g.ecount(), 10); // C(5,2)
        assert!(is_simple(&g));
        // Every pair must be present.
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        let m = u32::try_from(g.ecount()).expect("fits");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("ok");
            let key = (u.min(v), u.max(v));
            seen.insert(key);
        }
        for a in 0..5u32 {
            for b in (a + 1)..5 {
                assert!(seen.contains(&(a, b)), "missing edge {a}-{b}");
            }
        }
    }

    #[test]
    fn n_times_k_overflow() {
        let err = k_regular_game(u32::MAX, 2, false, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn directed_simple_no_self_loops() {
        let g = k_regular_game(7, 3, true, false, 0x7777).expect("ok");
        let m = u32::try_from(g.ecount()).expect("fits");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("ok");
            assert_ne!(u, v, "self-loop in simple directed run");
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            .. ProptestConfig::default()
        })]

        // Undirected simple: every vertex has degree exactly k, no
        // self-loops, no parallel edges.
        #[test]
        fn undirected_simple_invariants(
            n in 2u32..=24,
            k_in in 0u32..=6,
            seed in any::<u64>(),
        ) {
            let k = if n % 2 == 1 && k_in % 2 == 1 { k_in + 1 } else { k_in };
            let k = if k > n.saturating_sub(1) { n.saturating_sub(1) } else { k };
            let k = if n % 2 == 1 && k % 2 == 1 { k.saturating_sub(1) } else { k };

            let g = k_regular_game(n, k, false, false, seed)
                .expect("valid params");
            prop_assert_eq!(g.vcount() as u32, n);

            let n_us = n as usize;
            let m_expected = (n_us * k as usize) / 2;
            prop_assert_eq!(g.ecount(), m_expected);

            let mut deg = vec![0u32; n_us];
            let mut seen: std::collections::HashSet<(u32, u32)> =
                std::collections::HashSet::new();
            let m = u32::try_from(g.ecount()).expect("fits");
            for eid in 0..m {
                let (u, v) = g.edge(eid).expect("ok");
                prop_assert_ne!(u, v, "self-loop in simple run");
                let key = (u.min(v), u.max(v));
                prop_assert!(seen.insert(key), "duplicate edge");
                deg[u as usize] += 1;
                deg[v as usize] += 1;
            }
            for d in deg {
                prop_assert_eq!(d, k);
            }
        }

        // Directed simple: out-deg = in-deg = k everywhere, no self,
        // no duplicates within a direction.
        #[test]
        fn directed_simple_invariants(
            n in 2u32..=18,
            k in 0u32..=5,
            seed in any::<u64>(),
        ) {
            let k = if k > n.saturating_sub(1) { n.saturating_sub(1) } else { k };
            let g = k_regular_game(n, k, true, false, seed)
                .expect("valid params");
            let n_us = n as usize;
            prop_assert_eq!(g.ecount(), n_us * k as usize);

            let mut out_d = vec![0u32; n_us];
            let mut in_d = vec![0u32; n_us];
            let mut seen: std::collections::HashSet<(u32, u32)> =
                std::collections::HashSet::new();
            let m = u32::try_from(g.ecount()).expect("fits");
            for eid in 0..m {
                let (u, v) = g.edge(eid).expect("ok");
                prop_assert_ne!(u, v, "self-loop");
                prop_assert!(seen.insert((u, v)), "parallel directed edge");
                out_d[u as usize] += 1;
                in_d[v as usize] += 1;
            }
            for d in out_d {
                prop_assert_eq!(d, k);
            }
            for d in in_d {
                prop_assert_eq!(d, k);
            }
        }

        // Undirected multigraph: degree = k everywhere (loops contribute
        // 2). No graphicality constraint beyond parity.
        #[test]
        fn undirected_multi_invariants(
            n in 1u32..=20,
            k in 0u32..=8,
            seed in any::<u64>(),
        ) {
            let n_us = n as usize;
            let total = n_us * k as usize;
            if total % 2 != 0 {
                return Ok(());
            }
            let g = k_regular_game(n, k, false, true, seed)
                .expect("parity ok");
            prop_assert_eq!(g.ecount(), total / 2);
            let mut deg = vec![0u32; n_us];
            let m = u32::try_from(g.ecount()).expect("fits");
            for eid in 0..m {
                let (u, v) = g.edge(eid).expect("ok");
                deg[u as usize] += 1;
                deg[v as usize] += 1; // works for self-loops too
            }
            for d in deg {
                prop_assert_eq!(d, k);
            }
        }

        // Determinism: same (n,k,d,m,seed) → identical edge list.
        #[test]
        fn determinism(
            n in 2u32..=16,
            k in 0u32..=4,
            directed in any::<bool>(),
            multiple in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let n_us = n as usize;
            let total = n_us * k as usize;
            if !directed && (total % 2 != 0) {
                return Ok(());
            }
            if !multiple && k > n.saturating_sub(1) {
                return Ok(());
            }
            let a = k_regular_game(n, k, directed, multiple, seed)
                .expect("valid");
            let b = k_regular_game(n, k, directed, multiple, seed)
                .expect("valid");
            let collect = |g: &Graph| -> Vec<(u32, u32)> {
                let m = u32::try_from(g.ecount()).expect("fits");
                (0..m).map(|e| g.edge(e).expect("ok")).collect()
            };
            prop_assert_eq!(collect(&a), collect(&b));
        }
    }
}
