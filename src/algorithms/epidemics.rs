//! Epidemics models on graphs.
//!
//! Currently provides the **SIR** (susceptible–infected–recovered)
//! stochastic model, a direct translation of `igraph_sir`
//! (`references/igraph/src/misc/sir.c`).
//!
//! The simulation is a continuous-time Gillespie process. Every
//! individual is in one of three states:
//!
//! * **S** — susceptible (can catch the disease),
//! * **I** — infected (spreads the disease and may recover),
//! * **R** — recovered (immune, inert).
//!
//! A susceptible vertex with `k` infected neighbours becomes infected at
//! rate `k · beta`; an infected vertex recovers at rate `gamma`. Each run
//! starts from a single, uniformly random infected vertex and stops when
//! no infected individuals remain. Event times and the S/I/R population
//! sizes are recorded after every state transition.
//!
//! Determinism: randomness comes from the project's `SplitMix64` PRNG
//! seeded by the caller, so a given `seed` reproduces the same trajectory
//! bit-for-bit. (This means trajectories will *not* coincide with
//! upstream igraph, which uses a different RNG — only the statistical
//! behaviour matches.)

use crate::algorithms::properties::is_simple::{SimpleMode, is_simple_with_mode};
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of a single SIR simulation run.
///
/// All four vectors have the same length: one entry for the initial
/// state plus one entry per recorded state transition.
#[derive(Debug, Clone, PartialEq)]
pub struct Sir {
    /// Cumulative event times. `times[0] == 0.0`; strictly increasing.
    pub times: Vec<f64>,
    /// Number of susceptible individuals at each recorded time.
    pub no_s: Vec<usize>,
    /// Number of infected individuals at each recorded time.
    pub no_i: Vec<usize>,
    /// Number of recovered individuals at each recorded time.
    pub no_r: Vec<usize>,
}

/// Fenwick (binary-indexed) tree holding per-vertex event rates, with
/// O(log n) point update and O(log n) cumulative-rate search.
///
/// Mirrors `igraph_psumtree`: `search(r)` returns the vertex whose rate
/// interval contains the cumulative target `r ∈ [0, total)`.
struct PsumTree {
    n: usize,
    bit: Vec<f64>,
    values: Vec<f64>,
    total: f64,
}

impl PsumTree {
    fn new(n: usize) -> Self {
        Self {
            n,
            bit: vec![0.0; n + 1],
            values: vec![0.0; n],
            total: 0.0,
        }
    }

    fn get(&self, i: usize) -> f64 {
        self.values[i]
    }

    fn total(&self) -> f64 {
        self.total
    }

    fn set(&mut self, i: usize, v: f64) {
        let delta = v - self.values[i];
        self.values[i] = v;
        self.total += delta;
        let mut k = i + 1;
        while k <= self.n {
            self.bit[k] += delta;
            k += k & k.wrapping_neg();
        }
    }

    fn reset(&mut self) {
        for b in &mut self.bit {
            *b = 0.0;
        }
        for v in &mut self.values {
            *v = 0.0;
        }
        self.total = 0.0;
    }

    /// Smallest index whose inclusive prefix sum first exceeds `target`.
    ///
    /// `target` is expected in `[0, total)`. The result is clamped to
    /// `[0, n)` so FP drift in the BIT can never index out of range.
    fn search(&self, target: f64) -> usize {
        if self.n == 0 {
            return 0;
        }
        let mut idx: usize = 0;
        let mut remaining = target;
        let mut step = 1usize;
        while step.saturating_mul(2) <= self.n {
            step *= 2;
        }
        while step > 0 {
            let next = idx + step;
            if next <= self.n && self.bit[next] <= remaining {
                idx = next;
                remaining -= self.bit[next];
            }
            step >>= 1;
        }
        idx.min(self.n - 1)
    }
}

const S_S: u8 = 0;
const S_I: u8 = 1;
const S_R: u8 = 2;

/// Runs `no_sim` independent SIR epidemic simulations on `graph`.
///
/// Edge directions are ignored: an edge contributes to both endpoints'
/// neighbourhoods. The graph must be *simple* in its undirected view
/// (no self-loops, no parallel or mutual edges).
///
/// * `beta` — per-edge infection rate (rate for a susceptible with one
///   infected neighbour); the rate scales linearly with the number of
///   infected neighbours. Must be non-negative.
/// * `gamma` — recovery rate of an infected individual. Must be strictly
///   positive (otherwise the process would never terminate).
/// * `no_sim` — number of independent runs. Must be positive.
/// * `seed` — seed for the deterministic `SplitMix64` PRNG.
///
/// Returns one [`Sir`] trajectory per simulation.
///
/// # Errors
///
/// * The graph is empty (`vcount == 0`).
/// * `beta < 0`, `gamma <= 0`, or `no_sim == 0`.
/// * The graph is not simple in its undirected view.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sir};
///
/// // A small ring; every run starts with exactly one infected vertex.
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
///
/// let runs = sir(&g, 2.0, 1.0, 3, 0x5152).unwrap();
/// assert_eq!(runs.len(), 3);
/// for run in &runs {
///     // Every trajectory starts at t = 0 with one infected, four susceptible.
///     assert_eq!(run.times[0], 0.0);
///     assert_eq!(run.no_i[0], 1);
///     assert_eq!(run.no_s[0], 4);
///     assert_eq!(run.no_r[0], 0);
///     // Population is conserved at every step and ends with no infected.
///     for k in 0..run.times.len() {
///         assert_eq!(run.no_s[k] + run.no_i[k] + run.no_r[k], 5);
///     }
///     assert_eq!(*run.no_i.last().unwrap(), 0);
/// }
/// ```
pub fn sir(
    graph: &Graph,
    beta: f64,
    gamma: f64,
    no_sim: usize,
    seed: u64,
) -> IgraphResult<Vec<Sir>> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "Cannot run SIR model on empty graph.".to_string(),
        ));
    }
    if beta < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "The infection rate beta must be non-negative (got {beta})."
        )));
    }
    if gamma <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "The recovery rate gamma must be positive (got {gamma})."
        )));
    }
    if no_sim == 0 {
        return Err(IgraphError::InvalidArgument(
            "Number of SIR simulations must be positive.".to_string(),
        ));
    }
    if !is_simple_with_mode(graph, SimpleMode::DirectedAsUndirected)? {
        return Err(IgraphError::InvalidArgument(
            "SIR model only works with simple graphs.".to_string(),
        ));
    }

    let adj = build_undirected_adj(graph)?;
    let mut rng = SplitMix64::new(seed);
    let mut tree = PsumTree::new(n);
    let mut status = vec![S_S; n];

    let mut result = Vec::with_capacity(no_sim);
    for _ in 0..no_sim {
        result.push(run_one(
            &adj,
            beta,
            gamma,
            n,
            &mut rng,
            &mut tree,
            &mut status,
        ));
    }
    Ok(result)
}

fn build_undirected_adj(graph: &Graph) -> IgraphResult<Vec<Vec<usize>>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for eid in 0..m {
        let eid_u32 =
            u32::try_from(eid).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
        let (src, tgt) = graph.edge(eid_u32)?;
        // Simple-graph invariant guarantees src != tgt.
        adj[src as usize].push(tgt as usize);
        adj[tgt as usize].push(src as usize);
    }
    Ok(adj)
}

/// One SIR run. `tree` and `status` are reused across runs and reset here.
fn run_one(
    adj: &[Vec<usize>],
    beta: f64,
    gamma: f64,
    n: usize,
    rng: &mut SplitMix64,
    tree: &mut PsumTree,
    status: &mut [u8],
) -> Sir {
    let infected = rng.gen_index(n);

    for s in status.iter_mut() {
        *s = S_S;
    }
    status[infected] = S_I;
    let mut ns = n - 1;
    let mut ni = 1usize;
    let mut nr = 0usize;

    let mut times = vec![0.0_f64];
    let mut no_s = vec![ns];
    let mut no_i = vec![ni];
    let mut no_r = vec![nr];

    tree.reset();
    tree.set(infected, gamma);
    for &nei in &adj[infected] {
        tree.set(nei, beta);
    }

    while ni > 0 {
        let psum = tree.total();
        // Exponential waiting time with rate `psum`: -ln(1-U)/psum.
        // `psum > 0` is guaranteed because at least one infected vertex
        // contributes rate gamma > 0.
        let tt = -(1.0 - rng.gen_unit()).ln() / psum;
        let r = rng.gen_unit() * psum;
        let vchange = tree.search(r);

        if status[vchange] == S_I {
            status[vchange] = S_R;
            ni -= 1;
            nr += 1;
            tree.set(vchange, 0.0);
            for &nei in &adj[vchange] {
                if status[nei] == S_S {
                    let mut rate = tree.get(nei) - beta;
                    if rate < 0.0 {
                        rate = 0.0;
                    }
                    tree.set(nei, rate);
                }
            }
        } else {
            status[vchange] = S_I;
            ns -= 1;
            ni += 1;
            tree.set(vchange, gamma);
            for &nei in &adj[vchange] {
                if status[nei] == S_S {
                    let rate = tree.get(nei) + beta;
                    tree.set(nei, rate);
                }
            }
        }

        let last = *times.last().unwrap_or(&0.0);
        times.push(tt + last);
        no_s.push(ns);
        no_i.push(ni);
        no_r.push(nr);
    }

    Sir {
        times,
        no_s,
        no_i,
        no_r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).unwrap();
        }
        g
    }

    fn complete(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n {
            for j in (i + 1)..n {
                g.add_edge(i, j).unwrap();
            }
        }
        g
    }

    #[test]
    fn empty_graph_errors() {
        let g = Graph::with_vertices(0);
        assert!(sir(&g, 1.0, 1.0, 1, 0).is_err());
    }

    #[test]
    fn parameter_errors() {
        let g = ring(5);
        assert!(sir(&g, -0.1, 1.0, 1, 0).is_err()); // beta < 0
        assert!(sir(&g, 1.0, 0.0, 1, 0).is_err()); // gamma == 0
        assert!(sir(&g, 1.0, -1.0, 1, 0).is_err()); // gamma < 0
        assert!(sir(&g, 1.0, 1.0, 0, 0).is_err()); // no_sim == 0
    }

    #[test]
    fn non_simple_graph_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel edge
        assert!(sir(&g, 1.0, 1.0, 1, 0).is_err());

        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(0, 0).unwrap(); // self-loop
        g2.add_edge(1, 2).unwrap();
        assert!(sir(&g2, 1.0, 1.0, 1, 0).is_err());
    }

    #[test]
    fn produces_requested_number_of_runs() {
        let g = ring(10);
        let runs = sir(&g, 2.0, 1.0, 7, 0xABCD).unwrap();
        assert_eq!(runs.len(), 7);
    }

    #[test]
    fn initial_state_is_consistent() {
        let g = complete(6);
        let runs = sir(&g, 1.0, 1.0, 5, 42).unwrap();
        for run in &runs {
            #[allow(clippy::float_cmp)]
            {
                assert_eq!(run.times[0], 0.0);
            }
            assert_eq!(run.no_i[0], 1);
            assert_eq!(run.no_s[0], 5);
            assert_eq!(run.no_r[0], 0);
        }
    }

    #[test]
    fn population_conserved_and_terminates() {
        let g = complete(8);
        let runs = sir(&g, 3.0, 1.0, 10, 0x1234_5678).unwrap();
        for run in &runs {
            let len = run.times.len();
            assert_eq!(run.no_s.len(), len);
            assert_eq!(run.no_i.len(), len);
            assert_eq!(run.no_r.len(), len);
            for k in 0..len {
                assert_eq!(run.no_s[k] + run.no_i[k] + run.no_r[k], 8);
            }
            // Ends with nobody infected.
            assert_eq!(*run.no_i.last().unwrap(), 0);
            // S is non-increasing, R is non-decreasing.
            for k in 1..len {
                assert!(run.no_s[k] <= run.no_s[k - 1]);
                assert!(run.no_r[k] >= run.no_r[k - 1]);
            }
        }
    }

    #[test]
    fn times_strictly_increasing() {
        let g = complete(7);
        let runs = sir(&g, 2.0, 1.0, 4, 0x9999).unwrap();
        for run in &runs {
            for k in 1..run.times.len() {
                assert!(run.times[k] > run.times[k - 1]);
            }
        }
    }

    #[test]
    fn deterministic_with_seed() {
        let g = complete(6);
        let a = sir(&g, 1.5, 0.7, 5, 0xDEAD_BEEF).unwrap();
        let b = sir(&g, 1.5, 0.7, 5, 0xDEAD_BEEF).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_differ() {
        let g = complete(20);
        let a = sir(&g, 2.0, 0.5, 1, 1).unwrap();
        let b = sir(&g, 2.0, 0.5, 1, 2).unwrap();
        // Overwhelmingly likely to differ in length or trajectory.
        assert!(a != b);
    }

    #[test]
    fn zero_beta_recovers_immediately() {
        // With beta == 0 nobody else is infected: the single initial
        // case just recovers, giving exactly one transition.
        let g = complete(5);
        let runs = sir(&g, 0.0, 1.0, 6, 0x2468).unwrap();
        for run in &runs {
            assert_eq!(run.times.len(), 2);
            assert_eq!(run.no_r.last().copied(), Some(1));
            assert_eq!(run.no_s.last().copied(), Some(4));
        }
    }

    #[test]
    fn directed_graph_ignores_direction() {
        // A directed ring is simple in undirected view; SIR should run.
        let mut g = Graph::new(5, true).unwrap();
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        let runs = sir(&g, 2.0, 1.0, 3, 0x55).unwrap();
        assert_eq!(runs.len(), 3);
        for run in &runs {
            assert_eq!(*run.no_i.last().unwrap(), 0);
        }
    }
}
