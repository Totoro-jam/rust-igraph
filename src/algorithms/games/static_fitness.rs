//! Static-fitness random graph generator (ALGO-GN-013).
//!
//! Counterpart of `igraph_static_fitness_game()` from
//! `references/igraph/src/games/static_fitness.c:110` and the
//! power-law shaping convenience `igraph_static_power_law_game()`
//! from line 387 of the same file.
//!
//! The model (Goh–Kahng–Kim 2001) starts from `n` disconnected
//! vertices, each carrying a non-negative *fitness* value. For each of
//! the `m` desired edges:
//!
//! 1. Sample the source vertex `i` with probability proportional to
//!    `fitness_out[i]` (cumulative-sum + binary search).
//! 2. Sample the target vertex `j` with probability proportional to
//!    `fitness_in[j]` (or `fitness_out[j]` when undirected).
//! 3. Skip if it is a self-loop and `loops = false`. Skip and retry if
//!    the edge already exists and `multiple = false`. Otherwise commit.
//!
//! The "no-multi" mode uses a per-source `HashSet` of accepted
//! neighbours to detect duplicates in `O(1)` amortised — upstream uses
//! per-vertex sorted vectors, which gives `O(log d_i)` lookups; both
//! choices land at the same asymptotic `O(|V| + |E| log |E|)` overall.
//!
//! ## Two public entry points
//!
//! * [`static_fitness_game`] — the primitive: caller supplies the
//!   per-vertex fitness vector(s) directly.
//! * [`static_power_law_game`] — convenience that builds power-law
//!   fitness vectors `i^α` (with optional Cho et al. finite-size
//!   correction) and forwards to `static_fitness_game`.
//!
//! ## Difference from Chung–Lu (ALGO-GN-012)
//!
//! Chung–Lu fixes the **expected** degree of every vertex but the
//! realised edge count fluctuates with the variant. Here we instead
//! fix the **edge count** exactly at `m` and only the *shape* of the
//! degree distribution is controlled by the fitness vector. The two
//! samplers are also algorithmically distinct: Chung–Lu uses
//! Miller–Hagberg geometric skip, this one uses cumulative-sum +
//! binsearch.
//!
//! ## Determinism
//!
//! Reproducible given the inputs and `seed` against the shared
//! `SplitMix64` PRNG. The stream is *not* portable
//! to upstream igraph's GLIBC-style RNG, so conformance assertions are
//! structural (vertex/edge counts, validation rules, expected
//! degree-fitness correlation) rather than bit-exact.
//!
//! ## References
//!
//! * Goh K-I, Kahng B, Kim D: *Universal behaviour of load
//!   distribution in scale-free networks*. Phys. Rev. Lett.
//!   **87**(27):278701 (2001).
//! * Chung F, Lu L: *Connected components in a random graph with given
//!   degree sequences*. Annals of Combinatorics **6**, 125–145 (2002).
//! * Cho YS, Kim JS, Park J, Kahng B, Kim D: *Percolation transitions
//!   in scale-free networks under the Achlioptas process*. Phys. Rev.
//!   Lett. **103**:135702 (2009).

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::similar_names,
    clippy::many_single_char_names
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Largest vertex count we accept. Matches the `chung_lu` guard
/// (`IGRAPH_MAX_EXACT_REAL == 2^53`), the largest integer exactly
/// representable as `f64`. On 32-bit targets `usize::MAX < 2^53`, so
/// the cap is effectively the address-space limit and the const stays
/// valid across pointer widths.
const MAX_NODES: usize = if usize::BITS >= 64 {
    1usize.wrapping_shl(53)
} else {
    usize::MAX
};

fn check_fitness(label: &str, fitness: &[f64]) -> IgraphResult<()> {
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for (i, &x) in fitness.iter().enumerate() {
        if x.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}[{i}] is NaN; static-fitness scores must be finite"
            )));
        }
        if x < min_v {
            min_v = x;
        }
        if x > max_v {
            max_v = x;
        }
    }
    if fitness.is_empty() {
        return Ok(());
    }
    if min_v < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "{label} must be non-negative; got minimum {min_v}"
        )));
    }
    if !max_v.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "{label} must be finite; got maximum {max_v}"
        )));
    }
    Ok(())
}

/// Build the cumulative-sum vector. `cum[i] = Σ_{k=0..=i} fitness[k]`.
fn cumulative_sum(fitness: &[f64]) -> Vec<f64> {
    let mut cum = Vec::with_capacity(fitness.len());
    let mut acc = 0.0_f64;
    for &x in fitness {
        acc += x;
        cum.push(acc);
    }
    cum
}

/// Binary search: smallest `i` such that `cum[i] >= target`. Assumes
/// `cum` is non-decreasing and `target ∈ [0, cum.last()]`.
fn binsearch_cum(cum: &[f64], target: f64) -> usize {
    // Standard branchless lower_bound for non-strict monotone arrays.
    let mut lo = 0usize;
    let mut hi = cum.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cum[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo >= cum.len() { cum.len() - 1 } else { lo }
}

/// Count vertices with strictly positive fitness.
fn count_active(fitness: &[f64]) -> usize {
    fitness.iter().filter(|&&x| x > 0.0).count()
}

/// Compute the upper bound `max_no_of_edges` for the given fitness
/// layout, mirroring the C reference. Returns the float-valued bound so
/// callers can compare against the requested edge count without
/// worrying about `n * (n - 1)` overflowing `usize` for huge `n`.
fn max_edges(fitness_out: &[f64], fitness_in: Option<&[f64]>, loops: bool) -> f64 {
    let n = fitness_out.len();
    if n == 0 {
        return 0.0;
    }
    if let Some(fin) = fitness_in {
        // Directed: out-active × in-active, minus the diagonal when
        // self-loops are forbidden (only the both-active diagonal slots
        // disappear).
        let mut outn = 0usize;
        let mut inn = 0usize;
        let mut bothn = 0usize;
        for i in 0..n {
            let o = fitness_out[i] != 0.0;
            let ii = fin[i] != 0.0;
            if o {
                outn += 1;
            }
            if ii {
                inn += 1;
            }
            if o && ii {
                bothn += 1;
            }
        }
        let prod = outn as f64 * inn as f64;
        if loops { prod } else { prod - bothn as f64 }
    } else {
        // Undirected: choose 2 over active count, plus the diagonal
        // when self-loops are allowed.
        let active = count_active(fitness_out) as f64;
        if loops {
            active * (active + 1.0) / 2.0
        } else {
            active * (active - 1.0) / 2.0
        }
    }
}

/// Sample a random graph from the **static-fitness** model
/// (Goh–Kahng–Kim 2001).
///
/// * `no_of_edges` — exact number of edges to generate.
/// * `fitness_out` — non-negative, finite per-vertex fitness. Length
///   determines the resulting graph's vertex count `n`. Higher values
///   mean a vertex is more likely to be picked as the (out-)endpoint of
///   each edge; the *expected* (out-)degree is proportional to the
///   fitness, but unlike Chung–Lu the realised degrees fluctuate around
///   that mean and the total edge count is fixed.
/// * `fitness_in` — when `Some`, generates a **directed** graph with
///   independent out- and in-fitness. Must have the same length as
///   `fitness_out`. When `None`, generates an **undirected** graph.
/// * `loops` — when `true`, self-loops are permitted (and may be
///   sampled).
/// * `multiple` — when `true`, parallel edges are permitted; when
///   `false`, every accepted edge is unique.
/// * `seed` — initialises an internal `SplitMix64` PRNG so the output
///   is reproducible given the inputs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if:
/// * any fitness value is negative, NaN, or non-finite (`±∞`);
/// * `fitness_in` length differs from `fitness_out` length;
/// * the upper-bound `max_no_of_edges` is zero but `no_of_edges > 0`
///   (e.g. one-vertex graph without self-loops, or all-zero fitness);
/// * `!multiple && no_of_edges > max_no_of_edges` (impossible to
///   pack that many unique edges into the available slots);
/// * the requested vertex count exceeds the `u32::MAX` bound on
///   `VertexId` (or the more conservative `2^53` upper bound).
///
/// # Complexity
///
/// `O(|V| + |E| log |V|)` — the per-edge `log |V|` comes from the
/// cumulative-fitness binary search; for `multiple = false` an extra
/// `O(log d_i)` (here `O(1)` amortised via `HashSet`) per accepted edge
/// is added for the duplicate check.
///
/// # Examples
///
/// ```
/// use rust_igraph::static_fitness_game;
///
/// // Five vertices, decaying fitness, 8 undirected simple edges.
/// let fitness = [5.0, 4.0, 3.0, 2.0, 1.0];
/// let g = static_fitness_game(8, &fitness, None, false, false, 42).unwrap();
/// assert_eq!(g.vcount(), 5);
/// assert_eq!(g.ecount(), 8);
/// assert!(!g.is_directed());
/// ```
pub fn static_fitness_game(
    no_of_edges: u32,
    fitness_out: &[f64],
    fitness_in: Option<&[f64]>,
    loops: bool,
    multiple: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    let n = fitness_out.len();
    let directed = fitness_in.is_some();

    if n > MAX_NODES {
        return Err(IgraphError::InvalidArgument(format!(
            "static-fitness vertex count {n} exceeds the largest exactly representable f64 integer (2^53)"
        )));
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument(format!("static-fitness vertex count {n} exceeds u32::MAX"))
    })?;

    // n == 0 is a degenerate but legitimate case in the C reference
    // (returns an empty graph) — even when edges were requested. We
    // mirror that for backwards compatibility with upstream test
    // fixtures.
    if n == 0 {
        return Graph::new(0, directed);
    }

    if let Some(fin) = fitness_in {
        if fin.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "fitness_in length {} does not match fitness_out length {n}",
                fin.len()
            )));
        }
    }

    check_fitness("fitness_out", fitness_out)?;
    if let Some(fin) = fitness_in {
        check_fitness("fitness_in", fin)?;
    }

    // Compute the upper bound on realisable edges and apply both the
    // "no edges possible" and "too many edges" gates.
    let max_e = max_edges(fitness_out, fitness_in, loops);
    if max_e <= 0.0 && no_of_edges > 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "No edges are possible with the given fitness scores, but {no_of_edges} edges were requested"
        )));
    }
    if !multiple && f64::from(no_of_edges) > max_e {
        return Err(IgraphError::InvalidArgument(format!(
            "Requested {no_of_edges} simple edges but only {max_e} are possible with the given fitness scores"
        )));
    }

    if no_of_edges == 0 {
        return Graph::new(n_u32, directed);
    }

    // Build cumulative-sum vectors.
    let cum_out = cumulative_sum(fitness_out);
    let max_out = *cum_out.last().expect("n > 0 ⇒ cum_out non-empty");
    let (cum_in_storage, cum_in_view, max_in): (Vec<f64>, &[f64], f64) = match fitness_in {
        Some(fin) => {
            let c = cumulative_sum(fin);
            let m = *c.last().expect("n > 0 ⇒ cum_in non-empty");
            (c, &[], m)
        }
        None => (Vec::new(), &cum_out, max_out),
    };
    // We can't bind a borrow to `cum_in_storage` and return `(_, &_)`
    // in the same `let`, so do the borrow here.
    let cum_in: &[f64] = if directed {
        &cum_in_storage
    } else {
        cum_in_view
    };

    if max_out <= 0.0 || max_in <= 0.0 {
        // Already covered by max_e == 0 above (since active count == 0
        // ⇒ max_e == 0) but cheap to be defensive against numerical
        // edge cases.
        return Graph::new(n_u32, directed);
    }

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(no_of_edges as usize);

    if multiple {
        // Trivial sample-and-keep loop.
        let mut remaining = no_of_edges;
        while remaining > 0 {
            let from = pick(&cum_out, max_out, &mut rng);
            let to = pick(cum_in, max_in, &mut rng);
            if !loops && from == to {
                continue;
            }
            edges.push((from as VertexId, to as VertexId));
            remaining -= 1;
        }
    } else {
        // Reject duplicates. For undirected graphs canonicalise to
        // `(min, max)` so the same pair sampled from either direction
        // collapses to one entry in the set.
        let mut seen: HashSet<(u32, u32)> = HashSet::with_capacity(no_of_edges as usize);
        let mut remaining = no_of_edges;
        while remaining > 0 {
            let from = pick(&cum_out, max_out, &mut rng);
            let to = pick(cum_in, max_in, &mut rng);
            if !loops && from == to {
                continue;
            }
            let key = if directed || from <= to {
                (from as u32, to as u32)
            } else {
                (to as u32, from as u32)
            };
            if !seen.insert(key) {
                continue;
            }
            edges.push((from as VertexId, to as VertexId));
            remaining -= 1;
        }
    }

    let mut g = Graph::new(n_u32, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[inline]
fn pick(cum: &[f64], total: f64, rng: &mut SplitMix64) -> usize {
    let u = rng.gen_unit();
    let target = u * total;
    binsearch_cum(cum, target)
}

/// Sample a random graph whose degree distribution follows a
/// **power law** of prescribed exponent(s) (Goh et al. 2001).
///
/// Builds the fitness vector `fitness[i] = j^α` with
/// `j = n, n-1, …, 1` and `α = -1 / (exponent - 1)`, optionally with
/// the Cho et al. (2009) finite-size correction shifting `j` upward,
/// then delegates to [`static_fitness_game`].
///
/// * `no_of_nodes` — vertex count `n`.
/// * `no_of_edges` — exact number of edges to generate.
/// * `exponent_out` — power-law exponent for the (out-)degree
///   distribution. Must be `>= 2`. Pass [`f64::INFINITY`] to recover an
///   Erdős–Rényi sampler.
/// * `exponent_in` — `Some(e)` selects a **directed** graph with an
///   independent in-degree exponent (also `>= 2`); the in-fitness
///   vector is randomly shuffled before sampling to decorrelate the
///   in- and out-degree sequences. `None` selects an **undirected**
///   graph.
/// * `loops`, `multiple` — passed through to `static_fitness_game`.
/// * `finite_size_correction` — when `true`, apply the Cho et al.
///   shift on each exponent that satisfies `α < -0.5` (equivalently
///   `exponent < 3`).
/// * `seed` — initialises an internal `SplitMix64` PRNG so the
///   output is reproducible given the inputs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if:
/// * `exponent_out < 2` or `exponent_in = Some(e)` with `e < 2`;
/// * `exponent_out` is NaN, or `exponent_in = Some(e)` with NaN;
/// * any condition listed under [`static_fitness_game`] fires when
///   forwarding the synthesised fitness vectors.
///
/// # Examples
///
/// ```
/// use rust_igraph::static_power_law_game;
///
/// let g = static_power_law_game(50, 100, 2.5, None, false, true, true, 7).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert_eq!(g.ecount(), 100);
/// assert!(!g.is_directed());
/// ```
#[allow(clippy::too_many_arguments)]
pub fn static_power_law_game(
    no_of_nodes: u32,
    no_of_edges: u32,
    exponent_out: f64,
    exponent_in: Option<f64>,
    loops: bool,
    multiple: bool,
    finite_size_correction: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    if exponent_out.is_nan() {
        return Err(IgraphError::InvalidArgument(
            "exponent_out must not be NaN".into(),
        ));
    }
    if exponent_out < 2.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "Out-degree exponent must be >= 2, got {exponent_out}"
        )));
    }
    if let Some(e) = exponent_in {
        if e.is_nan() {
            return Err(IgraphError::InvalidArgument(
                "exponent_in must not be NaN".into(),
            ));
        }
        if e < 2.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "For directed graphs the in-degree exponent must be >= 2, got {e}"
            )));
        }
    }

    let n = no_of_nodes as usize;
    let directed = exponent_in.is_some();
    if n == 0 {
        return Graph::new(0, directed);
    }

    let fitness_out = build_power_law_fitness(n, exponent_out, finite_size_correction);

    if let Some(e_in) = exponent_in {
        let mut fitness_in = build_power_law_fitness(n, e_in, finite_size_correction);
        // Decorrelate in- and out-fitnesses by Fisher–Yates shuffling
        // the in-fitness vector — this is what upstream igraph does.
        let mut shuf_rng = SplitMix64::new(seed.wrapping_add(0xDEAD_BEEF_CAFE_BABE));
        fisher_yates(&mut fitness_in, &mut shuf_rng);
        static_fitness_game(
            no_of_edges,
            &fitness_out,
            Some(&fitness_in),
            loops,
            multiple,
            seed,
        )
    } else {
        static_fitness_game(no_of_edges, &fitness_out, None, loops, multiple, seed)
    }
}

/// Build the fitness vector `j^α` with `j = n, n-1, …, 1`, optionally
/// shifted up by the Cho et al. finite-size correction.
fn build_power_law_fitness(n: usize, exponent: f64, finite_size_correction: bool) -> Vec<f64> {
    let alpha = if exponent.is_finite() {
        -1.0 / (exponent - 1.0)
    } else {
        0.0
    };

    let mut j = n as f64;
    if finite_size_correction && alpha < -0.5 {
        // Cho et al. (2009), first page first column + footnote 7.
        let shift = (n as f64).powf(1.0 + 0.5 / alpha)
            * (10.0 * std::f64::consts::SQRT_2 * (1.0 + alpha)).powf(-1.0 / alpha)
            - 1.0;
        j += shift;
    }
    if j < n as f64 {
        j = n as f64;
    }

    let mut fitness = Vec::with_capacity(n);
    for _ in 0..n {
        fitness.push(j.powf(alpha));
        j -= 1.0;
    }
    fitness
}

/// In-place Fisher–Yates shuffle using the supplied PRNG.
fn fisher_yates(v: &mut [f64], rng: &mut SplitMix64) {
    let n = v.len();
    if n <= 1 {
        return;
    }
    for i in (1..n).rev() {
        let j = rng.gen_index(i + 1);
        v.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(g: &Graph, n: usize) -> Vec<u32> {
        let mut d = vec![0u32; n];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("edge id in bounds");
            d[u as usize] += 1;
            if u != v {
                d[v as usize] += 1;
            }
        }
        d
    }

    fn directed_in_out(g: &Graph, n: usize) -> (Vec<u32>, Vec<u32>) {
        let mut out = vec![0u32; n];
        let mut din = vec![0u32; n];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("edge id in bounds");
            out[u as usize] += 1;
            din[v as usize] += 1;
        }
        (out, din)
    }

    // --- empty / degenerate cases -----------------------------------

    #[test]
    fn zero_vertices_zero_edges_undirected() {
        let g = static_fitness_game(0, &[], None, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn zero_vertices_zero_edges_directed() {
        let in_w: [f64; 0] = [];
        let g = static_fitness_game(0, &[], Some(&in_w), true, true, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn zero_edges_returns_isolated_graph() {
        let f = vec![1.0; 5];
        let g = static_fitness_game(0, &f, None, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_vertex_no_loops_zero_edges_ok() {
        let g = static_fitness_game(0, &[1.0], None, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_vertex_no_loops_with_edges_errors() {
        let err = static_fitness_game(1, &[1.0], None, false, false, 0).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("No edges are possible"), "{msg}");
    }

    #[test]
    fn single_vertex_with_loops_works() {
        let g = static_fitness_game(3, &[1.0], None, true, true, 0).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 3);
    }

    // --- validation -------------------------------------------------

    #[test]
    fn negative_fitness_rejected() {
        let f = [1.0, -0.1, 2.0];
        let err = static_fitness_game(1, &f, None, false, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains("non-negative"));
    }

    #[test]
    fn nan_fitness_rejected() {
        let f = [1.0, f64::NAN, 2.0];
        let err = static_fitness_game(1, &f, None, false, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains("NaN"));
    }

    #[test]
    fn infinite_fitness_rejected() {
        let f = [1.0, f64::INFINITY, 2.0];
        let err = static_fitness_game(1, &f, None, false, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains("finite"));
    }

    #[test]
    fn fitness_in_length_mismatch_rejected() {
        let fo = [1.0, 2.0, 3.0];
        let fi = [1.0, 2.0];
        let err = static_fitness_game(1, &fo, Some(&fi), false, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains("length"));
    }

    #[test]
    fn too_many_simple_edges_rejected() {
        let f = vec![1.0; 4];
        // n=4 undirected simple no-loops ⇒ max 6 edges; ask for 7.
        let err = static_fitness_game(7, &f, None, false, false, 0).unwrap_err();
        assert!(format!("{err:?}").contains("simple edges"));
    }

    #[test]
    fn many_multi_edges_accepted() {
        let f = vec![1.0; 4];
        // With multiple=true the cap does NOT apply.
        let g = static_fitness_game(20, &f, None, false, true, 0).unwrap();
        assert_eq!(g.ecount(), 20);
    }

    // --- structural invariants --------------------------------------

    #[test]
    fn exact_edge_count_simple() {
        let f = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g = static_fitness_game(7, &f, None, false, false, 11).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 7);
        assert!(!g.is_directed());
    }

    #[test]
    fn no_self_loops_when_loops_false() {
        let f = vec![1.0; 10];
        let g = static_fitness_game(15, &f, None, false, true, 12).unwrap();
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (u, v) = g.edge(eid).unwrap();
            assert_ne!(u, v, "self-loop in edge {eid}");
        }
    }

    #[test]
    fn directed_round_trip_preserves_endpoints() {
        let fo = vec![1.0, 2.0, 3.0, 4.0];
        let fi = vec![3.0, 1.0, 2.0, 4.0];
        let g = static_fitness_game(10, &fo, Some(&fi), false, true, 33).unwrap();
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 10);
        // Total out-degree == ecount.
        let (out, din) = directed_in_out(&g, 4);
        assert_eq!(out.iter().sum::<u32>(), 10);
        assert_eq!(din.iter().sum::<u32>(), 10);
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let f = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g1 = static_fitness_game(8, &f, None, false, true, 99).unwrap();
        let g2 = static_fitness_game(8, &f, None, false, true, 99).unwrap();
        let m = u32::try_from(g1.ecount()).unwrap();
        for eid in 0..m {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn determinism_different_seed_differs() {
        let f = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g1 = static_fitness_game(8, &f, None, false, true, 1).unwrap();
        let g2 = static_fitness_game(8, &f, None, false, true, 2).unwrap();
        let mut e1: Vec<_> = (0..u32::try_from(g1.ecount()).unwrap())
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let mut e2: Vec<_> = (0..u32::try_from(g2.ecount()).unwrap())
            .map(|e| g2.edge(e).unwrap())
            .collect();
        e1.sort_unstable();
        e2.sort_unstable();
        assert_ne!(e1, e2);
    }

    #[test]
    fn degree_correlates_with_fitness() {
        // Statistical: with a wide fitness spread, the top-fitness
        // vertex should out-degree the bottom-fitness vertex with high
        // probability over a non-trivial number of edges.
        let n = 50;
        let fitness: Vec<f64> = (0..n).map(|i| 1.0 + (i as f64)).collect();
        let g = static_fitness_game(400, &fitness, None, false, true, 0x00C0_FFEE).unwrap();
        let d = deg(&g, n);
        // Mean of top quartile vs mean of bottom quartile.
        let q = n / 4;
        let bot: f64 = (0..q).map(|i| f64::from(d[i])).sum::<f64>() / (q as f64);
        let top: f64 = (n - q..n).map(|i| f64::from(d[i])).sum::<f64>() / (q as f64);
        assert!(
            top > 2.0 * bot,
            "top-quartile mean degree {top} should dominate bottom {bot}"
        );
    }

    // --- power-law wrapper ------------------------------------------

    #[test]
    fn power_law_basic_undirected() {
        let g = static_power_law_game(100, 30, 2.0, None, true, true, true, 42).unwrap();
        assert_eq!(g.vcount(), 100);
        assert_eq!(g.ecount(), 30);
        assert!(!g.is_directed());
    }

    #[test]
    fn power_law_basic_directed() {
        let g = static_power_law_game(100, 30, 2.0, Some(2.0), true, true, true, 42).unwrap();
        assert_eq!(g.vcount(), 100);
        assert_eq!(g.ecount(), 30);
        assert!(g.is_directed());
    }

    #[test]
    fn power_law_with_only_loops_simple() {
        let g = static_power_law_game(90, 40, 2.0, None, true, false, true, 7).unwrap();
        assert_eq!(g.vcount(), 90);
        assert_eq!(g.ecount(), 40);
    }

    #[test]
    fn power_law_with_only_multi() {
        let g = static_power_law_game(110, 50, 2.0, None, false, true, true, 8).unwrap();
        assert_eq!(g.vcount(), 110);
        assert_eq!(g.ecount(), 50);
    }

    #[test]
    fn power_law_zero_nodes() {
        let g = static_power_law_game(0, 0, 2.0, Some(2.0), false, false, true, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn power_law_zero_edges_undirected() {
        let g = static_power_law_game(10, 0, 2.0, None, false, false, true, 0).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn power_law_infinite_exponent_yields_uniform_fitness() {
        // exponent = ∞ ⇒ α = 0 ⇒ fitness ≡ 1.0 across all vertices ⇒
        // sampling reduces to picking uniformly random vertex pairs
        // (Erdős–Rényi limit).
        let fitness = build_power_law_fitness(20, f64::INFINITY, false);
        for &f in &fitness {
            assert!((f - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn power_law_exponent_too_low_rejected() {
        let err = static_power_law_game(100, 30, 1.5, None, true, true, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains(">= 2"));
    }

    #[test]
    fn power_law_in_exponent_too_low_rejected() {
        let err = static_power_law_game(100, 30, 2.0, Some(0.5), true, true, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains(">= 2"));
    }

    #[test]
    fn power_law_nan_exponent_rejected() {
        let err = static_power_law_game(100, 30, f64::NAN, None, true, true, true, 0).unwrap_err();
        assert!(format!("{err:?}").contains("NaN"));
    }

    // --- internal helpers -------------------------------------------

    #[test]
    fn binsearch_cum_finds_first_ge() {
        let cum = vec![1.0, 3.0, 6.0, 10.0];
        assert_eq!(binsearch_cum(&cum, 0.0), 0);
        assert_eq!(binsearch_cum(&cum, 0.5), 0);
        assert_eq!(binsearch_cum(&cum, 1.0), 0);
        assert_eq!(binsearch_cum(&cum, 1.5), 1);
        assert_eq!(binsearch_cum(&cum, 3.0), 1);
        assert_eq!(binsearch_cum(&cum, 5.5), 2);
        assert_eq!(binsearch_cum(&cum, 10.0), 3);
    }

    #[test]
    fn cumulative_sum_basic() {
        let v = vec![1.0, 2.0, 3.0];
        let c = cumulative_sum(&v);
        assert_eq!(c, vec![1.0, 3.0, 6.0]);
    }

    #[test]
    fn max_edges_undirected_no_loops() {
        let f = vec![1.0; 4];
        let m = max_edges(&f, None, false);
        assert!((m - 6.0).abs() < 1e-12);
    }

    #[test]
    fn max_edges_undirected_loops() {
        let f = vec![1.0; 4];
        let m = max_edges(&f, None, true);
        assert!((m - 10.0).abs() < 1e-12);
    }

    #[test]
    fn max_edges_directed_no_loops() {
        let fo = vec![1.0; 4];
        let fi = vec![1.0; 4];
        let m = max_edges(&fo, Some(&fi), false);
        assert!((m - 12.0).abs() < 1e-12);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            ..ProptestConfig::default()
        })]

        // Invariant: realised edge count always equals the request when
        // the request is feasible.
        #[test]
        fn fitness_exact_edge_count(
            n in 2usize..30,
            seed in any::<u64>(),
            edges in 1u32..20,
            multiple in any::<bool>(),
            loops in any::<bool>(),
        ) {
            let f: Vec<f64> = (0..n).map(|i| 1.0 + i as f64).collect();
            // Cap the request to the simple-graph upper bound when
            // multiple=false to keep the example feasible.
            let cap = if loops {
                (n * (n + 1) / 2) as u32
            } else {
                (n * (n - 1) / 2) as u32
            };
            let m = if multiple { edges } else { edges.min(cap.max(1)) };
            if !multiple && cap == 0 {
                return Ok(());
            }
            let g = static_fitness_game(m, &f, None, loops, multiple, seed).unwrap();
            prop_assert_eq!(g.vcount(), n as u32);
            prop_assert_eq!(g.ecount(), m as usize);
        }

        // Invariant: no self-loops appear when loops=false.
        #[test]
        fn fitness_no_loops_when_disabled(
            n in 3usize..20,
            seed in any::<u64>(),
            edges in 1u32..15,
        ) {
            let f: Vec<f64> = (0..n).map(|i| 1.0 + i as f64).collect();
            let cap = (n * (n - 1) / 2) as u32;
            let m = edges.min(cap);
            let g = static_fitness_game(m, &f, None, false, true, seed).unwrap();
            let me = u32::try_from(g.ecount()).unwrap();
            for eid in 0..me {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        // Invariant: power-law wrapper hits the exact edge count.
        #[test]
        fn power_law_exact_edge_count(
            n in 5u32..60,
            edges in 1u32..40,
            seed in any::<u64>(),
        ) {
            let g = static_power_law_game(n, edges, 2.5, None, true, true, true, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), edges as usize);
        }
    }
}
